# Ferrophone requirements

Version 0.2 · 2026-11-17 · supersedes the "SIP Deskphone App: Requirements (Draft v0.1)" doc.

Ferrophone is a free, open source softphone that behaves like a business desk
phone (line keys, BLF keys, transfer, park, paging) rather than a chat app. It
is a SIP user agent for people who sit behind a PBX such as Asterisk,
FreeSWITCH, FreePBX or 3CX. It ships on Linux first, then Windows, then macOS.

Architecture and the reasons behind each choice are in
[architecture.md](architecture.md) and [decisions.md](decisions.md). The
delivery plan is in [plan.md](plan.md).

## 1. Scope

**In scope:** desktop SIP user agent; desk-phone UI with a compact view; BLF
sidecar; headless core with a documented local control API; SIP and RTP
diagnostics for engineers and testers.

**Out of scope:** being a PBX or SIP proxy; video conferencing; a WhatsApp-style
messenger; mobile apps; embedded or Yocto targets; any cloud service or user
account run by us.

### Users

| Persona | Needs most |
| --- | --- |
| Office user on a PBX | Answer, transfer and park quickly; see colleagues' BLF lamps; voicemail light |
| Receptionist / operator | Large BLF grid or sidecar, many lines, attended transfer, park/pickup |
| VoIP engineer / tester | SIP trace, RTP stats, scriptable control, headless mode in CI |
| Home lab user | Easy setup against Asterisk/FreePBX, multiple accounts, sane defaults |

## 2. Priorities

| Label | Meaning |
| --- | --- |
| **MVP** | Needed for the first public release, 0.1 (milestones M1–M5 in plan.md) |
| **v1.0** | Needed for 1.0, the first release recommended for daily office use |
| **Later** | Wanted, not scheduled |

Every MVP requirement has an acceptance criterion (AC). An AC marked *[CI]* must
be covered by an automated test against the CI PBX (see NFR-09). The
"Engine gap" column marks work that has to go into `pjsua-rust` first (see
decisions.md, D-07).

## 3. Functional requirements

### Accounts and network

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-01 | Multiple SIP accounts (at least 8). Each registers over UDP, TCP or TLS, and can be enabled or disabled independently. | MVP | 3 accounts on 3 transports register at once; disabling one sends REGISTER with Expires: 0 and leaves the others registered. *[CI]* | — |
| FR-02 | Digest auth, outbound proxy, DNS NAPTR/SRV lookup, re-registration before expiry, failover to the next SRV target. | MVP | Registration survives a 120 s expiry cycle. With the first SRV target down, the account registers to the second within 35 s. *[CI]* | — |
| FR-03 | NAT handling: rport, STUN, keepalives (CRLF for TCP/TLS, OPTIONS or CRLF for UDP, configurable interval). | MVP | Behind a NAT container, an inbound call has two-way audio for 60 s. *[CI]* | — |
| FR-04 | ICE and TURN for hard NATs. | v1.0 | | |

### Calls

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-05 | Dial, answer, reject, hang up; dial pad; DTMF via RFC 4733 and SIP INFO, selectable per account. | MVP | Digits 0–9 * # sent each way are detected by the far end. *[CI]* | — |
| FR-06 | Hold, resume, mute, call waiting; at least 4 line appearances. | MVP | 4 simultaneous calls; answering a new one puts the active one on hold; resume restores two-way audio. *[CI]* | — |
| FR-07 | Blind transfer and attended transfer (REFER, REFER with Replaces), plus transfer by feature code. | MVP | Blind and attended transfer each connect A to C, and B drops on NOTIFY 200. *[CI]* | Attended transfer (`pjsua_call_xfer_replaces`) |
| FR-08 | Early media and PRACK (100rel). | MVP | Ringback from a 183 with SDP is heard; a 100rel-required INVITE gets PRACK. *[CI]* | — |
| FR-09 | Local 3-way conference using the conference bridge. | v1.0 | | |
| FR-10 | DND, call forwarding (always, busy, no answer), auto answer by caller or by Call-Info answer-after (intercom). | v1.0 | | |
| FR-11 | Call park and pickup, from feature codes and from a BLF key. | v1.0 | | |
| FR-12 | Call recording to WAV or Opus files, with a visible indicator while recording. | v1.0 | | |

### Desk phone

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-13 | Programmable key grid. Key types: line, speed dial, BLF, park, DTMF macro, URL action. Configured in the UI and in the config file. | MVP | Each key type can be created, edited, reordered and deleted, and survives a restart. | — |
| FR-14 | BLF through the dialog event package (RFC 4235) and through presence (RFC 3856), chosen per key with a per-account default. Lamp states: idle, ringing (blink), busy, unknown. Pressing a ringing BLF key does directed pickup. | MVP | Against Asterisk (dialog) and FreeSWITCH (presence), lamps follow the monitored extension within 1 s. *[CI]* | Buddy API with `subscribe_dlg_event` and presence |
| FR-15 | MWI (RFC 3842): solicited subscription and unsolicited NOTIFY; message count on the voicemail key, which dials the account's voicemail number. | MVP | Leaving a voicemail lights the key with the right count; listening clears it. *[CI]* | MWI callbacks |
| FR-16 | Sidecar mode: a window with only the BLF grid, usable beside a hardware phone. No audio device is needed. | v1.0 | | |
| FR-17 | Multicast paging receive with priority channels (reuse multicast-paging-utility's receiver). | Later | | |
| FR-36 | Compact softphone view: dial field, active call and a short key list, switchable from the desk view. | v1.0 | | |

### Audio

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-18 | Codecs G.711 µ-law and A-law, G.722, Opus; priority set per account. | MVP | Each codec is negotiated when it is the only one offered. *[CI]* | — |
| FR-19 | G.729 and iLBC. | Later | | |
| FR-20 | Echo cancellation (WebRTC AEC3 through PJSIP), noise suppression, AGC, adaptive jitter buffer. Each can be turned off. | MVP | Measured echo return loss enhancement ≥ 25 dB on a loopback rig (manual test). | — |
| FR-21 | Separate headset, speaker and ringer devices. Device list and hot-plug: unplugging the active headset during a call switches to the fallback device without dropping the call. | MVP | Unplug and replug a USB headset mid-call; audio continues within 2 s (manual test). | Device enumeration; hot-plug detection (PJSIP has no event, see D-07) |
| FR-22 | USB headset HID controls: hook switch, mute, lights (Jabra, Poly, EPOS first). | v1.0 | | |

### Security

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-23 | SRTP with SDES keys over TLS, per account: off, optional or mandatory. | MVP | With mandatory SRTP, a plain-RTP offer is rejected with 488; a TLS + SRTP call has audio. *[CI]* | — |
| FR-24 | DTLS-SRTP and ZRTP. | Later | | |

### Contacts and history

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-25 | Call history (in, out, missed; duration; account) with missed-call notification and a badge. Kept for 90 days by default; can be cleared. | MVP | A missed call raises a desktop notification and shows in history after a restart. | — |
| FR-26 | Local contacts, CardDAV sync, LDAP directory search. | v1.0 | | |

### Integration

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-27 | Click-to-dial: registers as the handler for `sip:`, `tel:` and `callto:` URIs; `ferroctl dial <uri>`. | MVP | Clicking a `tel:` link in a browser starts a call on the default account. | — |
| FR-28 | System tray icon with status, desktop notifications, global hotkeys for answer and hang up (through the XDG GlobalShortcuts portal on Wayland). | MVP | Hotkeys answer and hang up while the window is hidden, on GNOME and KDE Wayland. | — |
| FR-29 | Local control API v1: JSON-RPC 2.0 over a Unix socket (named pipe on Windows), with event notifications. Covers accounts, calls, DTMF, keys, BLF, MWI, state and events. Spec in [control-api.md](control-api.md). | MVP | Every UI action can be done through the API; the published schema matches the server (test generated from the schema). *[CI]* | — |
| FR-30 | Provisioning from a config file, an HTTPS URL or a QR code. | v1.0 | | |
| FR-37 | D-Bus interface on Linux (`org.ferrophone.Phone1`), mirroring the control API. | v1.0 | | |
| FR-38 | Test hooks in the control API (off by default, enabled by `--test-hooks`): WAV inject into a call, capture a call to WAV, DTMF receive verification, tone and level detection. Ported from voip-utility's analysis code. | MVP | A CI scenario plays a 1 kHz tone from A and detects it on B. *[CI]* | — |

### Media, messaging and diagnostics

| ID | Requirement | Priority | Acceptance criteria | Engine gap |
| --- | --- | --- | --- | --- |
| FR-31 | Video calls (H.264, VP8). | Later | | |
| FR-32 | SIP MESSAGE text chat. | Later | | |
| FR-33 | Live SIP trace viewer: message log, ladder diagram per call, filter by call or account, export as text or pcap. | MVP | Export of a 2-call session opens in Wireshark with all SIP messages decoded. | — (trace hook exists) |
| FR-34 | RTP stats per call, refreshed every second: codec, jitter, packet loss, RTT (from RTCP), estimated MOS (ITU-T G.107 E-model). | MVP | With 5% loss injected by netem, reported loss is 4–6% and MOS drops below 3.8. *[CI]* | Stream stats (`pjsua_call_get_stream_stat`) |
| FR-35 | Headless mode: `ferrophoned` runs with no UI, driven only through the control API, with null or file-backed audio. | MVP | The whole CI suite runs against `ferrophoned --headless --null-audio` in a container. *[CI]* | — |

FR-36 to FR-38 are new in v0.2. IDs are never reused.

## 4. Non-functional requirements

| ID | Area | Requirement | How it is checked |
| --- | --- | --- | --- |
| NFR-01 | Platforms | Linux x86_64 and aarch64 (Wayland and X11; GNOME and KDE tested) first; Windows 10/11 second; macOS 13+ third. | CI build matrix |
| NFR-02 | Packaging | Linux: Flatpak (Flathub), AppImage, .deb, .rpm. Windows: MSI and portable zip. macOS: signed, notarized .dmg. | Release job produces all artifacts |
| NFR-03 | Performance | Daemon: idle RSS < 30 MB, idle CPU < 0.5%. UI process: idle RSS < 150 MB (webview included). Cold start to a usable window < 2 s on a 2020-era laptop. | Benchmark job per release |
| NFR-04 | Audio latency | Latency added by the app < 60 ms mouth-to-ear on PipeWire at a 10 ms quantum. | Loopback measurement rig |
| NFR-05 | Reliability | Survives Wi-Fi to wired, VPN up/down and suspend/resume without a restart: re-registers within 10 s; active calls recover through re-INVITE where the PBX allows. A UI crash does not drop calls. | Scripted network-change test; kill-UI test *[CI]* |
| NFR-06 | Security | Credentials in the OS keyring (Secret Service, Windows Credential Manager, macOS Keychain), never in the config file. TLS certificate validation on by default. Control socket reachable only by the owning user. | Unit tests; socket permission test |
| NFR-07 | Privacy | No telemetry; no network traffic except to configured servers and update checks (update checks can be turned off). | Review gate |
| NFR-08 | Interop | Automated: Asterisk 22 and FreeSWITCH 1.10 in containers; Asterisk 20.6 in the lab (see testing.md). Manual before each release: FreePBX 17, 3CX v20, Kamailio 5.8, two SIP trunk providers. | Interop matrix in the release checklist |
| NFR-09 | Testability | Headless core runs in CI against containerized Asterisk and FreeSWITCH plus SIPp scenarios, scripted through the control API. | CI |
| NFR-10 | UX | Usable with the keyboard only; screen reader labels; light and dark themes following the OS; HiDPI. | Accessibility checklist per release |
| NFR-11 | Localization | All UI strings translatable from day one (ICU MessageFormat); English first. | Lint rule: no hard-coded UI strings |
| NFR-12 | Logging | Structured logs (`tracing`) with a level per subsystem (sip, media, api, ui); rotating log files; PJSIP log bridged in. | — |
| NFR-13 | API stability | Control API is versioned (`v1`); changes within v1 are additive only; deprecations last at least one minor release. | Schema diff check in CI |
| NFR-14 | License | Ferrophone source is GPL-2.0-or-later. Shipped binaries are distributed under GPL-3.0 terms (see D-02), so every dependency must be GPL-3.0 compatible (MIT, BSD, Apache-2.0, LGPL, MPL-2.0, GPL-2.0-or-later). | `cargo deny` and npm license check in CI |

## 5. Assumptions to revisit

- **PBX targets:** Asterisk and FreeSWITCH are the automated targets because both run in containers; the lab Asterisk 20.6 pair adds real TLS, SRTP and SRV failover ([testing.md](testing.md)). If a commercial PBX matters more (for example your own products), it moves into NFR-08's automated list.
- **voip-utility scope:** only the audio and DTMF analysis moves into the core (FR-38). Step-based JSON test scenarios stay outside it and run through the control API, for example a later `ferrophone-scenarios` tool.
- **pjproject fork:** we build against upstream PJSIP 2.17 plus the `mtu-clamp-fix-2.17` patch, carried as a patch until it is merged upstream.
