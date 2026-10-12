//! NCER cell banks (`RECN` / `KBEC`): sprite cells made of OAM entries.

use crate::error::{Error, Result};
use crate::nitro::{payload_off, u16_at, u32_at, Container};

/// One OAM (object) entry of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Oam {
    pub x: i16,
    pub y: i16,
    /// Width and height in pixels.
    pub w: u8,
    pub h: u8,
    pub tile: u16,
    pub palette: u8,
    pub hflip: bool,
    pub vflip: bool,
    pub bpp8: bool,
    pub priority: u8,
    /// Rotation/scale flag; such objects are drawn unrotated by the renderer.
    pub rotscale: bool,
    /// Double-size rotation/scale box: the (x, y) corner is that of a box twice `w` x `h`.
    pub double_size: bool,
    /// Hidden object (only meaningful without rotation/scale).
    pub disabled: bool,
}

/// A cell: its objects in draw order (first entry is on top on hardware).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub attr: u16,
    /// Bounding box (max_x, max_y, min_x, min_y) when the bank stores it.
    pub bounds: Option<[i16; 4]>,
    pub oams: Vec<Oam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ncer {
    /// VRAM mapping mode: 1D tile indices are multiplied by `1 << mapping_mode`
    /// (32/64/128/256 byte boundaries).
    pub mapping_mode: u32,
    pub cells: Vec<Cell>,
}

/// Object dimensions in pixels from (shape, size) attribute bits.
pub fn oam_size(shape: u16, size: u16) -> Option<(u8, u8)> {
    Some(match (shape, size) {
        (0, 0) => (8, 8),
        (0, 1) => (16, 16),
        (0, 2) => (32, 32),
        (0, 3) => (64, 64),
        (1, 0) => (16, 8),
        (1, 1) => (32, 8),
        (1, 2) => (32, 16),
        (1, 3) => (64, 32),
        (2, 0) => (8, 16),
        (2, 1) => (8, 32),
        (2, 2) => (16, 32),
        (2, 3) => (32, 64),
        _ => return None,
    })
}

impl Ncer {
    pub fn parse(data: &[u8]) -> Result<Ncer> {
        let c = Container::parse(data, b"RECN", "NCER")?;
        let b = c.block(b"KBEC", "NCER")?;
        let n = u16_at(b, 0)? as usize;
        let ext = u16_at(b, 2)? & 0xFF == 1;
        let table = payload_off(u32_at(b, 4)?, "NCER")?;
        let mapping_mode = u32_at(b, 8)?;
        let cell_size = if ext { 16 } else { 8 };
        let oam_base = table + n * cell_size;
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            let at = table + i * cell_size;
            let count = u16_at(b, at)? as usize;
            let attr = u16_at(b, at + 2)?;
            let off = u32_at(b, at + 4)? as usize;
            let bounds = if ext {
                let g = |k: usize| u16_at(b, at + 8 + k * 2).map(|v| v as i16);
                Some([g(0)?, g(1)?, g(2)?, g(3)?])
            } else {
                None
            };
            let mut oams = Vec::with_capacity(count);
            for j in 0..count {
                let o = oam_base + off + j * 6;
                let (a0, a1, a2) = (u16_at(b, o)?, u16_at(b, o + 2)?, u16_at(b, o + 4)?);
                let (w, h) = oam_size(a0 >> 14, a1 >> 14)
                    .ok_or_else(|| Error::invalid("NCER", "prohibited OAM shape"))?;
                let rotscale = a0 & 0x100 != 0;
                oams.push(Oam {
                    y: a0 as u8 as i8 as i16,
                    x: (((a1 & 0x1FF) << 7) as i16) >> 7,
                    w,
                    h,
                    tile: a2 & 0x3FF,
                    palette: (a2 >> 12) as u8,
                    hflip: !rotscale && a1 & 0x1000 != 0,
                    vflip: !rotscale && a1 & 0x2000 != 0,
                    bpp8: a0 & 0x2000 != 0,
                    priority: ((a2 >> 10) & 3) as u8,
                    rotscale,
                    double_size: rotscale && a0 & 0x200 != 0,
                    disabled: !rotscale && a0 & 0x200 != 0,
                });
            }
            cells.push(Cell { attr, bounds, oams });
        }
        Ok(Ncer {
            mapping_mode,
            cells,
        })
    }
}
