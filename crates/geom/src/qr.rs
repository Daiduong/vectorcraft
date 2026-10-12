//! QR Code symbols (ISO/IEC 18004), written from the public specification: byte-mode data,
//! versions 1–40, the four error correction levels, and the mask chosen by the penalty rules.
//!
//! [`QrCode::encode`] gives the module grid; [`QrCode::to_path`] draws its dark modules as one
//! closed rectangle per horizontal run, so the symbol is a single fillable path.

use crate::{PathData, Point, SubPath};

/// Error correction level: how much of the symbol can be damaged and still read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecc {
    /// About 7 % of the codewords.
    Low,
    /// About 15 %.
    Medium,
    /// About 25 %.
    Quartile,
    /// About 30 %.
    High,
}

impl Ecc {
    fn index(self) -> usize {
        match self {
            Ecc::Low => 0,
            Ecc::Medium => 1,
            Ecc::Quartile => 2,
            Ecc::High => 3,
        }
    }

    /// The two format-information bits (L 01, M 00, Q 11, H 10).
    fn format_bits(self) -> u32 {
        match self {
            Ecc::Low => 1,
            Ecc::Medium => 0,
            Ecc::Quartile => 3,
            Ecc::High => 2,
        }
    }

    /// `L`, `M`, `Q` or `H` (either case).
    pub fn from_letter(s: &str) -> Option<Self> {
        match s {
            "L" | "l" => Some(Ecc::Low),
            "M" | "m" => Some(Ecc::Medium),
            "Q" | "q" => Some(Ecc::Quartile),
            "H" | "h" => Some(Ecc::High),
            _ => None,
        }
    }

    pub fn letter(self) -> &'static str {
        ["L", "M", "Q", "H"][self.index()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QrError {
    #[error("nothing to encode")]
    Empty,
    #[error("{len} bytes don't fit in a QR code at level {level} (at most {max})")]
    TooLong { len: usize, level: &'static str, max: usize },
}

/// Error correction codewords per block, by level (L, M, Q, H) and version (index 0 unused).
const ECC_PER_BLOCK: [[u8; 41]; 4] = [
    [
        0, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30,
        30, 30, 30, 30, 30,
    ],
    [
        0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
        28, 28, 28, 28, 28, 28,
    ],
    [
        0, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30,
        30, 30, 30, 30, 30, 30,
    ],
    [
        0, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
        30, 30, 30, 30, 30, 30,
    ],
];

/// Error correction blocks, by level and version (index 0 unused).
const BLOCKS: [[u8; 41]; 4] = [
    [0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
    [
        0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45,
        47, 49,
    ],
    [
        0, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59,
        62, 65, 68,
    ],
    [
        0, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70,
        74, 77, 81,
    ],
];

/// Modules on a side for `version`.
fn side(version: usize) -> usize {
    version * 4 + 17
}

/// Data and error correction modules of `version` (everything outside the function patterns).
fn raw_modules(version: usize) -> usize {
    let mut n = (16 * version + 128) * version + 64;
    if version >= 2 {
        let align = version / 7 + 2;
        n -= (25 * align - 10) * align - 55;
        if version >= 7 {
            n -= 36;
        }
    }
    n
}

fn ecc_per_block(version: usize, ecc: Ecc) -> usize {
    ECC_PER_BLOCK.get(ecc.index()).and_then(|r| r.get(version)).copied().unwrap_or(0) as usize
}

fn blocks(version: usize, ecc: Ecc) -> usize {
    BLOCKS.get(ecc.index()).and_then(|r| r.get(version)).copied().unwrap_or(1).max(1) as usize
}

/// Data codewords `version` holds at `ecc`.
fn data_codewords(version: usize, ecc: Ecc) -> usize {
    (raw_modules(version) / 8).saturating_sub(ecc_per_block(version, ecc) * blocks(version, ecc))
}

/// Bits of the byte-mode character count for `version`.
fn count_bits(version: usize) -> usize {
    if version <= 9 { 8 } else { 16 }
}

/// Bytes a byte-mode segment can carry in `version` at `ecc`.
fn byte_capacity(version: usize, ecc: Ecc) -> usize {
    (data_codewords(version, ecc) * 8).saturating_sub(4 + count_bits(version)) / 8
}

/// Centres of the alignment patterns along one axis.
fn alignment_positions(version: usize) -> Vec<usize> {
    if version == 1 {
        return vec![];
    }
    let n = version / 7 + 2;
    let step = if version == 32 { 26 } else { (version * 4 + n * 2 + 1) / (n * 2 - 2) * 2 };
    let last = side(version) - 7;
    let mut out: Vec<usize> = (0..n - 1).map(|i| last - i * step).collect();
    out.push(6);
    out.reverse();
    out
}

// --- Reed–Solomon over GF(256), polynomial x^8 + x^4 + x^3 + x^2 + 1 ---------------------------

fn gf_mul(a: u8, b: u8) -> u8 {
    let (mut a, mut b, mut r) = (a as u16, b, 0u16);
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        a <<= 1;
        if a & 0x100 != 0 {
            a ^= 0x11D;
        }
        b >>= 1;
    }
    r as u8
}

/// The generator polynomial of degree `n` (roots α^0 … α^(n−1)), highest coefficient first,
/// without its leading 1.
fn rs_generator(n: usize) -> Vec<u8> {
    let mut g = vec![0u8; n];
    if let Some(last) = g.last_mut() {
        *last = 1;
    }
    let mut root = 1u8;
    for _ in 0..n {
        for j in 0..n {
            let next = g.get(j + 1).copied().unwrap_or(0);
            if let Some(c) = g.get_mut(j) {
                *c = gf_mul(*c, root) ^ next;
            }
        }
        root = gf_mul(root, 2);
    }
    g
}

/// The `generator.len()` error correction codewords of `data`.
fn rs_remainder(data: &[u8], generator: &[u8]) -> Vec<u8> {
    let mut r = vec![0u8; generator.len()];
    for &b in data {
        let factor = b ^ r.first().copied().unwrap_or(0);
        r.rotate_left(1);
        if let Some(last) = r.last_mut() {
            *last = 0;
        }
        for (c, &g) in r.iter_mut().zip(generator) {
            *c ^= gf_mul(g, factor);
        }
    }
    r
}

// --- The symbol ------------------------------------------------------------------------------

/// An encoded QR Code: a square of dark and light modules (without the quiet zone).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QrCode {
    pub version: usize,
    pub ecc: Ecc,
    pub mask: u8,
    size: usize,
    dark: Vec<bool>,
    function: Vec<bool>,
}

impl QrCode {
    /// Encode `data` in byte mode at `ecc`, in the smallest version that holds it.
    pub fn encode(data: &[u8], ecc: Ecc) -> Result<Self, QrError> {
        if data.is_empty() {
            return Err(QrError::Empty);
        }
        let version = (1..=40).find(|&v| byte_capacity(v, ecc) >= data.len()).ok_or(QrError::TooLong {
            len: data.len(),
            level: ecc.letter(),
            max: byte_capacity(40, ecc),
        })?;
        let codewords = codewords(data, version, ecc);
        let size = side(version);
        let mut qr = QrCode { version, ecc, mask: 0, size, dark: vec![false; size * size], function: vec![false; size * size] };
        qr.draw_function_patterns();
        qr.draw_codewords(&codewords);
        // The mask with the lowest penalty (ties keep the lower number).
        let mut best = (u32::MAX, 0u8);
        for mask in 0..8u8 {
            qr.apply_mask(mask);
            qr.draw_format(mask);
            let p = qr.penalty();
            if p < best.0 {
                best = (p, mask);
            }
            qr.apply_mask(mask);
        }
        qr.mask = best.1;
        qr.apply_mask(best.1);
        qr.draw_format(best.1);
        Ok(qr)
    }

    /// Modules on a side.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Whether the module at column `x`, row `y` is dark (false outside the symbol).
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        x < self.size && self.dark.get(y * self.size + x).copied().unwrap_or(false)
    }

    /// The dark modules as one path: the symbol's top-left corner at `origin`, each module
    /// `module` points wide, one closed rectangle per horizontal run of dark modules.
    pub fn to_path(&self, origin: Point, module: f64) -> PathData {
        let mut subpaths = vec![];
        for y in 0..self.size {
            let mut x = 0;
            while x < self.size {
                if !self.is_dark(x, y) {
                    x += 1;
                    continue;
                }
                let start = x;
                while self.is_dark(x, y) {
                    x += 1;
                }
                let (x0, x1) = (origin.x + start as f64 * module, origin.x + x as f64 * module);
                let (y0, y1) = (origin.y + y as f64 * module, origin.y + (y + 1) as f64 * module);
                subpaths.push(SubPath::polyline(&[Point::new(x0, y0), Point::new(x1, y0), Point::new(x1, y1), Point::new(x0, y1)], true));
            }
        }
        PathData { subpaths }
    }

    fn set(&mut self, x: usize, y: usize, dark: bool, function: bool) {
        if x >= self.size {
            return;
        }
        let i = y * self.size + x;
        if let Some(m) = self.dark.get_mut(i) {
            *m = dark;
        }
        if function && let Some(f) = self.function.get_mut(i) {
            *f = true;
        }
    }

    fn is_function(&self, x: usize, y: usize) -> bool {
        x < self.size && self.function.get(y * self.size + x).copied().unwrap_or(true)
    }

    fn draw_function_patterns(&mut self) {
        let n = self.size;
        // Timing patterns.
        for i in 0..n {
            self.set(6, i, i % 2 == 0, true);
            self.set(i, 6, i % 2 == 0, true);
        }
        // Finder patterns with their separators.
        for (cx, cy) in [(3, 3), (n - 4, 3), (3, n - 4)] {
            self.draw_finder(cx, cy);
        }
        // Alignment patterns, except where they would cover a finder.
        let pos = alignment_positions(self.version);
        let last = pos.len().saturating_sub(1);
        for (i, &ay) in pos.iter().enumerate() {
            for (j, &ax) in pos.iter().enumerate() {
                let on_finder = (i == 0 && j == 0) || (i == 0 && j == last) || (i == last && j == 0);
                if !on_finder {
                    for dy in 0..5 {
                        for dx in 0..5 {
                            let ring = (dx as i32 - 2).abs().max((dy as i32 - 2).abs());
                            self.set(ax + dx - 2, ay + dy - 2, ring != 1, true);
                        }
                    }
                }
            }
        }
        // Reserve the format areas (drawn for real once the mask is known) and the dark module.
        self.draw_format(0);
        self.draw_version();
    }

    /// A 7×7 finder centred on (`cx`, `cy`) and the light ring around it, clipped to the symbol.
    fn draw_finder(&mut self, cx: usize, cy: usize) {
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let (x, y) = (cx as i32 + dx, cy as i32 + dy);
                if x < 0 || y < 0 || x >= self.size as i32 || y >= self.size as i32 {
                    continue;
                }
                let ring = dx.abs().max(dy.abs());
                self.set(x as usize, y as usize, ring != 2 && ring != 4, true);
            }
        }
    }

    /// The 15 format bits (level and mask, BCH protected) in both copies, and the dark module.
    fn draw_format(&mut self, mask: u8) {
        let data = (self.ecc.format_bits() << 3) | mask as u32;
        let mut rem = data;
        for _ in 0..10 {
            rem = (rem << 1) ^ ((rem >> 9) * 0x537);
        }
        let bits = ((data << 10) | rem) ^ 0x5412;
        let bit = |i: u32| (bits >> i) & 1 != 0;
        let n = self.size;
        // Around the top-left finder.
        for i in 0..6 {
            self.set(8, i, bit(i as u32), true);
        }
        self.set(8, 7, bit(6), true);
        self.set(8, 8, bit(7), true);
        self.set(7, 8, bit(8), true);
        for i in 9..15 {
            self.set(14 - i, 8, bit(i as u32), true);
        }
        // Split between the other two finders.
        for i in 0..8 {
            self.set(n - 1 - i, 8, bit(i as u32), true);
        }
        for i in 8..15 {
            self.set(8, n - 15 + i, bit(i as u32), true);
        }
        self.set(8, n - 8, true, true);
    }

    /// The 18 version bits (Golay protected) in their two 6×3 blocks, from version 7.
    fn draw_version(&mut self) {
        if self.version < 7 {
            return;
        }
        let mut rem = self.version as u32;
        for _ in 0..12 {
            rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
        }
        let bits = ((self.version as u32) << 12) | rem;
        for i in 0..18 {
            let dark = (bits >> i) & 1 != 0;
            let (a, b) = (self.size - 11 + i % 3, i / 3);
            self.set(a, b, dark, true);
            self.set(b, a, dark, true);
        }
    }

    /// The codewords' bits in the zigzag order, two columns at a time from the bottom right,
    /// skipping the vertical timing pattern; leftover modules stay light.
    fn draw_codewords(&mut self, codewords: &[u8]) {
        let n = self.size;
        let total = codewords.len() * 8;
        let mut i = 0;
        let mut right = n as i32 - 1;
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            let upward = ((right + 1) & 2) == 0;
            for vert in 0..n {
                let y = if upward { n - 1 - vert } else { vert };
                for j in 0..2 {
                    let x = (right - j) as usize;
                    if !self.is_function(x, y) && i < total {
                        let byte = codewords.get(i >> 3).copied().unwrap_or(0);
                        self.set(x, y, (byte >> (7 - (i & 7))) & 1 != 0, false);
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
    }

    /// Flip the data modules where mask pattern `mask` says so (applying it twice undoes it).
    fn apply_mask(&mut self, mask: u8) {
        for y in 0..self.size {
            for x in 0..self.size {
                let flip = match mask {
                    0 => (x + y) % 2 == 0,
                    1 => y % 2 == 0,
                    2 => x % 3 == 0,
                    3 => (x + y) % 3 == 0,
                    4 => (x / 3 + y / 2) % 2 == 0,
                    5 => x * y % 2 + x * y % 3 == 0,
                    6 => (x * y % 2 + x * y % 3) % 2 == 0,
                    _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
                };
                if flip && !self.is_function(x, y) {
                    let d = self.is_dark(x, y);
                    self.set(x, y, !d, false);
                }
            }
        }
    }

    /// The mask penalty score: long runs, 2×2 blocks, finder-like patterns, dark balance.
    fn penalty(&self) -> u32 {
        let n = self.size;
        let mut score = 0u32;
        let line =
            |k: usize, horizontal: bool| -> Vec<bool> { (0..n).map(|i| if horizontal { self.is_dark(i, k) } else { self.is_dark(k, i) }).collect() };
        for k in 0..n {
            for horizontal in [true, false] {
                let l = line(k, horizontal);
                // Runs of five or more of one colour.
                let mut run = 1;
                for i in 1..=n {
                    if i < n && l.get(i) == l.get(i - 1) {
                        run += 1;
                    } else {
                        if run >= 5 {
                            score += 3 + (run - 5) as u32;
                        }
                        run = 1;
                    }
                }
                // 1:1:3:1:1 dark–light pattern with four light modules on one side.
                const A: [bool; 11] = [true, false, true, true, true, false, true, false, false, false, false];
                const B: [bool; 11] = [false, false, false, false, true, false, true, true, true, false, true];
                for w in l.windows(11) {
                    if w == A || w == B {
                        score += 40;
                    }
                }
            }
        }
        for y in 0..n.saturating_sub(1) {
            for x in 0..n.saturating_sub(1) {
                let c = self.is_dark(x, y);
                if c == self.is_dark(x + 1, y) && c == self.is_dark(x, y + 1) && c == self.is_dark(x + 1, y + 1) {
                    score += 3;
                }
            }
        }
        let total = n * n;
        let dark = self.dark.iter().filter(|&&d| d).count();
        // Each full 5 % step away from half dark costs 10.
        let k = ((dark * 20).abs_diff(total * 10) + total - 1) / total.max(1);
        score + 10 * k.saturating_sub(1) as u32
    }
}

/// The final codeword sequence for `data` in `version` at `ecc`: the bit stream padded to the
/// data capacity, split into blocks, each with its error correction, interleaved.
fn codewords(data: &[u8], version: usize, ecc: Ecc) -> Vec<u8> {
    let capacity = data_codewords(version, ecc);
    let mut bits: Vec<bool> = vec![];
    let mut push = |value: usize, len: usize| {
        for i in (0..len).rev() {
            bits.push((value >> i) & 1 != 0);
        }
    };
    push(0b0100, 4);
    push(data.len(), count_bits(version));
    for &b in data {
        push(b as usize, 8);
    }
    let limit = capacity * 8;
    let terminator = limit.saturating_sub(bits.len()).min(4);
    bits.extend(std::iter::repeat_n(false, terminator));
    while !bits.len().is_multiple_of(8) {
        bits.push(false);
    }
    let mut bytes: Vec<u8> = bits.chunks(8).map(|c| c.iter().fold(0u8, |a, &b| (a << 1) | b as u8)).collect();
    for pad in [0xEC, 0x11].into_iter().cycle() {
        if bytes.len() >= capacity {
            break;
        }
        bytes.push(pad);
    }

    let nblocks = blocks(version, ecc);
    let ecc_len = ecc_per_block(version, ecc);
    let raw = raw_modules(version) / 8;
    let short_len = raw / nblocks;
    let short_blocks = nblocks - raw % nblocks;
    let generator = rs_generator(ecc_len);
    let mut split: Vec<(Vec<u8>, Vec<u8>)> = vec![];
    let mut at = 0;
    for b in 0..nblocks {
        let len = short_len - ecc_len + usize::from(b >= short_blocks);
        let block = bytes.get(at..at + len).map(<[u8]>::to_vec).unwrap_or_default();
        at += len;
        let ec = rs_remainder(&block, &generator);
        split.push((block, ec));
    }
    let mut out = Vec::with_capacity(raw);
    let longest = split.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
    for i in 0..longest {
        out.extend(split.iter().filter_map(|(d, _)| d.get(i)));
    }
    for i in 0..ecc_len {
        out.extend(split.iter().filter_map(|(_, e)| e.get(i)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacities_match_the_specification() {
        // Total codewords and byte-mode capacities from the specification's tables.
        assert_eq!(raw_modules(1) / 8, 26);
        assert_eq!(raw_modules(7) / 8, 196);
        assert_eq!(raw_modules(40) / 8, 3706);
        assert_eq!(data_codewords(1, Ecc::Low), 19);
        assert_eq!(data_codewords(1, Ecc::High), 9);
        assert_eq!(data_codewords(40, Ecc::Low), 2956);
        assert_eq!(byte_capacity(1, Ecc::Medium), 14);
        assert_eq!(byte_capacity(3, Ecc::Medium), 42);
        assert_eq!(byte_capacity(10, Ecc::Quartile), 151);
        assert_eq!(byte_capacity(40, Ecc::Low), 2953);
        assert_eq!(byte_capacity(40, Ecc::High), 1273);
        // Every version and level splits into whole blocks with data left over.
        for v in 1..=40 {
            for e in [Ecc::Low, Ecc::Medium, Ecc::Quartile, Ecc::High] {
                assert!(raw_modules(v) / 8 / blocks(v, e) > ecc_per_block(v, e), "v{v} {e:?}");
            }
        }
    }

    #[test]
    fn alignment_positions_match_the_specification() {
        assert_eq!(alignment_positions(1), Vec::<usize>::new());
        assert_eq!(alignment_positions(2), vec![6, 18]);
        assert_eq!(alignment_positions(7), vec![6, 22, 38]);
        assert_eq!(alignment_positions(32), vec![6, 34, 60, 86, 112, 138]);
        assert_eq!(alignment_positions(40), vec![6, 30, 58, 86, 114, 142, 170]);
    }

    /// Evaluate the codeword polynomial at α^i: zero for every root of the generator when the
    /// error correction is right.
    fn syndromes(block: &[u8], n: usize) -> Vec<u8> {
        let mut alpha = 1u8;
        (0..n)
            .map(|_| {
                let s = block.iter().fold(0u8, |acc, &c| gf_mul(acc, alpha) ^ c);
                alpha = gf_mul(alpha, 2);
                s
            })
            .collect()
    }

    #[test]
    fn reed_solomon_codewords_have_zero_syndromes() {
        let data: Vec<u8> = (0..40u8).map(|i| i.wrapping_mul(37) ^ 0x5A).collect();
        for n in [7, 10, 17, 30] {
            let g = rs_generator(n);
            let mut block = data.clone();
            block.extend(rs_remainder(&data, &g));
            assert!(syndromes(&block, n).iter().all(|&s| s == 0), "degree {n}");
            block[3] ^= 1;
            assert!(syndromes(&block, n).iter().any(|&s| s != 0));
        }
    }

    #[test]
    fn known_format_bits() {
        // Level M, mask 0 and level L, mask 4, from the specification's format table.
        let fmt = |e: Ecc, m: u32| {
            let data = (e.format_bits() << 3) | m;
            let mut rem = data;
            for _ in 0..10 {
                rem = (rem << 1) ^ ((rem >> 9) * 0x537);
            }
            ((data << 10) | rem) ^ 0x5412
        };
        assert_eq!(fmt(Ecc::Medium, 0), 0b101010000010010);
        assert_eq!(fmt(Ecc::Low, 4), 0b110011000101111);
    }

    #[test]
    fn symbol_structure() {
        let qr = QrCode::encode(b"https://ape.photomates.nohost.me/", Ecc::Medium).unwrap();
        assert_eq!(qr.version, 3);
        assert_eq!(qr.size(), 29);
        // Finder corners and centres, timing pattern, dark module.
        for (x, y) in [(0, 0), (6, 6), (3, 3), (22, 0), (25, 3), (0, 22), (3, 25)] {
            assert!(qr.is_dark(x, y), "({x}, {y})");
        }
        for (x, y) in [(7, 7), (1, 1), (5, 3), (21, 7)] {
            assert!(!qr.is_dark(x, y), "({x}, {y})");
        }
        for i in 8..21 {
            assert_eq!(qr.is_dark(i, 6), i % 2 == 0);
            assert_eq!(qr.is_dark(6, i), i % 2 == 0);
        }
        assert!(qr.is_dark(8, qr.size() - 8));
        // The alignment pattern at (22, 22).
        assert!(qr.is_dark(22, 22) && !qr.is_dark(21, 22) && qr.is_dark(20, 22));
        // Both format copies agree.
        let a: Vec<bool> = (0..6).map(|i| qr.is_dark(8, i)).collect();
        let b: Vec<bool> = (0..6).map(|i| qr.is_dark(28 - i, 8)).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn version_information_from_version_7() {
        assert_eq!(QrCode::encode(&[b'a'; 106], Ecc::Medium).unwrap().version, 6);
        // Version 7's bits are 000111110010010100 (specification annex D), in both blocks.
        let v7 = QrCode::encode(&[b'a'; 120], Ecc::Medium).unwrap();
        assert_eq!(v7.version, 7);
        let n = v7.size();
        let top: u32 = (0..18).map(|i| u32::from(v7.is_dark(n - 11 + i % 3, i / 3)) << i).sum();
        let left: u32 = (0..18).map(|i| u32::from(v7.is_dark(i / 3, n - 11 + i % 3)) << i).sum();
        assert_eq!(top, 0b000111110010010100);
        assert_eq!(left, top);
    }

    #[test]
    fn too_long_and_empty_fail() {
        assert_eq!(QrCode::encode(b"", Ecc::Low), Err(QrError::Empty));
        assert!(matches!(QrCode::encode(&[0u8; 2954], Ecc::Low), Err(QrError::TooLong { max: 2953, .. })));
        assert!(QrCode::encode(&[0u8; 2953], Ecc::Low).is_ok());
    }

    #[test]
    fn path_has_one_rectangle_per_run() {
        let qr = QrCode::encode(b"VectorCraft", Ecc::Low).unwrap();
        let path = qr.to_path(Point::new(10.0, 20.0), 2.0);
        let runs: usize = (0..qr.size()).map(|y| (0..qr.size()).filter(|&x| qr.is_dark(x, y) && (x == 0 || !qr.is_dark(x - 1, y))).count()).sum();
        assert_eq!(path.subpaths.len(), runs);
        let first = &path.subpaths[0];
        assert!(first.closed);
        assert_eq!(first.anchors[0].p, Point::new(10.0, 20.0));
    }
}
