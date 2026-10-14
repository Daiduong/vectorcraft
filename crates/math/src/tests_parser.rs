//! Parser tests: no font needed.

use super::parser::*;
use super::*;

fn p(src: &str) -> Vec<Node> {
    parse(src).unwrap_or_else(|e| panic!("{src}: {e}"))
}

fn atom(ch: char, class: AtomClass) -> Node {
    Node::Atom { ch, class }
}

fn ord(ch: char) -> Node {
    atom(ch, AtomClass::Ord)
}

fn group(nodes: Vec<Node>) -> Box<Node> {
    Box::new(Node::Group(nodes))
}

fn err(src: &str) -> MathError {
    parse(src).expect_err(src)
}

#[test]
fn empty_and_blank_sources_are_empty_trees() {
    assert_eq!(p(""), vec![]);
    assert_eq!(p("  \n\t "), vec![]);
    assert_eq!(p("{}"), vec![Node::Group(vec![])]);
}

#[test]
fn ascii_atoms_are_classified_by_their_tex_type() {
    use AtomClass::*;
    let cases = [
        ('a', Ord),
        ('Z', Ord),
        ('7', Ord),
        ('+', Bin),
        ('=', Rel),
        ('<', Rel),
        ('>', Rel),
        (':', Rel),
        ('(', Open),
        ('[', Open),
        (')', Close),
        (']', Close),
        (',', Punct),
        (';', Punct),
        ('!', Close),
        ('.', Ord),
        ('/', Ord),
        ('|', Ord),
    ];
    for (c, class) in cases {
        assert_eq!(p(&c.to_string()), vec![atom(c, class)], "{c}");
    }
    // The ASCII hyphen and asterisk are the proper operator glyphs.
    assert_eq!(p("-"), vec![atom('\u{2212}', Bin)]);
    assert_eq!(p("*"), vec![atom('\u{2217}', Bin)]);
}

#[test]
fn whitespace_is_ignored_between_atoms() {
    assert_eq!(p("a  +\n b"), p("a+b"));
    assert_eq!(p("x ^ 2 _ i"), p("x^2_i"));
}

#[test]
fn greek_letters_and_variants() {
    assert_eq!(p(r"\alpha"), vec![ord('\u{3B1}')]);
    assert_eq!(p(r"\omega"), vec![ord('\u{3C9}')]);
    assert_eq!(p(r"\Gamma"), vec![ord('\u{393}')]);
    assert_eq!(p(r"\Omega"), vec![ord('\u{3A9}')]);
    assert_eq!(p(r"\varepsilon"), vec![ord('\u{3B5}')]);
    assert_eq!(p(r"\epsilon"), vec![ord('\u{3F5}')]);
    assert_eq!(p(r"\vartheta"), vec![ord('\u{3D1}')]);
    assert_eq!(p(r"\varphi"), vec![ord('\u{3C6}')]);
    assert_eq!(p(r"\phi"), vec![ord('\u{3D5}')]);
    // Every Greek name the subset lists.
    let lower = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi pi rho sigma tau upsilon phi chi psi omega";
    let upper = "Gamma Delta Theta Lambda Xi Pi Sigma Upsilon Phi Psi Omega";
    for name in lower.split(' ').chain(upper.split(' ')) {
        assert!(matches!(p(&format!(r"\{name}")).as_slice(), [Node::Atom { class: AtomClass::Ord, .. }]), "{name}");
    }
}

#[test]
fn symbols_map_to_unicode_with_their_class() {
    use AtomClass::*;
    let cases = [
        ("pm", '\u{B1}', Bin),
        ("mp", '\u{2213}', Bin),
        ("cdot", '\u{22C5}', Bin),
        ("times", '\u{D7}', Bin),
        ("div", '\u{F7}', Bin),
        ("le", '\u{2264}', Rel),
        ("ge", '\u{2265}', Rel),
        ("ne", '\u{2260}', Rel),
        ("approx", '\u{2248}', Rel),
        ("equiv", '\u{2261}', Rel),
        ("sim", '\u{223C}', Rel),
        ("propto", '\u{221D}', Rel),
        ("infty", '\u{221E}', Ord),
        ("partial", '\u{2202}', Ord),
        ("nabla", '\u{2207}', Ord),
        ("to", '\u{2192}', Rel),
        ("rightarrow", '\u{2192}', Rel),
        ("leftarrow", '\u{2190}', Rel),
        ("Rightarrow", '\u{21D2}', Rel),
        ("Leftarrow", '\u{21D0}', Rel),
        ("leftrightarrow", '\u{2194}', Rel),
        ("in", '\u{2208}', Rel),
        ("notin", '\u{2209}', Rel),
        ("subset", '\u{2282}', Rel),
        ("subseteq", '\u{2286}', Rel),
        ("supset", '\u{2283}', Rel),
        ("supseteq", '\u{2287}', Rel),
        ("cup", '\u{222A}', Bin),
        ("cap", '\u{2229}', Bin),
        ("emptyset", '\u{2205}', Ord),
        ("varnothing", '\u{2205}', Ord),
        ("forall", '\u{2200}', Ord),
        ("exists", '\u{2203}', Ord),
        ("neg", '\u{AC}', Ord),
        ("land", '\u{2227}', Bin),
        ("lor", '\u{2228}', Bin),
        ("angle", '\u{2220}', Ord),
        ("perp", '\u{22A5}', Rel),
        ("parallel", '\u{2225}', Rel),
        ("degree", '\u{B0}', Ord),
        ("prime", '\u{2032}', Ord),
        ("circ", '\u{2218}', Bin),
        ("bullet", '\u{2219}', Bin),
        ("ldots", '\u{2026}', Inner),
        ("cdots", '\u{22EF}', Inner),
        ("vdots", '\u{22EE}', Ord),
        ("ddots", '\u{22F1}', Ord),
    ];
    for (name, ch, class) in cases {
        assert_eq!(p(&format!(r"\{name}")), vec![atom(ch, class)], "\\{name}");
    }
}

#[test]
fn escaped_characters_are_ordinary_atoms() {
    assert_eq!(p(r"\%\$\&\#\_"), vec![ord('%'), ord('$'), ord('&'), ord('#'), ord('_')]);
    assert_eq!(p(r"\{"), vec![atom('{', AtomClass::Open)]);
    assert_eq!(p(r"\}"), vec![atom('}', AtomClass::Close)]);
    assert_eq!(p(r"\|"), vec![ord('\u{2016}')]);
}

#[test]
fn non_ascii_characters_are_accepted_as_they_are() {
    assert_eq!(p("α"), vec![ord('α')]);
    assert_eq!(p("≤"), vec![atom('≤', AtomClass::Rel)]);
    assert_eq!(p("×"), vec![atom('×', AtomClass::Bin)]);
    assert_eq!(p("한"), vec![ord('한')]);
}

#[test]
fn subscripts_and_superscripts() {
    assert_eq!(p("x^2"), vec![Node::Scripts { base: Box::new(ord('x')), sub: None, sup: Some(Box::new(ord('2'))) }]);
    assert_eq!(p("x_i"), vec![Node::Scripts { base: Box::new(ord('x')), sub: Some(Box::new(ord('i'))), sup: None }]);
    // Both, in either order, braced or not.
    let both = Node::Scripts { base: Box::new(ord('x')), sub: Some(group(vec![ord('i'), ord('j')])), sup: Some(Box::new(ord('2'))) };
    assert_eq!(p("x_{ij}^2"), vec![both.clone()]);
    assert_eq!(p("x^2_{ij}"), vec![both]);
    // Nested.
    assert_eq!(
        p("x^{y^z}"),
        vec![Node::Scripts {
            base: Box::new(ord('x')),
            sub: None,
            sup: Some(group(vec![Node::Scripts { base: Box::new(ord('y')), sub: None, sup: Some(Box::new(ord('z'))) }])),
        }]
    );
    // Scripts on a group and on nothing.
    assert!(matches!(&p("{ab}^2")[..], [Node::Scripts { base, .. }] if matches!(**base, Node::Group(_))));
    assert!(matches!(&p("^2")[..], [Node::Scripts { base, .. }] if **base == Node::Group(vec![])));
    // A command is a fine script.
    assert!(matches!(&p(r"x^\alpha")[..], [Node::Scripts { sup: Some(s), .. }] if **s == ord('\u{3B1}')));
    assert!(matches!(&p(r"x^\frac12")[..], [Node::Scripts { sup: Some(s), .. }] if matches!(**s, Node::Frac { .. })));
}

#[test]
fn primes_are_superscripts() {
    let prime = ord('\u{2032}');
    assert_eq!(p("f'"), vec![Node::Scripts { base: Box::new(ord('f')), sub: None, sup: Some(group(vec![prime.clone()])) }]);
    assert_eq!(p("f''"), vec![Node::Scripts { base: Box::new(ord('f')), sub: None, sup: Some(group(vec![prime.clone(), prime.clone()])) }]);
    // A power after the primes joins them, as in TeX.
    assert_eq!(p("f'^2"), vec![Node::Scripts { base: Box::new(ord('f')), sub: None, sup: Some(group(vec![prime.clone(), ord('2')])) }]);
    assert!(matches!(&p("f'_1")[..], [Node::Scripts { sub: Some(_), sup: Some(_), .. }]));
}

#[test]
fn double_scripts_are_errors() {
    for src in ["x^1^2", "x_1_2", "x^1'", "x'^1^2"] {
        assert!(matches!(err(src), MathError::Syntax { .. }), "{src}");
    }
    assert!(matches!(err("x^"), MathError::Syntax { .. }));
    assert!(matches!(err("x_}"), MathError::Syntax { .. }));
    assert!(matches!(err("x^^"), MathError::Syntax { .. }));
}

#[test]
fn fractions() {
    for (cmd, style) in [("frac", FracStyle::Auto), ("dfrac", FracStyle::Display), ("tfrac", FracStyle::Text)] {
        assert_eq!(
            p(&format!(r"\{cmd}{{a}}{{b+c}}")),
            vec![Node::Frac { num: group(vec![ord('a')]), den: group(vec![ord('b'), atom('+', AtomClass::Bin), ord('c')]), style }]
        );
    }
    // Single-token arguments, as in TeX.
    assert_eq!(p(r"\frac12"), vec![Node::Frac { num: Box::new(ord('1')), den: Box::new(ord('2')), style: FracStyle::Auto }]);
    assert!(matches!(err(r"\frac{1}"), MathError::Syntax { .. }));
    assert!(matches!(err(r"\frac"), MathError::Syntax { .. }));
    // Nested.
    assert!(
        matches!(&p(r"\frac{\frac{1}{2}}{3}")[..], [Node::Frac { num, .. }] if matches!(**num, Node::Group(ref g) if matches!(g[..], [Node::Frac { .. }])))
    );
}

#[test]
fn roots() {
    assert_eq!(p(r"\sqrt{x}"), vec![Node::Sqrt { degree: None, body: group(vec![ord('x')]) }]);
    assert_eq!(p(r"\sqrt x"), vec![Node::Sqrt { degree: None, body: Box::new(ord('x')) }]);
    assert_eq!(p(r"\sqrt[3]{x}"), vec![Node::Sqrt { degree: Some(group(vec![ord('3')])), body: group(vec![ord('x')]) }]);
    assert_eq!(
        p(r"\sqrt[n+1]{x}"),
        vec![Node::Sqrt { degree: Some(group(vec![ord('n'), atom('+', AtomClass::Bin), ord('1')])), body: group(vec![ord('x')]) }]
    );
    // The index can hold brackets of its own inside braces.
    assert!(matches!(&p(r"\sqrt[{[a]}]{x}")[..], [Node::Sqrt { degree: Some(_), .. }]));
    assert!(matches!(err(r"\sqrt[3{x}"), MathError::Syntax { .. }));
    assert!(matches!(err(r"\sqrt"), MathError::Syntax { .. }));
}

#[test]
fn left_right_with_every_delimiter() {
    let delims: [(&str, Option<char>); 15] = [
        ("(", Some('(')),
        (")", Some(')')),
        ("[", Some('[')),
        ("]", Some(']')),
        (r"\{", Some('{')),
        (r"\}", Some('}')),
        ("|", Some('|')),
        (r"\|", Some('\u{2016}')),
        (r"\langle", Some('\u{27E8}')),
        (r"\rangle", Some('\u{27E9}')),
        (".", None),
        (r"\lbrace", Some('{')),
        (r"\rbrace", Some('}')),
        (r"\vert", Some('|')),
        ("/", Some('/')),
    ];
    for (src, ch) in delims {
        let got = p(&format!(r"\left{src} x \right{src}"));
        assert_eq!(got, vec![Node::Fenced { open: ch, close: ch, body: vec![ord('x')] }], "{src}");
    }
    assert_eq!(p(r"\left( a \right."), vec![Node::Fenced { open: Some('('), close: None, body: vec![ord('a')] }]);
    // Nested, and inside other constructs.
    assert!(matches!(&p(r"\left(\left[x\right]\right)")[..], [Node::Fenced { body, .. }] if matches!(body[..], [Node::Fenced { .. }])));
    assert!(matches!(&p(r"\frac{\left(a\right)}{b}")[..], [Node::Frac { .. }]));
}

#[test]
fn unbalanced_left_right_are_errors() {
    for src in
        [r"\left( x", r"x \right)", r"\left( {x \right)}", r"\left x \right)", r"\left\foo x\right)", r"\left", r"\left( x \right", r"{\right)}"]
    {
        assert!(matches!(err(src), MathError::Syntax { .. }), "{src}: {:?}", parse(src));
    }
}

#[test]
fn big_operators_and_limits() {
    for (name, ch) in [("sum", '\u{2211}'), ("prod", '\u{220F}'), ("int", '\u{222B}'), ("oint", '\u{222E}')] {
        assert_eq!(p(&format!(r"\{name}")), vec![Node::BigOp { ch, limits: Limits::Auto }], "{name}");
    }
    assert_eq!(p(r"\sum\limits"), vec![Node::BigOp { ch: '\u{2211}', limits: Limits::Limits }]);
    assert_eq!(p(r"\int \nolimits"), vec![Node::BigOp { ch: '\u{222B}', limits: Limits::NoLimits }]);
    assert!(
        matches!(&p(r"\sum_{i=1}^n a_i")[..], [Node::Scripts { base, sub: Some(_), sup: Some(_) }, Node::Scripts { .. }] if matches!(**base, Node::BigOp { .. }))
    );
    // The modifier needs an operator in front of it.
    assert!(matches!(err(r"x\limits"), MathError::Syntax { .. }));
    assert!(matches!(err(r"\limits"), MathError::Syntax { .. }));
}

#[test]
fn function_names() {
    let names = "sin cos tan sec csc cot ln log exp sinh cosh tanh max min gcd lim";
    for name in names.split(' ') {
        assert!(matches!(&p(&format!(r"\{name}"))[..], [Node::Func { name: n, .. }] if n == name), "{name}");
    }
    // \lim and its kin take limits; \sin doesn't.
    assert_eq!(p(r"\lim"), vec![Node::Func { name: "lim".into(), limits: true }]);
    assert_eq!(p(r"\max"), vec![Node::Func { name: "max".into(), limits: true }]);
    assert_eq!(p(r"\sin"), vec![Node::Func { name: "sin".into(), limits: false }]);
    assert_eq!(p(r"\sin\limits"), vec![Node::Func { name: "sin".into(), limits: true }]);
    assert_eq!(p(r"\lim\nolimits"), vec![Node::Func { name: "lim".into(), limits: false }]);
    assert!(matches!(&p(r"\lim_{x\to 0} f")[..], [Node::Scripts { .. }, _]));
    assert_eq!(p(r"\operatorname{rank}"), vec![Node::Func { name: "rank".into(), limits: false }]);
    assert!(matches!(err(r"\operatorname{}"), MathError::Syntax { .. }));
}

#[test]
fn accents_and_wide_marks() {
    for (cmd, kind) in [
        ("hat", AccentKind::Hat),
        ("bar", AccentKind::Bar),
        ("tilde", AccentKind::Tilde),
        ("vec", AccentKind::Vec),
        ("dot", AccentKind::Dot),
        ("ddot", AccentKind::Ddot),
    ] {
        assert_eq!(p(&format!(r"\{cmd}{{x}}")), vec![Node::Accent { kind, body: group(vec![ord('x')]) }], "{cmd}");
        assert_eq!(p(&format!(r"\{cmd} x")), vec![Node::Accent { kind, body: Box::new(ord('x')) }], "{cmd} x");
    }
    for (cmd, kind) in [
        ("overline", WideKind::Overline),
        ("underline", WideKind::Underline),
        ("overrightarrow", WideKind::OverRightArrow),
        ("overleftarrow", WideKind::OverLeftArrow),
    ] {
        assert_eq!(p(&format!(r"\{cmd}{{AB}}")), vec![Node::Wide { kind, body: group(vec![ord('A'), ord('B')]) }], "{cmd}");
    }
    assert!(matches!(err(r"\hat"), MathError::Syntax { .. }));
}

#[test]
fn spacing_commands() {
    assert_eq!(
        p(r"\, \: \; \! \quad \qquad ~ \ "),
        vec![
            Node::Space(Space::Thin),
            Node::Space(Space::Medium),
            Node::Space(Space::Thick),
            Node::Space(Space::NegThin),
            Node::Space(Space::Quad),
            Node::Space(Space::QuadQuad),
            Node::Space(Space::Normal),
            Node::Space(Space::Normal),
        ]
    );
    assert_eq!(Node::Space(Space::Thin).atom_class(), None);
}

#[test]
fn text_keeps_its_spaces_and_braces() {
    assert_eq!(p(r"\text{if } x"), vec![Node::Text("if ".into()), ord('x')]);
    assert_eq!(p(r"\text{a {b} c}"), vec![Node::Text("a {b} c".into())]);
    assert_eq!(p(r"\text{50\% of \{x\}}"), vec![Node::Text("50% of {x}".into())]);
    assert_eq!(p(r"\text{}"), vec![Node::Text(String::new())]);
    assert_eq!(p(r"\text{a~b}"), vec![Node::Text("a b".into())]);
    assert!(matches!(err(r"\text{abc"), MathError::Syntax { .. }));
    assert!(matches!(err(r"\text x"), MathError::Syntax { .. }));
    // Commands aren't math inside text.
    assert_eq!(err(r"\text{a \alpha}"), MathError::UnknownCommand("alpha".into()));
}

#[test]
fn font_styles() {
    for (cmd, variant) in [("mathrm", Variant::Roman), ("mathbf", Variant::Bold), ("mathit", Variant::Italic), ("bm", Variant::BoldItalic)] {
        assert_eq!(p(&format!(r"\{cmd}{{ab}}")), vec![Node::Styled { variant, body: vec![ord('a'), ord('b')] }], "{cmd}");
        assert_eq!(p(&format!(r"\{cmd} a")), vec![Node::Styled { variant, body: vec![ord('a')] }], "{cmd} a");
    }
    // Nested constructs inside.
    assert!(matches!(&p(r"\mathbf{x^2+\frac12}")[..], [Node::Styled { body, .. }] if body.len() == 3));
}

#[test]
fn style_switches_hold_to_the_end_of_the_group() {
    assert_eq!(p(r"a\displaystyle b c"), vec![ord('a'), Node::StyleScope { level: StyleLevel::Display, body: vec![ord('b'), ord('c')] }]);
    assert_eq!(p(r"{\scriptstyle a} b"), vec![Node::Group(vec![Node::StyleScope { level: StyleLevel::Script, body: vec![ord('a')] }]), ord('b')]);
    assert!(
        matches!(&p(r"\left(\textstyle a\right)")[..], [Node::Fenced { body, .. }] if matches!(body[..], [Node::StyleScope { level: StyleLevel::Text, .. }]))
    );
    assert!(matches!(&p(r"\scriptscriptstyle x")[..], [Node::StyleScope { level: StyleLevel::ScriptScript, .. }]));
}

#[test]
fn the_guide_example_parses() {
    let nodes = p(r"y=\sqrt{x}+\dfrac{1}{\sin x}");
    assert_eq!(nodes.len(), 5);
    assert!(matches!(nodes[2], Node::Sqrt { .. }));
    assert!(matches!(&nodes[4], Node::Frac { style: FracStyle::Display, den, .. } if matches!(&**den, Node::Group(g) if g.len() == 2)));
}

#[test]
fn atom_classes_of_nodes_for_spacing() {
    use AtomClass::*;
    let class = |src: &str| p(src)[0].atom_class();
    assert_eq!(class("a"), Some(Ord));
    assert_eq!(class(r"\sum"), Some(Op));
    assert_eq!(class(r"\sin"), Some(Op));
    assert_eq!(class(r"\left(x\right)"), Some(Inner));
    assert_eq!(class(r"\frac12"), Some(Ord));
    assert_eq!(class(r"\sqrt x"), Some(Ord));
    assert_eq!(class("{a+b}"), Some(Ord));
    assert_eq!(class("=^a"), Some(Rel));
    assert_eq!(class(r"\sum_i"), Some(Op));
    assert_eq!(class(r"\text{a}"), Some(Ord));
    assert_eq!(class(r"\hat a"), Some(Ord));
}

#[test]
fn unknown_commands_name_the_command() {
    assert_eq!(err(r"\foo"), MathError::UnknownCommand("foo".into()));
    assert_eq!(err(r"a + \begin{matrix} b"), MathError::UnknownCommand("begin".into()));
    assert_eq!(err(r"\frac{\mathcal{A}}{b}"), MathError::UnknownCommand("mathcal".into()));
    assert_eq!(err(r"x^\unknownthing"), MathError::UnknownCommand("unknownthing".into()));
    assert_eq!(err(r"\\"), MathError::UnknownCommand("\\".into()));
    assert_eq!(err(r"\foo").to_string(), r"unknown command \foo");
}

#[test]
fn malformed_input_is_a_syntax_error_with_a_position() {
    for (src, at) in [("{", 1), ("}", 0), ("a}", 1), (r"\frac{a}{", 9), ("a&b", 1), ("$x$", 0), (r"a\", 2), ("a\u{1}", 1)] {
        match err(src) {
            MathError::Syntax { position, .. } => assert_eq!(position, at, "{src}"),
            e => panic!("{src}: {e:?}"),
        }
    }
    assert!(err("{a").to_string().contains("missing '}'"));
    assert!(matches!(err("x_"), MathError::Syntax { .. }));
}

#[test]
fn the_source_length_is_bounded() {
    assert!(parse(&"a".repeat(MAX_SOURCE_CHARS)).is_ok());
    assert_eq!(err(&"a".repeat(MAX_SOURCE_CHARS + 1)), MathError::TooLong { max: MAX_SOURCE_CHARS });
    // Characters, not bytes.
    assert!(parse(&"한".repeat(MAX_SOURCE_CHARS)).is_ok());
    assert_eq!(err(&"한".repeat(MAX_SOURCE_CHARS + 1)), MathError::TooLong { max: MAX_SOURCE_CHARS });
}

#[test]
fn nesting_is_bounded_for_every_kind_of_nesting() {
    let deep = |open: &str, close: &str, n: usize| format!("{}x{}", open.repeat(n), close.repeat(n));
    // Plain braces: fine up to the limit, an error beyond it.
    assert!(parse(&deep("{", "}", MAX_DEPTH - 1)).is_ok());
    assert_eq!(err(&deep("{", "}", MAX_DEPTH + 1)), MathError::TooDeep { max: MAX_DEPTH });
    // Script towers, fraction towers, roots, \left…\right, accents, styles.
    let n = MAX_DEPTH + 5;
    assert_eq!(err(&format!("x{}", "^{x".repeat(n) + &"}".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x{}", r"\frac{1}{".repeat(n), "}".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x{}", r"\sqrt{".repeat(n), "}".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x{}", r"\left(".repeat(n), r"\right)".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    // Commands nested without braces and style switches (which nest the rest of the row).
    assert_eq!(err(&format!("{}x", r"\hat".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x", r"\sqrt".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x", r"\frac".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x", r"\displaystyle ".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    assert_eq!(err(&format!("{}x", r"\mathbf".repeat(n))), MathError::TooDeep { max: MAX_DEPTH });
    // The maximum input that fits the length bound but nests far too deep fails cleanly.
    assert!(parse(&"{".repeat(MAX_SOURCE_CHARS)).is_err());
    assert!(parse(&r"\frac".repeat(MAX_SOURCE_CHARS / 5)).is_err());
    assert!(parse(&"^".repeat(MAX_SOURCE_CHARS)).is_err());
}

#[test]
fn flat_input_of_the_maximum_length_parses() {
    assert_eq!(p(&"a+".repeat(MAX_SOURCE_CHARS / 2)).len(), MAX_SOURCE_CHARS);
    assert_eq!(p(&"{}".repeat(MAX_SOURCE_CHARS / 2)).len(), MAX_SOURCE_CHARS / 2);
}

#[test]
fn arbitrary_junk_never_panics() {
    // A deterministic spread of fragments, joined in every order of a few lengths.
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
        r"\",
        r"\limits",
        "'",
        "~",
        "\u{0}",
        "한",
        r"\hat",
        r"\sum",
        r"\displaystyle",
        "&",
        "#",
        ".",
    ];
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..4000 {
        let mut src = String::new();
        for _ in 0..(seed % 12) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            src.push_str(frags[(seed >> 33) as usize % frags.len()]);
        }
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let _ = parse(&src);
    }
}
