pub mod mock;
pub mod real;

use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;

use crate::domain::VoucherStatus;

pub struct CreateVouchersRequest {
    pub n: i32,
    pub duration_hours: i32,
    pub note: String,
    pub quota: i32,
}

pub struct UnifyVoucher {
    pub unify_id: String,
    pub code: String,
    pub duration: i32,       // hours
    pub create_time: i64,    // Unix timestamp
}

#[async_trait]
pub trait UnifyClient: Send + Sync {
    /// Provision vouchers on Unify and return the created vouchers.
    async fn create_vouchers(&self, req: CreateVouchersRequest) -> Result<Vec<UnifyVoucher>>;

    /// Fetch live status for a set of vouchers identified by their Unify IDs.
    /// Uses create_time + note to query Unify, then matches by unify_id.
    async fn get_vouchers_status(
        &self,
        create_time: i64,
        note: &str,
        unify_ids: &[String],
    ) -> Result<HashMap<String, VoucherStatus>>;
}
