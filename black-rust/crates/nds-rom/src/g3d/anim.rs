//! Animation containers: `BCA0` (NSBCA, `JNT0` joint), `BTA0` (NSBTA, `SRT0` texture SRT),
//! `BTP0` (NSBTP, `PAT0` texture pattern), `BMA0` (NSBMA, `MAT0`) and `BVA0` (NSBVA, `VIS0`).
//!
//! Only the headers are decoded: per animation its name, frame count and track (node/material)
//! count. Track keyframe data is kept as the raw bytes of the animation (UNVERIFIED layout).

use super::{bad, blocks, u16_at, u32_at, Dict};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimInfo {
    pub name: String,
    /// The 4-byte tag at the start of the animation record (e.g. `J\0AC`).
    pub tag: [u8; 4],
    pub frames: u16,
    /// Number of animated nodes / materials / textures.
    pub tracks: u16,
    /// Absolute file offset of the animation record.
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimFile {
    /// File magic: `BCA0`, `BTA0`, `BTP0`, `BMA0` or `BVA0`.
    pub magic: [u8; 4],
    pub anims: Vec<AnimInfo>,
}

impl AnimFile {
    pub fn parse(d: &[u8]) -> Result<AnimFile> {
        let magic: [u8; 4] = super::rd(d, 0)?;
        let inner: &[u8; 4] = match &magic {
            b"BCA0" => b"JNT0",
            b"BTA0" => b"SRT0",
            b"BTP0" => b"PAT0",
            b"BMA0" => b"MAT0",
            b"BVA0" => b"VIS0",
            _ => return Err(bad("animation file", "unknown magic")),
        };
        let mut anims = Vec::new();
        for (tag, b) in blocks(d, &magic, "animation file")? {
            if &tag != inner {
                continue;
            }
            let dict = Dict::parse(d, b + 8)?;
            for (i, name) in dict.names.iter().enumerate() {
                let o = b + u32_at(d, dict.entry(i))? as usize;
                anims.push(AnimInfo {
                    name: name.clone(),
                    tag: super::rd(d, o)?,
                    frames: u16_at(d, o + 4)?,
                    tracks: u16_at(d, o + 6)?,
                    offset: o,
                });
            }
        }
        Ok(AnimFile { magic, anims })
    }
}
