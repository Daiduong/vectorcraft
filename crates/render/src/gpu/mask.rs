//! Aligned image opacity masks. Content stays editable; both textures use the image cache.
use vectorcraft_color::BlendMode;
use vectorcraft_doc::{ImageObject, Knockout, Node, NodeKind};

pub(super) fn aligned_image_mask(node: &Node) -> Result<Option<&ImageObject>, String> {
    let Some(mask) = node.extras().mask.as_deref().filter(|mask| !mask.disabled) else { return Ok(None) };
    let unsupported = || format!("opacity mask for object {} needs CPU rendering", node.id);
    let (NodeKind::Image(image), NodeKind::Image(art)) = (&node.kind, &mask.art.kind) else { return Err(unsupported()) };
    let m = &mask.art;
    // Clip/non-inverted masks have zero coverage outside their image. The other cases need
    // coverage outside the raster and different edge handling, so remain on the CPU.
    if !mask.clip
        || mask.invert
        || image.width != art.width
        || image.height != art.height
        || image.xf != art.xf
        || !m.visible
        || m.opacity != 1.0
        || m.blend != BlendMode::Normal
        || m.knockout == Knockout::On
        || m.isolate
        || !node.appearance.items.is_empty()
        || !m.appearance.items.is_empty()
        || crate::fx::has_object_fx(m)
        || m.appearance.effects.iter().any(|e| e.visible)
        || m.extras().mask.is_some()
        || m.extras().shaper.is_some()
        || m.extras().trace.is_some()
        || m.extras().intertwine.is_some()
    {
        return Err(unsupported());
    }
    Ok(Some(art))
}
