//! The OpenType `MATH` table of a font face: the constants, italic corrections, accent attachment
//! points, math kerning, size variants and glyph assemblies that equation layout needs.
//!
//! `read-fonts` (and so `skrifa`) has no parser for the table yet, so this module reads the bytes
//! itself. Every read is bounds-checked: a damaged table yields `None`/zero values, never a panic.
//! Distances are in font units (the face's units per em).
//!
//! Specification: <https://learn.microsoft.com/en-us/typography/opentype/spec/math>

use std::sync::Arc;

use skrifa::{FontRef, Tag};
use vectorcraft_text::{FontDb, FontFace};

/// Families to prefer when looking for a math font, best first: the open fonts a LaTeX user knows,
/// then the one Windows ships.
const MATH_FAMILIES: &[&str] = &[
    "STIX Two Math",
    "New Computer Modern Math",
    "Latin Modern Math",
    "Libertinus Math",
    "XITS Math",
    "TeX Gyre Pagella Math",
    "TeX Gyre Termes Math",
    "Asana Math",
    "Noto Sans Math",
    "Cambria Math",
];

/// The most families without "math" in their name that [`find_math_face`] opens to look for a MATH
/// table (opening one loads its font file).
const MAX_PROBED_FAMILIES: usize = 8;

/// Most glyph variants, assembly parts and kern heights read from one construction: a damaged table
/// can claim 65535 of them.
const MAX_ENTRIES: usize = 256;

/// Does `face` have an OpenType `MATH` table?
pub fn has_math_table(face: &FontFace) -> bool {
    MathFont::new(face).is_some()
}

/// A face with a `MATH` table: the preferred math families first (STIX Two Math, New Computer
/// Modern Math, Latin Modern Math, Cambria Math…), then any family named "…Math", then a few
/// other families. `None` when the database has no math font.
///
/// Installed system fonts are looked up by name, which loads the font file of each candidate that
/// exists; nothing else is loaded.
pub fn find_math_face(db: &FontDb) -> Option<Arc<FontFace>> {
    let math_face = |family: &str| -> Option<Arc<FontFace>> {
        if !db.has_family(family) {
            return None;
        }
        // `FontDb::face` falls back to the default family for a style it lacks: only a face of
        // the requested family with a MATH table counts.
        let face = db.face(family, "Regular")?;
        (face.family.eq_ignore_ascii_case(family) && has_math_table(&face)).then_some(face)
    };
    if let Some(face) = MATH_FAMILIES.iter().find_map(|f| math_face(f)) {
        return Some(face);
    }
    let families = db.families();
    if let Some(face) = families.iter().filter(|f| f.to_ascii_lowercase().contains("math")).find_map(|f| math_face(f)) {
        return Some(face);
    }
    // A math font added under an unrelated name (loaded fonts are listed first by the database).
    families.iter().filter(|f| !f.to_ascii_lowercase().contains("math")).take(MAX_PROBED_FAMILIES).find_map(|f| math_face(f))
}

/// The 51 `MathValueRecord` constants of the table's `MathConstants` subtable, in table order. The
/// other constants (`ScriptPercentScaleDown`, `DisplayOperatorMinHeight`…) are plain numbers with
/// their own accessors on [`MathFont`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathConstant {
    MathLeading,
    AxisHeight,
    AccentBaseHeight,
    FlattenedAccentBaseHeight,
    SubscriptShiftDown,
    SubscriptTopMax,
    SubscriptBaselineDropMin,
    SuperscriptShiftUp,
    SuperscriptShiftUpCramped,
    SuperscriptBottomMin,
    SuperscriptBaselineDropMax,
    SubSuperscriptGapMin,
    SuperscriptBottomMaxWithSubscript,
    SpaceAfterScript,
    UpperLimitGapMin,
    UpperLimitBaselineRiseMin,
    LowerLimitGapMin,
    LowerLimitBaselineDropMin,
    StackTopShiftUp,
    StackTopDisplayStyleShiftUp,
    StackBottomShiftDown,
    StackBottomDisplayStyleShiftDown,
    StackGapMin,
    StackDisplayStyleGapMin,
    StretchStackTopShiftUp,
    StretchStackBottomShiftDown,
    StretchStackGapAboveMin,
    StretchStackGapBelowMin,
    FractionNumeratorShiftUp,
    FractionNumeratorDisplayStyleShiftUp,
    FractionDenominatorShiftDown,
    FractionDenominatorDisplayStyleShiftDown,
    FractionNumeratorGapMin,
    FractionNumDisplayStyleGapMin,
    FractionRuleThickness,
    FractionDenominatorGapMin,
    FractionDenomDisplayStyleGapMin,
    SkewedFractionHorizontalGap,
    SkewedFractionVerticalGap,
    OverbarVerticalGap,
    OverbarRuleThickness,
    OverbarExtraAscender,
    UnderbarVerticalGap,
    UnderbarRuleThickness,
    UnderbarExtraDescender,
    RadicalVerticalGap,
    RadicalDisplayStyleVerticalGap,
    RadicalRuleThickness,
    RadicalExtraAscender,
    RadicalKernBeforeDegree,
    RadicalKernAfterDegree,
}

/// Number of [`MathConstant`] records in a `MathConstants` subtable.
const CONSTANT_RECORDS: usize = 51;
/// Where the first `MathValueRecord` starts in `MathConstants` (after four plain numbers).
const CONSTANTS_RECORDS_AT: usize = 8;
/// Where `radicalDegreeBottomRaisePercent` is: after the records.
const RADICAL_DEGREE_RAISE_AT: usize = CONSTANTS_RECORDS_AT + 4 * CONSTANT_RECORDS;

/// A corner of a glyph that math kerning cuts into (`MathKernInfoRecord`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernCorner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// One size of a glyph that grows: the glyph and its extent along the growing direction (height
/// for vertical variants, width for horizontal ones), in font units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphVariant {
    pub gid: u32,
    pub advance: f64,
}

/// One piece of a [`GlyphAssembly`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AssemblyPart {
    pub gid: u32,
    /// How far the part may overlap the one before it (its start connector), font units.
    pub start_connector: f64,
    /// How far the part may overlap the one after it (its end connector), font units.
    pub end_connector: f64,
    /// The part's extent along the growing direction, font units.
    pub full_advance: f64,
    /// Extenders may be repeated (or left out) to reach a size.
    pub extender: bool,
}

/// How a glyph is built from parts when no single variant is large enough: parts in reading order
/// (bottom to top for vertical glyphs, left to right for horizontal ones).
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphAssembly {
    pub parts: Vec<AssemblyPart>,
    /// Italic correction of the assembled glyph, font units.
    pub italic_correction: f64,
}

/// The `MATH` table of a face, parsed for lookups.
#[derive(Clone, Debug)]
pub struct MathFont<'a> {
    table: &'a [u8],
    upem: f64,
    constants_at: usize,
    glyph_info_at: usize,
    variants_at: usize,
}

impl<'a> MathFont<'a> {
    /// The `MATH` table of `face`; `None` when it has none, or its header is damaged.
    pub fn new(face: &'a FontFace) -> Option<Self> {
        let font = FontRef::from_index(face.file_data(), face.face_index()).ok()?;
        let table = font.table_data(Tag::new(b"MATH"))?.as_bytes();
        let upem = face.units_per_em();
        Self::from_table(table, upem)
    }

    fn from_table(table: &'a [u8], upem: f64) -> Option<Self> {
        if !(upem.is_finite() && upem > 0.0) || u16_at(table, 0)? != 1 {
            return None;
        }
        let constants_at = usize::from(u16_at(table, 4)?);
        // The constants subtable is the one thing every computation needs: reject a table whose
        // constants don't fit.
        if constants_at == 0 || table.len() < constants_at.saturating_add(RADICAL_DEGREE_RAISE_AT + 2) {
            return None;
        }
        Some(Self { table, upem, constants_at, glyph_info_at: usize::from(u16_at(table, 6)?), variants_at: usize::from(u16_at(table, 8)?) })
    }

    /// Units per em of the face the distances are in.
    pub fn units_per_em(&self) -> f64 {
        self.upem
    }

    /// A `MathValueRecord` constant, in font units (0 when the table is damaged).
    pub fn constant(&self, c: MathConstant) -> f64 {
        i16_at(self.table, self.constants_at + CONSTANTS_RECORDS_AT + 4 * (c as usize)).map_or(0.0, f64::from)
    }

    /// Percentage the font size shrinks to in script style (`ScriptPercentScaleDown`; 100 = no
    /// change). Defaults to 70 when the table has no sensible value.
    pub fn script_percent_scale_down(&self) -> f64 {
        percent(i16_at(self.table, self.constants_at), 70.0)
    }

    /// Percentage for script-script style (`ScriptScriptPercentScaleDown`; default 50).
    pub fn script_script_percent_scale_down(&self) -> f64 {
        percent(i16_at(self.table, self.constants_at + 2), 50.0)
    }

    /// Least height of a delimited sub-formula that makes its delimiters stretch.
    pub fn delimited_sub_formula_min_height(&self) -> f64 {
        u16_at(self.table, self.constants_at + 4).map_or(0.0, f64::from)
    }

    /// Least height of a big operator's glyph in display style.
    pub fn display_operator_min_height(&self) -> f64 {
        u16_at(self.table, self.constants_at + 6).map_or(0.0, f64::from)
    }

    /// How far (in percent of the radical's height) the degree of a root sits above its bottom.
    pub fn radical_degree_bottom_raise_percent(&self) -> f64 {
        i16_at(self.table, self.constants_at + RADICAL_DEGREE_RAISE_AT).map_or(0.0, f64::from)
    }

    /// Italic correction of glyph `gid`: how far the slanted glyph overhangs its advance (the
    /// superscript that follows moves right by it). 0 when the font gives none.
    pub fn italic_correction(&self, gid: u32) -> f64 {
        self.glyph_info_value(0, gid).unwrap_or(0.0)
    }

    /// Where over glyph `gid` an accent's own attachment point goes (x, font units); `None` when the
    /// font doesn't say (the glyph's middle is the usual guess).
    pub fn top_accent_attachment(&self, gid: u32) -> Option<f64> {
        self.glyph_info_value(2, gid)
    }

    /// Is the glyph in the table's list of extended shapes (tall or wide glyphs scripts attach to
    /// differently)?
    pub fn is_extended_shape(&self, gid: u32) -> bool {
        let Some(info) = self.glyph_info() else { return false };
        let Some(off) = u16_at(self.table, info + 4).filter(|o| *o != 0) else { return false };
        coverage_index(self.table, info + usize::from(off), gid).is_some()
    }

    /// The math kern of one corner of glyph `gid` at `height` above the baseline, font units:
    /// the amount a script may tuck in (negative) or must move out (positive). 0 without kerning.
    pub fn math_kern(&self, gid: u32, corner: KernCorner, height: f64) -> f64 {
        self.math_kern_at(gid, corner, height).unwrap_or(0.0)
    }

    fn math_kern_at(&self, gid: u32, corner: KernCorner, height: f64) -> Option<f64> {
        let kern_info = self.glyph_info()? + usize::from(u16_at(self.table, self.glyph_info()? + 6).filter(|o| *o != 0)?);
        let count = usize::from(u16_at(self.table, kern_info + 2)?);
        let index = coverage_index(self.table, kern_info + usize::from(u16_at(self.table, kern_info)?), gid)?;
        if index >= count {
            return None;
        }
        // The record: four offsets (top right, top left, bottom right, bottom left) to MathKern.
        let slot = match corner {
            KernCorner::TopRight => 0,
            KernCorner::TopLeft => 1,
            KernCorner::BottomRight => 2,
            KernCorner::BottomLeft => 3,
        };
        let kern = u16_at(self.table, kern_info + 4 + index * 8 + slot * 2).filter(|o| *o != 0)?;
        let kern = kern_info + usize::from(kern);
        let n = usize::from(u16_at(self.table, kern)?).min(MAX_ENTRIES);
        // n correction heights, then n + 1 kern values: the kern of the first height range whose
        // upper bound exceeds `height`.
        let mut i = 0;
        while i < n {
            let at = i16_at(self.table, kern + 2 + 4 * i)?;
            if height < f64::from(at) {
                break;
            }
            i += 1;
        }
        Some(f64::from(i16_at(self.table, kern + 2 + 4 * n + 4 * i)?))
    }

    /// Value of the `MathValueRecord` of glyph `gid` in one of the two coverage-indexed lists of
    /// `MathGlyphInfo`: `which` is the offset of its subtable's offset: 0 = italic corrections,
    /// 2 = top accent attachments.
    fn glyph_info_value(&self, which: usize, gid: u32) -> Option<f64> {
        let info = self.glyph_info()?;
        let list = info + usize::from(u16_at(self.table, info + which).filter(|o| *o != 0)?);
        let count = usize::from(u16_at(self.table, list + 2)?);
        let index = coverage_index(self.table, list + usize::from(u16_at(self.table, list)?), gid)?;
        if index >= count {
            return None;
        }
        i16_at(self.table, list + 4 + 4 * index).map(f64::from)
    }

    fn glyph_info(&self) -> Option<usize> {
        (self.glyph_info_at != 0).then_some(self.glyph_info_at)
    }

    /// Least overlap of two connected assembly parts, font units.
    pub fn min_connector_overlap(&self) -> f64 {
        if self.variants_at == 0 { 0.0 } else { u16_at(self.table, self.variants_at).map_or(0.0, f64::from) }
    }

    /// The sizes glyph `gid` comes in when it grows taller (parentheses, radicals, big operators),
    /// smallest first; empty when the font has none. The glyph itself is usually the first.
    pub fn vertical_variants(&self, gid: u32) -> Vec<GlyphVariant> {
        self.construction(gid, true).map(|c| self.variants_of(c)).unwrap_or_default()
    }

    /// The sizes glyph `gid` comes in when it grows wider (wide accents, long arrows).
    pub fn horizontal_variants(&self, gid: u32) -> Vec<GlyphVariant> {
        self.construction(gid, false).map(|c| self.variants_of(c)).unwrap_or_default()
    }

    /// How glyph `gid` is assembled from parts to grow taller than its largest variant.
    pub fn vertical_assembly(&self, gid: u32) -> Option<GlyphAssembly> {
        self.construction(gid, true).and_then(|c| self.assembly_of(c))
    }

    /// How glyph `gid` is assembled from parts to grow wider than its largest variant.
    pub fn horizontal_assembly(&self, gid: u32) -> Option<GlyphAssembly> {
        self.construction(gid, false).and_then(|c| self.assembly_of(c))
    }

    /// Offset of the `MathGlyphConstruction` of `gid`.
    fn construction(&self, gid: u32, vertical: bool) -> Option<usize> {
        let v = (self.variants_at != 0).then_some(self.variants_at)?;
        let vertical_count = usize::from(u16_at(self.table, v + 6)?);
        let (coverage_at, count, first) =
            if vertical { (v + 2, vertical_count, v + 10) } else { (v + 4, usize::from(u16_at(self.table, v + 8)?), v + 10 + 2 * vertical_count) };
        let coverage = u16_at(self.table, coverage_at).filter(|o| *o != 0)?;
        let index = coverage_index(self.table, v + usize::from(coverage), gid)?;
        if index >= count {
            return None;
        }
        let off = u16_at(self.table, first + 2 * index).filter(|o| *o != 0)?;
        Some(v + usize::from(off))
    }

    fn variants_of(&self, construction: usize) -> Vec<GlyphVariant> {
        let n = u16_at(self.table, construction + 2).map_or(0, usize::from).min(MAX_ENTRIES);
        (0..n)
            .filter_map(|i| {
                let at = construction + 4 + 4 * i;
                Some(GlyphVariant { gid: u32::from(u16_at(self.table, at)?), advance: f64::from(u16_at(self.table, at + 2)?) })
            })
            .collect()
    }

    fn assembly_of(&self, construction: usize) -> Option<GlyphAssembly> {
        let assembly = construction + usize::from(u16_at(self.table, construction).filter(|o| *o != 0)?);
        let italic_correction = i16_at(self.table, assembly).map_or(0.0, f64::from);
        let n = usize::from(u16_at(self.table, assembly + 4)?).min(MAX_ENTRIES);
        let parts: Vec<AssemblyPart> = (0..n)
            .filter_map(|i| {
                let at = assembly + 6 + 10 * i;
                Some(AssemblyPart {
                    gid: u32::from(u16_at(self.table, at)?),
                    start_connector: f64::from(u16_at(self.table, at + 2)?),
                    end_connector: f64::from(u16_at(self.table, at + 4)?),
                    full_advance: f64::from(u16_at(self.table, at + 6)?),
                    extender: u16_at(self.table, at + 8)? & 1 != 0,
                })
            })
            .collect();
        (!parts.is_empty()).then_some(GlyphAssembly { parts, italic_correction })
    }
}

/// A percentage from the table: out-of-range values (0, negative, over 100) fall back to `default`.
fn percent(v: Option<i16>, default: f64) -> f64 {
    match v {
        Some(p) if (1..=100).contains(&p) => f64::from(p),
        _ => default,
    }
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    let b = data.get(at..at.checked_add(2)?)?;
    Some(u16::from_be_bytes([*b.first()?, *b.get(1)?]))
}

fn i16_at(data: &[u8], at: usize) -> Option<i16> {
    u16_at(data, at).map(|v| v as i16)
}

/// Index of `gid` in the coverage table at `at` (format 1: sorted glyph list; format 2: ranges).
fn coverage_index(data: &[u8], at: usize, gid: u32) -> Option<usize> {
    let gid = u16::try_from(gid).ok()?;
    let count = usize::from(u16_at(data, at + 2)?);
    match u16_at(data, at)? {
        1 => {
            let (mut lo, mut hi) = (0, count);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                let g = u16_at(data, at + 4 + 2 * mid)?;
                match g.cmp(&gid) {
                    std::cmp::Ordering::Equal => return Some(mid),
                    std::cmp::Ordering::Less => lo = mid + 1,
                    std::cmp::Ordering::Greater => hi = mid,
                }
            }
            None
        }
        2 => {
            let (mut lo, mut hi) = (0, count);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                let rec = at + 4 + 6 * mid;
                let (start, end) = (u16_at(data, rec)?, u16_at(data, rec + 2)?);
                if gid < start {
                    hi = mid;
                } else if gid > end {
                    lo = mid + 1;
                } else {
                    return Some(usize::from(u16_at(data, rec + 4)?) + usize::from(gid - start));
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_format_1_is_a_binary_search_over_glyphs() {
        // format 1, three glyphs: 5, 9, 40
        let t = [0, 1, 0, 3, 0, 5, 0, 9, 0, 40];
        assert_eq!(coverage_index(&t, 0, 5), Some(0));
        assert_eq!(coverage_index(&t, 0, 9), Some(1));
        assert_eq!(coverage_index(&t, 0, 40), Some(2));
        assert_eq!(coverage_index(&t, 0, 6), None);
        assert_eq!(coverage_index(&t, 0, 70_000), None);
    }

    #[test]
    fn coverage_format_2_maps_ranges_to_indices() {
        // format 2, two ranges: 10..=12 from index 0, 20..=21 from index 3
        let t = [0, 2, 0, 2, 0, 10, 0, 12, 0, 0, 0, 20, 0, 21, 0, 3];
        assert_eq!(coverage_index(&t, 0, 11), Some(1));
        assert_eq!(coverage_index(&t, 0, 21), Some(4));
        assert_eq!(coverage_index(&t, 0, 13), None);
    }

    #[test]
    fn a_truncated_or_empty_table_reads_as_nothing() {
        assert!(MathFont::from_table(&[], 1000.0).is_none());
        assert!(MathFont::from_table(&[0, 1, 0, 0, 0, 10, 0, 0, 0, 0], 1000.0).is_none());
        assert_eq!(coverage_index(&[0, 1, 0xff, 0xff], 0, 1), None);
        assert_eq!(coverage_index(&[], 0, 1), None);
        assert_eq!(u16_at(&[1], 0), None);
        assert_eq!(u16_at(&[1, 2], usize::MAX), None);
    }

    #[test]
    fn a_minimal_table_gives_its_constants() {
        let mut t = vec![0u8, 1, 0, 0, 0, 10, 0, 0, 0, 0];
        t.extend([0, 60, 0, 40, 0, 0, 0x03, 0xe8]); // percents, delimited min, display op min
        for i in 0..CONSTANT_RECORDS as i16 {
            t.extend((i * 10).to_be_bytes());
            t.extend([0, 0]);
        }
        t.extend([0, 55]);
        let f = MathFont::from_table(&t, 1000.0).unwrap();
        assert_eq!(f.script_percent_scale_down(), 60.0);
        assert_eq!(f.script_script_percent_scale_down(), 40.0);
        assert_eq!(f.display_operator_min_height(), 1000.0);
        assert_eq!(f.constant(MathConstant::MathLeading), 0.0);
        assert_eq!(f.constant(MathConstant::AxisHeight), 10.0);
        assert_eq!(f.constant(MathConstant::RadicalKernAfterDegree), 500.0);
        assert_eq!(f.radical_degree_bottom_raise_percent(), 55.0);
        assert_eq!(f.italic_correction(3), 0.0);
        assert!(f.vertical_variants(3).is_empty());
        assert_eq!(f.min_connector_overlap(), 0.0);
    }
}
