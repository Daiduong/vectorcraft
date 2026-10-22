// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use vectorcraft_doc::Document;
use vectorcraft_svg::{ExportOptions, export, import};

#[test]
fn title_and_description_control_chars_are_filtered() {
    let mut d = Document::new(200.0, 200.0);
    d.title = "Poster\u{b}v2".into();
    d.metadata.description = "a\u{1}b".into();
    let svg = export(&d, &ExportOptions::default());
    assert!(import(&svg).is_ok(), "export wrote invalid XML: {svg}");
    assert!(svg.contains("<title>Poster v2</title>"));
    assert!(svg.contains("<desc>ab</desc>"));
}

#[test]
fn chosen_artboard_export_uses_the_artboard_name_for_title() {
    let mut d = Document::new(24.0, 24.0);
    d.title = "Icons".into();
    d.artboards[0].name = "arrow-right".into();
    let opts = ExportOptions { artboard: Some(0), title_from_artboard: true, ..ExportOptions::default() };
    let svg = export(&d, &opts);
    assert!(svg.contains("<title>arrow-right</title>"), "{svg}");
    assert!(!svg.contains("<title>Icons</title>"), "{svg}");
}

#[test]
fn title_and_description_markup_is_escaped() {
    let mut d = Document::new(200.0, 200.0);
    d.title = "A & <B>".into();
    d.metadata.description = "x & y".into();
    let svg = export(&d, &ExportOptions::default());
    assert!(import(&svg).is_ok(), "export wrote invalid XML: {svg}");
    assert!(svg.contains("<title>A &amp; &lt;B&gt;</title>"));
    assert!(svg.contains("<desc>x &amp; y</desc>"));
}
