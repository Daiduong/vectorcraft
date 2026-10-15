//! What Craft AI does inside VectorCraft. Craft AI is one assistant across the ArtCraft family
//! — the model, the runtime and the shared folder belong to the family — but its *work* is the
//! app's own: the system prompt tells the model which app it is inside and what that app does,
//! and every ready-made task is framed for that app's world, so the answer that comes back is
//! about vector paths and Pathfinder steps, not generic advice.
//!
//! Each app of the family carries its own `tasks.rs` — WordCraft's asks about documents and
//! styles, SoundCraft's about tracks and buses, and so on — same model, same shared folder,
//! different craft. The labels and hints are catalog keys: the panel shows them translated.

/// The app Craft AI is embedded in, as the shared folder's records name it.
pub const APP: &str = "VectorCraft";

/// One ready-made task of the Assistant tab.
pub struct Task {
    /// The label the picker shows (a catalog key).
    pub label: &'static str,
    /// The input field's hint (a catalog key).
    pub hint: &'static str,
}

/// The tasks, in the picker's order.
pub const TASKS: &[Task] = &[
    Task { label: "Ask anything", hint: "Ask in any language" },
    Task { label: "Explain a path or shape", hint: "Paste or describe the path, shapes or Pathfinder steps" },
    Task { label: "Describe artwork (alt text)", hint: "Describe the artwork for someone who cannot see it" },
    Task { label: "Name this artwork", hint: "What's in the file, for names" },
    Task { label: "How do I…", hint: "What you want to achieve" },
    Task { label: "Export advice", hint: "Where the artwork is going (web, print, other apps)" },
];

/// Tell the model where it is and how to answer: briefly, practically, in the language the user
/// wrote in, about this app's own world — the words of its interface, the files it opens and
/// writes. The last line is the honesty clause: a local model can be wrong, and it is told to
/// say so rather than invent.
pub fn system_prompt() -> String {
    "You are Craft AI, the assistant built into VectorCraft, a desktop vector-graphics editor. \
     VectorCraft's world, in its own words: paths, curves and anchor points; shapes, fills and \
     strokes with swatches; layers and artboards; the Pathfinder-style boolean operations of its \
     Pathfinder menu (unite, minus front, intersect, exclude); align and distribute; text, its \
     fonts and its character panel; SVG, PDF, EPS and AI files opened and written; PNG and JPEG \
     export; and the properties panel, the variables panel and the asset export panel. Answer in \
     the language the user wrote in, Arabic included. Be brief and practical: name the menu or \
     the panel, give the steps as a short list, say what to watch out for, and when a question is \
     not about VectorCraft or graphics, answer it anyway, briefly. When the file is shared \
     through CraftHub — the family's collaboration platform — answer about that world too: \
     joining by code or link, the peers in the session and what they are editing, comments \
     and their pins, contributions and the review queue before changes land, and named \
     versions. Never invent menu entries or panel names: if you are not sure VectorCraft \
     has a feature, say so plainly."
        .to_string()
}

/// Frame the user's input for the task they picked.
pub fn user_prompt(task: usize, input: &str) -> String {
    let input = input.trim();
    match TASKS[task.min(TASKS.len() - 1)].label {
        "Explain a path or shape" => format!(
            "In VectorCraft, explain in practical steps how to build or fix this path, shape or \
             Pathfinder result: {input}"
        ),
        "Describe artwork (alt text)" => format!("Write alt text, one or two plain and concrete sentences, for this vector artwork: {input}"),
        "Name this artwork" => format!(
            "Suggest short, fitting file and layer names for the artwork described below — a \
             handful of options, no explanations: {input}"
        ),
        "How do I…" => format!("In VectorCraft, how do I: {input}"),
        "Export advice" => format!(
            "Advise on exporting this artwork for where it is going — format, resolution, colour, \
             pitfalls — in a short list: {input}"
        ),
        _ => input.to_string(),
    }
}
