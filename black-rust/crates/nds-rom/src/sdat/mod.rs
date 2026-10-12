//! SDAT (Nitro sound archive) reader.
//!
//! Layout (SDK convention): a 0x40-byte header (`SDAT`, BOM, version, file size, header size,
//! block count, then offset/size of SYMB, INFO, FAT, FILE), then the blocks. SYMB holds names,
//! INFO holds per-entry metadata and the file id, FAT maps file id -> (offset, size) inside the
//! SDAT, FILE holds the embedded files (SSEQ, SSAR, SBNK, SWAR, STRM).

pub mod adpcm;
pub mod render;
pub mod sbnk;
pub mod ssar;
pub mod sseq;
pub mod strm;
pub mod swar;

use crate::error::{Error, Result};

/// The eight INFO/SYMB record kinds, in on-disk order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Seq,
    SeqArc,
    Bank,
    WaveArc,
    Player,
    Group,
    StrmPlayer,
    Strm,
}

impl Kind {
    pub const ALL: [Kind; 8] = [
        Kind::Seq,
        Kind::SeqArc,
        Kind::Bank,
        Kind::WaveArc,
        Kind::Player,
        Kind::Group,
        Kind::StrmPlayer,
        Kind::Strm,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Kind::Seq => "SEQ",
            Kind::SeqArc => "SEQARC",
            Kind::Bank => "BANK",
            Kind::WaveArc => "WAVEARC",
            Kind::Player => "PLAYER",
            Kind::Group => "GROUP",
            Kind::StrmPlayer => "STRMPLAYER",
            Kind::Strm => "STRM",
        }
    }
    fn index(self) -> usize {
        Kind::ALL.iter().position(|k| *k == self).unwrap()
    }
}

pub(crate) fn bad(what: &'static str, detail: impl Into<String>) -> Error {
    Error::invalid(what, detail)
}

/// Little-endian `i16`s of `b` (a trailing odd byte is ignored).
pub(crate) fn le_i16s(b: &[u8]) -> Vec<i16> {
    (0..b.len() / 2)
        .map(|i| i16::from_le_bytes([b[2 * i], b[2 * i + 1]]))
        .collect()
}

fn trunc(what: &'static str, offset: usize, need: usize, have: usize) -> Error {
    Error::Truncated {
        what,
        offset,
        need,
        have,
    }
}

pub(crate) fn u8_at(d: &[u8], o: usize, what: &'static str) -> Result<u8> {
    d.get(o).copied().ok_or_else(|| trunc(what, o, 1, d.len()))
}

pub(crate) fn u16_at(d: &[u8], o: usize, what: &'static str) -> Result<u16> {
    d.get(o..o.saturating_add(2))
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| trunc(what, o, 2, d.len()))
}

pub(crate) fn u32_at(d: &[u8], o: usize, what: &'static str) -> Result<u32> {
    d.get(o..o.saturating_add(4))
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| trunc(what, o, 4, d.len()))
}

/// Find a block by tag in a Nitro file (`magic[4] bom u16 ver u16 size u32 hdr u16 n u16`, then
/// `tag size payload`). Returns the whole block including its 8-byte header.
pub(crate) fn find_block<'a>(
    d: &'a [u8],
    magic: &[u8; 4],
    tag: &[u8; 4],
    what: &'static str,
) -> Result<&'a [u8]> {
    if d.len() < 16 || &d[..4] != magic {
        return Err(bad(what, "bad magic"));
    }
    let mut pos = u16_at(d, 12, what)? as usize;
    for _ in 0..u16_at(d, 14, what)? {
        let t = d
            .get(pos..pos + 4)
            .ok_or_else(|| trunc(what, pos, 8, d.len()))?;
        let size = u32_at(d, pos + 4, what)? as usize;
        if size < 8 {
            return Err(bad(what, "block smaller than its header"));
        }
        let end = (pos + size).min(d.len());
        if t == tag {
            return Ok(&d[pos..end]);
        }
        pos += size;
    }
    Err(bad(
        what,
        format!("no {} block", String::from_utf8_lossy(tag)),
    ))
}

/// Per-entry INFO record. Fields not meaningful for a kind are zero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Info {
    /// FAT file id (SEQ, SEQARC, BANK, WAVEARC, STRM).
    pub file_id: u32,
    /// SEQ: bank id. PLAYER: max sequences.
    pub bank: u16,
    pub volume: u8,
    pub channel_prio: u8,
    pub player_prio: u8,
    /// SEQ and STRM: player id.
    pub player: u8,
    /// BANK: the four SWAR ids (0xFFFF = none).
    pub wave_arcs: [u16; 4],
    /// PLAYER: channel mask.
    pub channel_mask: u16,
    /// PLAYER: heap size. WAVEARC: flag. SEQARC: reserved word.
    pub extra: u32,
    /// GROUP: `(type, entry)` pairs.
    pub group_items: Vec<(u32, u32)>,
    /// STRMPLAYER: channel numbers.
    pub channels: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub name: Option<String>,
    /// SEQARC only: names of the sequences inside the archive.
    pub sub_names: Vec<Option<String>>,
    pub info: Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FatEntry {
    pub offset: u32,
    pub size: u32,
}

/// A parsed SDAT. Borrows the file bytes; embedded files are sub-slices.
#[derive(Debug)]
pub struct Sdat<'a> {
    data: &'a [u8],
    /// Per kind, per id; `None` for holes in INFO.
    tables: Vec<Vec<Option<Entry>>>,
    pub fat: Vec<FatEntry>,
}

fn cstr(d: &[u8], o: usize) -> Option<String> {
    let s = d.get(o..)?;
    let n = s.iter().position(|&b| b == 0)?;
    Some(String::from_utf8_lossy(&s[..n]).into_owned())
}

impl<'a> Sdat<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Sdat<'a>> {
        const W: &str = "SDAT";
        if data.len() < 0x40 || &data[..4] != b"SDAT" {
            return Err(bad(W, "bad magic"));
        }
        let blk = |i: usize| -> Result<&'a [u8]> {
            let o = u32_at(data, 0x10 + i * 8, W)? as usize;
            let s = u32_at(data, 0x14 + i * 8, W)? as usize;
            data.get(o..o.saturating_add(s))
                .ok_or_else(|| trunc(W, o, s, data.len()))
        };
        let (symb, info, fat) = (blk(0)?, blk(1)?, blk(2)?);
        if symb.get(..4) != Some(b"SYMB")
            || info.get(..4) != Some(b"INFO")
            || fat.get(..4) != Some(b"FAT ")
        {
            return Err(bad(W, "bad block tag"));
        }
        let n = u32_at(fat, 8, W)? as usize;
        let mut fats = Vec::new();
        for i in 0..n {
            let o = 12 + i * 16;
            let (offset, size) = (u32_at(fat, o, W)?, u32_at(fat, o + 4, W)?);
            if (offset as usize).saturating_add(size as usize) > data.len() {
                return Err(bad(W, format!("FAT entry {i} out of range")));
            }
            fats.push(FatEntry { offset, size });
        }
        let mut tables = Vec::new();
        for (k, kind) in Kind::ALL.iter().enumerate() {
            let rec = u32_at(info, 8 + k * 4, W)? as usize;
            let mut table = Vec::new();
            if rec != 0 {
                let count = u32_at(info, rec, W)? as usize;
                for i in 0..count {
                    let eo = u32_at(info, rec + 4 + i * 4, W)? as usize;
                    table.push(if eo == 0 {
                        None
                    } else {
                        Some(Entry {
                            info: parse_info(info, eo, *kind)?,
                            ..Entry::default()
                        })
                    });
                }
            }
            let srec = u32_at(symb, 8 + k * 4, W)? as usize;
            if srec != 0 {
                let count = u32_at(symb, srec, W)? as usize;
                for i in 0..count.min(table.len()) {
                    let no = u32_at(symb, srec + 4 + i * 4, W)? as usize;
                    let Some(Some(e)) = table.get_mut(i) else {
                        continue;
                    };
                    if no == 0 {
                        continue;
                    }
                    if *kind == Kind::SeqArc {
                        // The name slot points to {name_off, sub_table_off}.
                        e.name = cstr(symb, u32_at(symb, no, W)? as usize);
                        let sub = u32_at(symb, no + 4, W)? as usize;
                        if sub != 0 {
                            for j in 0..u32_at(symb, sub, W)? as usize {
                                let so = u32_at(symb, sub + 4 + j * 4, W)? as usize;
                                e.sub_names
                                    .push(if so == 0 { None } else { cstr(symb, so) });
                            }
                        }
                    } else {
                        e.name = cstr(symb, no);
                    }
                }
            }
            tables.push(table);
        }
        Ok(Sdat {
            data,
            tables,
            fat: fats,
        })
    }

    pub fn entries(&self, kind: Kind) -> &[Option<Entry>] {
        &self.tables[kind.index()]
    }

    /// Number of present (non-hole) entries of a kind.
    pub fn count(&self, kind: Kind) -> usize {
        self.entries(kind).iter().flatten().count()
    }

    pub fn entry(&self, kind: Kind, id: usize) -> Option<&Entry> {
        self.entries(kind).get(id)?.as_ref()
    }

    /// Bytes of embedded file `file_id`.
    pub fn file(&self, file_id: u32) -> Result<&'a [u8]> {
        let f = self
            .fat
            .get(file_id as usize)
            .ok_or_else(|| bad("SDAT", format!("file id {file_id} not in FAT")))?;
        Ok(&self.data[f.offset as usize..(f.offset + f.size) as usize])
    }

    /// The file backing entry `id` of `kind` (SEQ, SEQARC, BANK, WAVEARC, STRM).
    pub fn entry_file(&self, kind: Kind, id: usize) -> Result<&'a [u8]> {
        let e = self
            .entry(kind, id)
            .ok_or_else(|| bad("SDAT", format!("no {} {id}", kind.name())))?;
        self.file(e.info.file_id)
    }
}

fn parse_info(d: &[u8], o: usize, kind: Kind) -> Result<Info> {
    const W: &str = "SDAT INFO";
    let mut i = Info::default();
    match kind {
        Kind::Seq => {
            i.file_id = u32_at(d, o, W)?;
            i.bank = u16_at(d, o + 4, W)?;
            i.volume = u8_at(d, o + 6, W)?;
            i.channel_prio = u8_at(d, o + 7, W)?;
            i.player_prio = u8_at(d, o + 8, W)?;
            i.player = u8_at(d, o + 9, W)?;
        }
        Kind::SeqArc | Kind::WaveArc => {
            i.file_id = u32_at(d, o, W)?;
            i.extra = u32_at(d, o + 4, W)?;
        }
        Kind::Bank => {
            i.file_id = u32_at(d, o, W)?;
            for (j, w) in i.wave_arcs.iter_mut().enumerate() {
                *w = u16_at(d, o + 4 + j * 2, W)?;
            }
        }
        Kind::Player => {
            i.bank = u16_at(d, o, W)?;
            i.channel_mask = u16_at(d, o + 2, W)?;
            i.extra = u32_at(d, o + 4, W)?;
        }
        Kind::Group => {
            for j in 0..u32_at(d, o, W)? as usize {
                i.group_items
                    .push((u32_at(d, o + 4 + j * 8, W)?, u32_at(d, o + 8 + j * 8, W)?));
            }
        }
        Kind::StrmPlayer => {
            let c = u8_at(d, o, W)? as usize;
            for j in 0..c.min(16) {
                i.channels.push(u8_at(d, o + 1 + j, W)?);
            }
        }
        Kind::Strm => {
            i.file_id = u32_at(d, o, W)?;
            i.volume = u8_at(d, o + 4, W)?;
            i.channel_prio = u8_at(d, o + 5, W)?;
            i.player = u8_at(d, o + 6, W)?;
        }
    }
    Ok(i)
}

/// Decoded audio: one `Vec<i16>` per channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcm {
    pub sample_rate: u32,
    pub channels: Vec<Vec<i16>>,
    /// `(loop_start, loop_end)` in samples, if the sound loops.
    pub looping: Option<(usize, usize)>,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        self.channels.first().map_or(0, Vec::len)
    }

    /// Interleaved little-endian bytes of all channels (for hashing).
    pub fn interleaved_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.frames() * self.channels.len() * 2);
        for f in 0..self.frames() {
            for c in &self.channels {
                out.extend_from_slice(&c[f].to_le_bytes());
            }
        }
        out
    }

    /// 16-bit PCM WAV file (RIFF).
    pub fn to_wav(&self) -> Vec<u8> {
        let data = self.interleaved_bytes();
        let ch = self.channels.len() as u16;
        let mut w = Vec::with_capacity(44 + data.len());
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&ch.to_le_bytes());
        w.extend_from_slice(&self.sample_rate.to_le_bytes());
        w.extend_from_slice(&(self.sample_rate * u32::from(ch) * 2).to_le_bytes());
        w.extend_from_slice(&(ch * 2).to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(data.len() as u32).to_le_bytes());
        w.extend_from_slice(&data);
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header() {
        let p = Pcm {
            sample_rate: 8000,
            channels: vec![vec![1, -2]],
            looping: None,
        };
        let w = p.to_wav();
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(w.len(), 44 + 4);
        assert_eq!(&w[44..], &[1, 0, 0xFE, 0xFF]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Sdat::parse(&[0; 100]).is_err());
        assert!(Sdat::parse(b"SDAT").is_err());
    }
}
