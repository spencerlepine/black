//! The ROM build the integration runs (`$BLACK_ROM_BUILD`: `main.sbin`, `OVY_<n>.sbin`, the overlay table
//! `main_table.sbin`, `main.elf` for the ARM / Thumb mapping symbols) and the extracted file system
//! (`black-rom/files`). D5: nothing of it is committed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// Where `main.sbin` is loaded.
pub const MAIN_BASE: u32 = 0x0200_4000;
/// `main.sbin`: code + data, then the autoload blocks, then their table (16-byte entries `dest, size, sinit,
/// bss`).
pub const MAIN_CODE_SIZE: usize = 0xA5E80;
const AUTOLOAD_TABLE: usize = MAIN_CODE_SIZE + 0x900;

/// One overlay of the build.
pub struct Overlay {
    pub base: u32,
    pub image: Vec<u8>,
    pub bss: u32,
    pub bss_size: u32,
    /// The static initialisers (`sinit` table) the loader runs after loading.
    pub sinit: Vec<u32>,
}

pub struct Rom {
    pub build: PathBuf,
    pub files: PathBuf,
    pub main: Vec<u8>,
    /// `(dest, bytes, bss size)` of the four autoload blocks.
    pub autoload: Vec<(u32, Vec<u8>, u32)>,
    overlays: HashMap<u16, Rc<Overlay>>,
    /// The overlay table (`main_table.sbin`): `(ram address, ram size, bss size, sinit start, sinit end)` by id.
    table: HashMap<u16, (u32, u32, u32, u32, u32)>,
    /// Mapping symbols (`$a` = true, `$t` / `$d` = false) by section: `None` = ARM9 main + autoloads, `Some(n)` =
    /// overlay `n`; sorted by address.
    arm: HashMap<Option<u16>, Vec<(u32, bool)>>,
}

impl Rom {
    /// `None` (skip) when the build or the files are missing.
    pub fn load() -> Option<Rom> {
        let build = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        let build = PathBuf::from(build);
        let files = build.join("../../files");
        let main = std::fs::read(build.join("main.sbin")).ok()?;
        let elf = std::fs::read(build.join("main.elf")).ok()?;
        let tab = std::fs::read(build.join("main_table.sbin")).ok()?;
        if !files.join("a/0/0/8").exists() {
            return None;
        }
        let mut autoload = Vec::new();
        let mut o = MAIN_CODE_SIZE;
        for i in 0..4 {
            let w = |k: usize| {
                u32::from_le_bytes(main[AUTOLOAD_TABLE + 16 * i + k..][..4].try_into().unwrap())
            };
            let (dst, size, bss) = (w(0), w(4) as usize, w(12));
            autoload.push((dst, main[o..o + size].to_vec(), bss));
            o += size;
        }
        assert_eq!(o, AUTOLOAD_TABLE, "F12: autoload layout");
        let mut table = HashMap::new();
        for e in tab.as_chunks::<32>().0 {
            let w = |k: usize| u32::from_le_bytes(e[4 * k..4 * k + 4].try_into().unwrap());
            table.insert(w(0) as u16, (w(1), w(2), w(3), w(4), w(5)));
        }
        Some(Rom {
            build,
            files,
            main: main[..MAIN_CODE_SIZE].to_vec(),
            autoload,
            overlays: HashMap::new(),
            table,
            arm: parse_elf(&elf),
        })
    }

    /// Overlay `n` (image, load address, BSS, static initialisers).
    pub fn overlay(&mut self, n: u16) -> Rc<Overlay> {
        if let Some(o) = self.overlays.get(&n) {
            return o.clone();
        }
        let image = std::fs::read(self.build.join(format!("OVY_{n}.sbin")))
            .unwrap_or_else(|e| panic!("F12: OVY_{n}.sbin: {e}"));
        let &(base, size, bss_size, s0, s1) = self
            .table
            .get(&n)
            .unwrap_or_else(|| panic!("F12: overlay {n} not in the overlay table"));
        assert_eq!(size as usize, image.len(), "F12: OVY_{n}.sbin size");
        let sinit = (s0..s1)
            .step_by(4)
            .map(|a| {
                let o = (a - base) as usize;
                u32::from_le_bytes(image[o..o + 4].try_into().unwrap())
            })
            .collect();
        let o = Rc::new(Overlay {
            base,
            image,
            bss: base + size,
            bss_size,
            sinit,
        });
        self.overlays.insert(n, o.clone());
        o
    }

    /// Whether the code at `a` is ARM (else Thumb), by the mapping symbols of main / the autoloads or of
    /// overlay `ov`.
    pub fn is_arm(&self, a: u32, ov: Option<u16>) -> bool {
        let Some(v) = self.arm.get(&ov) else {
            return false;
        };
        match v.binary_search_by(|e| e.0.cmp(&a)) {
            Ok(i) => v[i].1,
            Err(0) => false,
            Err(i) => v[i - 1].1,
        }
    }

    /// The path of ARC `n` (`files/a/X/Y/Z`).
    pub fn arc_path(&self, n: u32) -> PathBuf {
        self.files
            .join("a")
            .join((n / 100).to_string())
            .join((n / 10 % 10).to_string())
            .join((n % 10).to_string())
    }
}

type Mapping = HashMap<Option<u16>, Vec<(u32, bool)>>;

/// main.elf: the `$a` / `$t` / `$d` mapping symbols by section (`main` and the autoload sections, `OVY_<n>`).
fn parse_elf(d: &[u8]) -> Mapping {
    let u16_ = |o: usize| u16::from_le_bytes([d[o], d[o + 1]]);
    let u32_ = |o: usize| u32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let shoff = u32_(0x20) as usize;
    let (shentsize, shnum, shstrndx) = (
        u16_(0x2E) as usize,
        u16_(0x30) as usize,
        u16_(0x32) as usize,
    );
    // (name, type, file offset, size, link)
    let sh: Vec<(u32, u32, u32, u32, u32)> = (0..shnum)
        .map(|i| {
            let b = shoff + i * shentsize;
            (
                u32_(b),
                u32_(b + 4),
                u32_(b + 16),
                u32_(b + 20),
                u32_(b + 24),
            )
        })
        .collect();
    let cstr = |o: usize| -> String {
        let e = d[o..].iter().position(|&c| c == 0).unwrap();
        String::from_utf8_lossy(&d[o..o + e]).into_owned()
    };
    let names: Vec<String> = sh
        .iter()
        .map(|s| cstr(sh[shstrndx].2 as usize + s.0 as usize))
        .collect();
    let ovy = |n: &str| n.strip_prefix("OVY_").and_then(|x| x.parse::<u16>().ok());
    let mut arm: Mapping = HashMap::new();
    if let Some(st) = sh.iter().find(|s| s.1 == 2) {
        let stroff = sh[st.4 as usize].2 as usize;
        for i in 0..(st.3 / 16) as usize {
            let b = st.2 as usize + 16 * i;
            let (name, value, shndx) = (u32_(b), u32_(b + 4), u16_(b + 14) as usize);
            if shndx == 0 || shndx >= sh.len() {
                continue;
            }
            let is = match cstr(stroff + name as usize).as_str() {
                "$a" => true,
                "$t" | "$d" => false,
                _ => continue,
            };
            let key = ovy(&names[shndx]);
            if key.is_none()
                && !matches!(
                    names[shndx].as_str(),
                    "main" | "ITCM" | "DTCM" | "MAINEX" | "VRAM"
                )
            {
                continue;
            }
            arm.entry(key).or_default().push((value, is));
        }
    }
    for v in arm.values_mut() {
        v.sort_by_key(|e| e.0);
    }
    arm
}
