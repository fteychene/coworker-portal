use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Service {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub voucher_spec: VoucherSpec,
    pub external_service_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind")]
pub enum VoucherSpec {
    Monthly,
    Book { amount: i32, duration: i32 }, // duration in hours
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    pub id: i32,
    pub number: String,
    pub user_id: i32,
    pub service_id: i32,
    pub date: NaiveDate,
    pub amount: f64,
    pub is_paid: bool,
    pub issuer_address: String,
    pub billing_address: String,
    pub vouchers: Vec<Voucher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    pub unify_id: String,
    pub bill_id: i32,
    pub unify_create_time: i64,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub duration: i32, // hours
    pub status: VoucherStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum VoucherStatus {
    Valid,
    Used,
    Expired,
    Unknown,
}

impl VoucherStatus {
    pub fn as_str(&self) -> &str {
        match self {
            VoucherStatus::Valid => "Valid",
            VoucherStatus::Used => "Used",
            VoucherStatus::Expired => "Expired",
            VoucherStatus::Unknown => "Unknown",
        }
    }
}

impl From<&str> for VoucherStatus {
    fn from(s: &str) -> Self {
        match s {
            "Valid" => VoucherStatus::Valid,
            "Used" => VoucherStatus::Used,
            "Expired" => VoucherStatus::Expired,
            _ => VoucherStatus::Unknown,
        }
    }
}

/// Compute the next bill number given the last stored number and today's date.
/// Format: F + YYYYMM + NNN (global counter, not reset per month)
pub fn next_bill_number(last: Option<&str>, today: NaiveDate) -> String {
    let seq = last
        .and_then(|n| n.get(7..))
        .and_then(|s| s.parse::<u32>().ok())
        .map(|n| n + 1)
        .unwrap_or(1);
    format!("F{}{:03}", today.format("%Y%m"), seq)
}

/// Compute voucher duration in hours for a Monthly service:
/// hours from now until 23:59:59 on the last day of the current month.
pub fn monthly_duration_hours(now: DateTime<Utc>) -> i32 {
    use chrono::Datelike;
    let date = now.date_naive();
    let first_of_next = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    };
    let last_day = first_of_next - chrono::Duration::days(1);
    let end = last_day
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();
    let secs = (end - now).num_seconds().max(0);
    ((secs as f64) / 3600.0).ceil() as i32
}

/// Returns (voucher_count, duration_hours) from a VoucherSpec.
pub fn resolve_voucher_params(spec: &VoucherSpec, now: DateTime<Utc>) -> (i32, i32) {
    match spec {
        VoucherSpec::Monthly => (1, monthly_duration_hours(now)),
        VoucherSpec::Book { amount, duration } => (*amount, *duration),
    }
}

/// Format a 10-digit code as XXXXX-XXXXX.
pub fn format_code(code: &str) -> String {
    if code.len() == 10 {
        format!("{}-{}", &code[..5], &code[5..])
    } else {
        code.to_string()
    }
}
