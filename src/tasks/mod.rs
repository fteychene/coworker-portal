mod voucher_sync;

use chrono_tz::Europe::Paris;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::AppState;

/// Spawn all background tasks. Returns immediately; tasks run on the Tokio runtime.
pub async fn start(state: AppState, voucher_sync_cron: &str) -> anyhow::Result<()> {
    // Validate the cron expression early so a bad config fails at startup.
    voucher_sync_cron.parse::<croner::Cron>()
        .map_err(|e| anyhow::anyhow!("Invalid VOUCHER_SYNC_CRON expression {:?}: {}", voucher_sync_cron, e))?;

    let scheduler = JobScheduler::new().await?;

    let voucher_state = state.clone();
    let cron_expr = voucher_sync_cron.to_string();
    scheduler
        .add(Job::new_async_tz(&cron_expr, Paris, move |_id, _sched| {
            let s = voucher_state.clone();
            Box::pin(async move {
                voucher_sync::run(&s).await;
            })
        })?)
        .await?;

    scheduler.start().await?;
    tracing::info!(
        cron = voucher_sync_cron,
        timezone = "Europe/Paris",
        "Scheduler started — voucher sync scheduled"
    );
    Ok(())
}
