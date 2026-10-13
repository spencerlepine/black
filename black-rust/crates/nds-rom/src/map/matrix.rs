//! Map matrices (ARC 009): a grid of map-container ids (ARC 008 members).
//!
//! Layout: `flags:u32 width:u16 height:u16`, then `width*height` map-container ids (`u32`, row-major);
//! with `flags == 1` (2 of 255 matrices: member 0, the 29x27 overworld, and member 175) a second plane of
//! `width*height` zone ids follows (`u32` each; the game keeps the low `u16`). `0xFFFF_FFFF` = no map.
//! This is how the game reads the file (`FUN_02159D04` in overlay 10, F2a `black_field::map_matrix::load_from`):
//! the land plane is copied whole, then the zone plane; matrices without one give every block the zone's
//! default zone id.

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
    /// The zone-id plane when `flags == 1` (zone id of each block, same order and length as `cells`).
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
        let mut plane = || (0..n).map(|_| r.u32()).collect::<Result<Vec<u32>>>();
        let cells = plane()?;
        let cells_b = if flags == 1 { Some(plane()?) } else { None };
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
        for c in self.cells.iter().chain(self.cells_b.iter().flatten()) {
            o.extend(c.to_le_bytes());
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

    /// Zone id of column `x`, row `y` from the zone plane (`flags == 1` only), `None` out of range.
    pub fn zone_at(&self, x: usize, y: usize) -> Option<u16> {
        if x >= self.width as usize || y >= self.height as usize {
            return None;
        }
        Some(self.cells_b.as_ref()?[y * self.width as usize + x] as u16)
    }
}
