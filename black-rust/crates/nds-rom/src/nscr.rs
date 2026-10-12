//! NSCR screen (tilemap) files (`RCSN` / `NRCS`).

use crate::error::Result;
use crate::nitro::{u16_at, u32_at, Container};

/// One tilemap entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenEntry {
    pub tile: u16,
    pub hflip: bool,
    pub vflip: bool,
    pub palette: u8,
}

impl ScreenEntry {
    pub fn from_raw(v: u16) -> Self {
        ScreenEntry {
            tile: v & 0x3FF,
            hflip: v & 0x400 != 0,
            vflip: v & 0x800 != 0,
            palette: (v >> 12) as u8,
        }
    }

    pub fn to_raw(self) -> u16 {
        self.tile
            | (self.hflip as u16) << 10
            | (self.vflip as u16) << 11
            | (self.palette as u16) << 12
    }
}

/// Parsed screen: `width` x `height` pixels, `(width/8) * (height/8)` entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nscr {
    pub width: u16,
    pub height: u16,
    pub entries: Vec<ScreenEntry>,
}

impl Nscr {
    pub fn parse(data: &[u8]) -> Result<Nscr> {
        let c = Container::parse(data, b"RCSN", "NSCR")?;
        let b = c.block(b"NRCS", "NSCR")?;
        let width = u16_at(b, 0)?;
        let height = u16_at(b, 2)?;
        let size = u32_at(b, 8)? as usize;
        let raw = b
            .get(12..12usize.saturating_add(size).min(b.len()))
            .unwrap_or(&[]);
        let entries = (0..raw.len() / 2)
            .map(|i| u16_at(raw, i * 2).map(ScreenEntry::from_raw))
            .collect::<Result<_>>()?;
        Ok(Nscr {
            width,
            height,
            entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_bits_round_trip() {
        let e = ScreenEntry::from_raw(0xA5FF);
        assert_eq!(
            (e.tile, e.hflip, e.vflip, e.palette),
            (0x1FF, true, false, 10)
        );
        assert_eq!(e.to_raw(), 0xA5FF);
    }

    #[test]
    fn flips_apply_when_rendering() {
        use crate::{ncgr::Ncgr, nclr::Nclr, render};
        // One 4bpp tile with only pixel (0,0) set to colour 1.
        let mut data = vec![0u8; 32];
        data[0] = 1;
        let g = Ncgr {
            tiles_x: Some(1),
            tiles_y: Some(1),
            bpp: 4,
            scanned: false,
            data,
        };
        let pal = Nclr {
            depth: 3,
            colors: vec![0, 0x1F],
        };
        let scr = Nscr {
            width: 8,
            height: 8,
            entries: vec![ScreenEntry::from_raw(0xC00)],
        };
        let img = render::render_screen(&g, &pal, &scr);
        assert_eq!(img.pixel(7, 7), [255, 0, 0, 255]);
        assert_eq!(img.pixel(0, 0)[3], 0);
    }
}
