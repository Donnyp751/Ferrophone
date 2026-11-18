# Decision log

Each decision gives its status, the choice, why, and what it costs. To change a
decision, add a new entry that supersedes it; don't edit the old one.

| ID | Topic | Decision | Status |
| --- | --- | --- | --- |
| D-01 | SIP/media engine | PJSIP 2.17 through `pjsua-rust` | Accepted |
| D-02 | License | GPL-2.0-or-later source; binaries under GPL-3.0 | Accepted |
| D-03 | Process model | Headless daemon `ferrophoned`; UI, CLI and sidecar are clients | Accepted |
| D-04 | Control API | JSON-RPC 2.0 over a local socket, versioned v1 | Accepted |
| D-05 | GUI | Tauri 2 with React and TypeScript | Accepted |
| D-06 | Hosting | GitHub, personal account; GitHub Actions CI | Accepted |
| D-07 | pjsua-rust gaps | Add the missing features upstream in pjsua-rust, not in Ferrophone | Accepted |
| D-08 | UI views | Desk view in MVP; compact view in v1.0 | Accepted |
| D-09 | Scope | Audio only for 1.0; desktop only | Accepted |
| D-10 | PJSIP build | Pinned pjproject 2.17 built by our own script and linked statically | Accepted |
| D-11 | Config and state | TOML config, SQLite for history, OS keyring for secrets | Accepted |

---

## D-01 SIP/media engine: PJSIP via pjsua-rust

**Choice:** wrap PJSIP 2.17 through the `pjsua-rust` crate (Lake Country Embedded, MIT).

**Why:** PJSIP is the most mature option (MicroSIP is built on it) and covers
SIP, codecs, SRTP, ICE, AEC and the conference bridge. `pjsua-rust` already
provides safe bindings, a Tokio event channel and a SIP trace hook, so the
daemon starts from working code.

**Rejected:**

| Option | Reason |
| --- | --- |
| Baresip / libre (BSD-3) | Permissive license, but no existing Rust bindings of ours; we would redo work pjsua-rust already did |
| liblinphone | AGPL/GPLv3 only, heavy C++ SDK with its own threading |
| rsipstack (pure Rust) | Young; media, SRTP, ICE and codecs would all have to come from elsewhere |
| Own Rust stack | Biggest schedule risk |

**Cost:** PJSIP is GPL-2.0-or-later, so every distributed binary is GPL (D-02).
PJSIP's API is callback-heavy with global state, so all PJSIP calls run on
one dedicated thread (architecture.md §3).

## D-02 License: GPL-2.0-or-later

**Choice:** Ferrophone's own source is GPL-2.0-or-later. Every file gets an
SPDX header: `SPDX-License-Identifier: GPL-2.0-or-later`.

**Why:** the shipped binary is GPL anyway because it includes PJSIP, so a
permissive license on our code would add confusion without letting anyone ship
it closed.

**Binary license:** the binary links OpenSSL 3 (Apache-2.0), which is
compatible with GPL-3.0 but not with GPL-2.0. Because both PJSIP and Ferrophone
are "or later", release binaries are distributed under GPL-3.0. This doesn't
affect source contributions.

**Cost:** contributors must accept GPL. `pjsua-rust` stays MIT; that's fine
because MIT code can be included in GPL code.

## D-03 Process model: headless daemon plus clients

**Choice:** `ferrophoned` owns PJSIP, accounts, calls, BLF, MWI, history and
audio. The Tauri app, `ferroctl` and the BLF sidecar are clients of the control
API (D-04). One daemon per user session. The UI starts the daemon if it isn't
running (systemd user unit on Linux, a login item on macOS, a per-user startup
entry on Windows).

**Why:** calls survive a UI crash or reload (NFR-05); the same API serves the UI,
scripts, CRM integrations and CI (FR-29, FR-35); this mirrors Baresip's and
tSIP's proven designs.

**Cost:** the API becomes a public contract from the first release (NFR-13),
and there are two processes to package and supervise.

## D-04 Control API: JSON-RPC 2.0

**Choice:** JSON-RPC 2.0 messages, one per line (UTF-8 JSON separated by
newlines), over:

- Linux/macOS: Unix socket `$XDG_RUNTIME_DIR/ferrophone/control.sock` (macOS:
  `~/Library/Application Support/ferrophone/control.sock`), directory mode
  0700. Peer credentials are checked against the owning user.
- Windows: named pipe `\\.\pipe\ferrophone-<user SID>`, with an ACL for the
  owner only.
- Optional, off by default: TCP on loopback with a bearer token, for test rigs
  that can't reach the socket.

The first call is `system.hello {"api": "1"}`, which agrees on the API version.
Events reach subscribed clients as JSON-RPC notifications (`event.*`). The
schema is written as OpenRPC (`api/openrpc.json`) and generated from the Rust
types, so documentation, TypeScript types and conformance tests all come from
one source. A D-Bus bridge (FR-37) comes in v1.0.

**Why:** trivial to use from shell (`socat`), Python, Node and Rust; no codegen
toolchain is required; it streams events naturally.

**Rejected:** gRPC, because it's heavy for scripts and awkward behind a Tauri
webview. A custom JSON stream, because JSON-RPC gives request ids, errors and
batching for free.

## D-05 GUI: Tauri 2 + React

**Choice:** Tauri 2 with React 18, TypeScript and Vite. The webview never
opens the control socket directly. The Tauri Rust side holds one API
connection and exposes typed commands and an event bridge to the frontend.

**Why:** fast to iterate on a dense, custom desk-phone layout, with one UI
codebase on three OSes.

**Cost:** WebKitGTK on Linux brings memory overhead (hence the 150 MB UI budget
in NFR-03) and some rendering differences between distros. Global hotkeys on
Wayland need the GlobalShortcuts portal.

## D-06 Hosting

**Choice:** a GitHub repository under the maintainer's personal account,
with GitHub Actions for CI and release builds. Transfer to an organization
later if contributors arrive.

**Cost:** `pjsua-rust` and `voip-utility` live under the Lake-Country-Embedded
organization. Ferrophone depends on `pjsua-rust` through a pinned git revision
until it is published on crates.io.

## D-07 Close the pjsua-rust gaps upstream

**Choice:** add the missing features to `pjsua-rust` and submit them as
pull requests there. Ferrophone pins the resulting revision. Missing features:

| Gap | Needed by | PJSUA API |
| --- | --- | --- |
| Attended transfer | FR-07 | `pjsua_call_xfer_replaces` |
| Presence and dialog-event subscriptions | FR-14 | `pjsua_buddy_*` with `subscribe_dlg_event`, `on_buddy_dlg_event_state` (native in 2.17) |
| MWI | FR-15 | `on_mwi_info`, `mwi_enabled` account setting |
| Stream statistics | FR-34 | `pjsua_call_get_stream_stat`, `pjsua_call_get_stream_info` |
| Audio device list and hot-plug | FR-21 | `pjmedia_aud_dev_*`, `pjmedia_aud_dev_refresh`. PJSIP 2.17 has no device-change callback, so the daemon watches PipeWire/udev (Linux), `IMMNotificationClient` (Windows) or Core Audio listeners (macOS), then refreshes |
| Incoming-call headers (Call-Info) | FR-10 | `on_incoming_call` rdata access |

**Why:** these are generic PJSUA features that every pjsua-rust user benefits
from, and keeping all the FFI in one crate keeps Ferrophone free of `unsafe`.

**Cost:** our schedule depends on upstream review. Fallback: carry a branch
`ferrophone/main` of pjsua-rust and rebase it.

## D-08 UI views

**Choice:** the MVP ships the desk view (line keys, BLF grid, dial pad,
call panel, diagnostics drawer). The compact view (FR-36) and the sidecar
window (FR-16) come in v1.0, built from the same components.

## D-09 Scope

**Choice:** audio only through 1.0 (video is FR-31, Later); desktop only; no
mobile, embedded or cloud components.

## D-10 PJSIP build

**Choice:** `scripts/build-pjproject.sh` builds pjproject at a pinned tag (2.17)
with our `config_site.h` and patches (`third_party/pjproject/patches/`,
starting with `mtu-clamp-fix`) into `third_party/prefix/`. `pjsua-rust` finds
it through `PKG_CONFIG_PATH`. Release builds link PJSIP statically. Opus,
SRTP and OpenSSL come from the system on Linux packages, and are bundled in the
Flatpak runtime, AppImage, MSI and DMG.

**Why:** distro PJSIP packages vary in version and build options (video, AEC3,
Opus); pinning keeps behaviour identical across all packages.

## D-11 Config and state

**Choice:**

- Config: `$XDG_CONFIG_HOME/ferrophone/config.toml` (the platform equivalent
  on Windows and macOS): accounts, keys, audio, UI. The daemon owns writes; the UI
  edits through the API. The file is reloaded when edited by hand.
- State: SQLite (`rusqlite`) in `$XDG_DATA_HOME/ferrophone/` for call history and
  contacts.
- Secrets: OS keyring through the `keyring` crate. The config holds only a key
  reference.
