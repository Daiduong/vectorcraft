//! The synchronized property tables (spec §2.3: "every field the file saves is a property").
//!
//! One table over [`Node`] — every field, with the compile-time guard: a struct gaining a field
//! that doesn't sync is a build error, not a silent divergence. Two fields need special wire forms,
//! both because they are **not plain fields on the wire**:
//!
//! * `kind` — the [`NodeKind`] payload. Its container variants (`Layer`, `Group`, `Compound`,
//!   `Blend`, `Envelope`, `Repeat`) *contain the children*, and serializing them wholesale would
//!   re-send a whole subtree on every child edit — the dense-path hazard this app's row of the
//!   adoption table exists to measure. So `kind` syncs with the children **elided**: a group is
//!   `{"type":"group","clip":false}` and nothing else, and the children ride as their own objects
//!   (see [`kind_value`]; the elision is enforced on write too, [`set_kind`]).
//! * `extras` — the rarely-set parts ([`NodeExtras`]), which the file format writes as individual
//!   flat fields rather than one nested object. They sync as one `extras` object property (absent
//!   when nothing is set — most objects have none, and the wire stays lean).
//!
//! The document level is a whole-value table (see [`DOC_PROPS`]) — this app follows the deckcraft
//! row's first-cut decision rather than the pilot's per-entry split, and says so in the module
//! docs there.

use craft_collab::Value;
use vectorcraft_doc::{Node, NodeKind};

/// Declares one property table over a struct's fields: the name list, the getters/setters, the
/// differs, and the every-field-is-a-property guard.
macro_rules! props_table {
    ($ty:ident, $props:ident, $value:ident, $set:ident, $differs:ident, $guard:ident, ($($skip:ident),*), ($(($field:ident, $name:literal)),+ $(,)?)) => {
        /// Every field that syncs, by property name.
        pub const $props: &[&str] = &[$($name),+];

        /// The value of field property `prop`; `None` when `prop` is not a field.
        pub fn $value(obj: &$ty, prop: &str) -> Option<Value> {
            match prop {
                $($name => Some(Value::Json(serde_json::to_value(&obj.$field).ok()?)),)+
                _ => None,
            }
        }

        /// Write field property `prop`; `None` when `prop` is not a field.
        ///
        /// A value of the wrong shape is an error, never a panic — shipped code never crashes.
        pub fn $set(obj: &mut $ty, prop: &str, value: &Value) -> Option<Result<(), String>> {
            match prop {
                $($name => Some(
                    serde_json::from_value(value.to_json())
                        .map(|v| obj.$field = v)
                        .map_err(|e| e.to_string()),
                ),)+
                _ => None,
            }
        }

        /// Whether field property `prop` differs between `a` and `b`; `None` if not a field.
        pub fn $differs(a: &$ty, b: &$ty, prop: &str) -> Option<bool> {
            match prop {
                $($name => Some(a.$field != b.$field),)+
                _ => None,
            }
        }

        /// Fails to compile when the struct gains a field that isn't listed as a property above
        /// (or one of the explicitly skipped ones) — a new saved field must decide how it syncs.
        #[allow(dead_code)]
        fn $guard(obj: &$ty) {
            let $ty { $($field: _,)+ $($skip: _),* } = obj;
        }
    };
}

props_table!(
    Node,
    NODE_FIELDS,
    node_field_value,
    set_node_field,
    node_field_differs,
    every_node_field_is_a_property,
    (id, kind, extras),
    (
        (name, "name"),
        (visible, "visible"),
        (locked, "locked"),
        (opacity, "opacity"),
        (blend, "blend"),
        (isolate, "isolate"),
        (knockout, "knockout"),
        (knockout_shape, "knockoutShape"),
        (appearance, "appearance"),
        (bbox_angle, "bboxAngle"),
    )
);

/// The node kinds whose `kind` payload carries the children on the wire (the elision set).
const CHILD_KEYS: &[(&str, &str)] =
    &[("layer", "children"), ("group", "children"), ("compound", "children"), ("blend", "children"), ("envelope", "content"), ("repeat", "source")];

/// The value of the `kind` property: the real [`NodeKind`] payload with the children **elided** —
/// the children are their own objects, so an edit deep inside a container never re-serializes the
/// subtree (the dense-path rule this app's row measures; the tests pin it).
#[must_use]
pub fn kind_value(node: &Node) -> Option<Value> {
    let mut json = serde_json::to_value(&node.kind).ok()?;
    if let Some(map) = json.as_object_mut()
        && let Some(t) = map.get("type").and_then(serde_json::Value::as_str)
        && let Some((_, key)) = CHILD_KEYS.iter().find(|(kind, _)| *kind == t)
    {
        map.remove(*key);
    }
    Some(Value::Json(json))
}

/// Write the `kind` property: a container keeps its current children when the incoming value
/// carries none or an empty list (the normal wire form — membership syncs through `Create`/
/// `parent` writes, never through this property); a container value that carries them is
/// **refused** — the children belong to their own objects, and a property write must not smuggle
/// a subtree in behind the schema's back.
pub fn set_kind(node: &mut Node, value: &Value) -> Option<Result<(), String>> {
    let mut json = value.to_json();
    let Some(map) = json.as_object_mut() else {
        return Some(Err("kind must be an object".into()));
    };
    let wire_type = map.get("type").and_then(serde_json::Value::as_str).map(str::to_owned);
    let Some(t) = wire_type else {
        return Some(Err("kind carries no \"type\"".into()));
    };
    let Some((_, child_key)) = CHILD_KEYS.iter().find(|(kind, _)| *kind == t) else {
        return Some(serde_json::from_value::<NodeKind>(json).map(|k| node.kind = k).map_err(|e| e.to_string()));
    };
    let carried = map.remove(*child_key);
    if carried.is_some_and(|c| !matches!(c, serde_json::Value::Array(list) if list.is_empty())) {
        return Some(Err(format!("a '{t}' kind carries no {child_key} on the wire: children sync as their own objects")));
    }
    // The wire form: no children on a container. Keep the node's current ones (an empty list for
    // a node that had none, or a fresh creation).
    let current = node.children().map(|children| serde_json::to_value(children).ok()).unwrap_or(Some(serde_json::Value::Array(Vec::new())));
    if let Some(current) = current {
        map.insert((*child_key).to_owned(), current);
    }
    Some(serde_json::from_value::<NodeKind>(json).map(|k| node.kind = k).map_err(|e| e.to_string()))
}

/// Whether the `kind` property differs between two nodes — comparing the **elided** forms, so a
/// change deep inside a container is not the parent's kind change (the child's own diff carries it).
#[must_use]
pub fn kind_differs(a: &Node, b: &Node) -> bool {
    match (kind_value(a), kind_value(b)) {
        (Some(x), Some(y)) => x != y,
        // Serialization never fails for these shapes in practice; treat failure as "differs" so
        // the change is re-sent rather than silently dropped.
        _ => true,
    }
}

/// Parse a wire `kind` value for a **fresh creation** (no children to keep): a container takes an
/// empty child list, anything else parses as it is. A container value that carries a non-empty
/// child list is refused — the children belong to their own `Create` changes.
pub fn parse_fresh_kind(value: &Value) -> Result<NodeKind, String> {
    let mut json = value.to_json();
    let Some(map) = json.as_object_mut() else {
        return Err("kind must be an object".into());
    };
    let Some(t) = map.get("type").and_then(serde_json::Value::as_str).map(str::to_owned) else {
        return Err("kind carries no \"type\"".into());
    };
    if let Some((_, child_key)) = CHILD_KEYS.iter().find(|(kind, _)| *kind == t) {
        let carried = map.remove(*child_key);
        let empty = match &carried {
            None => true,
            Some(serde_json::Value::Array(list)) => list.is_empty(),
            _ => false,
        };
        if !empty {
            return Err(format!("a '{t}' kind carries no {child_key} on the wire: children sync as their own objects"));
        }
        map.insert((*child_key).to_owned(), serde_json::Value::Array(Vec::new()));
    }
    serde_json::from_value::<NodeKind>(json).map_err(|e| e.to_string())
}

/// The `extras` property's field list (the flat file fields, by their wire names).
const EXTRAS_FIELDS: &[&str] = &["mask", "trace", "wrap", "graph", "seriesIndex", "shaper", "graphicStyle", "attrs", "slice", "perspective"];

/// The value of the `extras` property: the rarely-set parts as one object, `None` when nothing is
/// set (most objects have none — the file format keeps them out of line for exactly that reason).
#[must_use]
pub fn extras_value(node: &Node) -> Option<Value> {
    let e = node.extras();
    let mut map = serde_json::Map::new();
    macro_rules! put {
        ($key:literal, $v:expr) => {
            if $v.is_some()
                && let Ok(v) = serde_json::to_value(&$v)
            {
                map.insert($key.to_owned(), v);
            }
        };
    }
    put!("mask", e.mask);
    put!("trace", e.trace);
    put!("wrap", e.wrap);
    put!("graph", e.graph);
    put!("seriesIndex", e.series_index);
    put!("shaper", e.shaper);
    put!("graphicStyle", e.graphic_style);
    put!("attrs", e.attrs);
    put!("slice", e.slice);
    put!("perspective", e.perspective);
    (!map.is_empty()).then_some(Value::Json(serde_json::Value::Object(map)))
}

/// Write the `extras` property. An empty object (or `null`) clears them all; an unknown key is an
/// error, never a silent drop — the model has nowhere to keep a foreign extra.
pub fn set_extras(node: &mut Node, value: &Value) -> Option<Result<(), String>> {
    let json = value.to_json();
    let map = match &json {
        serde_json::Value::Null => return Some(clear_extras(node)),
        serde_json::Value::Object(map) if map.is_empty() => return Some(clear_extras(node)),
        serde_json::Value::Object(map) => map,
        _ => return Some(Err("extras must be an object".into())),
    };
    for key in map.keys() {
        if !EXTRAS_FIELDS.contains(&key.as_str()) {
            return Some(Err(format!("unknown extras key '{key}'")));
        }
    }
    let mut inner = || -> Result<(), String> {
        let mut e = node.extras_mut();
        if let Some(v) = map.get("mask") {
            e.mask = serde_json::from_value(v.clone()).map_err(|err| format!("mask: {err}"))?;
        }
        if let Some(v) = map.get("trace") {
            e.trace = serde_json::from_value(v.clone()).map_err(|err| format!("trace: {err}"))?;
        }
        if let Some(v) = map.get("wrap") {
            e.wrap = serde_json::from_value(v.clone()).map_err(|err| format!("wrap: {err}"))?;
        }
        if let Some(v) = map.get("graph") {
            e.graph = serde_json::from_value(v.clone()).map_err(|err| format!("graph: {err}"))?;
        }
        if let Some(v) = map.get("seriesIndex") {
            e.series_index = serde_json::from_value(v.clone()).map_err(|err| format!("seriesIndex: {err}"))?;
        }
        if let Some(v) = map.get("shaper") {
            e.shaper = serde_json::from_value(v.clone()).map_err(|err| format!("shaper: {err}"))?;
        }
        if let Some(v) = map.get("graphicStyle") {
            e.graphic_style = serde_json::from_value(v.clone()).map_err(|err| format!("graphicStyle: {err}"))?;
        }
        if let Some(v) = map.get("attrs") {
            e.attrs = serde_json::from_value(v.clone()).map_err(|err| format!("attrs: {err}"))?;
        }
        if let Some(v) = map.get("slice") {
            e.slice = serde_json::from_value(v.clone()).map_err(|err| format!("slice: {err}"))?;
        }
        if let Some(v) = map.get("perspective") {
            e.perspective = serde_json::from_value(v.clone()).map_err(|err| format!("perspective: {err}"))?;
        }
        Ok(())
    };
    Some(inner())
}

/// Clear every extra (the guard's drop releases the allocation when they all went `None`).
fn clear_extras(node: &mut Node) -> Result<(), String> {
    let mut e = node.extras_mut();
    e.mask = None;
    e.trace = None;
    e.wrap = None;
    e.graph = None;
    e.series_index = None;
    e.shaper = None;
    e.graphic_style = None;
    e.attrs = None;
    e.slice = None;
    e.perspective = None;
    Ok(())
}

/// Whether the `extras` property differs between two nodes.
#[must_use]
pub fn extras_differs(a: &Node, b: &Node) -> bool {
    match (extras_value(a), extras_value(b)) {
        (Some(x), Some(y)) => x != y,
        (None, None) => false,
        _ => true,
    }
}

/// The full property list of a node (the plain fields plus `kind` and `extras`).
#[must_use]
pub fn node_props() -> Vec<&'static str> {
    let mut v = NODE_FIELDS.to_vec();
    v.push("kind");
    v.push("extras");
    v
}

/// The value of any node property (field, `kind` or `extras`); `None` when `prop` is none of them.
#[must_use]
pub fn node_value(node: &Node, prop: &str) -> Option<Value> {
    match prop {
        "kind" => kind_value(node),
        "extras" => extras_value(node),
        _ => node_field_value(node, prop),
    }
}

/// Write any node property (field, `kind` or `extras`); `None` when `prop` is none of them.
pub fn set_node(node: &mut Node, prop: &str, value: &Value) -> Option<Result<(), String>> {
    match prop {
        "kind" => set_kind(node, value),
        "extras" => set_extras(node, value),
        _ => set_node_field(node, prop, value),
    }
}

/// Whether any node property differs between `a` and `b`; `None` when `prop` is none of them.
pub fn node_differs(a: &Node, b: &Node, prop: &str) -> Option<bool> {
    match prop {
        "kind" => Some(kind_differs(a, b)),
        "extras" => Some(extras_differs(a, b)),
        _ => node_field_differs(a, b, prop),
    }
}

/// The document-level properties (whole-value LWW scalars, the deckcraft row's first-cut shape).
///
/// Every *saved* field of `Document` that is shared state syncs as one value: the setup
/// (`units`, `colorMode`, `rulerOrigin`, `setup`, `grid`, the artboards, guides), the panels
/// (`pageIsolate`, `pageKnockout`, `spotUseLab`, `pasteRemembersLayers`, `rasterEffectsPpi`,
/// `rasterEffects`, `slices`, `printSetup`, `assets`, `metadata`, `colorProfiles`), the resource
/// lists (`swatches`, `swatchGroups`, `graphicStyles`, `charStyles`, `paraStyles`, `symbols`,
/// `patterns`, `variables`, `graphDesigns`, `views`, `savedSelections`, `textThreads`) — each
/// last-writer-wins as a whole. Splitting the resource lists per entry (the pilot's `style`/
/// `collection` entries) is the documented follow-up; swatch edits during a live session are rare,
/// and the whole-value form cannot diverge from the model.
///
/// Not synced, honestly:
/// * `images` — the embedded/linked image map. Image **bytes** ride the ASSET channel
///   (content-addressed, spec §3.5), never a batch; the blob's `bytes`/`proxy` are `#[serde(skip)]`
///   anyway, so the property would carry half a value. The session wiring rebuilds the map from
///   the assets as they arrive.
/// * `version`, `template` — file-format metadata, not shared state.
/// * `pattern_edit`, `mask_edit`, `puppet` — runtime editing modes, never saved.
/// * `last_view`, `layers_open` — window state written at save time; each peer keeps its own.
/// * `unknown`, `extra` — foreign-data round-trip containers (data this version doesn't know).
/// * the id counter — private to the model; ids are the objects' own (see the adapter docs).
///
/// The model's `Document` cannot take the compile-time guard from outside its crate (its counter
/// is private), so this table is enumerated by hand and the list above is the review surface.
pub const DOC_PROPS: &[&str] = &[
    "title",
    "units",
    "colorMode",
    "rulerOrigin",
    "artboards",
    "setup",
    "grid",
    "guides",
    "textThreads",
    "swatches",
    "swatchGroups",
    "graphicStyles",
    "charStyles",
    "paraStyles",
    "symbols",
    "patterns",
    "variables",
    "graphDesigns",
    "views",
    "savedSelections",
    "pageIsolate",
    "pageKnockout",
    "spotUseLab",
    "pasteRemembersLayers",
    "rasterEffectsPpi",
    "rasterEffects",
    "slices",
    "slicesClipToArtboard",
    "printSetup",
    "assets",
    "metadata",
    "colorProfiles",
];

/// Write a document-level property; `None` when `prop` is not a synced scalar.
///
/// A value of the wrong shape is an error, never a panic.
pub fn set_doc_prop(doc: &mut vectorcraft_doc::Document, prop: &str, value: &Value) -> Option<Result<(), String>> {
    let json = value.to_json();
    match prop {
        "title" => Some(parse(json).map(|v| doc.title = v)),
        "units" => Some(parse(json).map(|v| doc.units = v)),
        "colorMode" => Some(parse(json).map(|v| doc.color_mode = v)),
        "rulerOrigin" => Some(parse(json).map(|v| doc.ruler_origin = v)),
        "artboards" => Some(parse(json).map(|v| doc.artboards = v)),
        "setup" => Some(parse(json).map(|v| doc.setup = v)),
        "grid" => Some(parse(json).map(|v| doc.grid = v)),
        "guides" => Some(parse(json).map(|v| doc.guides = v)),
        "textThreads" => Some(parse(json).map(|v| doc.text_threads = v)),
        "swatches" => Some(parse(json).map(|v| doc.swatches = v)),
        "swatchGroups" => Some(parse(json).map(|v| doc.swatch_groups = v)),
        "graphicStyles" => Some(parse(json).map(|v| doc.graphic_styles = v)),
        "charStyles" => Some(parse(json).map(|v| doc.char_styles = v)),
        "paraStyles" => Some(parse(json).map(|v| doc.para_styles = v)),
        "symbols" => Some(parse(json).map(|v| doc.symbols = v)),
        "patterns" => Some(parse(json).map(|v| doc.patterns = v)),
        "variables" => Some(parse(json).map(|v| doc.variables = v)),
        "graphDesigns" => Some(parse(json).map(|v| doc.graph_designs = v)),
        "views" => Some(parse(json).map(|v| doc.views = v)),
        "savedSelections" => Some(parse(json).map(|v| doc.saved_selections = v)),
        "pageIsolate" => Some(parse(json).map(|v| doc.page_isolate = v)),
        "pageKnockout" => Some(parse(json).map(|v| doc.page_knockout = v)),
        "spotUseLab" => Some(parse(json).map(|v| doc.spot_use_lab = v)),
        "pasteRemembersLayers" => Some(parse(json).map(|v| doc.paste_remembers_layers = v)),
        "rasterEffectsPpi" => Some(parse(json).map(|v| doc.raster_effects_ppi = v)),
        "rasterEffects" => Some(parse(json).map(|v| doc.raster_effects = v)),
        "slices" => Some(parse(json).map(|v| doc.slices = v)),
        "slicesClipToArtboard" => Some(parse(json).map(|v| doc.slices_clip_to_artboard = v)),
        "printSetup" => Some(parse(json).map(|v| doc.print_setup = v)),
        "assets" => Some(parse(json).map(|v| doc.assets = v)),
        "metadata" => Some(parse(json).map(|v| doc.metadata = v)),
        "colorProfiles" => Some(parse(json).map(|v| doc.color_profiles = v)),
        _ => None,
    }
}

/// The value of a document-level property; `None` when `prop` is not a synced scalar.
#[must_use]
pub fn doc_prop_value(doc: &vectorcraft_doc::Document, prop: &str) -> Option<Value> {
    match prop {
        "title" => Some(Value::Json(serde_json::to_value(&doc.title).ok()?)),
        "units" => Some(Value::Json(serde_json::to_value(doc.units).ok()?)),
        "colorMode" => Some(Value::Json(serde_json::to_value(doc.color_mode).ok()?)),
        "rulerOrigin" => Some(Value::Json(serde_json::to_value(doc.ruler_origin).ok()?)),
        "artboards" => Some(Value::Json(serde_json::to_value(&doc.artboards).ok()?)),
        "setup" => Some(Value::Json(serde_json::to_value(&doc.setup).ok()?)),
        "grid" => Some(Value::Json(serde_json::to_value(&doc.grid).ok()?)),
        "guides" => Some(Value::Json(serde_json::to_value(&doc.guides).ok()?)),
        "textThreads" => Some(Value::Json(serde_json::to_value(&doc.text_threads).ok()?)),
        "swatches" => Some(Value::Json(serde_json::to_value(&doc.swatches).ok()?)),
        "swatchGroups" => Some(Value::Json(serde_json::to_value(&doc.swatch_groups).ok()?)),
        "graphicStyles" => Some(Value::Json(serde_json::to_value(&doc.graphic_styles).ok()?)),
        "charStyles" => Some(Value::Json(serde_json::to_value(&doc.char_styles).ok()?)),
        "paraStyles" => Some(Value::Json(serde_json::to_value(&doc.para_styles).ok()?)),
        "symbols" => Some(Value::Json(serde_json::to_value(&doc.symbols).ok()?)),
        "patterns" => Some(Value::Json(serde_json::to_value(&doc.patterns).ok()?)),
        "variables" => Some(Value::Json(serde_json::to_value(&doc.variables).ok()?)),
        "graphDesigns" => Some(Value::Json(serde_json::to_value(&doc.graph_designs).ok()?)),
        "views" => Some(Value::Json(serde_json::to_value(&doc.views).ok()?)),
        "savedSelections" => Some(Value::Json(serde_json::to_value(&doc.saved_selections).ok()?)),
        "pageIsolate" => Some(Value::Json(serde_json::to_value(doc.page_isolate).ok()?)),
        "pageKnockout" => Some(Value::Json(serde_json::to_value(doc.page_knockout).ok()?)),
        "spotUseLab" => Some(Value::Json(serde_json::to_value(doc.spot_use_lab).ok()?)),
        "pasteRemembersLayers" => Some(Value::Json(serde_json::to_value(doc.paste_remembers_layers).ok()?)),
        "rasterEffectsPpi" => Some(Value::Json(serde_json::to_value(doc.raster_effects_ppi).ok()?)),
        "rasterEffects" => Some(Value::Json(serde_json::to_value(&doc.raster_effects).ok()?)),
        "slices" => Some(Value::Json(serde_json::to_value(&doc.slices).ok()?)),
        "slicesClipToArtboard" => Some(Value::Json(serde_json::to_value(doc.slices_clip_to_artboard).ok()?)),
        "printSetup" => Some(Value::Json(serde_json::to_value(&doc.print_setup).ok()?)),
        "assets" => Some(Value::Json(serde_json::to_value(&doc.assets).ok()?)),
        "metadata" => Some(Value::Json(serde_json::to_value(&doc.metadata).ok()?)),
        "colorProfiles" => Some(Value::Json(serde_json::to_value(&doc.color_profiles).ok()?)),
        _ => None,
    }
}

/// Deserialize a value, mapped to the adapter's error shape (wrong shapes are errors, never
/// panics).
fn parse<T: serde::de::DeserializeOwned>(json: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(json).map_err(|e| e.to_string())
}
