// SPDX-License-Identifier: GPL-2.0-or-later
//! `ferrophoned`: the Ferrophone daemon (decision D-03).
//!
//! M0 only starts the engine and sets up logging; the API server arrives in M1.

use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Default log filter. Subsystem targets: `ferro::sip`, `ferro::media`,
/// `ferro::api`; PJSIP's own output is logged by pjsua-rust (`pjsua_rust`).
const DEFAULT_LOG: &str = "info,pjsua_rust=warn";

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Run with no UI clients expected (CI, test rigs). Implies --null-audio.
    #[arg(long)]
    headless: bool,
    /// Use the null audio device.
    #[arg(long)]
    null_audio: bool,
    /// Log filter, e.g. `ferro::sip=debug,info`. Overrides FERROPHONE_LOG.
    #[arg(long)]
    log: Option<String>,
    /// PJSIP log level, 0-6.
    #[arg(long, default_value_t = 3)]
    pjsip_log_level: u32,
}

fn init_logging(filter: Option<&str>) {
    let filter = filter
        .map(EnvFilter::new)
        .or_else(|| EnvFilter::try_from_env("FERROPHONE_LOG").ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_LOG));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_logging(args.log.as_deref());

    let config = ferro_engine::EngineConfig {
        log_level: args.pjsip_log_level,
        null_audio: args.headless || args.null_audio,
        ..Default::default()
    };
    let (engine, mut events, thread) = ferro_engine::Engine::start(config)?;
    tracing::info!(target: "ferro::api", "ferrophoned {} ready", env!("CARGO_PKG_VERSION"));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            ev = events.recv() => match ev {
                Some(ev) => tracing::debug!(target: "ferro::sip", ?ev, "event"),
                None => break,
            },
        }
    }
    engine.shutdown();
    thread.join();
    Ok(())
}
