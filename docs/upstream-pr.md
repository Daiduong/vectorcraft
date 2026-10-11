A browser gesture can arrive as a key and a clipboard payload in separate frames. This ports the existing *craft routing so one gesture causes one supported edit, publishes editable SVG text plus PNG, imports external text instead of stale artwork, preserves focused text controls and cancels asynchronous paste when the document or field selection changed.

Browser recovery checkpoints include new/unsaved documents, ordered tabs, active document, both history stacks, embedded image bytes, selections, tool/view state and preferences. Existing legacy browser recovery is retained for import while competing writes are disabled.

Addresses the browser portion of #1209 and the local recovery foundation of #1114. Related to #954, but this PR does not change native macOS clipboard readers or claim the MacBook desktop report is resolved. Cloud storage, version timelines, collaboration and object merging remain outside this change.

Validation so far: nine standalone storage tests passed for this app at the root URL; the common harness extracting the actual rebased production clipboard routers/PNG decoder passed eight tests. All four rebased applications passed local WebAssembly checks. Native engine history round-trip tests are still running and are not yet claimed passed. No GitHub Actions build was requested.

This is a draft port of existing code, rebased onto upstream main as captured by the fork. Launcher, shared fonts, branding removals and hosting changes are excluded. See docs/browser-session-recovery.md for storage limits, migration and testing details. The JavaScript IndexedDB/lifecycle coordinator is an explicit architecture proposal for review against the Rust-only repository preference.

Pending before merge: repository-wide quality checks and real Chromium/Firefox/WebKit clipboard permission, menu/keyboard, text focus, stale destination, refresh/return, duplicated-tab and large-history review. Compilation and fake-DOM storage tests do not establish complete browser behavior. AI assistance was used to port and review the changes.
