//! Reviewed AI regions become ordinary editable slices; exports preserve vector artwork.
use super::*;
use serde_json::json;
use vectorcraft_doc::{Artboard, Document, SliceOptions};
use vectorcraft_geom::Rect;

const LIMIT: usize = 64;
const C: &str = "slice.applyAI";

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "slice.aiSnapshot", "AI Slice Snapshot", [], None,
            "{selectionOnly?: false, ignoreBackground?: true} read the visible artwork bounds and document guard → {uid, revision, rect: [x,y,width,height], regions: spatially separated artwork bounds (up to 64), selectionIds}", has_doc, snapshot),
        cmd!(
            "slice.applyAI",
            "Create AI Slices",
            [],
            None,
            "{uid, revision, selectionOnly?: false, selectionIds?: [id…] (required with selectionOnly), preserveSelection?: false (keep object selection), regions: [{name, x,y,width,height}]} add 1..64 reviewed regions as named user slices in one undo step; never moves artwork → {ids}",
            has_doc,
            apply
        ),
        cmd!(query "slice.exportAI", "Export AI Slices", [], None,
            "{uid, revision, selectionOnly?: false, selectionIds?: [id…] (required with selectionOnly), regions: [{name,x,y,width,height,sourceIds?: [id…],candidateIndex?: 0..63 (stable export number)}], ignoreBackground?: true, format?: svg|png, scale?: 1, folder?} crop visible artwork to each reviewed region. No folder → {files:[{name,dataBase64}],warnings}; folder → {files:[path],warnings}; refuses existing files. SVG preserves vectors and clipping; PNG preserves transparency. Up to 64 regions and 16 million raster pixels per batch", has_doc, export),
    ]
}

fn guard(s: &Session, p: &Value) -> Result<()> {
    let st = s.doc()?;
    if p.get("uid").and_then(Value::as_u64) != Some(st.uid) || p.get("revision").and_then(Value::as_u64) != Some(st.revision) {
        return Err(bad(C, "the document changed; detect elements again"));
    }
    if bool_or(p, "selectionOnly", false) && p.get("selectionIds") != Some(&json!(st.selection.objects.iter().map(|id| id.0).collect::<Vec<_>>())) {
        return Err(bad(C, "the selection changed; detect elements again"));
    }
    Ok(())
}

/// Prepare a disposable export copy. Only a bottommost, plain rectangular sheet-sized
/// backdrop is eligible; clipped/styled containers and the original document stay intact.
pub fn prepare_source(mut doc: Document, ignore_background: bool) -> Document {
    use std::sync::Arc;
    use vectorcraft_doc::{Node, NodeKind};
    fn visit(doc: &Document, node: &mut Arc<Node>, depth: usize) {
        if depth >= 32
            || !node.visible
            || !node.has_default_transparency()
            || !node.appearance.effects.is_empty()
            || !node.appearance.items.is_empty()
            || !matches!(node.kind, NodeKind::Group { clip: false, .. } | NodeKind::Layer { clip: false, .. })
        {
            return;
        }
        let Some(children) = node.children() else { return };
        let visible: Vec<_> = children.iter().filter(|n| n.visible && !n.is_template()).collect();
        let remove = visible.first().filter(|_| visible.len() >= 3).and_then(|n| {
            let NodeKind::Path { path, clipping: false, guide: false, .. } = &n.kind else { return None };
            let sub = path.subpaths.first()?;
            if path.subpaths.len() != 1
                || !sub.closed
                || sub.anchors.len() != 4
                || !n.has_default_transparency()
                || !n.appearance.effects.is_empty()
                || n.appearance.items.len() > 2
            {
                return None;
            }
            let bounds = path.bounds()?;
            if sub.anchors.iter().any(|a| {
                a.h_in != a.p || a.h_out != a.p || !((a.p.x == bounds.x0 || a.p.x == bounds.x1) && (a.p.y == bounds.y0 || a.p.y == bounds.y1))
            }) {
                return None;
            }
            let fill = n.appearance.fill()?;
            if !fill.visible
                || fill.opacity != 1.0
                || !fill.effects.is_empty()
                || fill.blend != vectorcraft_color::BlendMode::Normal
                || !matches!(fill.paint, vectorcraft_color::Paint::Solid { .. })
                || n.appearance.stroke().is_some_and(|s| s.visible && s.width > 0.0 && !matches!(s.paint, vectorcraft_color::Paint::None))
            {
                return None;
            }
            let painted = vectorcraft_render::painted_bounds(doc, n)?;
            let near = |a: Rect, b: Rect| {
                (a.x0 - b.x0).abs().max((a.x1 - b.x1).abs()) <= b.width() * 0.01 && (a.y0 - b.y0).abs().max((a.y1 - b.y1).abs()) <= b.height() * 0.01
            };
            if !doc.artboards.iter().any(|a| near(painted, a.rect)) {
                return None;
            }
            if !visible.iter().skip(1).all(|n| {
                vectorcraft_render::painted_bounds(doc, n)
                    .is_none_or(|r| r.x0 >= painted.x0 - 1.0 && r.y0 >= painted.y0 - 1.0 && r.x1 <= painted.x1 + 1.0 && r.y1 <= painted.y1 + 1.0)
            }) {
                return None;
            }
            Some(n.id)
        });
        if let Some(children) = Arc::make_mut(node).children_mut() {
            if let Some(id) = remove {
                children.retain(|n| n.id != id);
            }
            for child in children {
                visit(doc, child, depth + 1);
            }
        }
    }
    if ignore_background {
        let reference = doc.clone();
        for layer in &mut doc.layers {
            visit(&reference, layer, 0);
        }
    }
    doc
}

/// Preserve ancestors, clipping and appearance while isolating a detected source group.
/// Component members are retained together even when they have no common source group.
/// Manual regions without source IDs retain ordinary rectangular cropping.
pub fn region_source(doc: &Document, row: &Value) -> Result<Document> {
    let Some(value) = row.get("sourceIds") else { return Ok(doc.clone()) };
    let ids = value.as_array().filter(|ids| ids.len() <= 4096).ok_or_else(|| bad(C, "sourceIds must be an array of at most 4096 object IDs"))?;
    if ids.is_empty() {
        return Ok(doc.clone());
    }
    let mut keep = std::collections::HashSet::new();
    for id in ids {
        let id = NodeId(id.as_u64().ok_or_else(|| bad(C, "sourceIds must contain object IDs"))?);
        if !keep.insert(id) || doc.node(id).is_none() {
            return Err(bad(C, "sourceIds contain missing or duplicate objects"));
        }
    }
    let mut isolated = doc.clone();
    isolated.layers = doc.layers.iter().filter_map(|layer| fileio::selected_part(layer, &keep)).map(std::sync::Arc::new).collect();
    if isolated.layers.is_empty() {
        return Err(bad(C, "no source objects in this scope"));
    }
    Ok(isolated)
}

fn snapshot(s: &mut Session, p: &Value) -> Result<Value> {
    let st = s.doc()?;
    let doc = prepare_source(
        fileio::export_source(st, &json!({"selectedOnly":bool_or(p,"selectionOnly",false)}))?.into_owned(),
        bool_or(p, "ignoreBackground", true),
    );
    let r = doc
        .layers
        .iter()
        .filter(|n| n.visible && !n.is_template())
        .filter_map(|n| vectorcraft_render::painted_bounds(&doc, n))
        .reduce(|a, b| a.union(b))
        .filter(|r| slices::slice_rect_ok(*r))
        .ok_or_else(|| bad(C, "no artwork in this scope"))?;
    let areas = asset_regions(&doc)?;
    let rows: Vec<_> = areas
        .iter()
        .enumerate()
        .map(|(i, region)| {
            let r = region.rect;
            json!({"id":i+1,"x":r.x0,"y":r.y0,"width":r.width(),"height":r.height(),"sourceIds":region.source_ids})
        })
        .collect();
    Ok(
        json!({"uid":st.uid,"revision":st.revision,"rect":[r.x0,r.y0,r.width(),r.height()],"selectionIds":st.selection.objects.iter().map(|id| id.0).collect::<Vec<_>>(),"regions":rows}),
    )
}

/// Separate spatial islands of sibling artwork. Overlapping members stay together; clipped,
/// masked or styled containers are indivisible, so a screen does not become its interior paths.
/// The bounds come from the renderer, never from model-generated coordinates.
#[derive(Debug)]
struct AssetRegion {
    rect: Rect,
    source_ids: Vec<u64>,
}

fn asset_regions(doc: &Document) -> Result<Vec<AssetRegion>> {
    use vectorcraft_doc::{Node, NodeKind};
    fn owned(rect: Rect, node: &Node) -> AssetRegion {
        let source_ids = if node.id.0 == 0 { node.children().into_iter().flatten().map(|n| n.id.0).collect() } else { vec![node.id.0] };
        AssetRegion { rect, source_ids }
    }
    fn visit(doc: &Document, node: &Node, depth: usize, out: &mut Vec<AssetRegion>) -> Result<()> {
        if !node.visible || node.is_template() {
            return Ok(());
        }
        let Some(bounds) = vectorcraft_render::painted_bounds(doc, node).filter(|r| slices::slice_rect_ok(*r)) else { return Ok(()) };
        let protected = !matches!(node.kind, NodeKind::Group { clip: false, .. } | NodeKind::Layer { clip: false, .. })
            || node.extras().shaper.is_some()
            || !node.has_default_transparency()
            || !node.appearance.effects.is_empty()
            || !node.appearance.items.is_empty();
        let children = node.children();
        if protected || depth >= 32 || children.is_none() {
            out.push(owned(bounds, node));
        } else if let Some(children) = children {
            let parts: Vec<_> = children
                .iter()
                .filter(|n| n.visible && !n.is_template())
                .filter_map(|n| vectorcraft_render::painted_bounds(doc, n).filter(|r| slices::slice_rect_ok(*r)).map(|r| (n, r)))
                .collect();
            if parts.len() == 1 {
                if let Some((child, _)) = parts.first() {
                    visit(doc, child, depth + 1, out)?;
                }
            } else if parts.len() > 4096 {
                return Err(bad(C, "too many sibling objects; select a smaller group"));
            } else {
                // Connected components of intersecting painted bounds; bounded to 4096 siblings.
                let mut seen = vec![false; parts.len()];
                let mut components = Vec::new();
                for seed in 0..parts.len() {
                    if seen.get(seed).copied().unwrap_or(true) {
                        continue;
                    }
                    let mut queue = vec![seed];
                    if let Some(v) = seen.get_mut(seed) {
                        *v = true;
                    }
                    let mut cursor = 0;
                    while let Some(&i) = queue.get(cursor) {
                        let Some((_, a)) = parts.get(i) else { break };
                        for (j, (_, b)) in parts.iter().enumerate() {
                            if !seen.get(j).copied().unwrap_or(true)
                                && (a.x1.min(b.x1) - a.x0.max(b.x0)).max(0.0) * (a.y1.min(b.y1) - a.y0.max(b.y0)).max(0.0)
                                    > 0.02 * a.area().min(b.area())
                            {
                                if let Some(v) = seen.get_mut(j) {
                                    *v = true;
                                }
                                queue.push(j);
                            }
                        }
                        cursor += 1;
                    }
                    components.push(queue);
                }
                let component_bounds: Vec<_> = components
                    .iter()
                    .filter_map(|queue| queue.iter().filter_map(|i| parts.get(*i).map(|(_, r)| *r)).reduce(|a, b| a.union(b)))
                    .collect();
                if component_bounds.len() > 1 {
                    let largest = component_bounds.iter().copied().max_by(|a, b| a.area().total_cmp(&b.area()));
                    if let Some(main) = largest {
                        let small_area: f64 = component_bounds.iter().map(|r| r.area()).sum::<f64>() - main.area();
                        let gap = bounds.width().max(bounds.height()) * 0.05;
                        // Small detached captions belong to a dominant control, rather than
                        // turning each outlined digit above a slider into a separate asset.
                        if main.area() >= bounds.area() * 0.6
                            && small_area <= main.area() * 0.05
                            && component_bounds
                                .iter()
                                .all(|r| r.x0 <= main.x1 + gap && r.x1 >= main.x0 - gap && r.y0 <= main.y1 + gap && r.y1 >= main.y0 - gap)
                        {
                            out.push(owned(bounds, node));
                            return Ok(());
                        }
                    }
                }
                for queue in components {
                    if queue.len() == 1 {
                        if let Some((child, _)) = queue.first().and_then(|index| parts.get(*index)) {
                            visit(doc, child, depth + 1, out)?;
                        }
                    } else if let Some(r) = queue.iter().filter_map(|i| parts.get(*i).map(|(_, r)| *r)).reduce(|a, b| a.union(b)) {
                        // A painted backdrop makes its container one coherent asset. Without
                        // one, unpack spatial containers before their empty bounds can bridge
                        // neighbouring assets. Small nested details are not split here.
                        let coherent = queue.iter().filter_map(|i| parts.get(*i)).any(|(n, b)| {
                            b.area() >= r.area() * 0.8
                                && (!matches!(n.kind, NodeKind::Group { clip: false, .. } | NodeKind::Layer { clip: false, .. })
                                    || !n.has_default_transparency()
                                    || !n.appearance.effects.is_empty()
                                    || !n.appearance.items.is_empty())
                        });
                        let mut unpacked = Vec::new();
                        let mut split = false;
                        if !coherent {
                            for index in &queue {
                                if let Some((child, bounds)) = parts.get(*index) {
                                    let mut regions = Vec::new();
                                    if bounds.area() >= r.area() * 0.2 {
                                        visit(doc, child, depth + 1, &mut regions)?;
                                    } else {
                                        regions.push(owned(*bounds, child));
                                    }
                                    split |= regions.len() > 1;
                                    unpacked.extend(regions);
                                }
                            }
                        }
                        if split {
                            out.extend(unpacked);
                        } else {
                            let region = if queue.len() == parts.len() {
                                owned(r, node)
                            } else {
                                AssetRegion { rect: r, source_ids: queue.iter().filter_map(|i| parts.get(*i).map(|(n, _)| n.id.0)).collect() }
                            };
                            out.push(region);
                        }
                    }
                    if out.len() > LIMIT {
                        return Err(bad(C, "more than 64 separated elements; select a smaller group"));
                    }
                }
            }
        }
        if out.len() > LIMIT {
            return Err(bad(C, "more than 64 separated elements; select a smaller group"));
        }
        Ok(())
    }
    let root = Node::group(NodeId(0), doc.layers.clone());
    let mut out = Vec::new();
    visit(doc, &root, 0, &mut out)?;
    if out.is_empty() {
        return Err(bad(C, "no artwork in this scope"));
    }
    Ok(out)
}

fn regions(p: &Value) -> Result<Vec<(String, Rect)>> {
    let rows = p
        .get("regions")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty() && a.len() <= LIMIT)
        .ok_or_else(|| bad(C, "regions must contain 1..64 entries"))?;
    rows.iter()
        .map(|row| {
            let name = row
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty() && s.chars().count() <= 128 && !s.chars().any(char::is_control))
                .ok_or_else(|| bad(C, "each region needs a name of 1..128 printable characters"))?;
            let x = f64_req(row, "x", C)?;
            let y = f64_req(row, "y", C)?;
            let w = f64_req(row, "width", C)?;
            let h = f64_req(row, "height", C)?;
            let r = Rect::new(x, y, x + w, y + h);
            if w <= 0.0 || h <= 0.0 || !slices::slice_rect_ok(r) {
                return Err(bad(C, "each region needs finite canvas coordinates and a positive size"));
            }
            Ok((name.to_string(), r))
        })
        .collect()
}

fn apply(s: &mut Session, p: &Value) -> Result<Value> {
    guard(s, p)?;
    let regions = regions(p)?;
    let ids = s.edit("Create AI Slices", |doc, sel| {
        let ids: Vec<_> = regions.into_iter().map(|(name, rect)| slices::add_slice(doc, rect, SliceOptions { name, ..Default::default() })).collect();
        if !bool_or(p, "preserveSelection", false) {
            sel.set_slices(ids.iter().copied());
        }
        Ok(ids)
    })?;
    Ok(json!({"ids":ids.iter().map(|id| id.0).collect::<Vec<_>>()}))
}

fn export(s: &mut Session, p: &Value) -> Result<Value> {
    guard(s, p)?;
    let regions = regions(p)?;
    let format = str_param(p, "format").unwrap_or("svg");
    if !matches!(format, "svg" | "png") {
        return Err(bad(C, "format must be svg or png"));
    }
    let scale = f64_or(p, "scale", 1.0);
    if !scale.is_finite() || scale <= 0.0 || scale > 16.0 {
        return Err(bad(C, "scale must be greater than zero and at most 16"));
    }
    let mut pixels = 0u64;
    if format == "png" {
        for (_, r) in &regions {
            let (w, h) = vectorcraft_render::raster_size(*r, scale).map_err(|e| bad(C, e))?;
            pixels = pixels.saturating_add(u64::from(w) * u64::from(h));
        }
        if pixels > 16_000_000 {
            return Err(bad(C, "batch exceeds 16 million pixels; reduce scale or export fewer regions"));
        }
    }
    let doc = prepare_source(
        fileio::export_source(s.doc()?, &json!({"selectedOnly":bool_or(p,"selectionOnly",false)}))?.into_owned(),
        bool_or(p, "ignoreBackground", true),
    );
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut renderer = vectorcraft_render::Renderer::new();
    let mut bytes_total = 0usize;
    // Encode and validate the whole batch before creating files.
    for (i, (name, r)) in regions.iter().enumerate() {
        let row = p.get("regions").and_then(Value::as_array).and_then(|rows| rows.get(i)).ok_or_else(|| bad(C, "missing region"))?;
        let source = region_source(&doc, row)?;
        let mut stem = String::new();
        for c in fileio::safe_file_name(name).chars() {
            if stem.len() + c.len_utf8() > 180 {
                break;
            }
            stem.push(c);
        }
        let number = p["regions"].get(i).and_then(|row| row["candidateIndex"].as_u64()).filter(|n| *n < 64).map_or(i + 1, |n| n as usize + 1);
        let filename = format!("{number:02}-{stem}.{format}");
        let bytes = if format == "svg" {
            let mut cropped = fileio::art_over(&source, *r);
            cropped.artboards = vec![Artboard { id: 1, name: name.clone(), rect: *r, show_center_mark: false, show_cross_hairs: false }];
            let (svg, notes) = vectorcraft_svg::export_with_report(&cropped, &vectorcraft_svg::ExportOptions::default());
            warnings.extend(notes);
            svg.into_bytes()
        } else {
            renderer.render_region(&source, *r, scale, false).to_png().map_err(EngineError::Other)?
        };
        bytes_total = bytes_total.saturating_add(bytes.len());
        if bytes_total > 64 * 1024 * 1024 {
            return Err(bad(C, "encoded batch exceeds 64 MiB; export fewer regions"));
        }
        files.push((filename, bytes));
    }
    warnings.sort();
    warnings.dedup();
    if let Some(folder) = str_param(p, "folder").filter(|f| !f.trim().is_empty()) {
        // Use the existing path policy and exclusive writer; never overwrite a user's file.
        let paths: Vec<_> = files.iter().map(|(name, _)| std::path::Path::new(folder).join(name).to_string_lossy().into_owned()).collect();
        if paths.iter().any(|path| std::path::Path::new(path).exists()) {
            return Err(bad(C, "an output file already exists; choose an empty folder"));
        }
        fileio::create_dir(folder)?;
        let mut written = Vec::new();
        for ((_, bytes), path) in files.iter().zip(&paths) {
            if let Err(e) = fileio::write_new_file(path, bytes) {
                // Only roll back new files this export created.
                #[cfg(not(target_arch = "wasm32"))]
                for path in &written {
                    let _ = std::fs::remove_file(path);
                }
                return Err(e);
            }
            written.push(path.clone());
        }
        Ok(json!({"files":written,"warnings":warnings}))
    } else {
        Ok(
            json!({"files":files.into_iter().map(|(name,bytes)| json!({"name":name,"dataBase64":vectorcraft_svg::base64_encode(&bytes)})).collect::<Vec<_>>(),"warnings":warnings}),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn player_parameters_are_advertised() {
        super::super::params::check_params("slice.applyAI", &serde_json::json!({"uid":1,"revision":0,"preserveSelection":true,"regions":[]}))
            .unwrap();
        super::super::params::check_params("slice.exportAI", &serde_json::json!({"uid":1,"revision":0,"regions":[{"candidateIndex":3}]})).unwrap();
    }

    use super::*;
    use serde_json::json;
    fn fixture() -> (Session, Value) {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width":200,"height":100})).unwrap();
        s.execute("shape.rectangle", &json!({"x":10,"y":10,"width":40,"height":20})).unwrap();
        s.execute("shape.rectangle", &json!({"x":100,"y":10,"width":30,"height":20})).unwrap();
        let snapshot = s.execute("slice.aiSnapshot", &json!({})).unwrap();
        let p = json!({"uid":snapshot["uid"],"revision":snapshot["revision"],"regions":[{"name":"Play Button","x":8,"y":8,"width":44,"height":24},{"name":"Exit Button","x":98,"y":8,"width":34,"height":24}]});
        (s, p)
    }
    #[test]
    fn spatial_islands_keep_overlapping_and_clipped_parts_together() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><g><g><rect x="5" y="5" width="60" height="40" fill="#000"/><circle cx="35" cy="25" r="10" fill="#f00"/></g><g><rect x="100" y="5" width="60" height="40" fill="#000"/><circle cx="130" cy="25" r="10" fill="#f00"/></g></g></svg>"##;
        let doc = vectorcraft_svg::import(svg).unwrap();
        let mut session = Session::new();
        session.add_document(doc, None);
        let before = session.doc().unwrap().doc.clone();
        let snap = session.execute("slice.aiSnapshot", &json!({})).unwrap();
        assert_eq!(snap["regions"].as_array().unwrap().len(), 2);
        assert!(std::sync::Arc::ptr_eq(&before, &session.doc().unwrap().doc));
        let islands = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><g><g><rect x="5" y="5" width="20" height="20"/><rect x="100" y="5" width="20" height="20"/></g><rect x="50" y="5" width="20" height="20"/></g></svg>"##;
        assert_eq!(
            asset_regions(&vectorcraft_svg::import(islands).unwrap()).unwrap().len(),
            3,
            "empty container space must not bridge separate assets"
        );
        let caption = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><g><rect x="5" y="20" width="180" height="20"/><rect x="10" y="10" width="3" height="5"/><rect x="15" y="10" width="3" height="5"/></g></svg>"##;
        assert_eq!(
            asset_regions(&vectorcraft_svg::import(caption).unwrap()).unwrap().len(),
            1,
            "small detached captions stay with their dominant control"
        );
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><defs><clipPath id="c"><rect width="180" height="70"/></clipPath></defs><g clip-path="url(#c)"><rect x="5" y="5" width="60" height="40"/><rect x="100" y="5" width="60" height="40"/></g></svg>"##;
        let doc = vectorcraft_svg::import(svg).unwrap();
        assert_eq!(asset_regions(&doc).unwrap().len(), 1, "a complete clipped screen stays whole");
        if let Ok(path) = std::env::var("VECTORCRAFT_SLICING_SVG") {
            let doc = vectorcraft_svg::import(&std::fs::read_to_string(path).unwrap()).unwrap();
            let areas = asset_regions(&prepare_source(doc, true)).unwrap();
            eprintln!("Real SVG spatial regions ({}): {:?}", areas.len(), areas);
            assert!(areas.len() > 2, "the multi-asset fixture must separate into individual elements");
        }
    }
    #[test]
    fn sheet_background_is_optional_and_only_removed_in_export_copy() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><g><rect width="200" height="100" fill="#fff"/><rect x="10" y="10" width="40" height="20" fill="#f00"/><rect x="100" y="10" width="40" height="20" fill="#00f"/></g></svg>"##;
        let doc = vectorcraft_svg::import(svg).unwrap();
        let prepared = prepare_source(doc.clone(), true);
        assert_eq!(asset_regions(&doc).unwrap().len(), 1);
        assert_eq!(asset_regions(&prepared).unwrap().len(), 2);
        assert_eq!(asset_regions(&prepare_source(doc.clone(), false)).unwrap().len(), 1);
        let mut renderer = vectorcraft_render::Renderer::new();
        let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
        assert_eq!(renderer.render_region(&doc, bounds, 1.0, false).pixels[3], 255);
        assert_eq!(renderer.render_region(&prepared, bounds, 1.0, false).pixels[3], 0);
        let mut session = Session::new();
        session.add_document(doc, None);
        let snapshot = session.execute("slice.aiSnapshot", &json!({})).unwrap();
        let mut params = json!({"uid":snapshot["uid"],"revision":snapshot["revision"],"format":"png","regions":[{"name":"Button","x":8,"y":8,"width":44,"height":24}]});
        for (ignore, alpha) in [(true, 0), (false, 255)] {
            params["ignoreBackground"] = json!(ignore);
            let out = session.execute("slice.exportAI", &params).unwrap();
            let bytes = vectorcraft_format::base64_decode(out["files"][0]["dataBase64"].as_str().unwrap()).unwrap();
            assert_eq!(image::load_from_memory(&bytes).unwrap().to_rgba8().get_pixel(0, 0).0[3], alpha);
        }
        // A white background belonging to an individual button is never a sheet backdrop.
        let button = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><g><rect x="10" y="10" width="40" height="20" fill="#fff"/><circle cx="20" cy="20" r="3"/><circle cx="40" cy="20" r="3"/></g></svg>"##;
        let button = vectorcraft_svg::import(button).unwrap();
        assert_eq!(
            renderer.render_region(&button, bounds, 1.0, false).pixels,
            renderer.render_region(&prepare_source(button.clone(), true), bounds, 1.0, false).pixels
        );
    }
    #[test]
    fn regions_are_atomic_guarded_and_undoable_without_changing_art() {
        let (mut s, p) = fixture();
        let doc = s.doc().unwrap().doc.clone();
        let rect = doc.artboards[0].rect;
        let mut renderer = vectorcraft_render::Renderer::new();
        let before = renderer.render_region(&doc, rect, 1.0, false).pixels;
        let mut bad = p.clone();
        bad["regions"][1]["width"] = json!(-1);
        assert!(s.execute("slice.applyAI", &bad).is_err());
        assert!(s.doc().unwrap().doc.slices.is_empty());
        let out = s.execute("slice.applyAI", &p).unwrap();
        assert_eq!(out["ids"].as_array().unwrap().len(), 2);
        assert_eq!(s.doc().unwrap().doc.slices[0].options.name, "Play Button");
        assert_eq!(renderer.render_region(&s.doc().unwrap().doc, rect, 1.0, false).pixels, before);
        assert!(s.execute("slice.applyAI", &p).is_err());
        s.execute("edit.undo", &json!({})).unwrap();
        assert!(s.doc().unwrap().doc.slices.is_empty());
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc.slices.len(), 2);
    }
    #[test]
    fn exports_separate_vector_and_transparent_pixel_crops() {
        let (mut s, mut p) = fixture();
        let revision = s.doc().unwrap().revision;
        for format in ["svg", "png"] {
            p["format"] = json!(format);
            let out = s.execute("slice.exportAI", &p).unwrap();
            assert_eq!(out["files"].as_array().unwrap().len(), 2);
            for f in out["files"].as_array().unwrap() {
                let bytes = vectorcraft_format::base64_decode(f["dataBase64"].as_str().unwrap()).unwrap();
                if format == "svg" {
                    let text = String::from_utf8(bytes).unwrap();
                    assert!(text.contains("<path"));
                    let d = vectorcraft_svg::import(&text).unwrap();
                    assert_eq!(d.artboards[0].rect.height(), 24.0);
                    assert!(d.art_bounds().unwrap().width() < 50.0, "other button was removed");
                } else {
                    let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
                    assert_eq!(image.height(), 24);
                    assert_eq!(image.get_pixel(0, 0).0[3], 0, "crop background stays transparent");
                    assert!(image.pixels().any(|p| p.0[3] != 0));
                }
            }
        }
        assert_eq!(s.doc().unwrap().revision, revision);
        p["scale"] = json!(16);
        p["regions"][0]["width"] = json!(100000);
        assert!(s.execute("slice.exportAI", &p).is_err());
    }
    #[test]
    fn vector_crops_keep_clipping_and_group_opacity() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60"><defs><clipPath id="clip"><circle cx="30" cy="30" r="20"/></clipPath></defs><g clip-path="url(#clip)" opacity="0.6"><rect width="70" height="60" fill="#ff0000"/></g><rect x="90" y="10" width="20" height="20" fill="#0000ff"/></svg>"##;
        let doc = vectorcraft_svg::import(svg).unwrap();
        let mut s = Session::new();
        s.add_document(doc, None);
        let snap = s.execute("slice.aiSnapshot", &json!({})).unwrap();
        let out=s.execute("slice.exportAI",&json!({"uid":snap["uid"],"revision":snap["revision"],"format":"svg","regions":[{"name":"Masked Button","x":5,"y":5,"width":50,"height":50}]})).unwrap();
        let bytes = vectorcraft_format::base64_decode(out["files"][0]["dataBase64"].as_str().unwrap()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("clipPath"));
        let cropped = vectorcraft_svg::import(&text).unwrap();
        let mut renderer = vectorcraft_render::Renderer::new();
        let actual = renderer.render_region(&cropped, Rect::new(0.0, 0.0, 50.0, 50.0), 1.0, false);
        let expected = renderer.render_region(&s.doc().unwrap().doc, Rect::new(5.0, 5.0, 55.0, 55.0), 1.0, false);
        assert_eq!(actual.pixels, expected.pixels);
    }
    #[test]
    fn detected_sources_exclude_neighbours_and_preserve_ancestor_opacity() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><g opacity="0.6"><rect x="10" y="10" width="40" height="20" fill="#f00"/></g><rect x="100" y="10" width="40" height="20" fill="#00f"/></svg>"##;
        let mut session = Session::new();
        session.add_document(vectorcraft_svg::import(svg).unwrap(), None);
        let snap = session.execute("slice.aiSnapshot", &json!({})).unwrap();
        let first = snap["regions"].as_array().unwrap().iter().find(|r| r["x"].as_f64().unwrap() < 50.0).unwrap();
        assert!(!first["sourceIds"].as_array().unwrap().is_empty());
        let mut row = json!({"name":"Red Button","x":0,"y":0,"width":200,"height":100,"sourceIds":first["sourceIds"]});
        for format in ["png", "svg"] {
            let params = json!({"uid":snap["uid"],"revision":snap["revision"],"format":format,"regions":[row]});
            let exported = session.execute("slice.exportAI", &params).unwrap();
            let bytes = vectorcraft_format::base64_decode(exported["files"][0]["dataBase64"].as_str().unwrap()).unwrap();
            let pixels = if format == "png" {
                image::load_from_memory(&bytes).unwrap().to_rgba8().into_raw()
            } else {
                let imported = vectorcraft_svg::import(&String::from_utf8(bytes).unwrap()).unwrap();
                vectorcraft_render::Renderer::new().render_region(&imported, Rect::new(0.0, 0.0, 200.0, 100.0), 1.0, false).pixels
            };
            assert!((pixels[(15 * 200 + 15) * 4 + 3] as i32 - 153).abs() <= 1, "ancestor opacity stays applied");
            assert_eq!(pixels[(15 * 200 + 105) * 4 + 3], 0, "nearby blue button must not enter a source-isolated crop");
        }
        row["sourceIds"] = json!([u64::MAX]);
        assert!(session.execute("slice.exportAI", &json!({"uid":snap["uid"],"revision":snap["revision"],"regions":[row]})).is_err());
    }
    #[test]
    fn source_isolation_keeps_every_ungrouped_component_member() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><g><rect x="10" y="10" width="40" height="20" fill="#f00"/></g><circle cx="30" cy="20" r="5" fill="#00f"/><rect x="100" y="10" width="30" height="20" fill="#0f0"/></svg>"##;
        let mut session = Session::new();
        session.add_document(vectorcraft_svg::import(svg).unwrap(), None);
        let snapshot = session.execute("slice.aiSnapshot", &json!({})).unwrap();
        let mut row = snapshot["regions"].as_array().unwrap().iter().find(|r| r["x"].as_f64().unwrap() < 50.0).unwrap().clone();
        assert_eq!(row["sourceIds"].as_array().unwrap().len(), 2, "the red group and separate blue icon must both be retained");
        row["name"] = json!("Composite Button");
        row["x"] = json!(0);
        row["y"] = json!(0);
        row["width"] = json!(200);
        row["height"] = json!(100);
        let result = session
            .execute("slice.exportAI", &json!({"uid":snapshot["uid"],"revision":snapshot["revision"],"format":"png","regions":[row]}))
            .unwrap();
        let bytes = vectorcraft_format::base64_decode(result["files"][0]["dataBase64"].as_str().unwrap()).unwrap();
        let pixels = image::load_from_memory(&bytes).unwrap().to_rgba8();
        assert_eq!(pixels.get_pixel(30, 20).0, [0, 0, 255, 255], "blue icon stays above red background");
        assert_eq!(pixels.get_pixel(15, 15).0, [255, 0, 0, 255]);
        assert_eq!(pixels.get_pixel(105, 15).0[3], 0, "unrelated green control stays outside this source component");
    }
    #[test]
    fn changed_selection_is_rejected_and_filenames_stay_in_folder() {
        let (mut s, mut p) = fixture();
        let snap = s.execute("slice.aiSnapshot", &json!({"selectionOnly":true})).unwrap();
        p["selectionOnly"] = json!(true);
        p["selectionIds"] = snap["selectionIds"].clone();
        s.execute("select.all", &json!({})).unwrap();
        assert!(s.execute("slice.exportAI", &p).is_err());
        for name in ["../../outside", "\\bad:name", ".", ".."] {
            let safe = fileio::safe_file_name(name);
            assert!(!safe.contains('/'));
            assert!(!safe.contains('\\'));
            assert_ne!(safe, "..");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn never_overwrites_existing_exports() {
        let (mut s, mut p) = fixture();
        let folder = std::env::temp_dir().join(format!("vectorcraft-ai-slices-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        p["folder"] = json!(folder.to_string_lossy());
        let out = s.execute("slice.exportAI", &p).unwrap();
        let path = out["files"][0].as_str().unwrap();
        let before = std::fs::read(path).unwrap();
        assert!(s.execute("slice.exportAI", &p).is_err());
        assert_eq!(std::fs::read(path).unwrap(), before);
        std::fs::remove_dir_all(folder).unwrap();
    }
}
