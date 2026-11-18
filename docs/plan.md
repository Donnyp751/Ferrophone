# Project plan

Delivered in seven milestones: M0–M5 lead to the first public release, 0.1
(all MVP requirements, Linux only), and M6–M7 lead to 1.0. Each milestone has
exit criteria that must pass before the next one starts. Week estimates assume
one developer working with Claude Code, and get re-estimated at the end of each
milestone.

| Milestone | Goal | Requirements | Estimate |
| --- | --- | --- | --- |
| M0 Foundations | Repo, build, CI and the pjsua-rust gap list in progress | NFR-09, NFR-12, NFR-14 | 1–2 wk |
| M1 Headless phone | `ferrophoned` + `ferroctl` place and take calls through the API | FR-01–03, 05, 06, 08, 18, 23, 29, 35 | 3–4 wk |
| M2 Desk phone UI | Tauri app drives the daemon: lines, dial pad, call panel, tray | FR-25, 27, 28, NFR-10, 11 | 3 wk |
| M3 Desk phone features | Key grid, BLF, MWI, attended transfer; FreeSWITCH container | FR-07, 13, 14, 15 | 3 wk |
| M4 Diagnostics and test hooks | SIP trace viewer, RTP stats, test hooks | FR-33, 34, 38 | 2 wk |
| M5 Audio hardening, release 0.1 | Device roles, hot-plug, AEC tuning, network resilience, Linux packages | FR-20, 21, NFR-02 (Linux), 03–08 | 2–3 wk |
| M6 1.0 features | Conference, DND/forward/intercom, park, recording, sidecar, compact view, contacts, provisioning, ICE, D-Bus, HID | all v1.0 FRs | 6–8 wk |
| M7 Windows, then macOS | Ports, installers, signing | NFR-01, 02 | 3–4 wk each |

## M0 Foundations

- [ ] Create the GitHub repo (personal account); push `main`; branch protection
- [x] GPL-2.0-or-later `LICENSE`
- [x] Cargo workspace with the crate layout from architecture.md §2; `rustfmt`, `clippy -D warnings`, `cargo deny` (licenses per NFR-14)
- [x] `scripts/build-pjproject.sh`: pjproject 2.17 + `mtu-clamp-fix` patch + our `config_site.h` → `third_party/prefix`, cached in CI
- [x] Depend on `pjsua-rust` at a pinned git revision; a smoke test registers against a local Asterisk
- [x] `tests/interop/docker-compose.yml`: Asterisk 22.11 with 8 test extensions, voicemail and BLF hints
- [ ] FreeSWITCH 1.10 container with presence: moved to M3 (official images need a SignalWire token; only presence BLF needs it)
- [x] Lab tier: `lab` cargo feature, `tests/lab/lab.toml` loader, first lab test registering the 17 reserved extensions (docs/testing.md)
- [ ] Ask the lab owner for BLF hints, voicemail/MWI and pickup on the reserved extensions (testing.md, "What the lab PBX can't test yet")
- [x] GitHub Actions: build + unit tests on ubuntu-24.04 (x86_64, aarch64); interop job with compose
- [ ] Open pjsua-rust issues or PRs for every D-07 gap; start with stream stats and the buddy/dialog-event API
- [x] `tracing` setup with per-subsystem levels; PJSIP log bridge (provided by pjsua-rust, target `pjsip`)

**Status (2026-11-17):** locally, the lab test registers and unregisters all 17 reserved extensions (UDP, mTLS, mTLS + SRTP) and the container test passes. Remaining: GitHub repo, a first green CI run, pjsua-rust PRs, the lab-owner request.

**Exit:** a CI run on a clean machine builds everything and a test binary registers to the CI Asterisk; the same binary registers all reserved extensions on the lab PBX.

## M1 Headless phone

- [ ] `ferro-engine` actor thread with an `Engine` trait plus a fake for unit tests
- [ ] `ferro-core`: accounts (add, update, remove, enable), registration state, lines and call state machine, codec and SRTP settings, config.toml load and save, keyring secrets
- [ ] `ferro-api` types and OpenRPC generation; `api/openrpc.json` checked in, with a CI diff check
- [ ] `ferrophoned`: socket server, `hello`, event subscriptions, the `account.*`, `call.*` and `system.*` methods; `--headless --null-audio`; systemd user unit
- [ ] `ferro-client` + `ferroctl`: `status`, `accounts`, `dial`, `answer`, `hangup`, `hold`, `resume`, `dtmf`, `events --follow`, `raw`
- [ ] Integration tests for FR-01, 02, 03, 05, 06, 08, 18, 23, 35

**Exit:** all of M1's *[CI]* acceptance criteria pass against Asterisk and FreeSWITCH; a call made through `ferroctl` has two-way audio on a real headset.

## M2 Desk phone UI

- [ ] Tauri 2 app skeleton: connection manager in the Rust shell, daemon auto-start, reconnect banner, TS types generated from OpenRPC
- [ ] Desk view: 4+ line keys, call panel (hold, mute, transfer button, DTMF), dial pad, account status strip
- [ ] Account settings UI (writes through the API); audio device picker
- [ ] History view + missed-call badge and notifications (FR-25)
- [ ] Tray icon, global hotkeys via the portal on Wayland, URI handler registration (FR-27, 28)
- [ ] i18n scaffold, keyboard navigation, screen-reader labels, themes (NFR-10, 11)
- [ ] Wireframe review before building the desk layout; see open question Q-03

**Exit:** a non-developer can add an account and make, take, hold and blind-transfer calls using only the UI; killing the UI mid-call leaves the call up.

## M3 Desk phone features

- [ ] pjsua-rust: attended transfer, buddy and dialog-event subscriptions, MWI merged (or carried on a branch per D-07)
- [ ] Key grid model, pages, editor UI, config persistence (FR-13)
- [ ] BLF lamps (dialog and presence), directed pickup from a ringing lamp (FR-14)
- [ ] MWI count + voicemail key (FR-15)
- [ ] Attended and feature-code transfer flows in the UI (FR-07)
- [ ] FreeSWITCH 1.10 in `tests/interop` for presence BLF (moved from M0)

**Exit:** a 24-key BLF grid against Asterisk and FreeSWITCH tracks state within 1 s; all of M3's *[CI]* criteria pass.

## M4 Diagnostics and test hooks

- [ ] Trace stream + viewer: log, per-call ladder diagram, filters, text and pcap export (FR-33)
- [ ] pjsua-rust stream stats → stats panel with an E-model MOS estimate (FR-34)
- [ ] `ferro-audio` analysis ported from voip-utility: Goertzel/FFT tone detection, RMS level, DTMF verify; `test.*` methods (FR-38)
- [ ] netem loss test for FR-34

**Exit:** a tester can debug a one-way audio problem using only the trace and stats panels; the tone-detect CI scenario passes.

## M5 Audio hardening and release 0.1

- [ ] Headset, speaker and ringer roles; hot-plug watcher on Linux; fallback device (FR-21)
- [ ] AEC3, NS and AGC defaults and toggles; ERLE measurement on a loopback rig (FR-20)
- [ ] Network change handling and suspend/resume re-registration (NFR-05)
- [ ] Performance and latency measurements against NFR-03 and NFR-04; fix or re-baseline
- [ ] Packages: Flatpak manifest (PJSIP as a module), AppImage, .deb, .rpm
- [ ] Manual interop pass: FreePBX 17, 3CX v20, Kamailio 5.8, two trunk providers (NFR-08)
- [ ] User docs: install, first account, keys, ferroctl; control API reference published

**Exit:** release 0.1 tagged; every MVP requirement meets its AC; packages install on Ubuntu 24.04, Fedora 41 and through Flathub.

## M6 Release 1.0 (Linux)

Order: FR-36 compact view and FR-16 sidecar → FR-11 park/pickup → FR-10 DND,
forwarding and intercom → FR-09 conference → FR-12 recording → FR-26 contacts →
FR-30 provisioning → FR-04 ICE/TURN → FR-37 D-Bus → FR-22 headset HID. Reassess
after each.

## M7 Windows, then macOS

Named pipe transport, WASAPI device watcher, Windows Credential Manager, MSI +
portable zip. Then Core Audio, Keychain, launchd login item, signed and
notarized .dmg. The architecture work for both is already done in M1 (`ferro-api`
transport trait, keyring abstraction).

## Risks

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| pjsua-rust upstream review slows M3 | Medium | Medium | Carry `ferrophone/main` branch (D-07); start PRs in M0 |
| WebKitGTK memory or rendering issues | Medium | Low | UI budget set at 150 MB; test on GNOME and KDE early; Tauri lets us swap the renderer later |
| Wayland global hotkeys missing on some desktops | High | Low | GlobalShortcuts portal where present; document the fallback (ferroctl bound to desktop shortcuts) |
| PipeWire latency through the ALSA layer exceeds NFR-04 | Medium | Medium | Measure in M5; native PipeWire backend for PJSIP as a contingency |
| BLF differences between PBXs | High | Medium | Support both RFC 4235 and RFC 3856 from day one; interop matrix |
| GPL scares off integrators | Low | Low | The control API lets closed tools integrate without linking |

## Open questions

| ID | Question | Needed by |
| --- | --- | --- |
| Q-01 | Are there commercial PBXs (for example your own products) to add to automated interop? | M0 |
| Q-06 | Can BLF hints, voicemail/MWI and pickup be added for 6025–6033 etc. on the shared lab Asterisk, or should Ferrophone get its own lab PBX? | M3 |
| Q-02 | Is the Flathub app ID `io.github.<user>.Ferrophone` (personal hosting) acceptable, or is a domain planned? | M5 |
| Q-03 | Desk layout: number of line keys and BLF keys shown by default, key pages vs scroll. Settle with a wireframe in M2. | M2 |
| Q-04 | `trace.message` payload: raw SIP only, or raw plus parsed fields? | M4 |
| Q-05 | Publish pjsua-rust to crates.io, or keep it as a git dependency? | M5 |
