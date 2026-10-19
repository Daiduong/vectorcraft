# Native CorelDRAW import

CDR.1 adds read-only `.cdr` import to the shared engine. File > Open, File > Place,
drag and drop, the CLI, MCP and the browser use the same Rust reader. Open creates
editable paths, artboards, layers and groups. Save asks for a new VectorCraft
destination; the CDR source is never overwritten. There is no CDR exporter.

This is partial interoperability. Unsupported art is reported in the import
warnings. The reader does not substitute an embedded thumbnail or a rendered
preview for editable objects.

## Containers and version coverage

CDR is a family of binary formats. The reader handles little-endian RIFF records,
their `LIST` hierarchy, compressed `cmpr` lists, and ZIP packages. X4/X5 packages
carry `content/riffData.cdr`; newer packages carry `content/root.dat` with records
that reference the streams named by `content/dataFileList.dat`. Compressed RIFF
lists have two zlib streams: the record data and a table of record lengths.

The object argument tables refer to native geometry, transforms and fill/outline
styles. Those records are decoded into the document model, without launching a
converter or requiring CorelDRAW on the importing machine. The implementation
and its public source references are in [the CDR crate](../crates/cdr/README.md).

| File layout | Evidence |
|---|---|
| CDR 12 RIFF | Original, generated record fixture in unit and engine tests |
| CDR 15 ZIP | Original rectangle and two-page geometry files saved by CorelDRAW 27.2 |
| CDR 16 ZIP with external streams | The same original scenes saved by CorelDRAW 27.2 |
| CDR 27 ZIP with external streams | Original scenes saved by CorelDRAW 27.2, including its extended version and fill records |
| Other accepted versions between 6 and 27 | Experimental record-layout coverage; not a claim of full fidelity or a verified historical application version |
| Waldo (CDR 1/2), CMX, encrypted packages, ZIP64, newer unknown versions | Unsupported; returned as errors |

The installed CorelDRAW 27.2 did not produce files for requested save versions
6, 10, 12, 13 or 14, despite reporting no COM error. These attempts do not verify
older versions. No claim of universal CDR support is made.

## Verification and remaining work

The local test artwork is original: a red rectangle with a black outline, an
ellipse, a cubic curve, grouped objects, a rotated rectangle and two pages.
CorelDRAW independently exports each page to SVG. The engine test renders both
the native CDR import and that SVG and compares their dimensions and pixels.
The fixtures and application resources remain local; no Corel fonts, profiles,
vendor artwork or binaries are redistributed.

Corel-specific effects, text, bitmaps, PowerClip and other unsupported records
remain outside the initial import subset. Warnings state what was omitted or
approximated; see the crate README for the exact implemented paint and geometry
coverage. Export SVG or PDF from CorelDRAW when a file depends on those features.
Editable linear and radial fountain fills are also imported; advanced placement and
colour interpolation can be approximate. The feature and format parity scores
remain unchanged.

The parser enforces limits on input bytes, expanded data, copied record payloads,
record and point counts, nesting, pages and coordinates. ZIP CRCs and declared
sizes are checked. Unit/property tests cover truncated, mutated and hostile
records; the engine fuzz tests also open, place, render and export successful
imports.

Run the synthetic checks with `cargo test -p vectorcraft-cdr` and
`cargo test -p vectorcraft-engine cdr`. To run the independent application-oracle
checks, set `CDR_FIXTURE_DIR` to a local folder containing
`rectangle-v15.cdr`, `rectangle-v16.cdr`, `rectangle-v27.cdr`,
`geometry-v15.cdr`, `geometry-v16.cdr`, `geometry-v27.cdr`,
`rectangle-page1.svg`, `geometry-page1.svg` and `geometry-page2.svg`, then run
`cargo test -p vectorcraft-engine --test cdr_files -- --nocapture`.
`CDR_RENDER_DIR` optionally saves the compared PNGs outside the repository.
When also present, `gradient-v27.cdr` / `gradient-page1.svg` and
`group-transform-v27.cdr` / `group-transform-page1.svg` extend the render checks
to fountains and nonuniformly transformed, rotated groups.

Use a 100 by 100 mm page, with the rectangle at drawing coordinates
(10, 60) mm, 40 by 20 mm, red RGB fill and a 1 mm black outline. The geometry
scene adds a grouped blue ellipse, a green cubic curve and a rotated rectangle on
page 2. Save each scene in the named versions and export each page as SVG 1.1,
millimetres, embedded images and presentation attributes. The initial fixtures
inherit Corel's drawing origin: its SVG and native records both include a
roughly 0.038 mm origin offset. Compare against the exported page, rather than
assuming the UI's drawing coordinates are the page coordinates.

## Revision history

| Date | Change |
|---|---|
| 2026-10-19 | Initial bounded native import and original CorelDRAW application-oracle validation. |
