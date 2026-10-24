# Experimental loopback HTTP MCP transport


This experimental Rust transport reuses `vectorcraft_mcp::Server`, the native tool catalogue and
`Remote`. It is a proof for architecture discussion, not a public remote-access release.
The desktop editor and its file/selection state are shared by all authorized callers.

Build with `cargo build -p vectorcraft-cli --features http`. Ordinary builds keep stdio unchanged;
the HTTP module and its dependencies are not compiled for WebAssembly.

Create a private file containing 32 cryptographically random bytes encoded as 64 hexadecimal
characters. Start a separate desktop profile with `--control 33982` and automation read/write roots
restricted to a disposable artwork folder. Then run:

```text
vectorcraft-cli mcp --http 127.0.0.1:33981 --http-token-file PATH --connect 127.0.0.1:33982
```

The endpoint is `http://127.0.0.1:33981/mcp`. Every request, including `/health`, requires
`Authorization: Bearer TOKEN`. Tokens are loaded from a file rather than placed on the command
line and are never printed by the server. Protect the token file with the current user's OS ACL.

The listener and desktop destination must both be loopback IP addresses. Host and optional Origin
must match the literal listener address. The implementation does not trust forwarded headers or
enable cross-origin browser access. POST requires JSON and Accept values for both JSON and SSE.
Requests are capped at 12 MiB and HTTP headers at 16 KiB. One HTTP/1.1 request is handled per
connection with an absolute 10-second header-and-body deadline and a bounded response-write wait.
Requests require a Content-Length for POST; chunked/other Transfer-Encoding and Expect requests
are explicitly rejected. Rejected requests are **not** drained before closing their connections.
This local prototype does not support keep-alive or request pipelining. Duplicate security or
framing headers, unsupported protocol versions and malformed/batched messages are refused before
reaching the editor. Replies use JSON, notifications
receive empty 202 responses, and standalone GET/SSE and DELETE receive 405 responses. No HTTP
session table is allocated. Each request's protocol-version header selects its native revision.

Tool definitions, prompts and resource lists work while the desktop is closed. A desktop operation
then returns an explicit unreachable error; it never edits a separate headless document. The next
explicit operation can reconnect after the user opens the desktop. Requests run serially and are
not automatically replayed when the client disconnects. A missing native reply can mean the edit
completed: inspect the document before explicitly retrying it.

The original loopback transport deliberately omits OAuth, an authorization/settings interface, background
service installation, remote app launching, SSE progress/logging, and HTTP cancellation. It does
not advertise logging support. Long-running backend edits still execute serially and can delay other requests. A read deadline is
not an admission control system: malicious local clients can still monopolize the listener by
repeating stalled requests. Additional admission/rate controls are required before any public use. A bearer token alone does
not implement the MCP OAuth authorization flow. **Do not publish or tunnel this prototype.**

Transport behavior follows the public MCP specification:
<https://modelcontextprotocol.io/specification/2025-06-18/basic/transports>.

Tests in `crates/mcp/src/http/tests.rs` exercise the real loopback HTTP listener, authorization on
every request, Host/Origin checks, metadata fidelity, protocol isolation, framing/limits, desktop
absence and dispatch-without-replay. Existing MCP tests continue to exercise native tool behavior.

## Validation scope

The transport is optional and has no OAuth server, app settings or process-launch integration.
The Windows workspace gate also needs two existing SVG test assertions to use platform-aware
filesystem paths; those test-only portability corrections accompany this first part.
