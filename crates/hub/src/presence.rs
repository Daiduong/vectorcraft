//! The presence projection (CraftHub spec §6.1, §18.4 row 5): what VectorCraft's live UI state
//! looks like on the wire, and what a peer's state renders as here. Presence is ephemeral — it is
//! never stored, never journalled, never saved (§2.7's rule, kept verbatim in v3).
//!
//! VectorCraft's row in the per-app projection table: the cursor in **canvas (document)
//! coordinates**, the artboard being worked on (the `page` field, an `Artboard::id` — stable
//! where an index would drift when artboards reorder), the selection (the Layers-panel coloured
//! outlines, never edit locks — §6.7 does locks), the viewport rectangle (for follow mode), and
//! the type object being edited with its caret. VectorCraft has no timeline, no tracks and no
//! generation jobs — those fields belong to the siblings (FilmCraft, SoundCraft, ArtCraft) and
//! stay `None` here.

use craft_collab::adapter::UiState;

/// Everything the VectorCraft UI needs to render one peer: the projection is a two-way street —
/// the same fields go out and come back.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PeerView {
    /// The peer's cursor in document coordinates.
    pub cursor: Option<(f64, f64)>,
    /// The artboard the peer is on (a presence dot on the Artboards panel thumbnail; the canvas
    /// only shows same-artboard peers).
    pub artboard: Option<u64>,
    /// The peer's selection (frame-edge highlights in the layer colour, never edit locks).
    pub selection: Vec<craft_collab::ids::NodeId>,
    /// The peer's viewport, offered to the follow mode.
    pub viewport: Option<(f64, f64, f64, f64)>,
    /// The peer is typing in this type object (node id + caret + selection).
    pub editing_text: Option<(craft_collab::ids::NodeId, usize, Option<usize>)>,
}

impl PeerView {
    /// Project VectorCraft's UI state into the fields the hub's `UiState` carries.
    ///
    /// The selection is bounded (§2.12: at most 10,000 ids) by the core's sanitizer, not here —
    /// the app reports what it truly has and the hub enforces the budget.
    #[must_use]
    pub fn from_ui(ui: &UiState) -> Self {
        Self { cursor: ui.cursor, artboard: ui.page, selection: ui.selection.clone(), viewport: ui.viewport, editing_text: ui.editing_text }
    }

    /// Whether this peer shows anything on `artboard` at all (the Artboards panel shows everyone
    /// as a thumbnail dot; the canvas only the peers on the same artboard).
    #[must_use]
    pub fn on_artboard(&self, artboard: u64) -> bool {
        self.artboard == Some(artboard)
    }
}
