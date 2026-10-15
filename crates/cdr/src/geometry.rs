//! Translation of CorelDRAW geometry and style records into VectorCraft's model: Bézier path
//! records become [`PathData`], `trfd` matrices become [`Affine`]s, `outl` outlines become
//! [`StrokeLayer`]s and `fild` solid and fountain fills become [`FillLayer`]s.
//!
//! Record layouts follow libcdr's `CDRParser.cpp` (`readLoda`, `readLineAndCurve`, `readPath`,
//! `readTrfd`, `readOutl`, `readFild`, `readColor`), version by version. How they become drawing
//! follows `CDRContentCollector.cpp` and `CommonParser::processPath`.
//!
//! # Coordinate spaces
//!
//! CorelDRAW stores geometry in inches with y pointing up and the origin at the page centre:
//! coordinates are 16-bit thousandths of an inch before version 6 and 32-bit 1/254000 inch (0.1 µm)
//! from version 6 on. VectorCraft documents are points (1/72 in) with y pointing down and the
//! origin at the page's top-left corner. A path is mapped by its object's `trfd` matrices first,
//! still in CorelDRAW space, then by [`PageSpace::to_document`]:
//! `x_pt = 72 (x + w/2)`, `y_pt = 72 (h/2 − y)` for a `w × h` inch page, as libcdr does with
//! `-offsetX` and `height - y`.
//!
//! # What is approximate
//!
//! Anything VectorCraft cannot show exactly is translated to the nearest equivalent and noted in
//! [`Shape::notes`]: conical fountains become linear and square ones radial (as libcdr does),
//! palette and spot colours (models `0x19` and `0x0e`) need CorelDRAW's palette tables and fall
//! back to black, and a stretched (calligraphic) nib scales the width as libcdr does. Pattern,
//! bitmap, texture and PostScript fills are not translated. Rectangles, ellipses, polygons, text and
//! bitmaps are other record kinds and are counted in [`Translation::skipped`], not translated.

use std::collections::BTreeMap;

use vectorcraft_color::{Color, Gradient, GradientGeom, GradientKind, GradientPaint, GradientStop, Paint};
use vectorcraft_doc::appearance::{Appearance, AppearanceItem, Dash, FillLayer, LineCap, LineJoin, StrokeLayer};
use vectorcraft_geom::{Affine, BezPath, FillRule, PathData, Point, Rect};

use crate::Error;
use crate::riff::{CdrVersion, Chunk, FourCc, List, Riff};

/// Points per inch.
const PT_PER_IN: f64 = 72.0;
/// Most points read from one path record (each is at least 5 bytes, so data caps it too).
const MAX_POINTS: usize = 1 << 20;
/// Most gradient stops, dash entries or `loda` / `trfd` arguments read from one record.
const MAX_LIST: usize = 4096;

// ---------------------------------------------------------------------------------------------
// Bounded little-endian reader

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    what: &'static str,
    version: CdrVersion,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8], what: &'static str, version: CdrVersion) -> Self {
        Self { data, pos: 0, what, version }
    }

    fn short(&self) -> Error {
        Error::Record { record: self.what, message: format!("ends at byte {} of {}", self.pos, self.data.len()) }
    }

    fn bytes<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let end = self.pos.checked_add(N).ok_or_else(|| self.short())?;
        let b = self.data.get(self.pos..end).ok_or_else(|| self.short())?;
        self.pos = end;
        let mut out = [0; N];
        out.copy_from_slice(b);
        Ok(out)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.bytes::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_le_bytes(self.bytes()?))
    }
    fn i16(&mut self) -> Result<i16, Error> {
        Ok(i16::from_le_bytes(self.bytes()?))
    }
    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.bytes()?))
    }
    fn i32(&mut self) -> Result<i32, Error> {
        Ok(i32::from_le_bytes(self.bytes()?))
    }
    fn f64(&mut self) -> Result<f64, Error> {
        let v = f64::from_le_bytes(self.bytes()?);
        if v.is_finite() { Ok(v) } else { Err(Error::Record { record: self.what, message: format!("non-finite number at byte {}", self.pos - 8) }) }
    }

    fn skip(&mut self, n: usize) -> Result<(), Error> {
        self.seek(self.pos.saturating_add(n))
    }

    fn seek(&mut self, pos: usize) -> Result<(), Error> {
        if pos > self.data.len() {
            return Err(self.short());
        }
        self.pos = pos;
        Ok(())
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn wide(&self) -> bool {
        self.version >= CdrVersion(600)
    }

    /// libcdr's `readUnsigned`: 16 bits before version 6, 32 after.
    fn unsigned(&mut self) -> Result<u32, Error> {
        if self.wide() { self.u32() } else { self.u16().map(u32::from) }
    }

    /// libcdr's `readInteger`.
    fn integer(&mut self) -> Result<i32, Error> {
        if self.wide() { self.i32() } else { self.i16().map(i32::from) }
    }

    /// A coordinate in inches.
    fn coord(&mut self) -> Result<f64, Error> {
        if self.wide() { Ok(f64::from(self.i32()?) / 254_000.0) } else { Ok(f64::from(self.i16()?) / 1000.0) }
    }

    /// An angle in radians: tenths of a degree before version 6, millionths after.
    fn angle(&mut self) -> Result<f64, Error> {
        if self.wide() { Ok(f64::from(self.i32()?).to_radians() / 1_000_000.0) } else { Ok(f64::from(self.i16()?).to_radians() / 10.0) }
    }

    /// A 16.16 fixed-point number (version 4 transforms), as libcdr decodes it.
    fn fixed(&mut self) -> Result<f64, Error> {
        let v = self.u32()?;
        let int = f64::from((v >> 16) as u16 as i16);
        Ok(int + f64::from(v & 0xffff) / f64::from(0xffff_u32))
    }
}

/// X6+ records may be 16-byte stubs pointing into `content/data/*`, which this reader does not
/// follow (libcdr's `_redirectX6Chunk`).
fn redirected(data: &[u8], version: CdrVersion) -> bool {
    version >= CdrVersion(1600) && data.len() == 0x10
}

// ---------------------------------------------------------------------------------------------
// Colour

/// A colour as CorelDRAW stores it: a model, a palette (for palette colours) and four packed bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CorelColor {
    pub model: u16,
    pub palette: u16,
    pub value: u32,
}

impl CorelColor {
    fn read(r: &mut Reader<'_>) -> Result<Self, Error> {
        let v = r.version;
        if v >= CdrVersion(500) {
            let mut model = r.u16()?;
            if model == 0x01 && v >= CdrVersion(1300) {
                model = 0x19;
            }
            // CorelDRAW X7 writes model 0x1e for palette 0x1e without the palette field.
            if model == 0x1e {
                return Ok(Self { model: 0x19, palette: 0x1e, value: r.u32()? });
            }
            let palette = r.u16()?;
            r.skip(4)?;
            Ok(Self { model, palette, value: r.u32()? })
        } else if v >= CdrVersion(400) {
            let model = r.u16()?;
            let [c, m, y, k] = [r.u16()?, r.u16()?, r.u16()?, r.u16()?].map(|x| u32::from(x & 0xff));
            r.skip(2)?;
            Ok(Self { model, palette: 0, value: c | m << 8 | y << 16 | k << 24 })
        } else {
            let model = u16::from(r.u8()?);
            Ok(Self { model, palette: 0, value: r.u32()? })
        }
    }

    /// The colour in its own model where VectorCraft has one (CMYK, RGB, grey, Lab), else as RGB.
    /// Palette and spot colours need CorelDRAW's palette tables and are `Err` with the reason.
    pub fn to_color(&self) -> Result<Color, String> {
        let [c0, c1, c2, c3] = self.value.to_le_bytes();
        let f = |b: u8, max: f32| f32::from(b) / max;
        Ok(match self.model {
            // CMYK in percent.
            0x01 | 0x02 | 0x15 => Color::cmyk(f(c0, 100.0).min(1.0), f(c1, 100.0).min(1.0), f(c2, 100.0).min(1.0), f(c3, 100.0).min(1.0)),
            // CMYK in 0..255.
            0x03 | 0x11 => Color::cmyk(f(c0, 255.0), f(c1, 255.0), f(c2, 255.0), f(c3, 255.0)),
            0x04 => Color::cmyk(f(c0, 255.0), f(c1, 255.0), f(c2, 255.0), 0.0),
            // RGB, stored blue first.
            0x05 => Color::rgb8(c2, c1, c0),
            0x06 => hsb(u16::from_le_bytes([c0, c1]), f(c2, 255.0), f(c3, 255.0)),
            0x07 => hls(u16::from_le_bytes([c0, c1]), f(c2, 255.0), f(c3, 255.0)),
            0x08 => Color::gray(if c0 == 0 { 0.0 } else { 1.0 }),
            // Grey level: 255 is white; VectorCraft's grey is ink.
            0x09 => Color::gray(1.0 - f(c0, 255.0)),
            0x0b => yiq(c0, c1, c2),
            0x0c => Color::lab(f(c0, 255.0) * 100.0, f32::from(c1 as i8), f32::from(c2 as i8)),
            0x12 => Color::lab(f(c0, 255.0) * 100.0, f32::from(c1) - 128.0, f32::from(c2) - 128.0),
            // Registration: every plate at this tint.
            0x14 => {
                let t = f(c0, 100.0).min(1.0);
                Color::cmyk(t, t, t, t)
            }
            0x19 | 0x0e => {
                return Err(format!("palette colour (model {:#x}, palette {:#x}) needs CorelDRAW's palette tables", self.model, self.palette));
            }
            m => return Err(format!("unknown colour model {m:#x}")),
        })
    }
}

fn hue_rgb(hue: u16) -> (f32, f32, f32) {
    let h = f32::from(hue % 360) / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    match h as u8 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    }
}

fn hsb(hue: u16, s: f32, v: f32) -> Color {
    let (r, g, b) = hue_rgb(hue);
    let mix = |c: f32| v * (1.0 - s + s * c);
    Color::rgb(mix(r), mix(g), mix(b))
}

fn hls(hue: u16, l: f32, s: f32) -> Color {
    let (r, g, b) = hue_rgb(hue);
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let m = l - chroma / 2.0;
    Color::rgb(m + chroma * r, m + chroma * g, m + chroma * b)
}

/// YIQ in CorelDRAW's 0..255 encoding (100 is zero), as libcdr decodes it.
fn yiq(y: u8, i: u8, q: u8) -> Color {
    let centred = |v: u8| {
        let v = f32::from(v) - 100.0;
        if v <= 0.0 { v / 100.0 } else { v / 155.0 }
    };
    let (y, i, q) = (centred(y) * 0.5 + 0.5, centred(i) * 0.5957, centred(q) * 0.5226);
    let c = |v: f32| v.clamp(0.0, 1.0);
    Color::rgb(c(y + 0.9563 * i + 0.6210 * q), c(y - 0.2127 * i - 0.6474 * q), c(y - 1.1070 * i + 1.7046 * q))
}

fn color_or_black(c: &CorelColor, notes: &mut Vec<String>) -> Color {
    c.to_color().unwrap_or_else(|why| {
        notes.push(format!("{why}; drawn black"));
        Color::BLACK
    })
}

// ---------------------------------------------------------------------------------------------
// Transforms and page space

/// The `trfd` matrices of an object, in CorelDRAW space (inches, y up), in the order the file
/// gives them. libcdr applies them first to last.
pub fn decode_trfd(data: &[u8], version: CdrVersion) -> Result<Vec<Affine>, Error> {
    if redirected(data, version) {
        return Err(Error::Record { record: "trfd", message: "X6 record stored in content/data is not followed".into() });
    }
    let mut r = Reader::new(data, "trfd", version);
    let _chunk_length = r.unsigned()?;
    let count = r.unsigned()? as usize;
    let start = r.unsigned()? as usize;
    r.seek(start)?;
    let mut offsets = Vec::new();
    for _ in 0..count.min(MAX_LIST) {
        offsets.push(r.unsigned()? as usize);
    }
    let div = if version < CdrVersion(600) { 1000.0 } else { 254_000.0 };
    let mut out = Vec::new();
    for off in offsets {
        r.seek(off)?;
        if version >= CdrVersion(1300) {
            r.skip(8)?;
        }
        // Type 8 is a matrix; others (0x10, envelopes and the like) are not transforms.
        if r.u16()? != 0x08 {
            continue;
        }
        let m = if version >= CdrVersion(500) {
            if version >= CdrVersion(600) {
                r.skip(6)?;
            }
            [r.f64()?, r.f64()?, r.f64()? / div, r.f64()?, r.f64()?, r.f64()? / div]
        } else {
            [r.fixed()?, r.fixed()?, f64::from(r.i32()?) / 1000.0, r.fixed()?, r.fixed()?, f64::from(r.i32()?) / 1000.0]
        };
        out.push(corel_matrix(m));
    }
    Ok(out)
}

/// libcdr's `CDRTransform(v0, v1, x0, v3, v4, y0)`: `x' = v0·x + v1·y + x0`, `y' = v3·x + v4·y + y0`.
pub fn corel_matrix([v0, v1, x0, v3, v4, y0]: [f64; 6]) -> Affine {
    Affine::new([v0, v3, v1, v4, x0, y0])
}

/// The matrices of a `trfd` record applied in order, as one transform.
pub fn compose(matrices: &[Affine]) -> Affine {
    matrices.iter().fold(Affine::IDENTITY, |acc, m| *m * acc)
}

/// A page's size in inches. CorelDRAW centres its coordinate origin on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSpace {
    pub width_in: f64,
    pub height_in: f64,
}

impl PageSpace {
    /// From the document units `metadata.xml` uses (0.1 µm, 1/254000 in).
    pub fn from_metadata(page: &crate::PageInfo) -> Self {
        Self { width_in: page.width as f64 / 254_000.0, height_in: page.height as f64 / 254_000.0 }
    }

    /// CorelDRAW page space (inches, y up, origin at the centre) to VectorCraft document space
    /// (points, y down, origin at the top-left corner).
    pub fn to_document(&self) -> Affine {
        Affine::new([PT_PER_IN, 0.0, 0.0, -PT_PER_IN, PT_PER_IN * self.width_in / 2.0, PT_PER_IN * self.height_in / 2.0])
    }

    /// Page size in points.
    pub fn size_pt(&self) -> (f64, f64) {
        (self.width_in * PT_PER_IN, self.height_in * PT_PER_IN)
    }
}

/// Document page size from an `mcfg` record.
pub fn decode_mcfg(data: &[u8], version: CdrVersion) -> Result<PageSpace, Error> {
    let mut r = Reader::new(data, "mcfg", version);
    if version >= CdrVersion(1300) {
        r.skip(12)?;
    } else if version >= CdrVersion(900) {
        r.skip(4)?;
    } else if version >= CdrVersion(600) && version < CdrVersion(700) {
        r.skip(0x1c)?;
    }
    let (w, h) = if version < CdrVersion(400) {
        r.skip(2)?;
        let (x0, y0, x1, y1) = (r.coord()?, r.coord()?, r.coord()?, r.coord()?);
        ((x1 - x0).abs(), (y1 - y0).abs())
    } else {
        (r.coord()?, r.coord()?)
    };
    Ok(PageSpace { width_in: w, height_in: h })
}

// ---------------------------------------------------------------------------------------------
// Objects (`loda`) and Bézier paths

/// What a `loda` record draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    /// Lines and curves: a Bézier path record.
    Curve,
    /// The version 6+ path record (chunk type 0x25).
    Path,
    ArtisticText,
    Bitmap,
    ParagraphText,
    Polygon,
    Other(u32),
}

impl ShapeKind {
    fn from_chunk_type(t: u32, version: CdrVersion) -> Self {
        let modern = version >= CdrVersion(400);
        match (t, modern) {
            (0x01, true) | (0x00, false) => Self::Rectangle,
            (0x02, true) | (0x01, false) => Self::Ellipse,
            (0x03, true) | (0x02, false) => Self::Curve,
            (0x04, true) | (0x03, false) => Self::ArtisticText,
            (0x05, true) | (0x04, false) => Self::Bitmap,
            (0x06, true) | (0x05, false) => Self::ParagraphText,
            (0x25, _) => Self::Path,
            (0x14, _) => Self::Polygon,
            (t, _) => Self::Other(t),
        }
    }

    pub fn is_bezier(self) -> bool {
        matches!(self, Self::Curve | Self::Path)
    }
}

/// The parts of a `loda` record this module uses.
#[derive(Debug, Clone, PartialEq)]
pub struct Loda {
    pub kind: ShapeKind,
    /// Offset of the coordinates argument in the record.
    pub coords: Option<usize>,
    pub fill_id: Option<u32>,
    pub outline_id: Option<u32>,
    pub style_id: Option<u32>,
    /// A page size argument (in the page's own `loda`).
    pub page: Option<PageSpace>,
}

/// Read a `loda` (or `lobj`) record's argument table.
pub fn decode_loda(data: &[u8], version: CdrVersion) -> Result<Loda, Error> {
    if redirected(data, version) {
        return Err(Error::Record { record: "loda", message: "X6 record stored in content/data is not followed".into() });
    }
    let mut r = Reader::new(data, "loda", version);
    let _chunk_length = r.unsigned()?;
    let count = (r.unsigned()? as usize).min(MAX_LIST);
    let start_args = r.unsigned()? as usize;
    let start_types = r.unsigned()? as usize;
    let kind = ShapeKind::from_chunk_type(r.unsigned()?, version);
    r.seek(start_args)?;
    let mut offsets = Vec::with_capacity(count);
    for _ in 0..count {
        offsets.push(r.unsigned()? as usize);
    }
    // The type table is stored last argument first.
    r.seek(start_types)?;
    let mut types = Vec::with_capacity(count);
    for _ in 0..count {
        types.push(r.unsigned()?);
    }
    types.reverse();
    let mut loda = Loda { kind, coords: None, fill_id: None, outline_id: None, style_id: None, page: None };
    let modern = version >= CdrVersion(400);
    for (&off, &ty) in offsets.iter().zip(&types) {
        r.seek(off)?;
        match ty {
            0x1e => loda.coords = Some(off),
            // Before version 4 these hold the fill and outline inline (not translated here).
            0x14 if modern => loda.fill_id = Some(r.u32()?).filter(|&id| id != 0),
            0x0a if modern => loda.outline_id = Some(r.u32()?).filter(|&id| id != 0),
            0xc8 => loda.style_id = Some(r.unsigned()?).filter(|&id| id != 0),
            0x4aba => loda.page = Some(PageSpace { width_in: r.coord()?, height_in: r.coord()? }),
            _ => {}
        }
    }
    Ok(loda)
}

/// A Bézier path record: points (inches, CorelDRAW space) and one node-type byte each.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CorelPath {
    pub points: Vec<Point>,
    pub types: Vec<u8>,
}

/// Node-type bits (libcdr's `processPath`).
pub mod node {
    /// The node closes its subpath.
    pub const CLOSED: u8 = 0x08;
    /// Smooth node.
    pub const SMOOTH: u8 = 0x10;
    /// Symmetrical node.
    pub const SYMMETRIC: u8 = 0x20;
    /// The two high bits: 00 moves, 01 draws a line, 10 ends a curve, 11 is a control point.
    pub const SEGMENT: u8 = 0xc0;
    pub const MOVE: u8 = 0x00;
    pub const LINE: u8 = 0x40;
    pub const CURVE: u8 = 0x80;
    pub const CONTROL: u8 = 0xc0;
}

impl CorelPath {
    /// Read the coordinates argument of a curve (`kind` [`ShapeKind::Curve`]) or path
    /// ([`ShapeKind::Path`]) record, starting at `offset` in the `loda` data.
    pub fn decode(data: &[u8], offset: usize, kind: ShapeKind, version: CdrVersion) -> Result<Self, Error> {
        let mut r = Reader::new(data, "path coordinates", version);
        r.seek(offset)?;
        let mut count = match kind {
            ShapeKind::Curve => {
                let n = usize::from(r.u16()?);
                r.skip(2)?;
                n
            }
            ShapeKind::Path => {
                r.skip(4)?;
                let n = usize::from(r.u16()?) + usize::from(r.u16()?);
                r.skip(16)?;
                n
            }
            other => return Err(Error::Record { record: "path coordinates", message: format!("{other:?} is not a Bézier path") }),
        };
        let point_size = if r.wide() { 9 } else { 5 };
        // As libcdr does, a count larger than the data holds is cut to what is there.
        count = count.min(r.remaining() / point_size).min(MAX_POINTS);
        let mut points = Vec::with_capacity(count);
        for _ in 0..count {
            points.push(Point::new(r.coord()?, r.coord()?));
        }
        let mut types = Vec::with_capacity(count);
        for _ in 0..count {
            types.push(r.u8()?);
        }
        Ok(Self { points, types })
    }

    /// The path in CorelDRAW space. Follows `processPath`: control points (type `11`) wait for
    /// the next curve end, which uses the first two; a curve end with fewer becomes a line. A
    /// subpath closes when its last drawn node carries [`node::CLOSED`].
    pub fn to_bezpath(&self) -> BezPath {
        let mut bp = BezPath::new();
        let mut controls: Vec<Point> = Vec::new();
        let mut open = false;
        for (&p, &t) in self.points.iter().zip(&self.types) {
            let closes = t & node::CLOSED != 0;
            match t & node::SEGMENT {
                node::MOVE => {
                    controls.clear();
                    bp.move_to(p);
                    open = true;
                    continue;
                }
                node::CONTROL => {
                    controls.push(p);
                    continue;
                }
                _ if !open => {
                    // A segment without a start point starts a subpath there instead.
                    bp.move_to(p);
                    open = true;
                }
                node::LINE => bp.line_to(p),
                _ => match controls.as_slice() {
                    [c1, c2, ..] => bp.curve_to(*c1, *c2, p),
                    _ => bp.line_to(p),
                },
            }
            controls.clear();
            if closes {
                bp.close_path();
                open = false;
            }
        }
        bp
    }

    /// The path in document space: through `object` (the composed `trfd` matrices) then `page`.
    pub fn to_path_data(&self, object: Affine, page: &PageSpace) -> PathData {
        let mut bp = self.to_bezpath();
        bp.apply_affine(page.to_document() * object);
        PathData::from_bezpath(&bp)
    }
}

// ---------------------------------------------------------------------------------------------
// Outlines

/// An `outl` record.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    pub id: u32,
    /// Bit 0: no outline; bits 1–2: drawn (bit 2: dashed); bit 5: scales with the object.
    pub line_type: u16,
    /// 0 butt, 1 round, 2 square.
    pub cap: u16,
    /// 0 miter, 1 round, 2 bevel.
    pub join: u16,
    /// Width in inches.
    pub width_in: f64,
    /// Nib stretch, 1.0 for a round nib.
    pub stretch: f64,
    /// Nib angle in radians.
    pub angle: f64,
    pub color: CorelColor,
    /// Dash, gap, dash, gap… in multiples of the width.
    pub dashes: Vec<u16>,
    pub start_arrow: u32,
    pub end_arrow: u32,
}

impl Outline {
    pub const NONE: u16 = 0x01;
    pub const DASHED: u16 = 0x04;
    pub const SCALE_WITH_OBJECT: u16 = 0x20;

    pub fn decode(data: &[u8], version: CdrVersion) -> Result<Self, Error> {
        if redirected(data, version) {
            return Err(Error::Record { record: "outl", message: "X6 record stored in content/data is not followed".into() });
        }
        let mut r = Reader::new(data, "outl", version);
        let id = r.u32()?;
        if version >= CdrVersion(1300) {
            // A list of (id, length) blocks; the outline is the block with id 1.
            let mut skip = 0usize;
            loop {
                r.skip(skip)?;
                let block = r.u32()?;
                skip = r.u32()? as usize;
                if block == 1 {
                    break;
                }
            }
        }
        let line_type = r.u16()?;
        let cap = r.u16()?;
        let join = r.u16()?;
        if version >= CdrVersion(600) && version < CdrVersion(1300) {
            r.skip(2)?;
        }
        let width_in = r.coord()?;
        let stretch = f64::from(r.u16()?) / 100.0;
        if version >= CdrVersion(600) {
            r.skip(2)?;
        }
        let angle = r.angle()?;
        if version >= CdrVersion(1300) {
            r.skip(46)?;
        } else if version >= CdrVersion(600) {
            r.skip(52)?;
        }
        let color = CorelColor::read(&mut r)?;
        r.skip(if version < CdrVersion(600) { 10 } else { 16 })?;
        let n = usize::from(r.u16()?).min(r.remaining() / 2).min(MAX_LIST);
        let dash_at = r.pos;
        let mut dashes = Vec::with_capacity(n);
        for _ in 0..n {
            dashes.push(r.u16()?);
        }
        r.seek(dash_at + if version < CdrVersion(600) { 20 } else { 22 })?;
        let (start_arrow, end_arrow) = (r.u32()?, r.u32()?);
        Ok(Self { id, line_type, cap, join, width_in, stretch, angle, color, dashes, start_arrow, end_arrow })
    }

    /// The stroke this outline draws, or `None` for no outline. `object` is the object's
    /// transform, which scales the width when the outline scales with the object.
    pub fn to_stroke(&self, object: Affine, notes: &mut Vec<String>) -> Option<StrokeLayer> {
        if self.line_type & Self::NONE != 0 {
            return None;
        }
        let paint = Paint::solid(color_or_black(&self.color, notes));
        if self.line_type & 0x06 == 0 {
            // libcdr draws other line types as a black hairline.
            return Some(StrokeLayer::new(Paint::solid(Color::BLACK), 0.0));
        }
        let mut scale = 1.0;
        if self.line_type & Self::SCALE_WITH_OBJECT != 0 {
            let [a, b, c, d, _, _] = object.as_coeffs();
            scale = a.hypot(b).max(c.hypot(d));
        }
        if (self.stretch - 1.0).abs() > 1e-6 {
            notes.push(format!("calligraphic nib (stretch {:.0}%) drawn as a uniform width", self.stretch * 100.0));
            scale *= self.stretch;
        }
        let width = self.width_in * scale * PT_PER_IN;
        let mut stroke = StrokeLayer::new(paint, if width.is_finite() { width.max(0.0) } else { 0.0 });
        stroke.cap = match self.cap {
            1 => LineCap::Round,
            2 => LineCap::Square,
            _ => LineCap::Butt,
        };
        stroke.join = match self.join {
            1 => LineJoin::Round,
            2 => LineJoin::Bevel,
            _ => LineJoin::Miter,
        };
        if self.line_type & Self::DASHED != 0 && !self.dashes.is_empty() {
            let unit = if stroke.width > 0.0 { stroke.width } else { 1.0 };
            let pattern: Vec<f64> = self.dashes.iter().map(|&d| f64::from(d) * unit).collect();
            if pattern.iter().any(|&d| d > 0.0) {
                stroke.dash = Some(Dash { pattern, offset: 0.0, align_corners: false });
            }
        }
        if self.start_arrow != 0 || self.end_arrow != 0 {
            notes.push("arrowheads not translated".into());
        }
        Some(stroke)
    }
}

// ---------------------------------------------------------------------------------------------
// Fills

/// A fountain (gradient) fill.
#[derive(Debug, Clone, PartialEq)]
pub struct Fountain {
    /// 1 linear, 2 radial, 3 conical, 4 square.
    pub kind: u8,
    /// Edge pad in percent.
    pub edge_pad: i32,
    /// Angle in radians, counter-clockwise.
    pub angle: f64,
    /// Centre offset in percent of the half-size (radial, conical, square), y up.
    pub center: (i32, i32),
    pub mode: u8,
    /// Midpoint of a two-colour fountain, 0..1.
    pub midpoint: f64,
    /// (offset 0..1, colour).
    pub stops: Vec<(f64, CorelColor)>,
}

/// A `fild` record.
#[derive(Debug, Clone, PartialEq)]
pub enum CorelFill {
    None,
    Solid(CorelColor),
    Fountain(Fountain),
    /// Pattern (7), bitmap (9), full-colour (10), texture (11), PostScript and others.
    Unsupported(u16),
}

impl CorelFill {
    /// Decode a `fild` record: its id and fill.
    pub fn decode(data: &[u8], version: CdrVersion) -> Result<(u32, Self), Error> {
        if redirected(data, version) {
            return Err(Error::Record { record: "fild", message: "X6 record stored in content/data is not followed".into() });
        }
        let x3 = version >= CdrVersion(1300);
        let mut r = Reader::new(data, "fild", version);
        let id = r.u32()?;
        if x3 {
            r.skip(8)?;
        }
        let fill = match r.u16()? {
            0 => Self::None,
            1 => {
                r.skip(if x3 { 13 } else { 2 })?;
                Self::Solid(CorelColor::read(&mut r)?)
            }
            2 => Self::Fountain(Self::fountain(&mut r)?),
            t => Self::Unsupported(t),
        };
        Ok((id, fill))
    }

    fn fountain(r: &mut Reader<'_>) -> Result<Fountain, Error> {
        let v = r.version;
        let x3 = v >= CdrVersion(1300);
        r.skip(if x3 { 8 } else { 2 })?;
        let kind = r.u8()?;
        let edge_pad = if x3 {
            r.skip(17)?;
            i32::from(r.i16()?)
        } else if v >= CdrVersion(600) {
            r.skip(19)?;
            r.i32()?
        } else {
            r.skip(11)?;
            i32::from(r.i16()?)
        };
        let angle = r.angle()?;
        let center = (r.integer()?, r.integer()?);
        if v >= CdrVersion(600) {
            r.skip(2)?;
        }
        let mode = (r.unsigned()? & 0xff) as u8;
        let midpoint = f64::from(r.u8()?) / 100.0;
        r.skip(1)?;
        let n = usize::from((r.unsigned()? & 0xffff) as u16).min(MAX_LIST);
        if x3 {
            r.skip(3)?;
        }
        let mut stops = Vec::with_capacity(n);
        for _ in 0..n {
            let color = CorelColor::read(r)?;
            if v >= CdrVersion(1500) {
                r.skip(26)?;
            } else if x3 {
                r.skip(5)?;
            }
            let offset = f64::from(r.unsigned()? & 0xffff) / 100.0;
            if x3 {
                r.skip(3)?;
            }
            stops.push((offset, color));
        }
        Ok(Fountain { kind, edge_pad, angle, center, mode, midpoint, stops })
    }

    /// The fill layer, or `None` for no fill. `bounds` is the object's box in document space,
    /// which a fountain is laid out on.
    pub fn to_fill(&self, bounds: Option<Rect>, notes: &mut Vec<String>) -> Option<FillLayer> {
        match self {
            Self::None => None,
            Self::Solid(c) => Some(FillLayer::new(Paint::solid(color_or_black(c, notes)))),
            Self::Fountain(f) => f.to_paint(bounds, notes).map(FillLayer::new),
            Self::Unsupported(t) => {
                notes.push(format!("fill type {t} (pattern, bitmap, texture or PostScript) not translated"));
                None
            }
        }
    }
}

impl Fountain {
    fn to_paint(&self, bounds: Option<Rect>, notes: &mut Vec<String>) -> Option<Paint> {
        let (first, rest) = self.stops.split_first()?;
        if rest.is_empty() {
            return Some(Paint::solid(color_or_black(&first.1, notes)));
        }
        let kind = match self.kind {
            1 => GradientKind::Linear,
            2 => GradientKind::Radial,
            3 => {
                notes.push("conical fountain drawn as linear".into());
                GradientKind::Linear
            }
            4 => {
                notes.push("square fountain drawn as radial".into());
                GradientKind::Radial
            }
            k => {
                notes.push(format!("fountain type {k} drawn as linear"));
                GradientKind::Linear
            }
        };
        // CorelDRAW's first stop is at a radial fountain's rim (libcdr hands it to ODF as the
        // border colour); VectorCraft's radial ramp starts at the centre, so radial stops mirror.
        let radial = kind == GradientKind::Radial;
        let mut stops: Vec<GradientStop> = self
            .stops
            .iter()
            .map(|(t, c)| {
                let t = t.clamp(0.0, 1.0) as f32;
                GradientStop::new(if radial { 1.0 - t } else { t }, color_or_black(c, notes))
            })
            .collect();
        stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        if let [a, _] = stops.as_mut_slice()
            && self.midpoint > 0.0
            && self.midpoint < 1.0
        {
            let m = if radial { 1.0 - self.midpoint } else { self.midpoint } as f32;
            a.midpoint = m.clamp(0.13, 0.87);
        }
        let angle = self.angle.to_degrees();
        let mut paint = GradientPaint::new(Gradient { kind, stops });
        paint.angle = if angle.is_finite() { angle } else { 0.0 };
        if let Some(b) = bounds {
            paint.geom = Some(self.geom(kind, b, paint.angle));
        }
        Some(Paint::Gradient(Box::new(paint)))
    }

    /// Placement on the object's box: the edge pad shortens the ramp at both ends, and a radial
    /// fountain's centre moves by its offset (percent of the half-size, y up).
    fn geom(&self, kind: GradientKind, b: Rect, angle: f64) -> GradientGeom {
        let pad = (f64::from(self.edge_pad) / 100.0).clamp(0.0, 0.49);
        let mut g = GradientGeom::fit(kind, b, angle);
        if kind == GradientKind::Radial {
            let shift =
                vectorcraft_geom::Vec2::new(f64::from(self.center.0) / 100.0 * b.width() / 2.0, -f64::from(self.center.1) / 100.0 * b.height() / 2.0);
            g.start += shift;
            g.end = g.start + (g.end - (g.start - shift)) * (1.0 - pad);
        } else {
            let span = g.end - g.start;
            g.start += span * pad;
            g.end -= span * pad;
        }
        g
    }
}

// ---------------------------------------------------------------------------------------------
// Whole documents

/// One translated object.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// The path in document space (points, y down).
    pub path: PathData,
    /// CorelDRAW fills paths even-odd (libcdr writes `svg:fill-rule="evenodd"`).
    pub rule: FillRule,
    /// Fill below stroke, as CorelDRAW draws them.
    pub appearance: Appearance,
    /// What was approximated or left out.
    pub notes: Vec<String>,
}

/// The translated objects of one page, bottom first.
#[derive(Debug, Clone, PartialEq)]
pub struct PageShapes {
    /// Index of the page list in the file (CorelDRAW writes a master page before the pages).
    pub index: usize,
    /// The master page, which libcdr does not output: from version 4, the page list's first
    /// `flgs` record has a bit of `0x00ff0000` set (`CDRContentCollector::collectFlags`).
    pub master: bool,
    pub page: PageSpace,
    pub shapes: Vec<Shape>,
}

/// Every page's Bézier objects.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Translation {
    pub pages: Vec<PageShapes>,
    /// Objects of other kinds, by kind, that were not translated.
    pub skipped: BTreeMap<String, usize>,
    /// Records that could not be decoded (kind and reason).
    pub errors: Vec<String>,
}

/// Translate every Bézier path object of a RIFF tree. `default_page` is used until the file gives
/// a page size (an `mcfg` record or a page's own size argument).
pub fn translate(riff: &Riff<'_>, default_page: Option<PageSpace>) -> Translation {
    let version = riff.version;
    let mut t = Translation::default();
    let mut fills = BTreeMap::new();
    let mut outlines = BTreeMap::new();
    let mut page = default_page;
    // Styles come before the objects that use them; collect them first, as libcdr's styles pass.
    for (_, c) in riff.walk() {
        let Chunk::Record(rec) = c else { continue };
        let Some(data) = riff.data(rec) else { continue };
        match &rec.kind.0 {
            b"fild" | b"fill" => match CorelFill::decode(data, version) {
                Ok((id, f)) => {
                    fills.insert(id, f);
                }
                Err(e) => t.errors.push(e.to_string()),
            },
            b"outl" => match Outline::decode(data, version) {
                Ok(o) => {
                    outlines.insert(o.id, o);
                }
                Err(e) => t.errors.push(e.to_string()),
            },
            b"mcfg" if page.is_none() => page = decode_mcfg(data, version).ok(),
            _ => {}
        }
    }
    let ctx = Ctx { riff, version, fills: &fills, outlines: &outlines };
    let mut index = 0;
    for chunk in &riff.chunks {
        ctx.pages(chunk, &mut page, &mut index, &mut t);
    }
    t
}

struct Ctx<'a, 'r> {
    riff: &'a Riff<'r>,
    version: CdrVersion,
    fills: &'a BTreeMap<u32, CorelFill>,
    outlines: &'a BTreeMap<u32, Outline>,
}

/// Raw object before page placement (the page's size may come later in its list).
struct Pending {
    path: CorelPath,
    object: Affine,
    fill: Option<u32>,
    outline: Option<u32>,
    style: Option<u32>,
}

impl Ctx<'_, '_> {
    fn pages(&self, chunk: &Chunk, page: &mut Option<PageSpace>, index: &mut usize, t: &mut Translation) {
        let Chunk::List(list) = chunk else { return };
        if list.list_type != FourCc::PAGE {
            for c in &list.children {
                self.pages(c, page, index, t);
            }
            return;
        }
        let mut pending = Vec::new();
        let mut own_size = None;
        self.objects(list, &mut pending, &mut own_size, t);
        if own_size.is_some() {
            *page = own_size;
        }
        let space = page.unwrap_or(PageSpace { width_in: 0.0, height_in: 0.0 });
        // CorelDRAW writes objects front to back; VectorCraft stacks them bottom first.
        let shapes = pending.into_iter().rev().map(|p| self.shape(p, &space)).collect();
        let flags = list.children.iter().find_map(|c| match c {
            Chunk::Record(r) if r.kind.0 == *b"flgs" => self.riff.data(r).and_then(|d| d.get(..4)),
            _ => None,
        });
        let master = self.version >= CdrVersion(400) && flags.is_some_and(|f| u32::from_le_bytes([f[0], f[1], f[2], f[3]]) & 0x00ff_0000 != 0);
        t.pages.push(PageShapes { index: *index, master, page: space, shapes });
        *index += 1;
    }

    fn objects(&self, list: &List, out: &mut Vec<Pending>, own_size: &mut Option<PageSpace>, t: &mut Translation) {
        for c in &list.children {
            match c {
                // Pattern and clip definitions, not page content.
                Chunk::List(l) if matches!(&l.list_type.0, b"vect" | b"clpt") => {}
                Chunk::List(l) if l.list_type == FourCc::OBJECT => self.object(l, out, t),
                Chunk::List(l) => self.objects(l, out, own_size, t),
                Chunk::Record(r) if matches!(&r.kind.0, b"loda" | b"lobj") => {
                    // A page's own record, outside any object: it may carry the page size.
                    if let Some(Ok(l)) = self.riff.data(r).map(|d| decode_loda(d, self.version)) {
                        *own_size = l.page.or(*own_size);
                    }
                }
                Chunk::Record(_) => {}
            }
        }
    }

    fn object(&self, obj: &List, out: &mut Vec<Pending>, t: &mut Translation) {
        let mut loda = None;
        let mut matrices = Vec::new();
        let mut stack: Vec<&Chunk> = obj.children.iter().rev().collect();
        while let Some(c) = stack.pop() {
            match c {
                Chunk::List(l) if l.list_type != FourCc::OBJECT => stack.extend(l.children.iter().rev()),
                Chunk::List(_) => {}
                Chunk::Record(r) => {
                    let Some(data) = self.riff.data(r) else { continue };
                    match &r.kind.0 {
                        b"loda" | b"lobj" if loda.is_none() => loda = Some(data),
                        // The last trfd wins, as in libcdr's collectTransform.
                        b"trfd" => match decode_trfd(data, self.version) {
                            Ok(m) => matrices = m,
                            Err(e) => t.errors.push(e.to_string()),
                        },
                        _ => {}
                    }
                }
            }
        }
        let Some(data) = loda else { return };
        let l = match decode_loda(data, self.version) {
            Ok(l) => l,
            Err(e) => return t.errors.push(e.to_string()),
        };
        let Some(offset) = l.coords.filter(|_| l.kind.is_bezier()) else {
            *t.skipped.entry(format!("{:?}", l.kind)).or_default() += 1;
            return;
        };
        match CorelPath::decode(data, offset, l.kind, self.version) {
            Ok(path) => out.push(Pending { path, object: compose(&matrices), fill: l.fill_id, outline: l.outline_id, style: l.style_id }),
            Err(e) => t.errors.push(e.to_string()),
        }
    }

    fn shape(&self, p: Pending, page: &PageSpace) -> Shape {
        let path = p.path.to_path_data(p.object, page);
        let mut notes = Vec::new();
        let mut items = Vec::new();
        match p.fill.and_then(|id| self.fills.get(&id).map(|f| (id, f))) {
            Some((_, f)) => items.extend(f.to_fill(path.bounds(), &mut notes).map(AppearanceItem::Fill)),
            None if p.fill.is_some() => notes.push(format!("fill {} not found", p.fill.unwrap_or_default())),
            // Open paths are not filled, so only a closed one misses its style's fill.
            None if p.style.is_some() && path.is_closed() => notes.push("fill from a graphic style not translated".into()),
            None => {}
        }
        match p.outline.and_then(|id| self.outlines.get(&id)) {
            Some(o) => items.extend(o.to_stroke(p.object, &mut notes).map(AppearanceItem::Stroke)),
            None if p.outline.is_some() => notes.push(format!("outline {} not found", p.outline.unwrap_or_default())),
            None => {}
        }
        Shape { path, rule: FillRule::EvenOdd, appearance: Appearance { items, ..Appearance::default() }, notes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::riff::build;

    const V9: CdrVersion = CdrVersion(900);
    const V5: CdrVersion = CdrVersion(500);

    fn le32(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    /// A version 6+ `loda` with the given (type, payload) arguments.
    fn loda32(chunk_type: u32, args: &[(u32, Vec<u8>)]) -> Vec<u8> {
        let n = args.len() as u32;
        let header = 20u32;
        let offsets_at = header;
        let types_at = offsets_at + 4 * n;
        let mut payload_at = types_at + 4 * n;
        let mut offsets = Vec::new();
        let mut payload = Vec::new();
        for (_, p) in args {
            offsets.push(payload_at);
            payload.extend_from_slice(p);
            payload_at += p.len() as u32;
        }
        let mut v = Vec::new();
        for x in [payload_at, n, offsets_at, types_at, chunk_type] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        offsets.iter().for_each(|o| v.extend_from_slice(&o.to_le_bytes()));
        args.iter().rev().for_each(|(t, _)| v.extend_from_slice(&t.to_le_bytes()));
        v.extend_from_slice(&payload);
        v
    }

    /// Curve coordinates (version 6+): points in 1/254000 in.
    fn curve32(points: &[(i32, i32, u8)]) -> Vec<u8> {
        let mut v = (points.len() as u16).to_le_bytes().to_vec();
        v.extend_from_slice(&[0, 0]);
        points.iter().for_each(|(x, y, _)| v.extend_from_slice(&le32(&[*x, *y])));
        points.iter().for_each(|(_, _, t)| v.push(*t));
        v
    }

    const IN: i32 = 254_000;

    #[test]
    fn page_space_maps_inches_y_up_centre_to_points_y_down_corner() {
        let page = PageSpace { width_in: 8.5, height_in: 11.0 };
        let a = page.to_document();
        assert_eq!(a * Point::new(0.0, 0.0), Point::new(306.0, 396.0));
        assert_eq!(a * Point::new(-4.25, 5.5), Point::new(0.0, 0.0));
        assert_eq!(a * Point::new(4.25, -5.5), Point::new(612.0, 792.0));
        assert_eq!(page.size_pt(), (612.0, 792.0));
    }

    #[test]
    fn corel_matrices_compose_in_file_order() {
        // Scale 2 about the origin, then move 1 in right.
        let m = [corel_matrix([2.0, 0.0, 0.0, 0.0, 2.0, 0.0]), corel_matrix([1.0, 0.0, 1.0, 0.0, 1.0, 0.0])];
        assert_eq!(compose(&m) * Point::new(1.0, 1.0), Point::new(3.0, 2.0));
        // v1 is the y→x coefficient: a 90° turn counter-clockwise (y up).
        let turn = corel_matrix([0.0, -1.0, 0.0, 1.0, 0.0, 0.0]);
        assert_eq!(turn * Point::new(1.0, 0.0), Point::new(0.0, 1.0));
    }

    #[test]
    fn trfd_versions() {
        // Version 9: header (length, count, start), one offset, then type 8, 6 bytes, 6 doubles.
        let mut v = Vec::new();
        for x in [0u32, 1, 12, 16] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        v.extend_from_slice(&8u16.to_le_bytes());
        v.extend_from_slice(&[0; 6]);
        for d in [1.0f64, 0.0, 254_000.0, 0.0, -1.0, 508_000.0] {
            v.extend_from_slice(&d.to_le_bytes());
        }
        let m = decode_trfd(&v, V9).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0] * Point::new(0.0, 1.0), Point::new(1.0, 1.0));
        // Version 4: 16.16 fixed point and thousandths of an inch.
        let mut v = Vec::new();
        for x in [0u16, 1, 6, 8] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        v.extend_from_slice(&8u16.to_le_bytes());
        for x in [0x0002_0000u32, 0, 1000, 0, 0x0001_0000, 0] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        let m = decode_trfd(&v, CdrVersion(400)).unwrap();
        assert_eq!(m[0] * Point::new(1.0, 1.0), Point::new(3.0, 1.0));
        assert!(decode_trfd(&v[..10], CdrVersion(400)).is_err());
    }

    #[test]
    fn node_types_build_lines_curves_and_closed_subpaths() {
        let p = |x: f64, y: f64| Point::new(x, y);
        let path = CorelPath {
            points: vec![p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0), p(0.0, 0.0), p(5.0, 5.0), p(6.0, 5.0)],
            types: vec![
                node::MOVE,
                node::LINE,
                node::CONTROL,
                node::CONTROL,
                node::CURVE | node::CLOSED | node::SMOOTH,
                node::MOVE,
                node::CURVE, // a curve end without control points is a line
            ],
        };
        let bp = path.to_bezpath();
        use vectorcraft_geom::PathEl::*;
        assert_eq!(
            bp.elements(),
            [
                MoveTo(p(0.0, 0.0)),
                LineTo(p(1.0, 0.0)),
                CurveTo(p(1.0, 1.0), p(0.0, 1.0), p(0.0, 0.0)),
                ClosePath,
                MoveTo(p(5.0, 5.0)),
                LineTo(p(6.0, 5.0))
            ]
        );
        let doc = path.to_path_data(Affine::IDENTITY, &PageSpace { width_in: 2.0, height_in: 2.0 });
        assert_eq!(doc.subpaths.len(), 2);
        assert!(doc.subpaths[0].closed && !doc.subpaths[1].closed);
        // The closing anchor merged into the start: (0,0) in → (72,144) on a 2-inch page.
        assert_eq!(doc.subpaths[0].anchors.len(), 2);
        assert_eq!(doc.subpaths[0].anchors[0].p, p(72.0, 72.0));
        assert_eq!(doc.subpaths[0].anchors[0].h_in, p(72.0, 0.0));
    }

    #[test]
    fn curve_records_decode_in_both_precisions() {
        let rec = loda32(0x03, &[(0x1e, curve32(&[(0, 0, node::MOVE), (IN, 2 * IN, node::LINE)]))]);
        let l = decode_loda(&rec, V9).unwrap();
        assert_eq!(l.kind, ShapeKind::Curve);
        let path = CorelPath::decode(&rec, l.coords.unwrap(), l.kind, V9).unwrap();
        assert_eq!(path.points, [Point::new(0.0, 0.0), Point::new(1.0, 2.0)]);
        // Version 5: 16-bit header fields and thousandths of an inch.
        let mut v = Vec::new();
        for x in [0u16, 1, 10, 12, 3] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        v.extend_from_slice(&14u16.to_le_bytes());
        v.extend_from_slice(&0x1eu16.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&[0, 0]);
        v.extend_from_slice(&(-500i16).to_le_bytes());
        v.extend_from_slice(&1500i16.to_le_bytes());
        v.push(node::MOVE);
        let l = decode_loda(&v, V5).unwrap();
        let path = CorelPath::decode(&v, l.coords.unwrap(), l.kind, V5).unwrap();
        assert_eq!(path.points, [Point::new(-0.5, 1.5)]);
        // A count beyond the data is cut to what is there.
        let mut short = curve32(&[(0, 0, node::MOVE)]);
        short[0] = 200;
        let rec = loda32(0x03, &[(0x1e, short)]);
        assert_eq!(CorelPath::decode(&rec, decode_loda(&rec, V9).unwrap().coords.unwrap(), ShapeKind::Curve, V9).unwrap().points.len(), 1);
    }

    #[test]
    fn colours_keep_their_model() {
        let c = |model, value| CorelColor { model, palette: 0, value }.to_color();
        assert_eq!(c(0x02, u32::from_le_bytes([100, 50, 0, 10])), Ok(Color::cmyk(1.0, 0.5, 0.0, 0.1)));
        assert_eq!(c(0x03, u32::from_le_bytes([255, 0, 0, 0])), Ok(Color::cmyk(1.0, 0.0, 0.0, 0.0)));
        assert_eq!(c(0x05, u32::from_le_bytes([0x30, 0x20, 0x10, 0])), Ok(Color::rgb8(0x10, 0x20, 0x30)));
        assert_eq!(c(0x09, 255), Ok(Color::gray(0.0)));
        assert_eq!(c(0x12, u32::from_le_bytes([255, 128, 0, 0])), Ok(Color::lab(100.0, 0.0, -128.0)));
        assert_eq!(c(0x06, u32::from_le_bytes([120, 0, 255, 255])), Ok(Color::rgb(0.0, 1.0, 0.0)));
        assert_eq!(c(0x07, u32::from_le_bytes([0, 0, 128, 255])).map(|c| matches!(c, Color::Rgb { r, .. } if r > 0.99)), Ok(true));
        assert!(c(0x19, 5).is_err() && c(0x77, 0).is_err());
    }

    #[test]
    fn colour_records_by_version() {
        let mut v9 = 0x05u16.to_le_bytes().to_vec();
        v9.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        v9.extend_from_slice(&[0x30, 0x20, 0x10, 0]);
        let mut r = Reader::new(&v9, "t", V9);
        assert_eq!(CorelColor::read(&mut r).unwrap().to_color(), Ok(Color::rgb8(0x10, 0x20, 0x30)));
        // Version 13 reads model 1 as a palette colour.
        let mut v13 = 0x01u16.to_le_bytes().to_vec();
        v13.extend_from_slice(&[3, 0, 0, 0, 0, 0, 1, 0, 0, 0]);
        let mut r = Reader::new(&v13, "t", CdrVersion(1300));
        assert_eq!(CorelColor::read(&mut r).unwrap(), CorelColor { model: 0x19, palette: 3, value: 1 });
        // Version 4: four 16-bit components packed into bytes.
        let mut v4 = 0x02u16.to_le_bytes().to_vec();
        for x in [10u16, 20, 30, 40, 0] {
            v4.extend_from_slice(&x.to_le_bytes());
        }
        let mut r = Reader::new(&v4, "t", CdrVersion(400));
        assert_eq!(CorelColor::read(&mut r).unwrap().value, u32::from_le_bytes([10, 20, 30, 40]));
    }

    fn color9(model: u16, bytes: [u8; 4]) -> Vec<u8> {
        let mut v = model.to_le_bytes().to_vec();
        v.extend_from_slice(&[0; 6]);
        v.extend_from_slice(&bytes);
        v
    }

    /// A version 9 `outl` record.
    fn outl9(id: u32, line_type: u16, cap: u16, join: u16, width_in: f64, dashes: &[u16]) -> Vec<u8> {
        let mut v = id.to_le_bytes().to_vec();
        for x in [line_type, cap, join, 0] {
            v.extend_from_slice(&x.to_le_bytes());
        }
        v.extend_from_slice(&((width_in * 254_000.0) as i32).to_le_bytes());
        v.extend_from_slice(&100u16.to_le_bytes());
        v.extend_from_slice(&[0, 0]);
        v.extend_from_slice(&0i32.to_le_bytes());
        v.extend_from_slice(&[0; 52]);
        v.extend_from_slice(&color9(0x02, [0, 100, 100, 0]));
        v.extend_from_slice(&[0; 16]);
        v.extend_from_slice(&(dashes.len() as u16).to_le_bytes());
        let at = v.len();
        dashes.iter().for_each(|d| v.extend_from_slice(&d.to_le_bytes()));
        v.resize(at + 22, 0);
        v.extend_from_slice(&[0; 8]);
        v
    }

    #[test]
    fn outlines_map_caps_joins_width_and_dashes() {
        let o = Outline::decode(&outl9(7, 0x06, 1, 2, 0.5, &[3, 1]), V9).unwrap();
        assert_eq!((o.id, o.cap, o.join), (7, 1, 2));
        let mut notes = Vec::new();
        let s = o.to_stroke(Affine::scale(3.0), &mut notes).unwrap();
        assert_eq!((s.cap, s.join, s.width), (LineCap::Round, LineJoin::Bevel, 36.0));
        assert_eq!(s.paint, Paint::solid(Color::cmyk(0.0, 1.0, 1.0, 0.0)));
        assert_eq!(s.dash.as_ref().unwrap().pattern, [108.0, 36.0]);
        assert!(notes.is_empty(), "{notes:?}");
        // Scaling with the object, square caps, miter joins, no dash bit.
        let o = Outline::decode(&outl9(8, 0x02 | Outline::SCALE_WITH_OBJECT, 2, 0, 1.0 / 72.0, &[3, 1]), V9).unwrap();
        let s = o.to_stroke(Affine::scale(3.0), &mut notes).unwrap();
        assert_eq!((s.cap, s.join, s.dash.is_none()), (LineCap::Square, LineJoin::Miter, true));
        assert!((s.width - 3.0).abs() < 1e-3);
        let none = Outline::decode(&outl9(9, Outline::NONE, 0, 0, 1.0, &[]), V9).unwrap();
        assert!(none.to_stroke(Affine::IDENTITY, &mut notes).is_none());
        assert!(Outline::decode(&outl9(9, 0, 0, 0, 1.0, &[])[..40], V9).is_err());
    }

    /// A version 9 fountain `fild`.
    fn fountain9(kind: u8, pad: i32, angle_deg: i32, center: (i32, i32), mid: u8, stops: &[([u8; 4], u16)]) -> Vec<u8> {
        let mut v = 1u32.to_le_bytes().to_vec();
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&[0, 0]);
        v.push(kind);
        v.extend_from_slice(&[0; 19]);
        v.extend_from_slice(&pad.to_le_bytes());
        v.extend_from_slice(&(angle_deg * 1_000_000).to_le_bytes());
        v.extend_from_slice(&le32(&[center.0, center.1]));
        v.extend_from_slice(&[0, 0]);
        v.extend_from_slice(&0u32.to_le_bytes());
        v.push(mid);
        v.push(0);
        v.extend_from_slice(&(stops.len() as u32).to_le_bytes());
        for (rgb, at) in stops {
            v.extend_from_slice(&color9(0x05, *rgb));
            v.extend_from_slice(&u32::from(*at).to_le_bytes());
        }
        v
    }

    #[test]
    fn solid_and_fountain_fills() {
        let mut solid = 4u32.to_le_bytes().to_vec();
        solid.extend_from_slice(&1u16.to_le_bytes());
        solid.extend_from_slice(&[0, 0]);
        solid.extend_from_slice(&color9(0x09, [0, 0, 0, 0]));
        let (id, f) = CorelFill::decode(&solid, V9).unwrap();
        let mut notes = Vec::new();
        assert_eq!((id, f.to_fill(None, &mut notes).map(|l| l.paint)), (4, Some(Paint::solid(Color::gray(1.0)))));

        let b = Rect::new(0.0, 0.0, 200.0, 100.0);
        let rec = fountain9(1, 10, 90, (0, 0), 30, &[([0, 0, 255, 0], 0), ([255, 0, 0, 0], 100)]);
        let (_, f) = CorelFill::decode(&rec, V9).unwrap();
        let Some(Paint::Gradient(g)) = f.to_fill(Some(b), &mut notes).map(|l| l.paint) else { panic!() };
        assert_eq!(g.gradient.kind, GradientKind::Linear);
        assert!((g.angle - 90.0).abs() < 1e-9);
        assert_eq!(g.gradient.stops[0].color, Color::rgb8(255, 0, 0));
        assert!((g.gradient.stops[0].midpoint - 0.3).abs() < 1e-6);
        // 90°: bottom (y-down 100) to top, shortened 10% at each end.
        let geom = g.geom.unwrap();
        assert!((geom.start.y - 90.0).abs() < 1e-9 && (geom.end.y - 10.0).abs() < 1e-9, "{geom:?}");

        let rec = fountain9(2, 0, 0, (50, 50), 50, &[([0, 0, 0, 0], 0), ([255, 255, 255, 0], 100)]);
        let Some(Paint::Gradient(g)) = CorelFill::decode(&rec, V9).unwrap().1.to_fill(Some(b), &mut notes).map(|l| l.paint) else { panic!() };
        assert_eq!(g.gradient.kind, GradientKind::Radial);
        // The rim colour (first stop) moves to offset 1; the centre moves right and up.
        assert_eq!((g.gradient.stops[1].offset, g.gradient.stops[1].color), (1.0, Color::rgb8(0, 0, 0)));
        assert_eq!(g.geom.unwrap().start, Point::new(150.0, 25.0));

        let rec = fountain9(3, 0, 0, (0, 0), 50, &[([0; 4], 0), ([0; 4], 100)]);
        CorelFill::decode(&rec, V9).unwrap().1.to_fill(Some(b), &mut notes);
        assert!(notes.iter().any(|n| n.contains("conical")));
        let one = fountain9(1, 0, 0, (0, 0), 50, &[([1, 2, 3, 0], 0)]);
        assert!(matches!(CorelFill::decode(&one, V9).unwrap().1.to_fill(None, &mut notes).map(|l| l.paint), Some(Paint::Solid { .. })));
    }

    #[test]
    fn translate_places_objects_on_their_page() {
        let mut fill = 4u32.to_le_bytes().to_vec();
        fill.extend_from_slice(&1u16.to_le_bytes());
        fill.extend_from_slice(&[0, 0]);
        fill.extend_from_slice(&color9(0x05, [0, 0, 255, 0]));
        let mut page_size = le32(&[2 * IN, 2 * IN]);
        page_size.truncate(8);
        let tri = curve32(&[(0, 0, node::MOVE), (IN, 0, node::LINE), (0, IN, node::LINE | node::CLOSED)]);
        let obj = |dx: i32| {
            let mut m = Vec::new();
            for x in [0u32, 1, 12, 16] {
                m.extend_from_slice(&x.to_le_bytes());
            }
            m.extend_from_slice(&8u16.to_le_bytes());
            m.extend_from_slice(&[0; 6]);
            for d in [1.0f64, 0.0, f64::from(dx), 0.0, 1.0, 0.0] {
                m.extend_from_slice(&d.to_le_bytes());
            }
            build::list(
                b"LIST",
                b"obj ",
                &[
                    build::chunk(b"loda", &loda32(0x03, &[(0x14, 4u32.to_le_bytes().to_vec()), (0x1e, tri.clone())])),
                    build::list(b"LIST", b"trfl", &[build::chunk(b"trfd", &m)]),
                ],
            )
        };
        let file = build::list(
            b"RIFF",
            b"CDR9",
            &[
                build::list(b"LIST", b"filt", &[build::chunk(b"fild", &fill)]),
                build::list(b"LIST", b"page", &[build::chunk(b"flgs", &0x0098_0000u32.to_le_bytes())]),
                build::list(
                    b"LIST",
                    b"page",
                    &[
                        build::chunk(b"flgs", &0u32.to_le_bytes()),
                        build::chunk(b"loda", &loda32(0x00, &[(0x4aba, page_size)])),
                        build::list(b"LIST", b"layr", &[obj(0), obj(-IN)]),
                    ],
                ),
                build::list(
                    b"LIST",
                    b"page",
                    &[build::list(b"LIST", b"layr", &[build::list(b"LIST", b"obj ", &[build::chunk(b"loda", &loda32(0x01, &[]))])])],
                ),
            ],
        );
        let riff = crate::riff::parse(&file, &crate::Limits::default()).unwrap();
        let t = translate(&riff, None);
        assert!(t.errors.is_empty(), "{:?}", t.errors);
        assert_eq!(t.pages.iter().map(|p| (p.index, p.master)).collect::<Vec<_>>(), [(0, true), (1, false), (2, false)]);
        let p0 = &t.pages[1];
        assert_eq!(p0.page.size_pt(), (144.0, 144.0));
        // Bottom first: the second object in the file (moved 1 in left) is the first shape.
        let b0 = p0.shapes[0].path.bounds().unwrap();
        let b1 = p0.shapes[1].path.bounds().unwrap();
        assert_eq!((b0.x0, b0.y0, b0.x1, b0.y1), (0.0, 0.0, 72.0, 72.0));
        assert_eq!((b1.x0, b1.y0, b1.x1, b1.y1), (72.0, 0.0, 144.0, 72.0));
        assert!(p0.shapes[0].path.subpaths[0].closed);
        assert_eq!(p0.shapes[0].appearance.fill_paint(), Paint::solid(Color::rgb8(255, 0, 0)));
        assert_eq!(p0.shapes[0].rule, FillRule::EvenOdd);
        assert!(p0.shapes[0].appearance.stroke().is_none());
        // The second page keeps the size and counts its rectangle as skipped.
        assert_eq!(t.pages[2].page.size_pt(), (144.0, 144.0));
        assert_eq!(t.skipped.get("Rectangle"), Some(&1));
    }

    #[test]
    fn truncated_records_are_errors_not_panics() {
        let rec = fountain9(1, 0, 0, (0, 0), 50, &[([0; 4], 0), ([0; 4], 100)]);
        for n in 0..rec.len() {
            let _ = CorelFill::decode(&rec[..n], V9);
        }
        let o = outl9(1, 0x06, 0, 0, 1.0, &[1, 1]);
        for n in 0..o.len() {
            let _ = Outline::decode(&o[..n], V9);
        }
        let l = loda32(0x03, &[(0x1e, curve32(&[(0, 0, node::MOVE), (1, 1, node::LINE)]))]);
        for n in 0..l.len() {
            if let Ok(d) = decode_loda(&l[..n], V9)
                && let Some(off) = d.coords
            {
                let _ = CorelPath::decode(&l[..n], off, d.kind, V9);
            }
        }
        assert!(redirected(&[0; 16], CdrVersion(1600)));
        assert!(decode_loda(&[0; 16], CdrVersion(1600)).is_err());
    }
}
