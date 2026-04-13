pub mod jwt;
pub mod password;
pub mod routes;

use axum::{Router, routing::post};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(routes::login))
}
