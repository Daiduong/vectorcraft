//! The bounds cache and the spatial index stay right through commands, drags, undo and redo: the
//! cached bounds of every object equal its bounds measured again, and clicks and marquees find
//! with the index kept up to date what they find with one built afresh.

use serde_json::{Value, json};
use vectorcraft_doc::hit::{self, HitOptions};
use vectorcraft_doc::{Document, Node, bounds_cache};
use vectorcraft_geom::{Point, Rect};
use vectorcraft_tools::{PointerEvent, PointerKind};

use super::*;

/// A page of 120 rectangles and ellipses (enough for the layer to be indexed).
fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    for i in 0..120 {
        let (x, y) = ((i % 12) as f64 * 60.0 + 10.0, (i / 12) as f64 * 55.0 + 10.0);
        let shape = if i % 2 == 0 { "shape.rectangle" } else { "shape.ellipse" };
        s.execute(shape, &json!({"x": x, "y": y, "width": 30, "height": 20})).unwrap();
    }
    s
}

fn bits(r: Option<Rect>) -> Option<[u64; 4]> {
    r.map(|r| [r.x0, r.y0, r.x1, r.y1].map(f64::to_bits))
}

/// Every object's cached bounds are its bounds; the index finds what a fresh one finds.
fn check(d: &Document, what: &str) {
    fn walk(nodes: &[Arc<Node>], f: &mut dyn FnMut(&Arc<Node>)) {
        for n in nodes {
            f(n);
            if let Some(c) = n.children() {
                walk(c, f);
            }
        }
    }
    walk(&d.layers, &mut |n| {
        assert_eq!(bits(bounds_cache::geometric(n)), bits(n.geometric_bounds()), "{what}: geometric bounds of {}", n.id);
        assert_eq!(bits(bounds_cache::visual(n)), bits(n.visual_bounds()), "{what}: visual bounds of {}", n.id);
        assert_eq!(bits(bounds_cache::reach(n)), bits(n.reach_bounds()), "{what}: reach of {}", n.id);
    });
    let probes: Vec<Point> = (0..40).map(|i| Point::new((i * 37 % 800) as f64, (i * 53 % 600) as f64)).collect();
    let marquees: Vec<Rect> = (0..8).map(|i| Rect::new(i as f64 * 90.0, i as f64 * 60.0, i as f64 * 90.0 + 150.0, i as f64 * 60.0 + 120.0)).collect();
    let ask = |d: &Document| {
        let hits: Vec<_> = probes.iter().map(|p| hit::hit_test(d, *p, HitOptions::default())).collect();
        let tops: Vec<_> = probes.iter().map(|p| hit::objects_at(d, *p, HitOptions::default(), None)).collect();
        let boxes: Vec<_> = marquees.iter().map(|r| hit::marquee(d, *r, None, false)).collect();
        (hits, tops, boxes)
    };
    let kept = ask(d);
    let fresh = d.clone();
    fresh.spatial().clear();
    assert_eq!(kept, ask(&fresh), "{what}: the index kept up to date finds what a fresh one finds");
}

#[test]
fn bounds_and_index_follow_edits_drags_undo_and_redo() {
    let mut s = session();
    let doc = |s: &Session| s.doc().unwrap().doc.clone();
    check(&doc(&s), "built");
    assert!(s.doc().unwrap().doc.spatial().stats().0 >= 1, "the layer is indexed");
    s.execute("select.all", &json!({})).unwrap();
    s.execute("object.move", &json!({"dx": 5, "dy": 3})).unwrap();
    check(&doc(&s), "moved everything");
    let ids: Vec<u64> = s.doc().unwrap().selection.objects.iter().map(|id| id.0).collect();
    s.execute("select.set", &json!({"ids": ids[..6]})).unwrap();
    s.execute("object.group", &json!({})).unwrap();
    check(&doc(&s), "grouped");
    s.execute("object.move", &json!({"dx": -40, "dy": 10})).unwrap();
    check(&doc(&s), "moved the group");
    s.execute("select.set", &json!({"ids": [ids[50]]})).unwrap();
    s.execute("edit.clear", &json!({})).unwrap();
    check(&doc(&s), "deleted one below others");
    // A drag with the Selection tool: each preview frame, then the drop.
    s.execute("select.set", &json!({"ids": [ids[70]]})).unwrap();
    let at = s.doc().unwrap().doc.node(NodeId(ids[70])).unwrap().geometric_bounds().unwrap().center();
    let v = ViewInfo::default();
    s.select_tool("selection", v).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Down, at.x, at.y), v).unwrap();
    for k in 1..6 {
        s.pointer(&PointerEvent::new(PointerKind::Drag, at.x + 7.0 * k as f64, at.y + 3.0), v).unwrap();
        check(&doc(&s), &format!("drag frame {k}"));
    }
    s.pointer(&PointerEvent::new(PointerKind::Up, at.x + 35.0, at.y + 3.0), v).unwrap();
    check(&doc(&s), "dropped");
    for i in 0..4 {
        s.execute("edit.undo", &json!({})).unwrap();
        check(&doc(&s), &format!("undo {i}"));
    }
    for i in 0..4 {
        s.execute("edit.redo", &json!({})).unwrap();
        check(&doc(&s), &format!("redo {i}"));
    }
}

/// Shift-adding to a large selection and toggling targets stay as they were, with the membership
/// set kept beside the list.
#[test]
fn large_selections_answer_membership_like_the_list() {
    let mut s = session();
    let all: Vec<u64> = s.doc().unwrap().doc.selectable_art().iter().map(|id| id.0).collect();
    s.execute("select.set", &json!({"ids": all[..60]})).unwrap();
    for id in &all[50..80] {
        s.execute("select.toggle", &json!({"id": id})).unwrap();
    }
    let sel = &s.doc().unwrap().selection;
    let want: Vec<NodeId> = all[..50].iter().chain(&all[60..80]).map(|i| NodeId(*i)).collect();
    assert_eq!(sel.objects, want);
    assert!(all.iter().all(|i| sel.contains(NodeId(*i)) == want.contains(&NodeId(*i))));
    s.execute("select.inverse", &json!({})).unwrap();
    let sel = &s.doc().unwrap().selection;
    assert_eq!(sel.len(), all.len() - want.len());
    assert!(want.iter().all(|i| !sel.contains(*i)));
}

fn doc(s: &Session) -> Arc<Document> {
    s.doc().unwrap().doc.clone()
}

fn id_of(v: &Value) -> NodeId {
    NodeId(v["id"].as_u64().unwrap())
}

/// Run command `id`, then check.
fn run(s: &mut Session, what: &str, id: &str, p: Value) -> Value {
    let r = s.execute(id, &p).unwrap_or_else(|e| panic!("{what}: {id}: {e}"));
    check(&doc(s), what);
    r
}

/// Run command `id`, undo it and redo it, checking after each.
fn step(s: &mut Session, what: &str, id: &str, p: Value) -> Value {
    let r = run(s, what, id, p);
    run(s, &format!("{what}, undone"), "edit.undo", json!({}));
    run(s, &format!("{what}, redone"), "edit.redo", json!({}));
    r
}

/// Undo `n` steps and redo them, checking after each.
fn undo_redo(s: &mut Session, what: &str, n: usize) {
    for i in 0..n {
        run(s, &format!("{what}: undo {i}"), "edit.undo", json!({}));
    }
    for i in 0..n {
        run(s, &format!("{what}: redo {i}"), "edit.redo", json!({}));
    }
}

/// A press at `from`, drags to `to` in `frames` frames and a release with `tool`, checking every
/// frame.
fn drag(s: &mut Session, what: &str, tool: &str, from: Point, to: Point, frames: usize) {
    let v = ViewInfo::default();
    s.select_tool(tool, v).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Down, from.x, from.y), v).unwrap();
    for k in 1..=frames {
        let t = k as f64 / frames as f64;
        s.pointer(&PointerEvent::new(PointerKind::Drag, from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t), v).unwrap();
        check(&doc(s), &format!("{what}: frame {k}"));
    }
    s.pointer(&PointerEvent::new(PointerKind::Up, to.x, to.y), v).unwrap();
    assert!(!s.in_interaction(), "{what}: the drag ended");
    check(&doc(s), &format!("{what}: dropped"));
}

/// A click at `at` with `tool`.
fn click(s: &mut Session, tool: &str, at: Point) {
    let v = ViewInfo::default();
    s.select_tool(tool, v).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Down, at.x, at.y), v).unwrap();
    s.pointer(&PointerEvent::new(PointerKind::Up, at.x, at.y), v).unwrap();
}

fn geometric(s: &Session, id: NodeId) -> Option<Rect> {
    s.doc().unwrap().doc.node(id).and_then(Node::geometric_bounds)
}

fn visual(s: &Session, id: NodeId) -> Option<Rect> {
    s.doc().unwrap().doc.node(id).and_then(Node::visual_bounds)
}

/// The ids of the objects of the first layer, bottom first.
fn layer_ids(s: &Session) -> Vec<u64> {
    s.doc().unwrap().doc.layers[0].children().unwrap().iter().map(|n| n.id.0).collect()
}

/// Live effects that change the bounds: applied, edited, hidden, shown, removed and expanded,
/// on an object of an indexed layer and on a group inside it, each undone and redone.
#[test]
fn bounds_and_index_follow_live_effects() {
    let mut s = session();
    let ids = layer_ids(&s);
    let (a, b) = (NodeId(ids[13]), NodeId(ids[14]));
    let before = visual(&s, a);
    for (effect, params) in [
        ("path.offsetPath", json!({"offset": 12})),
        ("distort.roughen", json!({"size": 8})),
        ("stylize.dropShadow", json!({"x": 20, "y": 25, "blur": 4})),
    ] {
        step(&mut s, &format!("apply {effect}"), "effect.apply", json!({"effect": effect, "params": params, "ids": [a.0], "item": null}));
    }
    // (The document's bounds leave live effects out: the renderer works them out.)
    assert_eq!(visual(&s, a), before);
    assert_eq!(s.doc().unwrap().doc.node(a).unwrap().appearance.effects.len(), 3);
    step(&mut s, "offset edited", "effect.setParams", json!({"index": 0, "params": {"offset": 30}, "ids": [a.0], "item": null}));
    step(&mut s, "shadow edited", "effect.setParams", json!({"index": 2, "params": {"x": -40, "y": 60}, "ids": [a.0], "item": null}));
    step(&mut s, "offset hidden", "effect.setParams", json!({"index": 0, "visible": false, "ids": [a.0], "item": null}));
    step(&mut s, "offset shown", "effect.setParams", json!({"index": 0, "visible": true, "ids": [a.0], "item": null}));
    step(&mut s, "roughen removed", "effect.remove", json!({"index": 1, "ids": [a.0], "item": null}));
    // On a group of two: the group's bounds come from its members' and its own effects.
    run(&mut s, "select a and b", "select.set", json!({"ids": [a.0, b.0]}));
    let g = id_of(&step(&mut s, "grouped", "object.group", json!({})));
    step(&mut s, "group offset", "effect.apply", json!({"effect": "path.offsetPath", "params": {"offset": 15}, "ids": [g.0], "item": null}));
    step(&mut s, "member moved in the group", "object.move", json!({"ids": [b.0], "dx": 30, "dy": -12}));
    run(&mut s, "select b", "select.set", json!({"ids": [b.0]}));
    step(&mut s, "member moved in the group", "object.move", json!({"dx": 30, "dy": -12}));
    run(&mut s, "select a", "select.set", json!({"ids": [a.0]}));
    step(&mut s, "member's effect edited", "effect.setParams", json!({"index": 0, "params": {"offset": 5}, "item": null}));
    // Expand Appearance turns the effects into objects.
    run(&mut s, "select the group", "select.set", json!({"ids": [g.0]}));
    step(&mut s, "expanded", "effect.expandAppearance", json!({"ids": [g.0]}));
    undo_redo(&mut s, "effects", 12);
}

/// Compound paths made, edited (a member's anchor dragged with Direct Selection, a member's
/// direction reversed, the fill rule set), and released, in an indexed layer.
#[test]
fn bounds_and_index_follow_compound_paths() {
    let mut s = session();
    let square = id_of(&run(&mut s, "square", "path.create", json!({"d": "M300 300 L400 300 L400 400 L300 400 Z"})));
    let diamond = id_of(&run(&mut s, "diamond", "path.create", json!({"d": "M350 280 L420 350 L350 420 L280 350 Z"})));
    run(&mut s, "select both", "select.set", json!({"ids": [square.0, diamond.0]}));
    let c = id_of(&step(&mut s, "compound made", "object.compoundPath.make", json!({})));
    let made = geometric(&s, c).unwrap();
    // #1197: the overlap is a hole, which the hit test (and so the index) must agree on.
    step(&mut s, "a member reversed", "path.reverse", json!({"ids": [diamond.0]}));
    step(&mut s, "evenOdd", "path.setFillRule", json!({"rule": "evenOdd", "ids": [c.0]}));
    step(&mut s, "nonZero", "path.setFillRule", json!({"rule": "nonZero", "ids": [c.0]}));
    // Direct Selection: click the square's corner, then drag it out.
    run(&mut s, "nothing selected", "select.none", json!({}));
    let corner = Point::new(300.0, 300.0);
    click(&mut s, "directSelection", corner);
    check(&doc(&s), "anchor clicked");
    drag(&mut s, "corner dragged", "directSelection", corner, Point::new(250.0, 240.0), 4);
    let dragged = geometric(&s, c).unwrap();
    assert_ne!(dragged, made, "the drag moved a member's anchor");
    assert!(dragged.x0 < made.x0 - 20.0 && dragged.y0 < made.y0 - 20.0, "{dragged:?} {made:?}");
    undo_redo(&mut s, "anchor drag", 1);
    run(&mut s, "select the compound", "select.set", json!({"ids": [c.0]}));
    step(&mut s, "compound moved", "object.move", json!({"dx": -100, "dy": 50}));
    step(&mut s, "compound released", "object.compoundPath.release", json!({}));
    undo_redo(&mut s, "compound", 7);
}

/// Area type whose frame grows as text goes in (Auto Size), restyled and reshaped; point type
/// edited; area type wrapped round an object that moves; type on a path whose options and path
/// change. All in an indexed layer.
#[test]
fn bounds_and_index_follow_text_reflow() {
    let mut s = session();
    let area = id_of(&run(
        &mut s,
        "area type",
        "text.create",
        json!({"x": 100, "y": 100, "size": 12, "text": "One line", "area": {"width": 200, "height": 40, "fit": "autoHeight"}}),
    ));
    let one = geometric(&s, area).unwrap();
    step(&mut s, "text added", "text.editRange", json!({"id": area.0, "start": 8, "end": 8, "insert": " and a good many more words ".repeat(12)}));
    let grown = geometric(&s, area).unwrap();
    assert!(grown.height() > one.height() * 2.0, "the frame grows with the text: {one:?} {grown:?}");
    run(&mut s, "select the area", "select.set", json!({"ids": [area.0]}));
    step(&mut s, "size and leading", "text.setStyle", json!({"ids": [area.0], "size": 20, "leading": 30}));
    assert!(geometric(&s, area).unwrap().height() > grown.height(), "larger type, more lines");
    step(&mut s, "range restyled", "text.setRangeStyle", json!({"id": area.0, "start": 0, "end": 3, "size": 40}));
    step(&mut s, "frame narrowed", "text.reshapeArea", json!({"id": area.0, "anchors": [[0, 1], [0, 2]], "dx": -80, "dy": 0}));
    // Point type, longer.
    let point = id_of(&run(&mut s, "point type", "text.create", json!({"x": 420, "y": 300, "text": "Hi", "size": 24})));
    let short = geometric(&s, point).unwrap();
    step(&mut s, "point type edited", "text.editRange", json!({"id": point.0, "start": 2, "end": 2, "insert": " there, everyone"}));
    assert!(geometric(&s, point).unwrap().width() > short.width() * 2.0);
    // Area type wrapped round an object above it: moving the object reflows the text.
    let wrapped = id_of(&run(
        &mut s,
        "wrapped area type",
        "text.create",
        json!({"x": 450, "y": 350, "size": 12, "text": "word ".repeat(150), "area": {"width": 300, "height": 60, "fit": "autoHeight"}}),
    ));
    let plain = geometric(&s, wrapped).unwrap();
    let r = id_of(&run(&mut s, "wrap object", "shape.rectangle", json!({"x": 440, "y": 340, "width": 200, "height": 150})));
    step(&mut s, "wrap made", "object.textWrap.make", json!({"ids": [r.0], "offset": 6}));
    let around = geometric(&s, wrapped).unwrap();
    assert!(around.height() > plain.height(), "wrapped lines are shorter, so there are more: {plain:?} {around:?}");
    run(&mut s, "select the wrap object", "select.set", json!({"ids": [r.0]}));
    step(&mut s, "wrap object moved away", "object.move", json!({"dx": 0, "dy": 300}));
    step(&mut s, "wrap offset", "object.textWrap.options", json!({"ids": [r.0], "offset": 20}));
    step(&mut s, "wrap object moved back", "object.move", json!({"dx": 0, "dy": -300}));
    // The wrap object dragged with the Selection tool: the text reflows every frame.
    let at = geometric(&s, r).unwrap().center();
    drag(&mut s, "wrap object dragged", "selection", at, Point::new(at.x + 60.0, at.y + 40.0), 4);
    undo_redo(&mut s, "wrap drag", 1);
    step(&mut s, "wrap released", "object.textWrap.release", json!({"ids": [r.0]}));
    // Type on a path: its options and its path change.
    let curve = id_of(&run(&mut s, "curve", "path.create", json!({"d": "M100 450 C200 350 300 550 400 450"})));
    let on_path =
        id_of(&step(&mut s, "type on a path", "text.createInPath", json!({"path": curve.0, "mode": "onPath", "text": "Along the curve we go"})));
    run(&mut s, "select type on a path", "select.set", json!({"ids": [on_path.0]}));
    step(&mut s, "path type start", "type.pathOptions", json!({"ids": [on_path.0], "start": 0.3}));
    step(&mut s, "path type flipped", "type.pathOptions", json!({"ids": [on_path.0], "flip": true}));
    let before = geometric(&s, on_path);
    step(&mut s, "path type scaled", "object.scale", json!({"ids": [on_path.0], "sx": 150, "sy": 50}));
    assert_ne!(geometric(&s, on_path), before, "scaling the path moves the type");
    undo_redo(&mut s, "text", 20);
}

/// A group large enough to be indexed of its own, and a small one: members moved, edited and
/// deleted inside them (taking the large one below the size that is indexed and back), clipping
/// groups and opacity masks made and edited, all undone and redone.
#[test]
fn bounds_and_index_follow_large_and_small_groups_clips_and_masks() {
    let mut s = session();
    let ids = layer_ids(&s);
    run(&mut s, "select 70", "select.set", json!({"ids": ids[..70]}));
    let big = id_of(&step(&mut s, "large group", "object.group", json!({})));
    run(&mut s, "select 5", "select.set", json!({"ids": ids[100..105]}));
    let small = id_of(&step(&mut s, "small group", "object.group", json!({})));
    let members = |s: &Session, id: NodeId| s.doc().unwrap().doc.node(id).unwrap().children().unwrap().len();
    let min = vectorcraft_doc::spatial::MIN_MEMBERS;
    assert!(members(&s, big) >= min && members(&s, small) < min, "only the large group is indexed");
    for (k, g) in [(0, big), (1, small)] {
        let m = if k == 0 { ids[33] } else { ids[102] };
        run(&mut s, "select a member", "select.set", json!({"ids": [m]}));
        step(&mut s, &format!("member of group {k} moved"), "object.move", json!({"dx": 13, "dy": 7}));
        step(&mut s, &format!("member of group {k} restroked"), "stroke.set", json!({"weight": 9}));
        step(&mut s, &format!("member of group {k} given a shadow"), "effect.apply", json!({"effect": "stylize.dropShadow", "item": null}));
        run(&mut s, "select the group", "select.set", json!({"ids": [g.0]}));
        step(&mut s, &format!("group {k} moved"), "object.move", json!({"dx": -9, "dy": 4}));
    }
    // Ten members deleted: the large group drops below the size that is indexed; undo brings
    // it back.
    run(&mut s, "select 10 members", "select.set", json!({"ids": ids[20..30]}));
    step(&mut s, "ten members deleted", "edit.clear", json!({}));
    run(&mut s, "select a member", "select.set", json!({"ids": [ids[40]]}));
    step(&mut s, "a member of the smaller group moved", "object.move", json!({"dx": 50, "dy": 0}));
    // A clipping group of the layer's objects, its clip edited with Direct Selection, and a member.
    let clip = id_of(&run(&mut s, "clip path", "path.create", json!({"d": "M500 420 L700 420 L700 560 L500 560 Z"})));
    run(&mut s, "select clip and art", "select.set", json!({"ids": [ids[90], ids[91], ids[92], clip.0]}));
    let cg = id_of(&step(&mut s, "clipping group", "object.clippingMask.make", json!({})));
    let clipped = geometric(&s, cg).unwrap();
    run(&mut s, "nothing selected", "select.none", json!({}));
    let corner = Point::new(700.0, 560.0);
    click(&mut s, "directSelection", corner);
    drag(&mut s, "clip corner dragged", "directSelection", corner, Point::new(760.0, 590.0), 3);
    assert_ne!(geometric(&s, cg).unwrap(), clipped, "the clip's corner moved, and so did the group's bounds");
    undo_redo(&mut s, "clip drag", 1);
    run(&mut s, "select a clipped member", "select.set", json!({"ids": [ids[91]]}));
    step(&mut s, "clipped member moved", "object.move", json!({"dx": 0, "dy": 30}));
    run(&mut s, "select the clipping group", "select.set", json!({"ids": [cg.0]}));
    step(&mut s, "clip released", "object.clippingMask.release", json!({}));
    // An opacity mask.
    let mask = id_of(&run(&mut s, "mask", "shape.ellipse", json!({"x": 20, "y": 400, "width": 200, "height": 120})));
    run(&mut s, "select art and mask", "select.set", json!({"ids": [ids[110], ids[111], mask.0]}));
    let masked = id_of(&step(&mut s, "opacity mask", "transparency.makeOpacityMask", json!({})));
    run(&mut s, "select the masked", "select.set", json!({"ids": [masked.0]}));
    step(&mut s, "masked moved", "object.move", json!({"dx": 25, "dy": -10}));
    step(&mut s, "mask released", "transparency.releaseOpacityMask", json!({}));
    undo_redo(&mut s, "groups", 25);
}

/// Symbol instances when the symbol's art is redefined: they and the layer's index follow.
#[test]
fn bounds_and_index_follow_symbol_edits() {
    let mut s = session();
    run(&mut s, "art", "shape.rectangle", json!({"x": 300, "y": 300, "width": 40, "height": 20}));
    let inst = id_of(&step(&mut s, "symbol", "symbol.new", json!({"name": "Box"})));
    let mut others = vec![];
    for i in 0..4 {
        others.push(id_of(&step(&mut s, "placed", "symbol.place", json!({"name": "Box", "x": 100 + 150 * i, "y": 480}))));
    }
    let before = geometric(&s, others[0]).unwrap();
    run(&mut s, "select the instance", "select.set", json!({"ids": [inst.0]}));
    let art = NodeId(run(&mut s, "symbol edited", "symbol.edit", json!({}))["ids"][0].as_u64().unwrap());
    run(&mut s, "select the art", "select.set", json!({"ids": [art.0]}));
    run(&mut s, "art widened", "object.scale", json!({"sx": 300, "sy": 200}));
    step(&mut s, "symbol updated", "symbol.update", json!({"name": "Box"}));
    let after = geometric(&s, others[0]).unwrap();
    assert!(after.width() > before.width() * 2.0, "the other instances follow: {before:?} {after:?}");
    undo_redo(&mut s, "symbol", 8);
}

/// A layer that grows past the size that is indexed one command at a time, is undone back to
/// before its index was first built, and is redone forward.
#[test]
fn undo_before_the_index_was_built_and_redo_past_it() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
    let min = vectorcraft_doc::spatial::MIN_MEMBERS;
    let n = min + 16;
    for i in 0..n {
        let (x, y) = ((i % 12) as f64 * 60.0 + 10.0, (i / 12) as f64 * 55.0 + 10.0);
        run(&mut s, &format!("object {i}"), "shape.rectangle", json!({"x": x, "y": y, "width": 30, "height": 20}));
    }
    let indexed = |s: &Session| s.doc().unwrap().doc.spatial().stats().0 >= 1;
    assert!(indexed(&s), "the layer is indexed");
    run(&mut s, "select all", "select.all", json!({}));
    run(&mut s, "all moved", "object.move", json!({"dx": 7, "dy": 5}));
    // Back to a handful of objects, then forward again.
    let back = n - 5 + 1;
    for i in 0..back {
        run(&mut s, &format!("undo {i}"), "edit.undo", json!({}));
    }
    assert!(s.doc().unwrap().doc.layers[0].children().unwrap().len() < min);
    for i in 0..back {
        run(&mut s, &format!("redo {i}"), "edit.redo", json!({}));
    }
    assert!(s.doc().unwrap().doc.layers[0].children().unwrap().len() >= min, "large enough to be indexed again");
    // A new branch from the middle of the history.
    for i in 0..30 {
        run(&mut s, &format!("undo again {i}"), "edit.undo", json!({}));
    }
    run(&mut s, "select all", "select.all", json!({}));
    run(&mut s, "a new branch", "object.move", json!({"dx": -3, "dy": 2}));
    for i in 0..40 {
        run(&mut s, &format!("more {i}"), "shape.ellipse", json!({"x": (i * 19) % 760, "y": 300 + (i * 7) % 250, "width": 25, "height": 25}));
    }
    undo_redo(&mut s, "new branch", 41);
}
