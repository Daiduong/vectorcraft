//! A LaTeX math subset → syntax tree.
//!
//! The input is untrusted (typed into a dialog, pasted, sent over MCP): [`parse`] bounds the source
//! length and the nesting depth, never panics, and says what it could not read and where.

use crate::MathError;
use crate::symbols::{BIG_OPS, ESCAPED, FUNCTIONS, class_of_char, lookup_symbol};

/// Longest source [`parse`] reads, in characters.
pub const MAX_SOURCE_CHARS: usize = 10_000;
/// Deepest nesting of groups and commands [`parse`] follows.
pub const MAX_DEPTH: usize = 64;

/// TeX's atom types: what spacing a piece of the formula gets next to its neighbours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomClass {
    /// Ordinary: letters, digits, most symbols.
    Ord,
    /// Large operator or function name (`\sum`, `\sin`).
    Op,
    /// Binary operator (`+`, `\times`).
    Bin,
    /// Relation (`=`, `\le`).
    Rel,
    /// Opening delimiter.
    Open,
    /// Closing delimiter.
    Close,
    /// Punctuation (`,` `;`).
    Punct,
    /// A delimited sub-formula, or dots.
    Inner,
}

/// The font style letters are drawn in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Math italic letters, upright digits and symbols.
    #[default]
    Math,
    /// `\mathrm`.
    Roman,
    /// `\mathbf`.
    Bold,
    /// `\mathit`.
    Italic,
    /// `\bm`.
    BoldItalic,
}

/// Which style a fraction is set in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FracStyle {
    /// The style around it (`\frac`).
    Auto,
    /// Display style (`\dfrac`).
    Display,
    /// Text style (`\tfrac`).
    Text,
}

/// Whether a big operator puts its scripts above and below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limits {
    /// The operator's habit: above and below for sums in display style, beside for integrals.
    Auto,
    /// `\limits`.
    Limits,
    /// `\nolimits`.
    NoLimits,
}

/// An accent over a letter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentKind {
    Hat,
    Bar,
    Tilde,
    Vec,
    Dot,
    Ddot,
}

/// A rule or arrow that spans its argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WideKind {
    Overline,
    Underline,
    OverRightArrow,
    OverLeftArrow,
}

/// Horizontal space written as a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Space {
    /// `\,` (3/18 em).
    Thin,
    /// `\:` (4/18 em).
    Medium,
    /// `\;` (5/18 em).
    Thick,
    /// `\quad` (1 em).
    Quad,
    /// `\qquad` (2 em).
    QuadQuad,
    /// `\!` (-3/18 em).
    NegThin,
    /// `~` and `\ ` (a word space).
    Normal,
}

/// A style switch: `\displaystyle`, `\textstyle`, `\scriptstyle`, `\scriptscriptstyle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleLevel {
    Display,
    Text,
    Script,
    ScriptScript,
}

/// A node of the syntax tree.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// One character, drawn as a glyph (letters are mapped to their style by the layout).
    Atom { ch: char, class: AtomClass },
    /// `{…}`: its contents set as one unit.
    Group(Vec<Node>),
    /// A nucleus with a subscript and/or a superscript (`x_i^2`, `f'`).
    Scripts { base: Box<Node>, sub: Option<Box<Node>>, sup: Option<Box<Node>> },
    /// `\frac{num}{den}`.
    Frac { num: Box<Node>, den: Box<Node>, style: FracStyle },
    /// `\sqrt[degree]{body}`.
    Sqrt { degree: Option<Box<Node>>, body: Box<Node> },
    /// `\left open … \right close`; `None` is the null delimiter `.`.
    Fenced { open: Option<char>, close: Option<char>, body: Vec<Node> },
    /// A large operator symbol (`\sum`, `\int`).
    BigOp { ch: char, limits: Limits },
    /// An operator name set upright (`\sin`, `\lim`); `limits` puts the scripts of `\lim` below
    /// it in display style.
    Func { name: String, limits: bool },
    /// `\hat{x}` and the other accents.
    Accent { kind: AccentKind, body: Box<Node> },
    /// `\overline{…}` and the other wide marks.
    Wide { kind: WideKind, body: Box<Node> },
    /// Explicit space.
    Space(Space),
    /// `\text{…}`: upright text, spaces kept.
    Text(String),
    /// `\mathrm{…}`, `\mathbf{…}`, `\mathit{…}`, `\bm{…}`.
    Styled { variant: Variant, body: Vec<Node> },
    /// `\displaystyle …`: the rest of its group in another style.
    StyleScope { level: StyleLevel, body: Vec<Node> },
}

impl Node {
    /// The atom type spacing treats the node as; `None` for space, which isn't an atom.
    pub fn atom_class(&self) -> Option<AtomClass> {
        match self {
            Node::Atom { class, .. } => Some(*class),
            Node::BigOp { .. } | Node::Func { .. } => Some(AtomClass::Op),
            Node::Fenced { .. } => Some(AtomClass::Inner),
            Node::Scripts { base, .. } => base.atom_class(),
            Node::Space(_) => None,
            Node::Group(_)
            | Node::Frac { .. }
            | Node::Sqrt { .. }
            | Node::Accent { .. }
            | Node::Wide { .. }
            | Node::Text(_)
            | Node::Styled { .. }
            | Node::StyleScope { .. } => Some(AtomClass::Ord),
        }
    }
}

/// Parses a formula. An empty (or blank) source is an empty tree.
///
/// Fails with [`MathError::UnknownCommand`] for a command outside the supported subset,
/// [`MathError::Syntax`] for malformed input, [`MathError::TooLong`] past [`MAX_SOURCE_CHARS`] and
/// [`MathError::TooDeep`] past [`MAX_DEPTH`].
pub fn parse(src: &str) -> Result<Vec<Node>, MathError> {
    if src.chars().take(MAX_SOURCE_CHARS + 1).count() > MAX_SOURCE_CHARS {
        return Err(MathError::TooLong { max: MAX_SOURCE_CHARS });
    }
    let mut p = Parser { chars: src.chars().collect(), pos: 0, depth: 0 };
    p.parse_row(End::Eof)
}

/// What ends the row being read.
#[derive(Clone, Copy, PartialEq, Eq)]
enum End {
    Eof,
    /// A `}`.
    Brace,
    /// A `]` (the optional argument of `\sqrt`).
    Bracket,
    /// A `\right`.
    Right,
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn err<T>(&self, message: &str) -> Result<T, MathError> {
        Err(MathError::Syntax { message: message.to_string(), position: self.pos })
    }

    fn expect(&mut self, c: char) -> Result<(), MathError> {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            self.err(&format!("expected '{c}'"))
        }
    }

    /// Counts one more level of nesting.
    fn enter(&mut self) -> Result<(), MathError> {
        if self.depth >= MAX_DEPTH {
            return Err(MathError::TooDeep { max: MAX_DEPTH });
        }
        self.depth += 1;
        Ok(())
    }

    /// Is the next thing the command `\name` (and not a longer one)?
    fn at_command(&self, name: &str) -> bool {
        if self.peek() != Some('\\') {
            return false;
        }
        let mut i = self.pos + 1;
        for c in name.chars() {
            if self.chars.get(i) != Some(&c) {
                return false;
            }
            i += 1;
        }
        !self.chars.get(i).is_some_and(char::is_ascii_alphabetic)
    }

    /// Reads `\name` (the backslash is consumed): a run of letters, or one other character.
    fn command_name(&mut self) -> Result<String, MathError> {
        let mut name = String::new();
        while let Some(c) = self.peek().filter(char::is_ascii_alphabetic) {
            name.push(c);
            self.pos += 1;
        }
        if name.is_empty() {
            match self.bump() {
                Some(c) => name.push(c),
                None => return self.err("a backslash at the end of the formula"),
            }
        }
        Ok(name)
    }

    /// Reads items until the row's terminator, which is left unread.
    fn parse_row(&mut self, end: End) -> Result<Vec<Node>, MathError> {
        self.enter()?;
        let mut out = Vec::new();
        let r = self.row_items(end, &mut out);
        self.depth -= 1;
        r.map(|()| out)
    }

    fn row_items(&mut self, end: End, out: &mut Vec<Node>) -> Result<(), MathError> {
        loop {
            self.skip_ws();
            let Some(c) = self.peek() else {
                return match end {
                    End::Eof => Ok(()),
                    End::Brace => self.err("missing '}'"),
                    End::Bracket => self.err("missing ']'"),
                    End::Right => self.err("\\left without a matching \\right"),
                };
            };
            match (c, end) {
                ('}', End::Brace) | (']', End::Bracket) => return Ok(()),
                ('}', _) => return self.err("unexpected '}'"),
                ('\\', End::Right) if self.at_command("right") => return Ok(()),
                _ => {}
            }
            if let Some(level) = self.style_switch() {
                // The switch holds to the end of the group: the rest of the row is its body.
                let body = self.parse_row(end)?;
                out.push(Node::StyleScope { level, body });
                return Ok(());
            }
            out.push(self.parse_item()?);
        }
    }

    /// Reads `\displaystyle` and its kin when they come next.
    fn style_switch(&mut self) -> Option<StyleLevel> {
        const SWITCHES: [(&str, StyleLevel); 4] = [
            ("displaystyle", StyleLevel::Display),
            ("textstyle", StyleLevel::Text),
            ("scriptstyle", StyleLevel::Script),
            ("scriptscriptstyle", StyleLevel::ScriptScript),
        ];
        let (name, level) = SWITCHES.into_iter().find(|(n, _)| self.at_command(n))?;
        self.pos += 1 + name.len();
        Some(level)
    }

    /// A nucleus and the scripts that follow it.
    fn parse_item(&mut self) -> Result<Node, MathError> {
        let base = if matches!(self.peek(), Some('^' | '_' | '\'')) { Node::Group(Vec::new()) } else { self.nucleus()? };
        self.parse_scripts(base)
    }

    fn parse_scripts(&mut self, base: Node) -> Result<Node, MathError> {
        let (mut sub, mut sup): (Option<Node>, Option<Node>) = (None, None);
        // The superscript so far is only primes (`f'`): a `^` after them joins it, as in TeX.
        let mut only_primes = false;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('^') => {
                    self.pos += 1;
                    let arg = self.parse_arg()?;
                    match sup.take() {
                        None => sup = Some(arg),
                        Some(Node::Group(mut primes)) if only_primes => {
                            primes.push(arg);
                            sup = Some(Node::Group(primes));
                        }
                        Some(_) => return self.err("double superscript"),
                    }
                    only_primes = false;
                }
                Some('_') => {
                    self.pos += 1;
                    let arg = self.parse_arg()?;
                    if sub.replace(arg).is_some() {
                        return self.err("double subscript");
                    }
                }
                Some('\'') => {
                    if sup.is_some() {
                        return self.err("double superscript");
                    }
                    let mut primes = Vec::new();
                    while self.peek() == Some('\'') {
                        self.pos += 1;
                        primes.push(Node::Atom { ch: '\u{2032}', class: AtomClass::Ord });
                    }
                    sup = Some(Node::Group(primes));
                    only_primes = true;
                }
                _ => break,
            }
        }
        if sub.is_none() && sup.is_none() {
            return Ok(base);
        }
        Ok(Node::Scripts { base: Box::new(base), sub: sub.map(Box::new), sup: sup.map(Box::new) })
    }

    /// The argument of a command or script: a group or a single token.
    fn parse_arg(&mut self) -> Result<Node, MathError> {
        self.skip_ws();
        match self.peek() {
            None | Some('}') => self.err("missing argument"),
            _ => self.nucleus(),
        }
    }

    fn nucleus(&mut self) -> Result<Node, MathError> {
        self.skip_ws();
        let at = self.pos;
        let Some(c) = self.bump() else { return self.err("unexpected end of the formula") };
        match c {
            '{' => {
                let body = self.parse_row(End::Brace)?;
                self.expect('}')?;
                Ok(Node::Group(body))
            }
            '\\' => {
                // Commands nest (`\frac\frac…`): count them; groups are counted by `parse_row`.
                self.enter()?;
                let r = self.command();
                self.depth -= 1;
                r
            }
            '^' | '_' => {
                self.pos = at;
                self.err("a script needs something to attach to")
            }
            '}' => {
                self.pos = at;
                self.err("unexpected '}'")
            }
            '~' => Ok(Node::Space(Space::Normal)),
            '\'' => Ok(Node::Atom { ch: '\u{2032}', class: AtomClass::Ord }),
            '-' => Ok(Node::Atom { ch: '\u{2212}', class: AtomClass::Bin }),
            '*' => Ok(Node::Atom { ch: '\u{2217}', class: AtomClass::Bin }),
            '#' | '$' | '%' | '&' | '"' | '@' | '`' => {
                self.pos = at;
                self.err(&format!("'{c}' is not supported in a formula"))
            }
            c if c.is_control() => {
                self.pos = at;
                self.err("a control character in the formula")
            }
            c => Ok(Node::Atom { ch: c, class: class_of_char(c) }),
        }
    }

    /// The command after a backslash that has been read.
    fn command(&mut self) -> Result<Node, MathError> {
        let name = self.command_name()?;
        match name.as_str() {
            "frac" | "dfrac" | "tfrac" => {
                let style = match name.as_str() {
                    "dfrac" => FracStyle::Display,
                    "tfrac" => FracStyle::Text,
                    _ => FracStyle::Auto,
                };
                let num = Box::new(self.parse_arg()?);
                let den = Box::new(self.parse_arg()?);
                Ok(Node::Frac { num, den, style })
            }
            "sqrt" => {
                self.skip_ws();
                let degree = if self.peek() == Some('[') {
                    self.pos += 1;
                    let body = self.parse_row(End::Bracket)?;
                    self.expect(']')?;
                    Some(Box::new(Node::Group(body)))
                } else {
                    None
                };
                let body = Box::new(self.parse_arg()?);
                Ok(Node::Sqrt { degree, body })
            }
            "left" => {
                let open = self.delimiter()?;
                let body = self.parse_row(End::Right)?;
                self.pos += "\\right".len();
                let close = self.delimiter()?;
                Ok(Node::Fenced { open, close, body })
            }
            "right" => self.err("\\right without a matching \\left"),
            "hat" | "bar" | "tilde" | "vec" | "dot" | "ddot" => {
                let kind = match name.as_str() {
                    "hat" => AccentKind::Hat,
                    "bar" => AccentKind::Bar,
                    "tilde" => AccentKind::Tilde,
                    "vec" => AccentKind::Vec,
                    "dot" => AccentKind::Dot,
                    _ => AccentKind::Ddot,
                };
                Ok(Node::Accent { kind, body: Box::new(self.parse_arg()?) })
            }
            "overline" | "underline" | "overrightarrow" | "overleftarrow" => {
                let kind = match name.as_str() {
                    "overline" => WideKind::Overline,
                    "underline" => WideKind::Underline,
                    "overrightarrow" => WideKind::OverRightArrow,
                    _ => WideKind::OverLeftArrow,
                };
                Ok(Node::Wide { kind, body: Box::new(self.parse_arg()?) })
            }
            "text" | "textrm" | "mbox" => Ok(Node::Text(self.raw_text()?)),
            "operatorname" => {
                let name = self.raw_text()?;
                if name.trim().is_empty() {
                    return self.err("\\operatorname needs a name");
                }
                Ok(Node::Func { name, limits: false })
            }
            "mathrm" | "mathbf" | "mathit" | "bm" | "boldsymbol" => {
                let variant = match name.as_str() {
                    "mathrm" => Variant::Roman,
                    "mathbf" => Variant::Bold,
                    "mathit" => Variant::Italic,
                    _ => Variant::BoldItalic,
                };
                let body = match self.parse_arg()? {
                    Node::Group(items) => items,
                    one => vec![one],
                };
                Ok(Node::Styled { variant, body })
            }
            "," => Ok(Node::Space(Space::Thin)),
            ":" => Ok(Node::Space(Space::Medium)),
            ";" => Ok(Node::Space(Space::Thick)),
            "!" => Ok(Node::Space(Space::NegThin)),
            " " => Ok(Node::Space(Space::Normal)),
            "quad" => Ok(Node::Space(Space::Quad)),
            "qquad" => Ok(Node::Space(Space::QuadQuad)),
            "limits" | "nolimits" => self.err(&format!("\\{name} must follow a big operator")),
            _ => self.named_symbol(name),
        }
    }

    /// A command that stands for a symbol, big operator or function name.
    fn named_symbol(&mut self, name: String) -> Result<Node, MathError> {
        if let Some(&(_, ch)) = BIG_OPS.iter().find(|(n, _)| *n == name) {
            let limits = match self.limits_modifier() {
                Some(true) => Limits::Limits,
                Some(false) => Limits::NoLimits,
                None => Limits::Auto,
            };
            return Ok(Node::BigOp { ch, limits });
        }
        if let Some(&(_, takes_limits)) = FUNCTIONS.iter().find(|(n, _)| *n == name) {
            let limits = self.limits_modifier().unwrap_or(takes_limits);
            return Ok(Node::Func { name, limits });
        }
        if let Some((ch, class)) = lookup_symbol(&name) {
            return Ok(Node::Atom { ch, class });
        }
        let mut chars = name.chars();
        if let (Some(c), None) = (chars.next(), chars.next())
            && let Some(&(_, ch, class)) = ESCAPED.iter().find(|(e, ..)| *e == c)
        {
            return Ok(Node::Atom { ch, class });
        }
        Err(MathError::UnknownCommand(name))
    }

    /// `\limits` (`Some(true)`) or `\nolimits` (`Some(false)`) right after an operator.
    fn limits_modifier(&mut self) -> Option<bool> {
        let before = self.pos;
        self.skip_ws();
        for (name, v) in [("limits", true), ("nolimits", false)] {
            if self.at_command(name) {
                self.pos += 1 + name.len();
                return Some(v);
            }
        }
        self.pos = before;
        None
    }

    /// A delimiter after `\left`, `\right`: `None` for the null delimiter `.`.
    fn delimiter(&mut self) -> Result<Option<char>, MathError> {
        self.skip_ws();
        let at = self.pos;
        let Some(c) = self.bump() else { return self.err("a delimiter is missing") };
        let d = match c {
            '(' | ')' | '[' | ']' | '|' | '/' => Some(c),
            '.' => None,
            '\\' => {
                let name = self.command_name()?;
                let d = match name.as_str() {
                    "{" | "lbrace" => '{',
                    "}" | "rbrace" => '}',
                    "|" | "Vert" | "lVert" | "rVert" => '\u{2016}',
                    "vert" | "lvert" | "rvert" => '|',
                    "langle" => '\u{27E8}',
                    "rangle" => '\u{27E9}',
                    "lceil" => '\u{2308}',
                    "rceil" => '\u{2309}',
                    "lfloor" => '\u{230A}',
                    "rfloor" => '\u{230B}',
                    "backslash" => '\\',
                    _ => {
                        self.pos = at;
                        return self.err(&format!("\\{name} is not a delimiter"));
                    }
                };
                Some(d)
            }
            _ => {
                self.pos = at;
                return self.err(&format!("'{c}' is not a delimiter"));
            }
        };
        Ok(d)
    }

    /// The braced argument of `\text` as it is written: nested braces kept in balance, a few
    /// escapes (`\{`, `\%`…) resolved.
    fn raw_text(&mut self) -> Result<String, MathError> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return self.err("expected '{'");
        }
        self.pos += 1;
        let mut text = String::new();
        let mut level = 0usize;
        loop {
            let Some(c) = self.bump() else { return self.err("missing '}'") };
            match c {
                '{' => {
                    level += 1;
                    text.push(c);
                }
                '}' if level == 0 => return Ok(text),
                '}' => {
                    level -= 1;
                    text.push(c);
                }
                '\\' => match self.bump() {
                    Some(e @ ('{' | '}' | '%' | '$' | '&' | '#' | '_' | '\\')) => text.push(e),
                    Some(' ') => text.push(' '),
                    Some(e) if e.is_ascii_alphabetic() => {
                        self.pos -= 1;
                        let name = self.command_name()?;
                        return Err(MathError::UnknownCommand(name));
                    }
                    _ => return self.err("unsupported escape in text"),
                },
                '~' => text.push(' '),
                c if c.is_control() => return self.err("a control character in the text"),
                c => text.push(c),
            }
        }
    }
}
