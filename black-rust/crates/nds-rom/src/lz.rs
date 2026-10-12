//! Nintendo LZ77 decompression: LZ10 (type byte `0x10`) and LZ11 (type byte `0x11`).
//!
//! Both start with a 4-byte header: the type byte and the 24-bit decompressed size (LE). A size of
//! 0 is followed by a 32-bit size. Data is a sequence of 8-block groups led by a flag byte, MSB
//! first; a clear bit is a literal byte, a set bit is a back-reference.

use crate::error::{Error, Result};

/// Which LZ variant a stream uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LzKind {
    Lz10,
    Lz11,
}

fn bad(detail: impl Into<String>) -> Error {
    Error::invalid("LZ stream", detail)
}

/// Parse the header: kind, decompressed size and the offset of the first data byte.
fn header(data: &[u8]) -> Result<(LzKind, usize, usize)> {
    let kind = match data.first() {
        Some(0x10) => LzKind::Lz10,
        Some(0x11) => LzKind::Lz11,
        _ => return Err(bad("not an LZ10/LZ11 type byte")),
    };
    let word = data.get(1..4).ok_or_else(|| bad("truncated header"))?;
    let mut size = u32::from_le_bytes([word[0], word[1], word[2], 0]) as usize;
    let mut pos = 4;
    if size == 0 {
        let ext = data
            .get(4..8)
            .ok_or_else(|| bad("truncated extended size"))?;
        size = u32::from_le_bytes([ext[0], ext[1], ext[2], ext[3]]) as usize;
        pos = 8;
    }
    Ok((kind, size, pos))
}

/// Decompress an LZ10 or LZ11 stream, returning the data and the number of input bytes consumed.
fn decode(data: &[u8]) -> Result<(Vec<u8>, usize)> {
    let (kind, size, mut pos) = header(data)?;
    // Both formats expand by well under 1000:1 per input byte, so this bounds hostile sizes.
    let mut out: Vec<u8> = Vec::with_capacity(size.min(data.len().saturating_mul(1000)));
    let next = |pos: &mut usize| -> Result<usize> {
        let b = *data.get(*pos).ok_or_else(|| bad("truncated data"))?;
        *pos += 1;
        Ok(b as usize)
    };
    while out.len() < size {
        let flags = next(&mut pos)?;
        for bit in (0..8).rev() {
            if out.len() >= size {
                break;
            }
            if flags >> bit & 1 == 0 {
                out.push(next(&mut pos)? as u8);
                continue;
            }
            let b0 = next(&mut pos)?;
            let b1 = next(&mut pos)?;
            let (len, disp) = match kind {
                LzKind::Lz10 => ((b0 >> 4) + 3, ((b0 & 0xF) << 8 | b1) + 1),
                LzKind::Lz11 => match b0 >> 4 {
                    0 => {
                        let b2 = next(&mut pos)?;
                        (
                            ((b0 & 0xF) << 4 | b1 >> 4) + 0x11,
                            ((b1 & 0xF) << 8 | b2) + 1,
                        )
                    }
                    1 => {
                        let b2 = next(&mut pos)?;
                        let b3 = next(&mut pos)?;
                        (
                            ((b0 & 0xF) << 12 | b1 << 4 | b2 >> 4) + 0x111,
                            ((b2 & 0xF) << 8 | b3) + 1,
                        )
                    }
                    n => (n + 1, ((b0 & 0xF) << 8 | b1) + 1),
                },
            };
            if disp > out.len() {
                return Err(bad("back-reference before start of output"));
            }
            let from = out.len() - disp;
            for i in 0..len.min(size - out.len()) {
                out.push(out[from + i]);
            }
        }
    }
    Ok((out, pos))
}

/// Decompress an LZ10 or LZ11 stream. Trailing bytes after the stream are ignored.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    decode(data).map(|(out, _)| out)
}

/// Which variant `data` is, judged by the header alone.
pub fn kind(data: &[u8]) -> Option<LzKind> {
    header(data).ok().map(|(k, _, _)| k)
}

/// Decompress `data` if it is a well-formed LZ10/LZ11 stream, else `None`.
///
/// A stream counts as well-formed when it decodes completely and at most 3 bytes of
/// alignment padding follow it.
pub fn decompress_any(data: &[u8]) -> Option<Vec<u8>> {
    let (out, used) = decode(data).ok()?;
    (data.len() - used < 4).then_some(out)
}

/// Compress as LZ10 (greedy longest match). Meant for tests and tools, not speed.
pub fn compress_lz10(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() > 0xFF_FFFF {
        return Err(bad("input too large for LZ10"));
    }
    let mut out = vec![0x10];
    out.extend_from_slice(&(data.len() as u32).to_le_bytes()[..3]);
    if data.is_empty() {
        out.extend_from_slice(&[0; 4]);
    }
    let mut pos = 0;
    while pos < data.len() {
        let flag_at = out.len();
        out.push(0);
        for bit in (0..8).rev() {
            if pos >= data.len() {
                break;
            }
            let (mut best_len, mut best_disp) = (0, 0);
            for disp in 1..=pos.min(0x1000) {
                let max = (data.len() - pos).min(18);
                let len = (0..max)
                    .take_while(|&i| data[pos + i] == data[pos - disp + i])
                    .count();
                if len > best_len {
                    (best_len, best_disp) = (len, disp);
                }
            }
            if best_len >= 3 {
                out[flag_at] |= 1 << bit;
                let d = best_disp - 1;
                out.push(((best_len - 3) << 4 | d >> 8) as u8);
                out.push(d as u8);
                pos += best_len;
            } else {
                out.push(data[pos]);
                pos += 1;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
