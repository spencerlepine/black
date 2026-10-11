//! ARM9/ARM7 overlay table (`y9.bin` / `y7.bin`).

use crate::error::{le32, Error, Result};
use std::fmt::Write;

/// Size of one overlay table entry.
pub const ENTRY_SIZE: usize = 32;
/// Flag: the overlay is BLZ compressed.
pub const FLAG_COMPRESSED: u8 = 0x01;
/// Flag: the overlay is authenticated (set on every overlay by the SDK's tools).
pub const FLAG_AUTHENTICATED: u8 = 0x02;
/// The flags as they sit in the top byte of the last table word.
pub const WORD_COMPRESSED: u32 = 0x0100_0000;
pub const WORD_AUTHENTICATED: u32 = 0x0200_0000;

/// One overlay table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overlay {
    pub id: u32,
    pub ram_address: u32,
    pub ram_size: u32,
    pub bss_size: u32,
    pub sinit_start: u32,
    pub sinit_end: u32,
    pub file_id: u32,
    pub compressed_size: u32,
    pub flags: u8,
}

impl Overlay {
    pub fn compressed(&self) -> bool {
        self.flags & FLAG_COMPRESSED != 0
    }

    pub fn authenticated(&self) -> bool {
        self.flags & FLAG_AUTHENTICATED != 0
    }

    pub fn ram_end(&self) -> u32 {
        self.ram_address.wrapping_add(self.ram_size)
    }

    pub fn bss_end(&self) -> u32 {
        self.ram_end().wrapping_add(self.bss_size)
    }

    /// The last table word: compressed size in the low 24 bits, flags above.
    pub fn packed_word(&self) -> u32 {
        (self.compressed_size & 0xFF_FFFF) | (u32::from(self.flags) << 24)
    }

    pub fn to_bytes(&self) -> [u8; ENTRY_SIZE] {
        let words = [
            self.id,
            self.ram_address,
            self.ram_size,
            self.bss_size,
            self.sinit_start,
            self.sinit_end,
            self.file_id,
            self.packed_word(),
        ];
        let mut out = [0; ENTRY_SIZE];
        for (i, w) in words.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        out
    }

    fn parse(raw: &[u8]) -> Result<Overlay> {
        let w = |i: usize| le32(raw, i * 4, "overlay entry");
        let packed = w(7)?;
        Ok(Overlay {
            id: w(0)?,
            ram_address: w(1)?,
            ram_size: w(2)?,
            bss_size: w(3)?,
            sinit_start: w(4)?,
            sinit_end: w(5)?,
            file_id: w(6)?,
            compressed_size: packed & 0xFF_FFFF,
            flags: (packed >> 24) as u8,
        })
    }
}

/// Parse an overlay table.
pub fn parse_table(data: &[u8]) -> Result<Vec<Overlay>> {
    if !data.len().is_multiple_of(ENTRY_SIZE) {
        return Err(Error::invalid(
            "overlay table",
            format!("{} bytes is not a multiple of {ENTRY_SIZE}", data.len()),
        ));
    }
    (0..data.len() / ENTRY_SIZE)
        .map(|i| Overlay::parse(&data[i * ENTRY_SIZE..]))
        .collect()
}

/// Serialise an overlay table.
pub fn build_table(overlays: &[Overlay]) -> Vec<u8> {
    overlays.iter().flat_map(|o| o.to_bytes()).collect()
}

/// Human-readable listing plus summary, as printed by `parse_overlay_table.py`.
pub fn format_listing(overlays: &[Overlay]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "{} overlays\n", overlays.len());
    let _ = writeln!(
        s,
        "{:>4}  {:>10}  {:>8}  {:>8}  {:>10}  {:>5}  flags",
        "id", "ram", "size", "bss", "end", "file"
    );
    for ov in overlays {
        let mut flags = Vec::new();
        if ov.compressed() {
            flags.push("compressed");
        }
        if ov.authenticated() {
            flags.push("authenticated");
        }
        let flags = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.join(",")
        };
        let _ = writeln!(
            s,
            "{:>4}  0x{:08X}  0x{:06X}  0x{:06X}  0x{:08X}  {:>5}  {}",
            ov.id,
            ov.ram_address,
            ov.ram_size,
            ov.bss_size,
            ov.bss_end(),
            ov.file_id,
            flags
        );
    }
    if overlays.is_empty() {
        return s;
    }
    let lo = overlays.iter().map(|o| o.ram_address).min().unwrap_or(0);
    let hi = overlays.iter().map(|o| o.bss_end()).max().unwrap_or(0);
    let mut groups: std::collections::BTreeMap<u32, usize> = Default::default();
    for ov in overlays {
        *groups.entry(ov.ram_address).or_default() += 1;
    }
    let _ = writeln!(s, "\naddress range:  0x{lo:08X} - 0x{hi:08X}");
    let _ = writeln!(
        s,
        "compressed:     {} of {}",
        overlays.iter().filter(|o| o.compressed()).count(),
        overlays.len()
    );
    let _ = writeln!(
        s,
        "with bss:       {}",
        overlays.iter().filter(|o| o.bss_size != 0).count()
    );
    let _ = writeln!(
        s,
        "with sinit:     {}",
        overlays
            .iter()
            .filter(|o| o.sinit_start != o.sinit_end)
            .count()
    );
    let _ = writeln!(
        s,
        "load addresses: {} distinct, {} shared by multiple overlays",
        groups.len(),
        groups.values().filter(|&&c| c > 1).count()
    );
    s
}

/// `Overlay` blocks for `main.lsf`.
pub fn format_lsf(overlays: &[Overlay]) -> String {
    let mut s = String::new();
    for ov in overlays {
        let _ = write!(
            s,
            "Overlay ov{:03}\n{{\n\tAddress\t\t0x{:08X}\n\tAfter\t\tmain\n}}\n\n",
            ov.id, ov.ram_address
        );
    }
    s
}

/// CSV listing.
pub fn format_csv(overlays: &[Overlay]) -> String {
    let mut s = String::from(
        "id,ram_address,ram_size,bss_size,sinit_start,sinit_end,file_id,compressed_size,flags\n",
    );
    for ov in overlays {
        let _ = writeln!(
            s,
            "{},0x{:08X},0x{:X},0x{:X},0x{:08X},0x{:08X},{},0x{:X},0x{:02X}",
            ov.id,
            ov.ram_address,
            ov.ram_size,
            ov.bss_size,
            ov.sinit_start,
            ov.sinit_end,
            ov.file_id,
            ov.compressed_size,
            ov.flags
        );
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_roundtrip() {
        let ov = Overlay {
            id: 3,
            ram_address: 0x0210_0000,
            ram_size: 0x1234,
            bss_size: 0x20,
            sinit_start: 0x0210_1000,
            sinit_end: 0x0210_1004,
            file_id: 3,
            compressed_size: 0x11C,
            flags: 3,
        };
        let bytes = build_table(&[ov, ov]);
        assert_eq!(bytes.len(), 64);
        let back = parse_table(&bytes).unwrap();
        assert_eq!(back, vec![ov, ov]);
        assert!(back[0].compressed() && back[0].authenticated());
        assert_eq!(ov.packed_word(), 0x0300_011C);
        assert!(parse_table(&bytes[..33]).is_err());
        assert!(format_listing(&back).contains("compressed,authenticated"));
    }
}
