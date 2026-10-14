//! Nitro 3D (G3D) readers (A5): NSBMD/BMD0 models, NSBTX/BTX0 textures, animation headers, and a
//! glTF 2.0 exporter. See docs/migration/checklist/A.md (A5) for the format notes.
//!
//! - [`model::Bmd0`]: `BMD0` file with `MDL0` models and an optional embedded `TEX0`.
//! - [`tex::Tex0`]: texture + palette block (formats 1-7 incl. 4x4 compressed) -> [`crate::render::Image`].
//! - [`anim`]: `BCA0`/`BTA0`/`BTP0`/`BMA0`/`BVA0` headers (animation names, frame counts, track counts).
//! - [`gltf::export`]: meshes (display lists interpreted) + textures -> `.gltf` JSON, `.bin`, PNGs.

pub mod anim;
pub mod gltf;
pub mod model;
pub mod tex;

use crate::error::{Error, Result};

pub(crate) fn bad(what: &'static str, detail: impl Into<String>) -> Error {
    Error::Invalid {
        what,
        detail: detail.into(),
    }
}

/// Bounds-checked little-endian reads.
pub(crate) fn rd<const N: usize>(d: &[u8], o: usize) -> Result<[u8; N]> {
    d.get(o..o.saturating_add(N))
        .map(|b| b.try_into().unwrap())
        .ok_or(Error::Truncated {
            what: "G3D file",
            offset: o,
            need: N,
            have: d.len(),
        })
}
pub(crate) fn u8_at(d: &[u8], o: usize) -> Result<u8> {
    Ok(rd::<1>(d, o)?[0])
}
pub(crate) fn u16_at(d: &[u8], o: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(rd(d, o)?))
}
pub(crate) fn i16_at(d: &[u8], o: usize) -> Result<i16> {
    Ok(i16::from_le_bytes(rd(d, o)?))
}
pub(crate) fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(rd(d, o)?))
}
pub(crate) fn i32_at(d: &[u8], o: usize) -> Result<i32> {
    Ok(i32::from_le_bytes(rd(d, o)?))
}

/// A Nitro "3D dictionary": names plus fixed-size data entries.
pub(crate) struct Dict {
    pub names: Vec<String>,
    /// Absolute offset of the first data entry.
    pub data: usize,
    /// Bytes per data entry.
    pub unit: usize,
}

impl Dict {
    /// Layout: `rev:u8 count:u8 size:u16 | tree_off:u16 unit_off:u16 | (count+1) tree nodes of
    /// 4 bytes (root first) | unit_size:u16 data_size:u16 | count entries | count 16-byte names`.
    pub fn parse(d: &[u8], off: usize) -> Result<Dict> {
        let n = u8_at(d, off + 1)? as usize;
        let p = off + 8 + (n + 1) * 4;
        let unit = u16_at(d, p)? as usize;
        let data = p + 4;
        let names_at = data + n * unit;
        let mut names = Vec::with_capacity(n);
        for i in 0..n {
            let raw: [u8; 16] = rd(d, names_at + i * 16)?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(16);
            names.push(String::from_utf8_lossy(&raw[..end]).into_owned());
        }
        Ok(Dict { names, data, unit })
    }
    pub fn entry(&self, i: usize) -> usize {
        self.data + i * self.unit
    }
}

/// Block offsets of a `BMD0`/`BTX0`/`BCA0`... file: `(tag, absolute offset)`.
pub(crate) fn blocks(
    d: &[u8],
    magic: &[u8; 4],
    what: &'static str,
) -> Result<Vec<([u8; 4], usize)>> {
    if d.get(..4) != Some(magic) {
        return Err(bad(what, "bad magic"));
    }
    let n = u16_at(d, 14)? as usize;
    (0..n)
        .map(|i| {
            let o = u32_at(d, 16 + 4 * i)? as usize;
            Ok((rd::<4>(d, o)?, o))
        })
        .collect()
}
