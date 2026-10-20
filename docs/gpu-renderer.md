# Experimental GPU canvas

> **Last updated:** 2026-10-20 · **Status:** native opt-in prototype; broader rendering and platform validation remain open.

Build the desktop app with `cargo build --release --locked -p vectorcraft --features gpu-canvas`.
In Preferences › Performance › GPU Performance, select Canvas Renderer › GPU (Experimental).
CPU remains the default. The existing Graphics Processor preference selects the adapter separately;
turning GPU Performance off also selects CPU canvas rendering.

The GPU compiler accepts paths and compound paths, shared shaped text outlines, decoded images,
linear and radial gradients (including elliptical and focal geometry), clipping groups, centered
plain and dashed strokes, opacity and blend modes. Plain groups preserve paint order; isolated
groups use offscreen composition. CMYK transparency uses the same complemented CMY/K ink planes
and working-profile lookup as the CPU renderer, then converts them on the GPU. Bounded image/vector
pattern tiles reuse the CPU tile cache and repeat as GPU image paints. Color management
and image decoding remain shared CPU preparation work. Aligned image opacity masks reuse the shared decoded images and apply luminance masking on GPU; the editable source image and mask remain separate.

Nested/effect-heavy patterns, meshes, symbols, inline graphics, general masks/knockouts, live effects, complex strokes and
container appearances still select the existing CPU renderer for the **whole frame**. So do
backdrop-dependent non-isolated objects that need an opacity/blend layer (groups whose members
blend, and paths, compound paths or type whose fills or strokes blend), proof/separation,
overprint, outline and other unsupported preview modes. This is an opt-in implementation with
conservative fallback, not full GPU Preview parity.

Canvas dimensions use the host device's texture limit and a 256 MiB combined render-target
budget: two full-size RGBA8 textures for RGB, three for CMYK. Sizes are physical pixels,
including display scaling. These checks run before worker or foreground scene preparation
and before texture allocation; a rejected view uses CPU and can return to GPU after resizing.
The budget covers these targets, not total driver VRAM usage or every backend buffer. Image
residency and scene buffers have separate bounds. Individual image sides remain limited to
4096 pixels by the image upload policy. Other resource limits are 100,000 visited nodes, depth 64, 256 appearance
items per path, one million accumulated path elements, and finite transformed coordinates within
±65,536 pixels. Artboards are clipped to the view and text outside it is culled by its glyph
bounds and stroke reach, so off-screen geometry at deep zoom does not force a CPU frame. Pixel
Preview's document pixels are magnified with nearest filtering, as on the CPU canvas. Text is bounded to 64 KiB and 10,000 runs per object. Image storage is bounded to
64 MiB and 256 distinct images, including pattern tiles and both CMYK planes. The backend's
actual atlas allocator must fit every image before the frame is accepted. A host-side allocator
tracker follows submitted CMY/K scenes in order, retaining the renderer and clean uploads as
images enter, leave and return during panning. Allocator pressure or exceeding the retained-image
budget resets both tracker and renderer; a fresh allocator must still fit every image. Large
image, mip or document changes can therefore cause a warm-up frame.
Opacity/blend layers use painted bounds rather than the whole viewport. A conservative per-tile
command count checks Vello 0.11's fixed buffer before submission; excessively dense frames use CPU.
Unsupported scenes or device limits produce CPU fallback. Validation/allocation errors disable the GPU canvas for the session. Temporary queue or validation
backlogs use a current CPU frame and retry GPU automatically when submissions and scopes drain.
Restart the app to retry after a validation/allocation error. Builds without the feature or a suitable device still use the CPU.

Help › About reports the active canvas renderer; hover its label for the fallback reason.
Preferences labels plug-in folders and scratch disks as **Plug-ins & Scratch Disks**, separately
from **Performance**. The old category name remains accepted by automation.
The control method `ui.inspect` returns `canvasRenderer.active` and `canvasRenderer.reason`.
The same command exposed to MCP/control changes the preference:

```json
{"command":"prefs.set","params":{"key":"canvasRenderer","value":"gpu"}}
```

## Architecture and backend choice

The opt-in `vectorcraft-render/gpu` feature prepares a capability-checked Vello scene. Outside active interactions, preparation runs on the
existing render worker. During a drag or dialog preview it runs in the current UI frame, followed
by GPU submission or CPU fallback, so the artwork and selection outline show the same revision. `vectorcraft-ui-egui/gpu-canvas` renders it using eframe's device and
queue and registers the resulting texture with egui. No second adapter, graphics context or
CPU pixel readback is introduced. Vello outputs straight alpha; an original GPU compute pass converts it to premultiplied RGBA,
or converts two CMYK ink planes through the same 17⁴ XYZ lookup and sRGB transfer as the CPU.
The CMYK table is uploaded again only when its profile generation changes.

The worker retains at most one pending document snapshot and one completed result. The UI
checks document identity, revision, dimensions, renderer choice, fonts, color settings, profiles and plug-in generation before submitting a
prepared GPU scene. During panning alone it can show completed intermediate views using the existing
texture reprojection; edits, zoom/rotation, preview modes and other changed inputs still reject stale frames.
The bridge limits queued submissions, polls errors without waiting for GPU completion and frees
the egui texture registration when dropped. Context replacement clears GPU resources.

Published Vello 0.11.0 was selected for its compute renderer and wgpu 30 compatibility.
Vello GPU 0.3.0 (formerly Hybrid) was also measured during the spike; its CPU path preparation
exceeded the dense-document budget. Only the selected compute backend is included. It is
MIT OR Apache-2.0; the existing `vello_cpu` 0.2 API and CPU export path remain in place.
The application retains its adapter and driver compatibility policy. For hardware QA,
`WGPU_VALIDATION_INDIRECT_CALL=1` enables wgpu's extra indirect-argument checks; the probe
enables those checks explicitly.

## Image opacity masks and opaque knockouts

An enabled opacity mask is supported when its art and the object are plain images with equal
model dimensions and transforms, Clip on, Invert off, and no mask-art opacity, blend or effects.
Linked images moved together reuse both GPU uploads. Editing image or mask content, changing
profile-derived colors or selecting a different mip level refreshes the affected resource.
Disabled masks do not affect rendering. Unlinked masks can use the same path while aligned;
moving one independently selects CPU fallback.

The image is drawn into a bounded layer, then a GPU luminance-mask layer samples the separate
mask texture in display colors. This also applies to CMYK ink planes. Keeping sampling separate
avoids baking image and mask together before filtering; no destructive flattening, viewport-size
CPU mask or pixel readback is introduced. Other mask geometry and Clip/Invert combinations
remain CPU fallback.

A second narrow case accepts flat, full-opacity Normal knockout groups whose children are plain
paths with one opaque Normal solid paint each. Their alpha equals their knockout coverage, so
ordinary source-over composition replaces earlier children by that coverage. Translucent paints,
nested groups and complex knockout semantics still use CPU. Generated opaque crossing strokes,
including dashes, are included in the GPU/CPU comparisons.

## Reproducing the probe

```sh
cargo run --release --locked -p vectorcraft-render --features gpu-probe --example gpu_probe -- 10000 /tmp/vectorcraft-gpu
```

The Linux/Vulkan probe constructs original deterministic artwork, measures scene preparation
and completed GPU work separately, compares premultiplied pixels against the CPU renderer,
and writes CPU and compute PNGs. It also checks overlapping fills, centered strokes,
multiple opacity layers and large document coordinates on an integer-edge fixture; a channel
error greater than 4/255 fails that conformance check. This is a limited regression probe,
not the application's full performance suite or a universal raster-equivalence claim.

The canvas's existing render-time display measures CPU preparation (and submission for
foreground GPU frames); it does not measure GPU completion. Use the probe for completed-work timings. Run
`vectorcraft-cli perf` separately on an idle machine for the standard CPU/export budgets.

## Initial spike evidence (2026-10-17)

Before the current conservative command-buffer guard was introduced, an NVIDIA RTX 3080 using
Vulkan, 2880 × 1800, 50,000 generated solid-fill paths
with object opacity: compute preparation 19.70 ms, completed GPU work including alpha
conversion 6.67 ms, total 26.43 ms; CPU warm median 54.30 ms. The comparison Hybrid
backend took 158.84 ms including preparation. These are medians from a development
machine while builds were active, not idle-machine performance guarantees. Extra indirect
validation was enabled. A single-path compute frame took 0.64 ms versus CPU 3.19 ms.

The integer-edge opacity/stroke/large-coordinate fixture had maximum channel error 2/255.
For the dense scene the mean absolute channel difference was 0.2001/255 and the maximum
61/255, concentrated at antialiased edges; 3,264,401 pixels whose CPU 3 × 3 neighborhood
was flat within 1/255 differed by at most 1/255. This supports testing the prototype, not
claiming identical curve antialiasing or enabling Automatic selection. The final guarded compiler also passes
the densely overlapping 50,000-path probe after its estimate was matched to the backend
instruction sizes; a development build measured 84.33 ms including preparation versus CPU
54.62 ms. Those development timings are not directly comparable to the release spike above.

Native smoke checks passed renderer toggles, GPU Performance off/on, Outline fallback,
unsupported-gradient fallback, rotated and resized views, document changes, an empty
document and rapid edits with undo/redo. GPU and CPU window screenshots were inspected. A follow-up single-rectangle check used an actually
maximized 3840 × 2108 window: both renderer modes kept the held-drag artwork aligned with
its live selection outline after interaction rendering moved into the current frame.
A shader-validation failure also selected CPU without closing the app. Other devices,
platforms and actual device loss have not been tested.

To compare generated text, images, clips, gradients and blending in RGB and CMYK:

```sh
cargo test --locked -p vectorcraft-render --features gpu-probe --lib gpu::tests::hardware_rgb_and_cmyk_match_cpu -- --ignored --nocapture
```

The hardware test requires Vulkan, compares flat interiors separately from antialiased edges,
at 512 × 512, 3840 × 2160 and 5120 × 2880, and writes comparison PNGs to the temporary directory. It is explicitly ignored in ordinary
headless CI. For a locally owned document (read-only input, never added to the repository):

```sh
cargo run --release --locked -p vectorcraft-render --features gpu-probe --example gpu_probe -- --file INPUT /tmp/vectorcraft-file
```

To reproduce a specific native view, supply physical canvas dimensions, document-space center
and physical pixels per document point (native view zoom times display scaling):

```sh
cargo run --release --locked -p vectorcraft-render --features gpu-probe --example gpu_probe -- --file INPUT /tmp/vectorcraft-view --view WIDTH HEIGHT CENTER_X CENTER_Y PIXELS_PER_POINT
```

This mode renders the transparent artwork texture without the UI's paper/artboard background.

## Extension validation (2026-10-18)

On Linux, NVIDIA RTX 3080 / Vulkan, driver 610.57.04, generated fixtures cover all 16 blend
modes, text outlines, a clipped translucent gradient and blend, elliptical focal radial gradient,
an alpha image, repeated image patterns, aligned image masks with color/alpha and CMYK sources, a rotated masked Screen blend, opaque knockout strokes, and dashed strokes. Transparent and opaque backgrounds
pass in RGB and CMYK at 512 × 512, 3840 × 2160 and 5120 × 2880. The 5K case uses the same
geometry transform as 4K to isolate target size and row-stride changes. Flat 3 × 3 interior neighborhoods differ by
at most 2/255 in RGB and 3/255 in CMYK. Mean absolute channel errors are 0.0250–0.1461/255;
whole-image maxima are 35–70/255 at antialiased edges. Comparison images were inspected.
Resource tests cover changed image bytes invalidating their pattern tile, excess image counts,
excess command load, unsupported groups, offscreen culling (including far-off text and artboards at deep zoom) and malicious dash/paint values. Mask tests verify stable uploads during linked moves, changed mask bytes, unsupported mask semantics, disabled masks and strict knockout eligibility.
Atlas tests cover images entering, leaving and returning without a renderer reset, changed
dimensions and the cumulative residency budget; the hardware comparisons use the persistent tracker.
Native X11 checks on an isolated virtual display with the RTX 3080/Vulkan adapter exercised
CPU/GPU drags of a rectangle and an aligned masked image at a 4314 × 2544 canvas size. Held
frames were inspected and Undo restored the saved artwork after each gesture. These checks
establish interaction behavior, rather than virtual-display latency.

A locally owned CMYK scientific illustration with text, images, clips and image patterns also
renders on GPU when fitted to its first artboard at 4K. Its mean absolute channel difference was
0.0207/255, with a maximum of 3/255 across 7,685,107 flat interior pixels; the 100/255 maximum
occurred at thin antialiased edges. The original file and derived images remain local. Other
views may encounter unsupported artwork and fall back to CPU. A final release probe on the same
machine (load average 1.53, 16 logical CPUs) measured 80.52 ms warm CPU versus 3.18 ms preparation
+ 1.66 ms completed GPU work, 5.01 ms median total, for that first-board view at 3840 × 2160.
This is a headless per-view measurement, not end-to-end interaction latency or a promise for all
artboards. Native renderer toggles, GPU Performance off/on, Outline fallback and rotation were
checked in an isolated window with this file. These checks do not establish
appearance parity for every imported document or color profile.

After adding aligned image masks and the opaque-path knockout subset, a wider view of the
same local document also runs on GPU at 4314 × 2544. After upstream integration and the pan fixes,
the final release probe at the same center and scale measured 437.19 ms warm CPU versus 3.97 ms
preparation and 2.10 ms completed GPU work, 6.08 ms median total. Mean absolute channel difference was 0.0223/255, with a maximum of
75/255; the CPU and GPU captures were inspected. This measures that view on the NVIDIA RTX
3080/Vulkan machine, rather than end-to-end input latency. The maximized native preview also
reported GPU active without a fallback reason for the same view.

The final standard CPU `vectorcraft-cli perf` run passed nine of ten budgets. The 400% zoom
case took 37.91 ms against a 16 ms budget; an earlier run also exceeded it at 31.27 ms.
Fit-page rendering and background pan refresh took 88.81 ms and 89.48 ms against 100 ms budgets.
This GPU extension does not close the CPU performance gap.

## Remaining work

General masks/knockouts, effects, nested/complex patterns, meshes, symbols, inline graphics, advanced strokes and additional
non-isolated appearance semantics remain on CPU. Spatial indexing, dirty-region rendering, platform
coverage on macOS/Windows/integrated GPUs, device-loss injection, browser support and Automatic
selection remain open. Standard CPU exports, printing and headless MCP use their existing renderer.

## Revision history

| Date | Change |
|---|---|
| 2026-10-20 | Fall back to CPU for leaf objects whose fills or strokes blend under an object opacity/blend layer, as for groups (review on #2993). |
| 2026-10-18 | Clip artboards to the view and cull off-screen text by fresh glyph bounds, so deep zoom stays on GPU; use nearest filtering for Pixel Preview. |
| 2026-10-18 | Reuse renderer pipelines and checked image-atlas allocations while panning; accept compatible intermediate pan frames. Distinguish Plug-ins & Scratch Disks from Performance in Preferences. |
| 2026-10-18 | Add cached aligned image-mask resources with GPU luminance composition and a strict opaque-path knockout subset; preserve CPU fallback for other semantics. |
| 2026-10-18 | Replace the 4096-side canvas cap with device-aware dimensions and a combined 256 MiB render-target budget; add 5K hardware comparisons and budget/fallback checks. |
| 2026-10-18 | Extend shared preparation to text, images, gradients, clipping and CMYK; bound image residency and invalidate prepared frames on resource changes. |
| 2026-10-17 | Document the native opt-in subset, fallback, resource limits, backend choice and reproducible probe. |
