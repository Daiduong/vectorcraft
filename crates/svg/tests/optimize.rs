//! SVG Options → Optimize: oxvg shrinks the file while every other option keeps what it promises
//! (ids, metadata, editing data, style sheets, hidden layers, the profile, the encoding, decimals),
//! and Flatten alone merges objects.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use vectorcraft_color::{Color, Gradient, GradientKind, GradientPaint, GradientStop, Paint};
use vectorcraft_doc::{Appearance, Document, LayerColor, Node, NodeKind};
use vectorcraft_geom::{Rect, shapes};
use vectorcraft_svg::{Encoding, ExportOptions, ObjectIds, Profile, Styling, editing, export, export_full, import};

/// A named red square and a gradient square on the first layer, two blue squares of one
/// appearance on a second layer, and a hidden layer with a square.
fn doc() -> Document {
    let mut d = Document::new(200.0, 100.0);
    d.title = "Grüße €".into();
    let fill = |c| Appearance::basic(Paint::solid(c), Paint::None, 0.0);
    let l = d.layers[0].id;
    let mut a = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(10.123456, 10.0, 50.0, 50.0)), fill(Color::rgb8(255, 0, 0)));
    a.name = Some("Sky".into());
    d.insert(Some(l), usize::MAX, a).unwrap();
    let g = GradientPaint::new(Gradient {
        kind: GradientKind::Linear,
        stops: vec![GradientStop::new(0.0, Color::BLACK), GradientStop::new(1.0, Color::WHITE)],
    });
    let b = Node::path(
        d.alloc_id(),
        shapes::rectangle(Rect::new(60.0, 10.0, 100.0, 50.0)),
        Appearance::basic(Paint::Gradient(Box::new(g)), Paint::None, 0.0),
    );
    d.insert(Some(l), usize::MAX, b).unwrap();
    let mut blues = Node::layer(d.alloc_id(), "Blues", LayerColor::Preset(2));
    for x in [110.0, 150.0] {
        let p = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(x, 10.0, x + 30.0, 40.0)), fill(Color::rgb8(0, 0, 255)));
        blues.children_mut().unwrap().push(Arc::new(p));
    }
    d.layers.push(Arc::new(blues));
    let mut hidden = Node::layer(d.alloc_id(), "Notes", LayerColor::Preset(1));
    let p = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(10.0, 60.0, 40.0, 90.0)), fill(Color::rgb8(0, 255, 0)));
    hidden.children_mut().unwrap().push(Arc::new(p));
    hidden.visible = false;
    d.layers.push(Arc::new(hidden));
    d
}

fn objects(svg: &str) -> usize {
    let mut n = 0;
    import(svg).unwrap().walk(|node| n += usize::from(matches!(node.kind, NodeKind::Path { .. })));
    n
}

#[test]
fn optimize_shrinks_the_file_and_keeps_the_objects() {
    let d = doc();
    for styling in [Styling::PresentationAttributes, Styling::InlineStyle, Styling::InternalCss] {
        for minify in [false, true] {
            let plain = export(&d, &ExportOptions { styling, minify, ..Default::default() });
            let o = export_full(&d, &ExportOptions { styling, minify, optimize: true, ..Default::default() }, None);
            let svg = &o.svg;
            assert!(o.warnings.is_empty(), "{:?}", o.warnings);
            assert!(svg.len() < plain.len(), "{styling:?} {minify}: {} ≥ {}", svg.len(), plain.len());
            // One line, behind the XML declaration unless minified; the unused xlink namespace gone.
            assert_eq!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"), !minify, "{svg}");
            let markup = svg.trim_end().split_once('\n').map_or(svg.as_str(), |(_, m)| m);
            assert!(markup.starts_with("<svg") && !markup.contains('\n') && !svg.contains("xlink") && svg.ends_with("</svg>\n") == !minify, "{svg}");
            // Layer Names: the names stay as ids, the layers as groups, the objects apart.
            assert!(svg.contains("id=\"Sky\"") && svg.contains("id=\"Layer_1\"") && svg.contains("id=\"Blues\""), "{svg}");
            assert_eq!(svg.matches("<path").count(), 4, "{svg}");
            assert_eq!(objects(svg), objects(&plain), "{svg}");
            assert!(svg.contains("<title>Grüße €</title>"), "{svg}");
            if styling == Styling::InternalCss {
                // The style sheet stays one (oxvg would inline single-use classes).
                assert!(svg.contains("<style>") && svg.contains("class=\"cls-1\""), "{svg}");
            }
        }
    }
}

#[test]
fn ids_follow_the_object_ids_option() {
    let d = doc();
    let minimal = export(&d, &ExportOptions { object_ids: ObjectIds::Minimal, optimize: true, ..Default::default() });
    assert!(!minimal.contains("Sky") && !minimal.contains("linear-gradient") && minimal.contains("url(#"), "{minimal}");
    let unique = export(&d, &ExportOptions { object_ids: ObjectIds::Unique, optimize: true, ..Default::default() });
    assert!(unique.contains("-Sky\"") && unique.contains("-linear-gradient-1\""), "{unique}");
}

#[test]
fn metadata_and_editing_data_survive() {
    let d = doc();
    let svg = export(&d, &ExportOptions { metadata: true, optimize: true, minify: true, ..Default::default() });
    assert!(svg.starts_with("<svg") && svg.contains("<metadata>") && svg.contains("<dc:title>Grüße €</dc:title>"), "{svg}");
    for minify in [false, true] {
        let o = export_full(&d, &ExportOptions { preserve_editing: true, optimize: true, minify, ..Default::default() }, Some(b"native bytes"));
        let e = editing(&o.svg).unwrap();
        assert!(e.intact && !e.data.is_empty(), "{}", o.svg);
        assert!(o.svg.contains("id=\"Sky\""), "{}", o.svg);
    }
}

#[test]
fn entities_are_written_as_is_with_a_warning() {
    let d = doc();
    let plain = export_full(&d, &ExportOptions { styling: Styling::StyleEntities, ..Default::default() }, None);
    let o = export_full(&d, &ExportOptions { styling: Styling::StyleEntities, optimize: true, ..Default::default() }, None);
    assert_eq!(o.svg, plain.svg);
    assert!(o.warnings.iter().any(|w| w.starts_with("Optimize doesn't apply")), "{:?}", o.warnings);
}

#[test]
fn flatten_merges_paths_of_one_appearance() {
    let d = doc();
    let kept = export(&d, &ExportOptions { optimize: true, ..Default::default() });
    let flat = export(&d, &ExportOptions { optimize: true, flatten: true, ..Default::default() });
    assert_eq!(kept.matches("<path").count(), 4, "{kept}");
    assert_eq!(flat.matches("<path").count(), 3, "{flat}");
    assert!(flat.len() < kept.len());
    // Flatten without Optimize changes nothing.
    assert_eq!(export(&d, &ExportOptions { flatten: true, ..Default::default() }), export(&d, &ExportOptions::default()));
}

#[test]
fn latin1_keeps_the_declaration_and_its_references() {
    let d = doc();
    for minify in [false, true] {
        let o = export_full(&d, &ExportOptions { encoding: Encoding::Latin1, optimize: true, minify, ..Default::default() }, None);
        assert!(o.bytes().starts_with(b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>"), "{}", o.svg);
        assert!(o.svg.contains("<title>Gr\u{fc}&#xDF;e &#x20AC;</title>") || o.svg.contains("&#x20AC;"), "{}", o.svg);
        assert!(o.bytes().iter().all(|b| *b < 0x80 || *b == 0xfc || *b == 0xdf), "{}", o.svg);
        assert_eq!(objects(&o.svg), 4, "{}", o.svg);
    }
}

#[test]
fn hidden_layers_profile_and_decimals_are_kept() {
    let d = doc();
    let hidden = export(&d, &ExportOptions { hidden_layers: true, optimize: true, ..Default::default() });
    assert!(hidden.contains("id=\"Notes\"") && hidden.contains("display") && hidden.matches("<path").count() == 5, "{hidden}");
    let tiny = export(&d, &ExportOptions { profile: Profile::Tiny12, optimize: true, ..Default::default() });
    assert!(tiny.contains("baseProfile=\"tiny\"") && tiny.contains("version=\"1.2\""), "{tiny}");
    let two = export(&d, &ExportOptions { decimals: 2, optimize: true, ..Default::default() });
    assert!(two.contains("10.12") && !two.contains("10.123"), "{two}");
    // oxvg rounds to at most 5 places.
    let seven = export(&d, &ExportOptions { decimals: 7, optimize: true, ..Default::default() });
    assert!(seven.contains("10.12346"), "{seven}");
}
