//! Overworld player/NPC sprites (ARC 049) -> PNG atlases + `.atlas.ron`.
//!
//! Gen 5 overworld characters are textured quads: ARC 049 holds 6 BMD0 quad models (members 0..5)
//! followed by 758 BTX0 texture sets. Each BTX0 is one character (names like `t4x4hero.N`):
//! N textures of 32x32 (`t4x4`) or 64x64 (`t8x8`) pixels, format 3 (16-colour, colour 0
//! transparent), plus 1-2 palettes (2 for Pokemon-follower sets: presumably normal / shiny,
//! UNVERIFIED). Textures are the animation frames in index order. Which index is which
//! direction/step (and which frames are drawn mirrored: `flip` in the name) is selected by
//! NSBTA/model data and is UNVERIFIED; the atlas keeps the raw index order, 8 frames per row.
//! Member 6 = player `hero` (32 frames), 7 = `cycle` (bike, 16), 8 = `swim` (4).

use super::pokemon::Res;
use nds_rom::narc::Narc;
use nds_rom::render::Image;
use std::fmt::Write as _;
use std::path::Path;

const PER_ROW: usize = 8;

pub struct Palette {
    pub name: String,
    pub colors: Vec<[u8; 4]>,
}

pub struct Tex {
    pub name: String,
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u8>, // palette indices
}

pub struct Btx {
    pub textures: Vec<Tex>,
    pub palettes: Vec<Palette>,
}

pub struct CharSprite {
    pub name: String,
    /// One atlas per palette.
    pub atlases: Vec<Image>,
    pub frame_names: Vec<String>,
    pub ron: String,
}

fn u16at(b: &[u8], o: usize) -> Res<usize> {
    let s = b.get(o..o + 2).ok_or("BTX0 truncated")?;
    Ok(u16::from_le_bytes([s[0], s[1]]) as usize)
}
fn u32at(b: &[u8], o: usize) -> Res<usize> {
    let s = b.get(o..o + 4).ok_or("BTX0 truncated")?;
    Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize)
}

type Dict<'a> = (Vec<&'a [u8]>, Vec<String>);

/// Read a Nitro 3D name dictionary at `d` with `entry_len`-byte entries.
fn dict(b: &[u8], d: usize, entry_len: usize) -> Res<Dict<'_>> {
    let cnt = *b.get(d + 1).ok_or("dict")? as usize;
    let p = d + 8 + (cnt + 1) * 4 + 4;
    let ents = (0..cnt)
        .map(|i| {
            b.get(p + i * entry_len..p + (i + 1) * entry_len)
                .ok_or("dict entries")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let np = p + cnt * entry_len;
    let names = (0..cnt)
        .map(|i| {
            let n = b.get(np + i * 16..np + i * 16 + 16).ok_or("dict names")?;
            let end = n.iter().position(|&c| c == 0).unwrap_or(16);
            Ok(String::from_utf8_lossy(&n[..end]).into_owned())
        })
        .collect::<Result<Vec<_>, &str>>()?;
    Ok((ents, names))
}

/// Minimal BTX0 reader: 4/16/256-colour textures are decoded, direct-colour formats rejected.
pub fn parse_btx0(b: &[u8]) -> Res<Btx> {
    if b.get(..4) != Some(b"BTX0") {
        return Err("not BTX0".into());
    }
    let t = u32at(b, 16)?;
    if b.get(t..t + 4) != Some(b"TEX0") {
        return Err("no TEX0".into());
    }
    let info = u16at(b, t + 0x0E)?;
    let tex_data = u32at(b, t + 0x14)?;
    let pal_size = u32at(b, t + 0x30)? * 8;
    let pal_info = u32at(b, t + 0x34)?;
    let pal_data = u32at(b, t + 0x38)?;
    let (ents, names) = dict(b, t + info, 8)?;
    let mut textures = Vec::new();
    for (e, name) in ents.iter().zip(names) {
        let off = u16::from_le_bytes([e[0], e[1]]) as usize * 8;
        let p = u16::from_le_bytes([e[2], e[3]]) as usize;
        let (w, h, fmt) = (
            8usize << ((p >> 4) & 7),
            8usize << ((p >> 7) & 7),
            (p >> 10) & 7,
        );
        let bpp = match fmt {
            2 => 2,
            3 => 4,
            4 => 8,
            _ => return Err(format!("texture {name}: unsupported format {fmt}").into()),
        };
        let n = w * h * bpp / 8;
        let raw = b
            .get(t + tex_data + off..t + tex_data + off + n)
            .ok_or("texture data")?;
        let mut pixels = Vec::with_capacity(w * h);
        for &v in raw {
            match bpp {
                2 => (0..4).for_each(|k| pixels.push((v >> (2 * k)) & 3)),
                4 => pixels.extend([v & 15, v >> 4]),
                _ => pixels.push(v),
            }
        }
        textures.push(Tex { name, w, h, pixels });
    }
    let (pents, pnames) = dict(b, t + pal_info, 4)?;
    let mut palettes = Vec::new();
    for (e, name) in pents.iter().zip(pnames) {
        let off = u16::from_le_bytes([e[0], e[1]]) as usize * 8;
        let end = pents
            .iter()
            .map(|x| u16::from_le_bytes([x[0], x[1]]) as usize * 8)
            .filter(|&o| o > off)
            .min()
            .unwrap_or(pal_size);
        let raw = b
            .get(t + pal_data + off..t + pal_data + end)
            .ok_or("palette data")?;
        let colors = raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| {
                let v = u16::from_le_bytes(*c);
                let ex = |s: u32| {
                    let x = ((v >> s) & 31) as u8;
                    (x << 3) | (x >> 2)
                };
                [ex(0), ex(5), ex(10), 255]
            })
            .collect();
        palettes.push(Palette { name, colors });
    }
    if palettes.is_empty() {
        return Err("no palettes".into());
    }
    Ok(Btx { textures, palettes })
}

fn png_name(name: &str, pal: usize) -> String {
    if pal == 0 {
        format!("{name}.png")
    } else {
        format!("{name}_p{pal}.png")
    }
}

pub fn convert_btx(name: &str, arc_member: usize, b: &[u8]) -> Res<CharSprite> {
    let btx = parse_btx0(b)?;
    let first = btx.textures.first().ok_or("no textures")?;
    let (fw, fh) = (first.w, first.h);
    if btx.textures.iter().any(|t| (t.w, t.h) != (fw, fh)) {
        return Err("mixed texture sizes".into());
    }
    let n = btx.textures.len();
    let rows = n.div_ceil(PER_ROW);
    let mut atlases = Vec::new();
    for pal in &btx.palettes {
        let mut img = Image::new(PER_ROW.min(n) * fw, rows * fh);
        for (i, t) in btx.textures.iter().enumerate() {
            let (ox, oy) = ((i % PER_ROW) * fw, (i / PER_ROW) * fh);
            for y in 0..fh {
                for x in 0..fw {
                    let idx = t.pixels[y * fw + x] as usize;
                    // colour 0 is always transparent (some NPC sets lack the TEX flag; UNVERIFIED)
                    if idx == 0 {
                        continue;
                    }
                    let c = pal.colors.get(idx).copied().unwrap_or([255, 0, 255, 255]);
                    let o = ((oy + y) * img.width + ox + x) * 4;
                    img.rgba[o..o + 4].copy_from_slice(&c);
                }
            }
        }
        atlases.push(img);
    }
    let mut s = String::new();
    let _ = writeln!(s, "(");
    let _ = writeln!(s, "    source: \"ARC 049 member {arc_member}\",");
    let _ = writeln!(s, "    name: \"{name}\",");
    let _ = writeln!(s, "    frame_size: ({fw}, {fh}),");
    let _ = writeln!(s, "    columns: {},", PER_ROW.min(n));
    let _ = writeln!(
        s,
        "    // frames are in texture index order; per-direction layout and mirroring are UNVERIFIED"
    );
    let _ = writeln!(s, "    images: [");
    for (k, p) in btx.palettes.iter().enumerate() {
        let _ = writeln!(
            s,
            "        (palette: \"{}\", file: \"{}\"),",
            p.name,
            png_name(name, k)
        );
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(s, "    frames: [");
    for (i, t) in btx.textures.iter().enumerate() {
        let _ = writeln!(
            s,
            "        (name: \"{}\", rect: ({}, {}, {fw}, {fh})),",
            t.name,
            (i % PER_ROW) * fw,
            (i / PER_ROW) * fh
        );
    }
    let _ = writeln!(s, "    ],\n)");
    Ok(CharSprite {
        name: name.to_string(),
        atlases,
        frame_names: btx.textures.iter().map(|t| t.name.clone()).collect(),
        ron: s,
    })
}

/// Set name from the first texture name (`t4x4hero.1` -> `006_hero`), or just the member index.
pub fn set_name(member: usize, b: &[u8]) -> String {
    let base = parse_btx0(b)
        .ok()
        .and_then(|x| x.textures.first().map(|t| t.name.clone()))
        .map(|n| {
            let stem = n.split('.').next().unwrap_or("");
            stem.trim_start_matches("t4x4")
                .trim_start_matches("t8x8")
                .to_string()
        })
        .unwrap_or_default();
    format!("{member:03}_{base}")
}

/// Contact sheet: atlases of a selection of sets stacked vertically (hero, bike, swim, NPCs, Pokemon).
pub fn contact_sheet(files: &Path) -> Res<Image> {
    let narc = Narc::parse(&std::fs::read(files.join("a/0/4/9"))?)?;
    let imgs = [6usize, 7, 8, 12, 83, 140, 199]
        .iter()
        .map(|&i| Ok(convert_btx("c", i, narc.file(i))?.atlases.remove(0)))
        .collect::<Res<Vec<Image>>>()?;
    let w = imgs.iter().map(|i| i.width).max().unwrap_or(0);
    let h: usize = imgs.iter().map(|i| i.height + 4).sum();
    let mut sheet = Image::new(w, h);
    for p in sheet.rgba.as_chunks_mut::<4>().0 {
        *p = [200, 200, 200, 255];
    }
    let mut y0 = 0;
    for img in &imgs {
        for y in 0..img.height {
            for x in 0..img.width {
                let p = img.pixel(x, y);
                if p[3] != 0 {
                    let i = ((y0 + y) * w + x) * 4;
                    sheet.rgba[i..i + 4].copy_from_slice(&p);
                }
            }
        }
        y0 += img.height + 4;
    }
    Ok(sheet)
}

pub struct Report {
    pub sets: usize,
    pub frames: usize,
    pub failures: Vec<(usize, String)>,
}

pub fn extract(files: &Path, out: &Path) -> Res<Report> {
    let dir = out.join("sprites/overworld");
    std::fs::create_dir_all(&dir)?;
    let narc = Narc::parse(&std::fs::read(files.join("a/0/4/9"))?)?;
    let mut rep = Report {
        sets: 0,
        frames: 0,
        failures: vec![],
    };
    for i in 0..narc.len() {
        let b = narc.file(i);
        if b.get(..4) != Some(b"BTX0") {
            continue;
        }
        match convert_btx(&set_name(i, b), i, b) {
            Ok(s) => {
                for (k, a) in s.atlases.iter().enumerate() {
                    std::fs::write(dir.join(png_name(&s.name, k)), a.to_png()?)?;
                }
                std::fs::write(dir.join(format!("{}.atlas.ron", s.name)), &s.ron)?;
                rep.sets += 1;
                rep.frames += s.frame_names.len();
            }
            Err(e) => rep.failures.push((i, e.to_string())),
        }
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_extract_sprites::pokemon::image_hash;

    fn load() -> Option<Narc> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        Narc::parse(&std::fs::read(root.join("files/a/0/4/9")).ok()?).ok()
    }

    #[test]
    fn converts_all_overworld_sets() {
        let Some(a) = load() else { return };
        let mut sets = 0;
        for i in 0..a.len() {
            if a.file(i).get(..4) == Some(b"BTX0") {
                let s = convert_btx(&set_name(i, a.file(i)), i, a.file(i))
                    .unwrap_or_else(|e| panic!("member {i}: {e}"));
                assert!(s.atlases[0].opaque_pixels() > 0, "member {i}");
                sets += 1;
            }
        }
        assert_eq!(sets, 758);
    }

    #[test]
    fn hero_walk_frames_golden() {
        let Some(a) = load() else { return };
        let s = convert_btx(&set_name(6, a.file(6)), 6, a.file(6)).unwrap();
        assert_eq!(s.name, "006_hero");
        assert_eq!(s.frame_names.len(), 32);
        assert!(s.ron.contains("frame_size: (32, 32)"));
        assert_eq!(image_hash(&s.atlases[0]), GOLDEN_HERO);
    }
    const GOLDEN_HERO: &str = "71bcbe2fb5e0d7eb7cf4eea097e2da8072d54f9d";
}
