//! The math drawing style: its shipped defaults, lines, patches and lenient reading.

use super::mathdraw::*;
use super::{Document, LineRef, MathDrawStyle};
use serde_json::json;

/// The original tool's presets, value for value.
#[test]
fn the_defaults_are_the_original_presets() {
    let s = MathDrawStyle::default();
    assert_eq!(s.solid, [6.0, 2.5, 2.0, 1.0, 0.5]);
    let dashed: Vec<(f64, f64, f64)> = s.dashed.iter().map(|d| (d.width, d.dash, d.gap)).collect();
    assert_eq!(dashed, vec![(2.0, 12.0, 6.0), (4.8, 18.0, 9.0), (1.0, 6.0, 3.0), (1.0, 4.0, 2.0), (0.5, 4.0, 2.0)]);
    assert_eq!((s.dot_size, s.dot_border, s.hit_tolerance), (14.0, LineRef::Solid(3), 20.0));
    assert_eq!((s.label_x.offset, s.label_x.italic), ((-18.4365234375, -52.3120117188), true));
    assert_eq!((s.label_y.offset, s.label_y.italic), ((-48.0340679688, -7.9995117188), true));
    assert_eq!((s.label_o.offset, s.label_o.italic), ((-57.7846579688, -64.1459960938), false));
    assert_eq!((s.label_size, s.arrow_type, s.arrow_ratio), (73.33, 3, 1.0));
    assert_eq!((s.std_length, s.axis_line), (150.0, LineRef::Solid(2)));
    assert!(s.is_default());
}

#[test]
fn lines_give_their_weight_and_dashes() {
    let s = MathDrawStyle::default();
    assert_eq!(s.stroke(LineRef::Solid(1)), (6.0, vec![]));
    assert_eq!(s.stroke(LineRef::Solid(5)), (0.5, vec![]));
    assert_eq!(s.stroke(s.axis_line), (2.5, vec![]));
    assert_eq!(s.stroke(s.dot_border), (2.0, vec![]));
    assert_eq!(s.stroke(LineRef::Dashed(2)), (4.8, vec![18.0, 9.0]));
    assert_eq!(s.stroke(LineRef::Dashed(3)), (1.0, vec![6.0, 3.0]));
    // Out-of-range indices (only constructible in code) take the nearest preset.
    assert_eq!(s.stroke(LineRef::Solid(0)), (6.0, vec![]));
    assert_eq!(s.stroke(LineRef::Dashed(9)), (0.5, vec![4.0, 2.0]));
    for (name, line) in [("solid1", LineRef::Solid(1)), ("dashed5", LineRef::Dashed(5)), ("Solid3", LineRef::Solid(3))] {
        assert_eq!(LineRef::parse(name), Some(line), "{name}");
    }
    for bad in ["solid0", "solid6", "dashed", "dotted1", "", "solid-1", "solid 1x"] {
        assert_eq!(LineRef::parse(bad), None, "{bad}");
    }
    assert_eq!(LineRef::Dashed(4).name(), "dashed4");
}

#[test]
fn label_offsets_scale_with_the_label_size() {
    let mut s = MathDrawStyle::default();
    assert_eq!(s.label_offset(AxisLabelKind::X), s.label_x.offset);
    s.label_size = LABEL_REFERENCE_SIZE / 2.0;
    let (x, y) = s.label_offset(AxisLabelKind::Origin);
    assert!((x - s.label_o.offset.0 / 2.0).abs() < 1e-12 && (y - s.label_o.offset.1 / 2.0).abs() < 1e-12);
}

#[test]
fn the_json_form_is_camel_case_with_named_lines() {
    let v = serde_json::to_value(MathDrawStyle::default()).unwrap();
    assert_eq!(v["solid"], json!([6.0, 2.5, 2.0, 1.0, 0.5]));
    assert_eq!(v["dashed"][1], json!({"width": 4.8, "dash": 18.0, "gap": 9.0}));
    assert_eq!(v["dotBorder"], json!("solid3"));
    assert_eq!(v["axisLine"], json!("solid2"));
    assert_eq!(v["labelO"], json!({"offset": [-57.7846579688, -64.1459960938], "italic": false}));
    assert_eq!((v["dotSize"].as_f64(), v["hitTolerance"].as_f64(), v["labelSize"].as_f64()), (Some(14.0), Some(20.0), Some(73.33)));
    assert_eq!((v["arrowType"].as_u64(), v["arrowRatio"].as_f64(), v["stdLength"].as_f64()), (Some(3), Some(1.0), Some(150.0)));
    assert_eq!(v.as_object().unwrap().len(), 13);
    assert_eq!(serde_json::from_value::<MathDrawStyle>(v).unwrap(), MathDrawStyle::default());
}

#[test]
fn a_patch_merges_deeply() {
    let mut s = MathDrawStyle::default();
    s.patch(&json!({
        "solid": [null, 3, null, null, 0.25],
        "dashed": [null, {"gap": 4}, null, null, null],
        "labelX": {"italic": false},
        "dotBorder": "dashed4",
        "arrowType": 1,
        "stdLength": 100
    }))
    .unwrap();
    assert_eq!(s.solid, [6.0, 3.0, 2.0, 1.0, 0.25]);
    assert_eq!((s.dashed[1].width, s.dashed[1].dash, s.dashed[1].gap), (4.8, 18.0, 4.0));
    assert_eq!((s.label_x.offset, s.label_x.italic), (MathDrawStyle::default().label_x.offset, false));
    assert_eq!((s.dot_border, s.arrow_type, s.std_length), (LineRef::Dashed(4), 1, 100.0));
    assert!(!s.is_default());
    s.patch(&json!({})).unwrap();
    assert_eq!(s.std_length, 100.0);
}

/// Wrong keys, types and ranges are refused, and leave the style as it was.
#[test]
fn a_bad_patch_changes_nothing() {
    let start = MathDrawStyle::default();
    for bad in [
        json!(null),
        json!([1]),
        json!({"dotsize": 3}),
        json!({"dotSize": "14"}),
        json!({"dotSize": 0}),
        json!({"dotSize": 1e9}),
        json!({"solid": [1, 2, 3]}),
        json!({"solid": [1, 2, 3, 4, -1]}),
        json!({"solid": 2}),
        json!({"dashed": [null, null, null, null, {"dash": 0, "gap": 0}]}),
        json!({"dashed": [null, null, null, null, {"length": 1}]}),
        json!({"dashed": [1, null, null, null, null]}),
        json!({"labelX": {"offset": [1]}}),
        json!({"labelX": {"offset": [1, 1e9]}}),
        json!({"labelY": {"italic": 1}}),
        json!({"labelO": {"size": 3}}),
        json!({"labelO": true}),
        json!({"arrowType": 4}),
        json!({"arrowType": 0}),
        json!({"arrowType": 2.5}),
        json!({"arrowRatio": -1}),
        json!({"labelSize": 0.1}),
        json!({"axisLine": "solid9"}),
        json!({"axisLine": 2}),
        json!({"stdLength": 0}),
        json!({"hitTolerance": -1}),
        json!({"stdLength": 120, "nope": 1}),
    ] {
        let mut s = start.clone();
        let e = s.patch(&bad).expect_err(&bad.to_string());
        assert!(!e.to_string().is_empty());
        assert_eq!(s, start, "{bad} changed the style");
    }
    let mut s = start.clone();
    assert_eq!(s.patch(&json!({"labelO": {"size": 3}})), Err(PatchError::UnknownKey("labelO.size".into())));
}

/// A document's style is saved only when it isn't the default, and a file's style is read
/// leniently: what is out of range or unknown reads as the default.
#[test]
fn documents_save_the_style_only_when_set_and_read_it_leniently() {
    let mut d = Document::new(100.0, 100.0);
    let v = serde_json::to_value(&d).unwrap();
    assert!(v.get("math_draw").is_none());
    d.math_draw.patch(&json!({"dotSize": 10, "axisLine": "dashed1"})).unwrap();
    let v = serde_json::to_value(&d).unwrap();
    assert_eq!(v["math_draw"]["dotSize"], json!(10.0));
    let back: Document = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(back.math_draw, d.math_draw);

    let mut hostile = v;
    hostile["math_draw"] = json!({"dotSize": -5, "axisLine": "dashed2", "solid": "thick", "future": [1], "labelX": {"italic": false, "colour": 1}});
    let back: Document = serde_json::from_value(hostile.clone()).unwrap();
    let want = MathDrawStyle { axis_line: LineRef::Dashed(2), ..MathDrawStyle::default() };
    assert_eq!(back.math_draw, want);
    hostile["math_draw"] = json!("not a style");
    let back: Document = serde_json::from_value(hostile).unwrap();
    assert!(back.math_draw.is_default());
}
