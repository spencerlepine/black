//! Map matrices (ARC 009): a grid of map-container ids (ARC 008 members).
//!
//! Layout: `flags:u32 width:u16 height:u16` then `width*height` cells in row-major order. With
//! `flags == 0` a cell is one `u32`; with `flags == 1` (2 of 255 matrices, incl. member 0, the
//! 29x27 overworld) a cell is two `u32`s `(primary, secondary)`. `0xFFFF_FFFF` = no map.

use super::Rd;
use crate::error::{Error, Result};

pub const NO_MAP: u32 = 0xFFFF_FFFF;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapMatrix {
    /// 0 = single layer, 1 = two layers. Other values are rejected.
    pub flags: u32,
    pub width: u16,
    pub height: u16,
    /// Map container ids, `width * height`, row-major.
    pub cells: Vec<u32>,
    /// Second `u32` of each cell when `flags == 1` (role UNVERIFIED), same length as `cells`.
    pub cells_b: Option<Vec<u32>>,
}

impl MapMatrix {
    pub fn parse(d: &[u8]) -> Result<MapMatrix> {
        let mut r = Rd::new(d, "map matrix");
        let flags = r.u32()?;
        let width = r.u16()?;
        let height = r.u16()?;
        if flags > 1 {
            return Err(Error::invalid("map matrix", format!("flags {flags:#x}")));
        }
        let n = width as usize * height as usize;
        let mut cells = Vec::with_capacity(n);
        let mut cells_b = (flags == 1).then(|| Vec::with_capacity(n));
        for _ in 0..n {
            cells.push(r.u32()?);
            if let Some(b) = cells_b.as_mut() {
                b.push(r.u32()?);
            }
        }
        if !r.rest().is_empty() {
            return Err(Error::invalid("map matrix", "trailing bytes"));
        }
        Ok(MapMatrix {
            flags,
            width,
            height,
            cells,
            cells_b,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend(self.flags.to_le_bytes());
        o.extend(self.width.to_le_bytes());
        o.extend(self.height.to_le_bytes());
        for (i, c) in self.cells.iter().enumerate() {
            o.extend(c.to_le_bytes());
            if let Some(b) = &self.cells_b {
                o.extend(b[i].to_le_bytes());
            }
        }
        o
    }

    /// Container id at column `x`, row `y`, `None` when empty or out of range.
    pub fn map_at(&self, x: usize, y: usize) -> Option<u32> {
        if x >= self.width as usize || y >= self.height as usize {
            return None;
        }
        Some(self.cells[y * self.width as usize + x]).filter(|&v| v != NO_MAP)
    }
}
