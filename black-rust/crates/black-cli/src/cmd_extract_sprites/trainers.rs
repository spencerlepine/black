//! Trainer battle sprites -> PNG atlases + `.atlas.ron` (same conventions as `pokemon`).
//!
//! ARC 072 (760 members) holds 95 sets of 8 members; ARC 064 and 073 (48 members each, identical
//! sizes) hold 6 sets of 8 (player-character-like sets; UNVERIFIED which). Set layout (the
//! pokemon one-group layout, LZ11 where compressed):
//! `[tiled NCGR 64x64, scanned NCGR sheet 256x128, NCER, NANR, RCMN, RAMN, side table, NCLR (16 colours)]`.
//! The role of ARC 072 as "trainer class sprites" is inferred from the set count and a visual
//! check (UNVERIFIED).

use super::pokemon::{build_sprite, member, Orientation, Parts, Res, Sprite};
use nds_rom::narc::Narc;
use nds_rom::nclr::Nclr;
use std::path::Path;

const SET: usize = 8;
/// (ARC id, file-name prefix)
pub const ARCS: [(u32, &str); 3] = [(72, "trainer"), (64, "player_a"), (73, "player_b")];

pub fn set_count(narc: &Narc) -> usize {
    narc.len() / SET
}

pub fn convert_set(narc: &Narc, arc: u32, prefix: &str, set: usize) -> Res<Sprite> {
    let m = |i: usize| member(narc, set * SET + i);
    let pal = Nclr::parse(&m(7))?;
    let (sheet, ncer, nanr, rcmn, ramn, cellinfo) = (m(1), m(2), m(3), m(4), m(5), m(6));
    let mut s = build_sprite(&Parts {
        name: format!("{prefix}_{set:03}"),
        source: format!("ARC {arc:03} set {set}"),
        block: set,
        orientation: Orientation::Front,
        female: false,
        sheet: &sheet,
        ncer: &ncer,
        nanr: &nanr,
        rcmn: &rcmn,
        ramn: &ramn,
        cellinfo: &cellinfo,
        pal_n: &pal,
        pal_s: &pal,
    })?;
    // single palette: no shiny image
    s.ron = s
        .ron
        .lines()
        .filter(|l| !l.contains("shiny:"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    Ok(s)
}

/// Contact sheet: first frame of a selection of sets (ARC 072 sets, then ARC 064 sets 0..6).
pub fn contact_sheet(files: &Path) -> Res<nds_rom::render::Image> {
    let load = |arc: u32| -> Res<Narc> {
        let p = files.join(format!("a/{}/{}/{}", arc / 100, arc / 10 % 10, arc % 10));
        Ok(Narc::parse(&std::fs::read(p)?)?)
    };
    let (a72, a64) = (load(72)?, load(64)?);
    let mut picks: Vec<(&Narc, u32, &str, usize)> = [3, 8, 12, 20, 30, 37, 38, 40, 50, 70, 80, 90]
        .iter()
        .map(|&s| (&a72, 72, "trainer", s))
        .collect();
    picks.extend((0..6).map(|s| (&a64, 64, "player_a", s)));
    let (cell, cols) = (128usize, 6usize);
    let mut sheet = nds_rom::render::Image::new(cols * cell, picks.len().div_ceil(cols) * cell);
    for (n, (narc, arc, prefix, set)) in picks.into_iter().enumerate() {
        let s = convert_set(narc, arc, prefix, set)?;
        let fi = s.first_frame();
        let f = s.frame_image(&s.normal, fi);
        let (ox, oy) = s.frames[fi].origin;
        let (cx, cy) = (
            (n % cols) * cell + cell / 2,
            (n / cols) * cell + cell / 2 + 20,
        );
        for y in 0..f.height {
            for x in 0..f.width {
                let (dx, dy) = (cx as i32 + ox + x as i32, cy as i32 + oy + y as i32);
                let p = f.pixel(x, y);
                if p[3] != 0
                    && dx >= 0
                    && dy >= 0
                    && (dx as usize) < sheet.width
                    && (dy as usize) < sheet.height
                {
                    let i = (dy as usize * sheet.width + dx as usize) * 4;
                    sheet.rgba[i..i + 4].copy_from_slice(&p);
                }
            }
        }
    }
    Ok(sheet)
}

pub struct Report {
    pub sets: usize,
    pub failures: Vec<(String, String)>,
}

pub fn extract(files: &Path, out: &Path) -> Res<Report> {
    let dir = out.join("sprites/trainers");
    std::fs::create_dir_all(&dir)?;
    let mut rep = Report {
        sets: 0,
        failures: vec![],
    };
    for (arc, prefix) in ARCS {
        let p = files.join(format!("a/{}/{}/{}", arc / 100, arc / 10 % 10, arc % 10));
        let narc = Narc::parse(&std::fs::read(p)?)?;
        for set in 0..set_count(&narc) {
            match convert_set(&narc, arc, prefix, set) {
                Ok(s) => {
                    std::fs::write(dir.join(format!("{}.png", s.name)), s.normal.to_png()?)?;
                    std::fs::write(dir.join(format!("{}.atlas.ron", s.name)), &s.ron)?;
                    rep.sets += 1;
                }
                Err(e) => rep
                    .failures
                    .push((format!("{prefix}_{set:03}"), e.to_string())),
            }
        }
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_extract_sprites::pokemon::image_hash;

    fn load(id: u32) -> Option<Narc> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let p = root.join(format!("files/a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
        Narc::parse(&std::fs::read(p).ok()?).ok()
    }

    #[test]
    fn converts_all_trainer_sets() {
        let Some(a) = load(72) else { return };
        assert_eq!(set_count(&a), 95);
        let mut non_empty = 0;
        for set in 0..95 {
            let s = convert_set(&a, 72, "trainer", set).unwrap_or_else(|e| panic!("{set}: {e}"));
            non_empty += (s.normal.opaque_pixels() > 0 && !s.seqs.is_empty()) as usize;
        }
        // the rest are unused placeholder sets with an (almost) empty sheet
        assert!(non_empty >= 80, "only {non_empty} non-empty sets");
        for id in [64, 73] {
            let b = load(id).unwrap();
            assert_eq!(set_count(&b), 6);
            for set in 0..6 {
                convert_set(&b, id, "p", set).unwrap();
            }
        }
    }

    #[test]
    fn trainer_golden() {
        let Some(a) = load(72) else { return };
        let s = convert_set(&a, 72, "trainer", 40).unwrap();
        assert_eq!(image_hash(&s.normal), GOLDEN_T1, "trainer_040 atlas");
    }
    const GOLDEN_T1: &str = "fecaf78cb35dcaf6613cef82a46a77ff8a56842b";
}
