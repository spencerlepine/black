//! Zone headers (ARC 012, member 0): 427 records of 48 bytes.
//!
//! No loader code is symbolised in the field overlays (the 0x4C-byte `FUN_020139B0` allocation is
//! the unrelated runtime zone state), so field meanings come from data: self-index, value ranges
//! against the target archives and text banks. Everything not listed as confirmed stays `unk_XX`.

use super::Rd;
use crate::error::{Error, Result};

pub const ZONE_HEADER_SIZE: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneHeader {
    pub unk_00: u8,
    pub unk_01: u8,
    /// Range 0..=273, 104 distinct values.
    pub unk_02: u16,
    /// Index into ARC 009 (map matrices). Confirmed: always < 255, dimensions plausible.
    pub matrix_id: u16,
    /// Unique per zone, < 899 (ARC 057 field scripts?). UNVERIFIED.
    pub unk_06: u16,
    /// Unique per zone, < 899. UNVERIFIED.
    pub unk_08: u16,
    /// Unique per zone, < 472 (ARC 003 story text banks?). UNVERIFIED.
    pub unk_0a: u16,
    /// BGM sequence ids (SDAT SEQ 1000..=1178), probably per season. UNVERIFIED order.
    pub bgm: [u16; 4],
    /// 0xFFFF for most zones. UNVERIFIED (weather?).
    pub unk_14: u16,
    /// Self-referencing zone id. Confirmed: equals the record index for all 427.
    pub zone_id: u16,
    pub unk_18: u16,
    /// Place name index into text bank 89 (ARC 002). Confirmed: Nuvema Town = 4.
    pub name_id: u8,
    /// High byte of the u16 at 0x1A (small flag values, 0..=4).
    pub unk_1b: u8,
    pub unk_1c: u16,
    pub unk_1e: u16,
    /// Equals `matrix_id` in most zones.
    pub unk_20: u16,
    pub unk_22: u16,
    pub unk_24: u16,
    pub unk_26: u16,
    pub unk_28: u16,
    pub unk_2a: u16,
    pub unk_2c: u16,
    pub unk_2e: u16,
}

impl ZoneHeader {
    pub fn parse(d: &[u8]) -> Result<ZoneHeader> {
        let mut r = Rd::new(d, "zone header");
        let h = ZoneHeader {
            unk_00: r.u8()?,
            unk_01: r.u8()?,
            unk_02: r.u16()?,
            matrix_id: r.u16()?,
            unk_06: r.u16()?,
            unk_08: r.u16()?,
            unk_0a: r.u16()?,
            bgm: [r.u16()?, r.u16()?, r.u16()?, r.u16()?],
            unk_14: r.u16()?,
            zone_id: r.u16()?,
            unk_18: r.u16()?,
            name_id: r.u8()?,
            unk_1b: r.u8()?,
            unk_1c: r.u16()?,
            unk_1e: r.u16()?,
            unk_20: r.u16()?,
            unk_22: r.u16()?,
            unk_24: r.u16()?,
            unk_26: r.u16()?,
            unk_28: r.u16()?,
            unk_2a: r.u16()?,
            unk_2c: r.u16()?,
            unk_2e: r.u16()?,
        };
        Ok(h)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(ZONE_HEADER_SIZE);
        o.extend([self.unk_00, self.unk_01]);
        for v in [
            self.unk_02,
            self.matrix_id,
            self.unk_06,
            self.unk_08,
            self.unk_0a,
        ] {
            o.extend(v.to_le_bytes());
        }
        for v in self.bgm {
            o.extend(v.to_le_bytes());
        }
        for v in [self.unk_14, self.zone_id, self.unk_18] {
            o.extend(v.to_le_bytes());
        }
        o.extend([self.name_id, self.unk_1b]);
        for v in [
            self.unk_1c,
            self.unk_1e,
            self.unk_20,
            self.unk_22,
            self.unk_24,
            self.unk_26,
            self.unk_28,
            self.unk_2a,
            self.unk_2c,
            self.unk_2e,
        ] {
            o.extend(v.to_le_bytes());
        }
        o
    }

    /// Parse the whole ARC 012 member 0 (a multiple of 48 bytes).
    pub fn parse_table(d: &[u8]) -> Result<Vec<ZoneHeader>> {
        if !d.len().is_multiple_of(ZONE_HEADER_SIZE) {
            return Err(Error::invalid(
                "zone header table",
                format!("{:#x} bytes is not a multiple of 48", d.len()),
            ));
        }
        d.chunks(ZONE_HEADER_SIZE).map(ZoneHeader::parse).collect()
    }
}
