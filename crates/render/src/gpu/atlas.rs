//! Mirror the renderer's atlas allocation so panning need not rebuild its shader pipelines.
use std::collections::BTreeSet;

use compute_vello::low_level::vello_encoding::{Patch, Resolver};

use super::{ImageInventory, PreparedScene, check_images};

#[derive(Default)]
pub struct AtlasTracker {
    resolver: Resolver,
    packed: Vec<u8>,
    images: ImageInventory,
}

impl AtlasTracker {
    /// Check the next submission, in the same CMY/K order as the GPU renderer. `true` means
    /// its renderer must be recreated before rendering this frame. Call exactly once per
    /// submitted frame; after a rendering error discard both tracker and renderer.
    pub fn prepare(&mut self, frame: &PreparedScene) -> Result<bool, String> {
        check_images(&frame.images)?;
        let mut combined = self.images.clone();
        let changed = frame.images.iter().any(|(id, size)| self.images.get(id).is_some_and(|old| old != size));
        combined.extend(frame.images.iter().map(|(&id, &size)| (id, size)));
        // Bound retained images across views as well as within each prepared frame.
        let mut reset = changed || check_images(&combined).is_err();
        if reset {
            *self = Self::default();
        }
        if !self.fits(frame) {
            // A fragmented or full atlas can silently omit images. Retry a fresh allocator,
            // and require the actual renderer to start from the identical empty state.
            *self = Self::default();
            reset = true;
            if !self.fits(frame) {
                return Err("GPU image atlas could not fit this frame".into());
            }
        }
        self.images.extend(frame.images.iter().map(|(&id, &size)| (id, size)));
        Ok(reset)
    }

    fn fits(&mut self, frame: &PreparedScene) -> bool {
        for scene in std::iter::once(&frame.scene).chain(frame.cmyk.iter().map(|c| &c.black)) {
            let mut needed = BTreeSet::new();
            for patch in &scene.encoding().resources.patches {
                if let Patch::Image { image, .. } = patch {
                    needed.insert(image.data.id());
                    // Dirty affects upload reporting only, not allocation or eviction. It makes
                    // every requested resident image visible to this preflight's success check.
                    // The actual renderer retains its clean uploads and does not upload again.
                    self.resolver.mark_image_dirty(image);
                }
            }
            let (_, _, atlas) = self.resolver.resolve(scene.encoding(), &mut self.packed);
            let available: BTreeSet<_> = atlas.images.iter().map(|(image, _, _)| image.data.id()).collect();
            if !needed.is_subset(&available) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compute_vello::peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};
    use std::sync::Arc;
    use vectorcraft_geom::Affine;

    fn image() -> ImageData {
        ImageData {
            data: Blob::new(Arc::new(vec![255; 16])),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::AlphaPremultiplied,
            width: 2,
            height: 2,
        }
    }

    fn frame(images: &[ImageData]) -> PreparedScene {
        let mut scene = compute_vello::Scene::new();
        for image in images {
            scene.draw_image(image, Affine::IDENTITY);
        }
        PreparedScene { scene, cmyk: None, width: 32, height: 32, images: images.iter().map(|im| (im.data.id(), (im.width, im.height))).collect() }
    }

    #[test]
    fn panning_reuses_allocations_when_images_enter_leave_and_return() {
        let (a, b) = (image(), image());
        let mut tracker = AtlasTracker::default();
        for images in [vec![a.clone()], vec![a.clone(), b.clone()], vec![b.clone()], vec![], vec![a]] {
            assert!(!tracker.prepare(&frame(&images)).unwrap(), "ordinary pan rebuilt the renderer");
        }
        assert_eq!(tracker.images.len(), 2);
    }

    #[test]
    fn changing_dimensions_or_exceeding_retained_image_budget_resets() {
        let mut tracker = AtlasTracker::default();
        let mut a = image();
        assert!(!tracker.prepare(&frame(&[a.clone()])).unwrap());
        a.width = 1;
        assert!(tracker.prepare(&frame(&[a])).unwrap());
        for _ in 1..super::super::MAX_IMAGES {
            assert!(!tracker.prepare(&frame(&[image()])).unwrap());
        }
        assert!(tracker.prepare(&frame(&[image()])).unwrap());
        assert_eq!(tracker.images.len(), 1);
    }
}
