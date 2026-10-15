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
//! laid out left to right, and one layer holding every page's translated paths, bottom first.

use std::sync::Arc;

use vectorcraft_doc::{Artboard, ColorMode, Document, LayerColor, Node, NodeKind};
use vectorcraft_geom::{Affine, Rect};

use crate::geometry::{self, PageSpace, Translation};
use crate::riff::{self, CdrVersion, FileKind};
use crate::{Error, Limits, Metadata, Package, RiffKind};

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
            let stream = Package::open(bytes, limits)?.read_capped(&riff_root.path, limits.max_inflated_bytes)?;
            let tree = riff::parse(&stream, &limits)?;
            let page = doc.metadata.as_ref().and_then(|m| m.page.as_ref()).map(PageSpace::from_metadata);
            let translation = geometry::translate(&tree, page);
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
            let mut node = Node::path(id, s.path.transformed(shift), s.appearance.clone());
            if let NodeKind::Path { rule, .. } = &mut node.kind {
                *rule = s.rule;
            }
            children.push(Arc::new(node));
            s.notes.iter().for_each(|n| warn(n.clone()));
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
    use crate::riff::build;
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
    fn package_without_a_riff_stream_is_refused() {
        let only_meta = test_zip::build(&[("metadata/metadata.xml", b"<x/>")]);
        assert_eq!(import(&only_meta, Limits::default()).err(), Some(Error::NotCdr));
    }
}
