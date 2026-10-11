//! NDS / DSi ROM header.

use crate::crc::crc16;
use crate::error::{le16, le32, Error, Result};

/// Size of the NTR header proper.
pub const NTR_HEADER_SIZE: usize = 0x200;
/// Offset of the logo CRC.
pub const LOGO_CRC_OFFSET: usize = 0x15C;
/// Offset of the header CRC, which covers `0..0x15E`.
pub const HEADER_CRC_OFFSET: usize = 0x15E;
/// The compressed Nintendo logo, covered by the logo CRC.
const LOGO_RANGE: std::ops::Range<usize> = 0xC0..0x15C;

/// A binary (ARM9/ARM7) entry of the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BinaryInfo {
    pub rom_offset: u32,
    pub entry: u32,
    pub ram_address: u32,
    pub size: u32,
}

/// An `(offset, size)` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Region {
    pub offset: u32,
    pub size: u32,
}

/// Parsed ROM header.  The original bytes are kept so that unknown fields (and the DSi
/// extension area) survive a parse/serialise round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub title: [u8; 12],
    pub game_code: [u8; 4],
    pub maker_code: [u8; 2],
    pub unit_code: u8,
    pub encryption_seed: u8,
    pub device_capacity: u8,
    pub rom_version: u8,
    pub auto_start: u8,
    pub arm9: BinaryInfo,
    pub arm7: BinaryInfo,
    pub fnt: Region,
    pub fat: Region,
    pub arm9_overlays: Region,
    pub arm7_overlays: Region,
    pub banner_offset: u32,
    /// Used ROM size (`0x80`), excluding the DSi area.
    pub ntr_rom_size: u32,
    pub header_size: u32,
    pub logo_crc: u16,
    pub header_crc: u16,
    /// Total ROM size including the DSi area (`0x210`), zero on plain NTR ROMs.
    pub twl_rom_size: u32,
    raw: Vec<u8>,
}

fn binary(data: &[u8], off: usize) -> Result<BinaryInfo> {
    Ok(BinaryInfo {
        rom_offset: le32(data, off, "header")?,
        entry: le32(data, off + 4, "header")?,
        ram_address: le32(data, off + 8, "header")?,
        size: le32(data, off + 12, "header")?,
    })
}

fn region(data: &[u8], off: usize) -> Result<Region> {
    Ok(Region {
        offset: le32(data, off, "header")?,
        size: le32(data, off + 4, "header")?,
    })
}

impl Header {
    /// Parse a header from the start of `data` (at least `0x200` bytes).
    pub fn parse(data: &[u8]) -> Result<Header> {
        if data.len() < NTR_HEADER_SIZE {
            return Err(Error::Truncated {
                what: "ROM header",
                offset: 0,
                need: NTR_HEADER_SIZE,
                have: data.len(),
            });
        }
        let mut title = [0; 12];
        title.copy_from_slice(&data[0..12]);
        let mut game_code = [0; 4];
        game_code.copy_from_slice(&data[12..16]);
        Ok(Header {
            title,
            game_code,
            maker_code: [data[16], data[17]],
            unit_code: data[18],
            encryption_seed: data[19],
            device_capacity: data[20],
            rom_version: data[0x1E],
            auto_start: data[0x1F],
            arm9: binary(data, 0x20)?,
            arm7: binary(data, 0x30)?,
            fnt: region(data, 0x40)?,
            fat: region(data, 0x48)?,
            arm9_overlays: region(data, 0x50)?,
            arm7_overlays: region(data, 0x58)?,
            banner_offset: le32(data, 0x68, "header")?,
            ntr_rom_size: le32(data, 0x80, "header")?,
            header_size: le32(data, 0x84, "header")?,
            logo_crc: le16(data, LOGO_CRC_OFFSET, "header")?,
            header_crc: le16(data, HEADER_CRC_OFFSET, "header")?,
            twl_rom_size: le32(data, 0x210, "header").unwrap_or(0),
            raw: data.to_vec(),
        })
    }

    /// The bytes this header was parsed from (without field edits).
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    /// Number of overlays in the ARM9 overlay table.
    pub fn arm9_overlay_count(&self) -> usize {
        self.arm9_overlays.size as usize / 32
    }

    /// Number of FAT entries (files, overlays included).
    pub fn file_count(&self) -> usize {
        self.fat.size as usize / 8
    }

    /// Game title as text, trailing NULs stripped.
    pub fn title_str(&self) -> String {
        String::from_utf8_lossy(&self.title)
            .trim_end_matches('\0')
            .to_string()
    }

    /// Game code as text.
    pub fn game_code_str(&self) -> String {
        String::from_utf8_lossy(&self.game_code).to_string()
    }

    /// Whether the header advertises DSi extensions.
    pub fn is_dsi(&self) -> bool {
        self.unit_code & 2 != 0
    }

    /// CRC computed over the header bytes `0..0x15E`.
    pub fn compute_header_crc(&self) -> u16 {
        crc16(&self.to_bytes_without_crc_update()[..HEADER_CRC_OFFSET])
    }

    /// CRC computed over the Nintendo logo.
    pub fn compute_logo_crc(&self) -> u16 {
        crc16(&self.raw[LOGO_RANGE])
    }

    /// Whether the stored header CRC matches the header contents.
    pub fn header_crc_valid(&self) -> bool {
        self.compute_header_crc() == self.header_crc
    }

    /// Whether the stored logo CRC matches the logo.
    pub fn logo_crc_valid(&self) -> bool {
        self.compute_logo_crc() == self.logo_crc
    }

    /// Recompute the logo and header CRC fields.
    pub fn update_crcs(&mut self) {
        self.logo_crc = self.compute_logo_crc();
        self.header_crc = self.compute_header_crc();
    }

    fn to_bytes_without_crc_update(&self) -> Vec<u8> {
        let mut out = self.raw.clone();
        out[0..12].copy_from_slice(&self.title);
        out[12..16].copy_from_slice(&self.game_code);
        out[16..18].copy_from_slice(&self.maker_code);
        out[18] = self.unit_code;
        out[19] = self.encryption_seed;
        out[20] = self.device_capacity;
        out[0x1E] = self.rom_version;
        out[0x1F] = self.auto_start;
        let put = |out: &mut Vec<u8>, off: usize, vals: &[u32]| {
            for (i, v) in vals.iter().enumerate() {
                out[off + i * 4..off + i * 4 + 4].copy_from_slice(&v.to_le_bytes());
            }
        };
        for (off, b) in [(0x20, self.arm9), (0x30, self.arm7)] {
            put(
                &mut out,
                off,
                &[b.rom_offset, b.entry, b.ram_address, b.size],
            );
        }
        for (off, r) in [
            (0x40, self.fnt),
            (0x48, self.fat),
            (0x50, self.arm9_overlays),
            (0x58, self.arm7_overlays),
        ] {
            put(&mut out, off, &[r.offset, r.size]);
        }
        put(&mut out, 0x68, &[self.banner_offset]);
        put(&mut out, 0x80, &[self.ntr_rom_size, self.header_size]);
        if out.len() >= 0x214 {
            put(&mut out, 0x210, &[self.twl_rom_size]);
        }
        out[LOGO_CRC_OFFSET..LOGO_CRC_OFFSET + 2].copy_from_slice(&self.logo_crc.to_le_bytes());
        out[HEADER_CRC_OFFSET..HEADER_CRC_OFFSET + 2]
            .copy_from_slice(&self.header_crc.to_le_bytes());
        out
    }

    /// Serialise, writing every parsed field over the retained raw bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_without_crc_update()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> Vec<u8> {
        let mut h = vec![0u8; 0x200];
        h[..9].copy_from_slice(b"TESTTITLE");
        h[12..16].copy_from_slice(b"ABCD");
        h[0x20..0x24].copy_from_slice(&0x4000u32.to_le_bytes());
        h[0x54..0x58].copy_from_slice(&64u32.to_le_bytes());
        h
    }

    #[test]
    fn crc_and_roundtrip() {
        let mut header = Header::parse(&blank()).unwrap();
        assert!(!header.header_crc_valid());
        header.update_crcs();
        assert!(header.header_crc_valid());
        assert!(header.logo_crc_valid());
        let bytes = header.to_bytes();
        let again = Header::parse(&bytes).unwrap();
        assert_eq!(again.to_bytes(), bytes);
        assert_eq!(again.header_crc, header.header_crc);
        assert!(again.header_crc_valid());
        assert_eq!(again.title_str(), "TESTTITLE");
        assert_eq!(again.arm9_overlay_count(), 2);
        assert_eq!(
            u16::from_le_bytes([bytes[0x15E], bytes[0x15F]]),
            crc16(&bytes[..0x15E])
        );
    }

    #[test]
    fn too_short() {
        assert!(Header::parse(&[0; 16]).is_err());
    }
}
