//! SSAR: sequence archive (many short sequences sharing one event stream, used for sound effects).
//!
//! File: `SSAR` Nitro header, `DATA` block (`tag size`, `u32 data_offset`, `u32 count`, then
//! `count` records of `u32 offset, u16 bank, u8 volume, u8 channel_prio, u8 player_prio,
//! u8 player, u16 reserved`). `offset` is relative to `data_offset` (file-relative), the same base
//! the sequences' jump/call/open-track offsets use (UNVERIFIED: checked only by every sequence
//! decoding without leaving the stream).

use super::sseq::{decode, Sequence};
use super::{bad, find_block, u16_at, u32_at, u8_at};
use crate::error::Result;

const W: &str = "SSAR";

#[derive(Debug, Clone)]
pub struct SsarSeq {
    pub offset: u32,
    pub bank: u16,
    pub volume: u8,
    pub channel_prio: u8,
    pub player_prio: u8,
    pub player: u8,
    pub sequence: Sequence,
}

#[derive(Debug, Clone)]
pub struct Ssar {
    pub sequences: Vec<SsarSeq>,
}

impl Ssar {
    pub fn parse(f: &[u8]) -> Result<Ssar> {
        find_block(f, b"SSAR", b"DATA", W)?;
        let start = u16_at(f, 12, W)? as usize;
        let data_off = u32_at(f, start + 8, W)? as usize;
        let count = u32_at(f, start + 12, W)? as usize;
        let data = f
            .get(data_off..)
            .ok_or_else(|| bad(W, "data offset past end"))?;
        let mut sequences = Vec::new();
        for i in 0..count {
            let r = start + 16 + i * 12;
            let offset = u32_at(f, r, W)?;
            sequences.push(SsarSeq {
                offset,
                bank: u16_at(f, r + 4, W)?,
                volume: u8_at(f, r + 6, W)?,
                channel_prio: u8_at(f, r + 7, W)?,
                player_prio: u8_at(f, r + 8, W)?,
                player: u8_at(f, r + 9, W)?,
                sequence: decode(data, offset)?,
            });
        }
        Ok(Ssar { sequences })
    }

    pub fn event_count(&self) -> usize {
        self.sequences.iter().map(|s| s.sequence.events.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic() {
        let mut f = b"SSAR".to_vec();
        f.extend([0xFF, 0xFE, 0, 1, 0, 0, 0, 0, 16, 0, 1, 0]);
        f.extend(b"DATA");
        f.extend(0x10000u32.to_le_bytes());
        f.extend((16u32 + 8 + 8 + 12 * 2).to_le_bytes());
        f.extend(2u32.to_le_bytes());
        for (off, bank) in [(0u32, 3u16), (3, 4)] {
            f.extend(off.to_le_bytes());
            f.extend(bank.to_le_bytes());
            f.extend([100, 1, 2, 0, 0, 0]);
        }
        f.extend([0x3C, 100, 0x10, 0xFF, 0xFF, 0xFF][..].iter().copied()); // note+end, end
        let s = Ssar::parse(&f).unwrap();
        assert_eq!(s.sequences.len(), 2);
        assert_eq!(s.sequences[1].bank, 4);
        assert_eq!(s.event_count(), 3);
    }
}
