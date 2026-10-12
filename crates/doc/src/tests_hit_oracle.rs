//! The spatial index must not change what a click or a marquee finds: the indexed hit test,
//! `objects_at` and marquee against the functions as they were before the index (kept verbatim in
//! `brute`), on random documents with nested groups, clip groups, compound paths, sublayers,
//! hidden and locked objects, text, transforms and isolation mode, through random edits, undo and
//! redo (old versions asked about again).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use vectorcraft_color::{Color, Paint};
use vectorcraft_geom::{Affine, Point, Rect, shapes};

use crate::hit::{self, Hit, HitOptions};
use crate::{Appearance, Document, LayerColor, Node, NodeId, NodeKind};

/// The hit test and marquee before the spatial index, verbatim (bar paths to shared helpers).
mod brute {
    use crate::hit::{Hit, HitOptions, clip_contains, edits_contents, hit_leaf};
    use crate::{Document, Node, NodeId, NodeKind};
    use vectorcraft_geom::{Point, Rect};

    /// Topmost editable object under `p`.
    pub fn hit_test(doc: &Document, p: Point, opt: HitOptions) -> Option<Hit> {
        hit_test_skipping(doc, p, opt, &|_| false, true)
    }

    /// Topmost visible art for read-only sampling: locked objects and template layers participate,
    /// including art outside the editing scope. This never changes their editability.
    pub fn hit_test_visible(doc: &Document, p: Point, opt: HitOptions) -> Option<Hit> {
        hit_test_skipping(doc, p, opt, &|_| false, false)
    }

    /// The objects under `p` that [`Hit::top_object`] picks in `scope` (isolation mode), topmost
    /// first: what clicks there select, one below the other (Cmd/Ctrl-click selects behind).
    pub fn objects_at(doc: &Document, p: Point, opt: HitOptions, scope: Option<NodeId>) -> Vec<NodeId> {
        let opt = HitOptions { scope: scope.or(opt.scope), ..opt };
        let mut tops: Vec<NodeId> = vec![];
        // Each round leaves out the objects found so far (and everything inside them).
        while let Some(top) = hit_test_skipping(doc, p, opt, &|id| tops.contains(&id), true).map(|h| h.top_object(scope)) {
            if tops.contains(&top) {
                break;
            }
            tops.push(top);
        }
        tops
    }

    /// [`hit_test`] passing over the nodes `skip` names (and their contents).
    fn hit_test_skipping(doc: &Document, p: Point, opt: HitOptions, skip: &dyn Fn(NodeId) -> bool, editable_only: bool) -> Option<Hit> {
        let finish = |mut h: Hit| {
            h.contents_of = h.ancestry.iter().rev().copied().find(|a| doc.node(*a).is_some_and(edits_contents));
            h.layers = h.ancestry.iter().take_while(|a| doc.node(**a).is_some_and(Node::is_layer)).count();
            h
        };
        // Isolation mode: only what is inside the isolated container hits. The art around it is
        // locked, and clicks pass through it (#833).
        if editable_only
            && let Some(scope) = opt.scope
            && let Some(container) = doc.node(scope).filter(|n| n.is_container())
            && let Some(mut chain) = doc.ancestry(scope)
        {
            return hit_children(container, p, opt, skip, &mut chain, editable_only).map(finish);
        }
        let mut chain = Vec::new();
        for layer in doc.layers.iter().rev() {
            if !layer.visible || editable_only && (layer.locked || layer.is_template()) {
                continue;
            }
            // Pattern editing mode: only the tile art is editable.
            if editable_only && doc.pattern_edit.as_ref().is_some_and(|e| e.layer != layer.id) {
                continue;
            }
            chain.push(layer.id);
            let opt = if matches!(layer.kind, NodeKind::Layer { preview: false, .. }) { HitOptions { outline: true, ..opt } } else { opt };
            if let Some(h) = hit_children(layer, p, opt, skip, &mut chain, editable_only) {
                return Some(finish(h));
            }
            chain.pop();
        }
        None
    }

    fn hit_children(
        parent: &Node,
        p: Point,
        opt: HitOptions,
        skip: &dyn Fn(NodeId) -> bool,
        chain: &mut Vec<NodeId>,
        editable_only: bool,
    ) -> Option<Hit> {
        let children = parent.children()?;
        // A clip group (or a layer with a clipping mask) only hits inside its clipping path.
        if parent.clips()
            && let Some(clip) = children.first()
            && !clip_contains(clip, p)
        {
            return None;
        }
        for (i, c) in children.iter().enumerate().rev() {
            if parent.extras().shaper.is_some() {
                let construction = children.first().is_some_and(|source| Some(source.id) == opt.scope);
                if (i == 0) != construction {
                    continue;
                }
            }
            // Read-only samples can read locked/template art; hidden art never participates.
            if !c.visible || editable_only && (c.locked || c.is_template()) || skip(c.id) {
                continue;
            }
            // (An envelope's content sits where it was, not where the envelope draws it.)
            if !(edits_contents(c)
                || c.extras().shaper.is_some()
                    && c.children().and_then(|children| children.first()).is_some_and(|source| Some(source.id) == opt.scope))
                && let Some(b) = c.reach_bounds()
                && !b.inflate(opt.tol, opt.tol).contains(p)
            {
                continue;
            }
            chain.push(c.id);
            let hit = match &c.kind {
                // A sublayer whose Preview is off is clicked in outline, like an outline view.
                NodeKind::Layer { preview: false, .. } => hit_children(c, p, HitOptions { outline: true, ..opt }, skip, chain, editable_only),
                NodeKind::Layer { .. } | NodeKind::Group { .. } => hit_children(c, p, opt, skip, chain, editable_only),
                // Edit Contents: the envelope's content hits, undistorted.
                NodeKind::Envelope { editing: true, .. } => hit_children(c, p, opt, skip, chain, editable_only),
                // A blend's key objects first (Direct and Group Selection pick them); its steps hit
                // as the blend.
                NodeKind::Blend { .. } => hit_children(c, p, opt, skip, chain, editable_only)
                    .or_else(|| hit_leaf(c, p, opt).map(|kind| Hit { leaf: c.id, ancestry: chain.clone(), kind, contents_of: None, layers: 1 })),
                _ => hit_leaf(c, p, opt).map(|kind| Hit { leaf: c.id, ancestry: chain.clone(), kind, contents_of: None, layers: 1 }),
            };
            if hit.is_some() {
                return hit;
            }
            chain.pop();
        }
        None
    }

    /// Objects (children of layers, or of `scope` in isolation mode) touched by a marquee rect.
    pub fn marquee(doc: &Document, r: Rect, scope: Option<NodeId>, leaves: bool) -> Vec<NodeId> {
        let mut out = Vec::new();
        let tops: Vec<&std::sync::Arc<Node>> = match scope.and_then(|s| doc.node(s)) {
            Some(s) => s.children().map(|c| c.iter().collect()).unwrap_or_default(),
            // The objects of the layers, looking through sublayers (hidden, locked and template ones
            // are left out).
            None => doc
                .layers
                .iter()
                .filter(|l| l.visible && !l.locked && !l.is_template() && doc.pattern_edit.as_ref().is_none_or(|e| e.layer == l.id))
                .flat_map(|l| layer_objects(l))
                .collect(),
        };
        fn layer_objects(l: &Node) -> Vec<&std::sync::Arc<Node>> {
            let mut out = vec![];
            for c in l.children().into_iter().flatten() {
                if c.is_layer() {
                    if c.visible && !c.locked && !c.is_template() {
                        out.extend(layer_objects(c));
                    }
                } else {
                    out.push(c);
                }
            }
            out
        }
        fn touches(n: &Node, r: Rect) -> bool {
            match &n.kind {
                NodeKind::Path { path, .. } => vectorcraft_geom::hit::intersects_rect(path, r),
                NodeKind::Layer { children, .. } | NodeKind::Group { children, .. } | NodeKind::Compound { children, .. } => {
                    children.iter().any(|c| c.visible && touches(c, r))
                }
                _ => n.geometric_bounds().is_some_and(|b| b.intersect(r).area() > 0.0 || r.contains(b.origin())),
            }
        }
        fn collect_leaves(n: &Node, r: Rect, out: &mut Vec<NodeId>) {
            match &n.kind {
                NodeKind::Layer { children, .. } | NodeKind::Group { children, .. } => {
                    for c in children {
                        if c.visible && !c.locked {
                            collect_leaves(c, r, out);
                        }
                    }
                }
                _ => {
                    if touches(n, r) {
                        out.push(n.id);
                    }
                }
            }
        }
        for t in tops {
            if !t.visible || t.locked {
                continue;
            }
            if leaves {
                collect_leaves(t, r, &mut out);
            } else if touches(t, r) {
                out.push(t.id);
            }
        }
        out
    }
}

/// Deterministic xorshift.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (self.next() % 1_000_000) as f64 / 1_000_000.0 * (hi - lo)
    }
    fn n(&mut self, k: u64) -> u64 {
        self.next() % k.max(1)
    }
    fn chance(&mut self, k: u64) -> bool {
        self.n(k) == 0
    }
}

const PAGE: f64 = 600.0;

/// A random leaf: filled or stroked shapes (some wide strokes), unfilled outlines, rotated
/// shapes, point type, a compound path; sometimes hidden or locked.
fn leaf(d: &mut Document, r: &mut Rng) -> Node {
    let (x, y, s) = (r.f(-20.0, PAGE), r.f(-20.0, PAGE), r.f(2.0, 40.0));
    let fill = if r.chance(3) { Paint::None } else { Paint::solid(Color::rgb(0.2, 0.4, 0.6)) };
    let stroke = if r.chance(2) { Paint::solid(Color::BLACK) } else { Paint::None };
    let ap = Appearance::basic(fill, stroke, if r.chance(5) { r.f(5.0, 30.0) } else { r.f(0.0, 3.0) });
    let id = d.alloc_id();
    let mut n = match r.n(6) {
        0 => Node::path(id, shapes::ellipse(Rect::new(x, y, x + s, y + s * 0.7)), ap),
        1 => Node::path(id, shapes::star(Point::new(x, y), s, s / 2.0, 5, r.f(0.0, 90.0)), ap),
        2 => Node::path(id, shapes::line(Point::new(x, y), Point::new(x + s, y + r.f(-s, s))), ap),
        3 => {
            let t = crate::TextObject::point(Point::new(x, y), "Type here", crate::CharStyle::default());
            Node::new(id, NodeKind::Text(Box::new(t)))
        }
        4 => {
            let a = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(x, y, x + s, y + s)), Appearance::default());
            let b =
                Node::path(d.alloc_id(), shapes::ellipse(Rect::new(x + s * 0.25, y + s * 0.25, x + s * 0.75, y + s * 0.75)), Appearance::default());
            let mut c = Node::new(id, NodeKind::Compound { children: vec![Arc::new(a), Arc::new(b)], rule: vectorcraft_geom::FillRule::EvenOdd });
            c.appearance = ap.into();
            c
        }
        _ => Node::path(id, shapes::rectangle(Rect::new(x, y, x + s, y + s * 0.5)), ap),
    };
    if r.chance(4) {
        n.transform(Affine::rotate_about(r.f(0.0, 3.0), Point::new(x, y)), false);
    }
    n.visible = !r.chance(12);
    n.locked = r.chance(12);
    n
}

/// A random object: a leaf, or a group (some clip groups, some large enough to be indexed).
fn object(d: &mut Document, r: &mut Rng, depth: u32) -> Node {
    if depth > 2 || !r.chance(5) {
        return leaf(d, r);
    }
    let big = r.chance(4);
    let n = if big { 64 + r.n(40) } else { 1 + r.n(6) } as usize;
    let kids: Vec<Arc<Node>> = (0..n).map(|_| Arc::new(object(d, r, depth + 1))).collect();
    let id = d.alloc_id();
    let mut g = Node::group(id, kids);
    if let NodeKind::Group { clip, .. } = &mut g.kind {
        *clip = !big && r.chance(3);
    }
    g.visible = !r.chance(15);
    g.locked = r.chance(15);
    g
}

/// A document of `n` top-level objects in one layer, with a sublayer of its own objects (and a
/// second, sometimes hidden or outline, layer).
fn document(r: &mut Rng, n: usize) -> Document {
    let mut d = Document::new(PAGE, PAGE);
    let l = d.layers[0].id;
    for _ in 0..n {
        let o = object(&mut d, r, 0);
        d.insert(Some(l), usize::MAX, o).unwrap();
    }
    let sub = d.alloc_id();
    d.insert(Some(l), (n / 2).max(1), Node::layer(sub, "Sub", LayerColor::Preset(1))).unwrap();
    let m = if r.chance(2) { 70 } else { 5 };
    for _ in 0..m {
        let o = object(&mut d, r, 1);
        d.insert(Some(sub), usize::MAX, o).unwrap();
    }
    let l2 = d.add_layer(None);
    for _ in 0..(n / 3) {
        let o = object(&mut d, r, 0);
        d.insert(Some(l2), usize::MAX, o).unwrap();
    }
    if let Some(n) = d.node_mut(l2) {
        n.visible = !r.chance(4);
        // A locked layer: clicks pass it, read-only sampling reads it.
        n.locked = r.chance(4);
        if let NodeKind::Layer { preview, .. } = &mut n.kind {
            *preview = !r.chance(3);
        }
    }
    d
}

/// Every container id (isolation scopes to try).
fn containers(d: &Document) -> Vec<NodeId> {
    let mut v = vec![];
    d.walk(|n| {
        if n.is_container() {
            v.push(n.id);
        }
    });
    v
}

fn assert_same(d: &Document, r: &mut Rng, what: &str) {
    let scopes = containers(d);
    for k in 0..24 {
        // Points anywhere, and on the edges of objects.
        let p = if k % 2 == 0 {
            Point::new(r.f(-30.0, PAGE + 30.0), r.f(-30.0, PAGE + 30.0))
        } else {
            let ids: Vec<NodeId> = d.layers[0].children().map(|c| c.iter().map(|n| n.id).collect()).unwrap_or_default();
            let id = ids[r.n(ids.len() as u64) as usize];
            let b = d.node(id).and_then(Node::geometric_bounds).unwrap_or_default();
            Point::new(if r.chance(2) { b.x0 } else { b.x1 } + r.f(-1.0, 1.0), r.f(b.y0, b.y1))
        };
        let scope = if r.chance(5) { scopes.get(r.n(scopes.len() as u64) as usize).copied() } else { None };
        let opt = HitOptions { tol: r.f(0.0, 8.0), outline: r.chance(6), path_only: r.chance(6), type_path_only: r.chance(6), scope };
        let (a, b): (Option<Hit>, Option<Hit>) = (hit::hit_test(d, p, opt), brute::hit_test(d, p, opt));
        assert_eq!(a, b, "{what}: hit_test at {p:?} with {opt:?}");
        let (a, b) = (hit::hit_test_visible(d, p, opt), brute::hit_test_visible(d, p, opt));
        assert_eq!(a, b, "{what}: hit_test_visible at {p:?} with {opt:?}");
        if k % 3 == 0 {
            assert_eq!(hit::objects_at(d, p, opt, scope), brute::objects_at(d, p, opt, scope), "{what}: objects_at {p:?} {opt:?}");
        }
        let (x, y) = (r.f(-50.0, PAGE), r.f(-50.0, PAGE));
        let rect = Rect::new(x, y, x + r.f(0.0, 200.0), y + r.f(0.0, 150.0));
        let leaves = r.chance(2);
        let picked = brute::marquee(d, rect, scope, leaves);
        assert_eq!(hit::marquee(d, rect, scope, leaves), picked, "{what}: marquee {rect:?} {scope:?} {leaves}");
        // What the marquee picks is among the objects found in its rectangle.
        if !leaves {
            let found: Vec<NodeId> = hit::objects_in(d, rect, scope).iter().map(|n| n.id).collect();
            assert!(picked.iter().all(|id| found.contains(id)), "{what}: objects_in {rect:?} {scope:?}");
        }
    }
}

/// A random edit: move, delete, add on top or below, hide or lock, regroup, or change a member of
/// a large group.
fn edit(d: &mut Document, r: &mut Rng) {
    let l = d.layers[0].id;
    let ids: Vec<NodeId> = d.layers[0].children().map(|c| c.iter().map(|n| n.id).collect()).unwrap_or_default();
    let pick = |r: &mut Rng| ids[r.n(ids.len() as u64) as usize];
    match r.n(7) {
        0 => {
            for _ in 0..1 + r.n(5) {
                let id = pick(r);
                let (dx, dy) = (r.f(-100.0, 100.0), r.f(-100.0, 100.0));
                d.node_mut(id).unwrap().transform(Affine::translate((dx, dy)), false);
            }
        }
        1 => {
            let _ = d.remove(pick(r));
        }
        2 => {
            let o = object(d, r, 0);
            d.insert(Some(l), usize::MAX, o).unwrap();
        }
        3 => {
            let o = object(d, r, 0);
            let at = r.n(ids.len() as u64) as usize;
            d.insert(Some(l), at, o).unwrap();
        }
        4 => {
            let n = d.node_mut(pick(r)).unwrap();
            if r.chance(2) {
                n.visible = !n.visible;
            } else {
                n.locked = !n.locked;
            }
        }
        5 => {
            // Every object moves (a nudge of everything).
            for id in ids {
                d.node_mut(id).unwrap().transform(Affine::translate((1.0, 0.0)), false);
            }
        }
        _ => {
            // A member deep inside changes.
            let mut deep = vec![];
            d.walk(|n| {
                if !n.is_container() {
                    deep.push(n.id);
                }
            });
            if let Some(&id) = deep.get(r.n(deep.len() as u64) as usize) {
                d.node_mut(id).unwrap().transform(Affine::translate((r.f(-30.0, 30.0), 3.0)), false);
            }
        }
    }
}

/// Documents the oracle tries (each through a few edits, undo and redo).
const SEEDS: u64 = 6;

#[test]
fn indexed_hit_testing_finds_what_looking_at_everything_finds() {
    for seed in 1..=SEEDS {
        let mut r = Rng(0x9e37_79b9_7f4a_7c15 ^ seed.wrapping_mul(7919));
        let n = 80 + r.n(120) as usize;
        let mut d = document(&mut r, n);
        assert_same(&d, &mut r, &format!("seed {seed}"));
        let mut history = vec![d.clone()];
        for step in 0..6 {
            edit(&mut d, &mut r);
            assert_same(&d, &mut r, &format!("seed {seed} edit {step}"));
            history.push(d.clone());
        }
        // Undo and redo: older versions (sharing the index with the newer ones) asked again.
        for (i, old) in history.iter().enumerate().rev() {
            assert_same(old, &mut r, &format!("seed {seed} undo to {i}"));
        }
        assert_same(&d, &mut r, &format!("seed {seed} redo"));
        // The indexes followed every version they were asked about as a fresh build would.
        for v in history.iter().chain([&d]) {
            for c in containers(v) {
                if let Some(n) = v.node_arc(c) {
                    v.spatial.check(n);
                }
            }
        }
        assert!(d.spatial.stats().0 >= 1, "seed {seed}: the large layer is indexed");
    }
}

/// The cached bounds equal the bounds measured afresh after every kind of edit, undo and redo.
#[test]
fn cached_bounds_equal_measured_bounds_through_edits() {
    let bits = |r: Option<Rect>| r.map(|r| [r.x0, r.y0, r.x1, r.y1].map(f64::to_bits));
    let check = |d: &Document, what: &str| {
        fn walk(nodes: &[Arc<Node>], f: &mut dyn FnMut(&Arc<Node>)) {
            for n in nodes {
                f(n);
                if let Some(c) = n.children() {
                    walk(c, f);
                }
            }
        }
        walk(&d.layers, &mut |n| {
            assert_eq!(bits(crate::bounds_cache::geometric(n)), bits(n.geometric_bounds()), "{what}: geometric {}", n.id);
            assert_eq!(bits(crate::bounds_cache::visual(n)), bits(n.visual_bounds()), "{what}: visual {}", n.id);
            assert_eq!(bits(crate::bounds_cache::reach(n)), bits(n.reach_bounds()), "{what}: reach {}", n.id);
            assert_eq!(bits(crate::bounds_cache::pick(n)), bits(n.pick_bounds()), "{what}: pick {}", n.id);
        });
    };
    for seed in 1..=6u64 {
        let mut r = Rng(0x2545_f491_4f6c_dd1d ^ seed);
        let mut d = document(&mut r, 90);
        check(&d, "built");
        let mut history = vec![d.clone()];
        for step in 0..10 {
            edit(&mut d, &mut r);
            check(&d, &format!("seed {seed} edit {step}"));
            history.push(d.clone());
        }
        for (i, old) in history.iter().enumerate() {
            check(old, &format!("seed {seed} undo to {i}"));
        }
    }
}
