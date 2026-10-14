//! `.vectorcraft` format: round trips, robustness against garbage, versioning, unknown fields.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use serde_json::{Value, json};
use vectorcraft_doc::{Document, ImageBlob, ImageObject, LineRef, MathDrawStyle, MathSpec, Node, NodeId, NodeKind};
use vectorcraft_format::{SaveOptions, base64_decode, base64_encode, load, load_file, pdf_content, preview, save, save_with, sniff};
use vectorcraft_geom::{Affine, Rect, shapes};
use vectorcraft_testkit::fixtures;
use vectorcraft_testkit::invariants::{check_document, check_native_roundtrip, check_native_roundtrip_exact, doc_json, json_approx_eq};
use vectorcraft_testkit::strategies::arb_ops;

fn rich_doc() -> Document {
    (*fixtures::rich_session().doc().unwrap().doc).clone()
}

/// A document whose first layer holds an equation group (a rectangle standing for its outlines) with `spec`.
fn doc_with_math(spec: MathSpec) -> (Document, NodeId) {
    let mut d = Document::new(200.0, 100.0);
    let l = d.layers[0].id;
    let (gid, pid) = (d.alloc_id(), d.alloc_id());
    let art = Node::path(pid, shapes::rectangle(spec.bounds), vectorcraft_doc::Appearance::default_art());
    let mut g = Node::group(gid, vec![std::sync::Arc::new(art)]);
    g.extras_mut().math = Some(Box::new(spec));
    d.insert(Some(l), 0, g).unwrap();
    (d, gid)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn base64_roundtrip(data in prop::collection::vec(any::<u8>(), 0..300)) {
        let s = base64_encode(&data);
        prop_assert_eq!(s.len() % 4, 0);
        prop_assert_eq!(base64_decode(&s), Some(data.clone()));
        // Whitespace is ignored.
        let spaced: String = s.chars().flat_map(|c| [c, '\n']).collect();
        prop_assert_eq!(base64_decode(&spaced), Some(data));
    }

    /// An equation's spec (any source text, size, style and bounds) survives save → load, bit for bit.
    #[test]
    fn math_specs_roundtrip(
        source in ".{0,80}",
        size in 0.5f64..2000.0,
        display in any::<bool>(),
        (x, y, w, h) in (-1e5f64..1e5, -1e5f64..1e5, 0.0f64..1e4, 0.0f64..1e4),
    ) {
        let spec = MathSpec { source, size, display, bounds: Rect::new(x, y, x + w, y + h) };
        let (d, id) = doc_with_math(spec.clone());
        check_native_roundtrip_exact(&d).map_err(TestCaseError::fail)?;
        for compress in [false, true] {
            let back = load(&save(&d, compress)).unwrap();
            prop_assert_eq!(back.node(id).unwrap().extras().math.as_deref(), Some(&spec));
        }
    }

    /// A document's math drawing style (any valid values) survives save → load in every version,
    /// bit for bit.
    #[test]
    fn math_draw_styles_roundtrip(
        widths in prop::array::uniform5(0.01f64..1000.0),
        (width, dash, gap) in (0.01f64..1000.0, 0.0f64..10_000.0, 0.01f64..10_000.0),
        (dot, hit, size, ratio, std) in (0.1f64..1000.0, 0.0f64..1000.0, 0.5f64..2000.0, 0.01f64..100.0, 0.01f64..100_000.0),
        offset in (-1e5f64..1e5, -1e5f64..1e5),
        (italic, arrow, line) in (any::<bool>(), 1u8..=3, 1u8..=5),
    ) {
        let mut d = Document::new(200.0, 100.0);
        let mut style = MathDrawStyle { solid: widths, dot_size: dot, hit_tolerance: hit, label_size: size, arrow_type: arrow, arrow_ratio: ratio, std_length: std, ..MathDrawStyle::default() };
        style.dashed[line as usize - 1] = vectorcraft_doc::mathdraw::DashedLine { width, dash, gap };
        style.label_o = vectorcraft_doc::mathdraw::AxisLabel { offset, italic };
        style.axis_line = LineRef::Dashed(line);
        style.dot_border = LineRef::Solid(line);
        d.math_draw = style.clone();
        check_native_roundtrip_exact(&d).map_err(TestCaseError::fail)?;
        for version in 1..=vectorcraft_format::VERSION {
            for compress in [false, true] {
                let Ok(bytes) = save_with(&d, &SaveOptions { version, compress, ..SaveOptions::default() }) else { continue };
                prop_assert_eq!(&load(&bytes).unwrap().math_draw, &style, "version {} compress {}", version, compress);
            }
        }
    }

    #[test]
    fn base64_decode_never_panics(s in ".{0,64}") {
        let _ = base64_decode(&s);
    }

    /// Arbitrary bytes are rejected (or, if they happen to parse, yield a valid document) — never a panic.
    #[test]
    fn load_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        if let Ok(d) = load(&bytes) {
            check_document(&d).map_err(TestCaseError::fail)?;
        }
    }

    /// Mutating a valid file (byte flips / truncation) never panics the loader.
    #[test]
    fn load_mutated_file_never_panics(cut in 0usize..4000, flips in prop::collection::vec((0usize..4000, any::<u8>()), 0..8)) {
        let mut bytes = save(&Document::new(100.0, 100.0), false);
        for (i, b) in flips {
            let n = bytes.len();
            bytes[i % n] = b;
        }
        bytes.truncate(cut.min(bytes.len()));
        let _ = load(&bytes);
    }

    /// Documents produced by random editing survive save → load (numbers to 1e-12).
    #[test]
    fn random_documents_roundtrip(ops in arb_ops(5..40)) {
        let mut s = fixtures::session();
        for op in &ops {
            let _ = op.apply(&mut s);
        }
        let d = s.doc().unwrap().doc.clone();
        check_native_roundtrip(&d).map_err(TestCaseError::fail)?;
        // Pretty and compact encodings load to the same document.
        let a = doc_json(&load(&save(&d, true)).unwrap());
        let b = doc_json(&load(&save(&d, false)).unwrap());
        prop_assert_eq!(a, b);
    }

    /// Embedded image bytes survive the base64 round trip exactly (saves keep the images in use).
    #[test]
    fn images_roundtrip(data in prop::collection::vec(any::<u8>(), 0..2000), key in "[a-z \"\\\\]{1,8}") {
        let mut d = Document::new(10.0, 10.0);
        d.images.insert(key.clone(), ImageBlob::new("image/png", data.clone()));
        let layer = d.layers[0].id;
        let id = d.alloc_id();
        let im = ImageObject { key: key.clone(), width: 1, height: 1, xf: Affine::IDENTITY, link: None, placement: Default::default() };
        d.insert(Some(layer), 0, Node::new(id, NodeKind::Image(Box::new(im)))).unwrap();
        for pretty in [false, true] {
            let back = load(&save(&d, pretty)).unwrap();
            prop_assert_eq!(back.images.get(&key).map(|b| b.bytes.as_ref().clone()), Some(data.clone()));
        }
    }

    /// Compressed and older-version saves of random documents load to the same document.
    #[test]
    fn compressed_and_older_saves_roundtrip(ops in arb_ops(5..30)) {
        let mut s = fixtures::session();
        for op in &ops {
            let _ = op.apply(&mut s);
        }
        let d = s.doc().unwrap().doc.clone();
        let want = doc_json(&load(&save(&d, false)).unwrap());
        for o in [SaveOptions { compress: true, ..SaveOptions::default() }, SaveOptions { version: 2, ..SaveOptions::default() }, SaveOptions { version: 1, pretty: true, ..SaveOptions::default() }] {
            let back = load(&save_with(&d, &o).unwrap()).unwrap();
            prop_assert!(json_approx_eq(&doc_json(&back), &want, 1e-12), "{:?} differs", o);
        }
    }

    /// Mutating a compressed file (byte flips / truncation) never panics the loader or the sniffer.
    #[test]
    fn load_mutated_compressed_file_never_panics(cut in 0usize..2000, flips in prop::collection::vec((0usize..2000, any::<u8>()), 0..8)) {
        let mut bytes = save_with(&Document::new(100.0, 100.0), &SaveOptions { compress: true, ..SaveOptions::default() }).unwrap();
        for (i, b) in flips {
            let n = bytes.len();
            bytes[i % n] = b;
        }
        bytes.truncate(cut.min(bytes.len()));
        let _ = sniff(&bytes);
        let _ = preview(&bytes);
        if let Ok(d) = load(&bytes) {
            check_document(&d).map_err(TestCaseError::fail)?;
        }
    }

    /// Mutating a file carrying ICC profiles and a PDF (save options) never panics the readers.
    #[test]
    fn load_mutated_file_with_profiles_never_panics(cut in 0usize..3000, flips in prop::collection::vec((0usize..3000, any::<u8>()), 0..8)) {
        let o = SaveOptions {
            profiles: [("Studio RGB".to_string(), vec![1, 2, 3]), ("Press".to_string(), vec![])].into(),
            pdf: Some(b"%PDF-1.7".to_vec()),
            ..SaveOptions::default()
        };
        let mut bytes = save_with(&Document::new(100.0, 100.0), &o).unwrap();
        for (i, b) in flips {
            let n = bytes.len();
            bytes[i % n] = b;
        }
        bytes.truncate(cut.min(bytes.len()));
        let _ = pdf_content(&bytes);
        if let Ok(f) = load_file(&bytes) {
            check_document(&f.doc).map_err(TestCaseError::fail)?;
        }
    }
}

#[test]
fn rich_document_roundtrips() {
    let d = rich_doc();
    check_native_roundtrip(&d).unwrap();
    let bytes = save(&d, true);
    assert!(sniff(&bytes));
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["format"], "vectorcraft");
    assert_eq!(v["version"], vectorcraft_format::VERSION);
}

/// An equation is written as the `math` field of its group, in every version that keeps the node's
/// fields; files without it (every file before equations) read as plain groups, and an old file's
/// partial spec reads with the defaults for what it lacks.
#[test]
fn equation_spec_is_a_node_field_and_old_files_read() {
    let spec = MathSpec { source: r"\frac{a}{b}".into(), size: 18.0, display: false, bounds: Rect::new(1.0, 2.0, 31.0, 22.0) };
    let (d, id) = doc_with_math(spec.clone());
    for version in [3, 4] {
        let bytes = save_with(&d, &SaveOptions { version, ..SaveOptions::default() }).unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        let mut found = None;
        fn find(v: &Value, id: u64, out: &mut Option<Value>) {
            match v {
                Value::Object(m) => {
                    if m.get("id") == Some(&json!(id)) && m.contains_key("math") {
                        *out = m.get("math").cloned();
                    }
                    m.values().for_each(|c| find(c, id, out));
                }
                Value::Array(a) => a.iter().for_each(|c| find(c, id, out)),
                _ => {}
            }
        }
        find(&v, id.0, &mut found);
        let m = found.expect("the group has a math field");
        assert_eq!(m["source"], json!(r"\frac{a}{b}"));
        assert_eq!((m["size"].as_f64(), m["display"].as_bool()), (Some(18.0), Some(false)));
        assert!(m.get("bounds").is_some());
        assert_eq!(load(&bytes).unwrap().node(id).unwrap().extras().math.as_deref(), Some(&spec));
    }
    // A group with no equation writes no field, and its extras stay unallocated.
    let (plain, pid) = doc_with_math(spec);
    let mut plain = plain;
    plain.node_mut(pid).unwrap().extras_mut().math = None;
    assert!(!String::from_utf8_lossy(&save(&plain, false)).contains("\"math\""));
    assert!(load(&save(&plain, false)).unwrap().node(pid).unwrap().extras().math.is_none());
    // A spec from a file with fields missing (or an unknown one) still reads.
    let (mut d, id) = doc_with_math(MathSpec::default());
    d.node_mut(id).unwrap().extras_mut().math = Some(Box::new(MathSpec { source: "x".into(), ..MathSpec::default() }));
    let mut v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                if let Some(Value::Object(spec)) = m.get_mut("math") {
                    spec.retain(|k, _| k == "source");
                    spec.insert("future".into(), json!(1));
                }
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut v);
    let back = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    let got = back.node(id).unwrap().extras().math.as_deref().cloned().unwrap();
    assert_eq!((got.source.as_str(), got.size, got.display), ("x", MathSpec::default().size, MathSpec::default().display));
}

/// The math drawing style is the document's `math_draw` field, written only when it isn't the
/// default (so files without it, every file before it, read as the default), in v3 and v4 alike;
/// a value out of range in a file reads as its default rather than failing the file.
#[test]
fn math_draw_style_is_a_document_field_and_old_files_read() {
    let mut d = Document::new(200.0, 100.0);
    assert!(!String::from_utf8_lossy(&save(&d, false)).contains("math_draw"));
    d.math_draw.patch(&json!({"stdLength": 120, "axisLine": "solid1", "labelX": {"italic": false}})).unwrap();
    for version in [3, 4] {
        let bytes = save_with(&d, &SaveOptions { version, ..SaveOptions::default() }).unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        let m = &v["document"]["math_draw"];
        assert_eq!((m["stdLength"].as_f64(), m["axisLine"].as_str(), m["labelX"]["italic"].as_bool()), (Some(120.0), Some("solid1"), Some(false)));
        assert_eq!(load(&bytes).unwrap().math_draw, d.math_draw);
    }
    let mut v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    v["document"]["math_draw"]["dotSize"] = json!(-4);
    v["document"]["math_draw"]["future"] = json!({"a": 1});
    let back = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!((back.math_draw.dot_size, back.math_draw.std_length), (MathDrawStyle::default().dot_size, 120.0));
    v["document"].as_object_mut().unwrap().remove("math_draw");
    assert!(load(&serde_json::to_vec(&v).unwrap()).unwrap().math_draw.is_default());
}

#[test]
fn sniff_distinguishes_formats() {
    assert!(sniff(&save(&Document::new(1.0, 1.0), false)));
    assert!(!sniff(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"));
    assert!(!sniff(b"%PDF-1.7"));
    assert!(!sniff(b""));
    assert!(!sniff(&[0xff, 0xfe, 0x00]));
}

#[test]
fn rejects_newer_versions_and_other_formats() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(1.0, 1.0), false)).unwrap();
    v["version"] = json!(vectorcraft_format::VERSION + 1);
    assert!(matches!(load(&serde_json::to_vec(&v).unwrap()), Err(vectorcraft_format::FormatError::TooNew(_))));
    v["version"] = json!(vectorcraft_format::VERSION);
    v["format"] = json!("other");
    assert!(matches!(load(&serde_json::to_vec(&v).unwrap()), Err(vectorcraft_format::FormatError::NotVectorcraft(_))));
    v["format"] = json!("vectorcraft");
    v["images"] = json!({"x": {"mime": "image/png", "data": "not base64!"}});
    assert!(matches!(load(&serde_json::to_vec(&v).unwrap()), Err(vectorcraft_format::FormatError::BadImage(_))));
}

#[test]
fn unknown_fields_are_preserved() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(1.0, 1.0), false)).unwrap();
    v["document"]["unknown"] = json!({"futureFeature": {"a": [1, 2, 3]}});
    let d = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    let again: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    assert_eq!(again["document"]["unknown"]["futureFeature"]["a"], json!([1, 2, 3]));
}

#[test]
fn load_repairs_next_id() {
    let d = rich_doc();
    let mut v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    v["document"]["next_id"] = json!(1);
    let mut back = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    check_document(&back).unwrap();
    let fresh = back.alloc_id();
    assert!(back.node(fresh).is_none(), "allocated id {fresh} collides");
    let max = fixtures::all_ids(&back).into_iter().map(|NodeId(i)| i).max().unwrap();
    assert!(fresh.0 > max);
}

#[test]
fn approx_equal_documents_compare_equal() {
    let d = rich_doc();
    assert!(json_approx_eq(&doc_json(&d), &doc_json(&d), 0.0));
}

#[test]
fn bug_f64_bit_exact_roundtrip() {
    check_native_roundtrip_exact(&rich_doc()).unwrap();
    let mut d = Document::new(126.86291501015239, 214.69463130731182);
    d.raster_effects_ppi = 0.1 + 0.2;
    check_native_roundtrip_exact(&d).unwrap();
}

/// Saved selections come from the file: older files have none, and a hostile one is cut down to
/// what the Select menu can list, with names that fit and ids the document has.
#[test]
fn saved_selections_from_a_file_are_tidied() {
    let d = rich_doc();
    let mut v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    assert!(v["document"].get("saved_selections").is_none(), "nothing written when there are none");
    assert!(load(&serde_json::to_vec(&v).unwrap()).unwrap().saved_selections.is_empty());
    let id = fixtures::all_ids(&d).into_iter().next().unwrap();
    let missing = fixtures::all_ids(&d).into_iter().map(|NodeId(i)| i).max().unwrap() + 1;
    let mut list = vec![
        json!({"name": format!("  {}  ", "x".repeat(1000)), "objects": [id.0, missing, id.0]}),
        json!({"name": "   ", "objects": [id.0]}),
        json!({"name": "Dup", "objects": []}),
        json!({"name": "Dup", "objects": [id.0]}),
    ];
    list.extend((0..100).map(|i| json!({"name": format!("S{i}"), "objects": [id.0]})));
    v["document"]["saved_selections"] = json!(list);
    let back = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    let saved = &back.saved_selections;
    assert_eq!(saved.len(), vectorcraft_doc::SavedSelection::MAX);
    assert_eq!(saved[0].name, "x".repeat(vectorcraft_doc::SavedSelection::MAX_NAME));
    assert_eq!(saved[0].objects, vec![id], "a missing id is dropped, a repeated one kept once");
    assert_eq!((saved[1].name.as_str(), saved[1].objects.len()), ("Dup", 0), "blank names go, the first of a name stays");
    assert_eq!(saved[2].name, "S0");
    // What loads saves and loads again unchanged.
    assert_eq!(&load(&save(&back, false)).unwrap().saved_selections, saved);
    // Junk in the member is a format error, not a crash.
    v["document"]["saved_selections"] = json!([{"name": 5, "objects": "x"}]);
    assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
}
