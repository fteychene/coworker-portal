use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

use crate::{config::UnifyConfig, domain::VoucherStatus};
use super::{CreateVouchersRequest, UnifyClient, UnifyVoucher};

pub struct RealUnifyClient {
    client: reqwest::Client,
    base_url: String,
    site: String,
}

impl RealUnifyClient {
    pub async fn new(config: &UnifyConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .build()?;

        let this = Self {
            client,
            base_url: config.base_url.clone(),
            site: config.site.clone(),
        };
        this.login(&config.username, &config.password).await?;
        Ok(this)
    }

    async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.client
            .post(format!("{}/api/login", self.base_url))
            .json(&serde_json::json!({ "username": username, "password": password, "site_name": "default", "for_hotspot": "true" }))
            .send()
            .await?
            .error_for_status()?;
        tracing::info!("Unify login successful");
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct UnifyVoucherDto {
    #[serde(rename = "_id")]
    id: String,
    code: String,
    duration: i32,       // minutes
    status: String,
    status_expires: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct VoucherListResponse {
    data: Vec<UnifyVoucherDto>,
}

#[derive(Deserialize, Debug)]
struct CreateVoucherResponseItem {
    create_time: i64,
}

#[derive(Deserialize)]
struct CreateVoucherResponse {
    data: Vec<CreateVoucherResponseItem>,
}

fn map_status(dto: &UnifyVoucherDto) -> VoucherStatus {
    if dto.status_expires.is_some_and(|e| e <= 0) {
        return VoucherStatus::Expired;
    }
    tracing::info!(voucher_id=&dto.id, voucher_status=&dto.status, "Mapping voucher status");
    match dto.status.to_uppercase().as_str() {
        "VALID_ONE" | "VALID_MULTI" => VoucherStatus::Valid,
        "USED_MULTIPLE" | "EXPIRED" => VoucherStatus::Used,
        _ => VoucherStatus::Unknown,
    }
}

#[async_trait]
impl UnifyClient for RealUnifyClient {
    async fn create_vouchers(&self, req: CreateVouchersRequest) -> Result<Vec<UnifyVoucher>> {
        // Step 1: create the batch, get create_time
        let create_resp: CreateVoucherResponse = self.client
            .post(format!("{}/api/s/{}/cmd/hotspot", self.base_url, self.site))
            .json(&serde_json::json!({
                "cmd": "create-voucher",
                "n": req.n,
                "quota": req.quota,
                "expire_number": req.duration_hours,
                "expire_unit": 60,  // hour multiplier
                "note": req.note,
            }))
            .send().await?
            .error_for_status()?
            .json().await?;

        let create_time = create_resp.data.first()
            .map(|r| r.create_time)
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        // Step 2: retrieve the batch by create_time and filter by note
        let list_resp: VoucherListResponse = self.client
            .post(format!("{}/api/s/{}/stat/voucher", self.base_url, self.site))
            .json(&serde_json::json!({ "create_time": create_time }))
            .send().await?
            .error_for_status()?
            .json().await?;

        let vouchers = list_resp.data.into_iter()
            .filter(|v| v.id.starts_with(&req.note) || true) // note is on the voucher object
            .map(|v| UnifyVoucher {
                unify_id: v.id,
                code: v.code,
                duration: v.duration / 60, // minutes → hours
                create_time,
            })
            .collect();

        Ok(vouchers)
    }

    async fn get_vouchers_status(
        &self,
        create_time: i64,
        _note: &str,
        unify_ids: &[String],
    ) -> Result<HashMap<String, VoucherStatus>> {
        let resp: VoucherListResponse = self.client
            .post(format!("{}/api/s/{}/stat/voucher", self.base_url, self.site))
            .json(&serde_json::json!({ "create_time": create_time }))
            .send().await?
            .error_for_status()?
            .json().await?;

        let id_set: std::collections::HashSet<&str> =
            unify_ids.iter().map(|s| s.as_str()).collect();

        let mut map: HashMap<String, VoucherStatus> = resp.data.into_iter()
            .filter(|v| id_set.contains(v.id.as_str()))
            .map(|v| (v.id.clone(), map_status(&v)))
            .collect();

        // Vouchers absent from the response have been revoked — treat as Expired.
        for id in unify_ids {
            map.entry(id.clone()).or_insert(VoucherStatus::Expired);
        }

        Ok(map)
    }
}
