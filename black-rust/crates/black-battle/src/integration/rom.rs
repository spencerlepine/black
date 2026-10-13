//! Read-only ROM images the integration reads tables from (the ROM build `BLACK_ROM_BUILD`; D5:
//! nothing of it is committed) and the game data tables (`black-data`, from `black-rom/files`).

use black_data::extract::{read_tables, RomSource, Tables};
use std::path::PathBuf;

pub const OVY93_BASE: u32 = 0x021B_6100;
pub const OVY95_BASE: u32 = 0x0689_8020;
pub const OVY96_BASE: u32 = 0x0218_7F00;
pub const MAIN_BASE: u32 = 0x0200_4000;

pub struct Rom {
    pub ovy93: Vec<u8>,
    pub ovy95: Vec<u8>,
    pub ovy96: Vec<u8>,
    pub main: Vec<u8>,
    pub tables: Tables,
    /// The extracted ROM file system (`black-rom/files`).
    pub files: PathBuf,
}

impl Rom {
    /// `None` when the ROM build or files are missing.
    pub fn load() -> Option<Self> {
        let build = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        let files = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../black-rom/files"
        ));
        let build = PathBuf::from(build);
        let rd = |n: &str| std::fs::read(build.join(n)).ok();
        let src = RomSource {
            files: files.clone(),
            overlay93: build.join("OVY_93.sbin"),
            main_sbin: build.join("main.sbin"),
        };
        Some(Rom {
            ovy93: rd("OVY_93.sbin")?,
            ovy95: rd("OVY_95.sbin")?,
            ovy96: rd("OVY_96.sbin")?,
            main: rd("main.sbin")?,
            tables: read_tables(&src).ok()?,
            files,
        })
    }

    /// The image and base holding `addr`.
    fn image(&self, addr: u32) -> (&[u8], u32) {
        for (img, base) in [
            (&self.ovy93, OVY93_BASE),
            (&self.ovy95, OVY95_BASE),
            (&self.ovy96, OVY96_BASE),
            (&self.main, MAIN_BASE),
        ] {
            if addr >= base && ((addr - base) as usize) < img.len() {
                return (img, base);
            }
        }
        panic!("B16: ROM read outside the loaded images: {addr:#010x}");
    }

    pub fn r8(&self, addr: u32) -> u32 {
        let (img, base) = self.image(addr);
        img[(addr - base) as usize] as u32
    }

    pub fn r16(&self, addr: u32) -> u32 {
        self.r8(addr) | self.r8(addr + 1) << 8
    }

    pub fn r32(&self, addr: u32) -> u32 {
        self.r16(addr) | self.r16(addr + 2) << 16
    }

    pub fn bytes(&self, addr: u32, n: usize) -> Vec<u8> {
        (0..n as u32).map(|i| self.r8(addr + i) as u8).collect()
    }

    /// A handler-table getter `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr` (Thumb):
    /// `(table, n)`.
    pub fn getter(&self, g: u32) -> (u32, u32) {
        let g = g & !1;
        let (mov, str, ldr, bx) = (
            self.r16(g),
            self.r16(g + 2),
            self.r16(g + 4),
            self.r16(g + 6),
        );
        assert!(
            mov >> 8 == 0x21 && str == 0x6001 && ldr >> 8 == 0x48 && bx == 0x4770,
            "B16: handler getter {g:#x} is not a constant getter"
        );
        let table = self.r32(((g + 8) & !3) + (ldr & 0xFF) * 4);
        (table, mov & 0xFF)
    }
}
