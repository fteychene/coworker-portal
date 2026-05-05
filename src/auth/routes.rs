use axum::{Json, extract::State, http::StatusCode};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::AppState;
use super::password::{hash_django_password, verify_django_password};

type ApiResult<T> = Result<T, (StatusCode, Json<serde_json::Value>)>;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(FromRow)]
struct AuthUser {
    id: i32,
    password: String,
    first_name: String,
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "JWT token issued", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    tracing::info!(username = &body.username, "login");
    let user = sqlx::query_as::<_, AuthUser>(
        "SELECT id, password, first_name FROM auth_user WHERE username = $1 AND is_active = true",
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
        .generate(user.id, &body.username, &user.first_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse { token }))
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(FromRow)]
struct ResetTokenRow {
    user_id: i32,
    expires_at: chrono::DateTime<chrono::Utc>,
    used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[utoipa::path(
    post,
    path = "/forgot-password",
    tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset link sent if email exists"),
        (status = 503, description = "SMTP not configured"),
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let smtp = state.config.smtp.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({"error": "Réinitialisation par email non configurée"})),
    ))?;

    let user_id: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM auth_user WHERE email = $1 AND is_active = true",
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))))?;

    if let Some(uid) = user_id {
        let _ = sqlx::query(
            "DELETE FROM portal_password_reset_tokens WHERE user_id = $1 AND used_at IS NULL",
        )
        .bind(uid)
        .execute(&state.db)
        .await;

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        sqlx::query(
            "INSERT INTO portal_password_reset_tokens (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '30 minutes')",
        )
        .bind(uid)
        .bind(&token)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))))?;

        let reset_link = format!("{}/reset-password?token={}", state.config.app_base_url, token);
        if let Err(e) = send_reset_email(smtp, &body.email, &reset_link).await {
            tracing::error!(error = %e, "Failed to send reset email");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur lors de l'envoi de l'email"}))));
        }
    }

    Ok(Json(json!({})))
}

#[utoipa::path(
    post,
    path = "/reset-password",
    tag = "Auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 400, description = "Invalid, expired, or already-used token"),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let row = sqlx::query_as::<_, ResetTokenRow>(
        "SELECT user_id, expires_at, used_at FROM portal_password_reset_tokens WHERE token = $1",
    )
    .bind(&body.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))))?
    .ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": "Token invalide"}))))?;

    if row.used_at.is_some() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Token déjà utilisé"}))));
    }
    if row.expires_at < chrono::Utc::now() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Token expiré"}))));
    }
    if body.new_password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Le mot de passe doit contenir au moins 8 caractères"}))));
    }

    let hashed = hash_django_password(&body.new_password);

    sqlx::query("UPDATE auth_user SET password = $1 WHERE id = $2")
        .bind(&hashed)
        .bind(row.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))))?;

    sqlx::query("UPDATE portal_password_reset_tokens SET used_at = NOW() WHERE token = $1")
        .bind(&body.token)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Erreur serveur"}))))?;

    Ok(Json(json!({})))
}

async fn send_reset_email(
    smtp: &crate::config::SmtpConfig,
    to_email: &str,
    reset_link: &str,
) -> anyhow::Result<()> {
    let email = Message::builder()
        .from(smtp.from_email.parse()?)
        .to(to_email.parse()?)
        .subject("Réinitialisation de votre mot de passe")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Bonjour,\n\nCliquez sur le lien suivant pour réinitialiser votre mot de passe :\n{reset_link}\n\nCe lien expire dans 30 minutes.\n\nSi vous n'avez pas fait cette demande, ignorez cet email."
        ))?;

    let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());
    let mailer = if smtp.port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)?
    }
    .port(smtp.port)
    .credentials(creds)
    .build();

    mailer.send(email).await?;
    Ok(())
}

/// Authenticates against Django's login page and returns the `sessionid` cookie value.
pub async fn acquire_django_session(
    base_url: &str,
    accept_invalid_certs: bool,
    username: &str,
    password: &str,
) -> anyhow::Result<String> {
    tracing::info!(base_url, username, accept_invalid_certs, "Django: starting session acquisition");

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()?;

    let login_url = format!("{}/admin/login/", base_url);
    tracing::debug!(url = %login_url, "Django: GET login page");

    let get_res = client.get(&login_url).send().await
        .inspect_err(|e| tracing::error!(error = %e, url = %login_url, "Django: GET login page failed"))?;

    tracing::debug!(status = %get_res.status(), "Django: GET login page response");
    tracing::debug!(
        set_cookie = ?get_res.headers().get_all(reqwest::header::SET_COOKIE).iter().collect::<Vec<_>>(),
        "Django: GET Set-Cookie headers"
    );

    let csrf = extract_cookie(get_res.headers(), "csrftoken")
        .unwrap_or_default();
    tracing::debug!(csrf_found = !csrf.is_empty(), "Django: CSRF token extracted");

    if csrf.is_empty() {
        tracing::warn!("Django: csrftoken not found in GET response — POST may be rejected");
    }

    tracing::debug!(url = %login_url, username, "Django: POST credentials");

    let post_res: reqwest::Response = client
        .post(&login_url)
        .header("Cookie", format!("csrftoken={}", csrf))
        .header("Referer", &login_url)
        .form(&[
            ("username", username),
            ("password", password),
            ("csrfmiddlewaretoken", csrf.as_str()),
        ])
        .send()
        .await
        .inspect_err(|e| tracing::error!(error = %e, "Django: POST credentials failed"))?;

    tracing::debug!(status = %post_res.status(), "Django: POST response");
    tracing::debug!(
        set_cookie = ?post_res.headers().get_all(reqwest::header::SET_COOKIE).iter().collect::<Vec<_>>(),
        "Django: POST Set-Cookie headers"
    );

    let session = extract_cookie(post_res.headers(), "sessionid");
    match &session {
        Some(_) => tracing::info!(username, "Django: session acquired successfully"),
        None => tracing::warn!(
            username,
            post_status = %post_res.status(),
            "Django: sessionid not found in POST response — credentials may be wrong or Django rejected the login"
        ),
    }

    session.ok_or_else(|| anyhow::anyhow!("sessionid not found in Django response"))
}

fn extract_cookie(headers: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
    let prefix = format!("{}=", name);
    headers
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|hv| {
            hv.to_str().ok().and_then(|s| {
                s.split(';')
                    .next()
                    .and_then(|part| part.trim().strip_prefix(&prefix).map(str::to_string))
            })
        })
}
