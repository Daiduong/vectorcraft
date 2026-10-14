//! Math drawing style: the line weights, dash patterns, dot size and axis look the `mathdraw.*`
//! commands draw with, saved with the document.
//!
//! The defaults are the presets the math illustration tool these commands follow ships with
//! (its "일격필살" team style): five solid weights, five dashed lines, 14 pt dots with a white
//! border, and axes with Computer Modern style labels.
//!
//! The style is read from files and set over the control channel, so both are untrusted: a value
//! a file holds that is out of range, or of the wrong type, reads as its default
//! ([`MathDrawStyle`]'s `Deserialize`), and [`MathDrawStyle::patch`] refuses one.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A line of the style: one of the five solid weights or the five dashed lines (each 1–5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineRef {
    Solid(u8),
    Dashed(u8),
}

impl LineRef {
    /// `"solid1"` … `"solid5"`, `"dashed1"` … `"dashed5"`.
    pub fn name(self) -> String {
        match self {
            LineRef::Solid(i) => format!("solid{i}"),
            LineRef::Dashed(i) => format!("dashed{i}"),
        }
    }

    /// Reads [`LineRef::name`]'s form (any case).
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        let (kind, n) = if let Some(n) = s.strip_prefix("solid") {
            (LineRef::Solid as fn(u8) -> LineRef, n)
        } else {
            (LineRef::Dashed as fn(u8) -> LineRef, s.strip_prefix("dashed")?)
        };
        let i: u8 = n.parse().ok().filter(|i| (1..=5).contains(i))?;
        Some(kind(i))
    }

    /// The preset's index, 0–4 (an index out of 1–5 is taken as the nearest).
    fn index(i: u8) -> usize {
        usize::from(i.clamp(1, 5) - 1)
    }
}

impl Serialize for LineRef {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for LineRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        LineRef::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("not a line: {s:?}")))
    }
}

/// A dashed line preset: its weight, dash and gap lengths, in points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct DashedLine {
    pub width: f64,
    pub dash: f64,
    pub gap: f64,
}

/// Where an axis label goes and how it is set.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AxisLabel {
    /// Offset of the label from the end of its axis (from the origin for "O"), in points at
    /// [`LABEL_REFERENCE_SIZE`] and y up, as the original tool stores it; it scales with
    /// [`MathDrawStyle::label_size`] ([`MathDrawStyle::label_offset`]).
    #[serde(serialize_with = "pair")]
    pub offset: (f64, f64),
    /// Math italic (as Computer Modern Math Italic sets "x" and "y"), else roman (as "O").
    pub italic: bool,
}

fn pair<S: serde::Serializer>(v: &(f64, f64), s: S) -> Result<S::Ok, S::Error> {
    [v.0, v.1].serialize(s)
}

/// Which axis label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisLabelKind {
    X,
    Y,
    Origin,
}

/// The label size the label offsets are given at, in points.
pub const LABEL_REFERENCE_SIZE: f64 = 73.33;

/// The `mathdraw.*` commands' style.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MathDrawStyle {
    /// Solid line weights 1–5, in points.
    pub solid: [f64; 5],
    /// Dashed lines 1–5.
    pub dashed: [DashedLine; 5],
    /// Diameter of a dot, in points.
    pub dot_size: f64,
    /// The line a dot's white border is drawn with.
    pub dot_border: LineRef,
    /// How near a click must be to pick a path, in points (the math drawing tools).
    pub hit_tolerance: f64,
    pub label_x: AxisLabel,
    pub label_y: AxisLabel,
    pub label_o: AxisLabel,
    /// Font size of the axis labels, in points.
    pub label_size: f64,
    /// Arrowhead of the axes: 1 open V (stroked), 2 slim filled triangle, 3 concave filled
    /// triangle (swallowtail, the textbook look).
    pub arrow_type: u8,
    /// Scale of the arrowheads.
    pub arrow_ratio: f64,
    /// The unit length of a reference segment made automatically, in points.
    pub std_length: f64,
    /// The line the coordinate axes are drawn with.
    pub axis_line: LineRef,
}

impl Default for MathDrawStyle {
    fn default() -> Self {
        let dashed = |width, dash, gap| DashedLine { width, dash, gap };
        Self {
            solid: [6.0, 2.5, 2.0, 1.0, 0.5],
            dashed: [dashed(2.0, 12.0, 6.0), dashed(4.8, 18.0, 9.0), dashed(1.0, 6.0, 3.0), dashed(1.0, 4.0, 2.0), dashed(0.5, 4.0, 2.0)],
            dot_size: 14.0,
            dot_border: LineRef::Solid(3),
            hit_tolerance: 20.0,
            label_x: AxisLabel { offset: (-18.4365234375, -52.3120117188), italic: true },
            label_y: AxisLabel { offset: (-48.0340679688, -7.9995117188), italic: true },
            label_o: AxisLabel { offset: (-57.7846579688, -64.1459960938), italic: false },
            label_size: LABEL_REFERENCE_SIZE,
            arrow_type: 3,
            arrow_ratio: 1.0,
            std_length: 150.0,
            axis_line: LineRef::Solid(2),
        }
    }
}

/// Ranges of the style's numbers (inclusive), checked when the style is set or read.
pub mod limits {
    /// Line weights, in points.
    pub const WIDTH: (f64, f64) = (0.01, 1000.0);
    /// Dash and gap lengths, in points.
    pub const DASH: (f64, f64) = (0.0, 10_000.0);
    /// Dot diameter, in points.
    pub const DOT: (f64, f64) = (0.1, 1000.0);
    /// Hit tolerance, in points.
    pub const HIT: (f64, f64) = (0.0, 1000.0);
    /// Label offsets, in points.
    pub const OFFSET: (f64, f64) = (-100_000.0, 100_000.0);
    /// Label size, in points (as an equation's).
    pub const LABEL_SIZE: (f64, f64) = (0.5, 2000.0);
    /// Arrowhead types.
    pub const ARROW_TYPE: (u8, u8) = (1, 3);
    /// Arrowhead scale.
    pub const ARROW_RATIO: (f64, f64) = (0.01, 100.0);
    /// Reference segment unit, in points.
    pub const STD_LENGTH: (f64, f64) = (0.01, 100_000.0);
}

/// Why a style patch was refused. The messages are in English.
#[derive(Clone, Debug, PartialEq)]
pub enum PatchError {
    /// The patch (or the value of `key`) should be an object.
    NotObject(String),
    /// A key the style doesn't have (with the path to it, `labelX.size`).
    UnknownKey(String),
    /// A value of the wrong type or out of range, with what it should be.
    BadValue { key: String, want: String },
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatchError::NotObject(k) if k.is_empty() => write!(f, "the style must be an object of the keys to set"),
            PatchError::NotObject(k) => write!(f, "`{k}` must be an object"),
            PatchError::UnknownKey(k) => write!(f, "the math drawing style has no `{k}`"),
            PatchError::BadValue { key, want } => write!(f, "`{key}` must be {want}"),
        }
    }
}

fn number_in(v: &Value, key: &str, (lo, hi): (f64, f64)) -> Result<f64, PatchError> {
    v.as_f64()
        .filter(|n| n.is_finite() && *n >= lo && *n <= hi)
        .ok_or_else(|| PatchError::BadValue { key: key.to_string(), want: format!("a number from {lo} to {hi}") })
}

fn object<'a>(v: &'a Value, key: &str) -> Result<&'a Map<String, Value>, PatchError> {
    v.as_object().ok_or_else(|| PatchError::NotObject(key.to_string()))
}

fn line(v: &Value, key: &str) -> Result<LineRef, PatchError> {
    v.as_str()
        .and_then(LineRef::parse)
        .ok_or_else(|| PatchError::BadValue { key: key.to_string(), want: "a line, \"solid1\"…\"solid5\" or \"dashed1\"…\"dashed5\"".into() })
}

/// `v` as an array of exactly 5 entries.
fn five<'a>(v: &'a Value, key: &str, of: &str) -> Result<&'a Vec<Value>, PatchError> {
    v.as_array()
        .filter(|a| a.len() == 5)
        .ok_or_else(|| PatchError::BadValue { key: key.to_string(), want: format!("an array of 5 {of} (null keeps one)") })
}

impl MathDrawStyle {
    /// Whether this is the default style (which documents don't save).
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The weight and dash pattern (dash, gap; empty for a solid line) of `line`, in points.
    pub fn stroke(&self, line: LineRef) -> (f64, Vec<f64>) {
        match line {
            LineRef::Solid(i) => (self.solid.get(LineRef::index(i)).copied().unwrap_or(1.0), vec![]),
            LineRef::Dashed(i) => self.dashed.get(LineRef::index(i)).map_or((1.0, vec![4.0, 2.0]), |d| (d.width, vec![d.dash, d.gap])),
        }
    }

    /// The offset of the `kind` label at [`MathDrawStyle::label_size`] (points, y up).
    pub fn label_offset(&self, kind: AxisLabelKind) -> (f64, f64) {
        let l = self.label(kind);
        let k = self.label_size / LABEL_REFERENCE_SIZE;
        (l.offset.0 * k, l.offset.1 * k)
    }

    pub fn label(&self, kind: AxisLabelKind) -> &AxisLabel {
        match kind {
            AxisLabelKind::X => &self.label_x,
            AxisLabelKind::Y => &self.label_y,
            AxisLabelKind::Origin => &self.label_o,
        }
    }

    /// Set the keys `patch` gives (deeply: `{"labelX": {"italic": false}}` keeps the offset; in
    /// `solid` and `dashed` a null entry keeps that preset). All or nothing: on an error the style
    /// is unchanged.
    pub fn patch(&mut self, patch: &Value) -> Result<(), PatchError> {
        let map = object(patch, "")?;
        let mut next = self.clone();
        for (k, v) in map {
            next.set(k, v)?;
        }
        *self = next;
        Ok(())
    }

    /// Set one top-level key.
    fn set(&mut self, key: &str, v: &Value) -> Result<(), PatchError> {
        match key {
            "solid" => {
                let a = five(v, key, "weights")?;
                for (i, (w, e)) in self.solid.iter_mut().zip(a).enumerate() {
                    if !e.is_null() {
                        *w = number_in(e, &format!("solid[{i}]"), limits::WIDTH)?;
                    }
                }
            }
            "dashed" => {
                let a = five(v, key, "dashed lines")?;
                for (i, (d, e)) in self.dashed.iter_mut().zip(a).enumerate() {
                    if e.is_null() {
                        continue;
                    }
                    let at = format!("dashed[{i}]");
                    for (k, v) in object(e, &at)? {
                        let path = format!("{at}.{k}");
                        match k.as_str() {
                            "width" => d.width = number_in(v, &path, limits::WIDTH)?,
                            "dash" => d.dash = number_in(v, &path, limits::DASH)?,
                            "gap" => d.gap = number_in(v, &path, limits::DASH)?,
                            _ => return Err(PatchError::UnknownKey(path)),
                        }
                    }
                    if d.dash + d.gap <= 0.0 {
                        return Err(PatchError::BadValue { key: at, want: "a dash or a gap longer than 0".into() });
                    }
                }
            }
            "dotSize" => self.dot_size = number_in(v, key, limits::DOT)?,
            "dotBorder" => self.dot_border = line(v, key)?,
            "hitTolerance" => self.hit_tolerance = number_in(v, key, limits::HIT)?,
            "labelX" | "labelY" | "labelO" => {
                let label = match key {
                    "labelX" => &mut self.label_x,
                    "labelY" => &mut self.label_y,
                    _ => &mut self.label_o,
                };
                for (k, v) in object(v, key)? {
                    let path = format!("{key}.{k}");
                    match k.as_str() {
                        "offset" => {
                            let pair = v
                                .as_array()
                                .filter(|a| a.len() == 2)
                                .ok_or_else(|| PatchError::BadValue { key: path.clone(), want: "[x, y] in points".into() })?;
                            let x = pair.first().map_or(Ok(0.0), |x| number_in(x, &path, limits::OFFSET))?;
                            let y = pair.get(1).map_or(Ok(0.0), |y| number_in(y, &path, limits::OFFSET))?;
                            label.offset = (x, y);
                        }
                        "italic" => {
                            label.italic = v.as_bool().ok_or_else(|| PatchError::BadValue { key: path.clone(), want: "true or false".into() })?
                        }
                        _ => return Err(PatchError::UnknownKey(path)),
                    }
                }
            }
            "labelSize" => self.label_size = number_in(v, key, limits::LABEL_SIZE)?,
            "arrowType" => {
                let (lo, hi) = limits::ARROW_TYPE;
                self.arrow_type = v
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .filter(|n| (lo..=hi).contains(n))
                    .ok_or_else(|| PatchError::BadValue { key: key.to_string(), want: format!("{lo}, 2 or {hi}") })?;
            }
            "arrowRatio" => self.arrow_ratio = number_in(v, key, limits::ARROW_RATIO)?,
            "stdLength" => self.std_length = number_in(v, key, limits::STD_LENGTH)?,
            "axisLine" => self.axis_line = line(v, key)?,
            _ => return Err(PatchError::UnknownKey(key.to_string())),
        }
        Ok(())
    }
}

/// Read leniently: each key the style knows with a valid value is taken, anything else (a key from
/// a newer version, a value out of range) is left at its default, so a file always opens.
impl<'de> Deserialize<'de> for MathDrawStyle {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        let mut style = MathDrawStyle::default();
        if let Some(map) = v.as_object() {
            for (k, v) in map {
                // Key by key, so one bad value only loses itself (a partial patch of a nested
                // value is all or nothing).
                let mut one = style.clone();
                if one.set(k, v).is_ok() {
                    style = one;
                }
            }
        }
        Ok(style)
    }
}
