//! The opt-in physical size: width and height in points, so viewers that read unitless lengths
//! as CSS pixels keep the artwork's printed size.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node};
use vectorcraft_geom::{Rect, shapes};
use vectorcraft_svg::{ExportOptions, export, import};

fn doc() -> Document {
    let mut d = Document::new(72.0, 36.0);
    let l = d.layers[0].id;
    let fill = Appearance::basic(Paint::solid(Color::rgb8(40, 40, 39)), Paint::None, 0.0);
    let n = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(6.0, 3.0, 66.0, 33.0)), fill);
    d.insert(Some(l), usize::MAX, n).unwrap();
    d
}

#[test]
fn physical_size_writes_points_and_reads_back_the_same() {
    let d = doc();
    let plain = export(&d, &ExportOptions::default());
    assert!(plain.contains(" width=\"72\" height=\"36\" viewBox=\"0 0 72 36\""), "{plain}");
    let pt = export(&d, &ExportOptions { physical_size: true, ..Default::default() });
    assert!(pt.contains(" width=\"72pt\" height=\"36pt\" viewBox=\"0 0 72 36\""), "{pt}");
    // Responsive wins: no size at all.
    let resp = export(&d, &ExportOptions { physical_size: true, responsive: true, ..Default::default() });
    assert!(!resp.contains(" width=\"") && resp.contains("viewBox=\"0 0 72 36\""), "{resp}");
    // Reimported, the artboard and the art keep their size in points.
    let back = import(&pt).unwrap();
    let ab = back.artboards[0].rect;
    assert!((ab.width() - 72.0).abs() < 1e-6 && (ab.height() - 36.0).abs() < 1e-6, "{ab:?}");
    let b = back.layers[0].children().unwrap()[0].visual_bounds().unwrap();
    assert!((b.width() - 60.0).abs() < 1e-3 && (b.height() - 30.0).abs() < 1e-3, "{b:?}");
}
