//! The document knowledge the hub validates against (spec §3.6 phase A: `DocSchema`): which
//! kinds exist, which parents accept children — layers under the root, nodes under containers.

use craft_collab::adapter::ValidationCtx;
use craft_collab::change::Change;
use craft_collab::limits::Limits;

/// The vectorcraft document schema.
#[derive(Debug, Default)]
pub struct VectorSchema {
    limits: Limits,
}

impl craft_collab::adapter::DocSchema for VectorSchema {
    fn validate(&self, change: &Change, ctx: &ValidationCtx<'_>) -> Result<(), String> {
        match change {
            Change::Create { kind, parent, props, .. } => {
                if !crate::KINDS.contains(&kind.as_ref()) {
                    return Err(format!("unknown kind '{kind}'"));
                }
                if *parent == crate::ROOT {
                    if kind.as_ref() != "layer" {
                        return Err(format!("'{kind}' does not attach to the file root; only layers do"));
                    }
                } else {
                    let p = ctx.replica.node(*parent).ok_or_else(|| format!("unknown parent {parent:?}"))?;
                    let accepts = crate::CONTAINERS.contains(&p.kind.as_ref()) && (kind.as_ref() != "layer" || p.kind.as_ref() == "layer");
                    if !accepts {
                        return Err(format!("a '{}' accepts no '{}'", p.kind, kind));
                    }
                }
                for (p, _) in props {
                    if p.as_ref() != "parent" && p.as_ref() != "position" && !crate::props::node_props().contains(&p.as_ref()) {
                        return Err(format!("unknown property '{p}' for a '{kind}'"));
                    }
                }
                Ok(())
            }
            Change::Set { prop, .. } => {
                if prop.as_ref() == "parent" || prop.as_ref() == "position" {
                    return Ok(());
                }
                if !crate::props::node_props().contains(&prop.as_ref()) {
                    return Err(format!("unknown property '{prop}'"));
                }
                Ok(())
            }
            Change::Delete { .. } => Ok(()),
            Change::SetDoc { prop, .. } => {
                if crate::props::DOC_PROPS.contains(&prop.as_ref()) {
                    Ok(())
                } else {
                    Err(format!("unknown document property '{prop}'"))
                }
            }
            Change::Put { .. } | Change::Remove { .. } => Err("vectorcraft has no document-level entry lists in the first cut".into()),
        }
    }

    fn recompute(&self, _touched: &[craft_collab::ids::NodeId]) -> Vec<Change> {
        // The honest answer, like the pilot's for figcraft: VectorCraft's derived art — live
        // shapes (rectangle/ellipse/polygon recognition), blend steps, envelope warps, repeats —
        // is *evaluated on demand from its stored specs*, exactly like the render-side caches;
        // nothing in the document recomputes on its own after a change. So there is no
        // author-only recompute to compose (spec §5.7's map decides: "commands, not derived
        // state"). When a genuinely derived property lands, its recompute belongs here and
        // nowhere else.
        Vec::new()
    }

    fn limits(&self) -> &Limits {
        &self.limits
    }
}
