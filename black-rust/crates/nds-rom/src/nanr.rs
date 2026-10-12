//! NANR animation banks (`RNAN` / `KNBA`): sequences of frames that select cells.
//!
//! Layout (offsets are relative to the block payload): 24-byte header
//! `nseq:u16 nframes:u16 seq_off:u32 frame_off:u32 data_off:u32 +8 reserved`; sequences are
//! 16 bytes `count:u16 pad:u16 kind:u16 map:u16 mode:u32 first_frame_byte_off:u32`; frames are
//! 8 bytes `data_off:u32 duration:u16 0xBEEF`; frame data begins with `cell:u16`.

use crate::error::{Error, Result};
use crate::nitro::{u16_at, u32_at, Container};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimFrame {
    /// Cell index (first u16 of the frame data).
    pub cell: u16,
    /// Duration in video frames.
    pub duration: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    /// Element type (0 = cell index only; other values add transform data that is not decoded,
    /// UNVERIFIED because the ROM's files only use 0).
    pub kind: u16,
    pub mapping: u16,
    /// Play mode (1 forward, 2 forward loop, 3 reverse, 4 reverse loop).
    pub mode: u32,
    pub frames: Vec<AnimFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nanr {
    pub sequences: Vec<Sequence>,
}

impl Nanr {
    pub fn parse(data: &[u8]) -> Result<Nanr> {
        let c = Container::parse(data, b"RNAN", "NANR")?;
        let b = c.block(b"KNBA", "NANR")?;
        let n = u16_at(b, 0)? as usize;
        let seq_off = u32_at(b, 4)? as usize;
        let frame_off = u32_at(b, 8)? as usize;
        let data_off = u32_at(b, 12)? as usize;
        let mut sequences = Vec::with_capacity(n);
        for i in 0..n {
            let s = seq_off + i * 16;
            let count = u16_at(b, s)? as usize;
            let kind = u16_at(b, s + 4)?;
            let mapping = u16_at(b, s + 6)?;
            let mode = u32_at(b, s + 8)?;
            let first = u32_at(b, s + 12)? as usize;
            let mut frames = Vec::with_capacity(count);
            for j in 0..count {
                let f = frame_off + first + j * 8;
                let d = data_off + u32_at(b, f)? as usize;
                let cell = u16_at(b, d)?;
                let duration = u16_at(b, f + 4)?;
                frames.push(AnimFrame { cell, duration });
            }
            sequences.push(Sequence {
                kind,
                mapping,
                mode,
                frames,
            });
        }
        if sequences.is_empty() && n != 0 {
            return Err(Error::invalid("NANR", "no sequences"));
        }
        Ok(Nanr { sequences })
    }
}
