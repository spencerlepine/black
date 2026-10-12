//! STRM: streamed audio.
//!
//! File: `STRM` Nitro header, `HEAD` block (`u8 wave_type, u8 loop, u8 channels, u8 pad,
//! u16 sample_rate, u16 time, u32 loop_offset (samples), u32 samples, u32 data_offset,
//! u32 blocks, u32 block_size, u32 samples_per_block, u32 last_block_size, u32 last_block_samples,
//! u32 last_block_padded`), `DATA` block of interleaved blocks (per block, `block_size` bytes for
//! each channel in turn). ADPCM blocks each carry their own 4-byte header.

use super::{adpcm, bad, find_block, u16_at, u32_at, u8_at, Pcm};
use crate::error::Result;

const W: &str = "STRM";

#[derive(Debug, Clone)]
pub struct Strm<'a> {
    pub wave_type: u8,
    pub looping: bool,
    pub channels: usize,
    pub sample_rate: u16,
    pub loop_offset: u32,
    pub samples: u32,
    pub blocks: u32,
    pub block_size: u32,
    pub samples_per_block: u32,
    pub last_block_size: u32,
    pub last_block_samples: u32,
    pub last_block_padded: u32,
    data: &'a [u8],
}

impl<'a> Strm<'a> {
    pub fn parse(f: &'a [u8]) -> Result<Strm<'a>> {
        let head = find_block(f, b"STRM", b"HEAD", W)?;
        let data = find_block(f, b"STRM", b"DATA", W)?;
        let h = |o: usize| u32_at(head, 8 + o, W);
        let s = Strm {
            wave_type: u8_at(head, 8, W)?,
            looping: u8_at(head, 9, W)? != 0,
            channels: usize::from(u8_at(head, 10, W)?),
            sample_rate: u16_at(head, 12, W)?,
            loop_offset: h(8)?,
            samples: h(12)?,
            blocks: h(20)?,
            block_size: h(24)?,
            samples_per_block: h(28)?,
            last_block_size: h(32)?,
            last_block_samples: h(36)?,
            last_block_padded: h(40).unwrap_or(0),
            data: &data[8..],
        };
        if s.wave_type > 2 || s.channels == 0 || s.channels > 2 {
            return Err(bad(
                W,
                format!("wave type {} channels {}", s.wave_type, s.channels),
            ));
        }
        Ok(s)
    }

    pub fn decode(&self) -> Result<Pcm> {
        let mut chans: Vec<Vec<i16>> = vec![Vec::new(); self.channels];
        let mut pos = 0usize;
        for b in 0..self.blocks {
            let last = b + 1 == self.blocks;
            let (size, want) = if last {
                (self.last_block_size, self.last_block_samples)
            } else {
                (self.block_size, self.samples_per_block)
            };
            for ch in chans.iter_mut() {
                let chunk = self
                    .data
                    .get(pos..pos + size as usize)
                    .ok_or_else(|| bad(W, format!("block {b} runs past the data")))?;
                // Padded size of the last block is the stride between channels.
                pos += if last && self.last_block_padded != 0 {
                    self.last_block_padded as usize
                } else {
                    size as usize
                };
                let mut s: Vec<i16> = match self.wave_type {
                    0 => chunk.iter().map(|&x| i16::from(x as i8) << 8).collect(),
                    1 => super::le_i16s(chunk),
                    _ => adpcm::decode_block(chunk),
                };
                s.truncate(want as usize);
                ch.extend(s);
            }
        }
        let n = self.samples as usize;
        for c in &mut chans {
            c.truncate(n);
        }
        Ok(Pcm {
            sample_rate: u32::from(self.sample_rate),
            looping: self
                .looping
                .then_some((self.loop_offset as usize, chans[0].len())),
            channels: chans,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(wave_type: u8, ch: u8, blocks: &[Vec<u8>], last_samples: u32) -> Vec<u8> {
        let bs = blocks[0].len() as u32;
        let mut f = b"STRM".to_vec();
        f.extend([0xFF, 0xFE, 0, 1, 0, 0, 0, 0, 16, 0, 2, 0]);
        f.extend(b"HEAD");
        f.extend(0x58u32.to_le_bytes());
        f.extend([wave_type, 1, ch, 0, 0x40, 0x1F, 0, 0]);
        f.extend(0u32.to_le_bytes()); // loop offset
        let nb = blocks.len() as u32 / u32::from(ch);
        f.extend((last_samples + 2 * (nb - 1)).to_le_bytes());
        f.extend(0x68u32.to_le_bytes());
        f.extend(nb.to_le_bytes());
        f.extend(bs.to_le_bytes());
        f.extend(2u32.to_le_bytes());
        f.extend(bs.to_le_bytes());
        f.extend(last_samples.to_le_bytes());
        f.resize(0x68, 0);
        f.extend(b"DATA");
        f.extend(0x10000u32.to_le_bytes());
        for b in blocks {
            f.extend(b);
        }
        f
    }

    #[test]
    fn pcm16_stereo() {
        // Two blocks (1 per channel pair each... blocks=1, 2 channels): L then R.
        let f = file(1, 2, &[vec![1, 0, 2, 0], vec![3, 0, 4, 0]], 2);
        let p = Strm::parse(&f).unwrap().decode().unwrap();
        assert_eq!(p.channels, vec![vec![1, 2], vec![3, 4]]);
        assert_eq!(p.looping, Some((0, 2)));
    }

    #[test]
    fn rejects_truncated() {
        let f = file(1, 1, &[vec![1, 0, 2, 0]], 2);
        assert!(Strm::parse(&f).unwrap().decode().is_ok());
        assert!(Strm::parse(&f[..f.len() - 2]).unwrap().decode().is_err());
    }
}
