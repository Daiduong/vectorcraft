# VectorCraft browser recovery

Status: Rust implementation under review. No parity score increase is claimed.

Recovery uses Rust futures and web-sys IndexedDB APIs. Controls use egui theme visuals.
There is no handwritten JavaScript, inline JavaScript, npm dependency or DOM stylesheet.
Storage loading never blocks the editor. Every load uses a fresh random writer id so
reloads and duplicated tabs cannot overwrite another page's checkpoint. A reload reads
its previous committed id and a later visit offers the latest checkpoint. Snapshot,
previous checkpoint and discovery pointer commit atomically. Failed saves show Retry.
Restore/import require closing current documents in PhotoCraft and VectorCraft.
LightCraft replaces its loaded catalog on explicit restore. Its original photos remain
in existing content-addressed storage and recovery copies contain references to them.

Checkpoints preserve serializable workspace state and undo/redo history. Capture waits
for inactivity and pointer release. They are bounded to 512 MiB without history truncation.
Hard termination may lose changes since the last completed write. Storage eviction and
clearing site data remove recovery. Full-history capture can be expensive.
The legacy draft's JavaScript database is retained without migration or deletion.
Recovery import/export and persistent-storage requests are explicit user actions.
No fonts, images, external assets, brand changes or launcher code are included.

Validation and outstanding browser/platform checks are recorded in the PR description.
Revision history: 2026-10-12 — replace the noncompliant JavaScript coordinator with Rust.

Recovery operations are exposed through the existing engine command registry as
`session.recovery.capture {ui?}`, `session.recovery.inspect {bytes}` and `session.recovery.restore {bytes}`.
They are available to existing CLI/control/MCP dispatchers. JSON byte-array transport
is capped at 8 MiB. Browser storage uses the same engine serializer through the binary
adapter with a 512 MiB bound. Command restore requires an empty engine session and
returns UI metadata for the calling shell to apply.

## Contribution review and validation

Reviewed the published AGENTS.md rules, including Rust-only product code, checked
inputs, command dispatch, layering, clean-room provenance and asset requirements.
Formatting, strict native Clippy for engine/UI/web (all targets), WebAssembly
compilation and strict WebAssembly Clippy passed. Three native engine integration
tests passed for documents/history, malformed/wrong-app records and recovery command
dispatch. The native web-host architecture regression passed. Layering, asset and
branding checks passed. No fonts, external assets or branding changes are introduced.
Contributor credit remains @mmmbeer as requested. No lint or quality gate was weakened.

Full workspace cargo xtask ci is not certified locally: an earlier all-target compiler
was killed with SIGKILL in this runner. The browser-storage probe was compiled but has
not been executed in a real browser. Cross-browser clipboard/storage lifecycle, visual
UI screenshots, large-history performance and generated parity documentation remain
for pre-merge validation. The referenced private craftrules repository was inaccessible
(404), so unpublished standards cannot be certified from the public instructions alone.
