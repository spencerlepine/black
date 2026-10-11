//! A loaded ROM image and extraction of its contents (`dump_*`/`split` equivalents).

use crate::banner::banner_size;
use crate::error::{create_dir, le16, read_file, slice, write_file, Error, Result};
use crate::fnt::{parse_fat, FatEntry, Fnt};
use crate::header::Header;
use crate::overlay::{parse_table, Overlay};
use std::path::Path;

/// Sector size every file and region is aligned to.
pub const SECTOR: usize = 512;
/// Size of the ROM header region (header, then zero padding up to the ARM9).
pub const HEADER_REGION: usize = 0x4000;

/// Round `value` up to `boundary` (a power of two).
pub fn align(value: usize, boundary: usize) -> usize {
    (value + boundary - 1) & !(boundary - 1)
}

/// A ROM image held in memory with its parsed structures.
pub struct Rom {
    pub data: Vec<u8>,
    pub header: Header,
    pub fat: Vec<FatEntry>,
    pub fnt: Fnt,
}

impl Rom {
    pub fn open(path: impl AsRef<Path>) -> Result<Rom> {
        Rom::parse(read_file(path)?)
    }

    pub fn parse(data: Vec<u8>) -> Result<Rom> {
        let header = Header::parse(&data)?;
        let fat_bytes = slice(
            &data,
            header.fat.offset as usize,
            header.fat.size as usize,
            "FAT",
        )?;
        let fat = parse_fat(fat_bytes)?;
        let fnt_bytes = slice(
            &data,
            header.fnt.offset as usize,
            header.fnt.size as usize,
            "FNT",
        )?;
        let fnt = Fnt::parse(fnt_bytes)?;
        Ok(Rom {
            data,
            header,
            fat,
            fnt,
        })
    }

    /// The ARM9 binary exactly as stored (still BLZ compressed when it is).
    pub fn arm9(&self) -> Result<&[u8]> {
        let b = self.header.arm9;
        slice(&self.data, b.rom_offset as usize, b.size as usize, "ARM9")
    }

    pub fn arm7(&self) -> Result<&[u8]> {
        let b = self.header.arm7;
        slice(&self.data, b.rom_offset as usize, b.size as usize, "ARM7")
    }

    /// Raw ARM9 overlay table (`y9.bin`).
    pub fn arm9_table_bytes(&self) -> Result<&[u8]> {
        let r = self.header.arm9_overlays;
        slice(
            &self.data,
            r.offset as usize,
            r.size as usize,
            "overlay table",
        )
    }

    pub fn arm7_table_bytes(&self) -> Result<&[u8]> {
        let r = self.header.arm7_overlays;
        slice(
            &self.data,
            r.offset as usize,
            r.size as usize,
            "ARM7 overlay table",
        )
    }

    pub fn arm9_overlays(&self) -> Result<Vec<Overlay>> {
        parse_table(self.arm9_table_bytes()?)
    }

    /// Contents of the file with the given FAT id.
    pub fn file(&self, id: usize) -> Result<&[u8]> {
        let e = self
            .fat
            .get(id)
            .ok_or_else(|| Error::invalid("file id", format!("{id} is past the FAT")))?;
        slice(&self.data, e.start as usize, e.len(), "file")
    }

    pub fn banner(&self) -> Result<&[u8]> {
        let off = self.header.banner_offset as usize;
        if off == 0 {
            return Ok(&[]);
        }
        let version = le16(&self.data, off, "banner")?;
        slice(&self.data, off, banner_size(version), "banner")
    }

    /// Offset where the DSi payload and digests past the file tree begin: the next sector
    /// boundary after everything the NitroFS and the fixed regions occupy.
    pub fn twl_tail_offset(&self) -> usize {
        let h = &self.header;
        let mut end = HEADER_REGION;
        let mut bump = |off: u32, size: u32| {
            if off != 0 || size != 0 {
                end = end.max(off as usize + size as usize);
            }
        };
        bump(h.arm9.rom_offset, h.arm9.size);
        bump(h.arm7.rom_offset, h.arm7.size);
        bump(h.fnt.offset, h.fnt.size);
        bump(h.fat.offset, h.fat.size);
        bump(h.arm9_overlays.offset, h.arm9_overlays.size);
        bump(h.arm7_overlays.offset, h.arm7_overlays.size);
        if h.banner_offset != 0 {
            end = end.max(h.banner_offset as usize + self.banner().map_or(0, <[u8]>::len));
        }
        for e in &self.fat {
            end = end.max(e.end as usize);
        }
        align(end, SECTOR)
    }

    /// Data files (everything after the overlays) as `(file id, path)`.
    pub fn data_files(&self) -> Vec<(u16, String)> {
        let first = self.header.arm9_overlay_count() as u16;
        let mut files: Vec<_> = self
            .fnt
            .files()
            .into_iter()
            .filter(|f| f.0 >= first)
            .collect();
        files.sort_by_key(|f| f.0);
        files
    }
}

/// What [`extract`] wrote.
#[derive(Debug, Default)]
pub struct ExtractReport {
    pub overlays: usize,
    pub data_files: usize,
    pub tail_bytes: usize,
}

/// Extract everything needed to rebuild the ROM with [`crate::build::assemble`].
///
/// Layout of `outdir`: `header.bin` (the 0x4000-byte header region), `arm9.bin`, `arm7.bin`,
/// `y9.bin`, `y7.bin`, `banner.bin`, `fnt.bin`, `twl_tail.bin`, `files.txt` (data file order),
/// `overlay/overlay_NNNN.bin` and `data/<NitroFS path>`.
pub fn extract(rom: &Rom, outdir: &Path) -> Result<ExtractReport> {
    create_dir(outdir.join("overlay"))?;
    let mut report = ExtractReport::default();
    let head = HEADER_REGION.min(rom.data.len());
    write_file(outdir.join("header.bin"), &rom.data[..head])?;
    write_file(outdir.join("arm9.bin"), rom.arm9()?)?;
    write_file(outdir.join("arm7.bin"), rom.arm7()?)?;
    write_file(outdir.join("y9.bin"), rom.arm9_table_bytes()?)?;
    write_file(outdir.join("y7.bin"), rom.arm7_table_bytes()?)?;
    write_file(outdir.join("banner.bin"), rom.banner()?)?;
    let fnt = slice(
        &rom.data,
        rom.header.fnt.offset as usize,
        rom.header.fnt.size as usize,
        "FNT",
    )?;
    write_file(outdir.join("fnt.bin"), fnt)?;
    for index in 0..rom.header.arm9_overlay_count() {
        write_file(
            outdir.join(format!("overlay/overlay_{index:04}.bin")),
            rom.file(index)?,
        )?;
        report.overlays += 1;
    }
    let mut order = String::from(
        "# data files in ROM order, one path per line, ids start after the overlays\n",
    );
    for (id, path) in rom.data_files() {
        let target = outdir.join("data").join(&path);
        if let Some(parent) = target.parent() {
            create_dir(parent)?;
        }
        write_file(&target, rom.file(id as usize)?)?;
        order.push_str(&path);
        order.push('\n');
        report.data_files += 1;
    }
    write_file(outdir.join("files.txt"), order.as_bytes())?;
    let tail = rom.twl_tail_offset().min(rom.data.len());
    write_file(outdir.join("twl_tail.bin"), &rom.data[tail..])?;
    report.tail_bytes = rom.data.len() - tail;
    Ok(report)
}
