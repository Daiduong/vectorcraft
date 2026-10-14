//! Layout tests. They need a font with a MATH table, which the repository doesn't ship (fonts live
//! in craft-fonts): point `VECTORCRAFT_MATH_TEST_FONT` at one (STIX Two Math, New Computer Modern
//! Math, Latin Modern Math…) to run them; without it they skip, so CI without fonts passes.
//!
//! `VECTORCRAFT_MATH_DUMP_DIR=<dir>` also writes each formula the tests set as an SVG there, for a
//! look at the result.

use std::sync::Arc;

use vectorcraft_geom::{Rect, Shape};
use vectorcraft_text::{FontDb, FontFace};

use super::*;

const FONT_ENV: &str = "VECTORCRAFT_MATH_TEST_FONT";

/// The test font in a database of its own, or `None` (the test skips) when none is configured.
fn test_font() -> Option<(FontDb, Arc<FontFace>)> {
    let path = std::env::var_os(FONT_ENV)?;
    let bytes = std::fs::read(path).ok()?;
    let db = FontDb::with_font_dirs(vec![]);
    if db.add_font(bytes) == 0 {
        return None;
    }
    let face = find_math_face(&db)?;
    Some((db, face))
}

macro_rules! font_or_skip {
    () => {
        match test_font() {
            Some(f) => f,
            None => {
                eprintln!("skipped: set {FONT_ENV} to a font with a MATH table to run it");
                return;
            }
        }
    };
}

fn set(db: &FontDb, face: &FontFace, src: &str, display: bool) -> MathLayout {
    let layout = typeset(db, face, src, MathOptions { size_pt: 100.0, display }).unwrap_or_else(|e| panic!("{src}: {e}"));
    dump(db, &layout, src);
    layout
}

fn dump(db: &FontDb, layout: &MathLayout, src: &str) {
    let Some(dir) = std::env::var_os("VECTORCRAFT_MATH_DUMP_DIR") else { return };
    let paths: String = layout.to_paths(db).iter().map(|p| format!("<path d=\"{}\"/>", p.to_svg())).collect();
    let pad = 20.0;
    let b = layout.bounds;
    let (x, y, w, h) = (b.x0 - pad, b.y0 - pad, b.width() + 2.0 * pad, b.height() + 2.0 * pad);
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"{x} {y} {w} {h}\"><rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"white\"/>{paths}</svg>"
    );
    let name: String = src.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).take(40).collect();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(std::path::Path::new(&dir).join(format!("{name}.svg")), svg);
}

fn rules(l: &MathLayout) -> Vec<Rect> {
    l.items
        .iter()
        .filter_map(|i| match i {
            MathItem::Rule { rect } => Some(*rect),
            _ => None,
        })
        .collect()
}

fn glyphs(l: &MathLayout) -> usize {
    l.items.iter().filter(|i| matches!(i, MathItem::Glyph { .. })).count()
}

#[test]
fn the_guide_example_is_set_with_a_radical_a_fraction_bar_and_a_function_name() {
    let (db, face) = font_or_skip!();
    let l = set(&db, &face, r"y=\sqrt{x}+\dfrac{1}{\sin x}", false);
    // y = √ x + 1 s i n x, and the radical's sign.
    assert!(glyphs(&l) >= 10, "{} glyphs", glyphs(&l));
    let bars = rules(&l);
    // The radical's rule and the fraction bar.
    assert_eq!(bars.len(), 2, "{bars:?}");
    assert!(bars.iter().all(|r| r.width() > 0.0 && r.height() > 0.0));
    // About an em of height per line of a 100pt formula, and wider than it is tall.
    assert!(l.bounds.height() > 100.0 && l.bounds.height() < 400.0, "{:?}", l.bounds);
    assert!(l.bounds.width() > 300.0 && l.bounds.width() < 1200.0, "{:?}", l.bounds);
    assert_eq!(l.bounds.y0, 0.0);
    assert!(l.baseline > 0.0 && l.baseline < l.bounds.y1, "baseline {} in {:?}", l.baseline, l.bounds);
    // The fraction bar sits above the baseline (on the math axis), in the formula's middle third.
    let bar = bars.iter().min_by(|a, b| b.width().total_cmp(&a.width())).unwrap();
    assert!(bar.y0 < l.baseline && l.baseline - bar.y1 < 60.0, "bar {bar:?} baseline {}", l.baseline);
    let paths = l.to_paths(&db);
    assert_eq!(paths.len(), l.items.len().min(paths.len()));
    for p in &paths {
        let b = p.bounding_box();
        assert!(
            b.x0 >= l.bounds.x0 - 1e-6 && b.x1 <= l.bounds.x1 + 1e-6 && b.y0 >= -1e-6 && b.y1 <= l.bounds.y1 + 1e-6,
            "{b:?} outside {:?}",
            l.bounds
        );
    }
}

#[test]
fn a_larger_size_scales_the_layout() {
    let (db, face) = font_or_skip!();
    let small = typeset(&db, &face, r"\frac{a+b}{c}", MathOptions { size_pt: 10.0, display: false }).unwrap();
    let big = typeset(&db, &face, r"\frac{a+b}{c}", MathOptions { size_pt: 40.0, display: false }).unwrap();
    assert!((big.bounds.width() / small.bounds.width() - 4.0).abs() < 1e-6);
    assert!((big.bounds.height() / small.bounds.height() - 4.0).abs() < 1e-6);
    assert!((big.baseline / small.baseline - 4.0).abs() < 1e-6);
}

#[test]
fn superscripts_rise_and_subscripts_fall() {
    let (db, face) = font_or_skip!();
    let plain = set(&db, &face, "x", false);
    let sup = set(&db, &face, "x^2", false);
    let sub = set(&db, &face, "x_2", false);
    let both = set(&db, &face, "x_2^2", false);
    // Above and below the plain x, and wider.
    assert!(sup.baseline > plain.baseline, "{} {}", sup.baseline, plain.baseline);
    assert!(sup.bounds.height() > plain.bounds.height());
    assert!(sub.bounds.height() - sub.baseline > plain.bounds.height() - plain.baseline);
    assert!(sup.bounds.width() > plain.bounds.width() && sub.bounds.width() > plain.bounds.width());
    // Both scripts: the same column, with the exponent at least as high as alone.
    assert!(both.baseline >= sup.baseline - 1e-6);
    assert!(both.bounds.height() - both.baseline >= sub.bounds.height() - sub.baseline - 1e-6);
}

#[test]
fn scripts_are_set_smaller() {
    let (db, face) = font_or_skip!();
    let x = set(&db, &face, "x", false);
    let sup = set(&db, &face, "x^x", false);
    let (MathItem::Glyph { xf: base, .. }, MathItem::Glyph { xf: script, .. }) = (&sup.items[0], &sup.items[1]) else { panic!("two glyphs") };
    let scale = |a: &vectorcraft_geom::Affine| a.as_coeffs()[0];
    assert!(scale(script) < scale(base) * 0.9, "{} {}", scale(script), scale(base));
    assert!((scale(base) - 100.0 / face.units_per_em()).abs() < 1e-9);
    let _ = x;
}

#[test]
fn display_style_enlarges_operators_and_puts_limits_above_and_below() {
    let (db, face) = font_or_skip!();
    let text = set(&db, &face, r"\sum_{i=1}^n i", false);
    let display = set(&db, &face, r"\sum_{i=1}^n i", true);
    // Limits stack in display style: taller, narrower than the scripts beside the sign.
    assert!(display.bounds.height() > text.bounds.height());
    assert!(display.bounds.width() < text.bounds.width(), "{} vs {}", display.bounds.width(), text.bounds.width());
    // The integral keeps its scripts beside it even in display style.
    let int = set(&db, &face, r"\int_0^1 f", true);
    let int_limits = set(&db, &face, r"\int\limits_0^1 f", true);
    assert!(int.bounds.width() > int_limits.bounds.width(), "{} vs {}", int.bounds.width(), int_limits.bounds.width());
    // The display sign is bigger than the text one.
    let s_text = set(&db, &face, r"\sum", false);
    let s_disp = set(&db, &face, r"\sum", true);
    assert!(s_disp.bounds.height() > s_text.bounds.height() * 1.1);
}

#[test]
fn text_style_fractions_are_smaller_than_display_ones() {
    let (db, face) = font_or_skip!();
    let d = set(&db, &face, r"\dfrac{a+b}{c+d}", false);
    let t = set(&db, &face, r"\tfrac{a+b}{c+d}", true);
    assert!(d.bounds.height() > t.bounds.height());
    assert_eq!(rules(&d).len(), 1);
    assert_eq!(rules(&t).len(), 1);
    // The bar spans the wider of numerator and denominator.
    assert!(rules(&d)[0].width() > 100.0);
}

#[test]
fn a_radical_covers_its_content_and_grows_with_it() {
    let (db, face) = font_or_skip!();
    let small = set(&db, &face, r"\sqrt{x}", false);
    let tall = set(&db, &face, r"\sqrt{\dfrac{a}{b}}", false);
    assert_eq!(rules(&small).len(), 1);
    // The bar is the top of the formula (plus the sign's own ink at the same height).
    let bar = rules(&small)[0];
    assert!(bar.y0 < 5.0, "{bar:?}");
    assert!(tall.bounds.height() > small.bounds.height() * 1.5);
    let rooted = set(&db, &face, r"\sqrt[3]{x}", false);
    // The index sits in the crook of the sign (in some fonts it widens the root, in others not).
    assert!(rooted.bounds.width() >= small.bounds.width());
    assert!(rooted.bounds.height() >= small.bounds.height());
    // The root of a root is deeper still.
    let nested = set(&db, &face, r"\sqrt{1+\sqrt{x}}", false);
    assert_eq!(rules(&nested).len(), 2);
}

#[test]
fn delimiters_grow_to_cover_what_they_enclose() {
    let (db, face) = font_or_skip!();
    let small = set(&db, &face, r"\left(x\right)", false);
    let tall = set(&db, &face, r"\left(\dfrac{a}{b}\right)", true);
    let taller = set(&db, &face, r"\left[\dfrac{\dfrac{a}{b}}{\dfrac{c}{d}}\right]", true);
    assert!(tall.bounds.height() > small.bounds.height());
    assert!(taller.bounds.height() > tall.bounds.height());
    // A fixed one-glyph size for a small content, glyphs (or pieces) at the ends of a tall one.
    assert_eq!(glyphs(&small), 3);
    // A parenthesis tall enough for the fraction contains it.
    let fraction_bar = rules(&tall)[0];
    let (MathItem::Glyph { .. }, true) = (&tall.items[0], true) else { panic!() };
    assert!(fraction_bar.y0 > tall.bounds.y0 && fraction_bar.y1 < tall.bounds.y1);
    // Very tall content is covered by an assembly of parts, not one scaled glyph.
    let huge = set(&db, &face, r"\left\{\dfrac{\dfrac{\dfrac{a}{b}}{\dfrac{c}{d}}}{\dfrac{\dfrac{a}{b}}{\dfrac{c}{d}}}\right.", true);
    assert!(huge.bounds.height() > taller.bounds.height());
    assert!(glyphs(&huge) > 8, "{} glyphs", glyphs(&huge));
    // The null delimiter takes no glyph.
    assert!(glyphs(&huge) > glyphs(&set(&db, &face, r"\left.\dfrac{a}{b}\right.", true)));
}

#[test]
fn accents_sit_over_their_base_and_wide_marks_span_it() {
    let (db, face) = font_or_skip!();
    let x = set(&db, &face, "x", false);
    for cmd in [r"\hat", r"\bar", r"\tilde", r"\vec", r"\dot", r"\ddot"] {
        let a = set(&db, &face, &format!("{cmd}{{x}}"), false);
        assert_eq!(glyphs(&a), 2, "{cmd}");
        assert!(a.bounds.height() > x.bounds.height(), "{cmd} adds height");
    }
    // The accent's ink is over the letter: its x range overlaps the letter's.
    let a = set(&db, &face, r"\hat{x}", false);
    let boxes: Vec<Rect> = a.to_paths(&db).iter().map(|p| p.bounding_box()).collect();
    assert_eq!(boxes.len(), 2);
    let (lo, hi) = (boxes[0].x0.max(boxes[1].x0), boxes[0].x1.min(boxes[1].x1));
    assert!(hi - lo > 0.3 * boxes[1].width().min(boxes[0].width()), "{boxes:?}");
    // \overline and \underline are rules the width of the body.
    let over = set(&db, &face, r"\overline{abc}", false);
    let under = set(&db, &face, r"\underline{abc}", false);
    let plain = set(&db, &face, "abc", false);
    assert_eq!(rules(&over).len(), 1);
    assert_eq!(rules(&under).len(), 1);
    assert!(over.bounds.height() > plain.bounds.height());
    assert!(under.bounds.height() - under.baseline > plain.bounds.height() - plain.baseline);
    assert!((rules(&over)[0].width() - rules(&under)[0].width()).abs() < 1e-6);
    // The arrows stretch to the width of what they cover.
    let short = set(&db, &face, r"\overrightarrow{A}", false);
    let long = set(&db, &face, r"\overrightarrow{ABCDEFGH}", false);
    let (ws, wl) = (short.bounds.width(), long.bounds.width());
    assert!(wl > 3.0 * ws, "{ws} {wl}");
    let left = set(&db, &face, r"\overleftarrow{ABCDEFGH}", false);
    assert!((left.bounds.width() - wl).abs() / wl < 0.2);
}

#[test]
fn atoms_are_spaced_by_their_types() {
    let (db, face) = font_or_skip!();
    let width = |s: &str| set(&db, &face, s, false).bounds.width();
    // A relation is wider than the same symbols as ordinary atoms, a binary operator too.
    assert!(width("a=b") > width("a{=}b") + 10.0);
    assert!(width("a+b") > width("a{+}b") + 5.0);
    // A unary minus gets no space: -b is set like an ordinary atom.
    assert!((width("-b") - width("{-}b")).abs() < 1e-6);
    // After a comma, a thin space.
    assert!(width("a,b") > width("a{,}b") + 2.0);
    // No spacing in scripts.
    assert!((width("x^{a+b}") - width("x^{a{+}b}")).abs() < 1e-6);
    // Explicit spaces.
    let (a, b) = (width("ab"), width(r"a\quad b"));
    assert!(b - a > 90.0 && b - a < 110.0, "{a} {b}");
    assert!(width(r"a\qquad b") - a > 190.0);
    assert!(width(r"a\!b") < a);
}

#[test]
fn text_and_styles_are_set_upright_and_bold() {
    let (db, face) = font_or_skip!();
    let t = set(&db, &face, r"\text{if } x", false);
    // i f, a space, x.
    assert_eq!(glyphs(&t), 3);
    let it = set(&db, &face, "ab", false);
    let rm = set(&db, &face, r"\mathrm{ab}", false);
    let bf = set(&db, &face, r"\mathbf{ab}", false);
    let bm = set(&db, &face, r"\bm{ab}", false);
    let mi = set(&db, &face, r"\mathit{ab}", false);
    assert!(rm.bounds != it.bounds && bf.bounds != rm.bounds && bm.bounds != bf.bounds);
    assert!((mi.bounds.width() - it.bounds.width()).abs() < 1e-6);
    // Function names are upright text.
    let sin = set(&db, &face, r"\sin", false);
    let rm_sin = set(&db, &face, r"\mathrm{sin}", false);
    assert!((sin.bounds.width() - rm_sin.bounds.width()).abs() < 1e-6);
}

#[test]
fn italic_correction_moves_a_superscript_off_a_slanted_letter() {
    let (db, face) = font_or_skip!();
    // f is a letter with a strong italic correction.
    let l = set(&db, &face, "f^2", false);
    let (MathItem::Glyph { xf: base, gid, .. }, MathItem::Glyph { xf: sup, .. }) = (&l.items[0], &l.items[1]) else { panic!("two glyphs") };
    let adv = face.advance(*gid) * base.as_coeffs()[0];
    let gap = sup.translation().x - base.translation().x;
    assert!(gap >= adv - 1e-6, "the exponent starts at or after the advance: {gap} vs {adv}");
}

#[test]
fn a_formula_in_greek_and_symbols_is_set() {
    let (db, face) = font_or_skip!();
    let l = set(&db, &face, r"\alpha+\beta\le\Omega\cdot\infty\pm\partial\nabla\to\in\emptyset\forall\exists", true);
    assert_eq!(glyphs(&l), 15);
    let all = set(&db, &face, r"\Gamma\Delta\Theta\Lambda\Xi\Pi\Sigma\Upsilon\Phi\Psi\Omega\varepsilon\vartheta\varphi\epsilon\phi\varrho", false);
    assert_eq!(glyphs(&all), 17);
    let l = set(&db, &face, r"\ldots\cdots\vdots\ddots\angle\perp\parallel\degree\prime\circ\bullet\neg\land\lor", false);
    assert_eq!(glyphs(&l), 14);
}

#[test]
fn a_representative_formula_set() {
    let (db, face) = font_or_skip!();
    let formulas = [
        r"e^{i\pi}+1=0",
        r"\int_{-\infty}^{\infty}e^{-x^2}\,dx=\sqrt{\pi}",
        r"\lim_{n\to\infty}\left(1+\frac{1}{n}\right)^n=e",
        r"\sum_{k=1}^{n}k=\frac{n(n+1)}{2}",
        r"\oint_C \vec{F}\cdot d\vec{r}",
        r"\frac{\partial f}{\partial x}",
        r"\max_{x\in A}\hat{f}(x)\ge\bar{y}",
        r"x'+f''(x)",
        r"\prod_{i=1}^{n}\left[a_i+b_i\right]",
    ];
    for f in formulas {
        let l = set(&db, &face, f, true);
        assert!(!l.items.is_empty() && l.bounds.width() > 0.0 && l.bounds.height() > 0.0, "{f}");
        assert!(l.bounds.x1.is_finite() && l.bounds.y1.is_finite());
    }
}

#[test]
fn the_glyph_budget_and_bad_input_are_errors() {
    let (db, face) = font_or_skip!();
    let opt = MathOptions::default();
    assert_eq!(typeset(&db, &face, "", opt), Err(MathError::Empty));
    assert_eq!(typeset(&db, &face, r"\,\;", opt), Err(MathError::Empty));
    assert_eq!(typeset(&db, &face, &"a".repeat(layout::MAX_GLYPHS + 1), opt), Err(MathError::TooLarge { max: layout::MAX_GLYPHS }));
    assert!(typeset(&db, &face, &"a".repeat(layout::MAX_GLYPHS), opt).is_ok());
    assert!(matches!(typeset(&db, &face, r"\foo", opt), Err(MathError::UnknownCommand(c)) if c == "foo"));
    for bad in [0.0, -3.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(typeset(&db, &face, "x", MathOptions { size_pt: bad, display: false }), Err(MathError::BadSize(_))));
    }
    // A damaged formula never panics, whatever the combination.
    for src in [r"\left(", r"\frac{", r"x^", r"\sqrt[", r"}{", r"\left. \right", "^_", r"\text{\foo}"] {
        assert!(typeset(&db, &face, src, opt).is_err(), "{src}");
    }
}

#[test]
fn the_font_is_found_by_find_math_face_and_a_text_font_is_refused() {
    let (db, face) = font_or_skip!();
    assert!(mathfont::has_math_table(&face));
    assert!(mathfont::MathFont::new(&face).is_some_and(|m| m.constant(mathfont::MathConstant::AxisHeight) > 0.0));
    // The bundled UI font has no MATH table: not a math font.
    let inter = FontDb::with_font_dirs(vec![]);
    let text_face = inter.face("Inter", "Regular").unwrap();
    assert!(!mathfont::has_math_table(&text_face));
    assert_eq!(typeset(&inter, &text_face, "x", MathOptions::default()), Err(MathError::NoMathFont));
    // A database without a math font finds none.
    assert!(find_math_face(&inter).is_none());
    let _ = db;
}

#[test]
fn junk_formulas_never_panic_and_always_give_finite_layouts() {
    let (db, face) = font_or_skip!();
    let frags = [
        r"\frac",
        r"\sqrt",
        "[",
        "]",
        "{",
        "}",
        "^",
        "_",
        r"\left",
        r"\right",
        "(",
        ")",
        "|",
        r"\text{",
        "x",
        "1",
        " ",
        r"\sum",
        r"\int",
        r"\lim",
        r"\hat",
        r"\overline",
        r"\overrightarrow",
        r"\dfrac",
        r"\displaystyle",
        r"\scriptscriptstyle",
        "'",
        r"\alpha",
        r"\left(",
        r"\right)",
        r"\left.",
        r"\right.",
        "+",
        "=",
        "-",
        r"\,",
    ];
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let (mut ok, mut failed) = (0, 0);
    for round in 0..3000 {
        let src: String = (0..1 + next() % 14).map(|_| frags[next() % frags.len()]).collect();
        let display = round % 2 == 0;
        match typeset(&db, &face, &src, MathOptions { size_pt: 24.0, display }) {
            Ok(l) => {
                ok += 1;
                assert!(l.bounds.x0.is_finite() && l.bounds.x1.is_finite() && l.bounds.y1.is_finite() && l.baseline.is_finite(), "{src}");
                assert_eq!(l.bounds.y0, 0.0, "{src}");
                assert!(l.items.len() <= layout::MAX_GLYPHS, "{src}");
            }
            Err(_) => failed += 1,
        }
    }
    // Both kinds happen: the sweep exercises layout, not only the parser's errors.
    assert!(ok > 100 && failed > 100, "{ok} formulas set, {failed} refused");
}

#[test]
fn deeply_nested_but_valid_formulas_stay_within_budget() {
    let (db, face) = font_or_skip!();
    // Fractions nested to the depth limit, left-right pairs and script towers: each either sets or
    // is refused cleanly, quickly.
    let n = parser::MAX_DEPTH / 2 - 2;
    let fracs = format!("{}x{}", r"\frac{1}{".repeat(n), "}".repeat(n));
    let fenced = format!("{}x{}", r"\left(".repeat(n), r"\right)".repeat(n));
    let tower = format!("x{}{}", "^{x".repeat(n), "}".repeat(n));
    let started = std::time::Instant::now();
    for src in [fracs, fenced, tower] {
        match typeset(&db, &face, &src, MathOptions { size_pt: 12.0, display: true }) {
            Ok(l) => assert!(l.bounds.width().is_finite() && l.bounds.height().is_finite()),
            Err(e) => assert!(matches!(e, MathError::TooLarge { .. } | MathError::TooDeep { .. }), "{e}"),
        }
    }
    assert!(started.elapsed().as_secs() < 10);
}
