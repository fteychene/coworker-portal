use anyhow::Result;
use axum::{Json, response::Html, routing::get};
use axum_swagger_ui::swagger_ui;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

mod auth;
mod config;
mod domain;
mod error;
mod openapi;
mod routes;
mod unify;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt: Arc<auth::jwt::JwtService>,
    pub unify: Arc<dyn unify::UnifyClient>,
    pub config: Arc<config::Config>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "coworking_tooling=debug,info".parse().unwrap()),
        )
        .init();

    let config = config::Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;
    tracing::info!("Migrations applied");

    let unify_client: Arc<dyn unify::UnifyClient> = match config.unify.mode {
        config::UnifyMode::Mock => {
            tracing::info!("Unify: using mock client");
            Arc::new(unify::mock::MockUnifyClient)
        }
        config::UnifyMode::Real => {
            tracing::info!("Unify: connecting to {}", config.unify.base_url);
            Arc::new(unify::real::RealUnifyClient::new(&config.unify).await?)
        }
    };

    let state = AppState {
        db,
        jwt: Arc::new(auth::jwt::JwtService::new(
            &config.jwt_secret,
            config.jwt_expiry_hours,
        )),
        unify: unify_client,
        config: Arc::new(config.clone()),
    };

    let (router, api) = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .nest("/api/auth", auth::router())
        .nest("/api", routes::router())
        .split_for_parts();

    let app = router
        .route("/swagger", get(|| async { Html(swagger_ui("/api-docs/openapi.json"))}))
        .route("/api-docs/openapi.json", get(|| async move { Json(api) }))
        .fallback_service(ServeDir::new("public").fallback(ServeFile::new("public/index.html")))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("Listening on http://{}", config.listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
