# Test environments

Ferrophone is tested at three levels. The first two are automated.

| Tier | Where | PBX | Runs | What it covers |
| --- | --- | --- | --- | --- |
| Container | GitHub Actions and any Linux dev box | Asterisk 22.11 (`andrius/asterisk:22.11.0_debian-trixie`) in `tests/interop/`; FreeSWITCH, SIPp and netem join in later milestones | Every push | All *[CI]* acceptance criteria |
| Lab | Maintainer's lab network | Two Asterisk 20.6 servers (primary + alternate), DNS SRV records, mTLS CA | On demand: see "Running the tests" | Real-network checks: TLS/mTLS against a real CA, SRTP, SRV failover across two servers, NAT/VPN changes, audio on real hardware |
| Manual | Release checklist | FreePBX 17, 3CX v20, Kamailio 5.8, two trunk providers | Before each release | NFR-08 interop matrix |

## Running the tests

```sh
scripts/build-pjproject.sh
export PKG_CONFIG_PATH="$PWD/third_party/prefix/lib/pkgconfig"
cargo test --workspace                                        # unit tests

docker compose -f tests/interop/docker-compose.yml up -d --wait
FERROPHONE_INTEROP_HOST=127.0.0.1 cargo test --workspace --features ferro-engine/interop

FERROPHONE_LAB=tests/lab/lab.toml cargo test --workspace --features ferro-engine/lab -- --test-threads=1
```

Debugging: `FERROPHONE_PJSIP_LOG=5 RUST_LOG=info,pjsip=trace` prints every SIP message.

The container PBX has extensions 1001–1008 (test secret in `tests/interop/asterisk/pjsip.conf`),
BLF hints and voicemail on each, `*97` voicemail, `600` echo, `601` 1 kHz tone, and `**<ext>` directed pickup.

## Lab tier

- **Config:** `tests/lab/lab.toml` (gitignored, mode 0600). Copy
  `tests/lab/lab.toml.template`. No lab hostnames, addresses or credentials go
  into the repository: it is public.
- **Shared server:** the lab PBX also runs another product's test suites. Every
  endpoint has `max_contacts=1` and `remove_existing=yes`, so registering an
  extension that another suite uses would kick its device off. Ferrophone
  uses only its reserved extensions, and lab tests run serially.
- **Reserved extensions** (chosen 2026-11-17 because no other suite references them):

  | Tier | Extensions | Transport | Media |
  | --- | --- | --- | --- |
  | open | 6025–6033 (9) | UDP, TCP or TLS | RTP; `direct_media=yes` (media goes peer to peer) |
  | medium | 6056–6057 (2) | Mutual TLS (port 5062), client cert required | RTP |
  | strict | 6073–6078 (6) | Mutual TLS | SDES-SRTP mandatory |

  Any `6XXX` extension can dial any other; calls are recorded server-side by MixMonitor.

## What the lab PBX can't test yet

These need dialplan or config changes on the shared server. Ask the lab
owner before making them; the container tier covers them in the meantime.

| Gap | Blocks | Change needed |
| --- | --- | --- |
| No device-state hints for the reserved extensions | FR-14 BLF (dialog event and presence) | `exten => 6025,hint,PJSIP/6025` (etc.) in `[user]` |
| No voicemail boxes or `mailboxes=` on endpoints | FR-15 MWI | voicemail.conf boxes + `mailboxes=6025@default` on the endpoints |
| No call parking extension | FR-11 park/pickup (v1.0) | `res_parking` lot + dialplan include |
| No pickup group | FR-14 directed pickup | `pickup_group`/`named_pickup_group` or `Pickup()` feature code |
| No echo or test-tone extension in `[user]` | Audio checks without a second endpoint | Echo() extension for 6XXX |

## Other lab resources (later)

- Multicast paging zones 224.1.1.1–11, port 5000, G.711 µ-law: for FR-17.
- A commercial trunk provider account exists, shared with pjsua-rust's
  integration tests. Use it for the manual trunk interop pass only, never in
  automated runs (real calls, shared accounts).
