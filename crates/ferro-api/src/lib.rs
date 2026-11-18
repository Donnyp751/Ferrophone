// SPDX-License-Identifier: GPL-2.0-or-later
//! Control API v1 (docs/control-api.md, decision D-04).
//!
//! Types for requests, results and events are added in M1; the OpenRPC
//! schema in `api/openrpc.json` is generated from them.

/// API major version agreed in `system.hello`.
pub const API_VERSION: &str = "1";

/// Ferrophone-specific JSON-RPC error codes.
pub mod error_code {
    /// No `system.hello` yet, or the engine is still starting.
    pub const NOT_READY: i32 = -32010;
    /// Requested API version is not supported.
    pub const UNSUPPORTED_API: i32 = -32011;
    /// Bad or missing token (TCP only).
    pub const UNAUTHORIZED: i32 = -32012;
    /// Unknown account, call or key id.
    pub const NOT_FOUND: i32 = -32020;
    /// The object is in the wrong state for this method.
    pub const INVALID_STATE: i32 = -32021;
    /// All line appearances are busy.
    pub const NO_FREE_LINE: i32 = -32022;
    /// The engine rejected the request.
    pub const SIP_ERROR: i32 = -32030;
    /// A `test.*` method was called without `--test-hooks`.
    pub const TEST_HOOKS_DISABLED: i32 = -32040;
}
