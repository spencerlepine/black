//! Field map data readers (A6): zone headers, map matrices, map containers and zone events.
//!
//! ROM archives (ARC ID = decimal `XYZ` of `files/a/X/Y/Z`), see MIGRATION-AGENT.md §7 "A6":
//! - ARC 012: one member holding all zone headers, 48 bytes each ([`zone::ZoneHeader`]).
//! - ARC 009: map matrices, one member per matrix ([`matrix::MapMatrix`]).
//! - ARC 008: map containers, one member per map id ([`container::MapContainer`]).
//! - ARC 125: zone events, one member per zone id ([`events::ZoneEvents`]).
//!
//! All members are stored uncompressed. Unknown fields stay raw and are named `unk_XX` (hex byte
//! offset in the record).

pub mod container;
pub mod events;
pub mod matrix;
pub mod zone;

use crate::error::{Error, Result};

pub use container::MapContainer;
pub use events::ZoneEvents;
pub use matrix::MapMatrix;
pub use zone::ZoneHeader;

/// ARC IDs of the map archives.
pub const ARC_MAP_CONTAINER: u32 = 8;
pub const ARC_MAP_MATRIX: u32 = 9;
pub const ARC_ZONE_HEADERS: u32 = 12;
pub const ARC_ZONE_EVENTS: u32 = 125;

/// Little-endian cursor over a byte slice.
pub(crate) struct Rd<'a> {
    d: &'a [u8],
    pub pos: usize,
    what: &'static str,
}

impl<'a> Rd<'a> {
    pub fn new(d: &'a [u8], what: &'static str) -> Self {
        Rd { d, pos: 0, what }
    }
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        let s = self
            .d
            .get(self.pos..self.pos.saturating_add(n))
            .ok_or(Error::Truncated {
                what: self.what,
                offset: self.pos,
                need: n,
                have: self.d.len(),
            })?;
        self.pos += n;
        Ok(s)
    }
    pub fn arr<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut a = [0u8; N];
        a.copy_from_slice(self.bytes(N)?);
        Ok(a)
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.arr()?))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.arr()?))
    }
    pub fn rest(&mut self) -> &'a [u8] {
        let s = &self.d[self.pos.min(self.d.len())..];
        self.pos = self.d.len();
        s
    }
}
