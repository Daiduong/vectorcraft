use super::*;
use vectorcraft_doc::Appearance;
use vectorcraft_geom::shapes;

#[test]
fn alpha_shader_is_valid_without_a_graphics_device() {
    let module = naga::front::wgsl::parse_str(composite::COMPOSITE_SHADER).unwrap();
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty()).validate(&module).unwrap();
}

#[test]
fn canvas_uses_device_dimensions_and_accounts_for_all_color_targets() {
    let limits = CanvasLimits { max_texture_side: 8192, ..Default::default() };
    let mut doc = Document::new(100.0, 100.0);
    doc.color_mode = vectorcraft_doc::ColorMode::Cmyk;
    let mut node = rect(&mut doc, Rect::new(10.0, 10.0, 90.0, 90.0), Paint::solid(Color::cmyk(0.3, 0.2, 0.1, 0.0)));
    node.opacity = 0.5;
    insert(&mut doc, node);
    let mut renderer = Renderer::new();
    let opts = RenderOptions::default();
    // A maximized 5K display's physical canvas exceeds the former 4096-side cap.
    let frame = renderer.prepare_gpu_with_limits(&doc, 5120, 2880, Affine::IDENTITY, &opts, limits).unwrap();
    assert!(frame.cmyk.is_some());
    assert_eq!((frame.width, frame.height), (5120, 2880));
    let small_device = CanvasLimits { max_texture_side: 4096, ..limits };
    let error = renderer.prepare_gpu_with_limits(&doc, 5120, 2880, Affine::IDENTITY, &opts, small_device).err().unwrap();
    assert_eq!(error, "canvas exceeds graphics device limits");
    // The third CMYK target must count even when two RGB targets fit the budget.
    let budget = CanvasLimits { max_target_bytes: 5120 * 2880 * 8, ..limits };
    let error = renderer.prepare_gpu_with_limits(&doc, 5120, 2880, Affine::IDENTITY, &opts, budget).err().unwrap();
    assert_eq!(error, "GPU canvas render-target memory budget exceeded");
    doc.color_mode = vectorcraft_doc::ColorMode::Rgb;
    assert!(renderer.prepare_gpu_with_limits(&doc, 5120, 2880, Affine::IDENTITY, &opts, budget).is_ok());
    doc.color_mode = vectorcraft_doc::ColorMode::Cmyk;
    assert!(renderer.prepare_gpu_with_limits(&doc, 2560, 1440, Affine::IDENTITY, &opts, budget).is_ok());
    for (width, height) in [(0, 1), (1, 0), (8193, 1), (1, 8193), (u32::MAX, u32::MAX)] {
        assert!(limits.check(width, height, false).is_err());
    }
    assert!(limits.check(8192, 8192, false).is_err());
}

#[test]
fn active_gpu_geometry_stays_cached_after_eviction_frames() {
    let mut doc = Document::new(100.0, 100.0);
    for _ in 0..1100 {
        let id = doc.alloc_id();
        doc.insert(None, usize::MAX, Node::path(id, shapes::rectangle(Rect::new(10.0, 10.0, 90.0, 90.0)), Appearance::default_art())).unwrap();
    }
    let mut renderer = Renderer::new();
    renderer.prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).unwrap();
    let paths: Vec<_> = renderer.geom.values().filter_map(|entry| entry.path.clone()).collect();
    assert_eq!(paths.len(), 1100);
    for _ in 0..8 {
        renderer.prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).unwrap();
    }
    assert!(paths.iter().all(|path| Arc::strong_count(path) > 1), "active geometry was evicted and rebuilt");
}

#[test]
fn unsupported_art_and_modes_never_encode_partial_scenes() {
    let mut doc = Document::new(100.0, 100.0);
    let id = doc.alloc_id();
    let mut node = Node::path(id, shapes::rectangle(Rect::new(10.0, 10.0, 90.0, 90.0)), Appearance::default_art());
    doc.insert(None, usize::MAX, node.clone()).unwrap();
    let mut renderer = Renderer::new();
    assert!(renderer.prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_ok());
    node.knockout = Knockout::On;
    let id = doc.alloc_id();
    node.id = id;
    doc.insert(None, usize::MAX, node).unwrap();
    assert!(renderer.prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    assert!(renderer.prepare_gpu(&Document::new(100.0, 100.0), u32::MAX, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    assert!(renderer.prepare_gpu(&Document::new(100.0, 100.0), 100, 100, Affine::scale(f64::NAN), &RenderOptions::default()).is_err());
}

fn rect(doc: &mut Document, bounds: Rect, paint: Paint) -> Node {
    Node::path(doc.alloc_id(), shapes::rectangle(bounds), Appearance::basic(paint, Paint::None, 0.0))
}
fn insert(doc: &mut Document, node: Node) {
    doc.insert(None, usize::MAX, node).unwrap();
}

fn masked_image(doc: &mut Document, xf: Affine) -> Node {
    use vectorcraft_doc::{ImageBlob, ImageObject, OpacityMask};
    let color = if doc.color_mode == vectorcraft_doc::ColorMode::Cmyk {
        let data = (0..64).flat_map(|i| [if i % 8 < 4 { 40 } else { 170 }, 100, 20, 35]).collect();
        ImageBlob::cmyk_tiff(&vectorcraft_doc::cmyk::Inks { width: 8, height: 8, data }).unwrap()
    } else {
        let mut png = vec![];
        image::RgbaImage::from_fn(8, 8, |x, y| image::Rgba([220, 40, 100, if x < 4 && y < 4 { 128 } else { 255 }]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        ImageBlob::new("image/png", png)
    };
    doc.images.insert("mask-color".into(), color);
    let mut png = vec![];
    image::RgbaImage::from_fn(8, 8, |x, y| {
        if x == 0 {
            return image::Rgba([0, 0, 0, 255]);
        }
        if x == 7 {
            return image::Rgba([255, 255, 255, 255]);
        }
        let v = (x * 36) as u8;
        image::Rgba([v, 255 - v, v / 2, if y < 4 { 128 } else { 255 }])
    })
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .unwrap();
    doc.images.insert("generated-mask".into(), ImageBlob::new("image/png", png));
    let object =
        |key: &str| NodeKind::Image(Box::new(ImageObject { key: key.into(), width: 8, height: 8, xf, link: None, placement: Default::default() }));
    let mut image = Node::new(doc.alloc_id(), object("mask-color"));
    let mask = Node::new(doc.alloc_id(), object("generated-mask"));
    image.opacity = 0.65;
    image.extras_mut().mask = Some(Box::new(OpacityMask::new(mask, true)));
    image
}

/// Original generated artwork, with broad regions for strict interior comparisons. No font,
/// artwork or reference image is copied from another application.
fn fixture(cmyk: bool) -> Document {
    use vectorcraft_color::{Gradient, GradientGeom, GradientPaint, GradientStop};
    use vectorcraft_doc::{CharStyle, Dash, ImageBlob, ImageObject, TextObject};
    use vectorcraft_geom::Point;
    let mut doc = Document::new(512.0, 512.0);
    if cmyk {
        doc.color_mode = vectorcraft_doc::ColorMode::Cmyk;
    }
    let color = |r: f32, g: f32, b: f32| if cmyk { Color::cmyk(r, g, b, 0.1) } else { Color::rgb(r, g, b) };
    for (i, blend) in BlendMode::ALL.into_iter().enumerate() {
        let x = (i % 4) as f64 * 128.0;
        let y = 256.0 + (i / 4) as f64 * 64.0;
        let back = rect(&mut doc, Rect::new(x + 4.0, y + 4.0, x + 124.0, y + 60.0), Paint::solid(color(0.3, 0.65, 0.8)));
        insert(&mut doc, back);
        let mut top = rect(&mut doc, Rect::new(x + 20.0, y + 12.0, x + 110.0, y + 48.0), Paint::solid(color(0.8, 0.2, 0.4)));
        top.blend = blend;
        top.opacity = 0.65;
        insert(&mut doc, top);
    }
    let mut gradient = GradientPaint::new(Gradient {
        kind: GradientKind::Linear,
        stops: vec![
            GradientStop { opacity: 0.4, midpoint: 0.3, ..GradientStop::new(0.0, color(0.8, 0.2, 0.3)) },
            GradientStop::new(1.0, color(0.1, 0.7, 0.3)),
        ],
    });
    let mut clip = rect(&mut doc, Rect::new(12.0, 12.0, 112.0, 112.0), Paint::solid(color(0.2, 0.2, 0.8)));
    *clip.appearance = Appearance::basic(Paint::solid(color(0.2, 0.2, 0.8)), Paint::solid(color(0.1, 0.9, 0.2)), 4.0);
    let mut body = rect(&mut doc, Rect::new(0.0, 0.0, 150.0, 150.0), Paint::Gradient(Box::new(gradient.clone())));
    body.opacity = 0.7;
    let mut blend = rect(&mut doc, Rect::new(56.0, 56.0, 140.0, 96.0), Paint::solid(color(0.7, 0.4, 0.2)));
    blend.blend = BlendMode::Screen;
    let group = Node::new(doc.alloc_id(), NodeKind::Group { clip: true, children: vec![Arc::new(clip), Arc::new(body), Arc::new(blend)] });
    insert(&mut doc, group);
    gradient.gradient.kind = GradientKind::Radial;
    gradient.geom =
        Some(GradientGeom { start: Point::new(180.0, 60.0), end: Point::new(230.0, 60.0), aspect: 0.6, focal: Some(Point::new(170.0, 60.0)) });
    let ellipse = Node::path(
        doc.alloc_id(),
        shapes::ellipse(Rect::new(130.0, 16.0, 240.0, 112.0)),
        Appearance::basic(Paint::Gradient(Box::new(gradient)), Paint::None, 0.0),
    );
    insert(&mut doc, ellipse);
    let mut png = vec![];
    image::RgbaImage::from_fn(8, 8, |x, y| {
        image::Rgba([if x < 4 { 220 } else { 40 }, if y < 4 { 30 } else { 200 }, 80, if x < 4 && y < 4 { 128 } else { 255 }])
    })
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .unwrap();
    doc.images.insert("generated".into(), ImageBlob::new("image/png", png));
    let image = Node::new(
        doc.alloc_id(),
        NodeKind::Image(Box::new(ImageObject {
            key: "generated".into(),
            width: 8,
            height: 8,
            xf: Affine::translate((280.0, 16.0)) * Affine::scale(11.0),
            link: None,
            placement: Default::default(),
        })),
    );
    let mut tile_image = image.clone();
    if let NodeKind::Image(im) = &mut tile_image.kind {
        im.xf = Affine::IDENTITY;
    }
    let mut pattern = vectorcraft_doc::pattern::PatternDef::new("generated-pattern", vec![Arc::new(tile_image)]);
    pattern.tile = Rect::new(0.0, 0.0, 8.0, 8.0);
    doc.patterns.push(pattern);
    insert(&mut doc, image);
    let mut patterned =
        rect(&mut doc, Rect::new(400.0, 16.0, 504.0, 112.0), Paint::Pattern { pattern: "generated-pattern".into(), xf: Affine::scale(2.0) });
    patterned.opacity = 0.8;
    insert(&mut doc, patterned);
    let masked = masked_image(&mut doc, Affine::translate((280.25, 126.5)) * Affine::scale(9.0));
    insert(&mut doc, masked);
    let backdrop = rect(&mut doc, Rect::new(356.0, 120.0, 448.0, 214.0), Paint::solid(color(0.25, 0.4, 0.6)));
    insert(&mut doc, backdrop);
    let mut blended_mask = masked_image(&mut doc, Affine::translate((370.25, 128.5)) * Affine::rotate(0.08) * Affine::scale(9.0));
    blended_mask.blend = BlendMode::Screen;
    insert(&mut doc, blended_mask);
    let style = CharStyle { size: 38.0, fill: Paint::solid(color(0.1, 0.2, 0.3)), ..Default::default() };
    let text = Node::new(doc.alloc_id(), NodeKind::Text(Box::new(TextObject::point(Point::new(12.0, 170.0), "GPU ffi 123", style))));
    insert(&mut doc, text);
    let mut line = BezPath::new();
    line.move_to((12.0, 200.0));
    line.line_to((490.0, 200.0));
    let mut appearance = Appearance::basic(Paint::None, Paint::solid(color(0.1, 0.7, 0.2)), 4.0);
    if let Some(AppearanceItem::Stroke(stroke)) = appearance.items.last_mut() {
        stroke.dash = Some(Dash { pattern: vec![8.0, 4.0], offset: 2.0, align_corners: false });
    }
    let line = Node::path(doc.alloc_id(), vectorcraft_geom::PathData::from_bezpath(&line), appearance);
    insert(&mut doc, line);
    let mut strokes = vec![];
    for (index, (a, b)) in [((400.0, 225.0), (490.0, 245.0)), ((400.0, 245.0), (490.0, 225.0))].into_iter().enumerate() {
        let mut path = BezPath::new();
        path.move_to(a);
        path.line_to(b);
        let mut appearance = Appearance::basic(Paint::None, Paint::solid(color(0.3, 0.6, 0.2)), 4.0);
        if let Some(AppearanceItem::Stroke(stroke)) = appearance.items.last_mut() {
            stroke.cap = vectorcraft_doc::LineCap::Round;
            if index == 1 {
                stroke.dash = Some(Dash { pattern: vec![6.0, 4.0], offset: 0.0, align_corners: false });
            }
        }
        strokes.push(Arc::new(Node::path(doc.alloc_id(), vectorcraft_geom::PathData::from_bezpath(&path), appearance)));
    }
    let mut knockout = Node::group(doc.alloc_id(), strokes);
    knockout.knockout = Knockout::On;
    insert(&mut doc, knockout);
    doc
}

#[test]
fn common_artwork_and_both_color_modes_prepare_without_fallback() {
    let _fonts = crate::tests_fontchange::fonts_steady();
    let mut r = Renderer::new();
    for cmyk in [false, true] {
        let doc = fixture(cmyk);
        let p = r.prepare_gpu(&doc, 512, 512, Affine::IDENTITY, &RenderOptions::default()).unwrap();
        assert_eq!(p.cmyk.is_some(), cmyk);
        assert_eq!(p.images.len(), if cmyk { 7 } else { 4 });
        let second = r.prepare_gpu(&doc, 512, 512, Affine::IDENTITY, &RenderOptions::default()).unwrap();
        assert_eq!(p.images, second.images, "unchanged images must retain their upload identity");
    }
}

#[test]
fn replacing_image_bytes_also_invalidates_the_uploaded_pattern_tile() {
    let _fonts = crate::tests_fontchange::fonts_steady();
    let mut doc = fixture(false);
    let mut renderer = Renderer::new();
    let first = renderer.prepare_gpu(&doc, 512, 512, Affine::IDENTITY, &RenderOptions::default()).unwrap();
    let mut png = Vec::new();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([80, 100, 220, 255]))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    doc.images.insert("generated".into(), vectorcraft_doc::ImageBlob::new("image/png", png));
    let second = renderer.prepare_gpu(&doc, 512, 512, Affine::IDENTITY, &RenderOptions::default()).unwrap();
    assert_eq!(second.images.len(), 4);
    assert_eq!(
        second.images.keys().filter(|key| !first.images.contains_key(key)).count(),
        2,
        "both the image and its repeated tile must be uploaded again; masked images remain resident"
    );
}

#[test]
fn aligned_masks_reuse_uploads_across_moves_and_invalidate_changed_mask_bytes() {
    let mut doc = Document::new_with_mode(100.0, 100.0, vectorcraft_doc::ColorMode::Cmyk);
    let node = masked_image(&mut doc, Affine::scale(8.0));
    let id = node.id;
    insert(&mut doc, node);
    let mut renderer = Renderer::new();
    let opts = RenderOptions::default();
    let first = renderer.prepare_gpu(&doc, 256, 256, Affine::IDENTITY, &opts).unwrap();
    assert_eq!(first.images.len(), 3, "CMY, K and one display-color mask");
    {
        let n = doc.node_mut(id).unwrap();
        if let NodeKind::Image(im) = &mut n.kind {
            im.xf = Affine::translate((5.0, 10.0)) * im.xf;
        }
        let mut extras = n.extras_mut();
        let mask = Arc::make_mut(&mut extras.mask.as_mut().unwrap().art);
        if let NodeKind::Image(im) = &mut mask.kind {
            im.xf = Affine::translate((5.0, 10.0)) * im.xf;
        }
    }
    let moved = renderer.prepare_gpu(&doc, 256, 256, Affine::IDENTITY, &opts).unwrap();
    assert_eq!(first.images, moved.images, "moving linked art must not upload either texture again");
    let mut png = vec![];
    image::RgbaImage::from_pixel(8, 8, image::Rgba([128, 128, 128, 255]))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    doc.images.insert("generated-mask".into(), vectorcraft_doc::ImageBlob::new("image/png", png));
    let changed = renderer.prepare_gpu(&doc, 256, 256, Affine::IDENTITY, &opts).unwrap();
    assert_eq!(changed.images.keys().filter(|key| !moved.images.contains_key(key)).count(), 1, "only the changed mask needs an upload");
}

#[test]
fn unsupported_mask_semantics_use_cpu_and_disabled_masks_are_ignored() {
    let mut doc = Document::new(100.0, 100.0);
    let node = masked_image(&mut doc, Affine::scale(8.0));
    let id = node.id;
    insert(&mut doc, node);
    let mut renderer = Renderer::new();
    let opts = RenderOptions::default();
    let prepare = |renderer: &mut Renderer, doc: &Document| renderer.prepare_gpu(doc, 256, 256, Affine::IDENTITY, &opts);
    assert!(prepare(&mut renderer, &doc).is_ok());
    for case in 0..6 {
        let mut changed = doc.clone();
        {
            let n = changed.node_mut(id).unwrap();
            let mut extras = n.extras_mut();
            let mask = extras.mask.as_mut().unwrap();
            match case {
                0 => mask.invert = true,
                1 => mask.clip = false,
                2 => Arc::make_mut(&mut mask.art).opacity = 0.5,
                3 => Arc::make_mut(&mut mask.art).appearance.effects.push(vectorcraft_doc::Effect {
                    id: "blur.gaussian".into(),
                    params: serde_json::json!({}),
                    visible: true,
                }),
                4 => {
                    if let NodeKind::Image(im) = &mut Arc::make_mut(&mut mask.art).kind {
                        im.xf = Affine::translate((0.5, 0.0)) * im.xf;
                    }
                }
                _ => Arc::make_mut(&mut mask.art).kind = NodeKind::Group { children: vec![], clip: false },
            }
        }
        assert!(prepare(&mut renderer, &changed).is_err(), "case {case} must not silently ignore mask behavior");
        changed.node_mut(id).unwrap().extras_mut().mask.as_mut().unwrap().disabled = true;
        assert!(prepare(&mut renderer, &changed).is_ok(), "disabled mask in case {case} must not affect artwork");
    }
}

#[test]
fn unsupported_groups_outside_the_view_do_not_force_cpu_rendering() {
    let mut doc = Document::new(100.0, 100.0);
    let mut child = rect(&mut doc, Rect::new(1000.0, 1000.0, 1100.0, 1100.0), Paint::solid(Color::BLACK));
    child.opacity = 0.5;
    let mut group = Node::new(doc.alloc_id(), NodeKind::Group { clip: false, children: vec![Arc::new(child)] });
    group.knockout = Knockout::On;
    insert(&mut doc, group);
    let mut renderer = Renderer::new();
    assert!(renderer.prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_ok());
    assert!(renderer.prepare_gpu(&doc, 100, 100, Affine::translate((-1000.0, -1000.0)), &RenderOptions::default()).is_err());
}

#[test]
fn artboards_far_beyond_the_view_at_deep_zoom_stay_on_the_gpu() {
    let doc = Document::new(1000.0, 1000.0);
    let mut renderer = Renderer::new();
    let opts = RenderOptions { artboards: true, ..RenderOptions::default() };
    assert_eq!(doc.artboards.len(), 1);
    // 256× zoom on the artboard's centre puts its corners at ±128 000 px, beyond the GPU
    // coordinate limit.
    let view = Affine::scale(256.0) * Affine::translate((-500.0, -500.0));
    assert!(renderer.prepare_gpu(&doc, 100, 100, view, &opts).is_ok());
    let view = Affine::scale(128.0);
    assert!(renderer.prepare_gpu(&doc, 100, 100, view, &opts).is_ok());
}

#[test]
fn text_far_beyond_the_view_is_culled_instead_of_forcing_cpu_rendering() {
    use vectorcraft_doc::{CharStyle, TextObject};
    use vectorcraft_geom::Point;
    let mut doc = Document::new(10000.0, 10000.0);
    let near = rect(&mut doc, Rect::new(10.0, 10.0, 90.0, 90.0), Paint::solid(Color::BLACK));
    insert(&mut doc, near);
    let style = CharStyle { size: 24.0, fill: Paint::solid(Color::BLACK), ..Default::default() };
    let text = Node::new(doc.alloc_id(), NodeKind::Text(Box::new(TextObject::point(Point::new(9000.0, 9000.0), "far away", style))));
    // Grouped, as text usually is: containers holding text are not culled by their model boxes.
    let group = Node::new(doc.alloc_id(), NodeKind::Group { clip: false, children: vec![Arc::new(text)] });
    insert(&mut doc, group);
    let mut renderer = Renderer::new();
    // At 16× the text lies past 140 000 px, beyond the GPU coordinate limit.
    assert!(renderer.prepare_gpu(&doc, 100, 100, Affine::scale(16.0), &RenderOptions::default()).is_ok());
}

#[test]
fn only_flat_opaque_normal_path_knockouts_use_source_over() {
    let mut doc = Document::new(100.0, 100.0);
    let child = rect(&mut doc, Rect::new(10.0, 10.0, 90.0, 90.0), Paint::solid(Color::BLACK));
    let mut group = Node::group(doc.alloc_id(), vec![Arc::new(child)]);
    group.knockout = Knockout::On;
    assert!(opaque_knockout_group(&group));
    let id = group.id;
    insert(&mut doc, group);
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_ok());
    for case in 0..4 {
        let mut changed = doc.clone();
        let group = changed.node_mut(id).unwrap();
        let child = Arc::make_mut(group.children_mut().unwrap().first_mut().unwrap());
        match case {
            0 => child.opacity = 0.5,
            1 => child.blend = BlendMode::Screen,
            2 => {
                if let Some(AppearanceItem::Fill(fill)) = child.appearance.items.first_mut() {
                    fill.opacity = 0.5;
                }
            }
            _ => child.kind = NodeKind::Group { children: vec![], clip: false },
        }
        assert!(!opaque_knockout_group(group));
        assert!(Renderer::new().prepare_gpu(&changed, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    }
}

#[test]
fn nonisolated_blending_group_requires_fallback_but_an_isolated_one_prepares() {
    let mut doc = Document::new(100.0, 100.0);
    let mut child = rect(&mut doc, Rect::new(10.0, 10.0, 80.0, 80.0), Paint::solid(Color::BLACK));
    child.blend = BlendMode::Multiply;
    let mut group = Node::new(doc.alloc_id(), NodeKind::Group { clip: false, children: vec![Arc::new(child)] });
    group.opacity = 0.5;
    insert(&mut doc, group.clone());
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    doc.layers.clear();
    group.isolate = true;
    insert(&mut doc, group);
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_ok());
}

#[test]
fn a_leaf_whose_fill_blends_under_object_opacity_requires_fallback() {
    // A Multiply fill and a stroke under 50% object opacity: the fill must blend with the art
    // below, which the CPU reaches through a backdrop copy and a GPU layer can't.
    let mut doc = Document::new(100.0, 100.0);
    let backdrop = rect(&mut doc, Rect::new(0.0, 0.0, 100.0, 100.0), Paint::solid(Color::rgb(0.0, 0.0, 1.0)));
    insert(&mut doc, backdrop);
    let mut leaf = Node::path(
        doc.alloc_id(),
        shapes::rectangle(Rect::new(10.0, 10.0, 80.0, 80.0)),
        Appearance::basic(Paint::solid(Color::rgb(1.0, 0.0, 0.0)), Paint::solid(Color::BLACK), 4.0),
    );
    if let Some(AppearanceItem::Fill(fill)) = leaf.appearance.items.first_mut() {
        fill.blend = BlendMode::Multiply;
    }
    leaf.opacity = 0.5;
    let prepare = |leaf: &Node| {
        let mut doc = doc.clone();
        insert(&mut doc, leaf.clone());
        Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default())
    };
    assert!(prepare(&leaf).is_err());
    let mut screen = leaf.clone();
    screen.opacity = 1.0;
    screen.blend = BlendMode::Screen;
    assert!(prepare(&screen).is_err());
    let mut isolated = leaf.clone();
    isolated.isolate = true;
    assert!(prepare(&isolated).is_ok());
    // Opaque, or with the fill alone (its opacity folds into the paint), it blends inline.
    let mut opaque = leaf.clone();
    opaque.opacity = 1.0;
    assert!(prepare(&opaque).is_ok());
    let mut alone = leaf;
    alone.appearance.items.truncate(1);
    assert!(prepare(&alone).is_ok());
}

#[test]
fn malicious_dashes_and_gradient_values_return_fallback() {
    use vectorcraft_doc::Dash;
    let mut doc = Document::new(100.0, 100.0);
    let mut node = rect(&mut doc, Rect::new(10.0, 10.0, 80.0, 80.0), Paint::None);
    *node.appearance = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.0);
    if let Some(AppearanceItem::Stroke(st)) = node.appearance.items.last_mut() {
        st.dash = Some(Dash { pattern: vec![1e-100, 1e-100], offset: 0.0, align_corners: false });
    }
    insert(&mut doc, node);
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    let mut node = rect(&mut doc, Rect::new(10.0, 10.0, 80.0, 80.0), Paint::solid(Color::rgb(f32::NAN, 0.0, 0.0)));
    doc.layers.clear();
    insert(&mut doc, node.clone());
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
    node.opacity = f32::INFINITY;
    doc.layers.clear();
    insert(&mut doc, node);
    assert!(Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).is_err());
}

#[test]
fn too_many_distinct_images_fall_back_before_gpu_submission() {
    use vectorcraft_doc::{ImageBlob, ImageObject};
    let mut doc = Document::new(100.0, 100.0);
    for i in 0..257u16 {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(1, 1, image::Rgba([i as u8, (i / 256) as u8, 80, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let key = format!("generated-budget-{i}");
        doc.images.insert(key.clone(), ImageBlob::new("image/png", png));
        let node = Node::new(
            doc.alloc_id(),
            NodeKind::Image(Box::new(ImageObject {
                key,
                width: 1,
                height: 1,
                xf: Affine::scale_non_uniform(0.02, 10.0),
                link: None,
                placement: Default::default(),
            })),
        );
        insert(&mut doc, node);
    }
    let error = Renderer::new().prepare_gpu(&doc, 100, 100, Affine::IDENTITY, &RenderOptions::default()).err().unwrap();
    assert!(error.contains("atlas budget"), "{error}");
}

#[test]
fn excessive_full_viewport_overdraw_falls_back_before_fixed_gpu_buffers_overflow() {
    let mut doc = Document::new(1000.0, 1000.0);
    for _ in 0..50 {
        let node = rect(&mut doc, Rect::new(0.0, 0.0, 1000.0, 1000.0), Paint::solid(Color::BLACK));
        insert(&mut doc, node);
    }
    let error = Renderer::new().prepare_gpu(&doc, 4096, 4096, Affine::scale(4.096), &RenderOptions::default()).err().unwrap();
    assert!(error.contains("tile-command buffer"), "{error}");
}

#[test]
#[ignore = "requires a native GPU backend; run with --features gpu-probe -- --ignored --nocapture"]
fn hardware_rgb_and_cmyk_match_cpu() {
    let _fonts = crate::tests_fontchange::fonts_steady();
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.expect("requested GPU comparison needs an adapter");
        eprintln!("GPU assertions run on {:?}", adapter.get_info());
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();
        let mut gpu = compute_vello::Renderer::new(
            &device,
            compute_vello::RendererOptions {
                num_init_threads: std::num::NonZeroUsize::new(1),
                antialiasing_support: compute_vello::AaSupport::area_only(),
                ..Default::default()
            },
        )
        .unwrap();
        let mut conversion = CompositePass::new(&device);
        let mut atlas = AtlasTracker::default();
        let mut prepare = Renderer::new();
        let mut cpu = Renderer::new();
        for cmyk in [false, true] {
            for background in [None, Some([240, 240, 240, 255])] {
                // Keep the same geometry at 4K and 5K to isolate target size/stride changes
                // from scale-dependent differences between the two antialiasing algorithms.
                for (width, height, scale) in [(512, 512, 1.0), (3840, 2160, 4.0), (5120, 2880, 4.0)] {
                    let view = Affine::scale(scale);
                    let doc = fixture(cmyk);
                    let opts = RenderOptions { background, ..Default::default() };
                    let reference = cpu.render(&doc, width, height, view, &opts);
                    let prepared = prepare.prepare_gpu_with_limits(&doc, width, height, view, &opts, CanvasLimits::for_device(&device)).unwrap();
                    assert!(!atlas.prepare(&prepared).unwrap(), "changing the visible image set should preserve these small fixtures' renderer");
                    let descriptor = wgpu::TextureDescriptor {
                        label: Some("VectorCraft GPU regression"),
                        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    };
                    let color = device.create_texture(&descriptor);
                    let black = device.create_texture(&descriptor);
                    let target = device.create_texture(&descriptor);
                    let c = color.create_view(&Default::default());
                    let k = black.create_view(&Default::default());
                    let params = compute_vello::RenderParams {
                        base_color: peniko::Color::TRANSPARENT,
                        width,
                        height,
                        antialiasing_method: compute_vello::AaConfig::Area,
                    };
                    gpu.render_to_texture(&device, &queue, &prepared.scene, &c, &params).unwrap();
                    if let Some(cmyk) = &prepared.cmyk {
                        gpu.render_to_texture(&device, &queue, &cmyk.black, &k, &params).unwrap();
                    }
                    let bind = conversion.bind(&device, &c, &k, &target.create_view(&Default::default()), prepared.cmyk.as_ref());
                    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("GPU comparison readback"),
                        size: u64::from(width) * u64::from(height) * 4,
                        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    let mut encoder = device.create_command_encoder(&Default::default());
                    conversion.encode(&mut encoder, &bind, width as u16, height as u16);
                    encoder.copy_texture_to_buffer(
                        target.as_image_copy(),
                        wgpu::TexelCopyBufferInfo {
                            buffer: &buffer,
                            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: None },
                        },
                        target.size(),
                    );
                    queue.submit([encoder.finish()]);
                    let (tx, rx) = std::sync::mpsc::channel();
                    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                        let _ = tx.send(r);
                    });
                    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    rx.recv().unwrap().unwrap();
                    let pixels = buffer.slice(..).get_mapped_range().unwrap().to_vec();
                    buffer.unmap();
                    let actual = crate::Rendered { width, height, pixels };
                    compare(&reference, &actual, &format!("cmyk-{cmyk}-{width}x{height}-opaque-{}", background.is_some()));
                }
            }
        }
    });
}

fn compare(cpu: &crate::Rendered, gpu: &crate::Rendered, label: &str) {
    let mut max_interior = 0u8;
    let mut sum = 0u64;
    let mut max = 0u8;
    let w = cpu.width as usize;
    let h = cpu.height as usize;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let at = (y * w + x) * 4;
            let stable = (0..4).all(|ch| {
                let base = cpu.pixels[at + ch];
                (-1..=1isize).all(|dy| {
                    (-1..=1isize).all(|dx| {
                        let neighbor = ((y as isize + dy) as usize * w + (x as isize + dx) as usize) * 4 + ch;
                        base.abs_diff(cpu.pixels[neighbor]) <= 1
                    })
                })
            });
            for ch in 0..4 {
                let diff = cpu.pixels[at + ch].abs_diff(gpu.pixels[at + ch]);
                sum += u64::from(diff);
                max = max.max(diff);
                if stable {
                    max_interior = max_interior.max(diff);
                }
            }
        }
    }
    let mean = sum as f64 / ((w - 2) as f64 * (h - 2) as f64 * 4.0);
    eprintln!("{label}: mean={mean:.4}, max={max}, interior_max={max_interior}");
    let prefix = std::env::temp_dir().join(format!("vectorcraft-gpu-{label}"));
    std::fs::write(prefix.with_extension("cpu.png"), cpu.to_png().unwrap()).unwrap();
    std::fs::write(prefix.with_extension("gpu.png"), gpu.to_png().unwrap()).unwrap();
    assert!(max_interior <= 4, "{label}: flat interiors must match; maximum={max_interior}");
    assert!(mean <= 1.0, "{label}: a whole-image tolerance must not hide missing content: {mean}");
    assert!(max <= 80, "{label}: unexpectedly large edge error {max}");
}
