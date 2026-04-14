use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::{AppState, auth::CurrentUser, domain::VoucherStatus, error::AppError};

#[derive(FromRow)]
struct VoucherCheckRow {
    unify_id: String,
    unify_create_time: i64,
    code: String,
    duration: i32,
    bill_number: String,
    first_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct VoucherStatusResponse {
    pub unify_id: String,
    pub code: String,
    pub duration: i32,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct VoucherCheckResponse {
    pub data: Vec<VoucherStatusResponse>,
}

#[utoipa::path(
    get,
    path = "/bills/{id}/vouchers/check",
    tag = "Vouchers",
    security(("bearer_auth" = [])),
    params(
        ("id" = i32, Path, description = "Bill ID"),
    ),
    responses(
        (status = 200, description = "Live voucher status from Unify", body = VoucherCheckResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Bill not found or not owned by user"),
    )
)]
pub async fn check_vouchers(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(bill_id): Path<i32>,
) -> Result<Json<VoucherCheckResponse>, AppError> {
    // Verify bill belongs to user and fetch vouchers with note reconstruction data
    let rows = sqlx::query_as::<_, VoucherCheckRow>(
        r#"
        SELECT v.unify_id, v.unify_create_time, v.code, v.duration,
               b.number AS bill_number, u.first_name
        FROM voucher v
        JOIN billjobs_bill b ON b.id = v.bill_id
        JOIN auth_user u ON u.id = b.user_id
        WHERE v.bill_id = $1 AND b.user_id = $2
        "#,
    )
    .bind(bill_id)
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    if rows.is_empty() {
        return Err(AppError::NotFound);
    }

    let unify_ids: Vec<String> = rows.iter().map(|r| r.unify_id.clone()).collect();
    let create_time = rows[0].unify_create_time;
    let note = format!("{}_{}", rows[0].bill_number, rows[0].first_name);

    let statuses = state
        .unify
        .get_vouchers_status(create_time, &note, &unify_ids)
        .await
        .map_err(|e| AppError::Unify(e.to_string()))?;

    let mut data = Vec::with_capacity(rows.len());
    for r in rows {
        let status = statuses
            .get(&r.unify_id)
            .cloned()
            .unwrap_or(VoucherStatus::Unknown);

        sqlx::query("UPDATE voucher SET status = $1 WHERE unify_id = $2")
            .bind(status.as_str())
            .bind(&r.unify_id)
            .execute(&state.db)
            .await?;

        data.push(VoucherStatusResponse {
            unify_id: r.unify_id,
            code: crate::domain::format_code(&r.code),
            duration: r.duration,
            status: status.as_str().to_string(),
        });
    }

    Ok(Json(VoucherCheckResponse { data }))
}
