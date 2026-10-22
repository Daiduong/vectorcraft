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
    for version in [15, 16, 27] {
        for stem in ["bitmap", "powerclip", "corners", "corner-types", "polygons", "fill-rules", "bspline"] {
            if folder.join(format!("{stem}-v{version}.cdr")).exists() {
                cases.push((stem, version, 1));
            }
        }
    }
    for (stem, version, expected_pages) in cases {
        let name = format!("{stem}-v{version}.cdr");
        let bytes = std::fs::read(folder.join(&name)).unwrap();
        let loaded = fileio::load(&name, &bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(loaded.format.id, "cdr");
        assert!(!loaded.preview_only);
        if stem == "bitmap" {
            assert_eq!(loaded.doc.images.len(), 2, "both embedded artwork images must be retained");
        } else {
            assert!(loaded.doc.images.is_empty());
        }
        assert_eq!(loaded.doc.artboards.len(), expected_pages, "{name}");
        for (page, ab) in loaded.doc.artboards.iter().enumerate() {
            let oracle_name = format!("{stem}-page{}.svg", page + 1);
            let oracle = std::fs::read_to_string(folder.join(&oracle_name)).unwrap();
            let reference = vectorcraft_svg::import(&oracle).unwrap();
            let frame = reference.artboards[0].rect;
            assert!((ab.rect.width() - frame.width()).abs() < 0.01, "{name} page width");
            assert!((ab.rect.height() - frame.height()).abs() < 0.01, "{name} page height");
            let actual = Renderer::new().render_region(&loaded.doc, ab.rect, 1.0, stem != "bitmap");
            let expected = Renderer::new().render_region(&reference, frame, 1.0, stem != "bitmap");
            assert_eq!((actual.width, actual.height), (expected.width, expected.height));
            let mean = actual.pixels.iter().zip(&expected.pixels).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / actual.pixels.len() as f64;
            eprintln!("{name} page {}: mean byte difference {mean:.6}; warnings={:?}", page + 1, loaded.warnings);
            if let Ok(output) = std::env::var("CDR_RENDER_DIR") {
                std::fs::create_dir_all(&output).unwrap();
                std::fs::write(Path::new(&output).join(format!("{stem}-v{version}-page{}.png", page + 1)), actual.to_png().unwrap()).unwrap();
                std::fs::write(Path::new(&output).join(format!("{stem}-reference-page{}.png", page + 1)), expected.to_png().unwrap()).unwrap();
            }
            if stem == "bitmap" {
                // Corel applies its CMYK profile when exporting PNG. Import keeps continuous
                // alpha and numeric profile-free colour, so compare geometry/alpha separately.
                let alpha_mean = actual
                    .pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(expected.pixels.as_chunks::<4>().0.iter())
                    .map(|(a, b)| a[3].abs_diff(b[3]) as f64)
                    .sum::<f64>()
                    / (actual.pixels.len() / 4) as f64;
                assert!(alpha_mean < 0.1, "{name}: bitmap placement/alpha differs from Corel: {alpha_mean}");
                let actual_images: Vec<_> = loaded.doc.images.values().map(|b| image::load_from_memory(&b.bytes).unwrap().to_rgba8()).collect();
                let expected_images: Vec<_> = reference.images.values().map(|b| image::load_from_memory(&b.bytes).unwrap().to_rgba8()).collect();
                let original_pixels = image::open(folder.join("bitmap-original-checker.png")).unwrap().to_rgba8();
                assert!(actual_images.contains(&original_pixels), "native RGB/alpha must preserve the original generated PNG exactly");
                assert!(
                    actual_images.iter().any(|a| {
                        expected_images.iter().any(|b| {
                            a.dimensions() == b.dimensions()
                                && a.pixels().zip(b.pixels()).all(|(a, b)| {
                                    // Corel rounds straight RGB when exporting partial alpha and
                                    // replaces invisible RGB at alpha zero. Coverage is exact;
                                    // opaque colour is exact; premultiplied partial colour is
                                    // allowed one byte of export quantization.
                                    a[3] == b[3]
                                        && (0..3).all(|c| {
                                            if a[3] == 255 {
                                                a[c] == b[c]
                                            } else {
                                                let alpha = u32::from(a[3]);
                                                let pa = (u32::from(a[c]) * alpha + 127) / 255;
                                                let pb = (u32::from(b[c]) * alpha + 127) / 255;
                                                pa.abs_diff(pb) <= 1
                                            }
                                        })
                                })
                        })
                    }),
                    "RGB bitmap pixels/alpha must match the independent SVG export within export quantization"
                );
                assert!(loaded.warnings.iter().any(|w| w.contains("CMYK bitmap")));
            } else {
                assert!(mean < 0.1, "{name} page {} differs from the CorelDRAW SVG oracle: {mean:.6}", page + 1);
            }
        }
    }
}

#[test]
/// A locally supplied zero-page drawing retains every curve and matches its separate Corel SVG.
/// The user file and derived artwork remain local; only generated fixtures are committed.
fn private_zero_page_drawing_matches_its_local_vector_oracle() {
    let Ok(folder) = std::env::var("CDR_ZERO_PAGE_FIXTURE_DIR") else { return };
    let folder = Path::new(&folder);
    let loaded = fileio::load("zero-page.cdr", &std::fs::read(folder.join("input.cdr")).unwrap()).unwrap();
    let reference = vectorcraft_svg::import(&std::fs::read_to_string(folder.join("corel-artwork.svg")).unwrap()).unwrap();
    /// Count the native editable subpaths without treating group wrappers as art objects.
    fn counts(nodes: &[std::sync::Arc<vectorcraft_doc::Node>]) -> (usize, usize) {
        let mut result = (0, 0);
        for node in nodes {
            if let Some(path) = node.path_data() {
                result.0 += 1;
                result.1 += path.subpaths.len();
            }
            if let Some(children) = node.children() {
                let child = counts(children);
                result.0 += child.0;
                result.1 += child.1;
            }
        }
        result
    }
    assert_eq!(counts(&loaded.doc.layers), (180, 424));
    assert!(loaded.doc.images.is_empty() && !loaded.preview_only);
    assert!(loaded.warnings.iter().any(|warning| warning.contains("artboard was fitted")));
    let frame = loaded.doc.artboards[0].rect;
    let expected_frame = reference.artboards[0].rect;
    assert!((frame.width() - expected_frame.width()).abs() < 0.01);
    assert!((frame.height() - expected_frame.height()).abs() < 0.01);
    let actual = Renderer::new().render_region(&loaded.doc, frame, 2.0, true);
    let expected = Renderer::new().render_region(&reference, expected_frame, 2.0, true);
    assert_eq!((actual.width, actual.height), (expected.width, expected.height));
    let mean = actual.pixels.iter().zip(&expected.pixels).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / actual.pixels.len() as f64;
    // Check geometry/stacking independently as well as the original colours:
    // corresponding paths get distinct RGB paints so equal-coloured overlapping
    // objects cannot hide an internal boundary or paint-order regression.
    /// Assign corresponding paths distinct RGB paints to check internal boundaries and stacking.
    fn geometry_colours(nodes: &mut [std::sync::Arc<vectorcraft_doc::Node>], index: &mut u32) {
        for node in nodes {
            let node = std::sync::Arc::make_mut(node);
            node.opacity = 1.0;
            // SVG import represents a multi-subpath <path> as one painted compound
            // with unpainted members; CDR retains it as one multi-subpath PathData.
            let compound = matches!(node.kind, vectorcraft_doc::NodeKind::Compound { .. });
            if node.path_data().is_some() || compound {
                let n = *index;
                *index += 1;
                let color = vectorcraft_color::Color::rgb8((30 + n * 53 % 200) as u8, (30 + n * 97 % 200) as u8, (30 + n * 131 % 200) as u8);
                node.appearance = vectorcraft_doc::Appearance {
                    items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(color)))],
                    ..Default::default()
                }
                .into();
            }
            if !compound && let Some(children) = node.children_mut() {
                geometry_colours(children, index);
            }
        }
    }
    let mut actual_geometry = loaded.doc.clone();
    let mut expected_geometry = reference.clone();
    let (mut actual_count, mut expected_count) = (0, 0);
    geometry_colours(&mut actual_geometry.layers, &mut actual_count);
    geometry_colours(&mut expected_geometry.layers, &mut expected_count);
    assert_eq!((actual_count, expected_count), (180, 180));
    let a = Renderer::new().render_region(&actual_geometry, frame, 2.0, true);
    let b = Renderer::new().render_region(&expected_geometry, expected_frame, 2.0, true);
    let geometry_mean = a.pixels.iter().zip(&b.pixels).map(|(a, b)| a.abs_diff(*b) as f64).sum::<f64>() / a.pixels.len() as f64;
    eprintln!(
        "private zero-page CDR: 180 curves,424 subpaths; geometry mean {geometry_mean:.6}; colour mean {mean:.6}; warnings={:?}",
        loaded.warnings
    );
    if let Ok(output) = std::env::var("CDR_RENDER_DIR") {
        std::fs::create_dir_all(&output).unwrap();
        std::fs::write(Path::new(&output).join("zero-page-native.png"), actual.to_png().unwrap()).unwrap();
        std::fs::write(Path::new(&output).join("zero-page-reference.png"), expected.to_png().unwrap()).unwrap();
    }
    assert!(geometry_mean < 0.1, "native zero-page geometry/stacking differs from its Corel SVG oracle: {geometry_mean}");
    assert!(mean < 0.1, "native zero-page colours differ from this drawing's Corel SVG oracle: {mean}");
}
