//! Percentage lengths inside a `<symbol>` with a `viewBox`. SVG resolves them against the
//! symbol's own viewBox (<https://www.w3.org/TR/SVG/coords.html#Units>), but usvg resolves them
//! against the `<use>` element's width and height and then applies the viewBox transform on top,
//! which enlarges and moves the art. [`resolve`] rewrites them as plain numbers in viewBox units
//! before usvg runs, so they come out where a browser draws them.

use usvg::roxmltree;

use super::Edits;
use super::css::XNode;

/// The elements whose geometry attributes are rewritten. Gradients, patterns, masks, clip paths
/// and filters are left alone: their percentages can be fractions of a bounding box instead.
const SHAPES: &[&str] = &["rect", "circle", "ellipse", "line", "image", "use"];

/// Elements that start a new viewport (or a coordinate system of their own) for their content.
const VIEWPORTS: &[&str] = &["svg", "symbol", "pattern", "marker", "mask", "clipPath", "filter", "foreignObject"];

/// How a percentage of an attribute resolves: against the viewport's width, its height, or the
/// normalized diagonal (`sqrt((w² + h²) / 2)`).
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Diagonal,
}

/// The attributes rewritten and the axis each resolves against.
const ATTRIBUTES: &[(&str, Axis)] = &[
    ("x", Axis::X),
    ("y", Axis::Y),
    ("width", Axis::X),
    ("height", Axis::Y),
    ("cx", Axis::X),
    ("cy", Axis::Y),
    ("r", Axis::Diagonal),
    ("rx", Axis::X),
    ("ry", Axis::Y),
    ("x1", Axis::X),
    ("y1", Axis::Y),
    ("x2", Axis::X),
    ("y2", Axis::Y),
];

/// Rewrite the percentage geometry attributes of the shapes in every `<symbol>` with a valid
/// `viewBox` (in `svg`, parsed as `xml`) as numbers in viewBox units. Shapes inside a nested
/// viewport keep theirs: they resolve against that viewport.
pub(super) fn resolve(svg: &str, xml: &roxmltree::Document, edits: &mut Edits) {
    for symbol in xml.descendants().filter(|n| n.is_element() && n.tag_name().name() == "symbol") {
        let Some((w, h)) = symbol.attribute("viewBox").and_then(view_box_size) else { continue };
        let diagonal = ((w * w + h * h) / 2.0).sqrt();
        for n in symbol.descendants().filter(|n| n.is_element() && SHAPES.contains(&n.tag_name().name())) {
            if !in_viewport_of(n, symbol) {
                continue;
            }
            for (name, axis) in ATTRIBUTES {
                let Some(percent) = n.attribute(*name).and_then(percentage) else { continue };
                let basis = match axis {
                    Axis::X => w,
                    Axis::Y => h,
                    Axis::Diagonal => diagonal,
                };
                let value = percent / 100.0 * basis;
                if value.is_finite() {
                    edits.set(svg, n, name, &value.to_string());
                }
            }
        }
    }
}

/// Whether the nearest viewport-establishing ancestor of `n` is `symbol`.
fn in_viewport_of(n: XNode, symbol: XNode) -> bool {
    n.ancestors().skip(1).find(|a| a.is_element() && VIEWPORTS.contains(&a.tag_name().name())) == Some(symbol)
}

/// The width and height of a `viewBox` value (`min-x min-y width height`, separated by spaces
/// and/or commas), when both are positive and finite.
fn view_box_size(v: &str) -> Option<(f64, f64)> {
    let nums: Vec<f64> = v.split(|c: char| c.is_whitespace() || c == ',').filter(|s| !s.is_empty()).map(str::parse).collect::<Result<_, _>>().ok()?;
    let [_, _, w, h] = nums[..] else { return None };
    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some((w, h))
}

/// The number of a percentage length such as `50%` or ` 12.5% `.
fn percentage(v: &str) -> Option<f64> {
    let n: f64 = v.trim().strip_suffix('%')?.trim_end().parse().ok()?;
    n.is_finite().then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_view_boxes_and_percentages() {
        assert_eq!(view_box_size("0 0 100 50"), Some((100.0, 50.0)));
        assert_eq!(view_box_size("-5,-5, 20,10"), Some((20.0, 10.0)));
        assert_eq!(view_box_size("0 0 0 10"), None);
        assert_eq!(view_box_size("0 0 10"), None);
        assert_eq!(view_box_size("0 0 a 10"), None);
        assert_eq!(percentage("50%"), Some(50.0));
        assert_eq!(percentage(" 12.5% "), Some(12.5));
        assert_eq!(percentage("50"), None);
        assert_eq!(percentage("%"), None);
        assert_eq!(percentage("NaN%"), None);
    }
}
