# Native HTTP and OAuth experiment

This checkout contains two optional transports. The original `http` feature remains a loopback-only
bearer-token experiment. The `remote` feature adds an OAuth service for testing through a trusted
HTTPS reverse proxy. Neither is enabled in an ordinary upstream build. This is a local architecture
prototype; it has not been released. Hosted OpenAI calls and a full SDK scenario through the
public gateway pass. Gemini's structured tools work in user testing, with a preview-visibility
limitation documented below.

The full `cargo xtask ci` gate passes all ten stages on Windows with serialized tests: 8,437
workspace tests and the separate no-bundled-fonts test pass. Thirteen existing tests remain ignored
(timing/quality-report probes and a documented tracing-junction case); none were disabled for this
addition. Workspace all-target clippy with the optional remote features also passes, along with
154 remote MCP tests, 20 focused wizard tests and 8 host tests. CI follow-ups fix two SVG tests' platform-path
assumptions, supply required catalog entries, and use the shared asynchronous folder picker with
a regression test. The local manual review is complete. Native macOS/Linux validation, translation
review and independent maintainer review remain.
The wizard is not yet fully localized.

Manual-review fixes enforce character-based passphrase bounds, private-directory-only settings
storage and a matching read/write size limit. Unavailable artwork folders no longer prevent local
editor startup: automated file access stays denied until the owner repairs and validates the setup.
Windows replaces old explicit ACL grants on the private folder and existing credential files through
built-in Windows PowerShell/.NET, launched without a visible console. No script files or runtime
installation are required. The service itself remains Rust and the crate still forbids unsafe code.
In a Remote AI desktop build with preferences enabled, CLI automation-root flags are refused:
the wizard manages its folder authority, so command-line roots cannot replace its grants or
private exclusions. Ordinary builds and their existing CLI automation support are unchanged.

An unconfigured remote-access build does not create artwork folders or open a control listener.
Disabled saved connections also leave the implicit listener closed. A successful owner setup (or
explicit start of a saved connection) installs validated folder authority before activating the
control channel and HTTP service. Failure to protect or read the private store disables remote
automation without aborting the desktop editor; repair remains local. Explicit CLI control remains
an intentional opt-in, while a failing saved remote setup disables it rather than falling back to
unconfined access.

The desktop control channel retains upstream's trusted-loopback assumption: reserve its configured
port for the intended executable/profile. It does not authenticate other local programs. Folder
checks are not an OS sandbox and do not prevent local filesystem races or processes running as
the owner. Invalid JSON settings may still need a local restore; the repair flow covers readable
settings with unavailable folders. No native macOS/Linux execution or independent security audit
is claimed by the local review.

## OAuth desktop build

```text
cargo build -p vectorcraft -p vectorcraft-cli --features vectorcraft/remote-access,vectorcraft-cli/remote
vectorcraft --profile-dir ABSOLUTE_TEST_PROFILE --remote-setup
```

The native **Remote AI connections** wizard is under Preferences › Remote AI. It chooses up
to 32 approved artwork folders, the public HTTPS gateway origin, and an owner passphrase. Add folders
individually through the picker or paste paths, and remove unwanted grants from the list. It starts a separate mode
of the same Rust executable, which serves HTTP without opening an editor window. Closing the
editor leaves this service available. The optional Windows remote-access build uses the GUI subsystem
even in debug mode; closing a terminal is not part of its lifecycle. This prototype does not install Windows sign-in startup.
New passphrases require matching confirmation. An eye button reveals or masks both inputs locally;
leaving both blank preserves an existing passphrase. Done closes the saved Connections step without
stopping the service, and refuses unsaved drafts.
Artwork rows show green checks only for grants reported by the running service. Pending additions
show a red X; removals also enable Resync folders. The owner-only resync command validates the folder
draft, refuses pending work, gracefully stops the old service, and saves only the folder settings.
The UI installs the new authority before starting the service; OAuth state and the owner hash are
preserved. Status refreshes do not overwrite unsaved drafts. A failed start never shows new grants
as active. This changes access permissions, rather than copying artwork to a cloud service.
The explicit `open_vectorcraft` tool launches the configured executable and profile when needed;
metadata discovery never opens the editor. `list_shared_folders` reports the approved paths while
the editor is closed; use the existing `file.list` command to browse them. All clients share the live document and selection.

Default public paths, forwarded unchanged to TCP 3002, are:

```text
/vectorcraft-native/mcp
/vectorcraft-native/health
/vectorcraft-native/authorize
/vectorcraft-native/approve
/vectorcraft-native/register
/vectorcraft-native/token
/vectorcraft-native/revoke
/.well-known/oauth-authorization-server/vectorcraft-native
/.well-known/oauth-protected-resource/vectorcraft-native/mcp
```

The wizard defaults to **loopback HTTP only** (`127.0.0.1:3002`). Run the HTTPS gateway locally,
or use an encrypted tunnel to localhost. A gateway on another computer requires checking the LAN
option and entering its individual IPv4 address. This prototype's local listeners use IPv4.
Only that actual socket peer and loopback are then
accepted; forwarded identity headers never grant access. Restrict the host firewall to the same IP.
The public HTTPS endpoint does **not** encrypt the gateway-to-desktop HTTP hop: the wizard warns
about this before saving a LAN setup. Only opt in on a trusted private network or encrypted tunnel.
Older non-loopback settings without `trustedProxy` remain readable for repair but cannot start a
new service until the owner explicitly chooses the network policy and saves. Existing clients and
tokens are preserved when the public resource address stays unchanged.
Readable legacy settings still expose the authenticated local Stop action; Stop can persist a
disabled state while repair is needed, without weakening the private-storage requirements.

Use the full MCP URL when adding an OAuth connection to an AI client. The server supports dynamic
registration and separately registered confidential clients. If a client asks for credentials,
the wizard can register its exact callback and copy the generated ID and secret. The callback
policy accepts the hosted chat callback domains and Google's exact OAuth relay domains. It refuses
lookalike hosts, fragments, credentials and non-HTTPS production callbacks. New provider domains
require an explicit policy change; arbitrary redirect URIs are not accepted.

OAuth uses authorization codes with S256 PKCE, exact callback/client binding, an issuer response
parameter, same-browser consent cookies, a single resource audience, one-hour access tokens and
rotating fourteen-day refresh tokens. Reusing a refresh token revokes that connection's family.
Expired credentials, revoked clients and credentials for another resource are rejected. The local
store holds hashes of tokens and client secrets; the owner phrase uses Argon2id. State is committed
atomically before usable tokens are returned, and the private directory receives owner-only OS
permissions. Revoke individual clients or all authorizations in the wizard.

Management actions require a separate private token and an actual loopback peer. They cannot be
used through the reverse proxy. Remote commands, synthetic input and serialized UI inspection
cannot read or change wizard fields. While the wizard is open, remote control is paused; its
screenshots mask secrets. While the owner reveals a passphrase, screenshots are refused; queued
captures are discarded if visibility changed before delivery. Close it before requesting artwork edits. Explicit file targets are
checked before native dispatch, and native automation roots also confine commands, background
file operations and injected input. The private `PROFILE/Remote Access` folder is always excluded,
even when an approved parent contains it. Directory listings and background searches skip this
subtree. The service starts only after the local UI has installed a changed folder authority;
closing the wizard during a save keeps remote control paused until that transition completes.
This is an application-level file boundary, not an OS sandbox against other local processes.

The HTTP service checks Host/Origin and duplicate security headers, ignores forwarded identity,
bounds request bodies and admission, isolates consent retry budgets, and serializes native work
through a bounded worker. A missing mutation reply never triggers a replay. Unknown outcomes pause
new edits until the owner reviews the artwork and resumes from local settings. Requests use
stateless JSON Streamable HTTP; GET/SSE, streaming progress and HTTP cancellation are not provided.
Do not forward the desktop's separate loopback control port to the network.

Anonymous requests do not share route-wide lockout counters with real sign-ins. Passphrase retries
are limited to ten per validated browser sign-in, with at most two simultaneous Argon2 checks.
Pending sign-ins have per-client and separate saved/public-client bounds. Unapproved public
registrations expire after ten minutes and have a separate 32-entry capacity; they cannot fill
the 128-entry saved-client capacity. Owner approval makes a temporary client persistent. When
public registration is busy, create credentials in the local wizard or retry after expiry.
Authenticated local management has reserved request capacity, so public queue saturation does not
disable local recovery. These bounds do not claim availability against a sustained network attack;
deployments should also enforce appropriate limits at the trusted gateway.

Settings and auth state live in `PROFILE/Remote Access/`, excluded from every artwork grant.
Existing single-folder configurations remain readable; extra locations are stored in the optional
`additionalArtworkRoots` field without replacing the primary folder or existing OAuth authorizations.
The CLI also has owner-only development helpers: `remote-configure` accepts the phrase from stdin
and writes a new profile, and `remote-http --config PATH` runs the same OAuth service. These are
used by the external SDK development harness, not required by the end-user wizard.
The CLI requires `--trusted-proxy IP` as an explicit opt-in with a non-loopback `--bind`.

Tests under `remote/oauth`, `remote/server` and `remote/worker` cover authorization, expiry, replay,
revocation, persistence failure, file boundaries and missing mutation replies. UI tests cover
secret masking and refusal of automated settings access. A separate SDK client outside this Rust
repository tests discovery, three client-auth methods, real native editing, previews, save/reopen,
editor restart and service restart, both locally and through the public HTTPS gateway.
OpenAI's hosted client now completes discovery, lists shared folders and inspects the real native
desktop through the public gateway. A TCP timestamp-related stall on this gateway's Windows path
had left the final bytes of proxied requests unacknowledged, despite retransmission. Disabling
`net.ipv4.tcp_timestamps` only in the reverse-proxy container's network namespace resolves the
observed failures. The original Nginx image and HTTP/1.1 proxy settings were restored and retested;
changing the image alone was not a fix. Synthetic fixed/split uploads up to 12 KB, including
additional headers, also pass. The exact underlying driver or middlebox is unidentified.
Timeout diagnostics record byte counts and framing, never bodies or credentials. The HTTP
wrappers advertise only initialization-based 2025-03-26 and
2025-06-18; unsupported versions receive 400 with Connection: close before their body is consumed.
Upstream's partial modern result support does not constitute a complete 2026 HTTP transport.

Protocol references:
- <https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization>
- <https://developers.openai.com/plugins/build/auth>
- <https://developers.google.com/identity/account-linking/oauth-linking>

## Client compatibility notes for the proposal / PR

Local testing on 2026-10-18 distinguishes structured inspection from visual inspection:

- **ChatGPT:** the hosted connection can list shared folders and inspect the desktop. The user
  also reports opening an existing `.ai` document without import warnings, reading its layer tree,
  and receiving a rendered preview whose described visible artwork matches the document's hidden
  and visible layers. The user reports that inspection did not modify the file. This demonstrates
  the connection workflow, not comprehensive import fidelity or visual correctness for all files.
- **Gemini:** the user completed OAuth using credentials generated in the native wizard and
  reports successful file opening and structured inspection of an imported `.ai` document,
  including its layer tree, path geometry, strokes, fills and spot-color information. This is
  inspection of machine-readable document data, not evidence that Gemini viewed the rendered art.
- **Observed image limitation:** when asked to inspect raster artwork, Gemini reported receiving
  only dimensions. Gateway responses for `render_preview` and `screenshot` actually contained
  both standard MCP `type: "image"` blocks with `mimeType: "image/png"` and text metadata.
  Captured images had valid base64 and PNG signatures at 1024 x 576 and 1600 x 900. A smaller
  `render_preview` request with `max_side: 256` likewise returned a valid 256 x 144 PNG, but
  Gemini quoted only the dimensions from the text block and reported no visual access.
- **Scope:** this is an observed compatibility issue in the tested Gemini connection. The server
  response contains the image, but the logs cannot establish how Google's connector passes it
  into the model. Do not claim that all Gemini clients lack image support or that the exact
  Google-side failure is established. A general size limit has not been ruled out conclusively.
- **Raster versus vector:** the successful `.ai` document-tree inspection does not establish a
  raster-only import defect. The server renders both vector and raster artwork as PNG previews.
  Gemini's ability to visually inspect a rendered preview of an `.ai` document is still unverified;
  successful tree inspection must not be described as successful visual inspection.

These notes contain no user artwork, private file names, local account paths or OAuth credentials.

## Original loopback-only transport

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
Requests are capped at 12 MiB; duplicate security headers, unsupported protocol versions and
malformed/batched messages are refused before reaching the editor. Replies use JSON, notifications
receive empty 202 responses, and standalone GET/SSE and DELETE receive 405 responses. No HTTP
session table is allocated. Each request's protocol-version header selects its native revision.

Tool definitions, prompts and resource lists work while the desktop is closed. A desktop operation
then returns an explicit unreachable error; it never edits a separate headless document. The next
explicit operation can reconnect after the user opens the desktop. Requests run serially and are
not automatically replayed when the client disconnects. A missing native reply can mean the edit
completed: inspect the document before explicitly retrying it.

The original loopback transport deliberately omits OAuth, an authorization/settings interface, background
service installation, remote app launching, SSE progress/logging, and HTTP cancellation. It does
not advertise logging support. Long calls can delay other requests; connection/body deadlines,
admission bounds, and concurrency need further work before public use. A bearer token alone does
not implement the MCP OAuth authorization flow. **Do not publish or tunnel this prototype.**

Transport behavior follows the public MCP specification:
<https://modelcontextprotocol.io/specification/2025-06-18/basic/transports>.

Tests in `crates/mcp/src/http/tests.rs` exercise the real loopback HTTP listener, authorization on
every request, Host/Origin checks, metadata fidelity, protocol isolation, framing/limits, desktop
absence and dispatch-without-replay. Existing MCP tests continue to exercise native tool behavior.
