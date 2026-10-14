//! A small LaTeX-flavoured expression language for graphs and coordinates: `3x^2+\sin x*\cos x`,
//! `\frac{1}{x-1}`, `\log_{2}(x+1)`, `\sin^2 x`.
//!
//! [`parse_expr`] reads an expression in the variables it is given into an [`Expr`] tree, which
//! [`Expr::eval`] evaluates (NaN and infinities propagate, nothing panics) and [`Expr::diff`]
//! differentiates symbolically.
//!
//! Precedence, loosest first: `+ -`, then `* /` and implicit multiplication (`2x`, `x(x+1)`), then
//! `^` (right associative, the exponent is one operand: `x^2y` is `(x^2)·y`), then a function and
//! its argument, which bind tightest of all: `\sin 2x` is `(\sin 2)·x` and `\sin x^2` is
//! `(\sin x)^2` — unlike the way LaTeX typesets them, as in the tool this language comes from.
//! A unary minus is `0 − operand` (`-x^2` is `-(x^2)`).
//!
//! - One-argument functions: `\sqrt \sin \cos \tan \sec \csc \cot \ln \log \exp \sinh \cosh \tanh
//!   \sech \csch \coth \abs` (`\log` alone is base 10). A power after the name raises the value:
//!   `\sin^2 x` is `(\sin x)^2`.
//! - Two arguments: `\log_{b} x` (base `b`), `\frac{a}{b}` (also `\dfrac`, `\tfrac`), `\pow{a}{b}`,
//!   `\sqrt[n]{x}` (an odd root of a negative number is real).
//! - Constants `e` and `\pi`; `\cdot` and `\times` multiply, `\div` divides; `\left` and `\right`
//!   are ignored; `()`, `{}` and `[]` group.
//! - An argument is a number, a variable, a constant, a bracketed group or another function. A
//!   number may follow the name directly (`\ln2`), a variable may not (`\sinx` is an unknown
//!   command: write `\sin x`, `\sin{x}` or `\sin(x)`).
//!
//! The source is untrusted: its length ([`MAX_EXPR_CHARS`]), nesting ([`MAX_EXPR_DEPTH`]) and
//! size ([`MAX_EXPR_NODES`]) are bounded, and so are derivatives ([`MAX_DIFF_NODES`]).

use std::fmt;

/// Longest source [`parse_expr`] reads, in characters.
pub const MAX_EXPR_CHARS: usize = 1000;
/// Deepest nesting of brackets, function arguments and exponents [`parse_expr`] follows.
pub const MAX_EXPR_DEPTH: usize = 32;
/// Most nodes a parsed expression may have.
pub const MAX_EXPR_NODES: usize = 2000;
/// Most nodes a derivative may have ([`Expr::diff`]).
pub const MAX_DIFF_NODES: usize = 50_000;
/// Deepest tree an expression or a derivative may be: what evaluates and differentiates it
/// recurses this deep at most.
pub const MAX_TREE_DEPTH: usize = 512;

/// Why an expression could not be read or differentiated. The messages are in English.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum EvalError {
    #[error("the expression is empty")]
    Empty,
    #[error("the expression is longer than {max} characters")]
    TooLong { max: usize },
    #[error("the expression is nested deeper than {max} levels")]
    TooDeep { max: usize },
    #[error("the expression is larger than {max} terms")]
    TooLarge { max: usize },
    /// A command this language doesn't have, given without the backslash.
    #[error("unknown function \\{0} (put a space or braces between a function and its argument: \\sin x, \\sin{{x}})")]
    UnknownCommand(String),
    #[error("unknown variable {0}")]
    UnknownVariable(char),
    /// Malformed input; `position` counts characters from 0.
    #[error("{message} (at character {})", .position + 1)]
    Syntax { message: String, position: usize },
}

/// A binary operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    /// `a^b`.
    Pow,
    /// `Root(n, x)`: the `n`th root of `x` (`\sqrt[n]{x}`), real for an odd `n` and a negative `x`.
    Root,
}

/// A function of one argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    Sqrt,
    Sin,
    Cos,
    Tan,
    Sec,
    Csc,
    Cot,
    /// Natural logarithm.
    Ln,
    /// Base-10 logarithm (`\log` without a base).
    Log10,
    Exp,
    Sinh,
    Cosh,
    Tanh,
    Sech,
    Csch,
    Coth,
    Abs,
    /// −1, 0 or 1 (the derivative of `\abs`; not written in sources).
    Sign,
}

impl Func {
    /// The function's LaTeX name, without the backslash.
    pub fn name(self) -> &'static str {
        match self {
            Func::Sqrt => "sqrt",
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Sec => "sec",
            Func::Csc => "csc",
            Func::Cot => "cot",
            Func::Ln => "ln",
            Func::Log10 => "log",
            Func::Exp => "exp",
            Func::Sinh => "sinh",
            Func::Cosh => "cosh",
            Func::Tanh => "tanh",
            Func::Sech => "sech",
            Func::Csch => "csch",
            Func::Coth => "coth",
            Func::Abs => "abs",
            Func::Sign => "sign",
        }
    }

    fn by_name(name: &str) -> Option<Func> {
        Some(match name {
            "sqrt" => Func::Sqrt,
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "sec" => Func::Sec,
            "csc" => Func::Csc,
            "cot" => Func::Cot,
            "ln" => Func::Ln,
            "log" => Func::Log10,
            "exp" => Func::Exp,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "sech" => Func::Sech,
            "csch" => Func::Csch,
            "coth" => Func::Coth,
            "abs" => Func::Abs,
            _ => return None,
        })
    }

    fn apply(self, x: f64) -> f64 {
        match self {
            Func::Sqrt => x.sqrt(),
            Func::Sin => x.sin(),
            Func::Cos => x.cos(),
            Func::Tan => x.tan(),
            Func::Sec => 1.0 / x.cos(),
            Func::Csc => 1.0 / x.sin(),
            Func::Cot => x.cos() / x.sin(),
            Func::Ln => x.ln(),
            Func::Log10 => x.log10(),
            Func::Exp => x.exp(),
            Func::Sinh => x.sinh(),
            Func::Cosh => x.cosh(),
            Func::Tanh => x.tanh(),
            Func::Sech => 1.0 / x.cosh(),
            Func::Csch => 1.0 / x.sinh(),
            Func::Coth => 1.0 / x.tanh(),
            Func::Abs => x.abs(),
            Func::Sign => {
                if x > 0.0 {
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    // 0 stays 0 (and NaN stays NaN).
                    x
                }
            }
        }
    }
}

/// An expression tree.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    /// A variable, by name.
    Var(String),
    Bin(Op, Box<Expr>, Box<Expr>),
    Call(Func, Box<Expr>),
}

/// The `n`th root of `x`: real for an odd integer `n` and a negative `x`.
fn root(n: f64, x: f64) -> f64 {
    let odd = n.fract() == 0.0 && (n % 2.0).abs() == 1.0;
    if x < 0.0 && odd { -(-x).powf(1.0 / n) } else { x.powf(1.0 / n) }
}

impl Expr {
    /// The value with each variable taken from `env` (a variable `env` doesn't name is NaN).
    pub fn eval(&self, env: &[(&str, f64)]) -> f64 {
        self.eval_with(&|name| env.iter().find(|(n, _)| *n == name).map_or(f64::NAN, |(_, v)| *v))
    }

    /// The value of an expression in one variable at `x` (every variable is taken to be `x`).
    pub fn eval1(&self, x: f64) -> f64 {
        self.eval_with(&|_| x)
    }

    fn eval_with(&self, var: &dyn Fn(&str) -> f64) -> f64 {
        match self {
            Expr::Num(n) => *n,
            Expr::Var(v) => var(v),
            Expr::Call(f, a) => f.apply(a.eval_with(var)),
            Expr::Bin(op, a, b) => {
                let (a, b) = (a.eval_with(var), b.eval_with(var));
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                    Op::Pow => a.powf(b),
                    Op::Root => root(a, b),
                }
            }
        }
    }

    /// How many nodes the tree has.
    pub fn size(&self) -> usize {
        match self {
            Expr::Num(_) | Expr::Var(_) => 1,
            Expr::Call(_, a) => 1 + a.size(),
            Expr::Bin(_, a, b) => 1 + a.size() + b.size(),
        }
    }

    /// How deep the tree is (a number or a variable is 1).
    pub fn depth(&self) -> usize {
        match self {
            Expr::Num(_) | Expr::Var(_) => 1,
            Expr::Call(_, a) => 1 + a.depth(),
            Expr::Bin(_, a, b) => 1 + a.depth().max(b.depth()),
        }
    }

    /// Whether the expression uses the variable `var`.
    pub fn uses(&self, var: &str) -> bool {
        match self {
            Expr::Num(_) => false,
            Expr::Var(v) => v == var,
            Expr::Call(_, a) => a.uses(var),
            Expr::Bin(_, a, b) => a.uses(var) || b.uses(var),
        }
    }

    /// The derivative with respect to `var`, simplified only trivially (`0·a`, `1·a`, `a+0`,
    /// numbers folded) so that repeated derivatives stay small. Fails when the derivative would
    /// outgrow [`MAX_DIFF_NODES`] or [`MAX_TREE_DEPTH`].
    pub fn diff(&self, var: &str) -> Result<Expr, EvalError> {
        let mut budget = MAX_DIFF_NODES;
        let d = self.d(var, &mut budget)?;
        if d.depth() > MAX_TREE_DEPTH {
            return Err(EvalError::TooDeep { max: MAX_TREE_DEPTH });
        }
        Ok(d)
    }

    /// A copy of `self`, charged to `budget`.
    fn cl(&self, budget: &mut usize) -> Result<Expr, EvalError> {
        *budget = budget.checked_sub(self.size()).ok_or(EvalError::TooLarge { max: MAX_DIFF_NODES })?;
        Ok(self.clone())
    }

    fn d(&self, var: &str, budget: &mut usize) -> Result<Expr, EvalError> {
        *budget = budget.checked_sub(1).ok_or(EvalError::TooLarge { max: MAX_DIFF_NODES })?;
        if !self.uses(var) {
            return Ok(num(0.0));
        }
        Ok(match self {
            Expr::Num(_) => num(0.0),
            Expr::Var(_) => num(1.0),
            Expr::Bin(op, a, b) => {
                let (da, db) = (a.d(var, budget)?, b.d(var, budget)?);
                match op {
                    Op::Add => add(da, db),
                    Op::Sub => sub(da, db),
                    Op::Mul => add(mul(da, b.cl(budget)?), mul(a.cl(budget)?, db)),
                    Op::Div if !b.uses(var) => div(da, b.cl(budget)?),
                    Op::Div => {
                        let top = sub(mul(da, b.cl(budget)?), mul(a.cl(budget)?, db));
                        div(top, pow(b.cl(budget)?, num(2.0)))
                    }
                    // a^c: c·a^(c−1)·a′.
                    Op::Pow if !b.uses(var) => mul(mul(b.cl(budget)?, pow(a.cl(budget)?, sub(b.cl(budget)?, num(1.0)))), da),
                    // c^b: c^b·ln c·b′.
                    Op::Pow if !a.uses(var) => mul(mul(self.cl(budget)?, call(Func::Ln, a.cl(budget)?)), db),
                    // a^b·(b′·ln a + b·a′/a).
                    Op::Pow => {
                        let inner = add(mul(db, call(Func::Ln, a.cl(budget)?)), div(mul(b.cl(budget)?, da), a.cl(budget)?));
                        mul(self.cl(budget)?, inner)
                    }
                    // A constant index: ⁿ√x · x′ / (n·x).
                    Op::Root if !a.uses(var) => div(mul(self.cl(budget)?, db), mul(a.cl(budget)?, b.cl(budget)?)),
                    // A variable index: as x^(1/n).
                    Op::Root => pow(b.cl(budget)?, div(num(1.0), a.cl(budget)?)).d(var, budget)?,
                }
            }
            Expr::Call(f, a) => {
                let da = a.d(var, budget)?;
                let u = |budget: &mut usize| a.cl(budget);
                let outer = match f {
                    Func::Sqrt => div(num(1.0), mul(num(2.0), self.cl(budget)?)),
                    Func::Sin => call(Func::Cos, u(budget)?),
                    Func::Cos => neg(call(Func::Sin, u(budget)?)),
                    Func::Tan => pow(call(Func::Sec, u(budget)?), num(2.0)),
                    Func::Sec => mul(self.cl(budget)?, call(Func::Tan, u(budget)?)),
                    Func::Csc => neg(mul(self.cl(budget)?, call(Func::Cot, u(budget)?))),
                    Func::Cot => neg(pow(call(Func::Csc, u(budget)?), num(2.0))),
                    Func::Ln => div(num(1.0), u(budget)?),
                    Func::Log10 => div(num(1.0), mul(u(budget)?, num(std::f64::consts::LN_10))),
                    Func::Exp => self.cl(budget)?,
                    Func::Sinh => call(Func::Cosh, u(budget)?),
                    Func::Cosh => call(Func::Sinh, u(budget)?),
                    Func::Tanh => pow(call(Func::Sech, u(budget)?), num(2.0)),
                    Func::Sech => neg(mul(self.cl(budget)?, call(Func::Tanh, u(budget)?))),
                    Func::Csch => neg(mul(self.cl(budget)?, call(Func::Coth, u(budget)?))),
                    Func::Coth => neg(pow(call(Func::Csch, u(budget)?), num(2.0))),
                    Func::Abs => call(Func::Sign, u(budget)?),
                    Func::Sign => num(0.0),
                };
                mul(outer, da)
            }
        })
    }
}

// ---------- constructors with trivial simplification ----------

fn num(n: f64) -> Expr {
    Expr::Num(n)
}

fn is_num(e: &Expr, n: f64) -> bool {
    matches!(e, Expr::Num(v) if *v == n)
}

fn bin(op: Op, a: Expr, b: Expr) -> Expr {
    Expr::Bin(op, Box::new(a), Box::new(b))
}

fn call(f: Func, a: Expr) -> Expr {
    match a {
        Expr::Num(n) if f != Func::Sign => num(f.apply(n)),
        a => Expr::Call(f, Box::new(a)),
    }
}

fn add(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => num(x + y),
        (a, b) if is_num(&b, 0.0) => a,
        (a, b) if is_num(&a, 0.0) => b,
        (a, b) => bin(Op::Add, a, b),
    }
}

fn sub(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => num(x - y),
        (a, b) if is_num(&b, 0.0) => a,
        (a, b) => bin(Op::Sub, a, b),
    }
}

fn neg(a: Expr) -> Expr {
    sub(num(0.0), a)
}

fn mul(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => num(x * y),
        (a, b) if is_num(&a, 0.0) || is_num(&b, 0.0) => num(0.0),
        (a, b) if is_num(&a, 1.0) => b,
        (a, b) if is_num(&b, 1.0) => a,
        (a, b) => bin(Op::Mul, a, b),
    }
}

fn div(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) if y != 0.0 => num(x / y),
        (a, b) if is_num(&b, 1.0) => a,
        (a, b) if is_num(&a, 0.0) && !is_num(&b, 0.0) => num(0.0),
        (a, b) => bin(Op::Div, a, b),
    }
}

fn pow(a: Expr, b: Expr) -> Expr {
    match (a, b) {
        (Expr::Num(x), Expr::Num(y)) => num(x.powf(y)),
        (a, b) if is_num(&b, 1.0) => a,
        (_, b) if is_num(&b, 0.0) => num(1.0),
        (a, b) => bin(Op::Pow, a, b),
    }
}

// ---------- display ----------

/// Fully bracketed, the way the tool this language comes from shows a parsed expression:
/// `3x^2+\sin x*\cos x` → `((3*(x^2))+((\sin{x})*(\cos{x})))`.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{n}"),
            Expr::Var(v) => f.write_str(v),
            Expr::Call(func, a) => write!(f, "(\\{}{{{a}}})", func.name()),
            Expr::Bin(Op::Root, n, x) => write!(f, "(\\sqrt[{n}]{{{x}}})"),
            Expr::Bin(op, a, b) => {
                let o = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Pow | Op::Root => "^",
                };
                write!(f, "({a}{o}{b})")
            }
        }
    }
}

// ---------- tokens ----------

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Letter(char),
    Cmd(String),
    /// `+ - * / ^ _`
    Op(char),
    Open(char),
    Close(char),
}

fn syntax(message: impl Into<String>, position: usize) -> EvalError {
    EvalError::Syntax { message: message.into(), position }
}

fn tokenize(src: &str) -> Result<Vec<(Tok, usize)>, EvalError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let at = i;
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)) {
            let mut text = String::new();
            let mut dot = false;
            while let Some(&d) = chars.get(i) {
                if d.is_ascii_digit() || (d == '.' && !dot) {
                    dot |= d == '.';
                    text.push(d);
                    i += 1;
                } else {
                    break;
                }
            }
            let n = text.parse::<f64>().map_err(|_| syntax(format!("bad number {text}"), at))?;
            out.push((Tok::Num(n), at));
        } else if c.is_ascii_alphabetic() {
            out.push((Tok::Letter(c), at));
            i += 1;
        } else if c == '\\' {
            i += 1;
            let mut name = String::new();
            while let Some(&d) = chars.get(i).filter(|d| d.is_ascii_alphabetic()) {
                name.push(d);
                i += 1;
            }
            if name.is_empty() {
                // `\,` `\;` `\:` `\!` `\ ` are spaces; `\{` `\}` brackets.
                match chars.get(i) {
                    Some(',' | ';' | ':' | '!' | ' ') => {}
                    Some('{') => out.push((Tok::Open('{'), at)),
                    Some('}') => out.push((Tok::Close('}'), at)),
                    _ => return Err(syntax("a backslash must start a command", at)),
                }
                i += 1;
                continue;
            }
            match name.as_str() {
                "left" | "right" => {}
                "cdot" | "times" => out.push((Tok::Op('*'), at)),
                "div" => out.push((Tok::Op('/'), at)),
                _ => out.push((Tok::Cmd(name), at)),
            }
        } else if matches!(c, '+' | '-' | '*' | '/' | '^' | '_') {
            out.push((Tok::Op(c), at));
            i += 1;
        } else if c == '·' || c == '×' {
            out.push((Tok::Op('*'), at));
            i += 1;
        } else if c == '−' {
            out.push((Tok::Op('-'), at));
            i += 1;
        } else if matches!(c, '(' | '{' | '[') {
            out.push((Tok::Open(c), at));
            i += 1;
        } else if matches!(c, ')' | '}' | ']') {
            out.push((Tok::Close(c), at));
            i += 1;
        } else {
            return Err(syntax(format!("unexpected {c:?}"), at));
        }
    }
    Ok(out)
}

// ---------- parser ----------

/// Reads `src` as an expression in the variables `vars` (single letters, such as `["x"]`, or
/// `["t"]`, or `["f", "g", "h", "x"]`). A letter that isn't one of them is an error, except `e`.
pub fn parse_expr(src: &str, vars: &[&str]) -> Result<Expr, EvalError> {
    if src.chars().count() > MAX_EXPR_CHARS {
        return Err(EvalError::TooLong { max: MAX_EXPR_CHARS });
    }
    let toks = tokenize(src)?;
    if toks.is_empty() {
        return Err(EvalError::Empty);
    }
    let mut p = Parser { toks, at: 0, vars, end: src.chars().count() };
    let e = p.sum(0)?;
    if let Some((t, pos)) = p.toks.get(p.at) {
        let what = match t {
            Tok::Close(c) => format!("unmatched {c:?}"),
            _ => "unexpected text".to_string(),
        };
        return Err(syntax(what, *pos));
    }
    if e.size() > MAX_EXPR_NODES {
        return Err(EvalError::TooLarge { max: MAX_EXPR_NODES });
    }
    if e.depth() > MAX_TREE_DEPTH {
        return Err(EvalError::TooDeep { max: MAX_TREE_DEPTH });
    }
    Ok(e)
}

struct Parser<'a> {
    toks: Vec<(Tok, usize)>,
    at: usize,
    vars: &'a [&'a str],
    /// The source's length, the position of "the end".
    end: usize,
}

fn closing(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        _ => '}',
    }
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at).map(|(t, _)| t)
    }

    fn pos(&self) -> usize {
        self.toks.get(self.at).map_or(self.end, |(_, p)| *p)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).map(|(t, _)| t.clone());
        self.at += 1;
        t
    }

    fn eat_op(&mut self, op: char) -> bool {
        if self.peek() == Some(&Tok::Op(op)) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn deeper(depth: usize) -> Result<usize, EvalError> {
        if depth >= MAX_EXPR_DEPTH { Err(EvalError::TooDeep { max: MAX_EXPR_DEPTH }) } else { Ok(depth + 1) }
    }

    /// `term (('+' | '-') term)*`
    fn sum(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let mut e = self.term(depth)?;
        loop {
            if self.eat_op('+') {
                e = bin(Op::Add, e, self.term(depth)?);
            } else if self.eat_op('-') {
                e = bin(Op::Sub, e, self.term(depth)?);
            } else {
                return Ok(e);
            }
        }
    }

    /// Whether the next token can start an operand (for implicit multiplication).
    fn starts_operand(&self) -> bool {
        matches!(self.peek(), Some(Tok::Num(_) | Tok::Letter(_) | Tok::Cmd(_) | Tok::Open(_)))
    }

    /// `signed (('*' | '/') signed | operand)*`
    fn term(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let mut e = self.signed(depth)?;
        loop {
            if self.eat_op('*') {
                e = bin(Op::Mul, e, self.signed(depth)?);
            } else if self.eat_op('/') {
                e = bin(Op::Div, e, self.signed(depth)?);
            } else if self.starts_operand() {
                e = bin(Op::Mul, e, self.power(depth)?);
            } else {
                return Ok(e);
            }
        }
    }

    /// Leading signs, then a power: `-x^2` is `0 − x^2`.
    fn signed(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let mut negative = false;
        loop {
            if self.eat_op('-') {
                negative = !negative;
            } else if !self.eat_op('+') {
                break;
            }
        }
        let e = self.power(depth)?;
        Ok(if negative { bin(Op::Sub, num(0.0), e) } else { e })
    }

    /// `operand ('^' exponent)?`, right associative.
    fn power(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let base = self.operand(depth)?;
        if self.eat_op('^') {
            let exp = self.exponent(depth)?;
            return Ok(bin(Op::Pow, base, exp));
        }
        Ok(base)
    }

    /// What follows `^`: a signed power (`x^-2`, `2^2^3` = `2^(2^3)`).
    fn exponent(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let depth = Self::deeper(depth)?;
        if self.eat_op('-') {
            let e = self.power(depth)?;
            return Ok(bin(Op::Sub, num(0.0), e));
        }
        self.eat_op('+');
        self.power(depth)
    }

    /// A function's argument: an operand, possibly signed (`\sin -x`).
    fn argument(&mut self, depth: usize, of: &str) -> Result<Expr, EvalError> {
        let depth = Self::deeper(depth)?;
        let mut negative = false;
        loop {
            if self.eat_op('-') {
                negative = !negative;
            } else if !self.eat_op('+') {
                break;
            }
        }
        if !self.starts_operand() {
            return Err(syntax(format!("\\{of} needs an argument"), self.pos()));
        }
        let e = self.operand(depth)?;
        Ok(if negative { bin(Op::Sub, num(0.0), e) } else { e })
    }

    /// A number, a variable, a constant, a bracketed group or a function applied to its arguments.
    fn operand(&mut self, depth: usize) -> Result<Expr, EvalError> {
        let pos = self.pos();
        match self.next() {
            Some(Tok::Num(n)) => Ok(num(n)),
            Some(Tok::Letter(c)) => {
                let mut buf = [0u8; 4];
                let name: &str = c.encode_utf8(&mut buf);
                if self.vars.contains(&name) {
                    Ok(Expr::Var(name.to_string()))
                } else if c == 'e' {
                    Ok(num(std::f64::consts::E))
                } else {
                    Err(EvalError::UnknownVariable(c))
                }
            }
            Some(Tok::Open(open)) => {
                let depth = Self::deeper(depth)?;
                let e = self.sum(depth)?;
                match self.next() {
                    Some(Tok::Close(c)) if c == closing(open) => Ok(e),
                    _ => Err(syntax(format!("{open:?} is not closed"), pos)),
                }
            }
            Some(Tok::Cmd(name)) => self.command(&name, depth, pos),
            Some(Tok::Close(c)) => Err(syntax(format!("unmatched {c:?}"), pos)),
            Some(Tok::Op(o)) => Err(syntax(format!("{o:?} needs a number or variable before it"), pos)),
            None => Err(syntax("the expression ends too early", pos)),
        }
    }

    fn command(&mut self, name: &str, depth: usize, pos: usize) -> Result<Expr, EvalError> {
        match name {
            "pi" => Ok(num(std::f64::consts::PI)),
            "frac" | "dfrac" | "tfrac" => {
                let a = self.argument(depth, name)?;
                let b = self.argument(depth, name)?;
                Ok(bin(Op::Div, a, b))
            }
            "pow" => {
                let a = self.argument(depth, name)?;
                let b = self.argument(depth, name)?;
                Ok(bin(Op::Pow, a, b))
            }
            "sqrt" if self.peek() == Some(&Tok::Open('[')) => {
                self.at += 1;
                let inner = Self::deeper(depth)?;
                let n = self.sum(inner)?;
                if self.next() != Some(Tok::Close(']')) {
                    return Err(syntax("'[' is not closed", pos));
                }
                if n.uses_any() {
                    return Err(syntax("the index of \\sqrt[n] must be a number", pos));
                }
                let x = self.argument(depth, name)?;
                Ok(bin(Op::Root, n, x))
            }
            "log" if self.eat_op('_') => {
                let base = self.argument(depth, name)?;
                let power = self.function_power(depth, name)?;
                let x = self.argument(depth, name)?;
                let value = bin(Op::Div, Expr::Call(Func::Ln, Box::new(x)), Expr::Call(Func::Ln, Box::new(base)));
                Ok(match power {
                    Some(p) => bin(Op::Pow, value, p),
                    None => value,
                })
            }
            _ => {
                let Some(f) = Func::by_name(name) else {
                    return Err(EvalError::UnknownCommand(name.to_string()));
                };
                let power = self.function_power(depth, name)?;
                let x = self.argument(depth, name)?;
                let value = Expr::Call(f, Box::new(x));
                Ok(match power {
                    Some(p) => bin(Op::Pow, value, p),
                    None => value,
                })
            }
        }
    }

    /// `^n` right after a function's name (`\sin^2 x`).
    fn function_power(&mut self, depth: usize, name: &str) -> Result<Option<Expr>, EvalError> {
        if self.eat_op('^') { Ok(Some(self.argument(depth, &format!("{name}^"))?)) } else { Ok(None) }
    }
}

impl Expr {
    /// Whether any variable appears.
    fn uses_any(&self) -> bool {
        match self {
            Expr::Num(_) => false,
            Expr::Var(_) => true,
            Expr::Call(_, a) => a.uses_any(),
            Expr::Bin(_, a, b) => a.uses_any() || b.uses_any(),
        }
    }
}
