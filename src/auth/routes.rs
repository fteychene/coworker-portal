use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::AppState;
use super::password::verify_django_password;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(FromRow)]
struct AuthUser {
    id: i32,
    password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query_as::<_, AuthUser>(
        "SELECT id, password FROM auth_user WHERE username = $1 AND is_active = true",
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_django_password(&body.password, &user.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = state
        .jwt
        .generate(user.id, &body.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse { token }))
}
