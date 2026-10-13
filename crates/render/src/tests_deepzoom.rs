//! Deep zoom (#1087): shapes far larger than the view, seen up close, draw where they are.

use super::*;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Node};
use vectorcraft_geom::shapes;

/// A document holding one red rectangle `r`, unstroked.
fn doc_with(r: Rect) -> Document {
    let mut d = Document::new_infinite(vectorcraft_doc::ColorMode::Rgb);
    let id = d.alloc_id();
    let n = Node::path(id, shapes::rectangle(r), Appearance::basic(Paint::solid(Color::rgb(1.0, 0.0, 0.0)), Paint::None, 0.0));
    let layer = d.layers[0].id;
    d.insert(Some(layer), 0, n).unwrap();
    d
}

fn rgba(img: &Rendered, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], img.pixels[i + 3]]
}

fn red(p: [u8; 4]) -> bool {
    p[0] > 240 && p[1] < 15 && p[2] < 15 && p[3] > 240
}

fn white(p: [u8; 4]) -> bool {
    p[0] > 240 && p[1] > 240 && p[2] > 240
}

/// A square ±1e9 pt wide fills a 10 pt view at a zoom of a million: it was dropped as too large
/// (its corners 1e15 px away) before the guard band.
#[test]
fn a_huge_square_fills_the_view_at_deep_zoom() {
    let d = doc_with(Rect::new(-1.0e9, -1.0e9, 1.0e9, 1.0e9));
    let img = Renderer::new().render_region(&d, Rect::new(10.0, 10.0, 10.00005, 10.00005), 1.0e6, true);
    assert_eq!((img.width, img.height), (50, 50));
    for (x, y) in [(0, 0), (25, 25), (49, 49)] {
        assert!(red(rgba(&img, x, y)), "({x}, {y}): {:?}", rgba(&img, x, y));
    }
}

/// The corner of a square 1e8 pt from the origin, seen at a zoom of 100,000: its edges fall on
/// the right pixels (half the view red, half white).
#[test]
fn a_far_corner_lands_on_the_right_pixels() {
    let c = 1.0e8;
    let d = doc_with(Rect::new(-c, -c, c, c));
    // A 40 px view centred on the corner: 1 px is 1e-5 pt.
    let region = Rect::new(c - 20e-5, c - 20e-5, c + 20e-5, c + 20e-5);
    let img = Renderer::new().render_region(&d, region, 1.0e5, true);
    assert_eq!((img.width, img.height), (40, 40));
    assert!(red(rgba(&img, 10, 10)), "inside the square: {:?}", rgba(&img, 10, 10));
    assert!(white(rgba(&img, 30, 10)) && white(rgba(&img, 10, 30)) && white(rgba(&img, 30, 30)), "outside it");
    // The edge is at pixel 20, give or take one.
    assert!(red(rgba(&img, 18, 10)) && white(rgba(&img, 22, 10)), "{:?} {:?}", rgba(&img, 18, 10), rgba(&img, 22, 10));
}
