//! What the tools work out over the whole selection (or the whole document) for every pointer
//! move, kept for the document state it was worked out for: the selection's bounding box, the
//! Live Corners widgets, the brackets of type on a path, the points hovering labels, the anchors
//! Direct Selection picks. A hover then costs what the pointer is near, not the selection's size.
//!
//! The host keeps one [`ToolCache`] and hands it to every [`ToolContext`]. An entry is kept for
//! one document state: [`ToolContext::revision`] (which moves with every change of the document
//! or the selection), the document and selection the context points at, and the isolation. Any of
//! them changing drops every entry. Each entry's [`Slot`] names the options it depends on too
//! (zoom, preferences), so contexts differing only in those never share one.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use vectorcraft_doc::NodeId;

use crate::ToolContext;

/// The document state the entries were worked out for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StateKey {
    revision: (u64, u64),
    /// The addresses of the document and the selection, and a fingerprint of the selection: a
    /// context made by hand (tests, a document edited in place with no revision) still never
    /// gets another state's entries.
    doc: usize,
    selection: usize,
    selected: (usize, Option<NodeId>, Option<NodeId>, usize, usize),
    isolation: Option<NodeId>,
}

impl StateKey {
    fn of(cx: &ToolContext) -> Self {
        let s = cx.selection;
        Self {
            revision: cx.revision,
            doc: std::ptr::from_ref(cx.doc) as usize,
            selection: std::ptr::from_ref(s) as usize,
            selected: (s.objects.len(), s.objects.first().copied(), s.objects.last().copied(), s.anchors.len(), s.segments.len()),
            isolation: cx.isolation,
        }
    }
}

/// What an entry holds, with the options it depends on (floats by their bits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    /// [`crate::select::selection_box`] (visual bounds or not).
    SelectionBox(bool),
    /// [`crate::select::selection_bounds`] (visual bounds or not).
    SelectionBounds(bool),
    /// The Live Corners widgets on show ([`crate::corners::CornerWidgets::for_tool`]): any path
    /// or not, the zoom's bits, Show Corner Widget, the largest angle's bits.
    Corners { any_path: bool, zoom: u64, on: bool, max_angle: u64 },
    /// The brackets of the selected type on a path ([`crate::pathtype::Brackets::of`]).
    Brackets,
    /// The points hovering with the Selection tool labels, per selected object (visual bounds or
    /// not).
    HoverPoints(bool),
    /// The editable paths and area type frames Direct Selection picks anchors of, topmost first.
    AnchorOwners,
    /// The anchors of the selected paths and area type frames, in selection order.
    SelectedAnchors,
    /// The anchors whose handles Direct Selection shows (Show handles when multiple anchors are
    /// selected).
    HandleAnchors(bool),
    /// The selected paths the Pen and the Curvature tool edit, with where they are.
    EditablePaths,
    /// The paths whose anchors show with the Pen: the selected ones and those inside selected
    /// groups and compound paths ([`crate::draw2::shown_paths`]).
    ShownPaths,
    /// The editable blends' spines ([`crate::direct::spines`]).
    Spines,
    /// Is a Live Paint group selected?
    SelectionLivePaint,
    /// The anchors and centres of the selection's leaves ([`crate::guides::snap_pick`]).
    PickPoints,
    /// Whether each node can be edited ([`editable`]).
    Editable,
}

type Entries = HashMap<Slot, Arc<dyn Any + Send + Sync>>;

/// See the [module docs](self).
#[derive(Default)]
pub struct ToolCache(Mutex<Option<(StateKey, Entries)>>);

impl std::fmt::Debug for ToolCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ToolCache")
    }
}

impl ToolCache {
    /// The entry `slot` for the state of `cx`, made with `make` when there is none yet.
    pub(crate) fn get<T: Any + Send + Sync>(cx: &ToolContext, slot: Slot, make: impl FnOnce() -> T) -> Arc<T> {
        let me = cx.cache;
        let key = StateKey::of(cx);
        {
            let st = me.0.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some((k, entries)) = st.as_ref()
                && *k == key
                && let Some(v) = entries.get(&slot).and_then(|v| v.clone().downcast::<T>().ok())
            {
                return v;
            }
        }
        // Made without the lock held: making one entry may ask for another.
        let v = Arc::new(make());
        let mut st = me.0.lock().unwrap_or_else(PoisonError::into_inner);
        if st.as_ref().is_none_or(|(k, _)| *k != key) {
            *st = Some((key, Entries::new()));
        }
        if let Some((_, entries)) = st.as_mut() {
            entries.insert(slot, v.clone());
        }
        v
    }

    /// Drop every entry (a document closed: what it held goes with it).
    pub fn clear(&self) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// How many entries are kept (tests).
    pub fn len(&self) -> usize {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).as_ref().map_or(0, |(_, e)| e.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// [`vectorcraft_doc::Document::is_editable`] for every node of the document of `cx`, found in one
/// walk (each id where a lookup finds it: its first place in the tree), kept per state: for
/// checking a large selection without walking up from each object.
pub(crate) fn editable(cx: &ToolContext) -> Arc<vectorcraft_doc::IdMap<bool>> {
    ToolCache::get(cx, Slot::Editable, || {
        fn walk(n: &vectorcraft_doc::Node, editable: bool, out: &mut vectorcraft_doc::IdMap<bool>) {
            let editable = editable && n.visible && !n.locked;
            out.entry(n.id).or_insert(editable);
            for c in n.children().into_iter().flatten() {
                walk(c, editable, out);
            }
        }
        let mut out = vectorcraft_doc::IdMap::default();
        for l in &cx.doc.layers {
            walk(l, true, &mut out);
        }
        out
    })
}

/// The bits of a float, for a [`Slot`].
pub(crate) fn bits(v: f64) -> u64 {
    v.to_bits()
}
