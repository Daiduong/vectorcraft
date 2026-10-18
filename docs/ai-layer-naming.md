# AI-assisted layer naming

Desktop: open the Layers panel menu → **Name Layers with AI…**. Enter the name of an installed
Ollama vision model and choose the output language, whole document or highlighted rows/selection,
and group depth (1–16). Use **Play** to start or continue and **Pause** to suspend the queue after the
current group. Ready names appear as each request completes; the progress bar shows completed/total with moving diagonal stripes inside the completed portion while requests are running, including a small active cap during the first request. Pausing freezes the stripes; completion leaves a solid full bar. Explicit running, paused and complete labels distinguish the job state.
Review/edit them and choose **Apply Ready Names** to apply the completed portion in one undo step.
There is no Stop button. **Start Over** discards the current suggestions and checkpoint, without
undoing names already applied. Settings are retained with UI preferences.

Pausing retains ready names. Closing the dialog also keeps completed work in memory for the current
document during this app session; it does not apply any uncommitted names. Reopen the dialog and
press Play to skip completed groups. Applying a partial batch updates the checkpoint revision, so
Play can continue from the first unfinished group after applying, for example, 100 of 500 names.
Each applied batch has its own undo step. Checkpoints do not survive an app restart. Editing the
document invalidates its checkpoint; changing the naming scope or depth requires Start Over.
If a request fails, ready suggestions remain available to apply or continue.

The server defaults to `http://localhost:11434/api/chat` and the vision model to `qwen2.5vl:3b`.
The server URL, model and name language are one shared Ollama connection, so every AI dialog opens
on the same model. Plain `http://` is accepted only for a loopback host (`localhost`, `127.0.0.0/8`,
or `::1`). A remote server must use `https://`. Ollama and an image-capable model must be installed
separately; VectorCraft does not download a model.
Desktop networking uses `curl` from PATH and reports an actionable error if it is absent. Web
builds display an unsupported message. A generation sends artwork previews to the configured
server, only after Play is clicked. A remote server receives the same previews, and only over
HTTPS. There is no API-key setting.

Empty containers are skipped and keep their existing names.

Each request contains one 512-pixel preview of one target group, fitted on white, and its numeric ID. This avoids small vision models
assigning button text from the overview or another contact-sheet tile to an unrelated group.
Existing names stay in the review UI and are omitted from the model prompt to prevent copying
technical placeholders. The dialog shows processed/total groups while generation runs.

The implementation follows the [Ollama chat API](https://docs.ollama.com/api/chat): base64 PNG,
`stream: false`, a strict JSON schema in `format`, response in `message.content`. The schema requires
a `names` array with exactly one bounded name and the exact target ID. The prompt repeats the schema,
temperature is zero and the context window is 8192 tokens. Ordinary JSON mode alone cannot guarantee
this structure; see [structured outputs](https://docs.ollama.com/capabilities/structured-outputs).

Output is untrusted. The app requires exactly one bounded, nonblank name for every requested ID;
malformed JSON, extra/duplicate IDs, missing names and control characters are rejected. Requests time
out after 120 seconds (10 seconds to connect), responses are limited to 1 MiB, and candidate lists
to 1000 groups. Narrow the scope or depth for larger documents. Closing the dialog or choosing Start Over stops further requests; an in-flight request may finish
before its timeout, and its later result is discarded. Pausing lets that request finish and keeps
its result, then sends no further requests until Play.

Applying names checks the document UID and revision captured before generation. If the document
changed, generate again. It modifies only the internal layer/group names: tree order, node IDs,
geometry, paint, clips and resources stay intact. The existing SVG exporter handles names and XML
escaping; this feature does not rewrite arbitrary SVG source in place or convert groups into layers.

Automation:

- `ui.nameLayers {}` opens the settings/review dialog.
- `layer.namingCandidates {scope: "all" | "selection", depth: 1..16}` returns `{uid, revision, rows}`.
- `layer.applyNames {uid, revision, names: [{id, name}]}` applies a reviewed mapping in one history step.

Validation covers read-only review/cancellation, atomic rejection, stale revisions, Undo/Redo,
selection/depth, unchanged rendered pixels, SVG name round trips, malformed model output, a local
mock Ollama HTTP exchange including real rendered previews, and headless dialog layout. A real
model is required separately to assess semantic naming quality; protocol tests do not measure it.

Initial implementation validation on 2026-10-17: all ten `cargo xtask ci` steps passed (fmt, Clippy, workspace tests,
no-bundled-fonts, PDF dependencies, assets, brands, cursors, layering and WASM). Settings and review
screens were rendered and inspected with the egui test harness. Real-model semantic quality remains
unmeasured.

Play/Pause follow-up (2026-10-17): seven focused UI tests passed, including a controlled HTTP
request paused in flight, no subsequent request while paused, continuation skipping completed IDs,
partial apply/reopen, preservation of user edits, close/reopen with the latest completed batch,
and stale-document recovery. Settings, ready-name review and paused-player screenshots were rendered
and inspected with the headless egui GPU harness.

Ollama compatibility regression (2026-10-17): ordinary JSON mode allowed responses without `names`,
and small models copied supplied placeholder names or confused targets in a contact sheet. Strict
schema decoding, one isolated image per request and omission of existing names address those cases.
The opt-in `real_ollama_returns_the_required_names_array` test passed with local `qwen2.5vl:3b`
on four groups from a user-provided SVG (used locally only); the proposals were inspected. This is a
limited live compatibility check, not a semantic accuracy benchmark. Set `VECTORCRAFT_NAMING_SVG`
to a local SVG to repeat it, or omit that variable to use synthetic rectangles:

```sh
VECTORCRAFT_NAMING_OLLAMA_MODEL=qwen2.5vl:3b cargo test -p vectorcraft-ui-egui real_ollama_returns_the_required_names_array --lib -- --ignored --nocapture
```
