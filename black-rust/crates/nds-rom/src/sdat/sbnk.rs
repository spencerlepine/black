//! SBNK: instrument bank.
//!
//! File: `SBNK` Nitro header, `DATA` block (`tag size`, 32 reserved bytes, `u32 count`, then
//! `count` records of `u8 type, u16 offset, u8 reserved`; offsets are file-relative).
//! Record type 0 = empty, 1..=15 = single region (1 PCM, 2 PSG pulse, 3 PSG noise), 16 = drum set
//! (`lo, hi`, one region per key), 17 = key split (up to 8 ascending end keys, one region each).
//! A region is `swav u16, swar u16, note u8, attack, decay, sustain, release, pan` (for PSG the
//! `swav` field is the duty cycle); inside drum sets and splits a `u16` type precedes it.

use super::{bad, find_block, u16_at, u32_at, u8_at};
use crate::error::Result;

const W: &str = "SBNK";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    /// 1 = PCM (SWAV), 2 = PSG pulse, 3 = PSG noise, others unused by this ROM.
    pub kind: u16,
    pub swav: u16,
    /// Index into the bank's four wave-archive slots.
    pub swar: u16,
    pub note: u8,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
    pub pan: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instrument {
    Empty,
    Single(Region),
    Drum {
        lo: u8,
        hi: u8,
        regions: Vec<Region>,
    },
    Split {
        ends: Vec<u8>,
        regions: Vec<Region>,
    },
}

#[derive(Debug, Clone)]
pub struct Sbnk {
    pub instruments: Vec<Instrument>,
}

fn region(f: &[u8], o: usize, kind: u16) -> Result<Region> {
    Ok(Region {
        kind,
        swav: u16_at(f, o, W)?,
        swar: u16_at(f, o + 2, W)?,
        note: u8_at(f, o + 4, W)?,
        attack: u8_at(f, o + 5, W)?,
        decay: u8_at(f, o + 6, W)?,
        sustain: u8_at(f, o + 7, W)?,
        release: u8_at(f, o + 8, W)?,
        pan: u8_at(f, o + 9, W)?,
    })
}

impl Sbnk {
    pub fn parse(f: &[u8]) -> Result<Sbnk> {
        find_block(f, b"SBNK", b"DATA", W)?;
        let start = u16_at(f, 12, W)? as usize;
        let count = u32_at(f, start + 0x28, W)? as usize;
        let mut instruments = Vec::new();
        for i in 0..count {
            let r = start + 0x2C + i * 4;
            let ty = u8_at(f, r, W)?;
            let off = u16_at(f, r + 1, W)? as usize;
            instruments.push(match ty {
                0 => Instrument::Empty,
                1..=15 => Instrument::Single(region(f, off, u16::from(ty))?),
                16 => {
                    let (lo, hi) = (u8_at(f, off, W)?, u8_at(f, off + 1, W)?);
                    if hi < lo {
                        return Err(bad(W, format!("drum set {i}: hi < lo")));
                    }
                    let mut regions = Vec::new();
                    for k in 0..=usize::from(hi - lo) {
                        let o = off + 2 + k * 12;
                        regions.push(region(f, o + 2, u16_at(f, o, W)?)?);
                    }
                    Instrument::Drum { lo, hi, regions }
                }
                17 => {
                    let mut ends = Vec::new();
                    for k in 0..8 {
                        let e = u8_at(f, off + k, W)?;
                        if e == 0 {
                            break;
                        }
                        ends.push(e);
                    }
                    let mut regions = Vec::new();
                    for k in 0..ends.len() {
                        let o = off + 8 + k * 12;
                        regions.push(region(f, o + 2, u16_at(f, o, W)?)?);
                    }
                    Instrument::Split { ends, regions }
                }
                t => return Err(bad(W, format!("instrument {i}: record type {t}"))),
            });
        }
        Ok(Sbnk { instruments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic() {
        let mut f = b"SBNK".to_vec();
        f.extend([0xFF, 0xFE, 0, 1, 0, 0, 0, 0, 16, 0, 1, 0]);
        f.extend(b"DATA");
        f.extend(0x10000u32.to_le_bytes());
        f.extend([0; 32]);
        f.extend(2u32.to_le_bytes());
        // record table at 0x2C+16... computed: records start at 16 + 0x2C = 60
        let base = 16 + 0x2C + 8;
        f.extend([1]);
        f.extend((base as u16).to_le_bytes());
        f.extend([0]);
        f.extend([0, 0, 0, 0]);
        f.extend([5, 0, 1, 0, 60, 255, 1, 127, 2, 64]);
        let b = Sbnk::parse(&f).unwrap();
        assert_eq!(b.instruments.len(), 2);
        assert_eq!(b.instruments[1], Instrument::Empty);
        let Instrument::Single(r) = &b.instruments[0] else {
            panic!()
        };
        assert_eq!((r.swav, r.swar, r.note, r.pan, r.kind), (5, 1, 60, 64, 1));
        assert!(Sbnk::parse(&f[..68]).is_err());
    }
}
