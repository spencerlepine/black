//! `gfl_font.dat` reader: a small fixed-cell 1bpp bitmap font.
//!
//! Layout (derived from the file; no loader code was found in the decompilation):
//! `u32 data_offset (0x10)`, `u32 0` (unknown), `u32 glyph_count (0x120)`, `u8 width`,
//! `u8 height`, `u8` (1, probably bits per pixel), `u8` (1, unknown), then `glyph_count`
//! glyphs of `ceil(width / 8) * height` bytes each, rows top to bottom, MSB = leftmost pixel.
//! The glyphs are an 8x8 kana/symbol set; the mapping from glyph index to character code is
//! not recorded in the file (UNVERIFIED), so glyphs are keyed by index.

use crate::error::{Error, Result};
use crate::render::Image;

/// A parsed bitmap font.
#[derive(Debug, Clone)]
pub struct Font {
    pub width: usize,
    pub height: usize,
    /// Per-glyph row bytes, `row_bytes * height` each.
    glyphs: Vec<Vec<u8>>,
    row_bytes: usize,
}

impl Font {
    pub fn parse(data: &[u8]) -> Result<Font> {
        if data.len() < 16 {
            return Err(Error::Truncated {
                what: "gfl_font.dat",
                offset: 0,
                need: 16,
                have: data.len(),
            });
        }
        let u32_at = |o: usize| u32::from_le_bytes(data[o..o + 4].try_into().unwrap()) as usize;
        let (off, count) = (u32_at(0), u32_at(8));
        let (width, height) = (data[12] as usize, data[13] as usize);
        if width == 0 || height == 0 || data[14] != 1 || off < 16 {
            return Err(Error::invalid(
                "gfl_font.dat",
                format!("header off {off:#x} cell {width}x{height} bpp {}", data[14]),
            ));
        }
        let row_bytes = width.div_ceil(8);
        let size = row_bytes * height;
        let need = count
            .checked_mul(size)
            .and_then(|n| n.checked_add(off))
            .ok_or_else(|| Error::invalid("gfl_font.dat", "glyph count overflow"))?;
        if data.len() < need {
            return Err(Error::Truncated {
                what: "gfl_font.dat glyphs",
                offset: off,
                need: need - off,
                have: data.len().saturating_sub(off),
            });
        }
        let glyphs = (0..count)
            .map(|i| data[off + i * size..off + (i + 1) * size].to_vec())
            .collect();
        Ok(Font {
            width,
            height,
            glyphs,
            row_bytes,
        })
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Is pixel (x, y) of glyph `g` set?
    pub fn pixel(&self, g: usize, x: usize, y: usize) -> bool {
        self.glyphs.get(g).is_some_and(|b| {
            x < self.width
                && y < self.height
                && (b[y * self.row_bytes + x / 8] >> (7 - x % 8)) & 1 != 0
        })
    }

    /// Advance width of glyph `g`: columns up to the rightmost set pixel plus one pixel of
    /// spacing, or half the cell for an empty (space) glyph. Derived, not stored in the file.
    pub fn glyph_width(&self, g: usize) -> Option<usize> {
        self.glyphs.get(g)?;
        let right = (0..self.width)
            .rev()
            .find(|&x| (0..self.height).any(|y| self.pixel(g, x, y)));
        Some(right.map_or(self.width / 2, |x| (x + 2).min(self.width)))
    }

    /// Glyph atlas: `columns` glyphs per row, white on transparent.
    pub fn atlas(&self, columns: usize) -> Image {
        let columns = columns.max(1);
        let rows = self.len().div_ceil(columns);
        let mut img = Image::new(columns * self.width, rows * self.height);
        for g in 0..self.len() {
            let (ox, oy) = ((g % columns) * self.width, (g / columns) * self.height);
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.pixel(g, x, y) {
                        let i = ((oy + y) * img.width + ox + x) * 4;
                        img.rgba[i..i + 4].copy_from_slice(&[255; 4]);
                    }
                }
            }
        }
        img
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_font() {
        let mut d = vec![0x10, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 8, 8, 1, 1];
        d.extend([0u8; 8]);
        d.extend([0x80, 0x80, 0x80, 0x80, 0xE0, 0, 0, 0]);
        let f = Font::parse(&d).unwrap();
        assert_eq!(f.glyph_width(0), Some(4));
        assert_eq!(f.glyph_width(1), Some(4));
        assert!(f.pixel(1, 2, 4) && !f.pixel(1, 3, 4));
        assert!(Font::parse(&d[..20]).is_err());
    }
}
