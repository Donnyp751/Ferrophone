// SPDX-License-Identifier: GPL-2.0-or-later
//! M0 exit check: register to the containerized Asterisk in tests/interop.
//!
//! ```sh
//! docker compose -f tests/interop/docker-compose.yml up -d
//! FERROPHONE_INTEROP_HOST=127.0.0.1 cargo test -p ferro-engine --features interop \
//!     --test interop_register
//! ```
#![cfg(feature = "interop")]

mod common;

use std::time::Duration;

use ferro_engine::Engine;
use ferro_engine::types::{AccountConfig, TransportType};

#[tokio::test(flavor = "multi_thread")]
async fn registers_udp_and_tcp() -> anyhow::Result<()> {
    common::init_tracing();
    let Ok(host) = std::env::var(ferro_testkit::INTEROP_HOST_ENV) else {
        eprintln!("skipped: {} not set", ferro_testkit::INTEROP_HOST_ENV);
        return Ok(());
    };

    let (engine, mut events, thread) = Engine::start(common::engine_config())?;
    let udp = engine.add_transport(TransportType::Udp, 0, None).await?;
    let tcp = engine.add_transport(TransportType::Tcp, 0, None).await?;

    // Extensions and the shared test secret are defined in tests/interop/asterisk/pjsip.conf.
    let mut ids = Vec::new();
    for (user, transport, tp) in [
        ("1001", TransportType::Udp, udp),
        ("1002", TransportType::Tcp, tcp),
    ] {
        ids.push(
            engine
                .add_account(AccountConfig {
                    name: user.into(),
                    username: user.into(),
                    password: "ferrophone-test".into(),
                    server: host.clone(),
                    port: 5060,
                    transport,
                    transport_id: Some(tp.0),
                    ..Default::default()
                })
                .await?,
        );
    }

    let failed = common::wait_registration(&mut events, &ids, true, Duration::from_secs(10)).await;
    engine.shutdown();
    thread.join();
    assert!(failed.is_empty(), "not registered: {failed:?}");
    Ok(())
}
