//! A `.cdr` file to a VectorCraft [`Document`], whichever container it comes in.
//!
//! [`route`] tells the two containers apart by their first bytes, and [`import`] sends each to its
//! hook:
//!
//! - **ZIP package** (CorelDRAW X4 and later, `PK\x03\x04`): [`Package`] finds the RIFF stream
//!   (`content/riffData.cdr` or `content/root.dat`) and the metadata XML, which gives the page
//!   size, title and author.
//! - **Bare RIFF** (CorelDRAW 3 to 13, `RIFF….CDR?`): the bytes are the RIFF stream; the page size
//!   comes from its `mcfg` record.
//!
//! Both then go through the same [`riff::parse`] and [`geometry::translate`], so the two routes
//! produce the same document from the same RIFF stream. CorelDRAW 1–2 files (`WL`) are refused.
//!
//! The document has one artboard per CorelDRAW page (the master page is left out, as libcdr does),
//! laid out left to right, and one layer holding every page's translated paths, type and images
//! (embedded as PNG), bottom first.

use std::sync::Arc;

use vectorcraft_doc::{Artboard, ColorMode, Document, ImageBlob, ImageObject, LayerColor, Node, NodeKind};
use vectorcraft_geom::{Affine, Rect};

use crate::geometry::{self, Content, PageSpace, Translation};
use crate::riff::{self, CdrVersion, FileKind};
use crate::{Error, Layout, Limits, Metadata, Package, RiffKind};

/// Gap between artboards laid out side by side, in points.
pub const ARTBOARD_GAP: f64 = 36.0;

/// Which hook a file is handled by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// A ZIP package (X4 and later).
    Package,
    /// A bare RIFF stream (versions 3 to 13).
    Riff(CdrVersion),
}

/// The hook for `bytes`, from their signature.
pub fn route(bytes: &[u8]) -> Result<Route, Error> {
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return Ok(Route::Package);
    }
    match riff::identify(bytes) {
        Some(FileKind::Cdr(v)) => Ok(Route::Riff(v)),
        Some(FileKind::Waldo) => Err(Error::Riff("CorelDRAW 1–2 (WL) files are not supported".into())),
        Some(FileKind::OtherRiff(t)) => Err(Error::Riff(format!("RIFF form type {t} is not CorelDRAW"))),
        None => Err(Error::NotCdr),
    }
}

/// Is `bytes` a CorelDRAW file this reader routes: a bare `RIFF….CDR?` stream, or a ZIP package
/// holding `content/riffData.cdr` or `content/root.dat`? Other ZIP archives (Office documents,
/// fonts, plain archives) are not, so a caller can sniff by content before the extension.
pub fn is_cdr(bytes: &[u8]) -> bool {
    match route(bytes) {
        Ok(Route::Riff(_)) => true,
        Ok(Route::Package) => Package::open(bytes, Limits::default()).and_then(|mut p| p.layout()).is_ok_and(|l| l.riff.is_some()),
        Err(_) => false,
    }
}

/// Where the RIFF stream came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The package entry it was read from, and which kind of root that is.
    Package { entry: String, kind: RiffKind },
    /// The file itself.
    Riff,
}

/// An imported document and how it was read.
#[derive(Debug, Clone)]
pub struct Imported {
    pub document: Document,
    pub route: Route,
    pub source: Source,
    /// The RIFF stream's version.
    pub version: CdrVersion,
    /// The package's metadata (ZIP route only).
    pub metadata: Option<Metadata>,
    /// Per-page translation results, including notes, skipped objects and record errors.
    pub translation: Translation,
    /// What the document leaves out or approximates, each line once.
    pub warnings: Vec<String>,
}

/// Import a `.cdr` file held in memory.
pub fn import(bytes: &[u8], limits: Limits) -> Result<Imported, Error> {
    let r = route(bytes)?;
    match r {
        Route::Package => {
            let doc = crate::Document::open(bytes, limits)?;
            let riff_root = doc.layout.riff.clone().ok_or(Error::NotCdr)?;
            let mut package = Package::open(bytes, limits)?;
            let stream = package.read_capped(&riff_root.path, limits.max_inflated_bytes)?;
            // X6+: records too large for root.dat live in content/data files, which stubs name by
            // their place in the list (a file that can't be read leaves its records unread).
            let mut budget = limits.max_inflated_bytes;
            let mut streams = Vec::with_capacity(doc.layout.data_files.len());
            for name in &doc.layout.data_files {
                let bytes = package.read_capped(&Layout::data_file_path(name), budget).unwrap_or_default();
                budget = budget.saturating_sub(bytes.len() as u64);
                streams.push(bytes);
            }
            let tree = riff::parse(&stream, &limits)?;
            let page = doc.metadata.as_ref().and_then(|m| m.page.as_ref()).map(PageSpace::from_metadata);
            let translation = geometry::translate_with(&tree, page, &streams);
            let layer = match doc.layer_names() {
                [one] => Some(one.clone()),
                _ => None,
            };
            let source = Source::Package { entry: riff_root.path, kind: riff_root.kind };
            Ok(assemble(r, source, tree.version, doc.metadata, translation, layer))
        }
        Route::Riff(_) => {
            let tree = riff::parse(bytes, &limits)?;
            let translation = geometry::translate(&tree, None);
            Ok(assemble(r, Source::Riff, tree.version, None, translation, None))
        }
    }
}

fn assemble(
    route: Route,
    source: Source,
    version: CdrVersion,
    metadata: Option<Metadata>,
    translation: Translation,
    layer: Option<String>,
) -> Imported {
    let mut warnings: Vec<String> = Vec::new();
    let mut warn = |w: String| {
        if !warnings.contains(&w) {
            warnings.push(w);
        }
    };
    let pages: Vec<_> = translation.pages.iter().filter(|p| !p.master).collect();
    let cmyk = pages
        .iter()
        .flat_map(|p| &p.shapes)
        .flat_map(|s| &s.appearance.items)
        .any(|i| matches!(i.paint(), vectorcraft_color::Paint::Solid { color: vectorcraft_color::Color::Cmyk { .. }, .. }));
    let mut doc = Document::new_with_mode(1.0, 1.0, if cmyk { ColorMode::Cmyk } else { ColorMode::Rgb });
    doc.artboards.clear();
    doc.layers.clear();
    if let Some(m) = &metadata {
        if let Some(t) = &m.title {
            doc.title = t.clone();
        }
        if let Some(a) = &m.author {
            doc.metadata.author = a.clone();
        }
    }
    let mut children = Vec::new();
    let mut x = 0.0;
    for (n, p) in pages.iter().enumerate() {
        let (w, h) = p.page.size_pt();
        let (w, h) = (finite_side(w), finite_side(h));
        doc.artboards.push(Artboard {
            id: u32::try_from(n + 1).unwrap_or(u32::MAX),
            name: format!("Page {}", n + 1),
            rect: Rect::new(x, 0.0, x + w, h),
            show_center_mark: false,
            show_cross_hairs: false,
        });
        let shift = Affine::translate((x, 0.0));
        for s in &p.shapes {
            let id = doc.alloc_id();
            s.notes.iter().for_each(|n| warn(n.clone()));
            let node = match &s.content {
                // Nothing to draw (a bitmap that wasn't found: its note says so).
                Content::Path { path, .. } if path.is_empty() => continue,
                Content::Image { bitmap, xf } => match bitmap.to_png() {
                    Ok(png) => {
                        let blob = ImageBlob::new("image/png", png);
                        let key = blob.content_key();
                        doc.images.insert(key.clone(), blob);
                        let image = ImageObject {
                            key,
                            width: bitmap.width,
                            height: bitmap.height,
                            xf: shift * *xf,
                            link: None,
                            placement: Default::default(),
                        };
                        Node::new(id, NodeKind::Image(Box::new(image)))
                    }
                    Err(e) => {
                        warn(format!("a bitmap could not be stored: {e}"));
                        continue;
                    }
                },
                Content::Path { path, rule } => {
                    let mut node = Node::path(id, path.transformed(shift), s.appearance.clone());
                    if let NodeKind::Path { rule: r, .. } = &mut node.kind {
                        *r = *rule;
                    }
                    node
                }
                Content::Text(t) => {
                    let mut t = t.as_ref().clone();
                    t.xf = shift * t.xf;
                    Node::new(id, NodeKind::Text(Box::new(t)))
                }
            };
            children.push(Arc::new(node));
        }
        x += w + ARTBOARD_GAP;
    }
    if doc.artboards.is_empty() {
        // No page: one artboard of the document's page size (or a Letter page), so the file opens.
        let (w, h) = translation.pages.first().map_or((612.0, 792.0), |p| p.page.size_pt());
        doc.artboards.push(Artboard {
            id: 1,
            name: "Page 1".into(),
            rect: Rect::new(0.0, 0.0, finite_side(w), finite_side(h)),
            show_center_mark: false,
            show_cross_hairs: false,
        });
        warn("the file has no pages".into());
    }
    let master_shapes: usize = translation.pages.iter().filter(|p| p.master).map(|p| p.shapes.len()).sum();
    if master_shapes > 0 {
        warn(format!("{master_shapes} object(s) on the master page left out"));
    }
    for (kind, n) in &translation.skipped {
        warn(format!("{n} {kind} object(s) not translated yet"));
    }
    for e in &translation.errors {
        warn(format!("unreadable record: {e}"));
    }
    if pages.len() > 1 {
        warn("CorelDRAW layers are merged into one layer".into());
    }
    let id = doc.alloc_id();
    let mut node = Node::layer(id, layer.as_deref().unwrap_or("Layer 1"), LayerColor::Preset(0));
    if let Some(c) = node.children_mut() {
        *c = children;
    }
    doc.layers.push(Arc::new(node));
    Imported { document: doc, route, source, version, metadata, translation, warnings }
}

/// A page side in points VectorCraft accepts: finite and at least 1 pt.
fn finite_side(v: f64) -> f64 {
    if v.is_finite() { v.clamp(1.0, 16_383.0 * 72.0) } else { 612.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth as build;
    use crate::test_zip;

    #[test]
    fn routes_by_signature() {
        assert_eq!(route(b"PK\x03\x04rest"), Ok(Route::Package));
        assert_eq!(route(b"RIFF\0\0\0\0CDR9"), Ok(Route::Riff(CdrVersion(900))));
        assert_eq!(route(b"RIFF\0\0\0\0cdrD"), Ok(Route::Riff(CdrVersion(1300))));
        assert!(matches!(route(b"WL\x01\x02"), Err(Error::Riff(_))));
        assert!(matches!(route(b"RIFF\0\0\0\0CMX1"), Err(Error::Riff(_))));
        assert_eq!(route(b"%PDF-1.7"), Err(Error::NotCdr));
        assert_eq!(route(b""), Err(Error::NotCdr));
    }

    #[test]
    fn both_routes_reach_the_same_translation() {
        // A version 9 file with a document page size and one empty page.
        let mut mcfg = vec![0u8; 4];
        mcfg.extend_from_slice(&(2 * 254_000i32).to_le_bytes());
        mcfg.extend_from_slice(&(3 * 254_000i32).to_le_bytes());
        let riff_bytes = build::list(
            b"RIFF",
            b"CDR9",
            &[build::list(b"LIST", b"doc ", &[build::chunk(b"mcfg", &mcfg)]), build::list(b"LIST", b"page", &[build::chunk(b"flgs", &[0; 4])])],
        );
        let legacy = import(&riff_bytes, Limits::default()).unwrap();
        assert_eq!((legacy.route, &legacy.source), (Route::Riff(CdrVersion(900)), &Source::Riff));
        let zipped = test_zip::build(&[("content/riffData.cdr", &riff_bytes)]);
        let modern = import(&zipped, Limits::default()).unwrap();
        assert_eq!(modern.route, Route::Package);
        assert_eq!(modern.source, Source::Package { entry: "content/riffData.cdr".into(), kind: RiffKind::RiffData });
        assert_eq!(legacy.translation, modern.translation);
        assert_eq!(legacy.document.artboards, modern.document.artboards);
        assert_eq!(legacy.document.artboards[0].rect, Rect::new(0.0, 0.0, 144.0, 216.0));
    }

    #[test]
    fn sniffs_only_corel_containers() {
        let riff = crate::synth::triangle_riff();
        assert!(is_cdr(&riff) && is_cdr(&crate::synth::package(&riff, "t")));
        assert!(!is_cdr(&test_zip::build(&[("word/document.xml", b"<w/>")])));
        assert!(!is_cdr(b"RIFF\x04\x00\x00\x00WEBP") && !is_cdr(b"WL\x01\x02") && !is_cdr(b"PK\x03\x04 broken"));
    }

    #[test]
    fn x6_records_are_followed_into_their_data_files() {
        let i = import(&crate::synth::x6_package(), Limits::default()).unwrap();
        assert_eq!((i.version, &i.source), (CdrVersion(1600), &Source::Package { entry: "content/root.dat".into(), kind: RiffKind::Root }));
        assert!(i.translation.errors.is_empty(), "{:?}", i.translation.errors);
        let shapes = &i.translation.pages[0].shapes;
        assert_eq!(shapes.len(), 2);
        let text = shapes[0].text().unwrap();
        assert_eq!(text.plain_text(), "Ünïcode");
        assert_eq!((text.runs[0].style.font_family.as_str(), text.runs[0].style.size.round()), ("Verdana", 72.0));
        assert_eq!(shapes[1].path().unwrap().anchor_count(), 3);
        assert_eq!(shapes[1].appearance.fill_paint(), vectorcraft_color::Paint::solid(vectorcraft_color::Color::rgb8(255, 0, 0)));
        // Without its data file, the stubs can't be read, and say so.
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(crate::synth::x6_package())).unwrap();
        let root = std::io::Read::bytes(z.by_name("content/root.dat").unwrap()).map(Result::unwrap).collect::<Vec<u8>>();
        let without = test_zip::build(&[("content/root.dat", &root), ("content/dataFileList.dat", b"a.dat\n")]);
        let i = import(&without, Limits::default()).unwrap();
        assert!(i.translation.errors.iter().any(|e| e.contains("data file 0")), "{:?}", i.translation.errors);
        assert!(i.translation.pages[0].shapes.iter().all(|s| s.path().is_none()), "only the text object, without its text");
    }

    #[test]
    fn package_without_a_riff_stream_is_refused() {
        let only_meta = test_zip::build(&[("metadata/metadata.xml", b"<x/>")]);
        assert_eq!(import(&only_meta, Limits::default()).err(), Some(Error::NotCdr));
    }
}
