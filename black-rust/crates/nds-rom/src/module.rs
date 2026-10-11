//! ARM9/ARM7 module parameters and autoload blocks.

use crate::error::{le32, Error, Result};

const NITRO_BE: u32 = 0xDEC0_0621;
const NITRO_LE: u32 = 0x2106_C0DE;
/// Number of bytes of `ModuleParams` that precede the NitroCode magic.
pub const PARAMS_BEFORE_MAGIC: usize = 0x1C;
/// Offset of the compressed-static-end field inside `ModuleParams`.
pub const COMPRESSED_END_FIELD: usize = 0x14;

/// SDK `MIModuleParams`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleParams {
    /// Offset of the params block inside the image.
    pub offset: usize,
    pub autoload_list_start: u32,
    pub autoload_list_end: u32,
    pub autoload_start: u32,
    pub static_bss_start: u32,
    pub static_bss_end: u32,
    /// Zero for an uncompressed static module, else the RAM address where compression ends.
    pub compressed_static_end: u32,
    pub sdk_version: u32,
}

/// One autoload block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoloadBlock {
    pub load: u32,
    pub size: u32,
    pub bss: u32,
    /// Start of the static-initialiser list (`load` for 12-byte entries).
    pub sinit: u32,
}

/// Locate `ModuleParams` by scanning for the NitroCode magic pair.
pub fn find_module_params(image: &[u8]) -> Result<ModuleParams> {
    let mut off = PARAMS_BEFORE_MAGIC;
    while off + 8 <= image.len() {
        if le32(image, off, "module")? == NITRO_BE && le32(image, off + 4, "module")? == NITRO_LE {
            let base = off - PARAMS_BEFORE_MAGIC;
            let w = |i: usize| le32(image, base + i * 4, "module params");
            return Ok(ModuleParams {
                offset: base,
                autoload_list_start: w(0)?,
                autoload_list_end: w(1)?,
                autoload_start: w(2)?,
                static_bss_start: w(3)?,
                static_bss_end: w(4)?,
                compressed_static_end: w(5)?,
                sdk_version: w(6)?,
            });
        }
        off += 4;
    }
    Err(Error::NoModuleParams)
}

/// Decode the autoload list of a (decompressed) module loaded at `load`.
/// Returns the payload start address and the blocks.
pub fn autoload_blocks(image: &[u8], load: u32) -> Result<(u32, Vec<AutoloadBlock>)> {
    let p = find_module_params(image)?;
    let span = p.autoload_list_end.wrapping_sub(p.autoload_list_start);
    let payload = p.autoload_list_start.wrapping_sub(p.autoload_start);
    for stride in [12u32, 16] {
        if span == 0 || span % stride != 0 {
            continue;
        }
        let count = (span / stride) as usize;
        let base = p.autoload_list_start.wrapping_sub(load) as usize;
        let mut blocks = Vec::with_capacity(count);
        for i in 0..count {
            let at = base + stride as usize * i;
            let words = (0..stride as usize / 4)
                .map(|k| le32(image, at + k * 4, "autoload list"))
                .collect::<Result<Vec<_>>>()?;
            let last = *words.last().unwrap_or(&0);
            blocks.push(AutoloadBlock {
                load: words[0],
                size: words[1],
                bss: last,
                sinit: if stride == 16 { words[2] } else { words[0] },
            });
        }
        if blocks.iter().map(|b| b.size).sum::<u32>() == payload {
            return Ok((p.autoload_start, blocks));
        }
    }
    Err(Error::BadAutoloadList(span))
}

/// Footer the SDK appends after the ARM9 static module: NitroCode magic, params offset, zero.
pub const ARM9_FOOTER_SIZE: usize = 12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_params_and_autoloads() {
        let load = 0x0200_0000u32;
        let mut img = vec![0u8; 0x100];
        let base = 0x20;
        let list = load + 0x80;
        let words = [
            list,
            list + 12,
            load + 0x60,
            0,
            0,
            0,
            0x1234,
            NITRO_BE,
            NITRO_LE,
        ];
        for (i, w) in words.iter().enumerate() {
            img[base + i * 4..base + i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        // one 12-byte entry (load, size, bss) whose size equals list_start - autoload_start
        let size = list - (load + 0x60);
        for (i, w) in [0x0380_0000u32, size, 0x40].iter().enumerate() {
            img[0x80 + i * 4..0x80 + i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        let p = find_module_params(&img).unwrap();
        assert_eq!(p.offset, base);
        assert_eq!(p.sdk_version, 0x1234);
        let (start, blocks) = autoload_blocks(&img, load).unwrap();
        assert_eq!(start, load + 0x60);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].bss, 0x40);
        assert!(find_module_params(&[0; 64]).is_err());
    }
}
