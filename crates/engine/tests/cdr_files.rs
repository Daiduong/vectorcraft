//! Local, original CorelDRAW-authored oracles. No proprietary artwork, fonts or profiles are
//! committed. Set CDR_FIXTURE_DIR as described in docs/cdr-import.md to run these checks.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

use vectorcraft_engine::cmd::fileio;
use vectorcraft_render::Renderer;

#[test]
/// Compare every page of optional original CorelDRAW fixtures with independently exported SVGs.
fn original_corel_files_match_their_independent_svg_exports() {
    let Ok(folder) = std::env::var("CDR_FIXTURE_DIR") else { return };
    let folder = Path::new(&folder);
    let mut cases = Vec::new();
    for stem in ["rectangle", "geometry"] {
        for version in [15, 16, 27] {
            cases.push((stem, version, if stem == "rectangle" { 1 } else { 2 }));
        }
    }
    for stem in ["gradient", "group-transform"] {
        if folder.join(format!("{stem}-v27.cdr")).exists() {
            cases.push((stem, 27, 1));
        }
    }
    for (stem, version, expected_pages) in cases {
        let name = format!("{stem}-v{version}.cdr");
        let bytes = std::fs::read(folder.join(&name)).unwrap();
        let loaded = fileio::load(&name, &bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(loaded.format.id, "cdr");
        assert!(!loaded.preview_only);
        assert!(loaded.doc.images.is_empty());
        assert_eq!(loaded.doc.artboards.len(), expected_pages, "{name}");
        for (page, ab) in loaded.doc.artboards.iter().enumerate() {
            let oracle_name = format!("{stem}-page{}.svg", page + 1);
            let oracle = std::fs::read_to_string(folder.join(&oracle_name)).unwrap();
            let reference = vectorcraft_svg::import(&oracle).unwrap();
            let frame = reference.artboards[0].rect;
            assert!((ab.rect.width() - frame.width()).abs() < 0.01, "{name} page width");
            assert!((ab.rect.height() - frame.height()).abs() < 0.01, "{name} page height");
            let actual = Renderer::new().render_region(&loaded.doc, ab.rect, 1.0, true);
            let expected = Renderer::new().render_region(&reference, frame, 1.0, true);
            assert_eq!((actual.width, actual.height), (expected.width, expected.height));
            let mean = actual.pixels.iter().zip(&expected.pixels).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / actual.pixels.len() as f64;
            eprintln!("{name} page {}: mean byte difference {mean:.6}; warnings={:?}", page + 1, loaded.warnings);
            if let Ok(output) = std::env::var("CDR_RENDER_DIR") {
                std::fs::create_dir_all(&output).unwrap();
                std::fs::write(Path::new(&output).join(format!("{stem}-v{version}-page{}.png", page + 1)), actual.to_png().unwrap()).unwrap();
                std::fs::write(Path::new(&output).join(format!("{stem}-reference-page{}.png", page + 1)), expected.to_png().unwrap()).unwrap();
            }
            assert!(mean < 0.1, "{name} page {} differs from the CorelDRAW SVG oracle: {mean:.6}", page + 1);
        }
    }
}
