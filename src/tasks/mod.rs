mod voucher_sync;

use std::time::Duration;
use crate::AppState;

/// Spawn all background tasks. Returns immediately; tasks run on the Tokio runtime.
pub fn start(state: AppState, voucher_sync_interval_secs: u64) {
    let interval = Duration::from_secs(voucher_sync_interval_secs);
    tracing::info!(interval_secs = voucher_sync_interval_secs, "Scheduler: starting voucher sync task");

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await; // skip the immediate first tick at t=0
        loop {
            ticker.tick().await;
            voucher_sync::run(&state).await;
        }
    });
}
