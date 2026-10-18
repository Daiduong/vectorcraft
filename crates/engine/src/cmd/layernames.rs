//! Read-only naming candidates and atomic, guarded application of reviewed names.
use super::*;
use serde_json::json;
use std::collections::HashSet;

const LIMIT: usize = 1000;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "layer.namingCandidates",
            "Layer Naming Candidates",
            [],
            None,
            "{scope?: all|selection, depth?: 1..16} layers and groups, in document order; selection includes highlighted rows or selected objects and descendants. At most 1000 → {uid, revision, rows: [{id, name, parent}]}",
            has_doc,
            candidates
        ),
        cmd!(
            "layer.applyNames",
            "Apply Layer Names",
            [],
            None,
            "{uid, revision, names: [{id, name}]} apply reviewed layer/group names atomically in one undo step; reject stale documents, duplicate or missing ids and empty/oversized names → {renamed}",
            has_doc,
            apply
        ),
    ]
}

fn candidates(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.namingCandidates";
    let st = s.doc()?;
    let scope = p.get("scope").and_then(Value::as_str).unwrap_or("all");
    if !matches!(scope, "all" | "selection") {
        return Err(bad(C, "scope is all or selection"));
    }
    let depth = match p.get("depth") {
        None => 3,
        Some(v) => v.as_u64().filter(|n| (1..=16).contains(n)).ok_or_else(|| bad(C, "depth is 1..16"))?,
    };
    let roots = if scope == "all" {
        st.doc.layers.iter().map(|n| n.id).collect()
    } else if !st.layer_rows.is_empty() {
        st.layer_rows.clone()
    } else {
        st.selection.objects.clone()
    };
    if roots.is_empty() {
        return Err(bad(C, "select layers or groups first"));
    }
    let roots = super::layer::row_roots(&st.doc, &roots);
    let mut rows = Vec::new();
    fn visit(n: &vectorcraft_doc::Node, parent: Option<NodeId>, depth: u64, rows: &mut Vec<Value>) {
        if depth == 0 || rows.len() > LIMIT {
            return;
        }
        if matches!(n.kind, vectorcraft_doc::NodeKind::Layer { .. } | vectorcraft_doc::NodeKind::Group { .. }) {
            rows.push(json!({"id": n.id.0, "name": n.name, "parent": parent.map(|n| n.0)}));
        }
        if let Some(children) = n.children() {
            for child in children {
                visit(child, Some(n.id), depth - 1, rows);
            }
        }
    }
    for id in roots {
        if let Some(n) = st.doc.node(id) {
            visit(n, None, depth, &mut rows);
        }
    }
    if rows.len() > LIMIT {
        return Err(bad(C, "more than 1000 groups: narrow the selection or depth"));
    }
    if rows.is_empty() {
        return Err(bad(C, "no layers or groups in this scope"));
    }
    Ok(json!({"uid": st.uid, "revision": st.revision, "rows": rows}))
}

fn apply(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.applyNames";
    let st = s.doc()?;
    if p.get("uid").and_then(Value::as_u64) != Some(st.uid) || p.get("revision").and_then(Value::as_u64) != Some(st.revision) {
        return Err(bad(C, "the document changed; generate names again"));
    }
    let names = p
        .get("names")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty() && a.len() <= LIMIT)
        .ok_or_else(|| bad(C, "names must contain 1..1000 entries"))?;
    let mut seen = HashSet::new();
    let mut edits = Vec::new();
    for row in names {
        let id = row.get("id").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(C, "missing id"))?;
        if !seen.insert(id) {
            return Err(bad(C, "duplicate id"));
        }
        let name = row
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|n| !n.is_empty() && n.chars().count() <= 512 && !n.chars().any(char::is_control))
            .ok_or_else(|| bad(C, "name must be 1..512 printable characters"))?;
        let node = st
            .doc
            .node(id)
            .filter(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Layer { .. } | vectorcraft_doc::NodeKind::Group { .. }))
            .ok_or_else(|| bad(C, "id must name an existing layer or group"))?;
        if node.name.as_deref() != Some(name) {
            edits.push((id, name.to_string()));
        }
    }
    let renamed = edits.len();
    if renamed != 0 {
        s.edit("Name Layers with AI", |doc, _| {
            for (id, name) in edits {
                if let Some(node) = doc.node_mut(id) {
                    node.name = Some(name);
                }
            }
            Ok(())
        })?;
    }
    Ok(json!({"renamed": renamed}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_depth_and_svg_names_preserve_artwork() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        s.execute("shape.rectangle", &json!({"x": 10, "y": 20, "width": 30, "height": 40})).unwrap();
        s.execute("object.group", &json!({})).unwrap();
        let all = s.execute("layer.namingCandidates", &json!({"depth": 16})).unwrap();
        assert_eq!(all["rows"].as_array().unwrap().len(), 2);
        let shallow = s.execute("layer.namingCandidates", &json!({"depth": 1})).unwrap();
        assert_eq!(shallow["rows"].as_array().unwrap().len(), 1);
        s.execute("layer.highlight", &json!({"ids": []})).unwrap();
        let selected = s.execute("layer.namingCandidates", &json!({"scope": "selection"})).unwrap();
        assert_eq!(selected["rows"].as_array().unwrap().len(), 1);
        let doc = s.doc().unwrap().doc.clone();
        let bounds = doc.art_bounds().unwrap();
        let mut renderer = vectorcraft_render::Renderer::new();
        let before = renderer.render_region(&doc, bounds, 1.0, true).pixels;
        let names: Vec<_> =
            all["rows"].as_array().unwrap().iter().map(|r| json!({"id":r["id"], "name":format!("Play & Start {}",r["id"])})).collect();
        let current = s.execute("layer.namingCandidates", &json!({})).unwrap();
        s.execute("layer.applyNames", &json!({"uid":current["uid"], "revision":current["revision"], "names":names})).unwrap();
        let renamed = &s.doc().unwrap().doc;
        assert_eq!(renderer.render_region(renamed, bounds, 1.0, true).pixels, before);
        let svg = vectorcraft_svg::export(renamed, &vectorcraft_svg::ExportOptions::default());
        let imported = vectorcraft_svg::import(&svg).unwrap();
        for name in names {
            let mut found = false;
            imported.walk(|n| {
                found |= n.name.as_deref() == name["name"].as_str();
            });
            assert!(found);
        }
        for depth in [json!(0), json!(17), json!("bad")] {
            assert!(s.execute("layer.namingCandidates", &json!({"depth":depth})).is_err());
        }
    }

    #[test]
    fn naming_is_atomic_guarded_and_undoable() {
        let mut s = Session::new();
        s.execute("file.new", &json!({})).unwrap();
        let before = s.execute("layer.namingCandidates", &json!({})).unwrap();
        let id = before["rows"][0]["id"].clone();
        let mut p = json!({"uid": before["uid"], "revision": before["revision"], "names": [{"id": id, "name": "Play Button"}]});
        p["names"].as_array_mut().unwrap().push(json!({"id": 99999999, "name": "Bad"}));
        assert!(s.execute("layer.applyNames", &p).is_err());
        assert_eq!(s.execute("layer.namingCandidates", &json!({})).unwrap(), before);
        p["names"].as_array_mut().unwrap().pop();
        s.execute("layer.applyNames", &p).unwrap();
        assert!(s.execute("layer.applyNames", &p).is_err());
        assert_eq!(s.execute("layer.namingCandidates", &json!({})).unwrap()["rows"][0]["name"], "Play Button");
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.execute("layer.namingCandidates", &json!({})).unwrap()["rows"], before["rows"]);
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(s.execute("layer.namingCandidates", &json!({})).unwrap()["rows"][0]["name"], "Play Button");
    }
}
