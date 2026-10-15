# The collaboration behavior suite (VectorCraft's projections)

> The shared suite every Craft app runs before its CraftHub integration is enabled (the CraftHub
> specification §18.9): each row is one automated test with a written scenario, in the family's
> behaviour-suite tradition. This file is VectorCraft's copy of the shared catalog: the scenarios
> are identical in every app, and the *projections* column is where each app writes what the
> scenario means for its own UI.

## The scenarios

| # | Scenario | The guarantee under test | VectorCraft's projection |
|---|----------|--------------------------|-------------------------|
| 1 | Join by code, join by link | One argument that recognizes itself; the file opens in its own tab, other sessions untouched | A joined file opens as its own document window; the tool options and panels stay per-document |
| 2 | Concurrent editing converges | Snapshot-hash equality after arbitrary interleavings, drops and reconnects (per seed) | Node properties merge last-writer-wins per property; a group's children merge as their own objects; the converged render equals the reference render tree |
| 3 | Read-only, with its reason | The reason is shown (view link, role without edit, region lock), never just a badge | The mode bar shows "view link" / "viewer role" / "locked region" with the locked objects tinted |
| 4 | Batch refused with fixes | The server's refusal carries the fixes; the client applies them and lands on the server state | A refused path creation snaps back with a toast explaining why |
| 5 | Per-user undo | My steps are mine alone; later writes by others survive my undo | Undo after a peer's edit undoes only my edit; the peer's path stays where they put it |
| 6 | Offline, then resume as the same client | The recovery copy reattaches; unsent changes replay or land as a "(recovered copy)" beside the file | The unsent anchor drag replays on reconnect; a conflicting copy appears in the Welcome ▸ Online list |
| 7 | Cursor chat and its bounds | ≤100 characters, ephemeral, never enters the document or the journal | The bubble renders at the peer's cursor for its lifetime, then is gone |
| 8 | Spotlight and follow | Follow tracks view and page; any view movement of mine stops the follow | The viewport (scroll and zoom) animates to the presenter's artboard; my own pan breaks the leash instantly |
| 9 | Presence and per-app projections | Ephemeral, throttled to change-only, bounded sizes | The cursor in document coordinates, the artboard (a dot on the Artboards panel; the canvas shows same-artboard peers only), the Layers-panel selection outlines, the viewport, and the type-edit caret |
| 10 | Region locks | Locks sync as document properties; locked regions tint; the reason is shown | Locked objects tint; editing them is refused with "locked by <name>" |
| 11 | Contribution, review, merge | The branch lives inside the live session; merge is one undo step; each side's changes show | The review pane shows both sides' property diffs per object, Inspect-style |
| 12 | Comments and their anchors | Anchor kinds per app; comments are entries with ids and order | VectorCraft anchors a comment to the selected object; the pin rides the node (they ride the document-level lists for the first cut — the per-entry split is the documented follow-up) |
| 13 | Background file notifications | Delivered when the tab is not front; bounded, dismissible | The taskbar badge shows "3 new" for a backgrounded shared file |
| 14 | Closing a file ends its session alone | Other sessions keep syncing behind the front tab | Closing one shared window never drops another file's peers |
| 15 | The revert guard | `file.revert` is refused while its document is in a live session (including reconnect and pre-welcome) | The toast says "leave the session before reverting"; the document and its previews are intact |

## VectorCraft's adapter surface (what the suite drives)

* **Property table** — the 10 synchronized node fields plus the `kind` payload and the `extras`
  object, with the compile-time guard (`vectorcraft-hub::props`). A container's `kind` property
  carries its payload **without its children** — the children sync as their own objects, so an
  edit deep inside a group never re-serializes the subtree (the dense-path rule; the tests pin
  the elision on read and refuse a kind value that smuggles children on write).
* **The tree** — layers under the root, nodes under layers, groups, compounds, blends, envelopes
  and repeats (each container's children its own), ordered by fractional sibling keys kept beside
  the document in the adapter. A node that changes parents is one `parent` write plus the one
  position key it needs in the new parent's order.
* **The dense-path discipline (this row's own measurement)** — moving one node in a path-heavy
  document may touch thousands of anchors, so the adapter is pinned by three batch-size tests: a
  one-field edit on a megabyte document is exactly one small `Set`; a z-order move of one dense
  path re-sends no `kind` value at all (one position key, a few bytes); a geometric edit of one
  dense path re-sends exactly that node's `kind` value — bounded by the node's own size, never
  the parent's, never the siblings', never the document's. The untouched nodes stay `Arc`-shared
  between the two states (the model's copy-on-write spine aside), which is what keeps the next
  diff off them too.
* **The ids** — VectorCraft's node ids are `u64` from one document-wide counter, so the
  protocol's 64-bit client-space ids fit natively: no session translation map (unlike DeckCraft's
  `u32` row). `Document::fix_next_id` keeps the model's counter clear of session ids.
* **The document level** — the setup, artboards, guides, panels and resource lists sync as
  whole-value LWW scalars (the deckcraft row's first-cut shape; the per-entry split for the
  resource lists is the documented follow-up). Embedded images never sync as batch data: image
  **bytes** ride the asset channel (content-addressed, spec §3.5) and the session wiring
  rebuilds the image map as they arrive — the blob's `bytes`/`proxy` are `#[serde(skip)]`
  anyway.
* **Commands** — the unified `hub.*` catalog with the v2 `collab.*` twins
  (`vectorcraft-hub::commands`): same names in every Craft app.
* **Deep links** — `vectorcraft://join?<code\|token>&server=<url>` (`vectorcraft-hub::deeplink`).

## The enable rule

No app is enabled before its protocol core passes its simulation tests in CI for a full week
(nightly, fixed seeds) — and enabling happens through the app's own `crafthub` feature flag,
never through the core, so a regression is one flag flip away from off (§18.4).
