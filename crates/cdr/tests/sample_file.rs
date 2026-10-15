//! A real CorelDRAW X4 document. User-owned sample files are used locally and never committed
//! (AGENTS.md › Clean-room), so the test reads the file named by `CDR_SAMPLE`, or by default
//! `2019placement.cdr` in the libcdr checkout next to this repository, and skips when it is absent.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use cdr::riff::{self, CdrVersion, Chunk, FourCc, StreamId};
use cdr::{Break, Document, Limits, Package, RiffKind};

fn sample() -> Option<Vec<u8>> {
    let path = std::env::var_os("CDR_SAMPLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../libcdr/2019placement.cdr"));
    match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            eprintln!("skipping: no CorelDRAW sample at {} ({e}); set CDR_SAMPLE", path.display());
            None
        }
    }
}

#[test]
fn x4_sample_package_metadata_and_layers() {
    let Some(bytes) = sample() else { return };
    let doc = Document::open(&bytes, Limits::default()).unwrap();

    // Package layout: an X4 file keeps its whole drawing in one RIFF stream.
    let riff = doc.layout.riff.as_ref().unwrap();
    assert_eq!((riff.path.as_str(), riff.kind), ("content/riffData.cdr", RiffKind::RiffData));
    assert!(doc.layout.data_files.is_empty());
    assert_eq!(doc.layout.metadata_xml.as_deref(), Some("metadata/metadata.xml"));
    assert_eq!(doc.layout.textinfo_xml.as_deref(), Some("metadata/textinfo.xml"));
    assert_eq!(doc.layout.entries.len(), 6);

    // The RIFF header and the XMP agree on the version.
    assert_eq!(doc.version, Some(1400));
    let m = doc.metadata.as_ref().unwrap();
    assert_eq!(m.app_version, Some(1400));
    assert_eq!(m.product_name.as_deref(), Some("CorelDRAW X4"));
    assert!(m.author.is_some() && m.title.is_none());
    assert_eq!(m.language, Some(1033));

    // A US Letter page, 612 × 792 pt.
    let page = m.page.as_ref().unwrap();
    assert_eq!(page.size_name.as_deref(), Some("Letter"));
    assert_eq!(page.dimensions.as_deref(), Some("8.5 x 11 in"));
    assert!((page.width_pt() - 612.0).abs() < 1e-6 && (page.height_pt() - 792.0).abs() < 1e-6);
    assert_eq!(m.resolution, Some((300, 300)));
    assert!(m.clip_rect.is_some());

    assert_eq!(m.fonts_used, ["Arial", "Arial Unicode MS", "Wingdings"]);
    assert_eq!(m.styles, ["Default Graphic", "Default Artistic Text"]);
    assert_eq!(m.object_counts.get("Total"), Some(&17));
    assert_eq!(m.object_counts.get("Text"), Some(&7));
    assert_eq!(m.object_counts.get("Dimension/Linar"), Some(&0));
    assert_eq!(m.lists.get("Fills/Colors").unwrap(), &["CMYK", "RGB"]);

    // Layers and pages.
    assert_eq!((doc.page_count(), doc.layer_count()), (Some(1), Some(1)));
    assert_eq!(doc.layer_names(), ["Layer 1"]);
    let t = doc.text_info.as_ref().unwrap();
    assert!(t.page_names.is_empty());
    assert_eq!(t.object_names.len(), 1);

    // One stream per artistic text object; each starts a paragraph and continues with line breaks.
    assert_eq!(t.text_streams.len(), m.object_counts.get("Text").copied().unwrap() as usize);
    assert_eq!(t.text_streams.iter().map(|s| s.runs.len()).sum::<usize>(), 20);
    for s in &t.text_streams {
        assert_eq!(s.runs[0].brk, Break::Paragraph);
        assert!(s.runs[1..].iter().all(|r| r.brk == Break::Line));
        assert!(s.runs.iter().all(|r| r.lang == Some(1033) && !r.text.is_empty()));
    }
}

#[test]
fn x4_sample_truncations_fail_cleanly() {
    let Some(bytes) = sample() else { return };
    for len in (0..bytes.len()).step_by(997) {
        let _ = Document::open(&bytes[..len], Limits::default());
    }
}

#[test]
fn x4_sample_riff_tree() {
    let Some(bytes) = sample() else { return };
    let doc = Document::open(&bytes, Limits::default()).unwrap();
    let meta = doc.metadata.unwrap();
    let mut package = Package::open(&bytes, Limits::default()).unwrap();
    let stream = package.read_capped("content/riffData.cdr", 64 << 20).unwrap();
    let tree = riff::parse(&stream, &Limits::default()).unwrap();
    assert!(tree.warnings.is_empty(), "{:?}", tree.warnings);
    assert_eq!(tree.version, CdrVersion(1400));

    // One root RIFF of form CDRE: a version record, then two compressed lists (bitmaps, document).
    let [Chunk::List(root)] = tree.chunks.as_slice() else { panic!("{:?}", tree.chunks.len()) };
    assert_eq!((root.header.fourcc, root.list_type), (FourCc::RIFF, FourCc(*b"CDRE")));
    let kinds: Vec<String> = root.children.iter().map(|c| c.kind().as_text()).collect();
    assert_eq!(kinds, ["vrsn", "cmpr", "cmpr"]);

    // Each cmpr list inflates to its declared size into its own stream, and its chunks' length
    // fields are block-table indices.
    let cmprs: Vec<_> = tree.lists(FourCc::CMPR).collect();
    for (n, list) in cmprs.iter().enumerate() {
        let c = list.compression.as_ref().unwrap();
        assert_eq!(c.stream, StreamId(n + 1));
        assert_eq!(tree.stream(c.stream).map(<[u8]>::len), Some(c.uncompressed_size as usize));
        let first = list.children[0].header();
        assert_eq!((first.raw_length, first.length), (0, c.block_lengths[0]));
    }
    let doc_list = &cmprs[1].children;
    assert_eq!(doc_list.iter().map(|c| c.kind().as_text()).collect::<Vec<_>>(), ["doc ", "page", "page"]);

    // Records agree with the counts metadata.xml gives.
    let count = |k: &str| meta.object_counts.get(k).copied().unwrap() as usize;
    assert_eq!(tree.lists(FourCc::OBJECT).count(), count("Total"));
    assert_eq!(tree.records(FourCc(*b"txsm")).count(), count("Text"));
    assert_eq!(tree.records(FourCc(*b"bmp ")).count(), count("Bitmap"));
    assert_eq!(tree.records(FourCc::STLT).count(), 1, "stlt is a record from version 7");
    assert!(tree.lists(FourCc::LAYER).count() >= meta.layer_count.unwrap() as usize);

    // Every record's data lies inside its stream.
    for (_, c) in tree.walk() {
        if let Chunk::Record(r) = c {
            // A LIST read as a record (stlt) starts after its 4-byte list type.
            let list_type = if r.header.fourcc == FourCc::LIST { 4 } else { 0 };
            assert_eq!(tree.data(r).map(<[u8]>::len), Some(r.header.length as usize - list_type), "{}", r.kind);
        }
    }
}

#[test]
fn x4_sample_riff_truncations_fail_cleanly() {
    let Some(bytes) = sample() else { return };
    let mut package = Package::open(&bytes, Limits::default()).unwrap();
    let stream = package.read_capped("content/riffData.cdr", 64 << 20).unwrap();
    for len in (0..stream.len()).step_by(211) {
        let _ = riff::parse(&stream[..len], &Limits::default());
    }
}

#[test]
fn x4_sample_geometry() {
    use cdr::geometry::{self, PageSpace};
    use vectorcraft_doc::appearance::{LineCap, LineJoin};
    let Some(bytes) = sample() else { return };
    let meta = Document::open(&bytes, Limits::default()).unwrap().metadata.unwrap();
    let mut package = Package::open(&bytes, Limits::default()).unwrap();
    let stream = package.read_capped("content/riffData.cdr", 64 << 20).unwrap();
    let tree = riff::parse(&stream, &Limits::default()).unwrap();
    let page = meta.page.as_ref().map(PageSpace::from_metadata);
    let t = geometry::translate(&tree, page);
    assert!(t.errors.is_empty(), "{:?}", t.errors);

    // Every object is accounted for: the Bézier ones translated, the rest counted by kind,
    // matching metadata.xml's counts.
    let count = |k: &str| meta.object_counts.get(k).copied().unwrap() as usize;
    let skipped = |k: &str| t.skipped.get(k).copied().unwrap_or(0);
    assert_eq!(skipped("Rectangle"), count("Rect"));
    assert_eq!(skipped("ArtisticText"), count("Text"));
    assert_eq!(skipped("Bitmap"), count("Bitmap"));
    let shapes: Vec<_> = t.pages.iter().flat_map(|p| &p.shapes).collect();
    assert_eq!(shapes.len(), count("Curve"));

    // A master page with no objects, then the Letter page.
    assert_eq!(t.pages.iter().map(|p| (p.index, p.shapes.len())).collect::<Vec<_>>(), [(0, 0), (1, 2)]);
    assert_eq!(t.pages[1].page.size_pt(), (612.0, 792.0));

    // Nodes: the curves' anchors plus four per rectangle are CurveInfo/Nodes.
    let anchors: Vec<usize> = shapes.iter().map(|s| s.path.anchor_count()).collect();
    assert_eq!(anchors, [4, 2]);
    let nodes: usize = meta.fields.get("CurveInfo/Nodes").unwrap().parse().unwrap();
    assert_eq!(anchors.iter().sum::<usize>() + 4 * count("Rect"), nodes);

    // Placement: inside the drawing's extent (ClipRect, in 0.1 µm, y up from the page centre), and
    // where CorelDRAW's own page thumbnail shows the two arrows.
    let [left, top, right, bottom] = meta.clip_rect.unwrap().map(|v| v as f64 * 72.0 / 254_000.0);
    let (w, h) = (612.0, 792.0);
    let extent = vectorcraft_geom::Rect::new(left + w / 2.0, h / 2.0 - top, right + w / 2.0, h / 2.0 - bottom);
    let near = |a: f64, b: f64| (a - b).abs() < 0.1;
    let b0 = shapes[0].path.bounds().unwrap();
    let b1 = shapes[1].path.bounds().unwrap();
    for b in [b0, b1] {
        assert!(extent.contains(b.origin()) && extent.contains(vectorcraft_geom::Point::new(b.x1, b.y1)), "{b:?} outside {extent:?}");
    }
    assert!(near(b0.x0, 199.9) && near(b0.y0, 390.9) && near(b0.x1, 300.9) && near(b0.y1, 547.1), "{b0:?}");
    assert!(near(b1.x0, 348.4) && near(b1.x1, 431.6) && near(b1.y0, b1.y1), "{b1:?}");

    // Both are open, unfilled, 2 pt black (CMYK) lines with arrowheads.
    for s in &shapes {
        assert!(!s.path.is_closed() && s.appearance.fill().is_none());
        let stroke = s.appearance.stroke().unwrap();
        // 2 pt is 7055.6 units of 1/254000 in; the file stores 7056.
        assert!((stroke.width - 2.0).abs() < 1e-3, "{}", stroke.width);
        assert_eq!((stroke.cap, stroke.join), (LineCap::Butt, LineJoin::Miter));
        assert_eq!(stroke.paint, vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.0, 0.0, 1.0)));
        assert_eq!(s.notes, ["arrowheads not translated"]);
    }
}
