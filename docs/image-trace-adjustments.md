# Adjusting an Image Trace

Select a placed raster image, then choose **Image Trace** or a preset from its arrow
in Properties or the Control bar. The original image stays inside the tracing object so
that subsequent adjustments always use the original pixels.

To adjust the result, select the tracing object and click **Image Trace** in Properties or the
image icon in the Control bar: the Image Trace panel opens with **Advanced** expanded. An existing floating or docked panel is activated in
its current location. Opening the controls does not add a document undo step.

| Control | Range | Effect |
|---|---|---|
| Threshold (Black and White) | 0–255 | Higher values include lighter pixels as black ink. |
| Paths | 0–100% | Higher values fit outlines more closely to the image (see below). |
| Corners | 0–100% | Higher values turn more sharp bends into corner points (see below). |
| Noise | 1–100 px | An area in pixels: regions smaller than it are left out; higher values remove more small details (see below). |
| Method | Abutting / Overlapping | Chooses shared boundaries or stacked overlapping paths. |
| Create | Fills / Strokes | Creates filled areas, stroked lines, or both. |
| Ignore Color | Color picker | Leaves out the selected color (white by default). |

Sliders apply on release, and their numeric fields can be edited directly. Each committed
adjustment re-traces the selected object as one undo step and changes the preset to Custom.
While dragging, the preset and Enhanced row stay in place until release so the
slider retains its active interaction. Committed settings are synchronized immediately,
so Undo also restores the panel values before its next frame. View can show the source image, the result, outlines, or combinations without re-tracing.
Use **Expand** when finished to turn the result into ordinary editable vector objects;
after expanding, the tracing settings are no longer available. **Release** restores the source.

These controls use VectorCraft's existing tracing algorithm. Parameter effects follow its
implementation; they do not claim pixel-identical results with another application.
The terminology and general control behavior are described in the
[public Image Trace panel documentation](https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/image-trace-panel-options.html).

## Paths, Corners and Noise: documented meaning and VectorCraft's mapping

The public documentation describes what each control means, not the numbers behind it. VectorCraft
maps each control so that every documented property holds, and tests each one over every value.

| Control | Documented meaning (sources below) | VectorCraft's mapping |
|---|---|---|
| Paths | Sets the distance between the traced shape and the pixel shape: lower is a looser fit, higher a tighter one [1]. The scripting property `pathFitting` is the same rule seen from the tolerance side: a distance, lower is tighter [3]. | Paths sets the farthest a fitted curve may stray from the edge's points (anti-aliased edges): 0.495 px at 0, 0.31 px at 50, 0.08 px at 100, falling strictly with every step, `0.08 + 0.415 × (1 − Paths/100)^0.85` px. Paths 100 is the tightest fit the tracer makes. Hard (not anti-aliased) edges are known only to about half a pixel and get 2.5 times that. |
| Corners | Sets the emphasis on corners: how likely a sharp bend is to become a corner point; higher gives more corners [1]. The scripting property `cornerAngle` is the sharpness, in degrees, of a turn that counts as a corner [3]. | Corners sets the turn that makes a corner: 99.2° at 0 (only bends sharper than a right angle), 57.9° at 75, 35° at 100, falling strictly with every step, `99.2 − 64.2 × (Corners/100)^1.53` degrees. |
| Noise | An area in pixels that is ignored while tracing; higher gives less noise; suggested 1–10 for low-resolution and 20–50 for high-resolution images [1]. The scripting property `minArea` is the smallest feature traced, in square pixels [3]. | Noise is an area: a 4-connected region of fewer than Noise pixels (of the image as traced) is merged into its neighbour, whatever its shape (a 1 × 16 bar and a 4 × 4 block go at the same value); a region of exactly Noise pixels stays; Noise 0 and 1 leave every region. Unchanged by the Paths and Corners mapping. |

Where the documentation is silent (the tolerance in pixels, the corner angle in degrees, the curve
between the ends), the numbers are VectorCraft's own choices, tuned on code-drawn and public-domain test
images: they are not Illustrator's values and do not claim to match its output. Paths 0 and 50 and
Corners 75 keep the tuned values; Paths 100 stays the tightest fit the tracer has (0.08 px) and Corners
100 the most corners (35°), so that the top of each slider means what the documentation says it means.
Flat Logo mode keeps its own mapping (Paths 0.24 px at 0 to 0.04 px at 100, Corners as before), which
already has these properties.

Defaults and preset values are not published. VectorCraft's Default (Black and White, Threshold 128,
Paths 50, Corners 75, Noise 25) and its presets are its own choices; the Black and White default with
Threshold 128 matches commonly published tutorials ([4], [5]).

Tests (`crates/trace/src/tests.rs`, code-drawn images only):
- `paths_and_corners_map_strictly_and_paths_100_is_the_tightest_fit`: for every integer 0–100, a
  higher Paths gives a strictly smaller tolerance and a higher Corners a strictly smaller corner angle;
  each control acts alone; the ends are 0.08 / 0.495 px and 35° / 99.2°.
- `paths_100_gives_the_most_anchors_and_paths_0_the_fewest`: on soft and hard discs and wavy and bumpy
  blobs, no Paths value 0–100 gives more anchors or segments than Paths 100, and Paths 0 no more
  segments than Paths 100 (VectorCraft's previous mapping failed this: 23 segments at Paths 0 and 22
  at Paths 100 on the soft wavy blob).
- `a_higher_paths_never_gives_fewer_anchors_on_these_shapes`: on the discs and the hard bumpy blob,
  anchors and segments never fall from one Paths value to the next. This cannot be guaranteed for
  every shape: the curve fitter is greedy, and on the soft wavy blob a looser fit can split a long
  span where a tighter one does not (17 segments at Paths 0, 11 at 50, 22 at 100; 23, 13 and 22 with
  the previous mapping).
- `paths_100_strays_least_from_the_drawn_outline`: on anti-aliased shapes (whose pixel shape is the
  drawn outline), no Paths value 0–100 strays farther from it than Paths 100 does, and the mean
  distance falls from Paths 0 to 50 to 100. The largest distance does not fall at every single step,
  for the same reason as above.
- `a_higher_corners_never_gives_fewer_corner_points`: on regular polygons of 3 to 12 sides, corner
  points never fall over Corners 0–100 (at Paths 25, 50 and 75), and Corners 100 has more than 0.
- `noise_is_an_area_in_pixels_and_more_noise_never_keeps_more`: bars and blocks of 1 to 120 pixels,
  Noise 0–120: exactly the regions of at least Noise pixels are kept, and never more at a higher Noise.

Before and after on code-drawn shapes are in [image-trace-quality.md](image-trace-quality.md#paths-and-corners-as-documented).

Sources (public documentation; feature names and short phrases only):
1. Image Trace panel options: https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/image-trace-panel-options.html
2. Image Trace: https://helpx.adobe.com/illustrator/using/image-trace.html
3. Scripting reference, TracingOptions (`pathFitting`, `cornerAngle`, `minArea`): https://ai-scripting.docsforadobe.dev/jsobjref/TracingOptions/
4. https://designbuckle.com/how-to-trace-an-image-in-illustrator
5. https://illustratorhow.com/image-trace

Regression coverage: `crates/ui-egui/src/panels/image_trace.rs` tests opening the panel,
a quick trace leaving it as it was, repeated opening, floating ownership, slider commits, visible threshold
changes, undo, and the Properties entry point. Tracer tests live in `crates/trace/src/tests.rs`.
