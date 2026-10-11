//! Compression of the ARM9 and overlays into their ROM storage form.
//!
//! Replaces `compress_modules.py` together with the `compstatic.exe -9 -c` it drives, so no
//! Windows tooling is needed.

use crate::blz;
use crate::error::{create_dir, le32, read_file, write_file, Error, Result};
use crate::module::{ARM9_FOOTER_SIZE, COMPRESSED_END_FIELD};
use crate::overlay::{parse_table, WORD_AUTHENTICATED, WORD_COMPRESSED};
use std::collections::BTreeSet;
use std::path::Path;

/// The first 0x4000 bytes of the ARM9 (the secure area) are never compressed.
pub const ARM9_UNCOMPRESSED_PREFIX: usize = 0x4000;

/// Result of packing.
#[derive(Debug, Clone)]
pub struct Packed {
    /// Compressed ARM9 including the 12-byte SDK footer (what ends up as `packed/arm9.bin`).
    pub arm9: Vec<u8>,
    /// Overlay table with sizes and flags filled in.
    pub table: Vec<u8>,
    /// Overlays in ROM storage form.
    pub overlays: Vec<Vec<u8>>,
    /// Number of overlays actually stored compressed.
    pub compressed: usize,
    /// Number of overlays stored raw.
    pub raw: usize,
}

/// Parse an `overlay_compression.txt` style policy: ids the ROM stores uncompressed.
pub fn parse_policy(text: &str) -> Result<BTreeSet<usize>> {
    let mut ids = BTreeSet::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        ids.insert(
            line.parse::<usize>()
                .map_err(|_| Error::invalid("overlay policy", format!("bad id {line:?}")))?,
        );
    }
    Ok(ids)
}

/// Compress the static ARM9 module (`main.sbin`, which ends in the 12-byte SDK footer).
/// `load` is the module's RAM address, used for the compressed-end field in the params.
pub fn pack_arm9(main_sbin: &[u8], load: u32) -> Result<Vec<u8>> {
    if main_sbin.len() < ARM9_FOOTER_SIZE {
        return Err(Error::invalid("main.sbin", "shorter than the SDK footer"));
    }
    let (body, footer) = main_sbin.split_at(main_sbin.len() - ARM9_FOOTER_SIZE);
    let params = le32(footer, 4, "ARM9 footer")? as usize;
    let field = params + COMPRESSED_END_FIELD;
    if field + 4 > body.len() {
        return Err(Error::invalid(
            "ARM9 footer",
            "module params offset is out of range",
        ));
    }
    // compstatic skips a module whose params already name a compressed end
    let mut body = body.to_vec();
    body[field..field + 4].fill(0);
    let mut out = blz::compress(&body, ARM9_UNCOMPRESSED_PREFIX).unwrap_or(body);
    if out.len() != main_sbin.len() - ARM9_FOOTER_SIZE || blz::decompress(&out).is_ok() {
        let end = load.wrapping_add(out.len() as u32);
        out[field..field + 4].copy_from_slice(&end.to_le_bytes());
    }
    out.extend_from_slice(footer);
    Ok(out)
}

/// Pack every overlay and patch the overlay table.
///
/// `read_overlay(i)` supplies the linker output `OVY_<i>.sbin`; `raw` lists the ids the ROM
/// keeps uncompressed regardless of whether compression would help.
pub fn pack_overlays(
    table: &[u8],
    raw: &BTreeSet<usize>,
    mut read_overlay: impl FnMut(usize) -> Result<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<Vec<u8>>, usize)> {
    let count = parse_table(table)?.len();
    let mut table = table.to_vec();
    let mut overlays = Vec::with_capacity(count);
    let mut compressed = 0;
    for index in 0..count {
        let data = read_overlay(index)?;
        let field = index * 32 + 28;
        let (stored, word) = if raw.contains(&index) {
            (data, WORD_AUTHENTICATED)
        } else {
            match blz::compress(&data, 0) {
                Some(c) => {
                    compressed += 1;
                    let word = WORD_COMPRESSED | (c.len() as u32 & 0xFF_FFFF);
                    (c, word | WORD_AUTHENTICATED)
                }
                None => {
                    let word = le32(&table, field, "overlay table")?;
                    (data, word | WORD_AUTHENTICATED)
                }
            }
        };
        table[field..field + 4].copy_from_slice(&word.to_le_bytes());
        overlays.push(stored);
    }
    Ok((table, overlays, compressed))
}

/// Run the whole `compress_modules.py` pipeline from a build directory.
pub fn pack_build_dir(build: &Path, policy: &BTreeSet<usize>, load: u32) -> Result<Packed> {
    let arm9 = pack_arm9(&read_file(build.join("main.sbin"))?, load)?;
    let table = read_file(build.join("main_table.sbin"))?;
    let (table, overlays, compressed) = pack_overlays(&table, policy, |i| {
        read_file(build.join(format!("OVY_{i}.sbin")))
    })?;
    let raw = overlays.len() - compressed;
    Ok(Packed {
        arm9,
        table,
        overlays,
        compressed,
        raw,
    })
}

/// Write `arm9.bin`, `main_table.sbin` and `OVY_<n>.sbin` into `outdir`.
pub fn write_packed(packed: &Packed, outdir: &Path) -> Result<()> {
    create_dir(outdir)?;
    write_file(outdir.join("arm9.bin"), &packed.arm9)?;
    write_file(outdir.join("main_table.sbin"), &packed.table)?;
    for (i, data) in packed.overlays.iter().enumerate() {
        write_file(outdir.join(format!("OVY_{i}.sbin")), data)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_parsing() {
        let ids = parse_policy("# c\n80\n95 # x\n\n228\n").unwrap();
        assert_eq!(ids.into_iter().collect::<Vec<_>>(), vec![80, 95, 228]);
        assert!(parse_policy("abc").is_err());
    }
}
