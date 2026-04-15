use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::{AppState, auth::CurrentUser, domain::{Service, VoucherSpec}, error::AppError};

#[derive(FromRow)]
struct ServiceRow {
    id: i32,
    name: String,
    description: String,
    price: f64,
    kind: String,
    amount: Option<i32>,
    duration: Option<i32>,
    external_service_id: i32,
}

impl TryFrom<ServiceRow> for Service {
    type Error = AppError;

    fn try_from(row: ServiceRow) -> Result<Self, Self::Error> {
        let voucher_spec = match row.kind.as_str() {
            "Monthly" => VoucherSpec::Monthly,
            "Book" => VoucherSpec::Book {
                amount: row.amount.unwrap_or(1),
                duration: row.duration.unwrap_or(1),
            },
            _ => return Err(AppError::NotFound),
        };
        Ok(Service {
            id: row.id,
            name: row.name,
            description: row.description,
            price: row.price,
            voucher_spec,
            external_service_id: row.external_service_id,
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct ServicesResponse {
    pub data: Vec<Service>,
}

#[utoipa::path(
    get,
    path = "/services",
    tag = "Services",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of available services", body = ServicesResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_services(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> Result<Json<ServicesResponse>, AppError> {
    let rows = sqlx::query_as::<_, ServiceRow>(
        r#"
        SELECT id, name, description, price, kind, amount, duration, external_service_id
        FROM portal_service
        WHERE is_available = true
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let data = rows
        .into_iter()
        .map(Service::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ServicesResponse { data }))
}
