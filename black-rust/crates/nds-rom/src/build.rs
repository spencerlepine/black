//! ROM assembly (port of `make_rom.py`).

use crate::error::{le32, read_file, Error, Result};
use crate::fnt::{build_fat, FatEntry, Fnt};
use crate::rom::{align, HEADER_REGION, SECTOR};
use std::path::{Path, PathBuf};

const PAD: u8 = 0xFF;

/// Inputs of [`assemble`], mirroring the arguments of `make_rom.py`.
#[derive(Debug, Clone)]
pub struct BuildInputs {
    /// Header template; the rest of the 0x4000-byte header region is zero padding.
    pub header: PathBuf,
    pub fnt: PathBuf,
    /// DSi payload and digests carried over whole past the file tree.
    pub twl_tail: PathBuf,
    /// Compressed ARM9 (truncated to the header's ARM9 size).
    pub arm9: PathBuf,
    /// ARM7 binary (truncated to the header's ARM7 size).
    pub arm7: PathBuf,
    /// Overlay table.
    pub table: PathBuf,
    pub banner: PathBuf,
    /// Directory of overlays in their ROM storage form.
    pub overlays: PathBuf,
    /// File name pattern inside `overlays`; `{n}` is the overlay index, `{n:04}` zero padded.
    pub overlay_pattern: String,
    /// Data file order, one path per line (`#` starts a comment).
    pub files: PathBuf,
    /// Root of the NitroFS data tree.
    pub data_dir: PathBuf,
}

/// Summary of an assembled ROM.
#[derive(Debug, Clone, Copy)]
pub struct BuildReport {
    pub size: usize,
    pub files: usize,
}

/// Parse a `files.txt`-style list.
pub fn parse_file_list(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Expand an overlay file name pattern.
pub fn overlay_file_name(pattern: &str, index: usize) -> String {
    pattern
        .replace("{n:04}", &format!("{index:04}"))
        .replace("{n}", &index.to_string())
}

struct Image {
    rom: Vec<u8>,
}

impl Image {
    fn place(&mut self, what: &str, data: &[u8], at: Option<usize>) -> Result<(usize, usize)> {
        let start = at.unwrap_or_else(|| align(self.rom.len(), SECTOR));
        if start < self.rom.len() {
            return Err(Error::Overlap {
                what: what.to_string(),
                at: start,
                len: self.rom.len(),
            });
        }
        self.rom.resize(start, PAD);
        self.rom.extend_from_slice(data);
        Ok((start, data.len()))
    }
}

fn truncated(mut data: Vec<u8>, len: u32) -> Vec<u8> {
    data.truncate(len as usize);
    data
}

/// Assemble the ROM image.
pub fn assemble(inputs: &BuildInputs) -> Result<(Vec<u8>, BuildReport)> {
    let header = read_file(&inputs.header)?;
    let u = |off: usize| le32(&header, off, "header template");
    let fnt = read_file(&inputs.fnt)?;
    let order = parse_file_list(&String::from_utf8_lossy(&read_file(&inputs.files)?));
    let overlay_count = u(0x54)? as usize / 32;
    let total = overlay_count + order.len();

    let mut img = Image {
        rom: header.clone(),
    };
    if img.rom.len() > HEADER_REGION {
        return Err(Error::invalid(
            "header template",
            "larger than the header region",
        ));
    }
    img.rom.resize(HEADER_REGION, 0);

    img.place(
        "ARM9",
        &truncated(read_file(&inputs.arm9)?, u(0x2C)?),
        Some(u(0x20)? as usize),
    )?;
    img.place(
        "overlay table",
        &read_file(&inputs.table)?,
        Some(u(0x50)? as usize),
    )?;

    let mut fat = Vec::with_capacity(total);
    for index in 0..overlay_count {
        let path = inputs
            .overlays
            .join(overlay_file_name(&inputs.overlay_pattern, index));
        let (start, size) = img.place("overlay", &read_file(path)?, None)?;
        fat.push(FatEntry {
            start: start as u32,
            end: (start + size) as u32,
        });
    }
    img.place(
        "ARM7",
        &truncated(read_file(&inputs.arm7)?, u(0x3C)?),
        Some(u(0x30)? as usize),
    )?;
    img.place("FNT", &fnt, Some(u(0x40)? as usize))?;
    let fat_at = u(0x48)? as usize;
    img.place("FAT", &vec![0; total * 8], Some(fat_at))?;
    img.place(
        "banner",
        &read_file(&inputs.banner)?,
        Some(u(0x68)? as usize),
    )?;

    for name in &order {
        let (start, size) =
            img.place("data file", &read_file(inputs.data_dir.join(name))?, None)?;
        fat.push(FatEntry {
            start: start as u32,
            end: (start + size) as u32,
        });
    }
    let fat_bytes = build_fat(&fat);
    img.rom[fat_at..fat_at + fat_bytes.len()].copy_from_slice(&fat_bytes);

    let tail = align(img.rom.len(), SECTOR);
    img.rom.resize(tail, PAD);
    img.rom.extend_from_slice(&read_file(&inputs.twl_tail)?);
    let report = BuildReport {
        size: img.rom.len(),
        files: total,
    };
    Ok((img.rom, report))
}

/// Print the data file order of a base ROM's FNT (`make_rom.py --list-order`).
pub fn list_order(header: &[u8], fnt: &[u8]) -> Result<Vec<String>> {
    let first = le32(header, 0x54, "header")? as usize / 32;
    let end = le32(header, 0x4C, "header")? as usize / 8;
    let files = Fnt::parse(fnt)?.files();
    let by_id: std::collections::HashMap<u16, String> = files.into_iter().collect();
    (first..end)
        .map(|id| {
            by_id
                .get(&(id as u16))
                .cloned()
                .ok_or_else(|| Error::invalid("FNT", format!("no path for file id {id}")))
        })
        .collect()
}

/// Convenience: the default relative paths the Makefile uses for `build_dir`.
pub fn default_inputs(project: &Path, build_dir: &Path) -> BuildInputs {
    let packed = build_dir.join("packed");
    BuildInputs {
        header: project.join("black.us/rom_header_template.sbin"),
        fnt: project.join("black.us/fnt.bin"),
        twl_tail: project.join("black.us/twl_tail.bin"),
        arm9: packed.join("arm9.bin"),
        arm7: project.join("sub/build/arm7.sbin"),
        table: packed.join("main_table.sbin"),
        banner: build_dir.join("banner.bin"),
        overlays: packed,
        overlay_pattern: "OVY_{n}.sbin".into(),
        files: project.join("black.us/files.txt"),
        data_dir: project.join("files"),
    }
}
