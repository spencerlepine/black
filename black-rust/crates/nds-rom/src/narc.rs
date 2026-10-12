//! NARC archives (Nitro ARChive): a small container of numbered files.
//!
//! Layout: a 16-byte header (`NARC`, BOM `FE FF`, version, total size, header size, chunk count)
//! followed by three chunks: `BTAF` (file allocation table: start/end offset per file, relative to
//! the data), `BTNF` (file name table, in the same format as the ROM's FNT) and `GMIF` (the file
//! data). Most game archives are nameless, with a one-directory name table and no entries.

use crate::error::{le16, le32, slice, Error, Result};
use crate::fnt::Fnt;

const HEADER_LEN: usize = 0x10;
/// Byte-order mark as read little-endian (the bytes in the file are `FE FF`).
const BOM: u16 = 0xFFFE;
const PAD_BYTE: u8 = 0xFF;

/// A parsed NARC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Narc {
    /// Format version field (normally `0x0100`).
    pub version: u16,
    /// Raw BTNF chunk payload.
    fntb: Vec<u8>,
    /// Raw GMIF chunk payload (the area the FATB offsets point into).
    data: Vec<u8>,
    /// `(start, end)` of each file inside `data`.
    ranges: Vec<(usize, usize)>,
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

impl Narc {
    /// Parse a NARC image.
    pub fn parse(bytes: &[u8]) -> Result<Narc> {
        if slice(bytes, 0, 4, "NARC header")? != b"NARC" {
            return Err(Error::invalid("NARC", "bad magic"));
        }
        let bom = le16(bytes, 4, "NARC header")?;
        if bom != BOM {
            return Err(Error::invalid("NARC", format!("unsupported BOM {bom:#x}")));
        }
        let version = le16(bytes, 6, "NARC header")?;
        let total = le32(bytes, 8, "NARC header")? as usize;
        let header_size = le16(bytes, 12, "NARC header")? as usize;
        let chunks = le16(bytes, 14, "NARC header")? as usize;
        if header_size < HEADER_LEN || total > bytes.len() {
            return Err(Error::invalid("NARC", "bad header or size"));
        }
        let bytes = &bytes[..total];

        let (mut fatb, mut fntb, mut data) = (None, None, None);
        let mut off = header_size;
        for _ in 0..chunks {
            let magic = slice(bytes, off, 4, "NARC chunk")?;
            let size = le32(bytes, off + 4, "NARC chunk")? as usize;
            if size < 8 {
                return Err(Error::invalid("NARC", "chunk smaller than its header"));
            }
            let body = slice(bytes, off + 8, size - 8, "NARC chunk")?;
            match magic {
                b"BTAF" => fatb = Some(body),
                b"BTNF" => fntb = Some(body),
                b"GMIF" => data = Some(body),
                _ => return Err(Error::invalid("NARC", "unknown chunk")),
            }
            off += size;
        }
        let missing = |name| Error::invalid("NARC", format!("missing {name} chunk"));
        let fatb = fatb.ok_or_else(|| missing("BTAF"))?;
        let fntb = fntb.ok_or_else(|| missing("BTNF"))?;
        let data = data.ok_or_else(|| missing("GMIF"))?;

        let count = le16(fatb, 0, "FATB")? as usize;
        let mut ranges = Vec::with_capacity(count);
        for i in 0..count {
            let start = le32(fatb, 4 + i * 8, "FATB entry")? as usize;
            let end = le32(fatb, 8 + i * 8, "FATB entry")? as usize;
            if start > end || end > data.len() {
                return Err(Error::invalid(
                    "NARC",
                    format!("file {i} range {start:#x}..{end:#x} outside data"),
                ));
            }
            ranges.push((start, end));
        }
        Ok(Narc {
            version,
            fntb: fntb.to_vec(),
            data: data.to_vec(),
            ranges,
        })
    }

    /// Build a nameless NARC from `files`, each starting 4-byte aligned.
    pub fn build<F: AsRef<[u8]>>(files: &[F]) -> Narc {
        let mut data = Vec::new();
        let mut ranges = Vec::with_capacity(files.len());
        for f in files {
            let f = f.as_ref();
            data.resize(align4(data.len()), PAD_BYTE);
            ranges.push((data.len(), data.len() + f.len()));
            data.extend_from_slice(f);
        }
        data.resize(align4(data.len()), PAD_BYTE);
        // root directory only: sub-table at offset 4, first file id 0, 1 directory, no entries
        let mut fntb = Vec::new();
        fntb.extend_from_slice(&4u32.to_le_bytes());
        fntb.extend_from_slice(&0u16.to_le_bytes());
        fntb.extend_from_slice(&1u16.to_le_bytes());
        Narc {
            version: 0x0100,
            fntb,
            data,
            ranges,
        }
    }

    /// Serialise. Files are re-laid out 4-byte aligned in index order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let files: Vec<&[u8]> = (0..self.len()).map(|i| self.file(i)).collect();
        let rebuilt = Narc::build(&files);
        let (fatb, data) = (rebuilt.ranges, rebuilt.data);
        let fatb_size = 12 + fatb.len() * 8;
        let fntb_size = 8 + align4(self.fntb.len());
        let gmif_size = 8 + data.len();
        let total = HEADER_LEN + fatb_size + fntb_size + gmif_size;

        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(b"NARC");
        out.extend_from_slice(&BOM.to_le_bytes());
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&(total as u32).to_le_bytes());
        out.extend_from_slice(&(HEADER_LEN as u16).to_le_bytes());
        out.extend_from_slice(&3u16.to_le_bytes());
        out.extend_from_slice(b"BTAF");
        out.extend_from_slice(&(fatb_size as u32).to_le_bytes());
        out.extend_from_slice(&(fatb.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        for (s, e) in &fatb {
            out.extend_from_slice(&(*s as u32).to_le_bytes());
            out.extend_from_slice(&(*e as u32).to_le_bytes());
        }
        out.extend_from_slice(b"BTNF");
        out.extend_from_slice(&(fntb_size as u32).to_le_bytes());
        out.extend_from_slice(&self.fntb);
        out.resize(HEADER_LEN + fatb_size + fntb_size, PAD_BYTE);
        out.extend_from_slice(b"GMIF");
        out.extend_from_slice(&(gmif_size as u32).to_le_bytes());
        out.extend_from_slice(&data);
        out
    }

    /// Number of files.
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// File `i`. Panics when `i >= len()`.
    pub fn file(&self, i: usize) -> &[u8] {
        let (s, e) = self.ranges[i];
        &self.data[s..e]
    }

    /// File `i`, or `None` when out of range.
    pub fn get(&self, i: usize) -> Option<&[u8]> {
        (i < self.len()).then(|| self.file(i))
    }

    /// Iterate over the files in index order.
    pub fn files(&self) -> impl Iterator<Item = &[u8]> {
        (0..self.len()).map(|i| self.file(i))
    }

    /// Path of each file by index, when the archive carries a name table.
    ///
    /// `None` for nameless archives (or an unparsable name table). Files without a name get
    /// an empty string.
    pub fn names(&self) -> Option<Vec<String>> {
        let fnt = Fnt::parse(&self.fntb).ok()?;
        let files = fnt.files();
        if files.is_empty() {
            return None;
        }
        let mut names = vec![String::new(); self.len()];
        for (id, path) in files {
            if let Some(slot) = names.get_mut(id as usize) {
                *slot = path;
            }
        }
        Some(names)
    }
}

#[cfg(test)]
mod tests;
