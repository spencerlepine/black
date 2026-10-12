//! Shared parsing of the generic Nitro file container used by NCLR/NCGR/NSCR/NCER/NANR.
//!
//! Layout: `magic[4] bom:u16(0xFEFF) version:u16 size:u32 header_size:u16 nblocks:u16`, then
//! blocks `tag[4] size:u32 data...` (size includes the 8-byte block header). Magics and tags are
//! stored byte-reversed on disk (`RLCN` for NCLR, `TTLP` for PLTT, ...).

use crate::error::{Error, Result};

pub(crate) fn u16_at(d: &[u8], o: usize) -> Result<u16> {
    d.get(o..o + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| trunc(o, 2, d.len()))
}

pub(crate) fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| trunc(o, 4, d.len()))
}

/// Offsets inside blocks are relative to the payload (just after the 8-byte block header).
pub(crate) fn payload_off(v: u32, _what: &'static str) -> Result<usize> {
    Ok(v as usize)
}

fn trunc(offset: usize, need: usize, have: usize) -> Error {
    Error::Truncated {
        what: "Nitro graphics file",
        offset,
        need,
        have,
    }
}

/// Parsed container: the file's blocks as `(tag, payload-after-8-byte-header)`.
pub(crate) struct Container<'a> {
    pub blocks: Vec<([u8; 4], &'a [u8])>,
}

impl<'a> Container<'a> {
    pub fn parse(data: &'a [u8], magic: &[u8; 4], what: &'static str) -> Result<Self> {
        if data.len() < 16 || &data[..4] != magic {
            return Err(Error::invalid(what, "bad magic"));
        }
        let header = u16_at(data, 12)? as usize;
        let n = u16_at(data, 14)? as usize;
        let mut pos = header;
        let mut blocks = Vec::new();
        for _ in 0..n {
            let tag: [u8; 4] = data
                .get(pos..pos + 4)
                .ok_or_else(|| trunc(pos, 8, data.len()))?
                .try_into()
                .unwrap();
            let size = u32_at(data, pos + 4)? as usize;
            if size < 8 {
                return Err(Error::invalid(what, "block smaller than its header"));
            }
            // Some ROM files declare a last block longer than the file; clamp (lenient).
            let end = (pos + size).min(data.len());
            let payload = data
                .get(pos + 8..end)
                .ok_or_else(|| trunc(pos, size, data.len()))?;
            blocks.push((tag, payload));
            pos += size;
        }
        Ok(Container { blocks })
    }

    pub fn block(&self, tag: &[u8; 4], what: &'static str) -> Result<&'a [u8]> {
        self.blocks
            .iter()
            .find(|(t, _)| t == tag)
            .map(|(_, p)| *p)
            .ok_or_else(|| Error::invalid(what, "required block missing"))
    }
}
