#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
use serde_json::json;
use vectorcraft_doc::Document;
use vectorcraft_engine::Session;
#[test]
fn craft_session_tabs_history_and_saved_identity_round_trip() {
    let mut s = Session::new();
    s.add_document(Document::new(100., 100.), None);
    s.execute("shape.rectangle", &json!({"x":10,"y":10,"width":20,"height":20})).unwrap();
    s.execute("shape.rectangle", &json!({"x":40,"y":40,"width":10,"height":10})).unwrap();
    s.execute("edit.undo", &json!({})).unwrap();
    s.add_document(Document::new(200., 200.), None);
    s.set_active(0);
    let bytes = s.craft_capture(json!({"zoom":3})).unwrap();
    let mut r = Session::new();
    assert_eq!(r.craft_restore(&bytes).unwrap(), json!({"zoom":3}));
    assert_eq!(r.documents().len(), 2);
    assert_eq!(r.active_index(), Some(0));
    assert_eq!(r.active().unwrap().history.undo.len(), 1);
    assert_eq!(r.active().unwrap().history.redo.len(), 1);
    r.execute("edit.redo", &json!({})).unwrap();
    r.execute("edit.undo", &json!({})).unwrap();
    r.execute("edit.undo", &json!({})).unwrap();
    assert!(!r.active().unwrap().is_dirty());
    assert!(r.craft_restore(&bytes[..20]).is_err());
    assert_eq!(r.documents().len(), 2);
}

#[test]
fn recovery_inspection_rejects_wrong_app_and_malformed_envelopes() {
    for bytes in [b"".as_slice(), b"CRAFTS01", b"CRAFTS01\xff\xff\xff\xff"] {
        assert!(Session::craft_ui(bytes).is_err());
    }
    let meta = br#"{"app":"unrelated","schema":1,"ui":{}}"#;
    let mut bytes = b"CRAFTS01".to_vec();
    bytes.extend_from_slice(&(meta.len() as u32).to_le_bytes());
    bytes.extend_from_slice(meta);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    assert!(Session::craft_ui(&bytes).is_err());
}
