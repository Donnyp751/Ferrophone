# Architecture

How Ferrophone is put together. The reasons are in [decisions.md](decisions.md).

## 1. Processes

```
 ┌───────────────────────┐   ┌──────────────┐   ┌──────────────┐   ┌────────────────┐
 │ Ferrophone app (Tauri)│   │  ferroctl    │   │ scripts, CRM │   │ CI test runner │
 │ React UI ⇄ Rust shell │   │  (CLI)       │   │              │   │ (+ test hooks) │
 └──────────┬────────────┘   └──────┬───────┘   └──────┬───────┘   └───────┬────────┘
            │      JSON-RPC 2.0, one message per line, over a Unix socket / named pipe
            └────────────────┬──────┴──────────────────┴───────────────────┘
                     ┌───────┴───────────────────────────────────────────┐
                     │ ferrophoned                                       │
                     │  api server ── service layer ── state store       │
                     │                    │                │             │
                     │              engine actor      config / SQLite /  │
                     │           (dedicated thread)   keyring            │
                     │                    │                              │
                     │               pjsua-rust ── PJSIP 2.17 (static)   │
                     └────────────────────┼──────────────────────────────┘
                                 SIP / RTP│ audio devices
                                          ▼
                           PBX, trunks │ PipeWire / WASAPI / Core Audio
```

- **ferrophoned:** one per user session. It holds all phone state. With no
  clients connected it keeps registering and taking calls: it rings, notifies,
  and logs the missed call.
- **Ferrophone app:** a Tauri 2 window plus tray icon. Its Rust side holds one API
  connection and passes typed commands and events to React. If the daemon dies,
  the app shows "reconnecting" and restarts it.
- **ferroctl:** a thin CLI over the same API (`ferroctl dial`, `answer`,
  `status`, `events --follow`, `raw <method> <json>`). It is also the URI
  handler target for `sip:`, `tel:` and `callto:` links.

## 2. Repository layout

```
ferrophone/
├── Cargo.toml                  workspace
├── crates/
│   ├── ferro-api/              API types (serde), method and event enums, OpenRPC generation
│   ├── ferro-engine/           safe actor wrapper over pjsua-rust; the only crate that touches PJSIP
│   ├── ferro-core/             domain logic: accounts, lines, keys, BLF, MWI, history, config
│   ├── ferro-audio/            device watching, hot-plug, test-hook analysis (FFT, tone, level)
│   ├── ferro-client/           async Rust client for the API (used by app, ferroctl, tests)
│   ├── ferro-testkit/          test environment config: lab.toml loader, interop settings
│   ├── ferrophoned/            daemon binary: API server, service wiring, platform glue
│   └── ferroctl/               CLI binary
├── app/                        Tauri 2 app
│   ├── src-tauri/              Rust shell (uses ferro-client)
│   └── src/                    React + TypeScript UI (types generated from ferro-api)
├── api/openrpc.json            generated, checked in, diffed in CI
├── tests/
│   ├── interop/                docker-compose: Asterisk 22, FreeSWITCH 1.10, SIPp, netem NAT
│   └── scenarios/              Rust integration tests driving ferrophoned through ferro-client
├── third_party/pjproject/      config_site.h, patches/
├── scripts/                    build-pjproject.sh, release helpers
├── packaging/                  flatpak/, appimage/, deb/, rpm/, windows/, macos/, systemd/
└── docs/
```

Dependency direction: `ferrophoned → ferro-core → ferro-engine → pjsua-rust`.
`ferro-api` depends on nothing internal. Clients depend only on
`ferro-api` and `ferro-client`. `unsafe` is forbidden in every crate except
`ferro-engine` and the platform glue in `ferro-audio`.

## 3. Threading and the engine actor

PJSIP keeps global state and is callback-driven, and its threads must be
registered with it. So:

- `ferro-engine` starts **one OS thread** that owns the `PjsuaApp`. Every call
  into PJSIP goes through a command channel (`EngineCmd` + a oneshot reply).
  PJSIP callbacks arrive on the pjsua-rust Tokio event channel and are
  turned into typed `EngineEvent`s.
- `ferro-core` runs on a multi-threaded Tokio runtime. It turns engine events
  into domain state (`Line`, `Call`, `BlfKey`, `Mwi`) in a single state store
  and publishes changes on a broadcast channel.
- The API server is fed by that broadcast channel. Each client connection is
  a task with its own event subscription filter. A slow client is dropped,
  not allowed to block the daemon (bounded queue, 1024 events).

Domain IDs are stable UUIDs (`acc_…`, `call_…`, `key_…`), never PJSIP's
integer IDs, which get reused.

## 4. Domain model (core types)

| Type | Key fields |
| --- | --- |
| `Account` | id, display name, SIP URI, transport, registrar, outbound proxy, auth user, secret ref, codecs, SRTP mode, DTMF mode, BLF mode default (dialog / presence), voicemail number, enabled, registration state |
| `Line` | index 1..N, state (idle, ringing, active, held, …), current `Call` |
| `Call` | id, account, line, direction, remote URI and name, state, media state, start and answer times, held, muted, recording, transfer state, stats |
| `Key` | id, page, position, label, kind (line, speed_dial, blf, park, dtmf_macro, url) plus kind-specific fields |
| `BlfState` | target URI, mode, lamp (idle, ringing, busy, unknown), last NOTIFY time |
| `Mwi` | account, new and old message counts |
| `HistoryEntry` | id, account, direction, remote, start time, duration, outcome |

Call states as seen by clients: `dialing → ringing_out → early_media → active ⇄ held → ended`,
or `incoming → active | missed | rejected`, plus `transferring` while a
REFER is pending. `ended` carries the SIP status and reason.

## 5. Control API (v1 summary)

Full spec: [control-api.md](control-api.md). The schema is `api/openrpc.json`.

| Namespace | Methods |
| --- | --- |
| `system` | `hello`, `version`, `shutdown`, `log.setLevel` |
| `account` | `list`, `get`, `add`, `update`, `remove`, `setEnabled`, `register` |
| `call` | `list`, `get`, `dial`, `answer`, `reject`, `hangup`, `hold`, `resume`, `mute`, `dtmf`, `transfer.blind`, `transfer.attended`, `stats` |
| `key` | `list`, `set`, `remove`, `press` |
| `blf` | `list` |
| `mwi` | `get` |
| `audio` | `devices`, `setDevices`, `setLevels` |
| `history` | `list`, `clear` |
| `trace` | `subscribe`, `export` (text, pcap) |
| `events` | `subscribe {topics}`, `unsubscribe` |
| `test` | `injectWav`, `captureStart`, `captureStop`, `detectTone`, `expectDtmf`, `level` (only with `--test-hooks`) |

Event topics: `account.*`, `call.*`, `line.*`, `blf.*`, `mwi.*`, `audio.*`,
`history.*`, `trace.message`, `stats.call` (1 Hz).

## 6. Audio

- PJSIP's audio backend: PipeWire through its ALSA/PulseAudio
  compatibility layer on Linux at first. A native PipeWire backend for
  PJSIP is evaluated after the MVP if NFR-04 latency isn't met. WASAPI on
  Windows, Core Audio on macOS.
- Three device roles: headset (capture + playback), speaker (hands-free),
  ringer (ring only). The ringer plays through its own player into a second
  playback device, outside the call's conference bridge slot.
- Hot-plug: `ferro-audio` watches the OS for device changes, calls
  `pjmedia_aud_dev_refresh` through the engine, then moves an active call to
  the fallback device by name matching.
- AEC: PJSIP's WebRTC AEC3 at a 10 ms frame size.

## 7. Security

- Socket access: owner only (permissions and peer credential check, or a pipe ACL).
- Secrets: OS keyring; never written to the config, logs or traces. SIP traces
  redact the `Authorization` header digest response by default.
- TLS: system trust store, validation on; per-account pinned CA is optional.
- The Tauri webview has a strict CSP, no remote content, and only the
  commands the UI needs are allowed.

## 8. Testing

| Layer | Tooling |
| --- | --- |
| Unit | `cargo test` per crate; `ferro-core` tested against a fake engine (trait `Engine`) |
| API conformance | Generated from `openrpc.json`: every method, error code and event shape |
| Integration | `tests/scenarios` start `ferrophoned --headless --null-audio --test-hooks` against containerized Asterisk and FreeSWITCH; SIPp for edge cases (PRACK, 183, malformed messages) |
| Network | netem container for loss, jitter and NAT; scripted interface flaps for NFR-05 |
| UI | Vitest for components; Playwright against the Tauri dev build with a mocked API for key flows |
| Manual | Interop checklist (NFR-08), audio quality (FR-20, NFR-04), headset hot-plug (FR-21) |
