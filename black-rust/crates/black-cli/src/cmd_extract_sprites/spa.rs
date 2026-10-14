//! `effects`: particle textures from SPA files (A10d). The `black extract-sprites effects` job
//! converts the textures of every SPA member of ARC 006 (733 files), ARC 216 (18 of 52 members),
//! ARC 047 (4) and ARC 181 (4) to one PNG atlas + `.atlas.ron` per archive.
//!
//! SPA layout (reverse-engineered here from the ROM files; the emitter/resource block is skipped):
//! `" APS" "12_1"  u16 nres  u16 ntex  u32 0  u32 res_size  u32 tex_size  u32 tex_off  u32 0`
//! (0x20 bytes), then `res_size` bytes of particle-resource data, then `ntex` texture entries at
//! `0x20 + res_size`. Each entry: `" TPS"  u32 param  u32 pixel_bytes  u32 pal_off(=0x20+pixel_bytes)
//! u32 pal_bytes  u32 entry_bytes  +8 zero`, pixels at +0x20, palette (BGR555) at `+pal_off`.
//! `param` low nibble = DS texture format (1 A3I5, 2 4-colour, 3 16-colour, 4 256-colour, 6 A5I3),
//! bits 4..8 = log2(height/8), bits 8..12 = log2(width/8); all 2,728 textures in ARC 006 decode
//! with these rules and size = pixel_bytes (checked in the tests). UNVERIFIED: the upper param
//! bits (0x1f226, 0x15306: repeat/flip or frame flags) and colour-0 transparency for formats 2..4.

use super::items::{atlas_ron, load_narc, pack};
use super::pokemon::{member, Res};
use nds_rom::render::Image;
use std::path::Path;

/// (arc, stem)
pub const ARCS: [(u32, &str); 4] = [
    (6, "spa_arc006"),
    (216, "spa_arc216"),
    (47, "spa_arc047"),
    (181, "spa_arc181"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tex {
    pub param: u32,
    pub format: u8,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub palette: Vec<u16>,
}

fn u32le(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

/// Parse the textures of one SPA file. `None` if it is not an SPA.
pub fn parse(b: &[u8]) -> Option<Result<Vec<Tex>, String>> {
    if b.get(..4)? != b" APS" {
        return None;
    }
    Some(parse_inner(b))
}

fn parse_inner(b: &[u8]) -> Result<Vec<Tex>, String> {
    let ntex = u16::from_le_bytes(b.get(10..12).ok_or("short")?.try_into().unwrap()) as usize;
    let res = u32le(b, 0x10).ok_or("short")? as usize;
    let mut p = 0x20 + res;
    let mut out = vec![];
    for t in 0..ntex {
        if b.get(p..p + 4) != Some(b" TPS") {
            return Err(format!("texture {t}: bad magic at {p:#x}"));
        }
        let param = u32le(b, p + 4).ok_or("short")?;
        let px = u32le(b, p + 8).ok_or("short")? as usize;
        let pal_off = u32le(b, p + 12).ok_or("short")? as usize;
        let pal_sz = u32le(b, p + 16).ok_or("short")? as usize;
        let total = u32le(b, p + 20).ok_or("short")? as usize;
        let format = (param & 0xF) as u8;
        let height = 8usize << ((param >> 4) & 0xF);
        let width = 8usize << ((param >> 8) & 0xF);
        let bits = match format {
            1 | 4 | 6 => 8,
            2 => 2,
            3 => 4,
            f => return Err(format!("texture {t}: unsupported format {f}")),
        };
        if width * height * bits / 8 != px {
            return Err(format!(
                "texture {t}: {width}x{height} fmt {format} != {px} bytes"
            ));
        }
        if pal_off != 0x20 + px || total != pal_off + pal_sz {
            return Err(format!("texture {t}: odd layout"));
        }
        let pixels = b
            .get(p + 0x20..p + 0x20 + px)
            .ok_or("pixels out of range")?
            .to_vec();
        let pal = b
            .get(p + pal_off..p + pal_off + pal_sz)
            .ok_or("palette out of range")?;
        let palette = pal
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        out.push(Tex {
            param,
            format,
            width,
            height,
            pixels,
            palette,
        });
        p += total;
    }
    Ok(out)
}

fn rgb(e: u16) -> [u8; 3] {
    let c = |s: u32| {
        let v = ((e >> s) & 0x1F) as u32;
        ((v << 3) | (v >> 2)) as u8
    };
    [c(0), c(5), c(10)]
}

impl Tex {
    pub fn to_image(&self) -> Image {
        let mut img = Image::new(self.width, self.height);
        let col = |i: usize| rgb(self.palette.get(i).copied().unwrap_or(0));
        for n in 0..self.width * self.height {
            let (idx, alpha) = match self.format {
                1 => {
                    let v = self.pixels[n];
                    ((v & 0x1F) as usize, (v >> 5) as u32 * 255 / 7)
                }
                6 => {
                    let v = self.pixels[n];
                    ((v & 7) as usize, (v >> 3) as u32 * 255 / 31)
                }
                2 => {
                    let v = (self.pixels[n / 4] >> ((n % 4) * 2)) & 3;
                    (v as usize, if v == 0 { 0 } else { 255 })
                }
                3 => {
                    let v = (self.pixels[n / 2] >> ((n % 2) * 4)) & 15;
                    (v as usize, if v == 0 { 0 } else { 255 })
                }
                _ => {
                    let v = self.pixels[n];
                    (v as usize, if v == 0 { 0 } else { 255 })
                }
            };
            let c = col(idx);
            img.rgba[n * 4..n * 4 + 4].copy_from_slice(&[c[0], c[1], c[2], alpha as u8]);
        }
        img
    }
}

pub struct Report {
    pub spa_files: usize,
    pub textures: usize,
    pub failures: Vec<(String, String)>,
}

/// One archive -> (atlas, ron, textures per file)
pub fn build_arc(
    files: &Path,
    arc: u32,
    stem: &str,
    rep: &mut Report,
) -> Res<(Image, String, Vec<Image>)> {
    let narc = load_narc(files, arc)?;
    let mut images = vec![];
    let mut meta = vec![];
    for m in 0..narc.len() {
        let d = member(&narc, m);
        match parse(&d) {
            None => {}
            Some(Ok(texs)) => {
                rep.spa_files += 1;
                for (t, tex) in texs.iter().enumerate() {
                    images.push((format!("spa_{m:03}_{t}"), tex.to_image()));
                    meta.push(format!(
                        ", member: {m}, tex: {t}, format: {}, param: {:#x}",
                        tex.format, tex.param
                    ));
                }
            }
            Some(Err(e)) => rep.failures.push((format!("ARC {arc} member {m}"), e)),
        }
    }
    rep.textures += images.len();
    let (atlas, rects) = pack(&images, 2048);
    let entries: Vec<_> = images
        .iter()
        .zip(&rects)
        .zip(&meta)
        .map(|(((n, _), r), x)| (n.clone(), *r, x.clone()))
        .collect();
    let ron = atlas_ron(
        &format!("ARC {arc:03} (SPA particle files)"),
        stem,
        &atlas,
        &entries,
        "textures only (emitter data not converted); colour-0 transparency for 4/16/256-colour formats and the upper param bits UNVERIFIED",
    );
    Ok((atlas, ron, images.into_iter().map(|(_, i)| i).collect()))
}

pub fn extract(files: &Path, out: &Path) -> Res<Report> {
    let dir = out.join("sprites/effects");
    std::fs::create_dir_all(&dir)?;
    let mut rep = Report {
        spa_files: 0,
        textures: 0,
        failures: vec![],
    };
    for (arc, stem) in ARCS {
        let (atlas, ron, _) = build_arc(files, arc, stem, &mut rep)?;
        std::fs::write(dir.join(format!("{stem}.png")), atlas.to_png()?)?;
        std::fs::write(dir.join(format!("{stem}.atlas.ron")), ron)?;
    }
    Ok(rep)
}

/// Thumbnail grid of every 8th texture of each archive (keeps the sheet small).
pub fn contact_sheet(files: &Path) -> Res<Image> {
    let mut rep = Report {
        spa_files: 0,
        textures: 0,
        failures: vec![],
    };
    let mut keep = vec![];
    for (arc, stem) in ARCS {
        let (_, _, imgs) = build_arc(files, arc, stem, &mut rep)?;
        keep.extend(imgs.into_iter().step_by(8));
    }
    let refs: Vec<&Image> = keep.iter().collect();
    Ok(super::gfxarc::thumb_grid(&refs, 32, 40))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files() -> Option<std::path::PathBuf> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let f = root.join("files");
        if f.join("a/0/0/6").exists() {
            Some(f)
        } else {
            nds_rom::testenv::rom_missing("black-rom/files");
            None
        }
    }

    #[test]
    fn decodes_every_texture() {
        let Some(f) = files() else { return };
        let mut rep = Report {
            spa_files: 0,
            textures: 0,
            failures: vec![],
        };
        let (a, ron, imgs) = build_arc(&f, 6, "spa_arc006", &mut rep).unwrap();
        assert!(rep.failures.is_empty(), "{:?}", rep.failures);
        assert_eq!(rep.spa_files, 733);
        assert_eq!(rep.textures, 2728);
        assert_eq!(imgs.len(), 2728);
        assert!(a.opaque_pixels() > 100_000);
        assert!(ron.contains("spa_000_0"));
        // first texture of member 0: 64x64 A5I3, 2-colour palette
        let (w, h) = (imgs[0].width, imgs[0].height);
        assert_eq!((w, h), (64, 64));
        let empty = imgs.iter().filter(|i| i.opaque_pixels() == 0).count();
        assert!(empty * 20 < imgs.len(), "{empty} empty textures");
        for (arc, stem) in &ARCS[1..] {
            let (_, _, imgs) = build_arc(&f, *arc, stem, &mut rep).unwrap();
            assert!(rep.failures.is_empty(), "{:?}", rep.failures);
            assert!(!imgs.is_empty(), "{arc}");
        }
    }
}
