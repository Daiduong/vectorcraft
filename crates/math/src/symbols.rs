//! Tables of the LaTeX subset: commands for symbols, big operators and function names, and the
//! Unicode Mathematical Alphanumeric Symbols letters math fonts draw italic and bold with.

use crate::parser::{AtomClass, Variant};

/// A symbol command: `\name` is the character `ch`, spaced as an atom of class `class`.
pub(crate) const SYMBOLS: &[(&str, char, AtomClass)] = &[
    // Greek, lower case (the italic forms come from `styled_char`).
    ("alpha", '\u{3B1}', AtomClass::Ord),
    ("beta", '\u{3B2}', AtomClass::Ord),
    ("gamma", '\u{3B3}', AtomClass::Ord),
    ("delta", '\u{3B4}', AtomClass::Ord),
    ("epsilon", '\u{3F5}', AtomClass::Ord),
    ("varepsilon", '\u{3B5}', AtomClass::Ord),
    ("zeta", '\u{3B6}', AtomClass::Ord),
    ("eta", '\u{3B7}', AtomClass::Ord),
    ("theta", '\u{3B8}', AtomClass::Ord),
    ("vartheta", '\u{3D1}', AtomClass::Ord),
    ("iota", '\u{3B9}', AtomClass::Ord),
    ("kappa", '\u{3BA}', AtomClass::Ord),
    ("varkappa", '\u{3F0}', AtomClass::Ord),
    ("lambda", '\u{3BB}', AtomClass::Ord),
    ("mu", '\u{3BC}', AtomClass::Ord),
    ("nu", '\u{3BD}', AtomClass::Ord),
    ("xi", '\u{3BE}', AtomClass::Ord),
    ("pi", '\u{3C0}', AtomClass::Ord),
    ("varpi", '\u{3D6}', AtomClass::Ord),
    ("rho", '\u{3C1}', AtomClass::Ord),
    ("varrho", '\u{3F1}', AtomClass::Ord),
    ("sigma", '\u{3C3}', AtomClass::Ord),
    ("varsigma", '\u{3C2}', AtomClass::Ord),
    ("tau", '\u{3C4}', AtomClass::Ord),
    ("upsilon", '\u{3C5}', AtomClass::Ord),
    ("phi", '\u{3D5}', AtomClass::Ord),
    ("varphi", '\u{3C6}', AtomClass::Ord),
    ("chi", '\u{3C7}', AtomClass::Ord),
    ("psi", '\u{3C8}', AtomClass::Ord),
    ("omega", '\u{3C9}', AtomClass::Ord),
    // Greek, upper case.
    ("Gamma", '\u{393}', AtomClass::Ord),
    ("Delta", '\u{394}', AtomClass::Ord),
    ("Theta", '\u{398}', AtomClass::Ord),
    ("Lambda", '\u{39B}', AtomClass::Ord),
    ("Xi", '\u{39E}', AtomClass::Ord),
    ("Pi", '\u{3A0}', AtomClass::Ord),
    ("Sigma", '\u{3A3}', AtomClass::Ord),
    ("Upsilon", '\u{3A5}', AtomClass::Ord),
    ("Phi", '\u{3A6}', AtomClass::Ord),
    ("Psi", '\u{3A8}', AtomClass::Ord),
    ("Omega", '\u{3A9}', AtomClass::Ord),
    // Binary operators.
    ("pm", '\u{B1}', AtomClass::Bin),
    ("mp", '\u{2213}', AtomClass::Bin),
    ("cdot", '\u{22C5}', AtomClass::Bin),
    ("times", '\u{D7}', AtomClass::Bin),
    ("div", '\u{F7}', AtomClass::Bin),
    ("ast", '\u{2217}', AtomClass::Bin),
    ("star", '\u{22C6}', AtomClass::Bin),
    ("circ", '\u{2218}', AtomClass::Bin),
    ("bullet", '\u{2219}', AtomClass::Bin),
    ("cup", '\u{222A}', AtomClass::Bin),
    ("cap", '\u{2229}', AtomClass::Bin),
    ("setminus", '\u{2216}', AtomClass::Bin),
    ("land", '\u{2227}', AtomClass::Bin),
    ("lor", '\u{2228}', AtomClass::Bin),
    ("oplus", '\u{2295}', AtomClass::Bin),
    ("otimes", '\u{2297}', AtomClass::Bin),
    // Relations.
    ("le", '\u{2264}', AtomClass::Rel),
    ("leq", '\u{2264}', AtomClass::Rel),
    ("ge", '\u{2265}', AtomClass::Rel),
    ("geq", '\u{2265}', AtomClass::Rel),
    ("ne", '\u{2260}', AtomClass::Rel),
    ("neq", '\u{2260}', AtomClass::Rel),
    ("ll", '\u{226A}', AtomClass::Rel),
    ("gg", '\u{226B}', AtomClass::Rel),
    ("approx", '\u{2248}', AtomClass::Rel),
    ("equiv", '\u{2261}', AtomClass::Rel),
    ("sim", '\u{223C}', AtomClass::Rel),
    ("simeq", '\u{2243}', AtomClass::Rel),
    ("cong", '\u{2245}', AtomClass::Rel),
    ("propto", '\u{221D}', AtomClass::Rel),
    ("to", '\u{2192}', AtomClass::Rel),
    ("rightarrow", '\u{2192}', AtomClass::Rel),
    ("leftarrow", '\u{2190}', AtomClass::Rel),
    ("gets", '\u{2190}', AtomClass::Rel),
    ("Rightarrow", '\u{21D2}', AtomClass::Rel),
    ("Leftarrow", '\u{21D0}', AtomClass::Rel),
    ("leftrightarrow", '\u{2194}', AtomClass::Rel),
    ("Leftrightarrow", '\u{21D4}', AtomClass::Rel),
    ("mapsto", '\u{21A6}', AtomClass::Rel),
    ("implies", '\u{27F9}', AtomClass::Rel),
    ("iff", '\u{27FA}', AtomClass::Rel),
    ("in", '\u{2208}', AtomClass::Rel),
    ("notin", '\u{2209}', AtomClass::Rel),
    ("subset", '\u{2282}', AtomClass::Rel),
    ("subseteq", '\u{2286}', AtomClass::Rel),
    ("supset", '\u{2283}', AtomClass::Rel),
    ("supseteq", '\u{2287}', AtomClass::Rel),
    ("perp", '\u{22A5}', AtomClass::Rel),
    ("parallel", '\u{2225}', AtomClass::Rel),
    ("mid", '\u{2223}', AtomClass::Rel),
    // Ordinary symbols.
    ("infty", '\u{221E}', AtomClass::Ord),
    ("partial", '\u{2202}', AtomClass::Ord),
    ("nabla", '\u{2207}', AtomClass::Ord),
    ("emptyset", '\u{2205}', AtomClass::Ord),
    ("varnothing", '\u{2205}', AtomClass::Ord),
    ("forall", '\u{2200}', AtomClass::Ord),
    ("exists", '\u{2203}', AtomClass::Ord),
    ("neg", '\u{AC}', AtomClass::Ord),
    ("angle", '\u{2220}', AtomClass::Ord),
    ("degree", '\u{B0}', AtomClass::Ord),
    ("prime", '\u{2032}', AtomClass::Ord),
    ("ell", '\u{2113}', AtomClass::Ord),
    ("hbar", '\u{210F}', AtomClass::Ord),
    ("aleph", '\u{2135}', AtomClass::Ord),
    ("vert", '|', AtomClass::Ord),
    ("Vert", '\u{2016}', AtomClass::Ord),
    ("vdots", '\u{22EE}', AtomClass::Ord),
    ("ddots", '\u{22F1}', AtomClass::Ord),
    // Dots.
    ("ldots", '\u{2026}', AtomClass::Inner),
    ("dots", '\u{2026}', AtomClass::Inner),
    ("cdots", '\u{22EF}', AtomClass::Inner),
    // Delimiters used on their own.
    ("langle", '\u{27E8}', AtomClass::Open),
    ("rangle", '\u{27E9}', AtomClass::Close),
    ("lbrace", '{', AtomClass::Open),
    ("rbrace", '}', AtomClass::Close),
    ("lceil", '\u{2308}', AtomClass::Open),
    ("rceil", '\u{2309}', AtomClass::Close),
    ("lfloor", '\u{230A}', AtomClass::Open),
    ("rfloor", '\u{230B}', AtomClass::Close),
    ("colon", ':', AtomClass::Punct),
];

/// Commands written with a symbol after the backslash: `\{`, `\%`…
pub(crate) const ESCAPED: &[(char, char, AtomClass)] = &[
    ('{', '{', AtomClass::Open),
    ('}', '}', AtomClass::Close),
    ('|', '\u{2016}', AtomClass::Ord),
    ('%', '%', AtomClass::Ord),
    ('$', '$', AtomClass::Ord),
    ('&', '&', AtomClass::Ord),
    ('#', '#', AtomClass::Ord),
    ('_', '_', AtomClass::Ord),
];

/// Large operators: `\name` is the character, which grows in display style. Whether they take
/// limits (scripts above and below) by default is [`is_integral`]'s answer.
pub(crate) const BIG_OPS: &[(&str, char)] = &[
    ("sum", '\u{2211}'),
    ("prod", '\u{220F}'),
    ("coprod", '\u{2210}'),
    ("int", '\u{222B}'),
    ("iint", '\u{222C}'),
    ("iiint", '\u{222D}'),
    ("oint", '\u{222E}'),
    ("bigcup", '\u{22C3}'),
    ("bigcap", '\u{22C2}'),
];

/// Integral signs: their scripts go beside them unless `\limits` says otherwise.
pub(crate) fn is_integral(ch: char) -> bool {
    matches!(ch, '\u{222B}'..='\u{222E}')
}

/// Operator names set upright as text: `(name, takes limits)`. Limits are the operator's scripts
/// above and below it in display style (`\lim`).
pub(crate) const FUNCTIONS: &[(&str, bool)] = &[
    ("sin", false),
    ("cos", false),
    ("tan", false),
    ("sec", false),
    ("csc", false),
    ("cot", false),
    ("arcsin", false),
    ("arccos", false),
    ("arctan", false),
    ("sinh", false),
    ("cosh", false),
    ("tanh", false),
    ("ln", false),
    ("log", false),
    ("exp", false),
    ("deg", false),
    ("dim", false),
    ("ker", false),
    ("arg", false),
    ("lim", true),
    ("limsup", true),
    ("liminf", true),
    ("max", true),
    ("min", true),
    ("sup", true),
    ("inf", true),
    ("gcd", true),
    ("det", true),
    ("Pr", true),
];

pub(crate) fn lookup_symbol(name: &str) -> Option<(char, AtomClass)> {
    SYMBOLS.iter().find(|(n, ..)| *n == name).map(|&(_, ch, class)| (ch, class))
}

/// The class a character typed as itself (not through a command) is spaced as.
pub(crate) fn class_of_char(c: char) -> AtomClass {
    match c {
        '+' | '\u{2212}' | '\u{2217}' => AtomClass::Bin,
        '=' | '<' | '>' | ':' => AtomClass::Rel,
        '(' | '[' => AtomClass::Open,
        ')' | ']' | '!' | '?' => AtomClass::Close,
        ',' | ';' => AtomClass::Punct,
        c if c.is_ascii() => AtomClass::Ord,
        c => SYMBOLS.iter().find(|(_, ch, _)| *ch == c).map_or(AtomClass::Ord, |&(_, _, class)| class),
    }
}

/// `c` drawn in `variant`: the Mathematical Alphanumeric Symbols letter (U+1D400…) math fonts
/// draw italic and bold letters with. Characters the variant doesn't change (operators,
/// punctuation, digits in italic) come back as they are. Fonts lacking the letter get `c` instead
/// (the caller falls back).
pub(crate) fn styled_char(c: char, variant: Variant) -> char {
    let v = match variant {
        Variant::Math => match c {
            // Letters of math are italic; upper-case Greek and digits stay upright.
            'a'..='z' | 'A'..='Z' | '\u{3B1}'..='\u{3C9}' | '\u{3D1}' | '\u{3D5}' | '\u{3D6}' | '\u{3F0}' | '\u{3F1}' | '\u{3F5}' | '\u{2202}' => {
                Variant::Italic
            }
            _ => return c,
        },
        v => v,
    };
    let (latin_upper, latin_lower, greek_upper, greek_lower, greek_symbols, digit) = match v {
        Variant::Roman | Variant::Math => return c,
        Variant::Italic => (0x1D434, 0x1D44E, 0x1D6E2, 0x1D6FC, 0x1D716, None),
        Variant::Bold => (0x1D400, 0x1D41A, 0x1D6A8, 0x1D6C2, 0x1D6DC, Some(0x1D7CE)),
        Variant::BoldItalic => (0x1D468, 0x1D482, 0x1D71C, 0x1D736, 0x1D750, None),
    };
    let u = u32::from(c);
    let mapped = match c {
        // The italic h is a hole in the block: the Planck constant sign stands in.
        'h' if v == Variant::Italic => Some(0x210E),
        'A'..='Z' => Some(latin_upper + (u - 0x41)),
        'a'..='z' => Some(latin_lower + (u - 0x61)),
        '0'..='9' => digit.map(|d| d + (u - 0x30)),
        '\u{391}'..='\u{3A9}' if c != '\u{3A2}' => Some(greek_upper + (u - 0x391)),
        '\u{3B1}'..='\u{3C9}' => Some(greek_lower + (u - 0x3B1)),
        // ϵ ϑ ϰ ϕ ϱ ϖ, in the block's order.
        '\u{3F5}' => Some(greek_symbols),
        '\u{3D1}' => Some(greek_symbols + 1),
        '\u{3F0}' => Some(greek_symbols + 2),
        '\u{3D5}' => Some(greek_symbols + 3),
        '\u{3F1}' => Some(greek_symbols + 4),
        '\u{3D6}' => Some(greek_symbols + 5),
        '\u{2202}' => Some(greek_symbols - 1),
        _ => None,
    };
    mapped.and_then(char::from_u32).unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_map_into_the_alphanumeric_block() {
        assert_eq!(styled_char('x', Variant::Math), '\u{1D465}');
        assert_eq!(styled_char('A', Variant::Math), '\u{1D434}');
        assert_eq!(styled_char('h', Variant::Math), '\u{210E}');
        assert_eq!(styled_char('x', Variant::Roman), 'x');
        assert_eq!(styled_char('x', Variant::Bold), '\u{1D431}');
        assert_eq!(styled_char('x', Variant::BoldItalic), '\u{1D499}');
        assert_eq!(styled_char('3', Variant::Bold), '\u{1D7D1}');
        assert_eq!(styled_char('3', Variant::Math), '3');
        assert_eq!(styled_char('+', Variant::Bold), '+');
    }

    #[test]
    fn greek_is_italic_in_lower_case_and_upright_in_upper_case() {
        assert_eq!(styled_char('\u{3B1}', Variant::Math), '\u{1D6FC}');
        assert_eq!(styled_char('\u{3C9}', Variant::Math), '\u{1D714}');
        assert_eq!(styled_char('\u{3A9}', Variant::Math), '\u{3A9}');
        assert_eq!(styled_char('\u{3A9}', Variant::Bold), '\u{1D6C0}');
        assert_eq!(styled_char('\u{3F5}', Variant::Math), '\u{1D716}');
        assert_eq!(styled_char('\u{3D6}', Variant::Math), '\u{1D71B}');
        assert_eq!(styled_char('\u{3C6}', Variant::Math), '\u{1D711}');
        assert_eq!(styled_char('\u{2202}', Variant::Math), '\u{1D715}');
        assert_eq!(styled_char('\u{2202}', Variant::Roman), '\u{2202}');
    }

    #[test]
    fn every_symbol_name_is_unique() {
        for (i, (a, ..)) in SYMBOLS.iter().enumerate() {
            assert!(SYMBOLS.iter().skip(i + 1).all(|(b, ..)| a != b), "{a} twice");
            assert!(BIG_OPS.iter().all(|(b, _)| a != b) && FUNCTIONS.iter().all(|(b, _)| a != b), "{a} in two tables");
        }
    }
}
