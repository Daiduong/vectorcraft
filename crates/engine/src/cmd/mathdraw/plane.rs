//! Plane figures: marks on segments (length, angle, equal lengths, parallel), and constructions
//! (triangles from sides and angles, dots at anchor points, circles from segments, fractals by
//! repeated similarity maps, keeping what lies inside a rectangle).
//!
//! The commands that work on segments take `segments: [[x1, y1, x2, y2], …]` in document
//! coordinates, else the straight segments in the selection ([`selected_segments`]: open paths of
//! two anchors, in selected groups too), leaving out magenta ones (reference segments). Lengths
//! are in points. Each command is one undo step and answers the ids of what it drew: `id` (the
//! group, or the one object), and per part.

use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::sync::Arc;

use serde_json::{Map, Value, json};
use vectorcraft_color::Color;
use vectorcraft_doc::{Document, LineRef, MathDrawStyle, Node, NodeId, NodeKind};
use vectorcraft_geom::{Anchor, CubicBez, FillRule, ParamCurve, PathData, Point, Rect, SubPath, Vec2, kurbo, shapes};
use vectorcraft_pathops as po;

use super::graph::{ids, pairs};
use super::*;
use crate::cmd::edit::selected_roots;

/// The longest length (pt) a mark parameter takes.
const MAX_PT: f64 = 10_000.0;
/// Most segments a command takes (`angleMark` looks at every pair: fewer).
const MAX_SEGMENTS: usize = 10_000;
const MAX_ANGLE_SEGMENTS: usize = 1_000;
/// Most arcs `angleMark` draws.
const MAX_ARCS: usize = 10_000;
/// Most ticks or chevrons on a segment.
const MAX_TICKS: usize = 10;
/// Most dots `mathdraw.dots` draws.
const MAX_DOTS: usize = 10_000;
/// Most objects a fractal adds (counting the objects inside groups).
const MAX_FRACTAL_NODES: usize = 4096;
const MAX_FRACTAL_LEVELS: u64 = 8;
/// The handle length of a quarter ellipse, as a fraction of its radius.
const KAPPA: f64 = 0.552_284_749_830_793_4;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "mathdraw.lengthMark",
            "Mark Segment Length",
            [],
            None,
            "{segments?: [[x1, y1, x2, y2], …] pt (default the straight segments selected, not magenta ones), distance?: pt (default 20; how far the mark runs from the segment, on its left seen from its start; negative: on its right), gap?: pt (default 40; the hole in the middle, for the length), line?: line (default solid4), color?: \"#rrggbb\"} → {id, marks: [ids], skipped: [indices of segments no longer than the gap]}; per segment two quarter-ellipse arcs, rising from its ends and running beside it up to the gap",
            has_doc,
            length_mark
        ),
        cmd!(
            "mathdraw.angleMark",
            "Mark Angles",
            [],
            None,
            "{segments?: [[x1, y1, x2, y2], …] pt (at least two; default the straight segments selected, not magenta ones), size?: radius pt (default 40), line?: line (default solid4), color?: \"#rrggbb\"} → {id, arcs: [ids], vertices: [[x, y]…]}; an arc for each pair of segments that meet (at an end or crossing), in the smallest angle between them there; parallel segments and segments that don't reach each other are passed over",
            has_doc,
            angle_mark
        ),
        cmd!(
            "mathdraw.sameMark",
            "Mark Equal Lengths",
            [],
            None,
            "{segments?: [[x1, y1, x2, y2], …] pt (default the straight segments selected, not magenta ones), count?: 0–10 (default 1; 0: a small circle of radius `len`), gap?: pt between ticks (default 8), len?: tick length pt (default 20), angle?: degrees the ticks turn from square to the segment (default 0), line?: line (default solid3), color?: \"#rrggbb\"} → {id, marks: [ids]}; ticks across the middle of each segment",
            has_doc,
            same_mark
        ),
        cmd!(
            "mathdraw.parallelMark",
            "Mark Parallel Segments",
            [],
            None,
            "{segments?: [[x1, y1, x2, y2], …] pt (default the straight segments selected, not magenta ones), count?: 1–10 (default 1), gap?: pt between chevrons (default 8), len?: pt (default 20; each arm is half of it), angle?: degrees each arm turns back from square to the segment (default 45), line?: line (default solid3), color?: \"#rrggbb\"} → {id, marks: [ids]}; chevrons (>) pointing from each segment's start to its end, at its middle",
            has_doc,
            parallel_mark
        ),
        cmd!(
            "mathdraw.triangle",
            "Triangle by Sides and Angles",
            [],
            None,
            "{mode: \"SSS\" | \"SAS\" | \"ASA\", base: length (every mode), left?: length of the side from the base's start (SSS, SAS), right?: length of the side from the base's end (SSS), leftAngle?: degrees at the base's start (SAS, ASA), rightAngle?: degrees at the base's end (ASA), baseSegment?: [x1, y1, x2, y2] pt (default the straight segment selected: the base is drawn on it and its length is `base`), rotation?: degrees counter-clockwise of the base without a segment (default 0), unit?: pt per length without a segment (default 1), line?: line (default solid3), color?: \"#rrggbb\"} → {id, vertices: [[x, y] ×3]}; a closed triangle on the base with its apex on the base's left (above, unrotated); without a segment the base starts at the centre of the first artboard",
            has_doc,
            triangle
        ),
        cmd!(
            "mathdraw.dots",
            "Dots at Anchor Points",
            [],
            None,
            "{ids?: [object ids] (default the selection), dot?: diameter pt (default the style's dot size), color?: \"#rrggbb\"} → {id, dots: [ids], points: [[x, y]…]}; a dot (with the style's border) at every anchor point of the paths in the objects; with no objects one dot at the centre of the first artboard",
            has_doc,
            dots
        ),
        cmd!(
            "mathdraw.circleFromSegment",
            "Circle from Segment",
            [],
            None,
            "{mode: \"radius\" | \"diameter\", segments?: [[x1, y1, x2, y2], …] pt (default the straight segments selected, not magenta ones), line?: line (default solid3), color?: \"#rrggbb\"} → {id, circles: [ids]}; radius: centred on the segment's start through its end; diameter: on its middle through both ends",
            has_doc,
            circle_from_segment
        ),
        cmd!(
            "mathdraw.fractal",
            "Fractal Figure",
            [],
            None,
            "{levels: 1–8, baseSegment?: [x1, y1, x2, y2] pt (default the magenta segment selected), placeSegments?: [[x1, y1, x2, y2], …] pt (default the yellow segments selected), targetIds?: [object ids] (default the other selected objects)} → {id, count, levels: [copies per level]}; each place segment gives the map (move, turn, uniform scale) that takes the base segment onto it, start to start; level 1 is every map applied to copies of the targets, each next level every map applied to the level before; the copies (at most 4096 objects) go in one group, the originals stay",
            has_doc,
            fractal
        ),
        cmd!(
            "mathdraw.clipRect",
            "Keep Inside Rectangle",
            [],
            None,
            "{rectId?: object id of an axis-aligned rectangle path (default the frontmost one selected), ids?: [object ids] (default the other selected objects), keepRect?: bool (default false: the rectangle is deleted)} → {clipped: [ids], removed: [ids of paths wholly outside], skipped: [ids of what isn't a path: type, equations, images…], rect?: the rectangle's id when kept}; paths (in groups too) keep only what lies inside the rectangle: filled closed shapes by a boolean intersection, lines and curves cut where they cross it",
            has_doc,
            clip_rect
        ),
    ]
}

// ---------- shared ----------

/// A straight segment, from `a` to `b` (document coordinates, not of zero length).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Seg {
    a: Point,
    b: Point,
}

impl Seg {
    fn len(self) -> f64 {
        (self.b - self.a).hypot()
    }

    /// The unit vector from `a` to `b`.
    fn dir(self) -> Vec2 {
        (self.b - self.a) / self.len()
    }

    /// The unit normal on the segment's left as it appears on the page (y down), seen from `a`.
    fn normal(self) -> Vec2 {
        let t = self.dir();
        Vec2::new(t.y, -t.x)
    }

    fn mid(self) -> Point {
        self.a.midpoint(self.b)
    }
}

/// `[x1, y1, x2, y2]` as a segment on the canvas with a length.
fn segment_value(v: &Value) -> Option<Seg> {
    let c = finite_numbers::<4>(v).filter(|c| c.iter().all(|v| v.abs() <= crate::MAX_COORD))?;
    let [x1, y1, x2, y2] = c;
    let s = Seg { a: Point::new(x1, y1), b: Point::new(x2, y2) };
    (s.len() > 1e-9).then_some(s)
}

fn segment_param(p: &Value, key: &str, cmd: &str) -> Result<Option<Seg>> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(None),
        Some(v) => segment_value(v).map(Some).ok_or_else(|| bad(cmd, format!("`{key}` must be [x1, y1, x2, y2] on the canvas, its ends apart"))),
    }
}

fn segments_param(p: &Value, key: &str, max: usize, cmd: &str) -> Result<Option<Vec<Seg>>> {
    let Some(v) = p.get(key).filter(|v| !v.is_null()) else { return Ok(None) };
    let err = || bad(cmd, format!("`{key}` must be a list of [x1, y1, x2, y2] segments on the canvas, their ends apart (at most {max})"));
    let a = v.as_array().filter(|a| a.len() <= max).ok_or_else(err)?;
    a.iter().map(|e| segment_value(e).ok_or_else(err)).collect::<Result<Vec<_>>>().map(Some)
}

/// The straight segments in the selection (selected, or in selected groups) that `keep` keeps, in
/// selection order, each once.
fn selected_segments(s: &Session, keep: impl Fn(&Node) -> bool) -> Result<Vec<(NodeId, Seg)>> {
    let st = s.doc()?;
    let mut seen = HashSet::new();
    let mut out = vec![];
    for id in &st.selection.objects {
        let Some(n) = st.doc.node(*id) else { continue };
        n.walk(&mut |c| {
            if let Some((a, b)) = segment_ends(c)
                && keep(c)
                && seen.insert(c.id)
            {
                out.push((c.id, Seg { a, b }));
            }
        });
    }
    Ok(out)
}

/// The segments a mark command works on: `segments`, else the selected ones that aren't magenta.
fn segments(s: &Session, p: &Value, max: usize, cmd: &str) -> Result<Vec<Seg>> {
    let segs = match segments_param(p, "segments", max, cmd)? {
        Some(v) => v,
        None => {
            let v: Vec<Seg> = selected_segments(s, |n| !is_magenta(n))?.into_iter().map(|(_, s)| s).collect();
            if v.len() > max {
                return Err(bad(cmd, format!("select at most {max} segments")));
            }
            v
        }
    };
    if segs.is_empty() {
        return Err(bad(cmd, "select straight segments (or give `segments`)"));
    }
    Ok(segs)
}

/// The line and colour a drawing command strokes with (`line`, `color`).
struct Look {
    line: LineRef,
    color: Color,
}

impl Look {
    fn of(p: &Value, cmd: &str, line: LineRef) -> Result<Self> {
        if !p.is_object() && !p.is_null() {
            return Err(bad(cmd, "the parameters must be an object"));
        }
        Ok(Self { line: line_param(p, "line", cmd)?.unwrap_or(line), color: color_param(p, "color", cmd)?.unwrap_or(Color::BLACK) })
    }

    fn node(&self, d: &mut Document, style: &MathDrawStyle, path: PathData) -> Node {
        stroked(d, path, style, self.line, self.color)
    }
}

/// An optional length parameter in `range` (pt).
fn pt_param(p: &Value, key: &str, default: f64, range: RangeInclusive<f64>, cmd: &str) -> Result<f64> {
    match number_param(p, key, cmd)? {
        None => Ok(default),
        Some(v) if range.contains(&v) => Ok(v),
        Some(_) => Err(bad(cmd, format!("`{key}` must be from {} to {} pt", range.start(), range.end()))),
    }
}

/// One undo step labelled `label`: what `build` draws, inserted on top as one object (several in a
/// group named `name`). Answers what `build` answers with the `id` of what was inserted.
fn emit(
    s: &mut Session,
    label: &str,
    name: &str,
    cmd: &str,
    build: impl FnOnce(&mut Document, &MathDrawStyle) -> Result<(Vec<Node>, Map<String, Value>)>,
) -> Result<Value> {
    let st = s.doc()?;
    let parent = st.target_parent()?;
    let style = st.doc.math_draw.clone();
    s.edit(label, |d, _| {
        let (nodes, mut out) = build(d, &style)?;
        let id = insert_output(d, parent, name, nodes, cmd)?;
        out.insert("id".into(), json!(id.0));
        Ok(Value::Object(out))
    })
}

/// A subpath through the cubics `run`, which follow on from one another.
fn curve_subpath(run: &[CubicBez]) -> SubPath {
    // (point, in handle, out handle)
    let mut pts: Vec<(Point, Point, Point)> = vec![];
    for c in run {
        match pts.last_mut() {
            Some(last) => last.2 = c.p1,
            None => pts.push((c.p0, c.p0, c.p1)),
        }
        pts.push((c.p3, c.p2, c.p3));
    }
    SubPath::new(pts.into_iter().map(|(p, i, o)| Anchor::with_handles(p, i, o)).collect(), false)
}

fn artboard_centre(d: &Document) -> Point {
    d.artboards.first().map_or(Point::ZERO, |a| a.rect.center())
}

/// Whether `n`'s stroke is yellow (a fractal's place segment).
fn is_yellow(n: &Node) -> bool {
    let Some(s) = n.appearance.stroke().filter(|s| s.visible) else { return false };
    let Paint::Solid { color, .. } = &s.paint else { return false };
    let [r, g, b] = color.to_rgb_uncalibrated();
    r > 0.8 && g > 0.7 && b < 0.3
}

// ---------- marks ----------

fn length_mark(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.lengthMark";
    let look = Look::of(p, C, LineRef::Solid(4))?;
    let dist = pt_param(p, "distance", 20.0, -MAX_PT..=MAX_PT, C)?;
    if dist == 0.0 {
        return Err(bad(C, "`distance` must not be 0"));
    }
    let gap = pt_param(p, "gap", 40.0, 0.0..=MAX_PT, C)?;
    let segs = segments(s, p, MAX_SEGMENTS, C)?;
    if segs.iter().all(|sg| sg.len() <= gap) {
        return Err(bad(C, "every segment is no longer than the gap"));
    }
    emit(s, "Mark Segment Length", "Length Marks", C, |d, style| {
        let (mut marks, mut skipped) = (vec![], vec![]);
        for (i, sg) in segs.iter().enumerate() {
            let half = (sg.len() - gap) / 2.0;
            if half <= 1e-9 {
                skipped.push(i);
                continue;
            }
            let n = sg.normal();
            // A quarter ellipse from an end: out from the segment, then along it to the gap.
            let arc = |from: Point, along: Vec2| {
                let end = from + along * half + n * dist;
                curve_subpath(&[CubicBez::new(from, from + n * (dist * KAPPA), end - along * (half * KAPPA), end)])
            };
            let t = sg.dir();
            marks.push(look.node(d, style, PathData::new(vec![arc(sg.a, t), arc(sg.b, -t)])));
        }
        let mut out = Map::new();
        out.insert("marks".into(), ids(&marks));
        out.insert("skipped".into(), json!(skipped));
        Ok((marks, out))
    })
}

/// Where `s1` and `s2` meet, and the arc of the smallest angle between them there: the direction
/// it starts in and its sweep (radians, on the page). `None` for parallel segments and segments
/// that don't reach each other.
fn meeting(s1: Seg, s2: Seg) -> Option<(Point, f64, f64)> {
    let (d1, d2) = (s1.b - s1.a, s2.b - s2.a);
    let den = d1.cross(d2);
    if den.abs() <= 1e-9 * d1.hypot() * d2.hypot() {
        return None;
    }
    let w = s2.a - s1.a;
    let (u1, u2) = (w.cross(d2) / den, w.cross(d1) / den);
    // Within a millionth of a point of an end counts as at the end.
    let (e1, e2) = (1e-6 / d1.hypot(), 1e-6 / d2.hypot());
    if !(-e1..=1.0 + e1).contains(&u1) || !(-e2..=1.0 + e2).contains(&u2) {
        return None;
    }
    let at = s1.a + d1 * u1;
    // The directions each segment leaves the meeting point in.
    let rays = |u: f64, e: f64, d: Vec2| [(u > e).then(|| (-d).atan2()), (u < 1.0 - e).then(|| d.atan2())].into_iter().flatten();
    let mut best: Option<(f64, f64)> = None;
    for r1 in rays(u1, e1, d1) {
        for r2 in rays(u2, e2, d2) {
            let mut sweep = (r2 - r1).rem_euclid(std::f64::consts::TAU);
            if sweep > std::f64::consts::PI {
                sweep -= std::f64::consts::TAU;
            }
            if best.is_none_or(|(_, b)| sweep.abs() < b.abs() - 1e-12) {
                best = Some((r1, sweep));
            }
        }
    }
    best.map(|(start, sweep)| (at, start, sweep))
}

fn angle_mark(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.angleMark";
    let look = Look::of(p, C, LineRef::Solid(4))?;
    let r = pt_param(p, "size", 40.0, 0.01..=MAX_PT, C)?;
    let segs = segments(s, p, MAX_ANGLE_SEGMENTS, C)?;
    if segs.len() < 2 {
        return Err(bad(C, "it takes at least two segments"));
    }
    let mut meets = vec![];
    for (i, a) in segs.iter().enumerate() {
        for b in segs.iter().skip(i + 1) {
            meets.extend(meeting(*a, *b));
        }
    }
    if meets.is_empty() {
        return Err(bad(C, "no two of the segments meet"));
    }
    if meets.len() > MAX_ARCS {
        return Err(bad(C, format!("the segments meet {} times (at most {MAX_ARCS} arcs)", meets.len())));
    }
    emit(s, "Mark Angles", "Angle Marks", C, |d, style| {
        let arcs: Vec<Node> = meets
            .iter()
            .map(|&(at, start, sweep)| {
                let arc = kurbo::Arc::new(at, Vec2::new(r, r), start, sweep, 0.0);
                let bp: kurbo::BezPath = kurbo::Shape::path_elements(&arc, 0.01).collect();
                look.node(d, style, PathData::from_bezpath(&bp))
            })
            .collect();
        let mut out = Map::new();
        out.insert("arcs".into(), ids(&arcs));
        out.insert("vertices".into(), pairs(&meets.iter().map(|m| m.0).collect::<Vec<_>>()));
        Ok((arcs, out))
    })
}

/// The ticks (or chevrons) of `count` across the middle of `sg`, `gap` apart and `len` long,
/// turned by `angle` (radians); count 0 is a circle of radius `len`.
fn tick_path(sg: Seg, count: usize, gap: f64, len: f64, angle: f64, chevrons: bool) -> PathData {
    let (t, n, mid) = (sg.dir(), sg.normal(), sg.mid());
    if count == 0 {
        return shapes::ellipse(Rect::new(mid.x - len, mid.y - len, mid.x + len, mid.y + len));
    }
    let (sin, cos) = angle.sin_cos();
    let subpaths = (0..count)
        .map(|k| {
            let c = mid + t * ((k as f64 - (count - 1) as f64 / 2.0) * gap);
            if chevrons {
                // Two arms of half the length back from the apex, the chevron centred on `c`.
                let half = len / 2.0;
                let apex = c + t * (half * sin / 2.0);
                let arm = |side: f64| apex + (n * (side * cos) - t * sin) * half;
                SubPath::polyline(&[arm(1.0), apex, arm(-1.0)], false)
            } else {
                // The normal turned counter-clockwise (as seen) by the angle.
                let u = n * cos - t * sin;
                SubPath::polyline(&[c - u * (len / 2.0), c + u * (len / 2.0)], false)
            }
        })
        .collect();
    PathData::new(subpaths)
}

fn tick_marks(s: &mut Session, p: &Value, chevrons: bool) -> Result<Value> {
    let (cmd, label, name, counts, angle) = if chevrons {
        ("mathdraw.parallelMark", "Mark Parallel Segments", "Parallel Marks", 1..=MAX_TICKS, 45.0)
    } else {
        ("mathdraw.sameMark", "Mark Equal Lengths", "Equal Length Marks", 0..=MAX_TICKS, 0.0)
    };
    let look = Look::of(p, cmd, LineRef::Solid(3))?;
    let count = count_param(p, "count", 1, counts, cmd)?;
    let gap = pt_param(p, "gap", 8.0, 0.0..=MAX_PT, cmd)?;
    let len = pt_param(p, "len", 20.0, 0.01..=MAX_PT, cmd)?;
    let angle = match number_param(p, "angle", cmd)? {
        None => angle,
        Some(a) if a.abs() <= 360.0 => a,
        Some(_) => return Err(bad(cmd, "`angle` must be from -360 to 360 degrees")),
    };
    let segs = segments(s, p, MAX_SEGMENTS, cmd)?;
    emit(s, label, name, cmd, |d, style| {
        let marks: Vec<Node> = segs.iter().map(|sg| look.node(d, style, tick_path(*sg, count, gap, len, angle.to_radians(), chevrons))).collect();
        let mut out = Map::new();
        out.insert("marks".into(), ids(&marks));
        Ok((marks, out))
    })
}

fn same_mark(s: &mut Session, p: &Value) -> Result<Value> {
    tick_marks(s, p, false)
}

fn parallel_mark(s: &mut Session, p: &Value) -> Result<Value> {
    tick_marks(s, p, true)
}

// ---------- constructions ----------

/// The apex of a triangle on a base of length `base` from (0, 0) along +x, the apex above (y up),
/// from the mode's sides and angles.
fn apex(mode: &str, p: &Value, base: f64, cmd: &str) -> Result<(f64, f64)> {
    let length = |k: &str| -> Result<f64> {
        match number_param(p, k, cmd)? {
            Some(v) if v > 0.0 && v <= MAX_MATH => Ok(v),
            Some(_) => Err(bad(cmd, format!("`{k}` must be a length above 0 and at most {MAX_MATH}"))),
            None => Err(bad(cmd, format!("mode {mode} needs `{k}`"))),
        }
    };
    let angle = |k: &str| -> Result<f64> {
        match number_param(p, k, cmd)? {
            Some(v) if v > 0.0 && v < 180.0 => Ok(v.to_radians()),
            Some(_) => Err(bad(cmd, format!("`{k}` must be an angle above 0 and below 180 degrees"))),
            None => Err(bad(cmd, format!("mode {mode} needs `{k}`"))),
        }
    };
    match mode {
        "SSS" => {
            let (b, c) = (length("left")?, length("right")?);
            if !(base < b + c && b < base + c && c < base + b) {
                return Err(bad(cmd, format!("no triangle has the sides {base}, {b} and {c}")));
            }
            let x = (base * base + b * b - c * c) / (2.0 * base);
            Ok((x, (b * b - x * x).max(0.0).sqrt()))
        }
        "SAS" => {
            let (b, t) = (length("left")?, angle("leftAngle")?);
            Ok((b * t.cos(), b * t.sin()))
        }
        _ => {
            let (al, be) = (angle("leftAngle")?, angle("rightAngle")?);
            if al + be >= std::f64::consts::PI - 1e-12 {
                return Err(bad(cmd, "`leftAngle` and `rightAngle` must add up to less than 180 degrees"));
            }
            // The left side, by the law of sines.
            let b = base * be.sin() / (al + be).sin();
            Ok((b * al.cos(), b * al.sin()))
        }
    }
}

fn triangle(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.triangle";
    let look = Look::of(p, C, LineRef::Solid(3))?;
    let mode = text_param(p, "mode", C)?.ok_or_else(|| bad(C, "missing `mode`: \"SSS\", \"SAS\" or \"ASA\""))?;
    let takes: &[&str] = match mode {
        "SSS" => &["base", "left", "right"],
        "SAS" => &["base", "left", "leftAngle"],
        "ASA" => &["base", "leftAngle", "rightAngle"],
        _ => return Err(bad(C, "`mode` must be \"SSS\", \"SAS\" or \"ASA\"")),
    };
    if let Some(k) = ["left", "right", "leftAngle", "rightAngle"].into_iter().find(|k| !takes.contains(k) && p.get(*k).is_some_and(|v| !v.is_null()))
    {
        return Err(bad(C, format!("mode {mode} takes {} (not `{k}`)", takes.join(", "))));
    }
    let base = match number_param(p, "base", C)? {
        Some(v) if v > 0.0 && v <= MAX_MATH => v,
        Some(_) => return Err(bad(C, format!("`base` must be a length above 0 and at most {MAX_MATH}"))),
        None => return Err(bad(C, "missing `base`: the base's length")),
    };
    let (ax, ay) = apex(mode, p, base, C)?;
    let rotation = number_param(p, "rotation", C)?.unwrap_or(0.0);
    if rotation.abs() > 1e6 {
        return Err(bad(C, "`rotation` is out of range"));
    }
    let unit = match number_param(p, "unit", C)? {
        Some(u) if u > 0.0 && u <= super::MAX_UNIT => Some(u),
        Some(_) => return Err(bad(C, format!("`unit` must be above 0 and at most {} pt", super::MAX_UNIT))),
        None => None,
    };
    let seg = match segment_param(p, "baseSegment", C)? {
        Some(sg) => Some(sg),
        None => selected_segments(s, |n| !is_magenta(n))?.first().map(|(_, sg)| *sg),
    };
    // Where the base starts, its direction, and points per length.
    let (start, dir, k) = match seg {
        Some(sg) => (sg.a, sg.dir(), sg.len() / base),
        None => {
            let r = rotation.to_radians();
            (artboard_centre(&s.doc()?.doc), Vec2::new(r.cos(), -r.sin()), unit.unwrap_or(1.0))
        }
    };
    let up = Vec2::new(dir.y, -dir.x);
    let pts = [start, start + dir * (base * k), start + dir * (ax * k) + up * (ay * k)];
    emit(s, "Triangle by Sides and Angles", "Triangle", C, |d, style| {
        let n = look.node(d, style, PathData::single(SubPath::polyline(&pts, true)));
        let mut out = Map::new();
        out.insert("vertices".into(), pairs(&pts));
        Ok((vec![n], out))
    })
}

fn dots(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.dots";
    if !p.is_object() && !p.is_null() {
        return Err(bad(C, "the parameters must be an object"));
    }
    let color = color_param(p, "color", C)?.unwrap_or(Color::BLACK);
    let size = number_param(p, "dot", C)?;
    let (lo, hi) = vectorcraft_doc::mathdraw::limits::DOT;
    if size.is_some_and(|d| !(lo..=hi).contains(&d)) {
        return Err(bad(C, format!("`dot` must be a diameter from {lo} to {hi} pt")));
    }
    let targets = checked_ids_or_selection(s, p, C)?;
    let st = s.doc()?;
    let mut pts: Vec<Point> = vec![];
    if targets.is_empty() {
        pts.push(artboard_centre(&st.doc));
    } else {
        // Each place once (a closed path's end on its start, paths that meet).
        let mut seen = HashSet::new();
        for id in &targets {
            let Some(n) = st.doc.node(*id) else { continue };
            n.walk(&mut |c| {
                if let NodeKind::Path { path, guide: false, .. } = &c.kind {
                    for (_, _, a) in path.anchors() {
                        let key = ((a.p.x * 1e6).round() as i64, (a.p.y * 1e6).round() as i64);
                        if a.p.x.is_finite() && a.p.y.is_finite() && seen.insert(key) {
                            pts.push(a.p);
                        }
                    }
                }
            });
        }
        if pts.is_empty() {
            return Err(bad(C, "the objects have no anchor points"));
        }
        if pts.len() > MAX_DOTS {
            return Err(bad(C, format!("the objects have more than {MAX_DOTS} anchor points")));
        }
    }
    emit(s, "Dots at Anchor Points", "Dots", C, |d, style| {
        let mut style = style.clone();
        if let Some(size) = size {
            style.dot_size = size;
        }
        let dots: Vec<Node> = pts.iter().map(|at| dot(d, *at, &style, color, Color::WHITE)).collect();
        let mut out = Map::new();
        out.insert("dots".into(), ids(&dots));
        out.insert("points".into(), pairs(&pts));
        Ok((dots, out))
    })
}

fn circle_from_segment(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.circleFromSegment";
    let look = Look::of(p, C, LineRef::Solid(3))?;
    let diameter = match text_param(p, "mode", C)? {
        Some("radius") => false,
        Some("diameter") => true,
        Some(_) => return Err(bad(C, "`mode` must be \"radius\" or \"diameter\"")),
        None => return Err(bad(C, "missing `mode`: \"radius\" or \"diameter\"")),
    };
    let segs = segments(s, p, MAX_SEGMENTS, C)?;
    emit(s, "Circle from Segment", "Circles", C, |d, style| {
        let circles: Vec<Node> = segs
            .iter()
            .map(|sg| {
                let (c, r) = if diameter { (sg.mid(), sg.len() / 2.0) } else { (sg.a, sg.len()) };
                look.node(d, style, shapes::ellipse(Rect::new(c.x - r, c.y - r, c.x + r, c.y + r)))
            })
            .collect();
        let mut out = Map::new();
        out.insert("circles".into(), ids(&circles));
        Ok((circles, out))
    })
}

/// The similarity (move, turn, uniform scale; no reflection) that takes `from` onto `to`, start to
/// start and end to end.
fn similarity(from: Seg, to: Seg) -> Affine {
    let (f, t) = (from.b - from.a, to.b - to.a);
    let den = f.hypot2();
    // `t / f` as complex numbers: the turn and scale.
    let (a, b) = ((t.x * f.x + t.y * f.y) / den, (t.y * f.x - t.x * f.y) / den);
    Affine::translate(to.a.to_vec2()) * Affine::new([a, b, -b, a, 0.0, 0.0]) * Affine::translate(-from.a.to_vec2())
}

fn fractal(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.fractal";
    if !p.is_object() && !p.is_null() {
        return Err(bad(C, "the parameters must be an object"));
    }
    let levels = match p.get("levels").filter(|v| !v.is_null()) {
        None => return Err(bad(C, "missing `levels`: 1 to 8")),
        Some(v) => v
            .as_u64()
            .filter(|n| (1..=MAX_FRACTAL_LEVELS).contains(n))
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| bad(C, format!("`levels` must be a whole number from 1 to {MAX_FRACTAL_LEVELS}")))?,
    };
    let base_param = segment_param(p, "baseSegment", C)?;
    let places_param = segments_param(p, "placeSegments", MAX_SEGMENTS, C)?;
    let given_targets = match p.get("targetIds").filter(|v| !v.is_null()) {
        Some(_) => Some(checked_ids_param(s, p, "targetIds", C)?),
        None => None,
    };
    let roots = selected_roots(s)?;
    let st = s.doc()?;
    // The selected segments (at the top of the selection) and their roles by colour.
    let seg_of = |id: &NodeId| st.doc.node(*id).and_then(|n| segment_ends(n).map(|(a, b)| (*id, Seg { a, b }, is_magenta(n), is_yellow(n))));
    let segs: Vec<(NodeId, Seg, bool, bool)> = roots.iter().filter_map(seg_of).collect();
    let base = match base_param {
        Some(sg) => sg,
        None => segs.iter().find(|s| s.2).map(|s| s.1).ok_or_else(|| bad(C, "select a magenta base segment (or give `baseSegment`)"))?,
    };
    let places = match places_param {
        Some(v) => v,
        None => segs.iter().filter(|s| s.3).map(|s| s.1).collect(),
    };
    if places.is_empty() {
        return Err(bad(C, "select yellow place segments (or give `placeSegments`)"));
    }
    let helpers: HashSet<NodeId> = segs.iter().filter(|s| s.2 || s.3).map(|s| s.0).collect();
    let targets = given_targets.unwrap_or_else(|| roots.iter().filter(|id| !helpers.contains(id)).copied().collect());
    let originals: Vec<Node> = targets.iter().filter_map(|id| st.doc.node(*id).filter(|n| !n.is_layer()).cloned()).collect();
    if originals.is_empty() {
        return Err(bad(C, "select the objects to repeat (or give `targetIds`)"));
    }
    // What the copies come to, counting the objects inside groups, before making any.
    let per: usize = originals.iter().map(Node::count).sum();
    let mut total = 0usize;
    let mut at_level = per;
    for _ in 0..levels {
        at_level = at_level.saturating_mul(places.len());
        total = total.saturating_add(at_level);
    }
    if total > MAX_FRACTAL_NODES {
        return Err(bad(C, format!("the fractal would add {total} objects (at most {MAX_FRACTAL_NODES}): use fewer levels, places or objects")));
    }
    let maps: Vec<Affine> = places.iter().map(|to| similarity(base, *to)).collect();
    let parent = st.target_parent()?;
    s.edit("Fractal Figure", |d, _| {
        let mut done: Vec<Vec<Node>> = vec![];
        let mut prev = originals;
        for _ in 0..levels {
            let mut next = Vec::with_capacity(prev.len() * maps.len());
            for m in &maps {
                for n in &prev {
                    let mut c = d.reid(n);
                    // Stroke weights stay as drawn.
                    c.transform(*m, false);
                    next.push(c);
                }
            }
            done.push(std::mem::replace(&mut prev, next));
        }
        done.push(prev);
        // The first list is the originals.
        let per_level: Vec<usize> = done.iter().skip(1).map(Vec::len).collect();
        let copies: Vec<Node> = done.into_iter().skip(1).flatten().collect();
        let count = copies.len();
        let id = insert_output(d, parent, "Fractal", copies, C)?;
        Ok(json!({ "id": id.0, "count": count, "levels": per_level }))
    })
}

// ---------- keep inside a rectangle ----------

/// The bounds of `n` when it is a rectangle with sides along the axes: a closed path of four
/// corners joined by straight sides (a closing anchor on the first one is allowed).
fn axis_rect(n: &Node) -> Option<Rect> {
    let NodeKind::Path { path, guide: false, clipping: false, .. } = &n.kind else { return None };
    let [sp] = path.subpaths.as_slice() else { return None };
    if !sp.closed || (0..sp.segment_count()).any(|i| !sp.segment_is_line(i)) {
        return None;
    }
    let mut pts: Vec<Point> = sp.anchors.iter().map(|a| a.p).collect();
    if pts.len() == 5 && pts.first().zip(pts.last()).is_some_and(|(a, b)| (*a - *b).hypot() <= 1e-9) {
        pts.pop();
    }
    let [a, b, c, d] = pts.as_slice() else { return None };
    let r = Rect::from_points(*a, *c).union(Rect::from_points(*b, *d));
    let tol = 1e-6 * (1.0 + r.width().max(r.height()));
    let along_axis = |p: Point, q: Point| (p.x - q.x).abs() <= tol || (p.y - q.y).abs() <= tol;
    let corner = |p: Point| ((p.x - r.x0).abs() <= tol || (p.x - r.x1).abs() <= tol) && ((p.y - r.y0).abs() <= tol || (p.y - r.y1).abs() <= tol);
    let ok = r.width() > tol
        && r.height() > tol
        && [*a, *b, *c, *d].iter().all(|p| corner(*p))
        && along_axis(*a, *b)
        && along_axis(*b, *c)
        && along_axis(*c, *d)
        && along_axis(*d, *a);
    ok.then_some(r)
}

/// Where cubic `c` crosses the lines of `r`'s sides, as parameters strictly inside (0, 1).
fn crossings(c: &CubicBez, r: Rect, ts: &mut Vec<f64>) {
    for (coord, values) in [(0, [r.x0, r.x1]), (1, [r.y0, r.y1])] {
        let k = |p: Point| if coord == 0 { p.x } else { p.y };
        let (p0, p1, p2, p3) = (k(c.p0), k(c.p1), k(c.p2), k(c.p3));
        for v in values {
            let roots = kurbo::common::solve_cubic(p0 - v, 3.0 * (p1 - p0), 3.0 * (p2 - 2.0 * p1 + p0), p3 - 3.0 * p2 + 3.0 * p1 - p0);
            ts.extend(roots.into_iter().filter(|t| t.is_finite() && *t > 1e-12 && *t < 1.0 - 1e-12));
        }
    }
}

/// The parts of `sp` inside `r`, as open subpaths (`sp` itself when it lies wholly inside).
fn clip_subpath(sp: &SubPath, r: Rect, out: &mut Vec<SubPath>) {
    let tol = 1e-7 * (1.0 + r.width().max(r.height()));
    let inside = |p: Point| p.x >= r.x0 - tol && p.x <= r.x1 + tol && p.y >= r.y0 - tol && p.y <= r.y1 + tol;
    let n = sp.segment_count();
    if n == 0 || sp.anchors.iter().all(|a| inside(a.p) && inside(a.h_in) && inside(a.h_out)) {
        if sp.anchors.iter().all(|a| inside(a.p)) && !sp.anchors.is_empty() {
            out.push(sp.clone());
        }
        return;
    }
    let mut runs: Vec<Vec<CubicBez>> = vec![];
    let mut cur: Vec<CubicBez> = vec![];
    for i in 0..n {
        let c = sp.segment(i);
        let line = sp.segment_is_line(i);
        let mut ts = vec![0.0, 1.0];
        crossings(&c, r, &mut ts);
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|b, a| (*b - *a).abs() <= 1e-12);
        for w in ts.windows(2) {
            let [t0, t1] = [w.first().copied().unwrap_or(0.0), w.last().copied().unwrap_or(1.0)];
            if t1 - t0 <= 1e-12 {
                continue;
            }
            if inside(c.eval(0.5 * (t0 + t1))) {
                let piece = if line {
                    let (q0, q3) = (c.eval(t0), c.eval(t1));
                    CubicBez::new(q0, q0, q3, q3)
                } else {
                    c.subsegment(t0..t1)
                };
                cur.push(piece);
            } else if !cur.is_empty() {
                runs.push(std::mem::take(&mut cur));
            }
        }
    }
    if !cur.is_empty() {
        runs.push(cur);
    }
    // Round a closed path, the last part goes on into the first where they meet.
    if sp.closed && runs.len() >= 2 {
        let joins = match (runs.last().and_then(|r| r.last()), runs.first().and_then(|r| r.first())) {
            (Some(end), Some(start)) => (end.p3 - start.p0).hypot() <= 1e-9,
            _ => false,
        };
        if joins
            && let Some(mut last) = runs.pop()
            && let Some(first) = runs.first_mut()
        {
            last.append(first);
            *first = last;
        }
    }
    out.extend(runs.iter().filter(|r| !r.is_empty()).map(|r| curve_subpath(r)));
}

/// `path` cut to `r`: filled closed subpaths by a boolean intersection, the rest along its lines
/// and curves.
fn clip_path(path: &PathData, rule: FillRule, r: Rect, filled: bool) -> PathData {
    let mut out = vec![];
    let (mut rings, open): (Vec<&SubPath>, Vec<&SubPath>) = path.subpaths.iter().partition(|sp| sp.closed);
    if filled && !rings.is_empty() {
        let area = PathData::new(rings.iter().map(|sp| (*sp).clone()).collect());
        let within = area.control_bounds().is_some_and(|b| b.x0 >= r.x0 && b.x1 <= r.x1 && b.y0 >= r.y0 && b.y1 <= r.y1);
        if within {
            out.extend(area.subpaths);
            rings.clear();
        } else if let Ok(cut) = po::try_boolean(&area, rule, &shapes::rectangle(r), FillRule::NonZero, po::BoolOp::Intersect, po::DEFAULT_PRECISION) {
            out.extend(cut.subpaths);
            rings.clear();
        }
        // (A boolean that fails cuts the outlines instead.)
    }
    for sp in rings.into_iter().chain(open) {
        clip_subpath(sp, r, &mut out);
    }
    PathData::new(out)
}

/// The paths under `n` cut to `r` (by id), and what isn't a path.
fn collect_clips(n: &Node, r: Rect, skip: NodeId, clips: &mut Vec<(NodeId, PathData)>, skipped: &mut Vec<NodeId>) {
    if n.id == skip {
        return;
    }
    let filled = n.appearance.fill().is_some_and(|f| f.visible && !f.paint.is_none());
    match &n.kind {
        NodeKind::Path { guide: true, .. } | NodeKind::Path { clipping: true, .. } => skipped.push(n.id),
        NodeKind::Path { .. } | NodeKind::Compound { .. } => {
            if let Some((path, rule)) = super::super::pathops::node_path(n) {
                clips.push((n.id, clip_path(&path, rule, r, filled)));
            }
        }
        // Equations are groups of glyphs: left whole, as type is.
        NodeKind::Group { children, .. } if n.extras().math.is_none() => {
            for c in children {
                collect_clips(c, r, skip, clips, skipped);
            }
        }
        _ => skipped.push(n.id),
    }
}

fn clip_rect(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.clipRect";
    if !p.is_object() && !p.is_null() {
        return Err(bad(C, "the parameters must be an object"));
    }
    let keep_rect = flag_param(p, "keepRect", C)?.unwrap_or(false);
    let roots = selected_roots(s)?;
    let rect_param = match p.get("rectId").filter(|v| !v.is_null()) {
        Some(v) => Some(checked_id(s, v, C)?),
        None => None,
    };
    let mut targets = match p.get("ids").filter(|v| !v.is_null()) {
        Some(_) => checked_ids_param(s, p, "ids", C)?,
        None => roots.clone(),
    };
    let st = s.doc()?;
    let (rect_id, r) = match rect_param {
        Some(id) => {
            (id, st.doc.node(id).and_then(axis_rect).ok_or_else(|| bad(C, format!("object {} is not a rectangle with sides along the axes", id.0)))?)
        }
        None => roots
            .iter()
            .rev()
            .find_map(|id| st.doc.node(*id).and_then(axis_rect).map(|r| (*id, r)))
            .ok_or_else(|| bad(C, "select a rectangle with sides along the axes (or give `rectId`)"))?,
    };
    targets.retain(|id| *id != rect_id);
    let (mut clips, mut skipped) = (vec![], vec![]);
    for id in &targets {
        if let Some(n) = st.doc.node(*id) {
            collect_clips(n, r, rect_id, &mut clips, &mut skipped);
        }
    }
    if clips.is_empty() {
        return Err(bad(C, "there are no paths to cut (select them with the rectangle, or give `ids`)"));
    }
    s.edit("Keep Inside Rectangle", |d, sel| {
        let (mut clipped, mut removed) = (vec![], vec![]);
        for (id, path) in clips {
            if path.is_empty() {
                d.remove(id)?;
                sel.remove(id);
                removed.push(id.0);
                continue;
            }
            // A compound path's parts are paths painted by the compound.
            let parts: Option<Vec<Arc<Node>>> = matches!(d.node(id).map(|n| &n.kind), Some(NodeKind::Compound { .. })).then(|| {
                path.subpaths.iter().map(|sp| Arc::new(Node::path(d.alloc_id(), PathData::single(sp.clone()), Appearance::default()))).collect()
            });
            let n = d.node_mut(id).ok_or(EngineError::NoNode(id))?;
            match (&mut n.kind, parts) {
                (NodeKind::Compound { children, .. }, Some(parts)) => *children = parts,
                (NodeKind::Path { path: was, live, .. }, _) => {
                    *was = path;
                    // A live rectangle or ellipse cut is a plain path.
                    *live = None;
                }
                _ => {}
            }
            clipped.push(id.0);
        }
        let mut out = json!({ "clipped": clipped, "removed": removed, "skipped": skipped.iter().map(|i| i.0).collect::<Vec<_>>() });
        if keep_rect {
            out["rect"] = json!(rect_id.0);
        } else {
            d.remove(rect_id)?;
            sel.remove(rect_id);
        }
        Ok(out)
    })
}
