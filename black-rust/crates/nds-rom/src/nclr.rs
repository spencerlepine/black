//! NCLR palette files (`RLCN` / `TTLP`).

use crate::error::{Error, Result};
use crate::nitro::{payload_off, u16_at, u32_at, Container};

/// A parsed palette: BGR555 colour entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nclr {
    /// Bit depth code from the file: 3 = 4bpp (16-colour rows), 4 = 8bpp (256 colours).
    pub depth: u32,
    pub colors: Vec<u16>,
}

impl Nclr {
    pub fn parse(data: &[u8]) -> Result<Nclr> {
        let c = Container::parse(data, b"RLCN", "NCLR")?;
        let b = c.block(b"TTLP", "NCLR")?;
        let depth = u32_at(b, 0)?;
        let size = u32_at(b, 8)? as usize;
        let start = payload_off(u32_at(b, 12)?, "NCLR")?;
        let end = start.saturating_add(size).min(b.len());
        let raw = b
            .get(start..end)
            .ok_or_else(|| Error::invalid("NCLR", "palette data out of range"))?;
        let colors = (0..raw.len() / 2)
            .map(|i| u16_at(raw, i * 2))
            .collect::<Result<_>>()?;
        Ok(Nclr { depth, colors })
    }

    /// RGBA of colour `index` of palette row `row` (16 colours per row). Index 0 is transparent.
    pub fn rgba(&self, row: usize, index: usize) -> [u8; 4] {
        let e = self.colors.get(row * 16 + index).copied().unwrap_or(0);
        bgr555_to_rgba(e, index != 0)
    }
}

/// Expand a BGR555 entry to RGBA8.
pub fn bgr555_to_rgba(e: u16, opaque: bool) -> [u8; 4] {
    let c = |s: u32| {
        let v = ((e >> s) & 0x1F) as u32;
        ((v << 3) | (v >> 2)) as u8
    };
    [c(0), c(5), c(10), if opaque { 255 } else { 0 }]
}
