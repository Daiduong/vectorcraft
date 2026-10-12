use super::*;
use crate::craft_codec as codec;
use serde_json::json;
impl Session {
    pub fn craft_signature(&self, ui: &Value) -> String {
        json!([
            ui,
            self.active,
            self.docs
                .iter()
                .map(|d| json!([
                    d.uid,
                    d.revision,
                    d.selection,
                    d.is_dirty(),
                    d.active_layer,
                    d.layer_rows,
                    d.isolation,
                    d.view,
                    d.transparency_grid,
                    d.save_options,
                    d.converted,
                    d.mask_view,
                    d.last_transform,
                    d.layers_open.saved(&d.doc),
                    d.path
                ]))
                .collect::<Vec<_>>(),
            self.paint,
            self.fill_active,
            self.tool.id()
        ])
        .to_string()
    }
    pub fn craft_capture(&self, ui: Value) -> std::result::Result<Vec<u8>, String> {
        let mut blobs = Vec::new();
        let mut ids = std::collections::HashMap::new();
        let mut put = |d: &Arc<Document>| -> std::result::Result<usize, String> {
            let key = Arc::as_ptr(d) as usize;
            if let Some(i) = ids.get(&key) {
                return Ok(*i);
            }
            let i = codec::push(&mut blobs, encode_doc(d)?)?;
            ids.insert(key, i);
            Ok(i)
        };
        let docs=self.docs.iter().map(|d| {
            let current=put(&d.doc)?;let saved=put(&d.saved_doc)?;
            let mut stack=|s:&Vec<HistoryEntry>| -> std::result::Result<Vec<Value>,String> {s.iter().map(|h| Ok(json!([h.label,put(&h.doc)?,h.selection]))).collect()};
            Ok(json!({"blob":current,"saved":saved,"undo":stack(&d.history.undo)?,"redo":stack(&d.history.redo)?,"limit":d.history.limit,"path":d.path,"revision":d.revision,"selection":d.selection,"layer":d.active_layer,"rows":d.layer_rows,"isolation":d.isolation,"view":d.view,"grid":d.transparency_grid,"options":d.save_options,"converted":d.converted,"mask":d.mask_view,"transform":d.last_transform,"maskEdit":d.doc.mask_edit,"format":d.format,"openRows":d.layers_open.saved(&d.doc),"imported":d.imported_from,"losses":d.import_losses}))
        }).collect::<std::result::Result<Vec<_>,String>>()?;
        codec::pack(
            &json!({"app":"vectorcraft","schema":1,"ui":ui,"docs":docs,"active":self.active,"paint":self.paint,"fill":self.fill_active,"prefs":self.prefs,"tool":self.tool.id()}),
            &blobs,
        )
    }
    pub fn craft_restore(&mut self, bytes: &[u8]) -> std::result::Result<Value, String> {
        let (v, blobs) = codec::unpack(bytes)?;
        if v["app"] != "vectorcraft" || v["schema"] != 1 {
            return Err("Incompatible VectorCraft session".into());
        }
        let mut decoded = std::collections::HashMap::<usize, Arc<Document>>::new();
        let mut get = |id: usize| -> std::result::Result<Arc<Document>, String> {
            if let Some(d) = decoded.get(&id) {
                return Ok(d.clone());
            }
            let d = Arc::new(decode_doc(codec::blob(&blobs, id)?)?);
            decoded.insert(id, d.clone());
            Ok(d)
        };
        let mut docs = Vec::new();
        for m in v["docs"].as_array().ok_or("Missing documents")? {
            let doc = get(codec::decode(&m["blob"])?)?;
            let mut d = DocState::new((*doc).clone(), codec::decode(&m["path"])?);
            d.doc = doc;
            d.saved_doc = get(codec::decode(&m["saved"])?)?;
            let mut stack = |s: &Value| -> std::result::Result<Vec<HistoryEntry>, String> {
                s.as_array()
                    .ok_or("Missing history")?
                    .iter()
                    .map(|h| Ok(HistoryEntry { label: codec::decode(&h[0])?, doc: get(codec::decode(&h[1])?)?, selection: codec::decode(&h[2])? }))
                    .collect()
            };
            d.history = History { undo: stack(&m["undo"])?, redo: stack(&m["redo"])?, limit: codec::decode(&m["limit"])? };
            d.revision = codec::decode(&m["revision"])?;
            d.selection = codec::decode(&m["selection"])?;
            d.active_layer = codec::decode(&m["layer"])?;
            d.layer_rows = codec::decode(&m["rows"])?;
            d.isolation = codec::decode(&m["isolation"])?;
            d.view = codec::decode(&m["view"])?;
            d.transparency_grid = codec::decode(&m["grid"])?;
            d.save_options = codec::decode(&m["options"])?;
            d.converted = codec::decode(&m["converted"])?;
            d.mask_view = codec::decode(&m["mask"])?;
            d.last_transform = codec::decode(&m["transform"])?;
            d.layers_open = OpenRows::new(&d.doc, codec::decode(&m["openRows"])?);
            d.imported_from = codec::decode(&m["imported"])?;
            d.import_losses = codec::decode(&m["losses"])?;
            d.format = cmd::fileio::SAVE_FORMATS.iter().copied().find(|f| Some(*f) == m["format"].as_str()).unwrap_or("vectorcraft");
            docs.push(d);
        }
        let active: Option<usize> = codec::decode(&v["active"])?;
        if active.is_some_and(|i| i >= docs.len()) {
            return Err("Invalid active tab".into());
        }
        let paint = codec::decode(&v["paint"])?;
        let fill = codec::decode(&v["fill"])?;
        let prefs = codec::decode(&v["prefs"])?;
        let tool = vectorcraft_tools::create(v["tool"].as_str().ok_or("Missing tool")?);
        self.docs = docs;
        self.active = active;
        self.paint = paint;
        self.fill_active = fill;
        self.prefs = prefs;
        self.tool = tool;
        if v["app"] != "vectorcraft" || v["schema"] != 1 {
            return Err("Incompatible VectorCraft session".into());
        }
        Ok(v["ui"].clone())
    }
}

impl Session {
    pub fn craft_ui(bytes: &[u8]) -> std::result::Result<Value, String> {
        let (v, _) = crate::craft_codec::unpack(bytes)?;
        if v["app"] != "vectorcraft" || v["schema"] != 1 {
            return Err("Incompatible VectorCraft session".into());
        }
        Ok(v["ui"].clone())
    }
}

// Unlike a document export, recovery keeps editing modes and every embedded/linked image byte.
fn encode_doc(d: &Document) -> std::result::Result<Vec<u8>, String> {
    let mut blobs = Vec::new();
    let mut images = serde_json::Map::new();
    for (name, img) in &d.images {
        let i = codec::push(&mut blobs, img.bytes.as_ref().clone())?;
        let proxy = img.proxy.as_ref().map(|b| codec::push(&mut blobs, b.as_ref().clone())).transpose()?;
        images.insert(name.clone(), json!([i, proxy]));
    }
    codec::pack(&json!({"document":d,"images":images,"maskEdit":d.mask_edit,"puppet":d.puppet}), &blobs)
}
fn decode_doc(bytes: &[u8]) -> std::result::Result<Document, String> {
    let (v, blobs) = codec::unpack(bytes)?;
    let mut d: Document = codec::decode(&v["document"])?;
    for (name, img) in &mut d.images {
        let entry = &v["images"][name];
        img.bytes = Arc::new(codec::blob(&blobs, codec::decode(&entry[0])?)?.to_vec());
        let proxy: Option<usize> = codec::decode(&entry[1])?;
        img.proxy = proxy.map(|i| codec::blob(&blobs, i).map(|b| Arc::new(b.to_vec()))).transpose()?;
    }
    d.mask_edit = codec::decode(&v["maskEdit"])?;
    d.puppet = codec::decode(&v["puppet"])?;
    if let Some(pins) = &d.puppet {
        let mut restored = (**pins).clone();
        restored.result = restored.ids.iter().filter_map(|id| d.node_arc(*id).cloned()).collect();
        d.puppet = Some(Arc::new(restored));
    }
    d.fix_next_id();
    d.tidy_saved_selections();
    Ok(d)
}
