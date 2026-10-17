# AI-assisted layer naming

Desktop: open the Layers panel menu → **Name Layers with AI…**. Enter the name of an installed
Ollama vision model and choose the output language, whole document or highlighted rows/selection,
and group depth (1–16). Click **Generate Names**, review/edit the proposals, then **Apply Names**.
Settings are retained with UI preferences. Existing names are replaced only after Apply; Cancel
leaves the document unchanged. One Undo restores all names together.

The server defaults to `http://localhost:11434/api/chat`. Ollama and an image-capable model must be
installed separately; VectorCraft does not download a model. Desktop networking uses `curl` from
PATH and reports an actionable error if it is absent. Web builds display an unsupported message.
A generation sends artwork previews to the configured server, only after Generate is clicked.
A remote server receives the same previews. There is no API-key setting.

Empty containers are skipped and keep their existing names.

Each request contains the whole-artwork preview plus a 4-column contact sheet of up to 12 groups,
and a JSON list of their IDs, parents and existing names. The server is asked for names in the
chosen language. The implementation follows the [Ollama chat API](https://docs.ollama.com/api/chat):
base64 PNG images, `stream: false`, `format: "json"`, response in `message.content`.

Output is untrusted. The app requires exactly one bounded, nonblank name for every requested ID;
malformed JSON, extra/duplicate IDs, missing names and control characters are rejected. Requests time
out after 120 seconds (10 seconds to connect), responses are limited to 1 MiB, and candidate lists
to 1000 groups. Narrow the scope or depth for larger documents. Cancel stops further batches;
an in-flight request may finish before its timeout, and its result is discarded.

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

Validation on 2026-10-17: all ten `cargo xtask ci` steps passed (fmt, Clippy, workspace tests,
no-bundled-fonts, PDF dependencies, assets, brands, cursors, layering and WASM). Settings and review
screens were rendered and inspected with the egui test harness. Real-model semantic quality remains
unmeasured.
