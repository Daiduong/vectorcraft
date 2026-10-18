//! Import bounded, self-contained AI SVG without linked resources or embedded native data.
use super::*;

pub const MAX_SVG: usize = 512 * 1024;
pub fn validate(svg: &str) -> std::result::Result<(), String> {
    if svg.len() > MAX_SVG {
        return Err("SVG melebihi 512 KiB".into());
    }
    let xml = roxmltree::Document::parse(svg).map_err(|e| format!("SVG tidak valid: {e}"))?;
    if xml.root_element().tag_name().name() != "svg" {
        return Err("Respons bukan SVG".into());
    }
    let allowed = [
        "svg",
        "g",
        "defs",
        "path",
        "rect",
        "circle",
        "ellipse",
        "line",
        "polyline",
        "polygon",
        "text",
        "tspan",
        "title",
        "desc",
        "linearGradient",
        "radialGradient",
        "stop",
        "clipPath",
    ];
    for (i, n) in xml.descendants().enumerate() {
        if i > 5000 || n.ancestors().count() > 32 {
            return Err("SVG terlalu kompleks".into());
        }
        if !n.is_element() {
            continue;
        }
        if !allowed.contains(&n.tag_name().name()) {
            return Err(format!("Elemen SVG tidak didukung: {}", n.tag_name().name()));
        }
        for a in n.attributes() {
            let name = a.name().to_ascii_lowercase();
            let value = a.value().to_ascii_lowercase();
            if name.starts_with("on")
                || name == "href"
                || (name.starts_with("data-") && name != "data-name")
                || name == "base"
                || value.contains('@')
                || value.contains("javascript:")
                || value.contains("http:")
                || value.contains("https:")
                || value.contains("file:")
                || value.contains("data:")
            {
                return Err("SVG harus mandiri tanpa skrip, tautan, atau data tertanam".into());
            }
            for part in value.split("url(").skip(1) {
                let Some((reference, _)) = part.split_once(')') else {
                    return Err("Referensi SVG tidak valid".into());
                };
                if !reference.trim().trim_matches(['\'', '"']).starts_with('#') {
                    return Err("Referensi eksternal tidak diizinkan".into());
                }
            }
        }
    }
    Ok(())
}
fn open(s: &mut Session, p: &Value) -> Result<Value> {
    let svg = p.get("svg").and_then(Value::as_str).ok_or_else(|| bad("ai.openSvg", "svg wajib diisi"))?;
    validate(svg).map_err(|e| bad("ai.openSvg", e))?;
    let result = fileio::open_bytes(s, "AI Illustration.svg", svg.as_bytes(), None)?;
    if let Some(st) = s.active_mut() {
        st.doc = std::sync::Arc::new((*st.doc).clone());
    }
    Ok(result)
}
pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!("ai.openSvg", "Open AI illustration", [], None, "{svg: string} → new unsaved editable document", always, open)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_resources_and_preserves_session() {
        for svg in [
            "<svg><script/></svg>",
            "<svg><image href='file:///tmp/a'/></svg>",
            "<svg><rect fill='url(https://example.com/a)'/></svg>",
            "<svg><metadata/></svg>",
        ] {
            let mut s = Session::new();
            assert!(open(&mut s, &serde_json::json!({"svg":svg})).is_err());
            assert!(s.active().is_none());
        }
    }
    #[test]
    fn valid_svg_is_editable_and_unsaved() {
        let mut s = Session::new();
        open(&mut s,&serde_json::json!({"svg":"<svg xmlns='http://www.w3.org/2000/svg' width='64' height='64'><rect width='20' height='20' fill='red'/></svg>"})).unwrap();
        assert!(s.active().unwrap().is_dirty());
        assert!(s.active().unwrap().path.is_none());
    }
}
