//! Capability-checked native GPU scene preparation. Unsupported frames use the CPU renderer.
use std::collections::BTreeMap;
use std::sync::Arc;

use compute_vello::peniko::{self, Brush, Color as GpuColor, Fill, ImageData};
use vectorcraft_color::{BlendMode, Color, GradientKind, Paint};
use vectorcraft_doc::{AppearanceItem, Document, Knockout, Node, NodeKind, StrokeAlign, StrokeLayer, SymbolBoxes};
use vectorcraft_geom::{Affine, BezPath, FillRule, PathEl, Rect, Shape};

use crate::ink::Ink;
use crate::{AntiAlias, RenderOptions, Renderer};

mod atlas;
mod composite;
mod mask;
pub use atlas::AtlasTracker;
pub use composite::CompositePass;

const MAX_IMAGE_SIDE: u32 = 4096;
const MAX_NODES: usize = 100_000;
const MAX_ELEMENTS: usize = 1_000_000;
const MAX_DEPTH: usize = 64;
const MAX_IMAGE_BYTES: usize = 64 << 20;
const MAX_IMAGES: usize = 256;

/// Canvas allocation policy, checked before scene preparation and again before submission.
/// The budget covers all full-size RGBA8 targets: two for RGB, three for CMYK. Backend
/// buffers and image residency have separate limits; this is not a VRAM usage query.
#[derive(Clone, Copy, Debug)]
pub struct CanvasLimits {
    pub max_texture_side: u32,
    pub max_target_bytes: u64,
}

impl Default for CanvasLimits {
    fn default() -> Self {
        Self { max_texture_side: u32::from(u16::MAX), max_target_bytes: 256 << 20 }
    }
}

impl CanvasLimits {
    pub fn for_device(device: &wgpu::Device) -> Self {
        Self { max_texture_side: device.limits().max_texture_dimension_2d, ..Self::default() }
    }

    pub fn check(self, width: u32, height: u32, cmyk: bool) -> Result<(), String> {
        // PreparedScene stores dimensions as u16. Check before casts or size arithmetic.
        let side = self.max_texture_side.min(u32::from(u16::MAX));
        if width == 0 || height == 0 || width > side || height > side {
            return Err("canvas exceeds graphics device limits".into());
        }
        let bytes = u64::from(width) * u64::from(height) * if cmyk { 12 } else { 8 };
        if bytes > self.max_target_bytes {
            return Err("GPU canvas render-target memory budget exceeded".into());
        }
        Ok(())
    }
}

pub type ImageInventory = BTreeMap<u64, (u32, u32)>;

pub struct PreparedScene {
    pub scene: compute_vello::Scene,
    pub cmyk: Option<CmykComposition>,
    pub width: u16,
    pub height: u16,
    /// Stable image blob identities and dimensions; reused resources are uploaded only once.
    pub images: ImageInventory,
}

pub struct CmykComposition {
    pub black: compute_vello::Scene,
    pub lookup: Arc<[[f32; 4]]>,
    pub generation: u64,
    pub background: Option<[[u8; 4]; 3]>,
    pub rich_black: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ImageKey {
    Placed(crate::images::ImageKey, u16, u16),
    Tile(usize),
}

/// Retain the source allocation so tile identities cannot be recycled while cached/uploaded.
struct PixmapBytes(Arc<vello_cpu::Pixmap>);
impl AsRef<[u8]> for PixmapBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.data_as_u8_slice()
    }
}

enum Op {
    Fill(Arc<BezPath>, FillRule, Brush, Affine, Affine),
    Image(ImageData, Affine, peniko::ImageQuality),
    Push(f32, peniko::BlendMode, Option<(Arc<BezPath>, FillRule)>),
    PushMask,
    Pop,
}

struct Preparation<'a> {
    renderer: &'a mut Renderer,
    doc: &'a Document,
    opts: &'a RenderOptions,
    ink: Ink,
    fold_alpha: f32,
    view: Affine,
    visible: Rect,
    ops: Vec<Op>,
    nodes: usize,
    elements: usize,
    images: ImageInventory,
    symbols: SymbolBoxes<'a>,
}

impl Renderer {
    pub fn prepare_gpu(&mut self, doc: &Document, width: u32, height: u32, view: Affine, opts: &RenderOptions) -> Result<PreparedScene, String> {
        self.prepare_gpu_with_limits(doc, width, height, view, opts, CanvasLimits::default())
    }

    pub fn prepare_gpu_with_limits(
        &mut self,
        doc: &Document,
        width: u32,
        height: u32,
        view: Affine,
        opts: &RenderOptions,
        limits: CanvasLimits,
    ) -> Result<PreparedScene, String> {
        limits.check(width, height, false)?;
        validate_transform(view)?;
        if doc.page_knockout || doc.pattern_edit.is_some() || doc.mask_edit.is_some() {
            return Err("document editing or knockout mode needs CPU rendering".into());
        }
        if opts.outline
            || opts.proof.is_some()
            || opts.overprint_preview
            || opts.trim
            || opts.mask_view.is_some()
            || opts.isolated.is_some()
            || opts.anti_alias != AntiAlias::Art
            || opts.precise
            || opts.supersample
        {
            return Err("this preview mode needs CPU rendering".into());
        }
        self.check_plugins();
        self.check_symbols(doc);
        let ink = if self.blends_in_cmyk(doc, opts) {
            Ink::Cmy
        } else if opts.rich_black {
            Ink::RichBlack
        } else {
            Ink::Display
        };
        limits.check(width, height, ink == Ink::Cmy)?;
        self.begin_frame();
        let result = (|| {
            let (scene, mut images) = self.gpu_scene(doc, width, height, view, opts, ink)?;
            let cmyk = if ink == Ink::Cmy {
                let (black, black_images) = self.gpu_scene(doc, width, height, view, opts, Ink::K)?;
                images.extend(black_images);
                let (lookup, generation) = crate::ink::gpu_display_lut();
                Some(CmykComposition {
                    black,
                    lookup,
                    generation,
                    rich_black: opts.rich_black,
                    background: opts.background.map(|bg| [Ink::Cmy.fixed8(bg), Ink::K.fixed8(bg), bg]),
                })
            } else {
                None
            };
            check_images(&images)?;
            if !images.is_empty() && self.gpu_inventory.as_ref() != Some(&images) {
                // Use the backend's actual allocator, in the same CMY/K order as submission.
                // Its API otherwise silently omits an image when the atlas cannot fit it.
                let mut resolver = compute_vello::low_level::vello_encoding::Resolver::new();
                let mut packed = Vec::new();
                let mut uploaded = std::collections::BTreeSet::new();
                for scene in std::iter::once(&scene).chain(cmyk.iter().map(|c| &c.black)) {
                    let (_, _, atlas) = resolver.resolve(scene.encoding(), &mut packed);
                    uploaded.extend(atlas.images.iter().map(|(im, _, _)| im.data.id()));
                }
                if images.keys().any(|id| !uploaded.contains(id)) {
                    return Err("GPU image atlas could not fit this frame".into());
                }
                self.gpu_inventory = Some(images.clone());
            }
            Ok(PreparedScene { scene, cmyk, images, width: width as u16, height: height as u16 })
        })();
        self.end_frame();
        result
    }

    fn gpu_scene(
        &mut self,
        doc: &Document,
        width: u32,
        height: u32,
        view: Affine,
        opts: &RenderOptions,
        ink: Ink,
    ) -> Result<(compute_vello::Scene, ImageInventory), String> {
        let viewport = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
        let mut p = Preparation {
            renderer: self,
            doc,
            opts,
            ink,
            fold_alpha: 1.0,
            view,
            visible: view.inverse().transform_rect_bbox(viewport),
            ops: vec![],
            nodes: 0,
            elements: 0,
            images: BTreeMap::new(),
            symbols: doc.symbol_boxes().quick(),
        };
        if doc.page_isolate {
            p.push(1.0, BlendMode::Normal, None);
        }
        for layer in &doc.layers {
            p.node(layer, 0)?;
        }
        if doc.page_isolate {
            p.ops.push(Op::Pop);
        }
        let mut scene = compute_vello::Scene::new();
        if let Some(bg) = opts.background {
            if bg[3] != 0 && bg[3] != 255 {
                return Err("translucent canvas backgrounds need CPU rendering".into());
            }
            if bg[3] == 255 {
                scene.fill(Fill::NonZero, Affine::IDENTITY, ink.fixed(bg), None, &viewport);
            }
        }
        if opts.artboards {
            if doc.artboards.len() > 10000 {
                return Err("GPU artboard limit exceeded".into());
            }
            // Clip each artboard to the view so off-screen corners at deep zoom stay within the
            // coordinate limits instead of forcing the whole frame onto the CPU.
            let visible = p.visible.inflate(1.0, 1.0);
            for artboard in &doc.artboards {
                let clipped = artboard.rect.abs().intersect(visible);
                if clipped.is_zero_area() {
                    continue;
                }
                let path = clipped.to_path(0.1);
                validate_path(&path, view)?;
                scene.fill(Fill::NonZero, Affine::IDENTITY, ink.fixed([255; 4]), None, &(view * path));
            }
        }
        let layers = layer_bounds(&p.ops, view, viewport);
        check_commands(&p.ops, &layers, view, viewport, doc, opts)?;
        for (index, op) in p.ops.into_iter().enumerate() {
            match op {
                Op::Fill(path, rule, brush, local, paint_xf) => {
                    let screen = (view * local) * (*path).clone();
                    scene.fill(fill_rule(rule), Affine::IDENTITY, &brush, Some(view * local * paint_xf), &screen);
                }
                Op::Image(image, xf, quality) => scene.draw_image(peniko::ImageBrush::from(image).with_quality(quality).as_ref(), view * xf),
                Op::Push(alpha, blend, clip) => match clip {
                    Some((path, rule)) => scene.push_clip_layer(fill_rule(rule), Affine::IDENTITY, &(view * (*path).clone())),
                    None => scene.push_layer(Fill::NonZero, blend, alpha, Affine::IDENTITY, &layers.get(index).copied().unwrap_or(Rect::ZERO)),
                },
                Op::PushMask => {
                    scene.push_luminance_mask_layer(Fill::NonZero, 1.0, Affine::IDENTITY, &layers.get(index).copied().unwrap_or(Rect::ZERO))
                }
                Op::Pop => scene.pop_layer(),
            }
        }
        Ok((scene, p.images))
    }
}

/// Bound opacity/blend layers to their painted content. A full-viewport layer per object can
/// exhaust Vello's fixed command buffer at 4K even for a modest imported illustration.
fn layer_bounds(ops: &[Op], view: Affine, viewport: Rect) -> Vec<Rect> {
    let mut bounds = vec![Rect::ZERO; ops.len()];
    let mut stack: Vec<(usize, Option<Rect>, Option<Rect>)> = Vec::new();
    for (index, op) in ops.iter().enumerate() {
        let painted = match op {
            Op::Fill(path, _, _, local, _) => Some((view * *local).transform_rect_bbox(path.bounding_box())),
            Op::Image(image, xf, _) => Some((view * *xf).transform_rect_bbox(Rect::new(0.0, 0.0, f64::from(image.width), f64::from(image.height)))),
            Op::Push(_, _, _) | Op::PushMask => {
                let clip = match op {
                    Op::Push(_, _, Some((path, _))) => Some(view.transform_rect_bbox(path.bounding_box())),
                    _ => None,
                };
                stack.push((index, None, clip));
                None
            }
            Op::Pop => stack.pop().and_then(|(at, b, clip)| {
                let b = b.map(|b| clip.map_or(b, |c| b.intersect(c)));
                if let Some(b) = b {
                    bounds[at] = b.inflate(1.0, 1.0).intersect(viewport);
                }
                b
            }),
        };
        if let Some(painted) = painted
            && let Some((_, accumulated, _)) = stack.last_mut()
        {
            *accumulated = Some(accumulated.map_or(painted, |old| old.union(painted)));
        }
    }
    bounds
}

/// Vello 0.11 fixes its per-tile command storage at 2^23 words. Count conservatively before
/// submission: six words per solid/image draw, seven per gradient, ten per begin/end clip, with room for
/// chunk jumps and end markers. This avoids its silent blank frame on command-buffer overflow.
fn check_commands(ops: &[Op], layers: &[Rect], view: Affine, viewport: Rect, doc: &Document, opts: &RenderOptions) -> Result<(), String> {
    let cols = (viewport.width() as usize).div_ceil(16);
    let rows = (viewport.height() as usize).div_ceil(16);
    let mut counts = vec![1u32; cols * rows];
    let mut work = 0usize;
    let mut charge = |bounds: Rect, words: u32| -> Result<(), String> {
        let b = bounds.inflate(1.0, 1.0).intersect(viewport);
        if b.width() <= 0.0 || b.height() <= 0.0 {
            return Ok(());
        }
        let x0 = (b.x0 / 16.0).floor().max(0.0) as usize;
        let y0 = (b.y0 / 16.0).floor().max(0.0) as usize;
        let x1 = ((b.x1 / 16.0).ceil() as usize).min(cols);
        let y1 = ((b.y1 / 16.0).ceil() as usize).min(rows);
        work = work.saturating_add((x1.saturating_sub(x0)) * (y1.saturating_sub(y0)));
        if work > 4_000_000 {
            return Err("GPU tile-command preparation limit exceeded".into());
        }
        for y in y0..y1 {
            for x in x0..x1 {
                if let Some(c) = counts.get_mut(y * cols + x) {
                    *c = c.saturating_add(words);
                }
            }
        }
        Ok(())
    };
    if opts.background.is_some_and(|b| b[3] > 0) {
        charge(viewport, 8)?;
    }
    if opts.artboards {
        for board in &doc.artboards {
            charge(view.transform_rect_bbox(board.rect), 8)?;
        }
    }
    for (index, op) in ops.iter().enumerate() {
        match op {
            Op::Fill(path, _, brush, local, _) => {
                charge((view * *local).transform_rect_bbox(path.bounding_box()), if matches!(brush, Brush::Gradient(_)) { 7 } else { 6 })?
            }
            Op::Image(image, xf, _) => {
                charge((view * *xf).transform_rect_bbox(Rect::new(0.0, 0.0, f64::from(image.width), f64::from(image.height))), 6)?
            }
            Op::Push(_, _, Some((clip, _))) => charge(view.transform_rect_bbox(clip.bounding_box()), 10)?,
            Op::Push(_, _, None) => charge(layers.get(index).copied().unwrap_or(Rect::ZERO), 10)?,
            Op::PushMask => charge(layers.get(index).copied().unwrap_or(Rect::ZERO), 10)?,
            Op::Pop => {}
        }
    }
    let required: u64 = (counts.len() as u64) * 64 + counts.iter().map(|c| u64::from(c.saturating_sub(56).div_ceil(248)) * 256).sum::<u64>();
    if required > 1 << 23 {
        return Err("GPU tile-command buffer budget exceeded".into());
    }
    Ok(())
}

impl Preparation<'_> {
    fn node(&mut self, node: &Arc<Node>, depth: usize) -> Result<(), String> {
        self.nodes += 1;
        if self.nodes > MAX_NODES || depth > MAX_DEPTH {
            return Err("GPU scene exceeds its node or nesting limit".into());
        }
        if !node.visible || self.opts.hidden.contains(&node.id) {
            return Ok(());
        }
        if matches!(node.kind, NodeKind::Layer { template: true, .. }) && (self.opts.skip_templates || self.opts.hide_templates) {
            return Ok(());
        }
        if matches!(node.kind, NodeKind::Path { guide: true, .. }) && self.opts.skip_templates {
            return Ok(());
        }
        if self.opts.dim_templates && !self.opts.skip_templates && matches!(node.kind, NodeKind::Layer { dim_images: Some(_), .. }) {
            return Err("dimmed layer images need CPU rendering".into());
        }
        check_input(node)?;
        check_alpha(node.opacity)?;
        // Containers can contain text whose font changed without changing the node allocation.
        // Do not cull by stale model text boxes; glyph preparation below uses the font generation.
        if !contains_text(node, 0)
            && let Some(bounds) = self.renderer.bounds_of(&self.symbols, node)
        {
            if !bounds.is_finite() {
                return Err("non-finite object bounds".into());
            }
            if !crate::rects_overlap(bounds.inflate(1.0, 1.0), self.visible) {
                return Ok(());
            }
        }
        let mask = mask::aligned_image_mask(node)?;
        if mask.is_some() {
            self.nodes += 1;
            if self.nodes > MAX_NODES || depth >= MAX_DEPTH {
                return Err("GPU scene exceeds its node or nesting limit".into());
            }
        }
        if node.knockout == Knockout::On && !opaque_knockout_group(node)
            || crate::fx::has_object_fx(node)
            || !node.opacity.is_finite()
            || !(0.0..=1.0).contains(&node.opacity)
            || node.appearance.effects.iter().any(|e| e.visible)
            || node.extras().shaper.is_some()
            || node.extras().trace.is_some()
            || node.extras().intertwine.is_some()
        {
            return Err(format!("object {} needs CPU compositing", node.id));
        }
        // Non-isolated blending objects depend on the backdrop and cannot be replaced by a
        // transparent offscreen layer: a group whose members blend, or a leaf whose fills and
        // strokes do (e.g. a Multiply fill under 50% object opacity). The CPU draws them over a
        // copy of the backdrop. Objects with no opacity/blend layer remain inline.
        let fold = node.opacity < 1.0
            && node.blend == BlendMode::Normal
            && !node.isolate
            && matches!(node.kind, NodeKind::Path { .. })
            && crate::painted_items(node) == 1;
        let previous_alpha = self.fold_alpha;
        self.fold_alpha = if fold { node.opacity } else { 1.0 };
        // A masked image needs its own layer so the mask cannot erase the backdrop.
        let layered = !fold && node.opacity < 1.0 || node.blend != BlendMode::Normal || node.isolate || mask.is_some();
        if layered && !node.isolate && self.renderer.blends_through(node) {
            return Err("backdrop-dependent blending needs CPU rendering".into());
        }
        if layered {
            self.push(node.opacity, node.blend, None);
        }
        match &node.kind {
            NodeKind::Layer { children, template: false, preview: true, clip, .. } | NodeKind::Group { children, clip } => {
                if !node.appearance.items.is_empty() {
                    return Err("container appearances need CPU rendering".into());
                }
                if *clip {
                    if let Some((clip_node, rest)) = children.split_first() {
                        // Bound arbitrary clip geometry before the shared clip helper evaluates it.
                        if !matches!(clip_node.kind, NodeKind::Path { .. } | NodeKind::Compound { .. }) || crate::fx::has_fx(clip_node) {
                            return Err("this clipping geometry needs CPU rendering".into());
                        }
                        check_input(clip_node)?;
                        let outline = self.renderer.path_of(&self.symbols, clip_node).ok_or("missing clipping geometry")?;
                        self.path(&outline, Affine::IDENTITY)?;
                        if let Some(region) = self.renderer.clip_of(clip_node) {
                            let (fill, stroke) = self.renderer.clip_paint_of(clip_node);
                            self.push(1.0, BlendMode::Normal, Some((Arc::new(region.0.clone()), region.1)));
                            if let Some(fill) = fill {
                                self.node(&fill, depth + 1)?;
                            }
                            for child in rest {
                                self.node(child, depth + 1)?;
                            }
                            self.ops.push(Op::Pop);
                            if let Some(stroke) = stroke {
                                self.node(&stroke, depth + 1)?;
                            }
                        }
                    }
                } else {
                    for child in children {
                        self.node(child, depth + 1)?;
                    }
                }
            }
            NodeKind::Path { guide: false, rule, .. } | NodeKind::Compound { rule, .. } => {
                if crate::fx::has_fx(node) {
                    return Err("path effects need CPU rendering".into());
                }
                let path = self.renderer.path_of(&self.symbols, node).ok_or("missing path geometry")?;
                self.items(&node.appearance.items, &path, *rule, Affine::IDENTITY, path.bounding_box())?;
            }
            NodeKind::Text(t) => 'text: {
                if t.runs.len() > 10000 || t.runs.iter().map(|r| r.text.len()).sum::<usize>() > 65536 {
                    return Err("GPU text limit exceeded".into());
                }
                let g = self.renderer.text_geom_of(node, t);
                if !g.inlines.is_empty() {
                    return Err("inline graphics need CPU rendering".into());
                }
                let bounds = t.xf.transform_rect_bbox(g.bounds);
                // Containers skip culling text (their model boxes can be stale), so cull here by
                // the fresh glyph bounds and every stroke's reach: unculled, far-off glyphs exceed
                // the coordinate limits and force the whole frame onto the CPU.
                let run_reach = t.runs.iter().filter(|r| r.style.has_stroke()).map(|r| r.style.stroke_layer().reach()).fold(0.0, f64::max);
                let reach = node.appearance.outset() + 1.0;
                let painted = t.xf.transform_rect_bbox(g.bounds.inflate(run_reach, run_reach)).union(bounds).inflate(reach, reach);
                if !painted.is_finite() {
                    return Err("non-finite object bounds".into());
                }
                if !crate::rects_overlap(painted, self.visible) {
                    break 'text;
                }
                let (below, above) = node.appearance.split_contents();
                let all = t.xf * g.all.clone();
                self.items(below, &all, FillRule::NonZero, Affine::IDENTITY, bounds)?;
                if self.opts.highlight_substitutions {
                    for (on, path) in [
                        (self.doc.setup.highlight_substituted_fonts, Some(&g.substituted_fonts)),
                        (self.doc.setup.highlight_substituted_glyphs, Some(&g.substituted_glyphs)),
                        (self.opts.highlight_alternate_glyphs, self.opts.highlight_alternate_glyphs.then(|| g.alternate_glyphs(t))),
                    ] {
                        let Some(path) = path else { continue };
                        if on && !path.elements().is_empty() {
                            // Match the fixed CPU highlight rather than separating it as artwork.
                            let c = crate::SUBSTITUTED.to_rgba8();
                            self.fill(Arc::new(path.clone()), FillRule::NonZero, Brush::Solid(c.into()), t.xf, Affine::IDENTITY)?;
                        }
                    }
                }
                for (i, run) in t.runs.iter().enumerate() {
                    if run.inline.is_some() || run.style.overprint_fill || run.style.overprint_stroke {
                        return Err("this text appearance needs CPU rendering".into());
                    }
                    let path = g.run(i);
                    if path.elements().is_empty() {
                        continue;
                    }
                    self.paint_fill(&run.style.fill, &path, FillRule::NonZero, t.xf, g.bounds)?;
                    if run.style.has_stroke() {
                        self.stroke(&run.style.stroke_layer(), &path, t.xf, g.bounds)?;
                    }
                }
                self.items(above, &all, FillRule::NonZero, Affine::IDENTITY, bounds)?;
            }
            NodeKind::Image(im) => {
                self.image(im, self.ink)?;
                if let Some(mask) = mask {
                    self.ops.push(Op::PushMask);
                    // Mask luminance is always from display colors, including in CMYK scenes.
                    self.image(mask, Ink::Display)?;
                    self.ops.push(Op::Pop);
                }
            }
            _ => return Err(format!("object {} is outside the experimental GPU subset", node.id)),
        }
        if layered {
            self.ops.push(Op::Pop);
        }
        self.fold_alpha = previous_alpha;
        Ok(())
    }

    fn push(&mut self, opacity: f32, blend: BlendMode, clip: Option<(Arc<BezPath>, FillRule)>) {
        self.ops.push(Op::Push(opacity, gpu_blend(blend), clip));
    }

    fn items(&mut self, items: &[AppearanceItem], path: &BezPath, rule: FillRule, local: Affine, bounds: Rect) -> Result<(), String> {
        if items.len() > 256 {
            return Err("GPU appearance limit exceeded".into());
        }
        for item in items {
            if !item.visible() || item.paint().is_none() {
                continue;
            }
            match item {
                AppearanceItem::Fill(fill) => {
                    if fill.overprint || fill.effects.iter().any(|e| e.visible) {
                        return Err("appearance needs CPU rendering".into());
                    }
                    check_alpha(fill.opacity)?;
                    let layered = fill.opacity < 1.0 || fill.blend != BlendMode::Normal;
                    if layered {
                        self.push(fill.opacity, fill.blend, None);
                    }
                    self.paint_fill(&fill.paint, path, rule, local, bounds)?;
                    if layered {
                        self.ops.push(Op::Pop);
                    }
                }
                AppearanceItem::Stroke(stroke) => self.stroke(stroke, path, local, bounds)?,
            }
        }
        Ok(())
    }

    fn paint_fill(&mut self, paint: &Paint, path: &BezPath, rule: FillRule, local: Affine, bounds: Rect) -> Result<(), String> {
        if let Some((brush, xf)) = self.brush(paint, bounds, local)? {
            self.fill(Arc::new(path.clone()), rule, brush, local, xf)?;
        }
        Ok(())
    }

    fn fill(&mut self, path: Arc<BezPath>, rule: FillRule, brush: Brush, local: Affine, paint: Affine) -> Result<(), String> {
        self.path(&path, local)?;
        self.ops.push(Op::Fill(path, rule, brush, local, paint));
        Ok(())
    }

    fn stroke(&mut self, st: &StrokeLayer, path: &BezPath, local: Affine, bounds: Rect) -> Result<(), String> {
        if st.width <= 0.0 {
            return Ok(());
        }
        if st.align != StrokeAlign::Center
            || st.profile.is_some()
            || st.brush.is_some()
            || st.start_arrow.is_some()
            || st.end_arrow.is_some()
            || st.overprint
            || st.effects.iter().any(|e| e.visible)
            || !st.width.is_finite()
            || st.width > 10000.0
            || !st.miter_limit.is_finite()
            || !(1.0..=100.0).contains(&st.miter_limit)
            || st.path_gradient().is_some()
        {
            return Err("this stroke needs CPU rendering".into());
        }
        self.path(path, local)?;
        check_alpha(st.opacity)?;
        let scale = (self.view * local).determinant().abs().sqrt();
        let px = 1.0 / scale.max(1e-9);
        if let Some(dash) = &st.dash {
            if dash.pattern.len() > 6 || !dash.offset.is_finite() || dash.pattern.iter().any(|v| !v.is_finite() || *v < 0.0) {
                return Err("invalid GPU stroke dashes".into());
            }
            if let Some(min) = dash.pattern.iter().copied().filter(|v| *v > 0.0).min_by(f64::total_cmp) {
                let b = path.bounding_box();
                if (b.width().abs() + b.height().abs()) * path.elements().len() as f64 / min > 50000.0 {
                    return Err("GPU stroke dash limit exceeded".into());
                }
            }
        }
        let layered = st.opacity < 1.0 || st.blend != BlendMode::Normal;
        if layered {
            self.push(st.opacity, st.blend, None);
        }
        let outline = crate::effects::stroke::line_outline(path, st, st.width.max(px * 0.5), px * 0.25);
        self.paint_fill(&st.paint, &outline, FillRule::NonZero, local, bounds.inflate(st.width / 2.0, st.width / 2.0))?;
        if layered {
            self.ops.push(Op::Pop);
        }
        Ok(())
    }

    fn brush(&mut self, paint: &Paint, bounds: Rect, local: Affine) -> Result<Option<(Brush, Affine)>, String> {
        match paint {
            Paint::None => Ok(None),
            Paint::Solid { color, .. } => {
                check_color(color)?;
                let [r, g, b] = self.ink.rgb(color);
                Ok(Some((Brush::Solid(GpuColor::from_rgba8(q(r), q(g), q(b), q(self.fold_alpha))), Affine::IDENTITY)))
            }
            Paint::Gradient(g) => {
                if g.gradient.kind == GradientKind::Freeform || g.gradient.stops.len() > 256 {
                    return Err("this gradient needs CPU rendering".into());
                }
                let geom = g.resolve(bounds);
                if ![geom.start.x, geom.start.y, geom.end.x, geom.end.y].iter().all(|v| v.is_finite() && v.abs() <= 65536.0) {
                    return Err("invalid GPU gradient geometry".into());
                }
                let mut previous = 0.0;
                for stop in &g.gradient.stops {
                    check_color(&stop.color)?;
                    check_alpha(stop.opacity)?;
                    if !stop.offset.is_finite()
                        || !(previous..=1.0).contains(&stop.offset)
                        || !stop.midpoint.is_finite()
                        || !(0.0..=1.0).contains(&stop.midpoint)
                    {
                        return Err("invalid GPU gradient stops".into());
                    }
                    previous = stop.offset;
                }
                let stops: Vec<peniko::ColorStop> = g
                    .gradient
                    .expanded_stops()
                    .iter()
                    .map(|(o, c, a)| {
                        let [r, g, b] = self.ink.rgb(c);
                        peniko::ColorStop::from((*o, GpuColor::from_rgba8(q(r), q(g), q(b), q(*a * self.fold_alpha))))
                    })
                    .collect();
                if stops.is_empty() {
                    return Ok(None);
                }
                let (gradient, xf) = if g.gradient.kind == GradientKind::Radial {
                    let squash = geom.radial_squash();
                    validate_transform(squash)?;
                    let radius = geom.length().max(1e-6) as f32;
                    let grad = if let Some(focal) = geom.focal {
                        if ![focal.x, focal.y].iter().all(|v| v.is_finite() && v.abs() <= 65536.0) {
                            return Err("invalid GPU gradient focus".into());
                        }
                        peniko::Gradient::new_two_point_radial(squash.inverse() * focal, 0.0, geom.start, radius)
                    } else {
                        peniko::Gradient::new_radial(geom.start, radius)
                    };
                    (grad, squash)
                } else {
                    let end = if geom.start.distance(geom.end) < 1e-9 { geom.start + vectorcraft_geom::Vec2::new(1.0, 0.0) } else { geom.end };
                    (peniko::Gradient::new_linear(geom.start, end), Affine::IDENTITY)
                };
                Ok(Some((
                    Brush::Gradient(
                        gradient.with_stops(stops.as_slice()).with_interpolation_alpha_space(peniko::InterpolationAlphaSpace::Unpremultiplied),
                    ),
                    xf,
                )))
            }
            Paint::Pattern { pattern, xf } => {
                validate_transform(self.view * local * *xf)?;
                let def = self.doc.pattern(pattern).ok_or("missing pattern definition")?;
                check_pattern(def, self.doc)?;
                let before = crate::paint::decode_failures();
                let Some((pixels, placement)) = crate::pattern::pattern_image(self.doc, pattern, self.view * local * *xf, self.ink)? else {
                    return Ok(None);
                };
                if crate::paint::decode_failures() != before {
                    return Err("pattern image decoding failed".into());
                }
                let key = ImageKey::Tile(Arc::as_ptr(&pixels) as usize);
                let image = self.cached_image(key, pixels)?;
                Ok(Some((
                    Brush::Image(peniko::ImageBrush::from(image).with_extend(peniko::Extend::Repeat).with_alpha(self.fold_alpha)),
                    *xf * placement,
                )))
            }
        }
    }

    fn image(&mut self, im: &vectorcraft_doc::ImageObject, ink: Ink) -> Result<(), String> {
        if im.width == 0 || im.height == 0 {
            return Err("invalid GPU image dimensions".into());
        }
        validate_transform(self.view * im.xf)?;
        validate_path(&Rect::new(0.0, 0.0, f64::from(im.width), f64::from(im.height)).to_path(0.1), self.view * im.xf)?;
        if let Some(blob) = self.doc.images.get(&im.key) {
            let reader = image::ImageReader::new(std::io::Cursor::new(blob.bytes.as_slice())).with_guessed_format().map_err(|e| e.to_string())?;
            let (w, h) = reader.into_dimensions().map_err(|e| e.to_string())?;
            if u64::from(w) * u64::from(h) * 4 > MAX_IMAGE_BYTES as u64 {
                return Err("GPU image memory limit exceeded".into());
            }
        }
        let Some((key, pixels)) = self.renderer.image_pixmap(self.doc, &im.key, false, true) else {
            return Err("GPU image could not be decoded".into());
        };
        let (key, pixels) = crate::ink::ink_image(&key, &pixels, ink, self.doc.images.get(&im.key), true);
        let pixels = if self.opts.image_mips {
            crate::image_mip(&key, pixels, self.view * im.xf * Affine::scale_non_uniform(f64::from(im.width), f64::from(im.height)))
        } else {
            pixels
        };
        let (w, h) = (pixels.width(), pixels.height());
        if u32::from(w).max(u32::from(h)) > MAX_IMAGE_SIDE {
            return Err("GPU image side limit exceeded".into());
        }
        let image = self.cached_image(ImageKey::Placed(key, w, h), pixels)?;
        let xf = im.xf * Affine::scale_non_uniform(f64::from(im.width) / f64::from(w.max(1)), f64::from(im.height) / f64::from(h.max(1)));
        self.ops.push(Op::Image(image, xf, if self.opts.smooth_images { peniko::ImageQuality::Medium } else { peniko::ImageQuality::Low }));
        Ok(())
    }

    fn cached_image(&mut self, cache: ImageKey, pixels: Arc<vello_cpu::Pixmap>) -> Result<ImageData, String> {
        let (w, h) = (pixels.width(), pixels.height());
        let bytes = usize::from(w) * usize::from(h) * 4;
        if bytes > MAX_IMAGE_BYTES {
            return Err("GPU image memory limit exceeded".into());
        }
        let image = if let Some(image) = self.renderer.gpu_images.get(&cache, self.renderer.stamp) {
            image.clone()
        } else {
            let image = ImageData {
                data: peniko::Blob::new(Arc::new(PixmapBytes(pixels))),
                format: peniko::ImageFormat::Rgba8,
                alpha_type: peniko::ImageAlphaType::AlphaPremultiplied,
                width: w.into(),
                height: h.into(),
            };
            self.renderer.gpu_images.insert(cache, image.clone(), bytes, self.renderer.stamp);
            image
        };
        self.images.insert(image.data.id(), (w.into(), h.into()));
        check_images(&self.images)?;
        Ok(image)
    }

    fn path(&mut self, path: &BezPath, local: Affine) -> Result<(), String> {
        self.elements = self.elements.saturating_add(path.elements().len());
        if self.elements > MAX_ELEMENTS {
            return Err("GPU scene exceeds its path limit".into());
        }
        validate_path(path, self.view * local)
    }
}

/// Bound input geometry before cache/clip helpers allocate converted paths.
fn check_pattern(def: &vectorcraft_doc::pattern::PatternDef, doc: &Document) -> Result<(), String> {
    if !def.tile.is_finite() || def.tile.width() < 1e-3 || def.tile.height() < 1e-3 || def.art.len() > 1000 {
        return Err("GPU pattern tile limit exceeded".into());
    }
    let mut stack: Vec<_> = def.art.iter().map(|n| (n, 0usize)).collect();
    let mut count = 0;
    let mut image_bytes = 0u64;
    while let Some((node, depth)) = stack.pop() {
        count += 1;
        if count > 1000 || depth > 32 {
            return Err("GPU pattern art limit exceeded".into());
        }
        check_input(node)?;
        check_alpha(node.opacity)?;
        if !matches!(
            node.kind,
            NodeKind::Path { .. } | NodeKind::Compound { .. } | NodeKind::Group { .. } | NodeKind::Layer { .. } | NodeKind::Image(_)
        ) || crate::fx::has_fx(node)
            || node.extras().mask.is_some()
            || node.appearance.items.iter().any(|p| matches!(p.paint(), Paint::Pattern { .. }))
        {
            return Err("this pattern tile needs CPU rendering".into());
        }
        if let NodeKind::Image(im) = &node.kind {
            let blob = doc.images.get(&im.key).ok_or("missing pattern image")?;
            let reader = image::ImageReader::new(std::io::Cursor::new(blob.bytes.as_slice())).with_guessed_format().map_err(|e| e.to_string())?;
            let (w, h) = reader.into_dimensions().map_err(|e| e.to_string())?;
            image_bytes = image_bytes.saturating_add(u64::from(w) * u64::from(h) * 4);
            if image_bytes > MAX_IMAGE_BYTES as u64 {
                return Err("GPU pattern image budget exceeded".into());
            }
        }
        for item in &node.appearance.items {
            if let AppearanceItem::Stroke(st) = item
                && (!st.width.is_finite()
                    || st.width > 10000.0
                    || st.brush.is_some()
                    || st.profile.is_some()
                    || st
                        .dash
                        .as_ref()
                        .is_some_and(|d| d.pattern.len() > 6 || !d.offset.is_finite() || d.pattern.iter().any(|v| !v.is_finite() || *v < 1e-3)))
            {
                return Err("this pattern stroke needs CPU rendering".into());
            }
        }
        if let Some(children) = node.children() {
            if children.len() > 1000 {
                return Err("GPU pattern member limit exceeded".into());
            }
            stack.extend(children.iter().map(|c| (c, depth + 1)));
        }
    }
    if let Some(bounds) = def.art_bounds()
        && (!bounds.is_finite() || bounds.width() > def.width() * 16.0 || bounds.height() > def.height() * 16.0)
    {
        return Err("GPU pattern overlap limit exceeded".into());
    }
    Ok(())
}

fn contains_text(node: &Node, depth: usize) -> bool {
    depth > MAX_DEPTH
        || matches!(node.kind, NodeKind::Text(_))
        || node.children().is_some_and(|children| children.len() > MAX_NODES || children.iter().any(|c| contains_text(c, depth + 1)))
}

/// With one opaque Normal paint per path, each child's alpha is its knockout shape.
/// Source-over therefore replaces earlier children by that same coverage. Keep the shortcut
/// flat and strict: translucent paints, masks, effects and nested groups need real knockout.
fn opaque_knockout_group(node: &Node) -> bool {
    let NodeKind::Group { clip: false, children } = &node.kind else { return false };
    node.opacity == 1.0
        && node.blend == BlendMode::Normal
        && !node.isolate
        && !children.is_empty()
        && children.len() <= MAX_NODES
        && children.iter().all(|child| {
            matches!(child.kind, NodeKind::Path { guide: false, .. })
                && child.opacity == 1.0
                && child.blend == BlendMode::Normal
                && child.knockout != Knockout::On
                && !child.isolate
                && child.extras().mask.is_none()
                && !crate::fx::has_fx(child)
                && child.appearance.items.len() <= 256
                && child.appearance.items.iter().filter(|item| item.visible() && !item.paint().is_none()).count() == 1
                && child
                    .appearance
                    .items
                    .iter()
                    .filter(|item| item.visible() && !item.paint().is_none())
                    .all(|item| item.opacity() == 1.0 && item.blend() == BlendMode::Normal && matches!(item.paint(), Paint::Solid { .. }))
        })
}

fn check_input(node: &Node) -> Result<(), String> {
    let mut points = 0usize;
    let mut count = |path: &vectorcraft_geom::PathData| {
        for sub in &path.subpaths {
            points = points.saturating_add(sub.anchors.len()).saturating_add(2);
        }
    };
    match &node.kind {
        NodeKind::Path { path, .. } => count(path),
        NodeKind::Compound { children, .. } => {
            if children.len() > MAX_NODES {
                return Err("GPU compound member limit exceeded".into());
            }
            for child in children {
                if let Some(path) = child.path_data() {
                    count(path);
                }
            }
        }
        NodeKind::Text(t) if t.runs.len() > 10000 || t.runs.iter().map(|r| r.text.len()).sum::<usize>() > 65536 => {
            return Err("GPU text limit exceeded".into());
        }
        _ => {}
    }
    if points > MAX_ELEMENTS {
        return Err("GPU input geometry limit exceeded".into());
    }
    Ok(())
}

fn check_images(images: &BTreeMap<u64, (u32, u32)>) -> Result<(), String> {
    let bytes: u64 = images.values().map(|&(w, h)| u64::from(w) * u64::from(h) * 4).sum();
    if images.len() > MAX_IMAGES || bytes > MAX_IMAGE_BYTES as u64 {
        return Err("GPU image atlas budget exceeded".into());
    }
    Ok(())
}
fn q(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}
fn check_alpha(a: f32) -> Result<(), String> {
    if !a.is_finite() || !(0.0..=1.0).contains(&a) { Err("invalid GPU opacity".into()) } else { Ok(()) }
}
fn check_color(c: &Color) -> Result<(), String> {
    let values = match *c {
        Color::Rgb { r, g, b } => [r, g, b, 0.0],
        Color::Cmyk { c, m, y, k } => [c, m, y, k],
        Color::Gray { k } => [k, 0.0, 0.0, 0.0],
        Color::Lab { l, a, b } => [l, a, b, 0.0],
    };
    if values.iter().all(|v| v.is_finite()) { Ok(()) } else { Err("non-finite paint".into()) }
}
fn validate_transform(view: Affine) -> Result<(), String> {
    if !view.as_coeffs().iter().all(|v| v.is_finite() && v.abs() <= 1e12)
        || !view.determinant().is_finite()
        || !(1e-12..=1e12).contains(&view.determinant().abs())
    {
        Err("GPU canvas needs a finite, invertible view".into())
    } else {
        Ok(())
    }
}
fn fill_rule(rule: FillRule) -> Fill {
    if rule == FillRule::NonZero { Fill::NonZero } else { Fill::EvenOdd }
}
fn gpu_blend(blend: BlendMode) -> peniko::BlendMode {
    // The same PDF blend operators as the CPU renderer, applied independently to CMY/K planes.
    use peniko::Mix;
    let mix = match blend {
        BlendMode::Normal => Mix::Normal,
        BlendMode::Darken => Mix::Darken,
        BlendMode::Multiply => Mix::Multiply,
        BlendMode::ColorBurn => Mix::ColorBurn,
        BlendMode::Lighten => Mix::Lighten,
        BlendMode::Screen => Mix::Screen,
        BlendMode::ColorDodge => Mix::ColorDodge,
        BlendMode::Overlay => Mix::Overlay,
        BlendMode::SoftLight => Mix::SoftLight,
        BlendMode::HardLight => Mix::HardLight,
        BlendMode::Difference => Mix::Difference,
        BlendMode::Exclusion => Mix::Exclusion,
        BlendMode::Hue => Mix::Hue,
        BlendMode::Saturation => Mix::Saturation,
        BlendMode::Color => Mix::Color,
        BlendMode::Luminosity => Mix::Luminosity,
    };
    peniko::BlendMode::new(mix, peniko::Compose::SrcOver)
}

fn validate_path(path: &BezPath, view: Affine) -> Result<(), String> {
    let point = |p: vectorcraft_geom::Point| {
        let p = view * p;
        p.x.is_finite() && p.y.is_finite() && p.x.abs() <= 65536.0 && p.y.abs() <= 65536.0
    };
    if path.elements().iter().all(|e| match *e {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => point(p),
        PathEl::QuadTo(a, b) => point(a) && point(b),
        PathEl::CurveTo(a, b, c) => point(a) && point(b) && point(c),
        PathEl::ClosePath => true,
    }) {
        Ok(())
    } else {
        Err("GPU path exceeds its finite coordinate limits".into())
    }
}

#[cfg(test)]
mod tests;
