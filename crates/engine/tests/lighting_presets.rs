//! Preset dispatch without the engine unit tests' extra parameter audit.
#![allow(clippy::unwrap_used, clippy::panic)]

use serde_json::json;
use vectorcraft_engine::{EngineError, Session};

#[test]
fn lighting_presets_reject_extra_fields_without_changing_document_or_history() {
    for effect in ["threeD.revolve", "threeD.extrude", "threeD.inflate", "threeD.rotate"] {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width":400,"height":400})).unwrap();
        let id = s.execute("path.create", &json!({"d":"M100 100L200 100L200 200L100 200Z"})).unwrap()["id"].clone();
        s.execute("paint.setFill", &json!({"color":"#338ad6"})).unwrap();
        s.execute("effect.apply", &json!({"effect":effect,"item":null,"params":{"shade":false,"rotationX":13}})).unwrap();
        s.execute("paint.setFill", &json!({"color":"#ff8040"})).unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        s.doc_mut().unwrap().mark_saved();
        let before = s.doc().unwrap().doc.clone();
        let selection = s.doc().unwrap().selection.clone();
        let undo = s.doc().unwrap().history.undo.len();
        let redo = s.doc().unwrap().history.redo.len();
        assert!(redo > 0);

        for (key, value) in [
            ("visible", json!(false)),
            ("warpStyle", json!("arc")),
            ("params", json!({"ambient":200})),
            ("shade", json!(true)),
            ("ambient", json!(200)),
            ("unknown", json!(null)),
        ] {
            let mut params = json!({"index":0,"item":null,"preset":"right"});
            params[key] = value;
            let err = s.execute("effect.lightingPreset", &params).unwrap_err();
            assert!(
                matches!(err, EngineError::BadParams { ref cmd, ref msg }
                if cmd == "effect.lightingPreset" && msg.contains(&format!("`{key}`"))),
                "{err}"
            );
            let state = s.doc().unwrap();
            assert_eq!(state.doc, before, "{effect}, {key}");
            assert_eq!(state.selection, selection);
            assert_eq!(state.history.undo.len(), undo);
            assert_eq!(state.history.redo.len(), redo);
            assert!(!state.is_dirty());
            assert!(!s.in_interaction());
        }

        s.execute("effect.lightingPreset", &json!({"id":id,"index":0,"item":null,"target":"object","preset":"right"})).unwrap();
        let after = s.doc().unwrap().doc.clone();
        let node = after.node(vectorcraft_doc::NodeId(id.as_u64().unwrap())).unwrap();
        assert!(node.appearance.effects[0].visible);
        assert_eq!(node.appearance.effects[0].params["shade"], false);
        assert_eq!(node.appearance.effects[0].params["rotationX"], 13);
        assert_eq!(node.appearance.effects[0].params["lightRotation"], 0.0);
        assert_eq!(s.doc().unwrap().history.undo.len(), undo + 1);
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc, before);
        assert!(!s.doc().unwrap().is_dirty());
        s.execute("edit.redo", &json!({})).unwrap();
        assert_eq!(s.doc().unwrap().doc, after);
    }
}
