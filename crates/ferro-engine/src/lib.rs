// SPDX-License-Identifier: GPL-2.0-or-later
//! Engine actor (architecture.md §3).
//!
//! One dedicated OS thread owns the [`PjsuaApp`] and serves commands sent
//! through [`Engine`]. Callers never touch PJSIP directly, so ordering is
//! well defined and no other thread has to register with PJLIB. PJSIP's
//! callbacks arrive on [`EngineEvents`].
//!
//! PJSUA is a process-wide singleton: only one [`Engine`] can run at a time.

use std::sync::mpsc as std_mpsc;
use std::thread::JoinHandle;

use pjsua_rust::{AccountConfig, AccountId, PjsuaApp, PjsuaConfig, TlsConfig, TransportId};
pub use pjsua_rust::{SipEvent, TransportType};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

/// Errors from the engine.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// PJSIP rejected the request.
    #[error("pjsip: {0}")]
    Pj(#[from] pjsua_rust::PjError),
    /// The engine thread has stopped.
    #[error("engine thread is not running")]
    Stopped,
    /// The engine thread could not be started.
    #[error("failed to start engine thread: {0}")]
    Spawn(#[from] std::io::Error),
}

/// Result type for engine calls.
pub type Result<T> = std::result::Result<T, EngineError>;

/// Settings fixed at engine start.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// PJSIP log level, 0–6. PJSIP output is forwarded to `tracing`.
    pub log_level: u32,
    /// Use the null audio device (headless mode, CI).
    pub null_audio: bool,
    /// Conference bridge clock rate in Hz.
    pub clock_rate: u32,
    /// User-Agent header.
    pub user_agent: String,
    /// Nameservers for PJSIP's async resolver (enables SRV/NAPTR, FR-02).
    pub nameservers: Vec<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            log_level: 3,
            null_audio: true,
            clock_rate: 16_000,
            user_agent: format!("Ferrophone/{}", env!("CARGO_PKG_VERSION")),
            nameservers: Vec::new(),
        }
    }
}

/// Stream of events from PJSIP callbacks.
pub type EngineEvents = mpsc::UnboundedReceiver<SipEvent>;

type Reply<T> = oneshot::Sender<Result<T>>;

enum Cmd {
    AddTransport {
        kind: TransportType,
        port: u16,
        tls: Option<TlsConfig>,
        reply: Reply<TransportId>,
    },
    AddAccount {
        config: Box<AccountConfig>,
        reply: Reply<AccountId>,
    },
    RemoveAccount {
        id: AccountId,
        reply: Reply<()>,
    },
    Shutdown,
}

/// Handle to the engine thread. Cheap to clone; dropping the last handle
/// does not stop the engine, [`Engine::shutdown`] does.
#[derive(Debug, Clone)]
pub struct Engine {
    tx: std_mpsc::Sender<Cmd>,
}

/// Owns the engine thread; join it with [`EngineThread::join`] after shutdown.
#[derive(Debug)]
pub struct EngineThread(JoinHandle<()>);

impl EngineThread {
    /// Waits for the engine thread to exit.
    pub fn join(self) {
        let _ = self.0.join();
    }
}

impl std::fmt::Debug for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print account configs: they carry passwords.
        f.write_str(match self {
            Cmd::AddTransport { .. } => "AddTransport",
            Cmd::AddAccount { .. } => "AddAccount",
            Cmd::RemoveAccount { .. } => "RemoveAccount",
            Cmd::Shutdown => "Shutdown",
        })
    }
}

impl Engine {
    /// Starts PJSUA on a dedicated thread and returns once it is initialised.
    pub fn start(config: EngineConfig) -> Result<(Engine, EngineEvents, EngineThread)> {
        let (tx, rx) = std_mpsc::channel::<Cmd>();
        let (ready_tx, ready_rx) = std_mpsc::channel::<Result<EngineEvents>>();

        let handle = std::thread::Builder::new()
            .name("ferro-engine".into())
            .spawn(move || {
                let pj_config = PjsuaConfig {
                    log_level: config.log_level,
                    clock_rate: config.clock_rate,
                    null_audio: config.null_audio,
                    user_agent: Some(config.user_agent),
                    nameservers: config.nameservers,
                };
                let app = match PjsuaApp::new(pj_config) {
                    Ok((app, events)) => {
                        let _ = ready_tx.send(Ok(events));
                        app
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e.into()));
                        return;
                    }
                };
                info!(target: "ferro::sip", "engine started");
                run(&app, rx);
                info!(target: "ferro::sip", "engine stopping");
                drop(app);
            })?;

        let events = ready_rx.recv().map_err(|_| EngineError::Stopped)??;
        Ok((Engine { tx }, events, EngineThread(handle)))
    }

    async fn call<T>(&self, make: impl FnOnce(Reply<T>) -> Cmd) -> Result<T> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(make(reply))
            .map_err(|_| EngineError::Stopped)?;
        rx.await.map_err(|_| EngineError::Stopped)?
    }

    /// Creates a SIP transport. `port` 0 picks an ephemeral port.
    pub async fn add_transport(
        &self,
        kind: TransportType,
        port: u16,
        tls: Option<TlsConfig>,
    ) -> Result<TransportId> {
        self.call(|reply| Cmd::AddTransport {
            kind,
            port,
            tls,
            reply,
        })
        .await
    }

    /// Adds an account; registration starts immediately.
    pub async fn add_account(&self, config: AccountConfig) -> Result<AccountId> {
        self.call(|reply| Cmd::AddAccount {
            config: Box::new(config),
            reply,
        })
        .await
    }

    /// Removes an account, unregistering it first.
    pub async fn remove_account(&self, id: AccountId) -> Result<()> {
        self.call(|reply| Cmd::RemoveAccount { id, reply }).await
    }

    /// Asks the engine thread to destroy PJSUA and exit.
    pub fn shutdown(&self) {
        let _ = self.tx.send(Cmd::Shutdown);
    }
}

fn run(app: &PjsuaApp, rx: std_mpsc::Receiver<Cmd>) {
    while let Ok(cmd) = rx.recv() {
        debug!(target: "ferro::sip", ?cmd, "engine command");
        match cmd {
            Cmd::AddTransport {
                kind,
                port,
                tls,
                reply,
            } => {
                let _ = reply.send(
                    app.create_transport(kind, None, port, tls.as_ref(), None)
                        .map_err(Into::into),
                );
            }
            Cmd::AddAccount { config, reply } => {
                let _ = reply.send(app.add_account(&config).map_err(Into::into));
            }
            Cmd::RemoveAccount { id, reply } => {
                let _ = reply.send(app.remove_account(id).map_err(Into::into));
            }
            Cmd::Shutdown => break,
        }
    }
}

/// Re-exported configuration types, so callers don't depend on pjsua-rust
/// directly (architecture.md §2).
pub mod types {
    pub use pjsua_rust::{
        AccountConfig, AccountId, SrtpMode, TlsConfig, TransportId, TransportType,
    };
}
