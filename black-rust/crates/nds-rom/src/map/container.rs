//! Per-map containers (ARC 008): `magic[2] nsections:u16 offsets:u32[nsections+1]` then the
//! sections back to back (the last offset is the file size).
//!
//! Magics in the ROM: `WB` (572 files, 3 sections), `GC` (47, 4 sections), `NG` (22, 2 sections),
//! `RD` (8, 3 sections). Only `WB` is decoded into typed parts; the others expose raw sections.
//!
//! `WB` sections: 0 = BMD0 terrain model, 1 = permission grid, 2 = building placements.

use super::Rd;
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapContainer {
    pub magic: [u8; 2],
    pub sections: Vec<Vec<u8>>,
}

/// One 8-byte permission cell: four raw `u16`s (meaning UNVERIFIED; collision/height/attributes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionCell {
    pub unk_00: u16,
    pub unk_02: u16,
    pub unk_04: u16,
    pub unk_06: u16,
}

/// Permission grid (`WB` section 1): `width:u16 height:u16` (32x32, or 64x64 in a few maps) then cells, then
/// `trailing` bytes (0, 32 or 208 in the ROM; raw).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionGrid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<PermissionCell>,
    pub trailing: Vec<u8>,
}

/// Building placement (`WB` section 2): `count:u32` then 16-byte records. UNVERIFIED layout: the
/// first three words look like fx32 x/y/z (y mostly 0), the last word carries a model id in its
/// high byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingPlacement {
    pub pos: [i32; 3],
    pub unk_0c: u32,
}

impl MapContainer {
    pub fn parse(d: &[u8]) -> Result<MapContainer> {
        let mut r = Rd::new(d, "map container");
        let magic = r.arr::<2>()?;
        let n = r.u16()? as usize;
        let offs = (0..=n).map(|_| r.u32()).collect::<Result<Vec<_>>>()?;
        if offs[0] as usize != r.pos || offs[n] as usize != d.len() {
            return Err(Error::invalid("map container", "section table mismatch"));
        }
        let mut sections = Vec::with_capacity(n);
        for w in offs.windows(2) {
            let s = d
                .get(w[0] as usize..w[1] as usize)
                .ok_or_else(|| Error::invalid("map container", "section out of range"))?;
            sections.push(s.to_vec());
        }
        Ok(MapContainer { magic, sections })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend(self.magic);
        o.extend((self.sections.len() as u16).to_le_bytes());
        let mut off = 4 + 4 * (self.sections.len() + 1);
        for s in &self.sections {
            o.extend((off as u32).to_le_bytes());
            off += s.len();
        }
        o.extend((off as u32).to_le_bytes());
        for s in &self.sections {
            o.extend(s);
        }
        o
    }

    pub fn is_wb(&self) -> bool {
        &self.magic == b"WB" && self.sections.len() == 3
    }

    /// The BMD0 terrain model (`WB` only).
    pub fn model(&self) -> Option<&[u8]> {
        self.is_wb().then(|| self.sections[0].as_slice())
    }

    pub fn permissions(&self) -> Result<PermissionGrid> {
        if !self.is_wb() {
            return Err(Error::invalid("map container", "not a WB container"));
        }
        PermissionGrid::parse(&self.sections[1])
    }

    pub fn buildings(&self) -> Result<Vec<BuildingPlacement>> {
        if !self.is_wb() {
            return Err(Error::invalid("map container", "not a WB container"));
        }
        let mut r = Rd::new(&self.sections[2], "building placements");
        let n = r.u32()? as usize;
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(BuildingPlacement {
                pos: [r.u32()? as i32, r.u32()? as i32, r.u32()? as i32],
                unk_0c: r.u32()?,
            });
        }
        if !r.rest().is_empty() {
            return Err(Error::invalid("building placements", "trailing bytes"));
        }
        Ok(v)
    }
}

impl PermissionGrid {
    pub fn parse(d: &[u8]) -> Result<PermissionGrid> {
        let mut r = Rd::new(d, "permission grid");
        let width = r.u16()?;
        let height = r.u16()?;
        let mut cells = Vec::with_capacity(width as usize * height as usize);
        for _ in 0..width as usize * height as usize {
            cells.push(PermissionCell {
                unk_00: r.u16()?,
                unk_02: r.u16()?,
                unk_04: r.u16()?,
                unk_06: r.u16()?,
            });
        }
        Ok(PermissionGrid {
            width,
            height,
            cells,
            trailing: r.rest().to_vec(),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend(self.width.to_le_bytes());
        o.extend(self.height.to_le_bytes());
        for c in &self.cells {
            for v in [c.unk_00, c.unk_02, c.unk_04, c.unk_06] {
                o.extend(v.to_le_bytes());
            }
        }
        o.extend(&self.trailing);
        o
    }
}
