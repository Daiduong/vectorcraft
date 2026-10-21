//! PowerClip resource references, verified with original CDR 15/16/27 drawings.
//!
//! `doc / clpt / grp ` stores the contents under its `spnd` identifier. A frame's
//! argument 8005 names that group; the group's argument 3500 names the frame's
//! `spnd`. Contents already carry their complete transforms, including rotated
//! frames. This is distinct from bitmap crop geometry in argument 30.

use super::{Drawing, arguments, metadata};
use crate::{container::Chunk, reader::Reader};
use std::sync::Arc;
use vectorcraft_doc::{Node, NodeKind};
use vectorcraft_geom::Affine;

impl<'a> Drawing<'a> {
    /// Index borrowed clipping resources without materializing their artwork.
    pub(super) fn index_clips(&mut self, chunks: &'a [Chunk]) -> Result<(), String> {
        for chunk in chunks {
            if chunk.list.as_ref() == Some(b"obj ")
                && let Some(loda) = metadata(chunk, b"loda").or_else(|| metadata(chunk, b"lobj"))
            {
                let (_, args) = arguments(&loda.data)?;
                if let Some(reference) = args.get(&8005)
                    && Reader::new(reference).u32()? != 0
                {
                    let owner = chunk.child(b"spnd").ok_or("CDR PowerClip frame is missing its identifier")?;
                    let owner = Reader::new(&owner.data).u32()?;
                    // A contents resource has one owning frame. Repeated physical
                    // frames with the same identifier would turn a tiny acyclic
                    // resource graph into exponentially duplicated artwork/names.
                    if owner == 0 || !self.clip_owners.insert(owner) {
                        return Err("duplicate or invalid CDR PowerClip frame identifier".into());
                    }
                }
            }
            if chunk.list.as_ref() == Some(b"clpt") {
                for group in &chunk.children {
                    if group.list.as_ref() != Some(b"grp ") {
                        continue;
                    }
                    let Some(id) = group.child(b"spnd") else { continue };
                    let id = Reader::new(&id.data).u32()?;
                    if id == 0 || self.clips.insert(id, group).is_some() {
                        return Err("invalid or duplicate CDR PowerClip resource identifier".into());
                    }
                }
            }
            self.index_clips(&chunk.children)?;
        }
        Ok(())
    }

    /// Attach the referenced editable contents to an already decoded vector frame.
    /// The frame's fill paints below the contents and its outline above them.
    pub(super) fn powerclip(&mut self, source: &Chunk, mut frame: Node, reference: &[u8], parent: Affine) -> Result<Node, String> {
        let mut r = Reader::new(reference);
        let id = r.u32()?;
        if id == 0 {
            return Ok(frame);
        }
        if r.u32()? != 1 || r.u32()? != 0 || r.u32()? != 0 || r.pos != reference.len() {
            return Err("unsupported CDR PowerClip reference settings".into());
        }
        let group = self.clips.get(&id).copied().ok_or("unresolved CDR PowerClip contents")?;
        let owner = source.child(b"spnd").ok_or("CDR PowerClip frame is missing its identifier")?;
        let owner = Reader::new(&owner.data).u32()?;
        let loda = metadata(group, b"loda").or_else(|| metadata(group, b"lobj")).ok_or("CDR PowerClip resource is missing its owner")?;
        let (_, args) = arguments(&loda.data)?;
        let target = args.get(&3500).ok_or("CDR PowerClip resource is missing its owner reference")?;
        if Reader::new(target).u32()? != owner {
            return Err("CDR PowerClip owner and contents references disagree".into());
        }
        if !self.active_clips.insert(id) {
            return Err("cyclic CDR PowerClip reference".into());
        }
        // The common traversal depth/visit quotas also cover graph expansion,
        // including many empty or aliased resources that consume no point quota.
        let contents = self.children(&group.children, parent);
        self.active_clips.remove(&id);
        let contents = contents?;
        let NodeKind::Path { clipping, .. } = &mut frame.kind else {
            return Err("unsupported CDR PowerClip frame geometry".into());
        };
        *clipping = true;
        let name = frame.name.take();
        let mut children = Vec::with_capacity(contents.len() + 1);
        children.push(Arc::new(frame));
        children.extend(contents);
        let mut node = Node::new(self.id(), NodeKind::Group { children, clip: true });
        node.name = name;
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]

    use super::*;
    use vectorcraft_color::{Color, Paint};
    use vectorcraft_geom::FillRule;

    /// Decode an original generated container, including resources outside pages.
    fn fixture() -> Vec<Chunk> {
        crate::container::load(&crate::synth::powerclip()).unwrap().0
    }

    /// Find the resource pool in this original fixture.
    fn pool(chunks: &mut [Chunk]) -> &mut Vec<Chunk> {
        &mut chunks.iter_mut().find(|c| c.list.as_ref() == Some(b"doc ")).unwrap().children[0].children
    }

    /// The mask remains painted geometry, and both mask and oversized content remain editable.
    #[test]
    fn referenced_contents_retain_paths_paint_and_hole_rule() {
        let imported = crate::drawing::import(&fixture(), 1200).unwrap();
        assert!(imported.warnings.is_empty(), "{:?}", imported.warnings);
        assert!(imported.document.images.is_empty());
        let clip = &imported.document.layers[0].children().unwrap()[0];
        assert!(clip.clips());
        let nodes = clip.children().unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(matches!(nodes[0].kind, NodeKind::Path { rule: FillRule::EvenOdd, clipping: true, .. }));
        assert_eq!(nodes[0].appearance.fill_paint(), Paint::solid(Color::rgb8(220, 40, 70)));
        assert_eq!(nodes[1].appearance.fill_paint(), Paint::solid(Color::rgb8(20, 100, 230)));
        assert_eq!(nodes[0].path_data().unwrap().subpaths.len(), 2);
    }

    /// A dangling reference cannot quietly replace a populated frame with its bare outline.
    #[test]
    fn missing_or_mismatched_contents_fail_explicitly() {
        let mut chunks = fixture();
        pool(&mut chunks).clear();
        assert!(crate::drawing::import(&chunks, 1200).unwrap_err().contains("unresolved CDR PowerClip"));
        let mut chunks = fixture();
        let owner = pool(&mut chunks)[0].children.iter_mut().find(|c| c.id == *b"loda").unwrap();
        owner.data[20..24].copy_from_slice(&99u32.to_le_bytes());
        assert!(crate::drawing::import(&chunks, 1200).unwrap_err().contains("references disagree"));
    }

    /// Resource recursion must fail before it can exhaust the call stack or materialize art.
    #[test]
    fn cyclic_powerclip_resources_are_rejected() {
        let mut other = fixture();
        let page = other.iter_mut().find(|c| c.list.as_ref() == Some(b"page")).unwrap();
        let frame = page.children.remove(1);
        let mut chunks = fixture();
        // Start traversal directly at the resource's owning frame, so this also
        // exercises the runtime guard independently of the duplicate-owner guard.
        let source = chunks.iter_mut().find(|c| c.list.as_ref() == Some(b"page")).unwrap().children.remove(1);
        pool(&mut chunks)[0].children.push(frame);
        let mut drawing = Drawing::new(1200);
        drawing.index_clips(&chunks).unwrap();
        assert!(drawing.object(&source, Affine::IDENTITY).unwrap_err().contains("cyclic CDR PowerClip"));
    }

    /// Ambiguous group identifiers cannot bind to whichever resource happens to be last.
    #[test]
    fn duplicate_resource_identifiers_are_rejected() {
        let mut other = fixture();
        let duplicate = pool(&mut other).remove(0);
        let mut chunks = fixture();
        pool(&mut chunks).push(duplicate);
        assert!(crate::drawing::import(&chunks, 1200).unwrap_err().contains("duplicate CDR PowerClip"));
    }

    /// Two references per level must not exponentially copy names or empty artwork.
    #[test]
    fn a_small_branching_resource_graph_is_rejected_before_expansion() {
        /// Change a generated single-word argument without changing its table layout.
        fn word(chunk: &mut Chunk, id: u32, value: u32) {
            let loda = chunk.children.iter_mut().find(|c| c.id == *b"loda").unwrap();
            let data = &mut loda.data;
            let count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
            let offsets = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
            let types = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
            for index in 0..count {
                let tag = types + (count - 1 - index) * 4;
                if u32::from_le_bytes(data[tag..tag + 4].try_into().unwrap()) == id {
                    let at = offsets + index * 4;
                    let at = u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize;
                    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
                    return;
                }
            }
            panic!("the generated argument exists");
        }
        let mut chunks = fixture();
        let groups = pool(&mut chunks);
        groups.clear();
        for level in 0..15u32 {
            let mut template = fixture();
            let mut group = pool(&mut template).remove(0);
            group.children.iter_mut().find(|c| c.id == *b"spnd").unwrap().data = (8 + level).to_le_bytes().to_vec();
            word(&mut group, 3500, 10 + level);
            if level < 14 {
                group.children.retain(|c| c.list.as_ref() != Some(b"obj "));
                for _ in 0..2 {
                    let mut template = fixture();
                    let page = template.iter_mut().find(|c| c.list.as_ref() == Some(b"page")).unwrap();
                    let mut frame = page.children.remove(1);
                    frame.children.iter_mut().find(|c| c.id == *b"spnd").unwrap().data = (11 + level).to_le_bytes().to_vec();
                    word(&mut frame, 8005, 9 + level);
                    group.children.push(frame);
                }
            }
            groups.push(group);
        }
        let mut drawing = Drawing::new(1200);
        assert!(drawing.index_clips(&chunks).unwrap_err().contains("PowerClip frame identifier"));
        assert_eq!(drawing.objects_imported, 0);
        assert_eq!(drawing.visits, 0);
    }

    /// Reference expansion consumes the same global budget as ordinary page traversal.
    #[test]
    fn referenced_contents_cannot_bypass_the_visit_or_depth_budget() {
        let chunks = fixture();
        let mut drawing = Drawing::new(1200);
        drawing.index_clips(&chunks).unwrap();
        let page = chunks.iter().find(|c| c.list.as_ref() == Some(b"page")).unwrap();
        drawing.visits = 199_998;
        let error = drawing.children(&page.children, Affine::IDENTITY).unwrap_err();
        assert!(error.contains("reference expansion limit"));
        assert!(drawing.active_clips.is_empty(), "failed traversal must release its active reference");
        drawing.visits = 0;
        drawing.depth = 63;
        assert!(drawing.children(&page.children, Affine::IDENTITY).unwrap_err().contains("reference depth limit"));
        assert!(drawing.active_clips.is_empty());
    }
}
