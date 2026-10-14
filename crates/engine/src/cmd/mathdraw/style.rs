//! The math drawing style: `mathdraw.style` reads it, `mathdraw.setStyle` sets part of it.

use serde_json::Value;

use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "mathdraw.style",
            "Math Drawing Style",
            [],
            None,
            "{} → {solid: [5 weights, pt], dashed: [5 × {width, dash, gap}], dotSize, dotBorder: line, hitTolerance, labelX, labelY, labelO: {offset: [x, y] pt at labelSize 73.33, y up, italic: bool}, labelSize, arrowType: 1–3, arrowRatio, stdLength, axisLine: line}; the document's math drawing style (a line is \"solid1\"…\"solid5\" or \"dashed1\"…\"dashed5\")",
            has_doc,
            get
        ),
        cmd!(
            "mathdraw.setStyle",
            "Set Math Drawing Style",
            [],
            None,
            "{solid?: [5 weights in pt, null keeps one], dashed?: [5 × {width?, dash?, gap?} or null], dotSize?: pt, dotBorder?: line, hitTolerance?: pt, labelX?|labelY?|labelO?: {offset?: [x, y], italic?: bool}, labelSize?: pt, arrowType?: 1–3, arrowRatio?, stdLength?: pt, axisLine?: line} → the whole style, as mathdraw.style gives it; sets the keys given and keeps the rest, all or nothing (an unknown key, a value of the wrong type or out of range is refused)",
            has_doc,
            set
        ),
    ]
}

fn style_json(s: &Session) -> Result<Value> {
    serde_json::to_value(&s.doc()?.doc.math_draw).map_err(|e| EngineError::Other(e.to_string()))
}

fn get(s: &mut Session, _: &Value) -> Result<Value> {
    style_json(s)
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "mathdraw.setStyle";
    let mut next = s.doc()?.doc.math_draw.clone();
    next.patch(p).map_err(|e| bad(C, e.to_string()))?;
    if next != s.doc()?.doc.math_draw {
        s.edit("Math Drawing Style", |d, _| {
            d.math_draw = next;
            Ok(())
        })?;
    }
    style_json(s)
}
