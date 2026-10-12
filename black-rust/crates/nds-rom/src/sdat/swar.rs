//! SWAR: wave archive of SWAV samples.
//!
//! File: `SWAR` Nitro header, `DATA` block (`tag size`, 32 reserved bytes, `u32 count`, then
//! `count` file-relative `u32` offsets). Each SWAV: `u8 wave_type` (0 PCM8, 1 PCM16, 2 IMA-ADPCM),
//! `u8 loop_flag`, `u16 sample_rate`, `u16 time` (timer reload), `u16 loop_offset` (in 32-bit
//! words, includes the ADPCM header word), `u32 non_loop_len` (words), then the sample data
//! of `(loop_offset + non_loop_len) * 4` bytes.

use super::{adpcm, bad, find_block, u16_at, u32_at, u8_at, Pcm};
use crate::error::{Error, Result};

const W: &str = "SWAR";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveType {
    Pcm8,
    Pcm16,
    Adpcm,
}

#[derive(Debug, Clone)]
pub struct Swav {
    pub wave_type: WaveType,
    pub looping: bool,
    pub sample_rate: u16,
    pub time: u16,
    pub loop_offset: u16,
    pub non_loop_len: u32,
    pub data: Vec<u8>,
}

impl Swav {
    /// Parse the SWAV at file offset `o` of `f`.
    pub fn parse_at(f: &[u8], o: usize) -> Result<Swav> {
        let wave_type = match u8_at(f, o, W)? {
            0 => WaveType::Pcm8,
            1 => WaveType::Pcm16,
            2 => WaveType::Adpcm,
            t => return Err(bad(W, format!("wave type {t}"))),
        };
        let loop_offset = u16_at(f, o + 6, W)?;
        let non_loop_len = u32_at(f, o + 8, W)?;
        let len = (usize::from(loop_offset) + non_loop_len as usize) * 4;
        let data = f
            .get(o + 12..o + 12 + len)
            .ok_or(Error::Truncated {
                what: W,
                offset: o + 12,
                need: len,
                have: f.len().saturating_sub(o + 12),
            })?
            .to_vec();
        Ok(Swav {
            wave_type,
            looping: u8_at(f, o + 1, W)? != 0,
            sample_rate: u16_at(f, o + 2, W)?,
            time: u16_at(f, o + 4, W)?,
            loop_offset,
            non_loop_len,
            data,
        })
    }

    /// Decode to mono 16-bit samples with loop points in samples.
    pub fn decode(&self) -> Result<Pcm> {
        let lo = usize::from(self.loop_offset);
        let (samples, loop_start) = match self.wave_type {
            WaveType::Pcm8 => (
                self.data
                    .iter()
                    .map(|&b| i16::from(b as i8) << 8)
                    .collect::<Vec<_>>(),
                lo * 4,
            ),
            WaveType::Pcm16 => (super::le_i16s(&self.data), lo * 2),
            WaveType::Adpcm => {
                if self.data.len() < 4 {
                    return Err(bad(W, "ADPCM wave without header"));
                }
                (adpcm::decode_block(&self.data), lo.saturating_sub(1) * 8)
            }
        };
        let looping = self
            .looping
            .then_some((loop_start.min(samples.len()), samples.len()));
        Ok(Pcm {
            sample_rate: u32::from(self.sample_rate),
            channels: vec![samples],
            looping,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Swar {
    pub waves: Vec<Swav>,
}

impl Swar {
    pub fn parse(f: &[u8]) -> Result<Swar> {
        find_block(f, b"SWAR", b"DATA", W)?;
        let start = u16_at(f, 12, W)? as usize;
        let count = u32_at(f, start + 0x28, W)? as usize;
        let mut waves = Vec::new();
        for i in 0..count {
            let off = u32_at(f, start + 0x2C + i * 4, W)? as usize;
            waves.push(Swav::parse_at(f, off)?);
        }
        Ok(Swar { waves })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(wave_type: u8, loop_flag: u8, loop_off: u16, data: &[u8]) -> Vec<u8> {
        let mut f = b"SWAR".to_vec();
        f.extend([0xFF, 0xFE, 0, 1, 0, 0, 0, 0, 16, 0, 1, 0]);
        f.extend(b"DATA");
        f.extend(0x10000u32.to_le_bytes());
        f.extend([0; 32]);
        f.extend(1u32.to_le_bytes());
        f.extend(64u32.to_le_bytes());
        f.extend([wave_type, loop_flag, 0x40, 0x1F, 0, 0]);
        f.extend(loop_off.to_le_bytes());
        f.extend(((data.len() / 4) as u32 - u32::from(loop_off)).to_le_bytes());
        f.extend(data);
        f
    }

    #[test]
    fn pcm8_pcm16() {
        let a = Swar::parse(&archive(0, 1, 1, &[1, 0xFF, 2, 3, 4, 5, 6, 7])).unwrap();
        let p = a.waves[0].decode().unwrap();
        assert_eq!(p.channels[0][..2], [256, -256]);
        assert_eq!(p.looping, Some((4, 8)));
        assert_eq!(p.sample_rate, 8000);
        let a = Swar::parse(&archive(1, 0, 0, &[0, 0x80, 0xFF, 0x7F])).unwrap();
        let p = a.waves[0].decode().unwrap();
        assert_eq!(p.channels[0], vec![-32768, 32767]);
        assert_eq!(p.looping, None);
    }

    #[test]
    fn adpcm_wave_and_truncation() {
        let a = archive(2, 1, 1, &[100, 0, 0, 0, 0x70, 0xF0, 0, 0]);
        let p = Swar::parse(&a).unwrap().waves[0].decode().unwrap();
        assert_eq!(p.channels[0][..4], [100, 111, 113, 88]);
        assert_eq!(p.looping, Some((0, 8)));
        assert!(Swar::parse(&a[..a.len() - 2]).is_err());
    }
}
