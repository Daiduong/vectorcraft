//! Typesetting: syntax tree → glyphs and rules, by the rules of TeX's Appendix G with the
//! parameters taken from the font's `MATH` table.
//!
//! Working units are points, y pointing up, relative to a box's baseline; [`finish`] flips them
//! into the y-down layout space of [`MathLayout`]. Boxes carry height, depth and width as TeX's
//! do, built from the glyphs' ink.

use std::collections::HashMap;

use vectorcraft_geom::{Affine, Rect};
use vectorcraft_text::{FontDb, FontFace};

use crate::mathfont::{AssemblyPart, GlyphAssembly, KernCorner, MathConstant as K, MathFont};
use crate::output::{MathItem, MathLayout};
use crate::parser::{AccentKind, AtomClass, FracStyle, Limits, Node, Space, StyleLevel, Variant, WideKind};
use crate::symbols::{is_integral, styled_char};
use crate::{MathError, MathOptions};

/// Most glyphs (and rules) one formula may need.
pub const MAX_GLYPHS: usize = 4096;
/// Largest font size accepted, points.
const MAX_SIZE_PT: f64 = 100_000.0;
/// Most times an extender of a glyph assembly is repeated.
const MAX_EXTENDER_REPEATS: usize = 64;

/// Width of the space left around a fraction and a null delimiter, in ems (TeX's
/// `\nulldelimiterspace` is 1.2pt at 10pt).
const NULL_DELIMITER_EM: f64 = 0.12;
/// A word space, in ems.
const WORD_SPACE_EM: f64 = 1.0 / 3.0;
/// Space between an accent glyph that isn't a combining mark and what it accents, in ems.
const LOOSE_ACCENT_GAP_EM: f64 = 0.08;
/// TeX's `\delimiterfactor` (a delimiter covers this fraction of the content, in thousandths) and
/// `\delimitershortfall` (5pt at 10pt, in ems).
const DELIMITER_FACTOR: f64 = 0.901;
const DELIMITER_SHORTFALL_EM: f64 = 0.5;

/// Inter-atom spacing (TeXbook p. 170), rows = left atom, columns = right atom, both in the order
/// ord, op, bin, rel, open, close, punct, inner. 0: none; 1..=3: thin, medium, thick; 4..=6: the same
/// but only outside script styles; 9: cannot happen (a binary operator there has become an
/// ordinary atom).
const SPACING: [[u8; 8]; 8] = [
    [0, 1, 5, 6, 0, 0, 0, 4],
    [1, 1, 9, 6, 0, 0, 0, 4],
    [5, 5, 9, 9, 5, 9, 9, 5],
    [6, 6, 9, 0, 6, 0, 0, 6],
    [0, 0, 9, 0, 0, 0, 0, 0],
    [0, 1, 5, 6, 0, 0, 0, 4],
    [4, 4, 9, 4, 4, 4, 4, 4],
    [4, 1, 5, 6, 4, 0, 4, 4],
];

fn class_index(c: AtomClass) -> usize {
    match c {
        AtomClass::Ord => 0,
        AtomClass::Op => 1,
        AtomClass::Bin => 2,
        AtomClass::Rel => 3,
        AtomClass::Open => 4,
        AtomClass::Close => 5,
        AtomClass::Punct => 6,
        AtomClass::Inner => 7,
    }
}

/// TeX's four styles, and whether the formula is cramped (exponents sit lower).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Style {
    /// 0 display, 1 text, 2 script, 3 scriptscript.
    level: u8,
    cramped: bool,
}

impl Style {
    const DISPLAY: Style = Style { level: 0, cramped: false };
    const TEXT: Style = Style { level: 1, cramped: false };

    fn of(level: StyleLevel, cramped: bool) -> Style {
        let level = match level {
            StyleLevel::Display => 0,
            StyleLevel::Text => 1,
            StyleLevel::Script => 2,
            StyleLevel::ScriptScript => 3,
        };
        Style { level, cramped }
    }

    fn display(self) -> bool {
        self.level == 0
    }

    fn script(self) -> bool {
        self.level >= 2
    }

    fn cramp(self) -> Style {
        Style { cramped: true, ..self }
    }

    /// Superscripts and the like.
    fn sup(self) -> Style {
        Style { level: if self.level < 2 { 2 } else { 3 }, ..self }
    }

    /// Subscripts: cramped.
    fn sub(self) -> Style {
        self.sup().cramp()
    }

    fn num(self) -> Style {
        Style { level: (self.level + 1).min(3), ..self }
    }

    fn den(self) -> Style {
        self.num().cramp()
    }
}

/// What a node is set in: the style and the letters' font variant.
#[derive(Clone, Copy, Debug)]
struct Env {
    style: Style,
    variant: Variant,
}

/// A placed piece of a box, relative to the box's origin (left end of the baseline), y up.
#[derive(Clone, Debug)]
enum Item {
    Glyph { gid: u32, x: f64, y: f64, scale: f64 },
    Rule { x: f64, y: f64, w: f64, h: f64 },
}

/// A box: width, height above and depth below the baseline, and what is in it.
#[derive(Clone, Debug, Default)]
struct MBox {
    w: f64,
    h: f64,
    d: f64,
    /// Italic correction: how far the last glyph overhangs `w`.
    ic: f64,
    /// Where an accent over the box attaches (x), when the font says.
    attach: Option<f64>,
    /// The glyph, when the box is a single character (scripts attach to those differently).
    glyph: Option<u32>,
    items: Vec<Item>,
}

impl MBox {
    fn spacer(w: f64) -> MBox {
        MBox { w, ..MBox::default() }
    }

    /// Puts `child` in with its origin at (`dx`, `dy`), growing the box to hold it.
    fn place(&mut self, child: MBox, dx: f64, dy: f64) {
        self.w = self.w.max(dx + child.w);
        self.h = self.h.max(dy + child.h);
        self.d = self.d.max(child.d - dy);
        self.items.extend(child.items.into_iter().map(|item| match item {
            Item::Glyph { gid, x, y, scale } => Item::Glyph { gid, x: x + dx, y: y + dy, scale },
            Item::Rule { x, y, w, h } => Item::Rule { x: x + dx, y: y + dy, w, h },
        }));
    }

    /// Adds a filled rectangle with its bottom left corner at (`x`, `y`); the box grows to hold it.
    fn rule(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.w = self.w.max(x + w);
        self.h = self.h.max(y + h);
        self.d = self.d.max(-y);
        self.items.push(Item::Rule { x, y, w, h });
    }

    /// Moves the contents down by `by` (the baseline moves up relative to them).
    fn lower(&mut self, by: f64) {
        for item in &mut self.items {
            match item {
                Item::Glyph { y, .. } | Item::Rule { y, .. } => *y -= by,
            }
        }
        self.h -= by;
        self.d += by;
    }
}

/// A glyph, or glyphs assembled, as font-unit placements: (glyph, x, y) of each origin, y up,
/// relative to the construct's origin.
struct Construct {
    pieces: Vec<(u32, f64, f64)>,
    /// Advance width in font units.
    advance: f64,
    /// Ink bounds in font units, y up.
    ink: Rect,
}

struct Layouter<'a> {
    db: &'a FontDb,
    face: &'a FontFace,
    font: MathFont<'a>,
    /// Points per font unit at the formula's base size.
    unit: f64,
    /// The base size in points (the em).
    size: f64,
    script_scale: f64,
    script_script_scale: f64,
    glyphs: usize,
    /// Ink bounds per glyph, font units, y up.
    ink: HashMap<u32, Option<Rect>>,
}

/// Lays out `nodes` in `face`.
pub(crate) fn layout(db: &FontDb, face: &FontFace, nodes: &[Node], opt: MathOptions) -> Result<MathLayout, MathError> {
    if !(opt.size_pt.is_finite() && opt.size_pt > 0.0 && opt.size_pt <= MAX_SIZE_PT) {
        return Err(MathError::BadSize(opt.size_pt));
    }
    let font = MathFont::new(face).ok_or(MathError::NoMathFont)?;
    let upem = face.units_per_em();
    if !(upem.is_finite() && upem > 0.0) {
        return Err(MathError::BadMetrics);
    }
    let mut lay = Layouter {
        db,
        face,
        unit: opt.size_pt / upem,
        size: opt.size_pt,
        script_scale: font.script_percent_scale_down() / 100.0,
        script_script_scale: font.script_script_percent_scale_down() / 100.0,
        font,
        glyphs: 0,
        ink: HashMap::new(),
    };
    let env = Env { style: if opt.display { Style::DISPLAY } else { Style::TEXT }, variant: Variant::Math };
    let b = lay.row(nodes, env, None, None)?;
    lay.finish(b)
}

/// Classes of a row's nodes as spacing sees them: a binary operator with nothing to operate on
/// (first, or after an operator, relation, opening delimiter or punctuation; last, or before a
/// relation, closing delimiter or punctuation) is an ordinary atom. `lead` and `trail` are what
/// stands around the row, when something does.
fn effective_classes(nodes: &[Node], lead: Option<AtomClass>, trail: Option<AtomClass>) -> Vec<Option<AtomClass>> {
    let mut classes: Vec<Option<AtomClass>> = nodes.iter().map(Node::atom_class).collect();
    let mut prev: Option<(usize, AtomClass)> = lead.map(|c| (usize::MAX, c));
    for i in 0..classes.len() {
        let Some(mut class) = classes.get(i).copied().flatten() else { continue };
        if class == AtomClass::Bin
            && prev.is_none_or(|(_, p)| matches!(p, AtomClass::Bin | AtomClass::Op | AtomClass::Rel | AtomClass::Open | AtomClass::Punct))
        {
            class = AtomClass::Ord;
        } else if matches!(class, AtomClass::Rel | AtomClass::Close | AtomClass::Punct)
            && let Some((j, AtomClass::Bin)) = prev
            && let Some(slot) = classes.get_mut(j)
        {
            *slot = Some(AtomClass::Ord);
        }
        if let Some(slot) = classes.get_mut(i) {
            *slot = Some(class);
        }
        prev = Some((i, class));
    }
    if let Some((j, AtomClass::Bin)) = prev
        && trail.is_none_or(|t| matches!(t, AtomClass::Rel | AtomClass::Close | AtomClass::Punct))
        && let Some(slot) = classes.get_mut(j)
    {
        *slot = Some(AtomClass::Ord);
    }
    classes
}

impl Layouter<'_> {
    // --- units and parameters -------------------------------------------------------------

    fn scale(&self, st: Style) -> f64 {
        match st.level {
            0 | 1 => 1.0,
            2 => self.script_scale,
            _ => self.script_script_scale,
        }
    }

    /// Points per font unit in `st`.
    fn c(&self, st: Style) -> f64 {
        self.unit * self.scale(st)
    }

    /// The em of `st`, points.
    fn em(&self, st: Style) -> f64 {
        self.size * self.scale(st)
    }

    /// A `MATH` constant in points, at the size of `st`.
    fn k(&self, k: K, st: Style) -> f64 {
        self.font.constant(k) * self.c(st)
    }

    fn count(&mut self, n: usize) -> Result<(), MathError> {
        self.glyphs += n;
        if self.glyphs > MAX_GLYPHS {
            return Err(MathError::TooLarge { max: MAX_GLYPHS });
        }
        Ok(())
    }

    /// Ink bounds of glyph `gid` in font units, y up; `None` for a glyph without outline.
    fn ink_of(&mut self, gid: u32) -> Option<Rect> {
        if let Some(r) = self.ink.get(&gid) {
            return *r;
        }
        let outline = self.db.outline(self.face, gid);
        let r = (!outline.elements().is_empty()).then(|| {
            let b = vectorcraft_geom::Shape::bounding_box(&*outline);
            // The outline is y down.
            Rect::new(b.x0, -b.y1, b.x1, -b.y0)
        });
        self.ink.insert(gid, r);
        r
    }

    // --- glyph boxes ----------------------------------------------------------------------

    fn gid_of(&self, ch: char) -> Result<u32, MathError> {
        match self.face.glyph_for(ch) {
            0 => Err(MathError::MissingGlyph(ch)),
            g => Ok(g),
        }
    }

    /// The glyph for `ch` in `variant`, falling back to the plain character.
    fn styled_gid(&self, ch: char, variant: Variant) -> Result<u32, MathError> {
        match self.face.glyph_for(styled_char(ch, variant)) {
            0 => self.gid_of(ch),
            g => Ok(g),
        }
    }

    /// A box holding glyph `gid` at the size of `st`.
    fn glyph_box(&mut self, gid: u32, st: Style) -> Result<MBox, MathError> {
        self.count(1)?;
        let c = self.c(st);
        let (h, d) = self.ink_of(gid).map_or((0.0, 0.0), |r| (r.y1.max(0.0) * c, (-r.y0).max(0.0) * c));
        Ok(MBox {
            w: self.face.advance(gid) * c,
            h,
            d,
            ic: self.font.italic_correction(gid) * c,
            attach: self.font.top_accent_attachment(gid).map(|a| a * c),
            glyph: Some(gid),
            items: vec![Item::Glyph { gid, x: 0.0, y: 0.0, scale: c }],
        })
    }

    /// Upright text: glyph after glyph, no kerning; a space is a word space.
    fn text_run(&mut self, text: &str, st: Style) -> Result<MBox, MathError> {
        let c = self.c(st);
        let mut out = MBox::default();
        let mut x = 0.0;
        for ch in text.chars() {
            if ch == ' ' {
                x += WORD_SPACE_EM * self.em(st);
                continue;
            }
            let gid = self.gid_of(ch)?;
            let mut g = self.glyph_box(gid, st)?;
            g.glyph = None;
            let adv = self.face.advance(gid) * c;
            out.place(g, x, 0.0);
            x += adv;
        }
        out.w = x;
        Ok(out)
    }

    fn space_width(&self, s: Space, st: Style) -> f64 {
        let em = self.em(st);
        match s {
            Space::Thin => em * 3.0 / 18.0,
            Space::Medium => em * 4.0 / 18.0,
            Space::Thick => em * 5.0 / 18.0,
            Space::Quad => em,
            Space::QuadQuad => em * 2.0,
            Space::NegThin => -em * 3.0 / 18.0,
            Space::Normal => em * WORD_SPACE_EM,
        }
    }

    // --- rows -----------------------------------------------------------------------------

    /// Spacing between two atoms, points.
    fn spacing(&self, left: AtomClass, right: AtomClass, st: Style) -> f64 {
        let code = SPACING.get(class_index(left)).and_then(|row| row.get(class_index(right))).copied().unwrap_or(0);
        if (4..=6).contains(&code) && st.script() {
            return 0.0;
        }
        let mu = match code {
            1 | 4 => 3.0,
            2 | 5 => 4.0,
            3 | 6 => 5.0,
            _ => return 0.0,
        };
        mu * self.em(st) / 18.0
    }

    /// Sets `nodes` side by side with the spacing between atoms. `lead` and `trail` are the atoms
    /// around the row, when there are any (the delimiters of a `\left…\right`).
    fn row(&mut self, nodes: &[Node], env: Env, lead: Option<AtomClass>, trail: Option<AtomClass>) -> Result<MBox, MathError> {
        if let ([only], None, None) = (nodes, lead, trail) {
            return self.node(only, env);
        }
        let classes = effective_classes(nodes, lead, trail);
        let mut out = MBox::default();
        let mut x = 0.0;
        let mut prev = lead;
        for (node, class) in nodes.iter().zip(classes) {
            if let Some(class) = class {
                if let Some(p) = prev {
                    x += self.spacing(p, class, env.style);
                }
                prev = Some(class);
            }
            let b = self.node(node, env)?;
            let w = b.w;
            out.place(b, x, 0.0);
            x += w;
        }
        if let (Some(p), Some(t)) = (prev, trail) {
            x += self.spacing(p, t, env.style);
        }
        out.w = x;
        Ok(out)
    }

    fn node(&mut self, node: &Node, env: Env) -> Result<MBox, MathError> {
        match node {
            Node::Atom { ch, .. } => {
                let gid = self.styled_gid(*ch, env.variant)?;
                self.glyph_box(gid, env.style)
            }
            Node::Group(items) => self.row(items, env, None, None),
            Node::Scripts { base, sub, sup } => self.scripts(base, sub.as_deref(), sup.as_deref(), env),
            Node::Frac { num, den, style } => self.fraction(num, den, *style, env),
            Node::Sqrt { degree, body } => self.radical(degree.as_deref(), body, env),
            Node::Fenced { open, close, body } => self.fenced(*open, *close, body, env),
            Node::BigOp { ch, .. } => self.op_nucleus(*ch, env),
            Node::Func { name, .. } => self.text_run(name, env.style),
            Node::Accent { kind, body } => self.accent(*kind, body, env),
            Node::Wide { kind, body } => self.wide(*kind, body, env),
            Node::Space(s) => Ok(MBox::spacer(self.space_width(*s, env.style))),
            Node::Text(t) => self.text_run(t, env.style),
            Node::Styled { variant, body } => self.row(body, Env { variant: *variant, ..env }, None, None),
            Node::StyleScope { level, body } => self.row(body, Env { style: Style::of(*level, env.style.cramped), ..env }, None, None),
        }
    }

    // --- scripts and big operators --------------------------------------------------------

    /// A big operator's symbol: the display-size variant in display style, centred on the axis.
    fn op_nucleus(&mut self, ch: char, env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let mut gid = self.gid_of(ch)?;
        if st.display() {
            let min = self.font.display_operator_min_height();
            let variants = self.font.vertical_variants(gid);
            if let Some(v) = variants.iter().find(|v| v.advance >= min).or(variants.last()) {
                gid = v.gid;
            }
        }
        let mut b = self.glyph_box(gid, st)?;
        // Not a plain character: scripts attach to its box.
        b.glyph = None;
        let axis = self.k(K::AxisHeight, st);
        b.lower((b.h - b.d) / 2.0 - axis);
        Ok(b)
    }

    fn scripts(&mut self, base: &Node, sub: Option<&Node>, sup: Option<&Node>, env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let (nucleus, limits) = match base {
            Node::BigOp { ch, limits } => {
                let above_below = match limits {
                    Limits::Limits => true,
                    Limits::NoLimits => false,
                    Limits::Auto => st.display() && !is_integral(*ch),
                };
                (self.op_nucleus(*ch, env)?, above_below)
            }
            Node::Func { name, limits } => (self.text_run(name, st)?, *limits && st.display()),
            other => (self.node(other, env)?, false),
        };
        let sup_box = sup.map(|n| self.node(n, Env { style: st.sup(), ..env })).transpose()?;
        let sub_box = sub.map(|n| self.node(n, Env { style: st.sub(), ..env })).transpose()?;
        if limits { Ok(self.limits(nucleus, sub_box, sup_box, st)) } else { Ok(self.attach_scripts(nucleus, sub_box, sup_box, st)) }
    }

    /// Scripts above and below the nucleus, all centred (Appendix G rule 13a).
    fn limits(&self, nucleus: MBox, sub: Option<MBox>, sup: Option<MBox>, st: Style) -> MBox {
        let width = nucleus.w.max(sup.as_ref().map_or(0.0, |s| s.w)).max(sub.as_ref().map_or(0.0, |s| s.w));
        let delta = nucleus.ic;
        let (nh, nd, nw) = (nucleus.h, nucleus.d, nucleus.w);
        let mut out = MBox::default();
        out.place(nucleus, (width - nw) / 2.0, 0.0);
        if let Some(s) = sup {
            let rise = (self.k(K::UpperLimitGapMin, st) + s.d).max(self.k(K::UpperLimitBaselineRiseMin, st));
            let x = (width - s.w) / 2.0 + delta / 2.0;
            out.place(s, x, nh + rise);
        }
        if let Some(s) = sub {
            let drop = (self.k(K::LowerLimitGapMin, st) + s.h).max(self.k(K::LowerLimitBaselineDropMin, st));
            let x = (width - s.w) / 2.0 - delta / 2.0;
            out.place(s, x, -(nd + drop));
        }
        out.w = width;
        out
    }

    /// Scripts beside the nucleus (Appendix G rule 18).
    fn attach_scripts(&self, nucleus: MBox, sub: Option<MBox>, sup: Option<MBox>, st: Style) -> MBox {
        let c = self.c(st);
        let (mut u, mut v) = (0.0_f64, 0.0_f64);
        if nucleus.glyph.is_none() {
            // A box that isn't one character: scripts hang from its corners.
            u = nucleus.h - self.k(K::SuperscriptBaselineDropMax, st.sup());
            v = nucleus.d + self.k(K::SubscriptBaselineDropMin, st.sub());
        }
        let base_glyph = nucleus.glyph;
        let kern = |corner: KernCorner, height: f64| match base_glyph {
            Some(g) => self.font.math_kern(g, corner, height / c) * c,
            None => 0.0,
        };
        let shift_up = if st.cramped { K::SuperscriptShiftUpCramped } else { K::SuperscriptShiftUp };
        match (&sup, &sub) {
            (Some(p), None) => {
                u = u.max(self.k(shift_up, st)).max(p.d + self.k(K::SuperscriptBottomMin, st));
            }
            (None, Some(s)) => {
                v = v.max(self.k(K::SubscriptShiftDown, st)).max(s.h - self.k(K::SubscriptTopMax, st));
            }
            (Some(p), Some(s)) => {
                u = u.max(self.k(shift_up, st)).max(p.d + self.k(K::SuperscriptBottomMin, st));
                v = v.max(self.k(K::SubscriptShiftDown, st)).max(s.h - self.k(K::SubscriptTopMax, st));
                let gap = (u - p.d) - (s.h - v);
                let min_gap = self.k(K::SubSuperscriptGapMin, st);
                if gap < min_gap {
                    v += min_gap - gap;
                    let psi = self.k(K::SuperscriptBottomMaxWithSubscript, st) - (u - p.d);
                    if psi > 0.0 {
                        u += psi;
                        v -= psi;
                    }
                }
            }
            (None, None) => {}
        }
        let (base_w, ic) = (nucleus.w, nucleus.ic);
        let mut out = MBox::default();
        out.place(nucleus, 0.0, 0.0);
        let mut right = base_w;
        if let Some(p) = sup {
            let x = base_w + ic + kern(KernCorner::TopRight, u - p.d);
            right = right.max(x + p.w);
            out.place(p, x, u);
        }
        if let Some(s) = sub {
            let x = base_w + kern(KernCorner::BottomRight, s.h - v);
            right = right.max(x + s.w);
            out.place(s, x, -v);
        }
        out.w = right + self.k(K::SpaceAfterScript, st);
        out
    }

    // --- fractions, radicals, delimiters ---------------------------------------------------

    fn fraction(&mut self, num: &Node, den: &Node, style: FracStyle, env: Env) -> Result<MBox, MathError> {
        let fs = match style {
            FracStyle::Auto => env.style,
            FracStyle::Display => Style { level: 0, cramped: env.style.cramped },
            FracStyle::Text => Style { level: 1, cramped: env.style.cramped },
        };
        let n = self.node(num, Env { style: fs.num(), ..env })?;
        let d = self.node(den, Env { style: fs.den(), ..env })?;
        let axis = self.k(K::AxisHeight, fs);
        let theta = self.k(K::FractionRuleThickness, fs);
        let (mut u, mut v, num_gap, den_gap) = if fs.display() {
            (
                self.k(K::FractionNumeratorDisplayStyleShiftUp, fs),
                self.k(K::FractionDenominatorDisplayStyleShiftDown, fs),
                self.k(K::FractionNumDisplayStyleGapMin, fs),
                self.k(K::FractionDenomDisplayStyleGapMin, fs),
            )
        } else {
            (
                self.k(K::FractionNumeratorShiftUp, fs),
                self.k(K::FractionDenominatorShiftDown, fs),
                self.k(K::FractionNumeratorGapMin, fs),
                self.k(K::FractionDenominatorGapMin, fs),
            )
        };
        u = u.max(axis + theta / 2.0 + num_gap + n.d);
        v = v.max(den_gap + d.h - axis + theta / 2.0);
        let (nw, dw) = (n.w, d.w);
        let width = nw.max(dw);
        let pad = NULL_DELIMITER_EM * self.em(fs);
        let mut out = MBox::default();
        out.place(n, pad + (width - nw) / 2.0, u);
        out.place(d, pad + (width - dw) / 2.0, -v);
        if theta > 0.0 {
            self.count(1)?;
            out.rule(pad, axis - theta / 2.0, width, theta);
        }
        out.w = width + 2.0 * pad;
        Ok(out)
    }

    fn radical(&mut self, degree: Option<&Node>, body: &Node, env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let c = self.c(st);
        let b = self.node(body, Env { style: st.cramp(), ..env })?;
        let theta = self.k(K::RadicalRuleThickness, st);
        let gap0 = self.k(if st.display() { K::RadicalDisplayStyleVerticalGap } else { K::RadicalVerticalGap }, st);
        let extra_ascender = self.k(K::RadicalExtraAscender, st);
        let sign = self.gid_of('\u{221A}')?;
        let needed = b.h + b.d + gap0 + theta;
        let built = self.stretch(sign, true, needed / c)?;
        let ink = built.ink;
        let total = (ink.y1 - ink.y0) * c;
        // A sign taller than needed leaves the extra room above and below the body alike.
        let gap = gap0 + (total - needed).max(0.0) / 2.0;
        let rule_top = b.h + gap + theta;
        // The sign's top edge is the rule's.
        let sign_y = rule_top - ink.y1 * c;
        let sign_bottom = sign_y + ink.y0 * c;
        let mut out = MBox::default();
        let (mut sign_x, mut degree_top) = (0.0, 0.0_f64);
        if let Some(dn) = degree {
            let deg = self.node(dn, Env { style: Style::of(StyleLevel::ScriptScript, false), ..env })?;
            let before = self.k(K::RadicalKernBeforeDegree, st).max(0.0);
            let after = self.k(K::RadicalKernAfterDegree, st);
            let raise = self.font.radical_degree_bottom_raise_percent() / 100.0 * total;
            let y = sign_bottom + raise + deg.d;
            degree_top = y + deg.h;
            sign_x = (before + deg.w + after).max(0.0);
            out.place(deg, before, y);
        }
        self.count(built.pieces.len())?;
        for &(gid, px, py) in &built.pieces {
            out.items.push(Item::Glyph { gid, x: sign_x + px * c, y: sign_y + py * c, scale: c });
        }
        let sign_w = built.advance * c;
        let body_x = sign_x + sign_w;
        // The bar starts inside the sign's own bar, so they join without a seam.
        let bar_x = sign_x + sign_w.min(ink.x1 * c);
        out.h = out.h.max(sign_y + ink.y1 * c);
        out.d = out.d.max(-sign_bottom);
        out.w = out.w.max(sign_x + sign_w);
        let body_w = b.w;
        out.place(b, body_x, 0.0);
        self.count(1)?;
        out.rule(bar_x, rule_top - theta, body_x + body_w - bar_x, theta);
        out.h = out.h.max(rule_top + extra_ascender).max(degree_top);
        Ok(out)
    }

    fn fenced(&mut self, open: Option<char>, close: Option<char>, body: &[Node], env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let b = self.row(body, env, Some(AtomClass::Open), Some(AtomClass::Close))?;
        let axis = self.k(K::AxisHeight, st);
        let delta = (b.h - axis).max(b.d + axis);
        let target = (delta * 2.0 * DELIMITER_FACTOR).max(delta * 2.0 - DELIMITER_SHORTFALL_EM * self.em(st));
        let left = self.delimiter(open, target, st)?;
        let right = self.delimiter(close, target, st)?;
        let mut out = MBox::default();
        let (lw, bw) = (left.w, b.w);
        out.place(left, 0.0, 0.0);
        out.place(b, lw, 0.0);
        out.place(right, lw + bw, 0.0);
        Ok(out)
    }

    /// A delimiter at least `target` points tall, centred on the axis; `None` is the empty
    /// delimiter (a little space).
    fn delimiter(&mut self, ch: Option<char>, target: f64, st: Style) -> Result<MBox, MathError> {
        let Some(ch) = ch else { return Ok(MBox::spacer(NULL_DELIMITER_EM * self.em(st))) };
        let c = self.c(st);
        let gid = self.gid_of(ch)?;
        let built = self.stretch(gid, true, target / c)?;
        self.count(built.pieces.len())?;
        let ink = built.ink;
        // Ink centred on the axis.
        let axis = self.k(K::AxisHeight, st);
        let dy = axis - (ink.y0 + ink.y1) / 2.0 * c;
        let mut out = MBox::default();
        for &(g, px, py) in &built.pieces {
            out.items.push(Item::Glyph { gid: g, x: px * c, y: dy + py * c, scale: c });
        }
        out.w = built.advance * c;
        out.h = (dy + ink.y1 * c).max(0.0);
        out.d = (-(dy + ink.y0 * c)).max(0.0);
        Ok(out)
    }

    // --- accents and wide marks -----------------------------------------------------------

    /// The glyph for an accent: the first of `candidates` the font has, and whether it is the
    /// first choice (a combining mark, drawn at the height the font designed for accents).
    fn first_glyph(&self, candidates: &[char]) -> Result<(u32, bool), MathError> {
        for (i, &ch) in candidates.iter().enumerate() {
            let g = self.face.glyph_for(ch);
            if g != 0 {
                return Ok((g, i == 0));
            }
        }
        Err(MathError::MissingGlyph(candidates.first().copied().unwrap_or('\u{FFFD}')))
    }

    fn accent(&mut self, kind: AccentKind, body: &Node, env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let b = self.node(body, Env { style: st.cramp(), ..env })?;
        let candidates: &[char] = match kind {
            AccentKind::Hat => &['\u{302}', '\u{2C6}', '^'],
            AccentKind::Bar => &['\u{304}', '\u{AF}'],
            AccentKind::Tilde => &['\u{303}', '\u{2DC}', '~'],
            AccentKind::Vec => &['\u{20D7}', '\u{2192}'],
            AccentKind::Dot => &['\u{307}', '\u{2D9}'],
            AccentKind::Ddot => &['\u{308}', '\u{A8}'],
        };
        let (gid, combining) = self.first_glyph(candidates)?;
        let accent = Construct { pieces: vec![(gid, 0.0, 0.0)], advance: self.face.advance(gid), ink: self.ink_of(gid).unwrap_or(Rect::ZERO) };
        let c = self.c(st);
        // The accent's own attachment point goes over the base's; failing the font's word on
        // it, the middle of each.
        let accent_at = self.font.top_accent_attachment(gid).unwrap_or((accent.ink.x0 + accent.ink.x1) / 2.0) * c;
        let base_at = b.attach.unwrap_or(b.w / 2.0);
        self.put_over(b, &accent, base_at - accent_at, combining, st)
    }

    fn wide(&mut self, kind: WideKind, body: &Node, env: Env) -> Result<MBox, MathError> {
        let st = env.style;
        let b = self.node(body, Env { style: st.cramp(), ..env })?;
        match kind {
            WideKind::Overline => {
                let (gap, theta, extra) =
                    (self.k(K::OverbarVerticalGap, st), self.k(K::OverbarRuleThickness, st), self.k(K::OverbarExtraAscender, st));
                let (h, w) = (b.h, b.w);
                let mut out = MBox::default();
                out.place(b, 0.0, 0.0);
                self.count(1)?;
                out.rule(0.0, h + gap, w, theta);
                out.h = h + gap + theta + extra;
                out.w = w;
                Ok(out)
            }
            WideKind::Underline => {
                let (gap, theta, extra) =
                    (self.k(K::UnderbarVerticalGap, st), self.k(K::UnderbarRuleThickness, st), self.k(K::UnderbarExtraDescender, st));
                let (d, w) = (b.d, b.w);
                let mut out = MBox::default();
                out.place(b, 0.0, 0.0);
                self.count(1)?;
                out.rule(0.0, -(d + gap + theta), w, theta);
                out.d = d + gap + theta + extra;
                out.w = w;
                Ok(out)
            }
            WideKind::OverRightArrow | WideKind::OverLeftArrow => {
                let candidates: &[char] = if kind == WideKind::OverRightArrow { &['\u{20D7}', '\u{2192}'] } else { &['\u{20D6}', '\u{2190}'] };
                let (gid, combining) = self.first_glyph(candidates)?;
                let c = self.c(st);
                let arrow = self.stretch(gid, false, b.w / c)?;
                // Centred over the body by its ink.
                let dx = (b.w - (arrow.ink.x1 - arrow.ink.x0) * c) / 2.0 - arrow.ink.x0 * c;
                self.put_over(b, &arrow, dx, combining, st)
            }
        }
    }

    /// Puts `mark` over `b`, its origin `dx` to the right of `b`'s. A combining mark sits where the
    /// font drew it, raised when the base is taller than the font allows for; any other glyph sits
    /// just above the base by its ink.
    fn put_over(&mut self, b: MBox, mark: &Construct, dx: f64, natural: bool, st: Style) -> Result<MBox, MathError> {
        let c = self.c(st);
        self.count(mark.pieces.len())?;
        let dy = if natural { (b.h - self.k(K::AccentBaseHeight, st)).max(0.0) } else { b.h + LOOSE_ACCENT_GAP_EM * self.em(st) - mark.ink.y0 * c };
        let (w, h) = (b.w, b.h);
        let mut out = MBox::default();
        out.place(b, 0.0, 0.0);
        for &(gid, px, py) in &mark.pieces {
            out.items.push(Item::Glyph { gid, x: dx + px * c, y: dy + py * c, scale: c });
        }
        out.h = h.max(dy + mark.ink.y1 * c);
        out.w = w;
        Ok(out)
    }

    // --- growing glyphs -------------------------------------------------------------------

    fn single(&mut self, gid: u32) -> Construct {
        let advance = self.face.advance(gid);
        let ink = self.ink_of(gid).unwrap_or(Rect::new(0.0, 0.0, advance, 0.0));
        Construct { pieces: vec![(gid, 0.0, 0.0)], advance, ink }
    }

    /// Glyph `gid` grown to `target` font units along its direction: the smallest variant that
    /// reaches it, else an assembly of parts, else the largest variant.
    fn stretch(&mut self, gid: u32, vertical: bool, target: f64) -> Result<Construct, MathError> {
        let variants = if vertical { self.font.vertical_variants(gid) } else { self.font.horizontal_variants(gid) };
        if !target.is_finite() {
            return Ok(self.single(gid));
        }
        if let Some(v) = variants.iter().find(|v| v.advance >= target) {
            return Ok(self.single(v.gid));
        }
        let assembly = if vertical { self.font.vertical_assembly(gid) } else { self.font.horizontal_assembly(gid) };
        if let Some(a) = assembly
            && let Some(built) = self.assemble(&a, target, vertical)?
        {
            return Ok(built);
        }
        Ok(self.single(variants.last().map_or(gid, |v| v.gid)))
    }

    /// Builds a glyph of length `target` (font units) from the parts of `asm`: extenders repeated
    /// until it is long enough, the joints overlapped to make it just `target`.
    fn assemble(&mut self, asm: &GlyphAssembly, target: f64, vertical: bool) -> Result<Option<Construct>, MathError> {
        let min_overlap = self.font.min_connector_overlap();
        let has_extender = asm.parts.iter().any(|p| p.extender);
        let mut repeats = 1;
        let seq: Vec<&AssemblyPart> = loop {
            let seq: Vec<&AssemblyPart> = asm.parts.iter().flat_map(|p| std::iter::repeat_n(p, if p.extender { repeats } else { 1 })).collect();
            let sum: f64 = seq.iter().map(|p| p.full_advance).sum();
            let longest = sum - min_overlap * seq.len().saturating_sub(1) as f64;
            if longest >= target || !has_extender || repeats >= MAX_EXTENDER_REPEATS {
                break seq;
            }
            repeats += 1;
        };
        if seq.is_empty() {
            return Ok(None);
        }
        self.count(seq.len())?;
        let sum: f64 = seq.iter().map(|p| p.full_advance).sum();
        let joints = seq.len() - 1;
        let overlap = if joints == 0 {
            0.0
        } else {
            let max_overlap =
                seq.windows(2).filter_map(|w| Some(w.first()?.end_connector.min(w.get(1)?.start_connector))).fold(f64::INFINITY, f64::min);
            // Not `clamp`: a damaged font may give a maximum below the minimum.
            ((sum - target) / joints as f64).max(min_overlap).min(max_overlap.max(min_overlap))
        };
        let mut pieces = Vec::with_capacity(seq.len());
        let (mut at, mut advance) = (0.0, 0.0_f64);
        let mut ink: Option<Rect> = None;
        for part in seq {
            let (px, py) = if vertical { (0.0, at) } else { (at, 0.0) };
            pieces.push((part.gid, px, py));
            if let Some(r) = self.ink_of(part.gid) {
                let r = Rect::new(r.x0 + px, r.y0 + py, r.x1 + px, r.y1 + py);
                ink = Some(ink.map_or(r, |i| i.union(r)));
            }
            advance = if vertical { advance.max(self.face.advance(part.gid)) } else { at + part.full_advance };
            at += part.full_advance - overlap;
        }
        Ok(Some(Construct { pieces, advance, ink: ink.unwrap_or(Rect::ZERO) }))
    }

    // --- output ---------------------------------------------------------------------------

    /// Flips the finished box into layout space (y down, the ink's top at 0).
    fn finish(&mut self, b: MBox) -> Result<MathLayout, MathError> {
        let mut ink: Option<Rect> = None;
        for item in &b.items {
            let r = match *item {
                Item::Glyph { gid, x, y, scale } => {
                    let Some(g) = self.ink_of(gid) else { continue };
                    Rect::new(x + g.x0 * scale, y + g.y0 * scale, x + g.x1 * scale, y + g.y1 * scale)
                }
                Item::Rule { x, y, w, h } => Rect::new(x, y, x + w, y + h),
            };
            ink = Some(ink.map_or(r, |i| i.union(r)));
        }
        let ink = ink.ok_or(MathError::Empty)?;
        if ![ink.x0, ink.y0, ink.x1, ink.y1].iter().all(|v| v.is_finite()) {
            return Err(MathError::BadMetrics);
        }
        let top = ink.y1;
        let face_id = self.face.id();
        let items = b
            .items
            .iter()
            .map(|item| match *item {
                Item::Glyph { gid, x, y, scale } => MathItem::Glyph { font_id: face_id, gid, xf: Affine::new([scale, 0.0, 0.0, scale, x, top - y]) },
                Item::Rule { x, y, w, h } => MathItem::Rule { rect: Rect::new(x, top - (y + h), x + w, top - y) },
            })
            .collect();
        Ok(MathLayout { items, bounds: Rect::new(ink.x0, 0.0, ink.x1, top - ink.y0), baseline: top })
    }
}
