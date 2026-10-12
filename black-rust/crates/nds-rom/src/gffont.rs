//! Game Freak NFTR fonts (the dialogue/menu fonts, ARC 023 = `files/a/0/2/3`).
//!
//! Members 0..=4 of that NARC are `RTFN` (NFTR 1.1, little endian) files:
//! `0` 12x15 (5064 glyphs), `1` 10x10, `2` 11x13, `3` 13x8 number font (32 glyphs),
//! `4` 12x12 symbol font (41 glyphs). Members 5..=7 are palettes (`RLCN`).
//!
//! Layout used here: `FINF` (line feed, default widths, encoding `1` = UTF-16, offsets of
//! `CGLP`/`CWDH`/`CMAP`), `CGLP` (cell w/h, cell size, baseline, max width, bpp, then glyphs),
//! `CMAP` chain (types 0 direct, 1 table, 2 scan). Section magics are stored byte-reversed.
//! Each `CGLP` glyph is `3` header bytes `(left, glyph width, char width)` followed by
//! `cell_w * cell_h` pixels of `bpp` bits, row-major, MSB first. The `CWDH` section in this ROM
//! has a non-standard body (not a width table) and is ignored; the per-glyph header carries
//! the widths (UNVERIFIED: header byte order `(left, glyph, char)` follows the NFTR convention
//! and agrees with the ink extents). Characters are keyed by the UTF-16 unit used by
//! [`crate::msg`].

use crate::error::{le16, le32, slice, Error, Result};
use crate::render::Image;
use std::collections::BTreeMap;

fn bad(detail: impl Into<String>) -> Error {
    Error::invalid("NFTR font", detail)
}

/// One glyph: metrics and `bpp`-bit levels, `0` = background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    /// Left bearing in pixels.
    pub left: u8,
    /// Width of the inked part.
    pub glyph_width: u8,
    /// Cursor advance.
    pub char_width: u8,
    /// `cell_w * cell_h` levels, row-major.
    pub levels: Vec<u8>,
}

/// A parsed NFTR font.
#[derive(Debug, Clone)]
pub struct GfFont {
    pub cell_w: usize,
    pub cell_h: usize,
    pub baseline: u8,
    pub max_width: u8,
    pub bpp: u8,
    pub line_feed: u8,
    /// FINF encoding id (1 = UTF-16).
    pub encoding: u8,
    glyphs: Vec<Glyph>,
    map: BTreeMap<u16, u16>,
}

impl GfFont {
    pub fn parse(d: &[u8]) -> Result<GfFont> {
        if slice(d, 0, 4, "NFTR header")? != b"RTFN" {
            return Err(bad("bad magic"));
        }
        if slice(d, 0x10, 4, "FINF")? != b"FNIF" {
            return Err(bad("no FINF"));
        }
        let line_feed = *d.get(0x19).ok_or_else(|| bad("truncated FINF"))?;
        let encoding = *d.get(0x1f).ok_or_else(|| bad("truncated FINF"))?;
        let sec = |off: u32| {
            (off as usize)
                .checked_sub(8)
                .ok_or_else(|| bad("bad offset"))
        };
        let cglp = sec(le32(d, 0x20, "FINF")?)?;
        let cmap = sec(le32(d, 0x28, "FINF")?)?;
        if slice(d, cglp, 4, "CGLP")? != b"PLGC" {
            return Err(bad("no CGLP"));
        }
        let size = le32(d, cglp + 4, "CGLP")? as usize;
        let h = slice(d, cglp + 8, 8, "CGLP")?;
        let (cell_w, cell_h) = (h[0] as usize, h[1] as usize);
        let cell = u16::from_le_bytes([h[2], h[3]]) as usize;
        let (baseline, max_width, bpp) = (h[4], h[5], h[6]);
        let px = cell_w * cell_h * bpp as usize;
        if cell == 0 || !matches!(bpp, 1 | 2 | 4 | 8) || 3 + px.div_ceil(8) > cell {
            return Err(bad(format!("cell {cell_w}x{cell_h} size {cell} bpp {bpp}")));
        }
        let body = slice(d, cglp + 16, size.saturating_sub(16), "CGLP glyphs")?;
        let glyphs = body
            .chunks_exact(cell)
            .map(|g| {
                let mut bit = 0;
                let levels = (0..cell_w * cell_h)
                    .map(|_| {
                        let mut v = 0u8;
                        for _ in 0..bpp {
                            v = v << 1 | (g[3 + bit / 8] >> (7 - bit % 8)) & 1;
                            bit += 1;
                        }
                        v
                    })
                    .collect();
                Glyph {
                    left: g[0],
                    glyph_width: g[1],
                    char_width: g[2],
                    levels,
                }
            })
            .collect::<Vec<_>>();
        let mut map = BTreeMap::new();
        let mut p = cmap;
        for _ in 0..64 {
            if slice(d, p, 4, "CMAP")? != b"PAMC" {
                return Err(bad("no CMAP"));
            }
            let (first, last) = (le16(d, p + 8, "CMAP")?, le16(d, p + 10, "CMAP")?);
            let (ty, next) = (le16(d, p + 12, "CMAP")?, le32(d, p + 16, "CMAP")?);
            let at = p + 20;
            match ty {
                0 => {
                    let start = le16(d, at, "CMAP")?;
                    for c in first..=last {
                        map.insert(c, start + (c - first));
                    }
                }
                1 => {
                    for c in first..=last {
                        let i = le16(d, at + 2 * (c - first) as usize, "CMAP")?;
                        if i != 0xFFFF {
                            map.insert(c, i);
                        }
                    }
                }
                2 => {
                    for k in 0..le16(d, at, "CMAP")? as usize {
                        map.insert(
                            le16(d, at + 2 + 4 * k, "CMAP")?,
                            le16(d, at + 4 + 4 * k, "CMAP")?,
                        );
                    }
                }
                _ => return Err(bad(format!("CMAP type {ty}"))),
            }
            if next == 0 || next as usize > d.len() {
                break;
            }
            p = sec(next)?;
        }
        if let Some(i) = map.values().find(|&&i| i as usize >= glyphs.len()) {
            return Err(bad(format!("CMAP index {i} >= {} glyphs", glyphs.len())));
        }
        Ok(GfFont {
            cell_w,
            cell_h,
            baseline,
            max_width,
            bpp,
            line_feed,
            encoding,
            glyphs,
            map,
        })
    }

    /// Number of stored glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Number of mapped character codes.
    pub fn code_count(&self) -> usize {
        self.map.len()
    }

    /// Glyph by storage index.
    pub fn glyph_at(&self, index: usize) -> Option<&Glyph> {
        self.glyphs.get(index)
    }

    /// Storage index of a UTF-16 character code.
    pub fn glyph_index(&self, code: u16) -> Option<usize> {
        self.map.get(&code).map(|&i| i as usize)
    }

    /// Glyph for a UTF-16 character code.
    pub fn glyph(&self, code: u16) -> Option<&Glyph> {
        self.glyph_at(self.glyph_index(code)?)
    }

    /// Cursor advance of a character code.
    pub fn char_width(&self, code: u16) -> Option<u8> {
        self.glyph(code).map(|g| g.char_width)
    }

    /// Summed advance of a string; unmapped characters count as 0.
    pub fn text_width(&self, s: &str) -> usize {
        s.encode_utf16()
            .map(|c| self.char_width(c).unwrap_or(0) as usize)
            .sum()
    }

    /// Glyph atlas by storage index, `columns` per row, black with alpha = level.
    pub fn atlas(&self, columns: usize) -> Image {
        let columns = columns.max(1);
        let rows = self.len().div_ceil(columns);
        let mut img = Image::new(columns * self.cell_w, rows * self.cell_h);
        let top = (1u32 << self.bpp) - 1;
        for (n, g) in self.glyphs.iter().enumerate() {
            let (ox, oy) = ((n % columns) * self.cell_w, (n / columns) * self.cell_h);
            for (i, &v) in g.levels.iter().enumerate() {
                if v != 0 {
                    let a = (v as u32 * 255 / top) as u8;
                    let o = ((oy + i / self.cell_w) * img.width + ox + i % self.cell_w) * 4;
                    img.rgba[o..o + 4].copy_from_slice(&[0, 0, 0, a]);
                }
            }
        }
        img
    }
}

/// Parse every `RTFN` member of a NARC (member index, font).
pub fn fonts_from_narc(narc: &crate::narc::Narc) -> Vec<(usize, GfFont)> {
    (0..narc.len())
        .filter(|&i| narc.file(i).starts_with(b"RTFN"))
        .filter_map(|i| GfFont::parse(narc.file(i)).ok().map(|f| (i, f)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Vec<u8> {
        let mut d = vec![0u8; 0x2c];
        d[..4].copy_from_slice(b"RTFN");
        d[0x10..0x14].copy_from_slice(b"FNIF");
        d[0x19] = 4;
        d[0x1f] = 1;
        d[0x20..0x24].copy_from_slice(&0x34u32.to_le_bytes());
        // CGLP: 2 glyphs of 4x2, 2bpp -> 3 + 2 bytes
        d.extend_from_slice(b"PLGC");
        d.extend_from_slice(&(16u32 + 10).to_le_bytes());
        d.extend_from_slice(&[4, 2, 5, 0, 2, 4, 2, 0]);
        d.extend_from_slice(&[0, 4, 5, 0b1110_0100, 0b0000_0001]);
        d.extend_from_slice(&[1, 2, 3, 0, 0]);
        let cmap = d.len();
        d.extend_from_slice(b"PAMC");
        d.extend_from_slice(&24u32.to_le_bytes());
        d.extend_from_slice(&[0x41, 0, 0x42, 0, 1, 0, 0, 0]);
        d.extend_from_slice(&0u32.to_le_bytes());
        d.extend_from_slice(&[0, 0, 1, 0]);
        d[0x28..0x2c].copy_from_slice(&((cmap + 8) as u32).to_le_bytes());
        d
    }

    #[test]
    fn synthetic() {
        let f = GfFont::parse(&tiny()).unwrap();
        assert_eq!((f.len(), f.cell_w, f.cell_h, f.bpp), (2, 4, 2, 2));
        let a = f.glyph(0x41).unwrap();
        assert_eq!(a.levels, [3, 2, 1, 0, 0, 0, 0, 1]);
        assert_eq!(f.char_width(0x42), Some(3));
        assert_eq!(f.text_width("AB?"), 8);
        assert_eq!(f.atlas(2).width, 8);
    }

    #[test]
    fn rejects_garbage() {
        assert!(GfFont::parse(b"NOPE").is_err());
        assert!(GfFont::parse(&tiny()[..0x30]).is_err());
    }
}
