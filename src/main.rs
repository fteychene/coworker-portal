use anyhow::Result;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;

mod auth;
mod config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt: std::sync::Arc<auth::jwt::JwtService>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let state = AppState {
        db,
        jwt: std::sync::Arc::new(auth::jwt::JwtService::new(
            &config.jwt_secret,
            config.jwt_expiry_hours,
        )),
    };

    let app = Router::new()
        .nest("/api/auth", auth::router())
        .fallback_service(ServeDir::new("public"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    println!("Listening on http://{}", config.listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
