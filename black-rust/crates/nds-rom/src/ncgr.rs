//! NCGR character (tile) files (`RGCN` / `RAHC`).

use crate::error::{Error, Result};
use crate::nitro::{payload_off, u16_at, u32_at, Container};

/// Parsed tile graphics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ncgr {
    /// Dimensions in tiles; `None` when the file stores 0xFFFF (unknown, use a free layout).
    pub tiles_x: Option<u16>,
    pub tiles_y: Option<u16>,
    /// Bits per pixel: 4 or 8.
    pub bpp: u8,
    /// True for "scanned" (linear, row-major pixels) data; false for 8x8-tiled data.
    pub scanned: bool,
    /// Raw pixel data as stored.
    pub data: Vec<u8>,
}

fn nibble(data: &[u8], i: usize) -> u8 {
    let b = data[i / 2];
    if i.is_multiple_of(2) {
        b & 0xF
    } else {
        b >> 4
    }
}

impl Ncgr {
    pub fn parse(data: &[u8]) -> Result<Ncgr> {
        let c = Container::parse(data, b"RGCN", "NCGR")?;
        let b = c.block(b"RAHC", "NCGR")?;
        let ty = u16_at(b, 0)?;
        let tx = u16_at(b, 2)?;
        let bpp = match u32_at(b, 4)? {
            3 => 4,
            4 => 8,
            d => return Err(Error::invalid("NCGR", format!("unknown depth code {d}"))),
        };
        let scanned = u32_at(b, 12)? & 0xFF != 0;
        let size = u32_at(b, 16)? as usize;
        let start = payload_off(u32_at(b, 20)?, "NCGR")?;
        let end = start.saturating_add(size).min(b.len());
        let raw = b
            .get(start..end)
            .ok_or_else(|| Error::invalid("NCGR", "pixel data out of range"))?;
        let dim = |v: u16| (v != 0xFFFF).then_some(v);
        Ok(Ncgr {
            tiles_x: dim(tx),
            tiles_y: dim(ty),
            bpp,
            scanned,
            data: raw.to_vec(),
        })
    }

    /// Bytes per 8x8 tile.
    pub fn tile_bytes(&self) -> usize {
        8 * self.bpp as usize
    }

    /// Number of whole tiles stored.
    pub fn tile_count(&self) -> usize {
        self.data.len() / self.tile_bytes()
    }

    /// Tiles per row of the stored image (32 when the file leaves it unknown).
    pub fn row_tiles(&self) -> usize {
        self.tiles_x.map_or(32, usize::from).max(1)
    }

    /// Palette indices (64, row-major) of tile `n`. For tiled files this is the n-th stored
    /// tile; for scanned files it is the 8x8 block at tile position `(n % row, n / row)` of the
    /// linear bitmap.
    pub fn tile_pixels(&self, n: usize) -> Option<[u8; 64]> {
        if self.scanned {
            let row = self.row_tiles();
            let (bx, by) = ((n % row) * 8, (n / row) * 8);
            let w = row * 8;
            let mut out = [0u8; 64];
            for y in 0..8 {
                for x in 0..8 {
                    let i = (by + y) * w + bx + x;
                    if i / (8 / self.bpp as usize) >= self.data.len() {
                        return None;
                    }
                    out[y * 8 + x] = if self.bpp == 8 {
                        self.data[i]
                    } else {
                        nibble(&self.data, i)
                    };
                }
            }
            return Some(out);
        }
        let tb = self.tile_bytes();
        let t = self.data.get(n * tb..(n + 1) * tb)?;
        let mut out = [0u8; 64];
        for (i, p) in out.iter_mut().enumerate() {
            *p = if self.bpp == 8 { t[i] } else { nibble(t, i) };
        }
        Some(out)
    }

    /// Whole-image palette indices as `(width, height, pixels)`. Tiled data is laid out
    /// `tiles_x` tiles wide (up to 32 if the file leaves it unknown); scanned data is read
    /// linearly with width `tiles_x * 8`.
    pub fn to_indices(&self) -> (usize, usize, Vec<u8>) {
        let n = self.tile_count();
        let tx = self.tiles_x.map_or(0, usize::from);
        let tx = if tx == 0 { n.clamp(1, 32) } else { tx };
        let ty = n.div_ceil(tx);
        let (w, h) = (tx * 8, ty * 8);
        let mut img = vec![0u8; w * h];
        if self.scanned {
            let per_byte = 8 / self.bpp as usize;
            for (i, px) in img.iter_mut().enumerate().take(self.data.len() * per_byte) {
                *px = if self.bpp == 8 {
                    self.data[i]
                } else {
                    nibble(&self.data, i)
                };
            }
        } else {
            for t in 0..n {
                let Some(px) = self.tile_pixels(t) else { break };
                let (bx, by) = ((t % tx) * 8, (t / tx) * 8);
                for y in 0..8 {
                    let at = (by + y) * w + bx;
                    img[at..at + 8].copy_from_slice(&px[y * 8..y * 8 + 8]);
                }
            }
        }
        (w, h, img)
    }
}
