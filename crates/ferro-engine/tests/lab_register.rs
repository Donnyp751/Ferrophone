// SPDX-License-Identifier: GPL-2.0-or-later
//! M0 exit check: register every reserved extension on the lab PBX, then
//! unregister them so the extensions are left free.
//!
//! ```sh
//! FERROPHONE_LAB=tests/lab/lab.toml cargo test -p ferro-engine --features lab \
//!     --test lab_register -- --test-threads=1
//! ```
#![cfg(feature = "lab")]

mod common;

use std::time::Duration;

use ferro_engine::Engine;
use ferro_engine::types::{AccountConfig, SrtpMode, TlsConfig, TransportType};
use ferro_testkit::{LabConfig, Tier};

#[tokio::test(flavor = "multi_thread")]
async fn registers_all_reserved_extensions() -> anyhow::Result<()> {
    common::init_tracing();
    let Some(lab) = LabConfig::from_env() else {
        eprintln!("skipped: {} not set", ferro_testkit::LAB_ENV);
        return Ok(());
    };
    let lab = lab?;
    let tls = lab
        .tls
        .as_ref()
        .expect("lab.toml needs [tls] for the TLS tiers");

    let (engine, mut events, thread) = Engine::start(common::engine_config())?;
    let udp = engine.add_transport(TransportType::Udp, 0, None).await?;
    let tls_tp = engine
        .add_transport(
            TransportType::Tls,
            0,
            Some(TlsConfig {
                ca_list_file: Some(tls.ca_file.display().to_string()),
                cert_file: Some(tls.client_cert_file.display().to_string()),
                privkey_file: Some(tls.client_key_file.display().to_string()),
                verify_server: true,
                ..Default::default()
            }),
        )
        .await?;

    let mut ids = Vec::new();
    for ext in &lab.extensions {
        let server = lab.server_of(ext);
        let (transport, port, transport_id, srtp) = match ext.tier {
            Tier::Open => (TransportType::Udp, server.udp_port, udp, SrtpMode::Disabled),
            Tier::Medium => (
                TransportType::Tls,
                server.mtls_port,
                tls_tp,
                SrtpMode::Disabled,
            ),
            Tier::Strict => (
                TransportType::Tls,
                server.mtls_port,
                tls_tp,
                SrtpMode::Mandatory,
            ),
        };
        let id = engine
            .add_account(AccountConfig {
                name: format!("lab-{}", ext.user),
                username: ext.user.clone(),
                password: ext.password.clone(),
                server: server.host.clone(),
                port,
                transport,
                transport_id: Some(transport_id.0),
                srtp,
                reg_timeout: Some(120),
                ..Default::default()
            })
            .await?;
        ids.push((id, ext.user.clone()));
    }

    let all: Vec<_> = ids.iter().map(|(id, _)| *id).collect();
    let failed = common::wait_registration(&mut events, &all, true, Duration::from_secs(20)).await;

    // Always unregister, pass or fail, so the shared PBX is left clean.
    // pjsua_acc_del sends REGISTER with Expires: 0 but destroys the account
    // before the reply, so no registration event follows; give the
    // transactions a moment to complete before PJSUA is torn down.
    for (id, _) in &ids {
        let _ = engine.remove_account(*id).await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    engine.shutdown();
    thread.join();

    let named: Vec<_> = failed
        .iter()
        .map(|(id, why)| {
            let user = ids
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, u)| u.as_str())
                .unwrap_or("?");
            format!("{user}: {why}")
        })
        .collect();
    assert!(named.is_empty(), "not registered: {named:?}");
    Ok(())
}
