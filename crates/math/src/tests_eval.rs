//! The expression language: precedence, functions, errors, bounds and derivatives.

use super::eval::*;
use std::f64::consts::{E, LN_2, PI};

fn p(src: &str) -> Expr {
    parse_expr(src, &["x"]).unwrap_or_else(|e| panic!("{src}: {e}"))
}

fn at(src: &str, x: f64) -> f64 {
    p(src).eval1(x)
}

fn err(src: &str) -> EvalError {
    parse_expr(src, &["x"]).expect_err(src)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}

#[test]
fn the_guide_example_brackets_as_the_original_tool_shows_it() {
    assert_eq!(p(r"3x^2+\sin x*\cos x").to_string(), r"((3*(x^2))+((\sin{x})*(\cos{x})))");
}

/// Function arguments bind tightest, then `^`, then `*` `/` and implicit multiplication, then `+ -`.
#[test]
fn precedence_puts_function_arguments_first() {
    assert_eq!(p(r"\sin 2x").to_string(), r"((\sin{2})*x)");
    assert!(close(at(r"\sin 2x", 1.5), 2f64.sin() * 1.5));
    assert_eq!(p(r"\sin x^2").to_string(), r"((\sin{x})^2)");
    assert_eq!(p(r"\sin^2 x").to_string(), r"((\sin{x})^2)");
    assert_eq!(p(r"\sin^{2}x").to_string(), r"((\sin{x})^2)");
    assert_eq!(p("2x^2").to_string(), "(2*(x^2))");
    assert_eq!(p("x^2x").to_string(), "((x^2)*x)");
    assert_eq!(p("1+2*3").to_string(), "(1+(2*3))");
    assert_eq!(p("1-2-3").to_string(), "((1-2)-3)");
    assert_eq!(p("8/2/2").to_string(), "((8/2)/2)");
    assert!(close(at("2^3^2", 0.0), 512.0), "right associative");
    assert_eq!(p("x(x+1)").to_string(), "(x*(x+1))");
    assert_eq!(p("(x+1)(x-1)").to_string(), "((x+1)*(x-1))");
    assert_eq!(p(r"\sin x \cos x").to_string(), r"((\sin{x})*(\cos{x}))");
    assert_eq!(p(r"\sin\cos x").to_string(), r"(\sin{(\cos{x})})");
    assert_eq!(p(r"2\pi x").to_string(), format!("((2*{PI})*x)"));
}

#[test]
fn unary_minus_is_zero_minus_its_operand() {
    assert_eq!(p("-x^2").to_string(), "(0-(x^2))");
    assert!(close(at("-x^2", 3.0), -9.0));
    assert!(close(at("--x", 3.0), 3.0));
    assert!(close(at("2*-3", 0.0), -6.0));
    assert!(close(at("x^-1", 4.0), 0.25));
    assert!(close(at("x^-2x", 2.0), 0.5), "x^(−2)·x");
    assert!(close(at(r"\sin -x", 1.0), -(1f64.sin())));
    assert!(close(at("+x", 2.0), 2.0));
}

#[test]
fn arguments_need_a_space_or_brackets_but_numbers_may_follow_directly() {
    assert!(close(at(r"\ln2", 0.0), LN_2));
    assert!(close(at(r"\ln 2", 0.0), LN_2));
    for src in [r"\sin x", r"\sin{x}", r"\sin(x)", r"\sin  x"] {
        assert!(close(at(src, 0.7), 0.7f64.sin()), "{src}");
    }
    assert_eq!(err(r"\sinx"), EvalError::UnknownCommand("sinx".into()));
    assert!(err(r"\sinx").to_string().contains(r"\sin x"));
}

#[test]
fn every_function_evaluates() {
    let x = 0.7f64;
    let cases: &[(&str, f64)] = &[
        (r"\sqrt x", x.sqrt()),
        (r"\sin x", x.sin()),
        (r"\cos x", x.cos()),
        (r"\tan x", x.tan()),
        (r"\sec x", 1.0 / x.cos()),
        (r"\csc x", 1.0 / x.sin()),
        (r"\cot x", 1.0 / x.tan()),
        (r"\ln x", x.ln()),
        (r"\log x", x.log10()),
        (r"\exp x", x.exp()),
        (r"\sinh x", x.sinh()),
        (r"\cosh x", x.cosh()),
        (r"\tanh x", x.tanh()),
        (r"\sech x", 1.0 / x.cosh()),
        (r"\csch x", 1.0 / x.sinh()),
        (r"\coth x", 1.0 / x.tanh()),
        (r"\abs{x-1}", 0.3),
        (r"\log_{2} x", x.log2()),
        (r"\log_2 x", x.log2()),
        (r"\log_{10}(x)", x.log10()),
        (r"\frac{1}{x}", 1.0 / x),
        (r"\dfrac{x}{2}", x / 2.0),
        (r"\tfrac{x}{2}", x / 2.0),
        (r"\pow{x}{3}", x.powi(3)),
        (r"\sqrt[3]{x}", x.cbrt()),
        (r"\cos^{3}x", x.cos().powi(3)),
        (r"\tan^2 x", x.tan().powi(2)),
        (r"\cot^2 x", (1.0 / x.tan()).powi(2)),
        (r"\log_2^2 x", x.log2().powi(2)),
        ("e^x", x.exp()),
        ("e", E),
        (r"\pi", PI),
        (r"2 \cdot x", 2.0 * x),
        (r"2 \times x", 2.0 * x),
        (r"x \div 2", x / 2.0),
        (r"\left( x+1 \right)^2", (x + 1.0).powi(2)),
        (r"[x+1]", x + 1.0),
        (r"\{x\}", x),
        (r"x\,+\;1", x + 1.0),
        (".5x", 0.5 * x),
        ("1.25", 1.25),
    ];
    for (src, want) in cases {
        assert!(close(at(src, x), *want), "{src}: {} ≠ {want}", at(src, x));
    }
    assert!(close(at(r"\sqrt[3]{x}", -8.0), -2.0), "an odd root of a negative number is real");
    assert!(at(r"\sqrt[2]{x}", -4.0).is_nan());
    assert!(at(r"\sqrt x", -1.0).is_nan());
    assert!(at(r"\ln x", 0.0) == f64::NEG_INFINITY);
    assert!(at(r"\frac{1}{x}", 0.0).is_infinite());
}

#[test]
fn several_variables_and_the_environment() {
    let e = parse_expr("f*g+h x", &["f", "g", "h", "x"]).unwrap();
    assert!(close(e.eval(&[("f", 2.0), ("g", 3.0), ("h", 4.0), ("x", 5.0)]), 26.0));
    assert!(e.eval(&[("f", 2.0)]).is_nan(), "an unbound variable is NaN");
    let t = parse_expr(r"\cos t", &["t"]).unwrap();
    assert!(close(t.eval(&[("t", 0.0)]), 1.0));
    assert_eq!(parse_expr("x", &["t"]), Err(EvalError::UnknownVariable('x')));
    // `e` is a variable when one is named so.
    assert!(close(parse_expr("e", &["e"]).unwrap().eval1(2.0), 2.0));
}

#[test]
fn malformed_and_oversized_sources_are_errors() {
    assert_eq!(err(""), EvalError::Empty);
    assert_eq!(err("  \t"), EvalError::Empty);
    assert_eq!(err("y"), EvalError::UnknownVariable('y'));
    for src in [
        "x+",
        "(x",
        "x)",
        "{x",
        "[x)",
        r"\frac{1}",
        r"\sin",
        r"\sin^",
        "*x",
        "x**2",
        r"\",
        r"\@",
        "x # 2",
        r"\sqrt[x]{2}",
        r"\sqrt[2{x}",
        "|x|",
        "^2",
        "x_2",
    ] {
        assert!(
            matches!(parse_expr(src, &["x"]), Err(EvalError::Syntax { .. } | EvalError::UnknownVariable(_) | EvalError::UnknownCommand(_))),
            "{src}"
        );
    }
    let e = err("x + (2");
    assert!(matches!(e, EvalError::Syntax { position: 4, .. }), "{e:?}");
    assert_eq!(err(r"\arcsin x"), EvalError::UnknownCommand("arcsin".into()));

    let long = "x+".repeat(MAX_EXPR_CHARS / 2) + "x";
    assert_eq!(err(&long), EvalError::TooLong { max: MAX_EXPR_CHARS });
    let fits = "x+".repeat(MAX_EXPR_CHARS / 2 - 1) + "x";
    assert!(fits.chars().count() <= MAX_EXPR_CHARS);
    assert!(close(at(&fits, 1.0), (MAX_EXPR_CHARS / 2) as f64));
    let deep = "(".repeat(MAX_EXPR_DEPTH + 1) + "x" + &")".repeat(MAX_EXPR_DEPTH + 1);
    assert_eq!(err(&deep), EvalError::TooDeep { max: MAX_EXPR_DEPTH });
    let ok = "(".repeat(MAX_EXPR_DEPTH) + "x" + &")".repeat(MAX_EXPR_DEPTH);
    assert!(close(at(&ok, 2.0), 2.0));
    let nested = r"\sin".repeat(MAX_EXPR_DEPTH + 1) + " x";
    assert_eq!(err(&nested), EvalError::TooDeep { max: MAX_EXPR_DEPTH });
    let towers = "2^".repeat(MAX_EXPR_DEPTH + 1) + "2";
    assert_eq!(err(&towers), EvalError::TooDeep { max: MAX_EXPR_DEPTH });
    // Long flat sources stay within the node and tree bounds.
    let product = "x".repeat(MAX_TREE_DEPTH - 1);
    let e = p(&product);
    assert!(e.size() <= MAX_EXPR_NODES && e.depth() <= MAX_TREE_DEPTH);
    // A chain longer than the tree may be deep (absurd, but within the length) is refused.
    assert_eq!(err(&"x".repeat(MAX_EXPR_CHARS)), EvalError::TooDeep { max: MAX_TREE_DEPTH });
    let signs = "-x*".repeat(MAX_EXPR_CHARS / 3 - 1) + "x";
    assert!(p(&signs).size() <= MAX_EXPR_NODES);
}

/// Junk never panics: a deterministic mix of the language's pieces.
#[test]
fn junk_never_panics() {
    const PIECES: &[&str] = &[
        "x", "2", ".", "+", "-", "*", "/", "^", "_", "(", ")", "{", "}", "[", "]", r"\sin", r"\frac", r"\log", r"\sqrt", r"\pi", "e", " ", r"\", "y",
        "1e5", r"\left", "é",
    ];
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    for _ in 0..20_000 {
        let mut src = String::new();
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let len = (seed % 24) as usize;
        for k in 0..len {
            let i = ((seed >> (k % 48)) as usize).wrapping_mul(k + 1) % PIECES.len();
            src.push_str(PIECES[i]);
        }
        if let Ok(e) = parse_expr(&src, &["x"]) {
            let _ = e.eval1(0.5);
            let _ = e.to_string();
            if let Ok(d) = e.diff("x") {
                let _ = d.eval1(0.5);
            }
        }
    }
}

/// A central difference of `e` at `x`.
fn numeric(e: &Expr, x: f64) -> f64 {
    let h = 1e-5;
    (e.eval1(x + h) - e.eval1(x - h)) / (2.0 * h)
}

#[test]
fn derivatives_match_difference_quotients() {
    let sources = [
        "x^3-x",
        r"3x^2+\sin x*\cos x",
        r"\sqrt x",
        r"\sqrt{x^2+1}",
        r"\tan x",
        r"\sec x",
        r"\csc x",
        r"\cot x",
        r"\ln x",
        r"\log x",
        r"\log_{2}(x+1)",
        r"\exp(2x)",
        r"\sinh x",
        r"\cosh x",
        r"\tanh x",
        r"\sech x",
        r"\csch x",
        r"\coth x",
        r"\abs{x-1}",
        r"\frac{\sin x}{x^2+1}",
        r"\frac{1}{x}",
        "2^x",
        "x^x",
        r"\sqrt[3]{x}",
        r"\sin^2 x",
        r"e^{-x^2}",
        "-x^2",
    ];
    for src in sources {
        let e = p(src);
        let d = e.diff("x").unwrap_or_else(|err| panic!("{src}: {err}"));
        for x in [0.3, 0.7, 1.3, 2.1] {
            let (want, got) = (numeric(&e, x), d.eval1(x));
            assert!((want - got).abs() <= 1e-5 * (1.0 + want.abs()), "{src} at {x}: d = {d}: {got} vs {want}");
        }
    }
}

#[test]
fn third_derivatives_are_right_and_stay_small() {
    let e = p(r"\sin x");
    let d3 = e.diff("x").unwrap().diff("x").unwrap().diff("x").unwrap();
    for x in [0.0, 0.5, 2.0] {
        assert!(close(d3.eval1(x), -x.cos()), "{d3}");
    }
    let cubic = p("x^3-x");
    let d1 = cubic.diff("x").unwrap();
    let d2 = d1.diff("x").unwrap();
    let d3 = d2.diff("x").unwrap();
    assert!(close(d1.eval1(2.0), 11.0) && close(d2.eval1(2.0), 12.0) && close(d3.eval1(2.0), 6.0));
    assert_eq!(d3, Expr::Num(6.0), "numbers fold: {d3}");
    let q = p(r"\frac{\sin x}{x^2+1}");
    let mut d = q.clone();
    for _ in 0..3 {
        d = d.diff("x").unwrap();
    }
    assert!(d.size() < 2000 && d.depth() < 100, "{} nodes, depth {}", d.size(), d.depth());
}

#[test]
fn trivial_terms_are_simplified_away() {
    assert_eq!(p("x").diff("x").unwrap(), Expr::Num(1.0));
    assert_eq!(p("3x").diff("x").unwrap(), Expr::Num(3.0));
    assert_eq!(p("5").diff("x").unwrap(), Expr::Num(0.0));
    assert_eq!(p("x+7").diff("x").unwrap(), Expr::Num(1.0));
    assert_eq!(p("x^2").diff("x").unwrap().to_string(), "(2*x)");
    assert_eq!(p(r"\sin x").diff("x").unwrap().to_string(), r"(\cos{x})");
    assert_eq!(p(r"\abs x").diff("x").unwrap().to_string(), r"(\sign{x})");
    // Another variable is a constant.
    let e = parse_expr("a x^2", &["a", "x"]).unwrap();
    assert_eq!(e.diff("a").unwrap().to_string(), "(x^2)");
}

/// Derivatives that would grow past the bounds fail instead of eating memory or the stack.
#[test]
fn runaway_derivatives_are_refused() {
    let product = vec!["(x+1)"; 190].join("");
    let mut e = p(&product);
    let mut failed = false;
    for _ in 0..3 {
        match e.diff("x") {
            Ok(d) => {
                assert!(d.size() <= MAX_DIFF_NODES && d.depth() <= MAX_TREE_DEPTH);
                e = d;
            }
            Err(err) => {
                assert!(matches!(err, EvalError::TooLarge { .. } | EvalError::TooDeep { .. }), "{err}");
                failed = true;
                break;
            }
        }
    }
    assert!(failed, "a 190-factor product's third derivative is out of bounds");
}
