//! SVG import units: physical lengths keep their printed size, pixels and user units map 1:1 to
//! points, and the document's units follow the file's.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use vectorcraft_doc::{Document, Node, NodeKind, Unit};
use vectorcraft_geom::Rect;
use vectorcraft_svg::import;

fn art(d: &Document) -> Vec<&Node> {
    let mut v = Vec::new();
    d.walk(|n| {
        if !n.is_layer() {
            v.push(n)
        }
    });
    v
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

fn close_rect(a: Rect, b: Rect) -> bool {
    close(a.x0, b.x0) && close(a.y0, b.y0) && close(a.x1, b.x1) && close(a.y1, b.y1)
}

fn text_sizes(d: &Document) -> Vec<f64> {
    art(d)
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Text(t) => Some(t.runs.iter().map(|r| r.style.size).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn a4_in_millimetres_opens_at_its_printed_size() {
    let d = import(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="210mm" height="297mm" viewBox="0 0 210 297">
        <rect x="10" y="20" width="100" height="50"/></svg>"#,
    )
    .unwrap();
    let mm = 72.0 / 25.4;
    assert!(close_rect(d.artboards[0].rect, Rect::new(0.0, 0.0, 595.28, 841.89)), "{:?}", d.artboards[0].rect);
    assert_eq!(d.units, Unit::Millimeters);
    let b = art(&d)[0].geometric_bounds().unwrap();
    assert!(close_rect(b, Rect::new(10.0 * mm, 20.0 * mm, 110.0 * mm, 70.0 * mm)), "{b:?}");
}

#[test]
fn physical_units_without_a_view_box() {
    let d = import(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="8.5in" height="11in">
        <rect width="1in" height="2cm" stroke="black" stroke-width="1mm"/></svg>"#,
    )
    .unwrap();
    assert!(close_rect(d.artboards[0].rect, Rect::new(0.0, 0.0, 612.0, 792.0)));
    assert_eq!(d.units, Unit::Inches);
    let n = art(&d)[0];
    assert!(close_rect(n.geometric_bounds().unwrap(), Rect::new(0.0, 0.0, 72.0, 72.0 / 2.54 * 2.0)));
    assert!(close(n.appearance.stroke_width(), 72.0 / 25.4));
    for (w, unit) in [("100pt", Unit::Points), ("10pc", Unit::Picas), ("10cm", Unit::Centimeters)] {
        let d = import(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="10"/>"#)).unwrap();
        assert_eq!(d.units, unit, "{w}");
    }
}

#[test]
fn unitless_and_pixels_map_one_to_one() {
    for w in ["100", "100px"] {
        let d = import(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="50"><rect width="100" height="50"/></svg>"#)).unwrap();
        assert_eq!(d.artboards[0].rect, Rect::new(0.0, 0.0, 100.0, 50.0), "{w}");
        assert_eq!(d.units, Unit::Pixels, "{w}");
        assert!(close_rect(art(&d)[0].geometric_bounds().unwrap(), Rect::new(0.0, 0.0, 100.0, 50.0)));
    }
    // No size at all: the viewBox gives it, in user units.
    let d = import(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 40"/>"#).unwrap();
    assert_eq!(d.artboards[0].rect, Rect::new(0.0, 0.0, 30.0, 40.0));
    assert_eq!(d.units, Unit::Pixels);
}

#[test]
fn font_size_units_are_honoured() {
    let d = import(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="400">
        <text x="0" y="20" font-size="12pt">a</text>
        <text x="0" y="40" font-size="16px">b</text>
        <text x="0" y="60" font-size="0.5in">c</text>
        <g font-size="10"><text x="0" y="80" font-size="2em">d</text><text x="0" y="100" font-size="150%">e</text></g>
        <text x="0" y="120">f<tspan font-size="larger">g</tspan></text>
        <text x="0" y="140" font-size="x-large">h</text>
        </svg>"#,
    )
    .unwrap();
    let sizes = text_sizes(&d);
    let want = [12.0, 16.0, 36.0, 20.0, 15.0, 12.0, 14.4, 18.0];
    assert_eq!(sizes.len(), want.len(), "{sizes:?}");
    for (s, w) in sizes.iter().zip(want) {
        assert!(close(*s, w), "{sizes:?}");
    }
}

#[test]
fn text_lengths_in_physical_units() {
    let d = import(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="10in" height="10in">
        <text x="1in" y="2in" font-size="20" letter-spacing="0.1em" stroke="red" stroke-width="1pt">Hi</text></svg>"#,
    )
    .unwrap();
    let NodeKind::Text(t) = &art(&d)[0].kind else { panic!() };
    let o = t.xf.translation();
    assert!(close(o.x, 72.0) && close(o.y, 144.0), "{o:?}");
    let st = t.first_style();
    assert!(close(st.tracking, 100.0), "{}", st.tracking);
    // User units of a physical root are CSS pixels (0.75 pt): that scale moves into the type.
    assert!(close(t.xf.as_coeffs()[0], 1.0), "{:?}", t.xf);
    assert!(close(st.stroke_width, 1.0), "{}", st.stroke_width);
    assert!(close(st.size, 15.0), "{}", st.size);
}

#[test]
fn absolute_lengths_in_a_pixel_document_keep_their_size() {
    let d = import(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="400">
        <rect width="1in" height="10mm" stroke="black" stroke-width="2pt"/></svg>"#,
    )
    .unwrap();
    let n = art(&d)[0];
    assert!(close_rect(n.geometric_bounds().unwrap(), Rect::new(0.0, 0.0, 72.0, 720.0 / 25.4)), "{:?}", n.geometric_bounds());
    assert!(close(n.appearance.stroke_width(), 2.0));
}

/// #1561: the issue's fixture, a `<symbol>` with a 100×100 viewBox drawn by a 260×180 `<use>`.
fn symbol_fixture(rect_w: &str, rect_h: &str, cx: &str, cy: &str, r: &str, view_box: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="440" height="300" viewBox="0 0 440 300">
        <rect width="440" height="300" fill="#fff2cc"/>
        <defs>
          <symbol id="icon"{view_box}>
            <rect width="{rect_w}" height="{rect_h}" fill="#e63946"/>
            <circle cx="{cx}" cy="{cy}" r="{r}" fill="#457b9d"/>
          </symbol>
        </defs>
        <use xlink:href="#icon" x="50" y="40" width="260" height="180"/>
        </svg>"##
    )
}

fn pixel(img: &vectorcraft_testkit::raster::Image, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
}

/// #1561: percentages in a `<symbol>` with a viewBox resolve against the viewBox, not the
/// `<use>` element's size, so they import like the same numbers in viewBox units.
#[test]
fn symbol_percentages_resolve_against_the_symbol_view_box() {
    use vectorcraft_testkit::raster::{assert_similar, render_artboard};
    let vb = r#" viewBox="0 0 100 100""#;
    let percent = import(&symbol_fixture("100%", "100%", "50%", "50%", "35%", vb)).unwrap();
    let numbers = import(&symbol_fixture("100", "100", "50", "50", "35", vb)).unwrap();
    let (p, n) = (render_artboard(&percent), render_artboard(&numbers));
    assert_similar(&p, &n, 8.0, 0.001);
    // `xMidYMid meet` puts a red 180×180 square at (90, 40) with the blue circle at its centre
    // (180, 130), radius 63; the background shows beside it, inside the `<use>` viewport.
    assert_eq!(pixel(&p, 180, 130), [0x45, 0x7b, 0x9d], "the circle's centre");
    assert_eq!(pixel(&p, 100, 50), [0xe6, 0x39, 0x46], "the square's corner");
    assert_eq!(pixel(&p, 290, 100), [0xff, 0xf2, 0xcc], "right of the square, inside the viewport");
}

/// A symbol without a viewBox has no coordinate system of its own: its percentages still
/// resolve against the `<use>` element's width and height.
#[test]
fn symbol_percentages_without_a_view_box_use_the_use_size() {
    use vectorcraft_testkit::raster::{assert_similar, render_artboard};
    let percent = import(&symbol_fixture("50%", "50%", "50%", "50%", "10%", "")).unwrap();
    let r = 0.1 * ((260.0f64 * 260.0 + 180.0 * 180.0) / 2.0).sqrt();
    let numbers = import(&symbol_fixture("130", "90", "130", "90", &r.to_string(), "")).unwrap();
    assert_similar(&render_artboard(&percent), &render_artboard(&numbers), 8.0, 0.001);
}

/// Shapes in a viewport nested in the symbol resolve against that viewport, so they're left as
/// they are.
#[test]
fn symbol_percentages_in_a_nested_svg_are_left_alone() {
    use vectorcraft_testkit::raster::{assert_similar, render_artboard};
    let fixture = |w: &str| {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="200" height="200" viewBox="0 0 200 200">
            <defs><symbol id="s" viewBox="0 0 100 100"><svg width="50" height="50"><rect width="{w}" height="{w}" fill="red"/></svg></symbol></defs>
            <use xlink:href="#s" width="200" height="200"/></svg>"##
        )
    };
    let percent = import(&fixture("50%")).unwrap();
    let numbers = import(&fixture("25")).unwrap();
    assert_similar(&render_artboard(&percent), &render_artboard(&numbers), 8.0, 0.001);
}
