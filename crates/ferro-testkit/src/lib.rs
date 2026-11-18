// SPDX-License-Identifier: GPL-2.0-or-later
//! Test environment configuration.
//!
//! The lab config lives outside git (`tests/lab/lab.toml`, see
//! `tests/lab/lab.toml.template` and docs/testing.md). Tests find it through
//! the `FERROPHONE_LAB` environment variable and skip when it isn't set.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Environment variable that points at the lab config file.
pub const LAB_ENV: &str = "FERROPHONE_LAB";
/// Environment variable with the containerized PBX's address (tests/interop).
pub const INTEROP_HOST_ENV: &str = "FERROPHONE_INTEROP_HOST";

/// Errors loading a test config.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read.
    #[error("reading {path}: {source}")]
    Read {
        /// File that failed.
        path: PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// The file is not valid lab TOML.
    #[error("parsing {path}: {source}")]
    Parse {
        /// File that failed.
        path: PathBuf,
        /// Underlying error.
        source: Box<toml::de::Error>,
    },
    /// An extension names a server that isn't defined.
    #[error("extension {user} refers to unknown server {server:?}")]
    UnknownServer {
        /// Extension user part.
        user: String,
        /// Server name it referenced.
        server: String,
    },
}

/// The lab PBX environment.
#[derive(Debug, Clone, Deserialize)]
pub struct LabConfig {
    /// PBX servers by name (`primary`, `alternate`).
    pub servers: BTreeMap<String, Server>,
    /// DNS settings for SRV failover tests.
    pub dns: Option<Dns>,
    /// TLS material for TLS and mutual-TLS tiers.
    pub tls: Option<Tls>,
    /// Extensions reserved for Ferrophone.
    pub extensions: Vec<Extension>,
}

/// One PBX.
#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    /// Host name or address.
    pub host: String,
    /// SIP over UDP and TCP.
    pub udp_port: u16,
    /// SIP over TLS, server authentication only.
    pub tls_port: u16,
    /// SIP over mutual TLS.
    pub mtls_port: u16,
}

/// DNS settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Dns {
    /// Resolver to use instead of the system one.
    pub resolver: String,
    /// Domain with SRV records pointing at primary then alternate.
    pub srv_domain: String,
    /// Domain with an A record and no SRV.
    pub nosrv_domain: String,
}

/// TLS file paths.
#[derive(Debug, Clone, Deserialize)]
pub struct Tls {
    /// CA certificate that signed the PBX certificates.
    pub ca_file: PathBuf,
    /// Client certificate for mutual TLS.
    pub client_cert_file: PathBuf,
    /// Client private key for mutual TLS.
    pub client_key_file: PathBuf,
}

/// Security tier of a lab extension (docs/testing.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Any transport, plain RTP.
    Open,
    /// Mutual TLS, plain RTP.
    Medium,
    /// Mutual TLS, SRTP mandatory.
    Strict,
}

/// A reserved extension.
#[derive(Clone, Deserialize)]
pub struct Extension {
    /// User part, e.g. `6025`.
    pub user: String,
    /// SIP digest password.
    pub password: String,
    /// Name of the server in [`LabConfig::servers`].
    pub server: String,
    /// Security tier.
    pub tier: Tier,
}

impl fmt::Debug for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Keep passwords out of test output.
        f.debug_struct("Extension")
            .field("user", &self.user)
            .field("server", &self.server)
            .field("tier", &self.tier)
            .finish_non_exhaustive()
    }
}

impl LabConfig {
    /// Loads the lab config named by `FERROPHONE_LAB`, or `None` when unset.
    ///
    /// A relative path is tried as given, then against the workspace root,
    /// because `cargo test` runs each crate's tests from that crate's directory.
    pub fn from_env() -> Option<Result<Self, ConfigError>> {
        std::env::var_os(LAB_ENV).map(|p| Self::load(&resolve(Path::new(&p))))
    }

    /// Loads and checks a lab config file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_owned(),
            source,
        })?;
        let cfg: LabConfig = toml::from_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source: Box::new(source),
        })?;
        for ext in &cfg.extensions {
            if !cfg.servers.contains_key(&ext.server) {
                return Err(ConfigError::UnknownServer {
                    user: ext.user.clone(),
                    server: ext.server.clone(),
                });
            }
        }
        Ok(cfg)
    }

    /// The server an extension registers to.
    pub fn server_of(&self, ext: &Extension) -> &Server {
        &self.servers[&ext.server]
    }

    /// Extensions of one tier.
    pub fn tier(&self, tier: Tier) -> impl Iterator<Item = &Extension> {
        self.extensions.iter().filter(move |e| e.tier == tier)
    }
}

fn resolve(path: &Path) -> PathBuf {
    if path.is_absolute() || path.exists() {
        return path.to_owned();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidate = root.join(path);
    if candidate.exists() {
        candidate
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/lab/lab.toml.template");
        let cfg = LabConfig::load(&path).expect("template must stay valid");
        assert_eq!(cfg.extensions.len(), 1);
        assert_eq!(cfg.extensions[0].tier, Tier::Open);
    }

    #[test]
    fn debug_hides_password() {
        let ext = Extension {
            user: "6025".into(),
            password: "hunter2".into(),
            server: "primary".into(),
            tier: Tier::Open,
        };
        assert!(!format!("{ext:?}").contains("hunter2"));
    }
}
