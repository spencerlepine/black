//! Item icons (ARC 025) -> one PNG atlas + `.atlas.ron`, plus the small atlas helpers shared with `ui`.
//!
//! ARC 025 = `[NANR, NCER, then NCGR/NCLR members]`; every icon is a 32x32 4bpp NCGR (one cell in
//! the NCER) with a 256-entry NCLR (row 0 used). The item -> (NCGR member, NCLR member) table is
//! `u16 pairs x 627` in the ARM9 at 0x0209EB6C (read by the item graphics loader at 0x02020F34:
//! mode 1 = NCGR, mode 2 = NCLR; max item id 0x272). Item 0 maps to the blank pair (1001, 1002).

use super::pokemon::{member, Res};
use nds_rom::narc::Narc;
use nds_rom::ncer::Ncer;
use nds_rom::ncgr::Ncgr;
use nds_rom::nclr::Nclr;
use nds_rom::render::{render_cell, Image};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// (x, y, w, h)
pub type Rect = (usize, usize, usize, usize);

pub const ITEM_COUNT: usize = 627;
const ARM9_BASE: usize = 0x0200_4000;
const TABLE_ADDR: usize = 0x0209_EB6C;

pub fn load_narc(files: &Path, id: u32) -> Res<Narc> {
    let p = files.join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
    Ok(Narc::parse(&std::fs::read(p)?)?)
}

/// `<rom_root>/build/black.us/main.sbin` (override with `BLACK_ROM_BUILD`).
pub fn main_sbin(files: &Path) -> PathBuf {
    if let Some(b) = std::env::var_os("BLACK_ROM_BUILD") {
        return PathBuf::from(b).join("main.sbin");
    }
    files
        .parent()
        .unwrap_or(Path::new("."))
        .join("build/black.us/main.sbin")
}

/// Item id -> (NCGR member, NCLR member) of ARC 025, read from an explicit `main.sbin`.
pub fn item_table_at(main: &Path) -> Res<Vec<(usize, usize)>> {
    let b = std::fs::read(main).map_err(|e| {
        format!(
            "{}: {e} (ARM9 needed for the item icon table)",
            main.display()
        )
    })?;
    let o = TABLE_ADDR - ARM9_BASE;
    let t = b.get(o..o + ITEM_COUNT * 4).ok_or("main.sbin too small")?;
    Ok(t.chunks(4)
        .map(|c| {
            (
                u16::from_le_bytes([c[0], c[1]]) as usize,
                u16::from_le_bytes([c[2], c[3]]) as usize,
            )
        })
        .collect())
}

/// Render the first cell of the icon NCER with the given graphics and palette.
pub fn icon_image(narc: &Narc, ncer: &Ncer, gfx: usize, pal: usize) -> Res<Image> {
    let g = Ncgr::parse(&member(narc, gfx))?;
    let p = Nclr::parse(&member(narc, pal))?;
    let (img, _) = render_cell(ncer, 0, &g, &p).ok_or("no cell 0")?;
    Ok(img)
}

/// Shelf-pack named images into one atlas (max width `max_w`). Returns the atlas and rects
/// `(x, y, w, h)` in input order.
pub fn pack(
    images: &[(String, Image)],
    max_w: usize,
) -> (Image, Vec<(usize, usize, usize, usize)>) {
    let (mut x, mut y, mut rh, mut wmax) = (0usize, 0usize, 0usize, 0usize);
    let mut rects = vec![];
    for (_, im) in images {
        if x + im.width > max_w && x > 0 {
            x = 0;
            y += rh;
            rh = 0;
        }
        rects.push((x, y, im.width, im.height));
        x += im.width;
        wmax = wmax.max(x);
        rh = rh.max(im.height);
    }
    let mut atlas = Image::new(wmax.max(1), (y + rh).max(1));
    for ((_, im), &(rx, ry, w, h)) in images.iter().zip(&rects) {
        for yy in 0..h {
            let d = ((ry + yy) * atlas.width + rx) * 4;
            let s = yy * w * 4;
            atlas.rgba[d..d + w * 4].copy_from_slice(&im.rgba[s..s + w * 4]);
        }
    }
    (atlas, rects)
}

/// `.atlas.ron` text for a flat list of named rects. `extra` = per-entry extra fields text.
pub fn atlas_ron(
    source: &str,
    name: &str,
    atlas: &Image,
    entries: &[(String, Rect, String)],
    note: &str,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "(");
    let _ = writeln!(s, "    source: \"{source}\",");
    let _ = writeln!(s, "    name: \"{name}\",");
    let _ = writeln!(s, "    image: \"{name}.png\",");
    let _ = writeln!(s, "    size: ({}, {}),", atlas.width, atlas.height);
    if !note.is_empty() {
        let _ = writeln!(s, "    // {note}");
    }
    let _ = writeln!(s, "    entries: [");
    for (n, (x, y, w, h), extra) in entries {
        let _ = writeln!(
            s,
            "        (name: \"{n}\", rect: ({x}, {y}, {w}, {h}){extra}),"
        );
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(s, ")");
    s
}

pub struct Report {
    pub items: usize,
    pub unique_icons: usize,
    pub failures: Vec<(String, String)>,
}

/// Build the item atlas: one 32x32 entry per item id 0..627.
pub fn build(files: &Path) -> Res<(Image, String, Report)> {
    build_at(files, &main_sbin(files))
}

/// [`build`] with an explicit `main.sbin` path (the item table is read from it).
pub fn build_at(files: &Path, main: &Path) -> Res<(Image, String, Report)> {
    let narc = load_narc(files, 25)?;
    let table = item_table_at(main)?;
    let ncer = Ncer::parse(&member(&narc, 1))?;
    let mut cache: std::collections::HashMap<(usize, usize), Image> = Default::default();
    let mut rep = Report {
        items: 0,
        unique_icons: 0,
        failures: vec![],
    };
    let mut images = vec![];
    let mut meta = vec![];
    for (id, &(g, p)) in table.iter().enumerate() {
        let im = match cache.get(&(g, p)) {
            Some(i) => i.clone(),
            None => match icon_image(&narc, &ncer, g, p) {
                Ok(i) => {
                    cache.insert((g, p), i.clone());
                    i
                }
                Err(e) => {
                    rep.failures.push((format!("item {id}"), e.to_string()));
                    continue;
                }
            },
        };
        meta.push((id, g, p));
        images.push((format!("item_{id:03}"), im));
        rep.items += 1;
    }
    rep.unique_icons = cache.len();
    let (atlas, rects) = pack(&images, 1024);
    let entries: Vec<_> = images
        .iter()
        .zip(&rects)
        .zip(&meta)
        .map(|(((n, _), r), (id, g, p))| {
            (n.clone(), *r, format!(", item: {id}, ncgr: {g}, nclr: {p}"))
        })
        .collect();
    let ron = atlas_ron(
        "ARC 025 + ARM9 0x0209EB6C table",
        "items",
        &atlas,
        &entries,
        "32x32 icons; item id = ARC 024 index; ncgr/nclr = ARC 025 member indices (many items share icons)",
    );
    Ok((atlas, ron, rep))
}

pub fn extract(files: &Path, out: &Path) -> Res<Report> {
    extract_at(files, out, &main_sbin(files))
}

/// [`extract`] with an explicit `main.sbin` path.
pub fn extract_at(files: &Path, out: &Path, main: &Path) -> Res<Report> {
    let dir = out.join("sprites/items");
    std::fs::create_dir_all(&dir)?;
    let (atlas, ron, rep) = build_at(files, main)?;
    std::fs::write(dir.join("items.png"), atlas.to_png()?)?;
    std::fs::write(dir.join("items.atlas.ron"), ron)?;
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_extract_sprites::pokemon::image_hash;

    pub fn files_dir() -> Option<PathBuf> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let f = root.join("files");
        (f.join("a/0/2/5").exists() && main_sbin(&f).exists()).then_some(f)
    }

    #[test]
    fn converts_all_items() {
        let Some(f) = files_dir() else { return };
        let t = item_table_at(&main_sbin(&f)).unwrap();
        assert_eq!(t[1], (2, 3));
        assert_eq!(t[0], (1001, 1002));
        let (atlas, ron, rep) = build(&f).unwrap();
        assert_eq!(rep.items, ITEM_COUNT, "{:?}", rep.failures);
        assert!(rep.unique_icons > 400);
        assert!(atlas.opaque_pixels() > 50_000);
        assert!(ron.contains("item_001"));
    }

    #[test]
    fn item_golden() {
        let Some(f) = files_dir() else { return };
        let narc = load_narc(&f, 25).unwrap();
        let ncer = Ncer::parse(&member(&narc, 1)).unwrap();
        // item 4 = Poke Ball (ARC 024 #4: ball id 0 -> table (6,7))
        let t = item_table_at(&main_sbin(&f)).unwrap();
        let im = icon_image(&narc, &ncer, t[4].0, t[4].1).unwrap();
        assert_eq!((im.width, im.height), (32, 32));
        assert_eq!(image_hash(&im), GOLDEN_ITEM4, "item 4");
    }
    const GOLDEN_ITEM4: &str = "0ff834be9a4af5ec29108d969b500553adcf471d";
}
