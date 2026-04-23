mod monthly_usage;
mod voucher_sync;

use chrono_tz::Europe::Paris;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::AppState;

/// Spawn all background tasks. Returns immediately; tasks run on the Tokio runtime.
pub async fn start(state: AppState, voucher_sync_cron: &str, monthly_usage_cron: &str) -> anyhow::Result<()> {
    // Validate both expressions early so a bad config fails at startup.
    voucher_sync_cron.parse::<croner::Cron>()
        .map_err(|e| anyhow::anyhow!("Invalid VOUCHER_SYNC_CRON {:?}: {}", voucher_sync_cron, e))?;
    monthly_usage_cron.parse::<croner::Cron>()
        .map_err(|e| anyhow::anyhow!("Invalid MONTHLY_USAGE_CRON {:?}: {}", monthly_usage_cron, e))?;

    let scheduler = JobScheduler::new().await?;

    // Voucher sync: Mon–Fri, every hour 09:00–19:00 (Paris).
    let s = state.clone();
    let cron = voucher_sync_cron.to_string();
    scheduler.add(Job::new_async_tz(&cron, Paris, move |_id, _sched| {
        let s = s.clone();
        Box::pin(async move { voucher_sync::run(&s).await })
    })?).await?;

    // Monthly usage diary: every day at 23:00 (Paris).
    let s = state.clone();
    let cron = monthly_usage_cron.to_string();
    scheduler.add(Job::new_async_tz(&cron, Paris, move |_id, _sched| {
        let s = s.clone();
        Box::pin(async move { monthly_usage::run(&s).await })
    })?).await?;

    scheduler.start().await?;
    tracing::info!(
        voucher_sync_cron,
        monthly_usage_cron,
        timezone = "Europe/Paris",
        "Scheduler started"
    );
    Ok(())
}