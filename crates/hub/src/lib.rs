//! # vectorcraft-hub — VectorCraft's CraftHub client path
//!
//! The [`HubAdapter`] that maps VectorCraft's [`Document`](vectorcraft_doc::Document) onto
//! `craft-collab`'s generalized object/property model (the CraftHub specification §3.6 phase C,
//! §18.4 row 5 — the dense-paths app of the family). Nothing here is a collaboration core: every
//! tested semantic — the object/property table, the fractional sibling keys, one batch per user
//! action, the server-ordered reconciliation, per-user undo — lives in the shared core, and this
//! crate is the app-side binding that speaks it.
//!
//! What belongs here, per the family rule (§3.6: "adapters for an app ship with the app"):
//!
//! * [`props`] — the synchronized property table over [`Node`](vectorcraft_doc::Node) with the
//!   compile-time guard: a struct gaining a field that doesn't sync is a build error. The `kind`
//!   property syncs with the container payload **elided** — the children ride as their own
//!   objects, so an edit deep inside a group never re-serializes the subtree (the dense-path
//!   rule; the tests measure it).
//! * [`VectorcraftHub`] — the adapter: `view`/`apply`/`diff` over the real document, the sibling
//!   position keys (kept beside the document, exactly where a replica would keep them), and the
//!   presence projection.
//! * [`commands`] — the unified `hub.*` command catalog (same names in every Craft app, §18.3)
//!   with the `collab.*` compatibility mapping.
//! * [`deeplink`] — `vectorcraft://join?...` links (§18.5).
//! * [`schema`] — the document knowledge the hub server validates batches against: which kinds
//!   exist, which parents accept children, and the honest recompute answer (VectorCraft's live
//!   shapes, blends, envelopes and repeats are evaluated on demand from their stored specs —
//!   derived data recomputed per client, never synced).
//!
//! ## The tree, mapped honestly
//!
//! VectorCraft's model is one recursive tree: a document holds **layers** (the top-level list,
//! bottom first), and every container node — `Layer`, `Group`, `Compound`, `Blend`, `Envelope`
//! (its content), `Repeat` (its source) — holds children. The generic tree maps it directly:
//! the root's children are the layers, and a container's children are its own.
//!
//! The document level syncs as whole-value LWW properties (the deckcraft row's first-cut shape —
//! see [`props::DOC_PROPS`] for the honest list of what syncs and what does not). The per-entry
//! split for the resource lists (swatches, styles, symbols) is the documented follow-up; the
//! pilot's `style`/`collection` entries show the shape it takes.
//!
//! ## The ids (no translation map)
//!
//! VectorCraft's node ids are `u64` from one document-wide counter — the protocol's 64-bit
//! client-space ids fit natively, so there is no session translation map (unlike DeckCraft's
//! `u32` row): existing file objects ride as space-0 ids, and a node created during a live
//! session takes its hub id as its `NodeId` directly. The model's own counter mints *under*
//! `MAX_NEXT_ID` (2^62) while client spaces live in the high 32 bits, and
//! [`Document::fix_next_id`](vectorcraft_doc::Document::fix_next_id) lifts the counter above any
//! session id it sees — the two spaces cannot collide.
//!
//! ## The dense-path discipline (this row's own rule)
//!
//! Moving one node in a path-heavy document may touch thousands of points — the geometry is
//! stored in the path data itself, so a *translate* rewrites every anchor. The adapter keeps the
//! batch honest in the two ways the adoption table demands:
//!
//! * a **z-order move** (or a rename, a style change, anything but the geometry) costs exactly
//!   one property write on the one node — the untouched nodes stay `Arc`-shared with the
//!   previous state and never re-serialize (the copy-on-write the model already does for undo);
//! * a **geometric edit** re-sends the one path it touched as that node's `kind` value — one
//!   property, one node, never the parent's and never the siblings'.
//!
//! Both are pinned by measurement tests: a path-heavy document (many nodes, thousands of anchors
//! each) edited in one place produces a batch whose serialized size stays edit-sized while the
//! document is megabytes.
//!
//! What deliberately does **not** live here at all: networking, sessions, the UI. The session
//! wiring (a v3 `NetLink` behind the app's `crafthub` feature flag) lands as the follow-up once
//! this contract is reviewed; nothing here changes any user-visible behavior.

pub mod commands;
pub mod deeplink;
pub mod presence;
pub mod props;
pub mod schema;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use craft_collab::adapter::{ApplyReport, DocumentView, HubAdapter, UiState, ViewNode};
use craft_collab::change::Change;
use craft_collab::frac;
use craft_collab::ids::NodeId as HubId;
use craft_collab::value::Value;
use vectorcraft_doc::geom::{FillRule, PathData};
use vectorcraft_doc::{Document, LayerColor, Node, NodeId, NodeKind};

/// The root every layer hangs under (space 0, counter 0 — never a real id: the model's counter
/// starts at 1).
pub const ROOT: HubId = HubId(0);

/// The object kinds: the real `NodeKind` discriminants (the container set is [`CONTAINERS`]).
pub const KINDS: &[&str] =
    &["layer", "group", "path", "compound", "text", "image", "symbol", "blend", "envelope", "mesh", "repeat", "placedDocument"];

/// The kinds that accept children — exactly the variants `Node::children` answers for.
pub const CONTAINERS: &[&str] = &["layer", "group", "compound", "blend", "envelope", "repeat"];

/// The adapter: sibling keys and the presence projection in one value.
///
/// The sibling position keys are adapter state (spec §2.3: "existing documents take keys by their
/// current order at first sync"). They sit behind a mutex so the adapter stays shareable; a
/// session owns one adapter per open document and the family's document discipline is
/// single-threaded per session, so contention is not a concern — correctness is.
pub struct VectorcraftHub {
    schema: schema::VectorSchema,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    /// Sibling position keys by raw node id.
    positions: BTreeMap<u64, String>,
}

impl VectorcraftHub {
    /// A fresh adapter with the default limits and an empty key map.
    #[must_use]
    pub fn new() -> Self {
        Self { schema: schema::VectorSchema::default(), inner: Mutex::new(Inner::default()) }
    }

    /// The sibling key of the node with raw id `raw`, if one has been assigned.
    #[must_use]
    pub fn position_key(&self, raw: u64) -> Option<String> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).positions.get(&raw).cloned()
    }

    /// The generic id of a document id: the raw value *is* the identity (the model's ids are
    /// `u64` with room for the client space — see the module docs).
    #[must_use]
    pub fn hub_id(raw: u64) -> HubId {
        HubId(raw)
    }

    /// The kind string of a node (the real discriminant; the payload syncs inside `kind`).
    #[must_use]
    pub fn kind_name(node: &Node) -> &'static str {
        match &node.kind {
            NodeKind::Layer { .. } => "layer",
            NodeKind::Group { .. } => "group",
            NodeKind::Path { .. } => "path",
            NodeKind::Compound { .. } => "compound",
            NodeKind::Text(_) => "text",
            NodeKind::Image(_) => "image",
            NodeKind::SymbolInstance { .. } => "symbol",
            NodeKind::Blend { .. } => "blend",
            NodeKind::Envelope { .. } => "envelope",
            NodeKind::Mesh(_) => "mesh",
            NodeKind::Repeat(_) => "repeat",
            NodeKind::PlacedDocument(_) => "placedDocument",
        }
    }

    /// A default kind for a kind string (a `Create` without a `kind` property — the diff always
    /// sends one, so this only feeds lenient applies and tests). The payload-carrying kinds
    /// (text, image, symbol, blend spec, envelope, mesh, repeat, placed document) have no honest
    /// default: their `kind` property must carry the payload.
    #[must_use]
    pub fn default_kind(kind: &str) -> Option<NodeKind> {
        let kind = match kind {
            "layer" => NodeKind::Layer {
                color: LayerColor::Preset(0),
                template: false,
                printable: true,
                children: Vec::new(),
                clip: false,
                preview: true,
                dim_images: None,
            },
            "group" => NodeKind::Group { children: Vec::new(), clip: false },
            "path" => NodeKind::Path { path: PathData::default(), rule: FillRule::NonZero, live: None, clipping: false, guide: false },
            "compound" => NodeKind::Compound { children: Vec::new(), rule: FillRule::NonZero },
            _ => return None,
        };
        Some(kind)
    }

    // ---- Locating objects in the real model ---------------------------------------------

    /// The `Arc` of the node with raw id `raw`, anywhere in the tree.
    fn find_arc(doc: &Document, raw: u64) -> Option<&Arc<Node>> {
        fn in_list(nodes: &[Arc<Node>], raw: u64) -> Option<&Arc<Node>> {
            for n in nodes {
                if n.id.0 == raw {
                    return Some(n);
                }
                if let Some(found) = n.children().and_then(|ch| in_list(ch, raw)) {
                    return Some(found);
                }
            }
            None
        }
        in_list(&doc.layers, raw)
    }

    /// The parent of the node with raw id `raw` (`None` = a top-level layer, under the root).
    fn parent_raw(doc: &Document, raw: u64) -> Option<u64> {
        doc.parent_of(NodeId(raw)).map(|p| p.0)
    }

    /// The children of `parent` as raw ids, in document order.
    fn children_of(doc: &Document, parent: HubId) -> Vec<u64> {
        let list = if parent == ROOT { Some(&doc.layers) } else { doc.children(Some(NodeId(parent.0))) };
        list.map(|l| l.iter().map(|n| n.id.0).collect()).unwrap_or_default()
    }

    // ---- Position keys ------------------------------------------------------------------

    /// Assign sibling keys to every node that has none, by current order (the first-sync rule).
    fn first_sync_keys(&self, doc: &Document) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        fn assign(nodes: &[Arc<Node>], positions: &mut BTreeMap<u64, String>) {
            let missing: Vec<u64> = nodes.iter().map(|n| n.id.0).filter(|c| !positions.contains_key(c)).collect();
            if !missing.is_empty()
                && let Ok(keys) = frac::n_between(None, None, missing.len())
            {
                for (id, key) in missing.into_iter().zip(keys) {
                    positions.insert(id, key);
                }
            }
            for n in nodes {
                if let Some(ch) = n.children() {
                    assign(ch, positions);
                }
            }
        }
        assign(&doc.layers, &mut inner.positions);
    }

    /// Record `key` for `raw`.
    fn set_key(&self, raw: u64, key: String) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).positions.insert(raw, key);
    }

    /// Drop the keys of a subtree (before the document forgets it).
    fn drop_keys(&self, doc: &Document, raw: u64) {
        let mut doomed = vec![raw];
        let mut stack = Self::children_of(doc, HubId(raw));
        while let Some(next) = stack.pop() {
            stack.extend(Self::children_of(doc, HubId(next)));
            doomed.push(next);
        }
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        for d in doomed {
            inner.positions.remove(&d);
        }
    }

    /// Sort one parent's children by `(key, id)` in the document itself, after a key change.
    /// The `Arc`s move, not their contents — the untouched nodes stay shared with the previous
    /// state (the copy-on-write discipline the measurement tests pin).
    fn reorder_children(doc: &mut Document, parent: HubId, keys: &BTreeMap<u64, String>) {
        let rank = |id: &u64| (keys.get(id).cloned().unwrap_or_default(), *id);
        if parent == ROOT {
            doc.layers.sort_by_key(|n| rank(&n.id.0));
        } else if let Ok(ch) = doc.children_mut(Some(NodeId(parent.0))) {
            ch.sort_by_key(|n| rank(&n.id.0));
        }
    }

    /// The insertion index for `(key, id)` among `members` ordered by `(key, id)`.
    fn insertion_index(keys: &BTreeMap<u64, String>, members: &[u64], id: u64, key: &str) -> usize {
        members.iter().filter(|m| (keys.get(*m).cloned().unwrap_or_default(), **m) < (key.to_owned(), id)).count()
    }
}

impl Default for VectorcraftHub {
    fn default() -> Self {
        Self::new()
    }
}

impl HubAdapter for VectorcraftHub {
    type Doc = Document;

    fn view(&self, doc: &Document) -> DocumentView {
        self.first_sync_keys(doc);
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let key = |id: &u64| inner.positions.get(id).cloned().unwrap_or_default();
        let mut nodes = Vec::new();
        let mut stack: Vec<(&Arc<Node>, HubId)> = doc.layers.iter().map(|l| (l, ROOT)).rev().collect();
        while let Some((arc, parent)) = stack.pop() {
            let id = arc.id.0;
            let props = props::node_props().iter().filter_map(|p| props::node_value(arc, p).map(|v| (Arc::from(*p), v))).collect();
            nodes.push(ViewNode { id: HubId(id), kind: Arc::from(Self::kind_name(arc)), parent, position: key(&id), props });
            if let Some(ch) = arc.children() {
                stack.extend(ch.iter().map(|c| (c, HubId(id))).rev());
            }
        }
        let doc_props = props::DOC_PROPS.iter().filter_map(|p| props::doc_prop_value(doc, p).map(|v| ((*p).to_owned(), v))).collect();
        DocumentView { nodes, entries: Vec::new(), doc: doc_props }
    }

    fn apply(&self, doc: &mut Document, changes: &[Change]) -> ApplyReport {
        let mut report = ApplyReport::default();
        // The first-sync rule (spec §2.3) at first contact: a document that was never viewed
        // (a fresh replica catching up on history) takes keys by its current order, so the
        // reorders below sort by real keys instead of the empty default.
        self.first_sync_keys(doc);
        for change in changes {
            match change {
                Change::Create { id, kind, parent, position, props } => {
                    let raw = id.0;
                    // The kind payload rides the `kind` property; without it only the
                    // default-able containers can land.
                    let node_kind = props
                        .iter()
                        .find(|(p, _)| p.as_ref() == "kind")
                        .and_then(|(_, v)| props::parse_fresh_kind(v).ok())
                        .or_else(|| Self::default_kind(kind.as_ref()));
                    let Some(node_kind) = node_kind else {
                        report.skipped += 1;
                        continue;
                    };
                    let mut node = Node::new(NodeId(raw), node_kind);
                    for (p, v) in props {
                        if p.as_ref() == "parent" || p.as_ref() == "position" {
                            continue;
                        }
                        match props::set_node(&mut node, p, v) {
                            Some(Ok(())) => report.applied += 1,
                            Some(Err(_)) | None => report.skipped += 1,
                        }
                    }
                    self.set_key(raw, position.clone());
                    let keys = self.inner.lock().unwrap_or_else(|e| e.into_inner()).positions.clone();
                    let members = Self::children_of(doc, *parent);
                    let at = Self::insertion_index(&keys, &members, raw, position);
                    if doc.insert(parent_as_option(*parent), at, node).is_ok() {
                        report.applied += 1;
                    } else {
                        report.skipped += 1;
                    }
                }
                Change::Set { id, prop, value } => {
                    let raw = id.0;
                    if prop.as_ref() == "position" {
                        let Value::Json(serde_json::Value::String(k)) = value else {
                            report.skipped += 1;
                            continue;
                        };
                        if Self::find_arc(doc, raw).is_none() {
                            // A position write on an object that isn't there reorders nothing (a
                            // skip, never a root-level sort by accident).
                            report.skipped += 1;
                            continue;
                        }
                        let owner = Self::parent_raw(doc, raw).map_or(ROOT, HubId);
                        self.set_key(raw, k.clone());
                        let keys = self.inner.lock().unwrap_or_else(|e| e.into_inner()).positions.clone();
                        Self::reorder_children(doc, owner, &keys);
                        report.applied += 1;
                    } else if prop.as_ref() == "parent" {
                        let Some(p) = value.to_json().as_u64().map(HubId) else {
                            report.skipped += 1;
                            continue;
                        };
                        if p.0 == raw {
                            report.skipped += 1; // nothing parents itself
                            continue;
                        }
                        // The root parent is the top-level layer list. The insertion index by
                        // key, in the new parent's list; `move_node` refuses cycles itself
                        // ("cannot move an object into itself") and a wrong parent is a skip,
                        // never a panic.
                        let keys = self.inner.lock().unwrap_or_else(|e| e.into_inner()).positions.clone();
                        let key = keys.get(&raw).cloned().unwrap_or_default();
                        let members = Self::children_of(doc, p);
                        let at = Self::insertion_index(&keys, &members, raw, &key);
                        if doc.move_node(NodeId(raw), parent_as_option(p), at).is_ok() {
                            report.applied += 1;
                        } else {
                            report.skipped += 1;
                        }
                    } else if let Some(node) = doc.node_mut(NodeId(raw)) {
                        match props::set_node(node, prop, value) {
                            Some(Ok(())) => report.applied += 1,
                            Some(Err(_)) | None => report.skipped += 1,
                        }
                    } else {
                        report.skipped += 1;
                    }
                }
                Change::Delete { id } => {
                    let raw = id.0;
                    if Self::find_arc(doc, raw).is_some() {
                        self.drop_keys(doc, raw);
                        // One Delete per subtree root: the children leave with the `Arc`.
                        if doc.remove(NodeId(raw)).is_ok() {
                            report.applied += 1;
                        }
                    }
                    // Deleting a deleted object does nothing (v2 tolerance, kept).
                }
                Change::SetDoc { prop, value } => match props::set_doc_prop(doc, prop, value) {
                    Some(Ok(())) => report.applied += 1,
                    Some(Err(_)) | None => report.skipped += 1,
                },
                Change::Put { .. } | Change::Remove { .. } => {
                    report.skipped += 1; // no document-level entry lists in the first cut
                }
            }
        }
        report
    }

    fn diff(&self, before: &Document, after: &Document) -> Vec<Change> {
        let mut sets = Vec::new();
        let mut creates = Vec::new();
        let mut membership = Vec::new();

        // Every node's parent in the before tree (one walk; membership compares against it).
        let mut before_parent: std::collections::HashMap<u64, HubId> = std::collections::HashMap::new();
        {
            let mut stack: Vec<(&Arc<Node>, HubId)> = before.layers.iter().map(|l| (l, ROOT)).rev().collect();
            while let Some((arc, parent)) = stack.pop() {
                before_parent.insert(arc.id.0, parent);
                if let Some(ch) = arc.children() {
                    stack.extend(ch.iter().map(|c| (c, HubId(arc.id.0))).rev());
                }
            }
        }

        // Field sets, creations and membership, walking the after-tree parents-first (a `Create`
        // never precedes its parent's).
        {
            let mut stack: Vec<(&Arc<Node>, HubId)> = after.layers.iter().map(|l| (l, ROOT)).rev().collect();
            while let Some((arc, parent)) = stack.pop() {
                let hid = HubId(arc.id.0);
                match Self::find_arc(before, arc.id.0) {
                    Some(b) if Arc::ptr_eq(b, arc) => {}
                    Some(b) => {
                        for prop in props::node_props() {
                            if props::node_differs(b, arc, prop).unwrap_or(false)
                                && let Some(v) = props::node_value(arc, prop)
                            {
                                sets.push(Change::Set { id: hid, prop: Arc::from(prop), value: v });
                            }
                        }
                    }
                    None => {
                        let props_vec = props::node_props().iter().filter_map(|p| props::node_value(arc, p).map(|v| (Arc::from(*p), v))).collect();
                        creates.push(Change::Create {
                            id: hid,
                            kind: Arc::from(Self::kind_name(arc)),
                            parent,
                            position: String::new(),
                            props: props_vec,
                        });
                    }
                }
                // Membership: a node that kept its identity but changed parents is one `parent`
                // write (a cross-layer move, into or out of a group). The position key it needs
                // in the new parent's order comes from the order pass below.
                if let Some(&old) = before_parent.get(&arc.id.0)
                    && old != parent
                {
                    membership.push(Change::Set { id: hid, prop: Arc::from("parent"), value: Value::Json(parent.0.into()) });
                }
                if let Some(ch) = arc.children() {
                    stack.extend(ch.iter().map(|c| (c, hid)).rev());
                }
            }
        }

        // Deletions: one Delete per subtree root gone from the after tree — a container's
        // children leave with it, so a node whose (before-)parent is also gone needs no Delete
        // of its own.
        let before_ids: std::collections::BTreeSet<u64> = all_node_ids(before).collect();
        let after_ids: std::collections::BTreeSet<u64> = all_node_ids(after).collect();
        let mut deletes = Vec::new();
        for &raw in before_ids.difference(&after_ids) {
            let parent_gone = Self::parent_raw(before, raw).is_some_and(|p| !after_ids.contains(&p));
            if !parent_gone {
                deletes.push(Change::Delete { id: HubId(raw) });
            }
        }

        // Order and moves per parent (the LIS discipline — an insertion in the middle costs one
        // key), over every parent that exists in either document.
        let mut order_changes = Vec::new();
        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            let mut parents: Vec<HubId> = vec![ROOT];
            for doc in [before, after] {
                for raw in container_ids(doc) {
                    let id = HubId(raw);
                    if !parents.contains(&id) {
                        parents.push(id);
                    }
                }
            }
            for parent in parents {
                let old = Self::children_of(before, parent);
                let new = Self::children_of(after, parent);
                if old == new {
                    continue;
                }
                // First-sync: the before children without keys take them by order.
                let unkeyed: Vec<u64> = old.iter().copied().filter(|c| !inner.positions.contains_key(c)).collect();
                if !unkeyed.is_empty()
                    && let Ok(fresh) = frac::n_between(None, None, unkeyed.len())
                {
                    for (id, key) in unkeyed.into_iter().zip(fresh) {
                        inner.positions.insert(id, key);
                    }
                }
                // The longest strictly-rising run of keys over the new order, counting only
                // children that already have keys AND already belonged to this parent (a child
                // new to this parent — created or moved in — always takes a fresh key).
                let has_key = |c: &u64| inner.positions.contains_key(c);
                let old_member = |c: &u64| old.contains(c);
                let n = new.len();
                let mut len = vec![0usize; n];
                let mut prev = vec![usize::MAX; n];
                let mut best_end = None;
                let mut best_len = 0usize;
                for i in 0..n {
                    if has_key(&new[i]) && old_member(&new[i]) {
                        len[i] = 1;
                        for j in 0..i {
                            if has_key(&new[j])
                                && old_member(&new[j])
                                && inner.positions.get(&new[j]).is_some_and(|a| inner.positions.get(&new[i]).is_some_and(|b| a < b))
                                && len[j] + 1 > len[i]
                            {
                                len[i] = len[j] + 1;
                                prev[i] = j;
                            }
                        }
                        if len[i] > best_len {
                            best_len = len[i];
                            best_end = Some(i);
                        }
                    }
                }
                let mut kept = vec![false; n];
                let mut walk = best_end;
                while let Some(i) = walk {
                    kept[i] = true;
                    walk = if prev[i] == usize::MAX { None } else { Some(prev[i]) };
                }
                // Walk the new order; assign fresh keys to runs of not-kept children between
                // the kept neighbours.
                let mut last_kept: Option<String> = None;
                let mut pending: Vec<u64> = Vec::new();
                for (i, c) in new.iter().enumerate() {
                    if kept[i] {
                        let next = inner.positions.get(c).cloned();
                        flush_keys(&mut pending, last_kept.as_deref(), next.as_deref(), &mut inner, &mut order_changes);
                        last_kept = next;
                    } else {
                        pending.push(*c);
                    }
                }
                flush_keys(&mut pending, last_kept.as_deref(), None, &mut inner, &mut order_changes);
            }
        }

        // The creations' keys, now that the order pass has assigned them.
        {
            let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            for create in &mut creates {
                if let Change::Create { id, position, .. } = create {
                    *position = inner.positions.get(&id.0).cloned().unwrap_or_default();
                }
            }
        }

        // Document-level scalars.
        let mut doc_level = Vec::new();
        for prop in props::DOC_PROPS {
            if props::doc_prop_value(before, prop) != props::doc_prop_value(after, prop)
                && let Some(v) = props::doc_prop_value(after, prop)
            {
                doc_level.push(Change::SetDoc { prop: Arc::from(*prop), value: v });
            }
        }

        // Batch order (§2.3): field writes, creations (parents first), order (position keys,
        // then the parent writes a cross-container move needs), deletes, doc level.
        let mut batch = sets;
        batch.append(&mut creates);
        batch.append(&mut order_changes);
        batch.append(&mut membership);
        batch.extend(deletes);
        batch.append(&mut doc_level);
        batch
    }

    fn presence(&self, doc: &Document, ui: &UiState) -> Option<craft_collab::presence::PresenceMsg> {
        // The adoption table's row for VectorCraft: the artboard is the projection (a peer on
        // another artboard renders only in the Artboards panel); the canvas cursor, the
        // selection outlines and the viewport ride the same message for the follow mode.
        let page = ui.page.or_else(|| doc.artboards.first().map(|a| u64::from(a.id)))?;
        Some(craft_collab::presence::PresenceMsg {
            cursor: ui.cursor,
            page: Some(page),
            selection: ui.selection.clone(),
            viewport: ui.viewport,
            editing_text: ui.editing_text,
            playhead: None,    // VectorCraft has no timeline
            track_range: None, // VectorCraft has no tracks
            generating: None,  // VectorCraft runs no generation jobs
        })
    }

    fn schema(&self) -> &dyn craft_collab::adapter::DocSchema {
        &self.schema
    }
}

/// Flush a run of pending keys between `prev` and `next` (the LIS discipline's helper).
fn flush_keys(pending: &mut Vec<u64>, prev: Option<&str>, next: Option<&str>, inner: &mut Inner, out: &mut Vec<Change>) {
    if pending.is_empty() {
        return;
    }
    let count = pending.len();
    if let Ok(fresh) = frac::n_between(prev, next, count) {
        for (id, key) in pending.drain(..).zip(fresh) {
            inner.positions.insert(id, key.clone());
            out.push(Change::Set { id: HubId(id), prop: Arc::from("position"), value: Value::Json(serde_json::Value::String(key)) });
        }
    }
}

/// The root as the model's own parent form (`None` = the top-level layer list).
fn parent_as_option(parent: HubId) -> Option<NodeId> {
    (parent != ROOT).then_some(NodeId(parent.0))
}

/// Every node id in the document (pre-order).
fn all_node_ids(doc: &Document) -> impl Iterator<Item = u64> + '_ {
    let mut v: Vec<u64> = Vec::new();
    for layer in &doc.layers {
        layer.walk(&mut |n| v.push(n.id.0));
    }
    v.into_iter()
}

/// The ids of the container nodes (the parents whose child lists can change).
fn container_ids(doc: &Document) -> Vec<u64> {
    let mut v = Vec::new();
    for layer in &doc.layers {
        collect_containers(layer, &mut v);
    }
    v
}

/// Collect the containers under one node.
fn collect_containers(arc: &Arc<Node>, out: &mut Vec<u64>) {
    if let Some(ch) = arc.children() {
        out.push(arc.id.0);
        for c in ch {
            collect_containers(c, out);
        }
    }
}
