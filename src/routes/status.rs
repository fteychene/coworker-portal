use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    /// Whether invoice PDF download is available (superuser Django session is active).
    pub invoice_available: bool,
}

#[utoipa::path(
    get,
    path = "/status",
    tag = "Status",
    responses(
        (status = 200, description = "Application feature availability", body = StatusResponse),
    )
)]
pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let invoice_available = state.superuser_session.read().await.is_some();
    Json(StatusResponse { invoice_available })
}
