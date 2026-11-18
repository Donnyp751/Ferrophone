# Control API v1

Status: **draft**. It is frozen at release 0.1. After that, changes within v1
are additive only (NFR-13). The schema, `api/openrpc.json`, is the authority;
this page explains the conventions.

## Transport and framing

- JSON-RPC 2.0. Each message is one UTF-8 JSON object followed by `\n`.
  Message size is limited to 1 MiB.
- Endpoints:
  - Linux: `$XDG_RUNTIME_DIR/ferrophone/control.sock`
  - macOS: `~/Library/Application Support/ferrophone/control.sock`
  - Windows: `\\.\pipe\ferrophone-<SID>`
  - Optional: `tcp://127.0.0.1:<port>` with `--api-tcp`; requires a token.
- Override with `FERROPHONE_SOCKET` or `ferrophoned --socket <path>`.

## Session

1. The client sends `system.hello`:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"system.hello","params":{"api":"1","client":"ferroctl/0.1.0","token":null}}
   ```
2. The server replies with its version and capabilities:
   ```json
   {"jsonrpc":"2.0","id":1,"result":{"api":"1","server":"ferrophoned/0.1.0","capabilities":["test-hooks"]}}
   ```
   Any other method sent before `hello` fails with `-32010 not_ready`.
3. To receive events, the client calls `events.subscribe` with topic patterns:
   ```json
   {"jsonrpc":"2.0","id":2,"method":"events.subscribe","params":{"topics":["call.*","blf.*"]}}
   ```
   Events arrive as notifications, with a sequence number that increases per connection:
   ```json
   {"jsonrpc":"2.0","method":"event","params":{"seq":17,"topic":"call.state","data":{"call":"call_7Hq…","state":"active"}}}
   ```
4. After a reconnect, the client calls `*.list` to rebuild its state. The API
   doesn't replay events.

## Conventions

- Method names are `namespace.verb` in camelCase; params and results are JSON
  objects in camelCase.
- IDs are opaque prefixed strings (`acc_`, `call_`, `key_`).
- Times are RFC 3339 UTC; durations are integer milliseconds.
- Commands that start SIP work (`call.dial`, `call.transfer.*`) return as soon as
  the request is accepted. The result arrives through events.

## Errors

Standard JSON-RPC codes apply (-32700 to -32603). Ferrophone codes:

| Code | Name | Meaning |
| --- | --- | --- |
| -32010 | `not_ready` | No `system.hello` yet, or the engine is still starting |
| -32011 | `unsupported_api` | Requested API version not supported |
| -32012 | `unauthorized` | Bad or missing token (TCP only) |
| -32020 | `not_found` | Unknown account, call or key id |
| -32021 | `invalid_state` | For example, `call.resume` on a call that isn't held |
| -32022 | `no_free_line` | All line appearances are busy |
| -32030 | `sip_error` | The engine rejected the request; `data` has `pjStatus` and `reason` |
| -32040 | `test_hooks_disabled` | A `test.*` method was called without `--test-hooks` |

## Example: dial, wait for answer, send DTMF, hang up

```sh
socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/ferrophone/control.sock <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"system.hello","params":{"api":"1","client":"sh"}}
{"jsonrpc":"2.0","id":2,"method":"call.dial","params":{"account":"acc_default","uri":"sip:600@pbx.lan"}}
EOF
```

`ferroctl` wraps this: `ferroctl dial 600 --wait-answer && ferroctl dtmf 1234 && ferroctl hangup`.

## Open items

- Full method and event list with parameter schemas: written alongside M1 and
  generated into `api/openrpc.json`.
- Whether `trace.message` carries raw SIP text or parsed fields as well (lean: raw text plus call id and direction).
