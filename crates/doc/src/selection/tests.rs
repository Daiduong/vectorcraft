use std::sync::Arc;

use super::*;
use crate::Node;

#[test]
fn basic_ops() {
    let mut s = Selection::default();
    s.add(NodeId(1));
    s.add(NodeId(1));
    s.toggle(NodeId(2));
    assert_eq!(s.objects, vec![NodeId(1), NodeId(2)]);
    s.toggle(NodeId(1));
    assert_eq!(s.objects, vec![NodeId(2)]);
    s.clear();
    assert!(s.is_empty());
}

#[test]
fn guides_are_selected_on_their_own() {
    let mut s = Selection::default();
    s.add(NodeId(1));
    s.set_guides([2, 0, 2]);
    assert_eq!((s.objects.clone(), s.guides.clone()), (vec![], vec![2, 0]));
    assert!(s.has_objects_or_guides() && s.is_empty());
    // Shift-adding an object keeps them; a new selection doesn't.
    s.add(NodeId(1));
    assert_eq!(s.guides, vec![2, 0]);
    s.set([NodeId(1)]);
    assert!(s.guides.is_empty());
    // Toggled, a guide joins (or leaves) the selected objects.
    s.toggle_guide(0);
    assert_eq!((s.objects.clone(), s.guides.clone()), (vec![NodeId(1)], vec![0]));
    s.toggle_guide(0);
    assert_eq!((s.objects.clone(), s.guides.clone()), (vec![NodeId(1)], vec![]));
    // Pruning drops guides the document no longer has.
    let mut d = Document::new(100.0, 100.0);
    d.guides.push(crate::Guide::new(true, 10.0));
    s.set_guides([0, 1]);
    s.prune(&d);
    assert_eq!(s.guides, vec![0]);
}

#[test]
fn guides_show_and_lock_with_their_layer() {
    let mut d = Document::new(100.0, 100.0);
    let layer = d.layers[0].id;
    let g = crate::Guide { layer: Some(layer), ..crate::Guide::new(true, 10.0) };
    assert!(d.guide_shown(&g) && d.guide_editable(&g));
    d.node_mut(layer).unwrap().locked = true;
    assert!(d.guide_shown(&g) && !d.guide_editable(&g), "locked: shown, not editable");
    d.node_mut(layer).unwrap().visible = false;
    assert!(!d.guide_shown(&g) && !d.guide_editable(&g), "hidden");
    // On no layer, or a layer that's gone: always there.
    assert!(d.guide_shown(&crate::Guide::new(false, 5.0)));
    let gone = crate::Guide { layer: Some(NodeId(9999)), ..crate::Guide::new(true, 1.0) };
    assert!(d.guide_shown(&gone) && d.guide_editable(&gone));
    d.guides.push(g);
    assert_eq!(d.guides_on(layer).count(), 1);
    // Guides on no layer, or a gone one, are adopted by the top visible, unlocked layer.
    d.node_mut(layer).unwrap().visible = true;
    d.node_mut(layer).unwrap().locked = false;
    d.guides.push(crate::Guide::new(false, 5.0));
    d.guides.push(gone);
    assert_eq!(d.adopt_guides(), 2);
    assert!(d.guides.iter().all(|g| g.layer == Some(layer)));
    assert_eq!(d.adopt_guides(), 0, "once");
    // A guide saved before guides had layers reads as one on no layer.
    let old: crate::Guide = serde_json::from_str(r#"{"vertical": true, "pos": 5.0}"#).unwrap();
    assert_eq!(old.layer, None);
    assert!(!serde_json::to_string(&crate::Guide::new(true, 5.0)).unwrap().contains("layer"), "nothing new written");
}

#[test]
fn selection_changes_clear_the_target() {
    let mut s = Selection::default();
    s.set([NodeId(2), NodeId(3)]);
    s.target = Some(NodeId(1));
    assert_eq!(s.subjects(), &[NodeId(1)]);
    s.add(NodeId(4));
    assert_eq!(s.target, None);
    assert_eq!(s.subjects(), &[NodeId(2), NodeId(3), NodeId(4)]);
    s.target = Some(NodeId(1));
    s.remove(NodeId(2));
    assert_eq!(s.target, None);
    s.target = Some(NodeId(1));
    s.set([NodeId(3)]);
    assert_eq!(s.target, None);
}

#[test]
fn a_key_object_needs_two_or_more_selected_objects() {
    let mut s = Selection::default();
    s.set([NodeId(1), NodeId(2)]);
    s.key = Some(NodeId(2));
    s.remove(NodeId(1));
    assert_eq!(s.key, Some(NodeId(2)), "another object left: remove keeps the key");
    s.remove(NodeId(2));
    assert_eq!(s.key, None, "the key object itself left");
    // Pruning drops objects the document no longer has and clears the key once no other
    // selected object lies outside it.
    let mut d = Document::new(100.0, 100.0);
    let layer = d.layers[0].id;
    let mut ids = vec![];
    for _ in 0..4 {
        let id = d.alloc_id();
        ids.push(d.insert(Some(layer), usize::MAX, crate::Node::group(id, vec![])).unwrap());
    }
    s.set(ids.iter().copied());
    s.key = Some(ids[3]);
    d.remove(ids[0]).unwrap();
    s.prune(&d);
    assert_eq!(s.key, Some(ids[3]), "three objects left: the key stays");
    s.toggle(ids[1]);
    s.prune(&d);
    assert_eq!(s.key, Some(ids[3]), "two objects left: the key stays");
    s.toggle(ids[2]);
    s.prune(&d);
    assert_eq!((s.objects.clone(), s.key), (vec![ids[3]], None), "one object left");
    // A group key selected with only its own member is cleared too; another selected object
    // outside the group keeps it.
    let member = d.alloc_id();
    d.insert(Some(ids[3]), usize::MAX, crate::Node::group(member, vec![])).unwrap();
    s.set([member, ids[3]]);
    s.key = Some(ids[3]);
    s.prune(&d);
    assert_eq!(s.key, None, "a group and its own member");
    s.set([member, ids[3], ids[2]]);
    s.key = Some(ids[3]);
    s.prune(&d);
    assert_eq!(s.key, Some(ids[3]), "another object outside the group");
}

#[test]
fn deselecting_uneditable_objects_drops_their_anchors_segments_and_key() {
    let mut d = Document::new(100.0, 100.0);
    let layer = d.layers[0].id;
    let [a, b, c] = [(); 3].map(|()| {
        let id = d.alloc_id();
        d.insert(Some(layer), usize::MAX, crate::Node::group(id, vec![])).unwrap()
    });
    let mut s = Selection::default();
    s.set([a, b, c]);
    s.anchors.insert(a, BTreeSet::from([(0, 0)]));
    s.anchors.insert(b, BTreeSet::from([(0, 1)]));
    s.segments.insert(a, BTreeSet::from([(0, 0)]));
    (s.key, s.target, s.slices) = (Some(a), Some(layer), vec![c]);
    // Nothing hidden or locked: the selection stays as it is.
    let before = s.clone();
    s.deselect_uneditable(&d);
    assert_eq!(s, before);
    d.node_mut(a).unwrap().locked = true;
    d.node_mut(c).unwrap().visible = false;
    s.deselect_uneditable(&d);
    assert_eq!(s.objects, vec![b]);
    assert_eq!(s.anchors.keys().copied().collect::<Vec<_>>(), vec![b]);
    assert!(s.segments.is_empty());
    assert_eq!((s.key, s.target, s.slices.len()), (None, None, 0));
}

/// Xorshift step: a number below `n`.
fn below(r: &mut u64, n: u64) -> u64 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    *r % n.max(1)
}

/// A group nested up to `depth` deep (empty at the bottom), hidden or locked one time in six each.
fn random_group(d: &mut Document, r: &mut u64, depth: u64, ids: &mut Vec<NodeId>) -> Arc<Node> {
    let id = d.alloc_id();
    ids.push(id);
    let count = if depth == 0 { 0 } else { 1 + below(r, 4) };
    let children = (0..count)
        .map(|_| {
            let k = below(r, depth);
            random_group(d, r, k, ids)
        })
        .collect();
    let mut n = Node::group(id, children);
    n.locked = below(r, 6) == 0;
    n.visible = below(r, 6) != 0;
    Arc::new(n)
}

/// A layer of random groups, with a sublayer of its own when `sublayer`, hidden or locked one time
/// in six each.
fn random_layer(d: &mut Document, r: &mut u64, sublayer: bool, ids: &mut Vec<NodeId>) -> Arc<Node> {
    let mut layer = Node::layer(d.alloc_id(), "L", crate::LayerColor::Preset(0));
    let mut children: Vec<Arc<Node>> = (0..1 + below(r, 4)).map(|_| random_group(d, r, 3, ids)).collect();
    if sublayer {
        children.push(random_layer(d, r, false, ids));
    }
    if let Some(ch) = layer.children_mut() {
        *ch = children;
    }
    layer.locked = below(r, 6) == 0;
    layer.visible = below(r, 6) != 0;
    Arc::new(layer)
}

/// Some of `ids` (and sometimes an id not in the document) in random order, with anchors,
/// segments, a key, a target, slices and guides.
fn random_selection(r: &mut u64, ids: &[NodeId]) -> Selection {
    let mut s = Selection::default();
    let gone = (below(r, 2) == 0).then_some(NodeId(u64::MAX));
    s.set(ids.iter().copied().filter(|_| below(r, 2) == 0).chain(gone));
    let k = below(r, s.objects.len() as u64) as usize;
    s.objects.rotate_left(k);
    for &o in &s.objects {
        let pick = below(r, 4);
        if pick < 2 {
            s.anchors.insert(o, BTreeSet::from([(0, pick as usize)]));
        }
        if pick == 1 {
            s.segments.insert(o, BTreeSet::from([(0, 0)]));
        }
    }
    s.key = s.objects.get(below(r, s.objects.len() as u64) as usize).copied();
    s.target = ids.get(below(r, ids.len() as u64) as usize).copied().filter(|_| below(r, 2) == 0);
    s.slices = ids.iter().copied().take(below(r, 3) as usize).collect();
    s.guides = vec![1, 0];
    s
}

/// Deselecting hidden and locked objects in one pass leaves the selection exactly as removing them
/// one at a time does: on random nested documents (objects hidden or locked themselves or through
/// a group, layer or sublayer) and random selections.
#[test]
fn deselect_uneditable_matches_removing_one_at_a_time() {
    let (mut none_dropped, mut some_dropped, mut key_kept) = (0, 0, 0);
    for seed in 1..400u64 {
        let mut r = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        let mut d = Document::new(100.0, 100.0);
        let mut ids = vec![];
        for _ in 0..3 {
            let layer = random_layer(&mut d, &mut r, true, &mut ids);
            d.layers.push(layer);
        }
        // Every fourth selection is of editable objects only (but maybe one not in the document).
        if seed % 4 == 0 {
            ids.retain(|i| d.is_editable(*i));
        }
        let s = random_selection(&mut r, &ids);
        let mut one_at_a_time = s.clone();
        for o in s.objects.iter().copied().filter(|o| !d.is_editable(*o)) {
            one_at_a_time.remove(o);
        }
        let mut batched = s.clone();
        batched.deselect_uneditable(&d);
        assert_eq!(batched, one_at_a_time, "seed {seed}");
        none_dropped += usize::from(batched == s);
        some_dropped += usize::from(batched.objects.len() < s.objects.len() && !batched.objects.is_empty());
        key_kept += usize::from(batched.key.is_some() && batched.objects.len() < s.objects.len());
    }
    // The random cases reach each branch often.
    assert!(none_dropped > 10 && some_dropped > 100 && key_kept > 50, "{none_dropped} {some_dropped} {key_kept}");
}
