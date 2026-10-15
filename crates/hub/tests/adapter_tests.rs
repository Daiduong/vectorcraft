//! The adapter's ported convergence suite (CraftHub spec §3.6 phase C: the ported tests, the
//! family's most valuable testing asset), over VectorCraft's real document — plus this row's own
//! mandatory measurement tests: a path-heavy document edited in one place must produce an
//! edit-sized batch, pinned three ways (a field edit, a z-order move, a geometric translate).

// Tests may unwrap: a failing assertion is the point of a test (the family's clippy rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use craft_collab::adapter::HubAdapter;
use vectorcraft_doc::geom::kurbo::{BezPath, PathEl, Point};
use vectorcraft_doc::geom::shapes;
use vectorcraft_doc::{Appearance, Document, Node, NodeKind};
use vectorcraft_hub::VectorcraftHub;

fn sample() -> Document {
    // Two layers, paths and a group of two inside the first, a compound in the second — the
    // shape of a real small illustration.
    let mut doc = Document::new(595.0, 842.0);
    let layer = doc.layers[0].id;
    let one = Node::path(doc.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(50.0, 50.0, 150.0, 120.0)), Appearance::default_art());
    doc.insert(Some(layer), 0, one).unwrap();
    let two = Node::path(doc.alloc_id(), shapes::ellipse(vectorcraft_doc::geom::Rect::new(200.0, 60.0, 280.0, 140.0)), Appearance::default_art());
    doc.insert(Some(layer), 1, two).unwrap();
    let a = Node::path(doc.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(0.0, 0.0, 40.0, 40.0)), Appearance::default_art());
    let b = Node::path(doc.alloc_id(), shapes::polygon(Point::new(80.0, 20.0), 20.0, 6, 0.0), Appearance::default_art());
    let group = Node::group(doc.alloc_id(), vec![Arc::new(a), Arc::new(b)]);
    doc.insert(Some(layer), 2, group).unwrap();
    let second = doc.add_layer(Some("Art"));
    let c1 = Node::path(doc.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(0.0, 0.0, 30.0, 90.0)), Appearance::default_art());
    let c2 = Node::path(doc.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(40.0, 0.0, 70.0, 90.0)), Appearance::default_art());
    let mut compound = Node::new(doc.alloc_id(), NodeKind::Compound { children: Vec::new(), rule: Default::default() });
    compound.kind = NodeKind::Compound { children: vec![Arc::new(c1), Arc::new(c2)], rule: Default::default() };
    doc.insert(Some(second), 0, compound).unwrap();
    doc
}

fn assert_same_state(hub: &VectorcraftHub, a: &Document, b: &Document, what: &str) {
    let (va, vb) = (hub.view(a), hub.view(b));
    assert_eq!(va.nodes.len(), vb.nodes.len(), "{what}: node count");
    for (na, nb) in va.nodes.iter().zip(vb.nodes.iter()) {
        assert_eq!(na.id, nb.id, "{what}: node id");
        assert_eq!(na.kind, nb.kind, "{what}: kind of {na:?}");
        assert_eq!(na.parent, nb.parent, "{what}: parent of {na:?}");
        assert_eq!(na.position, nb.position, "{what}: position of {na:?}");
        assert_eq!(na.props, nb.props, "{what}: props of {na:?}");
    }
    assert_eq!(va.doc, vb.doc, "{what}: doc props");
}

/// Every node `Arc` by id (to prove the untouched ones stay shared between two states).
fn arcs_by_id(doc: &Document) -> std::collections::BTreeMap<u64, Arc<Node>> {
    let mut out = std::collections::BTreeMap::new();
    for layer in &doc.layers {
        layer.walk(&mut |n| {
            out.entry(n.id.0).or_insert_with(|| Arc::clone(&lookup(doc, n.id.0).unwrap()));
        });
    }
    out
}

/// The `Arc` of a node by id (the tree-walking lookup the adapter uses).
fn lookup(doc: &Document, raw: u64) -> Option<Arc<Node>> {
    fn in_list(nodes: &[Arc<Node>], raw: u64) -> Option<Arc<Node>> {
        for n in nodes {
            if n.id.0 == raw {
                return Some(Arc::clone(n));
            }
            if let Some(found) = n.children().and_then(|ch| in_list(ch, raw)) {
                return Some(found);
            }
        }
        None
    }
    in_list(&doc.layers, raw)
}

/// A dense path of `n` anchors (a traced or freehand path's realistic density).
fn dense_path(n: usize) -> vectorcraft_doc::geom::PathData {
    let mut bp = BezPath::new();
    bp.push(PathEl::MoveTo(Point::new(0.0, 0.0)));
    for i in 1..n {
        let (x, y) = (i as f64 * 0.37, (i as f64 * 0.11).sin() * 40.0);
        bp.push(PathEl::LineTo(Point::new(x, y)));
    }
    vectorcraft_doc::geom::PathData::from_bezpath(&bp)
}

/// A path-heavy document: `nodes` dense paths on one layer (millions of bytes on the wire).
fn dense_document(nodes: usize, anchors: usize) -> Document {
    let mut doc = Document::new(595.0, 842.0);
    let layer = doc.layers[0].id;
    for _ in 0..nodes {
        let p = Node::path(doc.alloc_id(), dense_path(anchors), Appearance::default_art());
        doc.insert(Some(layer), usize::MAX, p).unwrap();
    }
    doc
}

/// The ancestor chain of `victim` (its layer included): the nodes VectorCraft's copy-on-write
/// re-allocates when `victim` is edited through `node_mut` — `Arc::make_mut` clones the node and
/// its spine because the document clone shares them. What the model *guarantees* is that every
/// node **off** that chain keeps its `Arc` between the two states; that is the sharing the
/// adapter's fast path and the undo history ride on. (The wire measurement — the actual point of
/// these tests — is unaffected: the diff compares the re-allocated spine's content and emits
/// nothing for it.)
fn spine_of(doc: &Document, victim: vectorcraft_doc::NodeId) -> std::collections::HashSet<u64> {
    doc.ancestry(victim).into_iter().flatten().map(|id| id.0).collect()
}

/// The serialized size of a batch (the measurement the adoption table demands).
fn batch_json(changes: &[craft_collab::change::Change]) -> String {
    serde_json::to_string(changes).unwrap()
}

/// The core promise, kept: applying the diff of two documents to the first makes it the second —
/// edits, creations, deletions, moves between containers, reorders, and the document level.
#[test]
fn apply_diff_roundtrip() {
    let hub = VectorcraftHub::new();
    let a = sample();
    let mut b = a.clone();

    // Field edits on a top-level path and a path inside the group.
    let first = b.layers[0].children().unwrap()[0].id;
    b.node_mut(first).unwrap().name = Some("Renamed".into());
    b.node_mut(first).unwrap().opacity = 0.5;
    let group = b.layers[0].children().unwrap()[2].id;
    let child = b.layers[0].children().unwrap()[2].children().unwrap()[1].id;
    b.node_mut(child).unwrap().locked = true;

    // A creation inside the group, a deletion, a move to the other layer.
    let fresh = Node::path(b.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(5.0, 5.0, 25.0, 25.0)), Appearance::default_art());
    b.insert(Some(group), 1, fresh).unwrap();
    let gone = b.layers[0].children().unwrap()[1].id;
    b.remove(gone).unwrap();
    let moved = b.layers[0].children().unwrap()[1].id;
    let second_layer = b.layers[1].id;
    b.move_node(moved, Some(second_layer), 0).unwrap();

    // A z-order move (to the front of the layer).
    let back = b.layers[0].children().unwrap()[0].id;
    b.move_node(back, Some(b.layers[0].id), usize::MAX).unwrap();

    // The document level.
    b.title = "renamed file".into();
    b.artboards[0].name = "Cover".into();
    b.page_isolate = true;

    let changes = hub.diff(&a, &b);
    assert!(!changes.is_empty());

    let mut applied = a.clone();
    hub.apply(&mut applied, &changes);
    assert_same_state(&hub, &applied, &b, "roundtrip");
    assert!(hub.diff(&applied, &b).is_empty(), "the applied state diffs clean against the target");
}

/// An insertion in the middle costs one position key — the siblings keep theirs.
#[test]
fn insertion_in_the_middle_costs_one_key() {
    let hub = VectorcraftHub::new();
    let a = sample();
    let mut b = a.clone();
    let layer = b.layers[0].id;
    let fresh = Node::path(b.alloc_id(), shapes::rectangle(vectorcraft_doc::geom::Rect::new(10.0, 10.0, 60.0, 60.0)), Appearance::default_art());
    let fresh_id = fresh.id;
    b.insert(Some(layer), 2, fresh).unwrap();

    let changes = hub.diff(&a, &b);
    let positions = changes.iter().filter(|c| matches!(c, craft_collab::change::Change::Set { prop, .. } if prop.as_ref() == "position")).count();
    let creates = changes.iter().filter(|c| matches!(c, craft_collab::change::Change::Create { .. })).count();
    assert_eq!(creates, 1, "one created node");
    assert_eq!(positions, 1, "exactly one position write: the inserted node");

    let mut applied = a.clone();
    hub.apply(&mut applied, &changes);
    assert_same_state(&hub, &applied, &b, "insert");
    // And the inserted node really sits third.
    assert_eq!(applied.layers[0].children().unwrap()[2].id, fresh_id);
}

/// Two clients editing different fields of the same path converge (the server orders the
/// batches; last-writer-wins per property, and neither side loses the other's write).
#[test]
fn two_clients_converge() {
    let hub = VectorcraftHub::new();
    let base = sample();

    let mut local = base.clone();
    let mut remote = base.clone();
    let first = local.layers[0].children().unwrap()[0].id;
    local.node_mut(first).unwrap().name = Some("Local name".into());
    remote.node_mut(first).unwrap().opacity = 0.25;

    let local_batch = hub.diff(&base, &local);
    let remote_batch = hub.diff(&base, &remote);
    assert_eq!(local_batch.len(), 1, "one field write locally: {local_batch:?}");
    assert_eq!(remote_batch.len(), 1, "one field write remotely: {remote_batch:?}");

    // Both sides receive both batches (the server echoes each to everyone).
    hub.apply(&mut local, &remote_batch);
    hub.apply(&mut remote, &local_batch);
    assert_same_state(&hub, &local, &remote, "convergence");
}

/// A deleted container is one change (its children leave with it), and deleting a deleted object
/// does nothing.
#[test]
fn subtree_delete_is_one_change_and_tolerant() {
    let hub = VectorcraftHub::new();
    let a = sample();
    let mut b = a.clone();
    let group = b.layers[0].children().unwrap()[2].id;
    b.remove(group).unwrap();

    let changes = hub.diff(&a, &b);
    let deletes = changes.iter().filter(|c| matches!(c, craft_collab::change::Change::Delete { .. })).count();
    assert_eq!(deletes, 1, "one Delete for the whole group");

    let mut applied = a.clone();
    hub.apply(&mut applied, &changes);
    assert_same_state(&hub, &applied, &b, "delete");
    // Tolerant re-apply: deleting a deleted object does nothing (skipped, not an error).
    let report = hub.apply(&mut applied, &changes);
    assert_eq!(report.applied, 0);
}

/// The dense-path measurement, form one: a one-field edit on a path-heavy document produces a
/// batch of exactly one small `Set` — the document is megabytes, the batch is bytes, and every
/// untouched node stays `Arc`-shared with the previous state (nothing else re-serialized).
#[test]
fn one_field_edit_on_a_dense_document_stays_edit_sized() {
    let hub = VectorcraftHub::new();
    let a = dense_document(60, 3000);
    let doc_json = serde_json::to_string(&a).unwrap();
    assert!(doc_json.len() > 1_000_000, "the document is megabytes on the wire ({} bytes)", doc_json.len());

    let mut b = a.clone();
    let victim = b.layers[0].children().unwrap()[7].id;
    b.node_mut(victim).unwrap().name = Some("touched".into());

    let changes = hub.diff(&a, &b);
    assert_eq!(changes.len(), 1, "exactly one change: {changes:?}");
    let json = batch_json(&changes);
    assert!(json.len() < 1024, "the batch is edit-sized ({} bytes), not document-sized ({} bytes)", json.len(), doc_json.len());
    assert!(matches!(&changes[0], craft_collab::change::Change::Set { id, prop, .. } if id.0 == victim.0 && prop.as_ref() == "name"));

    // Shared structure: every node off the copy-on-write spine (the edited node and its layer)
    // is the same `Arc` — the siblings never re-serialize, which is what keeps the *next* diff
    // off them too.
    let (before, after) = (arcs_by_id(&a), arcs_by_id(&b));
    let spine = spine_of(&b, victim);
    for (id, arc) in &before {
        if !spine.contains(id)
            && let Some(other) = after.get(id)
        {
            assert!(Arc::ptr_eq(arc, other), "node {id} re-serialized");
        }
    }
}

/// The dense-path measurement, form two: a z-order move of one dense node touches no geometry at
/// all — no `kind` value is re-sent (the dense path never re-serializes), only the one node's
/// position key moves, and every untouched node stays `Arc`-shared.
#[test]
fn moving_one_dense_node_touches_no_geometry() {
    let hub = VectorcraftHub::new();
    let a = dense_document(60, 3000);
    let doc_json = serde_json::to_string(&a).unwrap();
    assert!(doc_json.len() > 1_000_000, "the document is megabytes on the wire ({} bytes)", doc_json.len());

    let mut b = a.clone();
    let victim = b.layers[0].children().unwrap()[7].id;
    b.move_node(victim, Some(b.layers[0].id), 0).unwrap(); // to the back of the layer

    let changes = hub.diff(&a, &b);
    assert!(!changes.is_empty());
    // Only position writes, all on the one moved node.
    for c in &changes {
        match c {
            craft_collab::change::Change::Set { id, prop, .. } => {
                assert_eq!(id.0, victim.0, "a write on another node: {c:?}");
                assert_eq!(prop.as_ref(), "position", "a non-position write: {c:?}");
            }
            other => panic!("unexpected change: {other:?}"),
        }
    }
    let json = batch_json(&changes);
    assert!(json.len() < 256, "the move batch is a few bytes ({} bytes)", json.len());

    let mut applied = a.clone();
    hub.apply(&mut applied, &changes);
    assert_same_state(&hub, &applied, &b, "move");
    // And no node's kind was re-sent — the geometry never left the client.
    assert!(!changes.iter().any(|c| matches!(c, craft_collab::change::Change::Set { prop, .. } if prop.as_ref() == "kind")));

    // Shared structure: a z-order move re-allocates only the copy-on-write spine (the layer
    // holding the moved node); the moved node's own `Arc` travels between list slots without
    // re-serializing, and every other node is untouched bytes.
    let (before, after) = (arcs_by_id(&a), arcs_by_id(&b));
    let spine = spine_of(&b, victim);
    let ancestors: std::collections::HashSet<u64> = spine.iter().copied().filter(|id| *id != victim.0).collect();
    for (id, arc) in &before {
        if !ancestors.contains(id)
            && let Some(other) = after.get(id)
        {
            assert!(Arc::ptr_eq(arc, other), "node {id} re-serialized by a z-order move");
        }
    }
}

/// The dense-path measurement, form three: a geometric edit of one dense path (a translate
/// rewrites every anchor — the hazard this row names) re-sends exactly that one node's `kind`
/// value: one property, one node, bounded by the node's own size — never the parent's, never the
/// siblings', never the document's.
#[test]
fn translating_one_dense_path_resends_only_that_path() {
    let hub = VectorcraftHub::new();
    let a = dense_document(60, 3000);
    let doc_json = serde_json::to_string(&a).unwrap();
    assert!(doc_json.len() > 1_000_000, "the document is megabytes on the wire ({} bytes)", doc_json.len());

    let mut b = a.clone();
    let victim = b.layers[0].children().unwrap()[7].id;
    // A translate: every anchor moves by (3, 5), exactly what the canvas move command does.
    let node = b.node_mut(victim).unwrap();
    let NodeKind::Path { path, .. } = &mut node.kind else { panic!("not a path") };
    for sp in &mut path.subpaths {
        for anchor in &mut sp.anchors {
            anchor.p.x += 3.0;
            anchor.p.y += 5.0;
            if anchor.h_in.x != anchor.p.x - 3.0 || anchor.h_in.y != anchor.p.y - 5.0 {
                anchor.h_in.x += 3.0;
                anchor.h_in.y += 5.0;
            }
            if anchor.h_out.x != anchor.p.x - 3.0 || anchor.h_out.y != anchor.p.y - 5.0 {
                anchor.h_out.x += 3.0;
                anchor.h_out.y += 5.0;
            }
        }
    }

    let changes = hub.diff(&a, &b);
    // Exactly one change: the kind (the path geometry) of the one node.
    assert_eq!(changes.len(), 1, "one change: {changes:?}");
    assert!(matches!(&changes[0], craft_collab::change::Change::Set { id, prop, .. } if id.0 == victim.0 && prop.as_ref() == "kind"));

    // The batch is bounded by the node's own payload, and stays a small fraction of the document.
    let json = batch_json(&changes);
    let victim_json = serde_json::to_string(&lookup(&b, victim.0).unwrap()).unwrap();
    assert!(json.len() < victim_json.len() * 2, "the batch carries the one path ({} bytes vs the node's {} bytes)", json.len(), victim_json.len());
    assert!(json.len() * 20 < doc_json.len(), "the batch is a small fraction of the document ({} of {} bytes)", json.len(), doc_json.len());

    // The roundtrip still holds, and every node off the copy-on-write spine stays shared.
    let mut applied = a.clone();
    hub.apply(&mut applied, &changes);
    assert_same_state(&hub, &applied, &b, "translate");
    let (before, after) = (arcs_by_id(&a), arcs_by_id(&b));
    let spine = spine_of(&b, victim);
    for (id, arc) in &before {
        if !spine.contains(id)
            && let Some(other) = after.get(id)
        {
            assert!(Arc::ptr_eq(arc, other), "node {id} re-serialized by a sibling's translate");
        }
    }
}

/// A container's `kind` property never smuggles a subtree: the wire form is elided, an edit deep
/// inside a group is not the group's kind change, and a value carrying children is refused.
#[test]
fn kind_payload_rides_without_its_children() {
    let hub = VectorcraftHub::new();
    let a = sample();
    let mut b = a.clone();
    let group = b.layers[0].children().unwrap()[2].id;
    let child = b.layers[0].children().unwrap()[2].children().unwrap()[0].id;
    b.node_mut(child).unwrap().visible = false; // deep inside the group

    let changes = hub.diff(&a, &b);
    // The child's own write is there (VectorCraft's field is `visible` — the wire name matches
    // the model, unlike figcraft's `hidden`); the group's kind property is not re-sent.
    assert!(changes.iter().any(|c| matches!(c, craft_collab::change::Change::Set { id, prop, value } if id.0 == child.0 && prop.as_ref() == "visible" && value.to_json() == serde_json::json!(false))));
    assert!(!changes.iter().any(|c| matches!(c, craft_collab::change::Change::Set { id, prop, .. } if id.0 == group.0 && prop.as_ref() == "kind")));

    // A kind value smuggling a subtree in is refused, never applied (an *empty* child list is
    // the normal elided wire form and stays legal — the container keeps its current children).
    let mut hostile = sample();
    let report = hub.apply(
        &mut hostile,
        &[craft_collab::change::Change::Set {
            id: VectorcraftHub::hub_id(group.0),
            prop: Arc::from("kind"),
            value: craft_collab::Value::Json(serde_json::json!({
                "type": "group",
                "clip": false,
                "children": [{ "id": 99, "visible": true, "locked": false }]
            })),
        }],
    );
    assert_eq!(report.applied, 0);
    assert_eq!(report.skipped, 1);
}

/// The command catalog resolves the v2 names onto their `hub.*` twins, and the deep link parses
/// the family format with the vectorcraft scheme.
#[test]
fn catalog_and_deeplink() {
    assert_eq!(vectorcraft_hub::commands::resolve("collab.share").map(|c| c.name), Some("hub.share"));
    assert_eq!(vectorcraft_hub::commands::resolve("hub.peers").map(|c| c.name), Some("hub.peers"));
    let link = vectorcraft_hub::deeplink::JoinLink::parse("vectorcraft://join?code=AB12CD34&server=wss%3A%2F%2Fhub.example.com");
    assert_eq!(link.map(|l| (l.code, l.server)), Some((Some("AB12CD34".into()), Some("wss://hub.example.com".into()))));
}
