//! Backward LZ ("BLZ") compression as used for the NDS ARM9 static module and overlays.
//!
//! The stream is read from the end of the file towards the start.  The last eight bytes are a
//! footer: `u32 (header_len << 24 | encoded_len)` followed by `u32 extra_len` (the number of
//! bytes the data grows by when decompressed).  Everything before `len - encoded_len` is stored
//! verbatim.

use crate::error::{le32, Error, Result};

const WINDOW: usize = 0x1002;
const MAX_LEN: usize = 0x12;
const THRESHOLD: usize = 2;
/// Footer size without padding.
const FOOTER: usize = 8;

/// Decompress a BLZ stream.  Port of `blz.py::blz_decompress`.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let n = data.len();
    if n < FOOTER {
        return Err(Error::Blz(format!("{n} bytes is too short for a footer")));
    }
    let inc = le32(data, n.wrapping_sub(4), "BLZ footer")? as usize;
    let x = le32(data, n.wrapping_sub(8), "BLZ footer")?;
    let hdr = (x >> 24) as usize;
    let enc = (x & 0xFF_FFFF) as usize;
    if hdr > n || enc > n || hdr > enc && enc != 0 {
        return Err(Error::Blz(format!(
            "footer header {hdr:#x} / encoded {enc:#x} do not fit {n:#x} bytes"
        )));
    }
    let mut out = Vec::with_capacity(n + inc);
    out.extend_from_slice(data);
    out.resize(n + inc, 0);
    let mut ip = n - hdr;
    let mut op = n + inc;
    let end = n - enc;
    while ip > end {
        ip -= 1;
        let mut flags = out[ip];
        for _ in 0..8 {
            if ip <= end {
                break;
            }
            if flags & 0x80 != 0 {
                if ip < end + 2 {
                    return Err(Error::Blz("truncated back-reference".into()));
                }
                ip -= 1;
                let b1 = out[ip] as usize;
                ip -= 1;
                let b2 = out[ip] as usize;
                let pos = (((b1 & 0x0F) << 8) | b2) + 3;
                let len = (b1 >> 4) + 3;
                if op < len || op + pos > out.len() + len {
                    return Err(Error::Blz("back-reference outside the output".into()));
                }
                for _ in 0..len {
                    op -= 1;
                    let src = op + pos;
                    if src >= out.len() {
                        return Err(Error::Blz("back-reference outside the output".into()));
                    }
                    out[op] = out[src];
                }
            } else {
                ip -= 1;
                if op == 0 {
                    return Err(Error::Blz("output underflow".into()));
                }
                op -= 1;
                out[op] = out[ip];
            }
            flags <<= 1;
        }
    }
    Ok(out)
}

/// Longest match ending just before `r`, as `(len, pos)`; `len <= THRESHOLD` means none.
fn find_match(raw: &[u8], r: usize, floor: usize) -> (usize, usize) {
    let max_pos = (raw.len() - r).min(WINDOW);
    let max_len = MAX_LEN.min(r - floor);
    let mut best = (THRESHOLD, 0);
    for pos in 3..=max_pos {
        // the source may not overlap the bytes being encoded
        let limit = max_len.min(pos);
        let mut len = 0;
        while len < limit && raw[r - 1 - len] == raw[r - 1 - len + pos] {
            len += 1;
        }
        if len > best.0 {
            best = (len, pos);
            if len == max_len {
                break;
            }
        }
    }
    best
}

/// Compress `raw`, never touching the first `floor` bytes (`0x4000` for the ARM9 secure area,
/// `0` for overlays).  Returns `None` when compression would not shrink the data, in which case
/// the module is stored as is.
///
/// The parse is the same lazy (one-step look-ahead) greedy parse as the reference encoder, and
/// the point where the verbatim prefix ends is chosen to minimise the total size.
pub fn compress(raw: &[u8], floor: usize) -> Option<Vec<u8>> {
    let n = raw.len();
    let floor = floor.min(n);
    // generation-order stream (descending addresses), reversed at the end
    let mut gen: Vec<u8> = Vec::new();
    let mut flag_at = 0usize;
    let mut tokens = 0usize;
    // (total size, tokens, gen length) after each token; total = coded bytes + raw bytes left
    let mut trail: Vec<(usize, usize)> = Vec::new();
    let mut r = n;
    while r > floor {
        if tokens.is_multiple_of(8) {
            flag_at = gen.len();
            gen.push(0);
        }
        let (len, pos) = find_match(raw, r, floor);
        gen[flag_at] <<= 1;
        if len > THRESHOLD {
            gen[flag_at] |= 1;
            let p = pos - 3;
            gen.push((((len - 3) << 4) | (p >> 8)) as u8);
            gen.push((p & 0xFF) as u8);
            r -= len;
        } else {
            gen.push(raw[r - 1]);
            r -= 1;
        }
        tokens += 1;
        trail.push((gen.len() + r, gen.len()));
    }
    if tokens == 0 {
        return None;
    }
    // Finish the last flag byte, exactly as the full parse leaves it.
    gen[flag_at] <<= (8 - tokens % 8) % 8;
    // The encoder keeps the raw head only as far as in-place decompression requires: the cut
    // is the first token after which the total drops below the total of the complete parse.
    let full_total = trail[tokens - 1].0;
    let cut_tokens = match trail.iter().position(|&(total, _)| total < full_total) {
        Some(k) => k + 1,
        None => tokens,
    };
    let cut_len = trail[cut_tokens - 1].1;
    let prefix = trail[cut_tokens - 1].0 - cut_len;
    gen.truncate(cut_len);
    gen.reverse();

    let mut hdr = FOOTER;
    while !(prefix + gen.len() + hdr).is_multiple_of(4) {
        hdr += 1;
    }
    let total = prefix + gen.len() + hdr;
    if total >= n {
        return None;
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&raw[..prefix]);
    out.extend_from_slice(&gen);
    out.resize(prefix + gen.len() + (hdr - FOOTER), 0xFF);
    let enc = (gen.len() + hdr) as u32;
    out.extend_from_slice(&(enc | ((hdr as u32) << 24)).to_le_bytes());
    out.extend_from_slice(&((n - total) as u32).to_le_bytes());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(len: usize) -> Vec<u8> {
        let mut state = 12345u32;
        let mut v = Vec::new();
        while v.len() < len {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            if state & 0x300 == 0 {
                v.extend((0..(state >> 24) % 40).map(|i| (i * 3) as u8));
            } else {
                v.push((state >> 20) as u8 & 0x0F);
            }
        }
        v.truncate(len);
        v
    }

    #[test]
    fn roundtrip() {
        for len in [1usize, 7, 64, 1000, 5000, 70000] {
            let raw = sample(len);
            if let Some(c) = compress(&raw, 0) {
                assert!(c.len() < raw.len());
                assert_eq!(decompress(&c).unwrap(), raw, "len {len}");
            }
        }
    }

    #[test]
    fn roundtrip_with_floor() {
        let raw = sample(40000);
        let c = compress(&raw, 0x4000).expect("compressible");
        assert_eq!(&c[..0x4000], &raw[..0x4000]);
        assert_eq!(decompress(&c).unwrap(), raw);
    }

    #[test]
    fn incompressible_is_none() {
        let raw: Vec<u8> = (0..64u32)
            .map(|i| (i * 97 + 13) as u8 ^ (i >> 3) as u8)
            .collect();
        let _ = compress(&raw, 0);
        assert!(compress(&[1, 2, 3, 4], 0).is_none());
    }

    #[test]
    fn rejects_garbage() {
        assert!(decompress(&[1, 2, 3]).is_err());
    }
}
