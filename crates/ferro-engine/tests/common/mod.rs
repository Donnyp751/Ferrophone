// SPDX-License-Identifier: GPL-2.0-or-later
//! Shared helpers for engine integration tests.
#![allow(dead_code)]

use std::collections::HashSet;
use std::time::Duration;

use ferro_engine::types::AccountId;
use ferro_engine::{EngineConfig, EngineEvents, SipEvent};

/// Engine config for tests. `FERROPHONE_PJSIP_LOG=5` shows full SIP messages.
pub fn engine_config() -> EngineConfig {
    let log_level = std::env::var("FERROPHONE_PJSIP_LOG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    EngineConfig {
        log_level,
        ..Default::default()
    }
}

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_test_writer()
        .try_init();
}

/// Waits until every account in `want` reports `is_registered == registered`.
/// Returns the accounts that did not get there before `timeout`, with their
/// last SIP status.
pub async fn wait_registration(
    events: &mut EngineEvents,
    want: &[AccountId],
    registered: bool,
    timeout: Duration,
) -> Vec<(AccountId, String)> {
    let mut pending: HashSet<AccountId> = want.iter().copied().collect();
    let mut last: std::collections::HashMap<AccountId, String> = Default::default();
    let deadline = tokio::time::Instant::now() + timeout;
    while !pending.is_empty() {
        let ev = match tokio::time::timeout_at(deadline, events.recv()).await {
            Ok(Some(ev)) => ev,
            _ => break,
        };
        if let SipEvent::RegistrationState {
            account_id,
            code,
            reason,
            is_registered,
            ..
        } = ev
        {
            last.insert(account_id, format!("{code} {reason}"));
            if is_registered == registered {
                pending.remove(&account_id);
            }
        }
    }
    pending
        .into_iter()
        .map(|id| {
            (
                id,
                last.get(&id)
                    .cloned()
                    .unwrap_or_else(|| "no response".into()),
            )
        })
        .collect()
}
