//! SVG Options → Optimize: the exported markup rewritten by oxvg, a Rust port of SVGO, with the
//! jobs that would undo another option left out (see [`ExportOptions::optimize`]).

use oxvg_ast::parse::roxmltree::parse;
use oxvg_ast::serialize::Node as _;
use oxvg_ast::visitor::Info;
use oxvg_optimiser::Jobs;

use crate::{ExportOptions, ObjectIds, Profile, Styling};

/// oxvg's number clean-up takes at most this many decimal places.
const MAX_PRECISION: u8 = 5;

/// `svg` (the whole file, UTF-8) optimized as `opts` ask, on one line, without an XML
/// declaration. The error names what oxvg refused.
pub(crate) fn optimize(svg: &str, opts: &ExportOptions) -> Result<String, String> {
    let jobs = jobs(opts);
    let run = || -> Result<String, String> {
        parse(svg, |dom, allocator| {
            let info = Info::new(allocator);
            jobs.run(dom, &info).map_err(|e| e.to_string())?;
            dom.serialize().map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string())?
    };
    // oxvg is third-party code: a panic in it must not take the document down (an export must
    // always write something; wasm aborts on any panic regardless).
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).unwrap_or_else(|_| Err("internal error in the optimiser".into()))
}

/// oxvg's default jobs, minus those that would undo another SVG option, with the precision of
/// the Decimal option.
fn jobs(o: &ExportOptions) -> Jobs {
    // The writer decides the XML declaration (Minify, the encoding) and the DOCTYPE (Style
    // Entities, which skip optimization altogether).
    let mut j = Jobs { remove_x_m_l_proc_inst: None, remove_doctype: None, ..Jobs::default() };
    // Numbers keep the Decimal option's places (within what oxvg allows).
    let p = o.decimals.min(MAX_PRECISION);
    if let Some(c) = &mut j.cleanup_numeric_values {
        c.float_precision = p;
    }
    if let Some(c) = &mut j.convert_path_data {
        c.tolerance.precision = i32::from(p);
    }
    if let Some(c) = &mut j.convert_shape_to_path {
        c.float_precision = i32::from(p);
    }
    if let Some(c) = &mut j.apply_transforms {
        c.transform_precision = Some(f64::from(p));
    }
    // Object IDs: Layer Names and Unique keep every id as written (readable names, or prefixes
    // that keep inlined SVGs apart); Minimal lets the referenced ones shrink. Named layers stay
    // even when empty.
    match o.object_ids {
        ObjectIds::Minimal => {}
        ObjectIds::LayerNames | ObjectIds::Unique => j.cleanup_ids = None,
    }
    if o.object_ids == ObjectIds::LayerNames {
        j.remove_empty_containers = None;
    }
    // `<metadata>` holds File Info (Include Metadata) or the editing data.
    if o.metadata || o.preserve_editing {
        j.remove_metadata = None;
    }
    // Internal CSS stays a style sheet (oxvg would move single-use classes into attributes).
    if o.styling == Styling::InternalCss {
        j.inline_styles = None;
    }
    // SVG Tiny 1.2 is named by its `version` attribute, deprecated in SVG 2.
    if o.profile == Profile::Tiny12 {
        j.remove_deprecated_attrs = None;
    }
    // Hidden layers and objects are written not displayed on purpose.
    if o.hidden_layers {
        j.remove_hidden_elems = None;
    }
    // Flatten: merge paths of one appearance and dissolve groups (objects lose their identity).
    if !o.flatten {
        j.merge_paths = None;
        j.collapse_groups = None;
        j.move_group_attrs_to_elems = None;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimizes_markup_and_reports_bad_xml() {
        let svg = r##"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10">
  <path d="M0 0 L10 0 L10 10 L0 10 Z" fill="#ff0000" opacity="0.50"/>
</svg>"##;
        let out = optimize(svg, &ExportOptions::default()).unwrap();
        assert!(!out.contains("xlink") && !out.contains('\n') && !out.contains("<?xml"), "{out}");
        assert!(out.contains("fill=\"red\"") && out.contains("opacity=\".5\""), "{out}");
        assert!(optimize("<svg", &ExportOptions::default()).is_err());
    }
}
