# Ferrophone: notes for Claude

Desk-phone style SIP softphone. Rust daemon `ferrophoned` (owns PJSIP through pjsua-rust) + Tauri 2/React UI + `ferroctl` CLI; all clients use the JSON-RPC 2.0 control API.

- Requirements, decisions, plan: `docs/`. The IDs (FR-xx, NFR-xx, D-xx, Q-xx) are stable: never renumber or reuse them; add new IDs at the end.
- To change a decision, add a new D-entry that supersedes it in `docs/decisions.md`; don't rewrite history.
- Only `ferro-engine` (and platform glue in `ferro-audio`) may use `unsafe` or touch PJSIP.
- Generic PJSUA features go upstream into pjsua-rust (D-07), not into this repo.
- Every source file starts with `// SPDX-License-Identifier: GPL-2.0-or-later`.
- The control API is a public contract: changes within v1 are additive only; regenerate `api/openrpc.json`.
- Related local checkouts: `~/projects/pjproject` (2.17), `~/projects/voip-utility`, `~/projects/multicast-paging-utility`.
- Lab test config: `tests/lab/lab.toml` (gitignored), generated from `~/projects/z-series-monorepo/testing/config/environment.json`. Use only the reserved extensions in docs/testing.md; never commit lab hosts or credentials.
