use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;

use crate::domain::VoucherStatus;
use super::{CreateVouchersRequest, UnifyClient, UnifyVoucher};

pub struct MockUnifyClient;

#[async_trait]
impl UnifyClient for MockUnifyClient {
    async fn create_vouchers(&self, req: CreateVouchersRequest) -> Result<Vec<UnifyVoucher>> {
        let mut rng = rand::thread_rng();
        let create_time = chrono::Utc::now().timestamp();

        let vouchers = (0..req.n)
            .map(|_| {
                let code = format!("{:010}", rng.gen_range(0u64..10_000_000_000u64));
                let unify_id = uuid::Uuid::new_v4().to_string();
                tracing::debug!(note = %req.note, unify_id = %unify_id, "mock: created voucher");
                UnifyVoucher { unify_id, code, duration: req.duration_hours, create_time }
            })
            .collect();

        Ok(vouchers)
    }

    async fn get_vouchers_status(
        &self,
        _create_time: i64,
        _note: &str,
        unify_ids: &[String],
    ) -> Result<HashMap<String, VoucherStatus>> {
        // Mock always returns Valid for all known IDs
        Ok(unify_ids.iter().map(|id| (id.clone(), VoucherStatus::Valid)).collect())
    }
}
