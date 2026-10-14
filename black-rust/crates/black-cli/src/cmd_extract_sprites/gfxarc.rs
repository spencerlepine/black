//! Generic walker for UI/effect archives made of `NCLR / NCGR / NCER / NANR / NSCR` members in
//! "set" order (A10d), plus the per-category entry points `battle-hud` and `menus`.
//!
//! Pairing rules (no loader tables were traced for these archives, so ALL pairings are UNVERIFIED
//! heuristics, checked only by a contact sheet):
//! - A bank of cells (NCER) uses the graphics (NCGR) from the run of NCGRs directly before its run
//!   of NCERs, aligned to the END of that run (`[gfx, gfx, cell]` pairs the second gfx; the first
//!   one is then a stand-alone tile sheet).
//! - An animation bank (NANR) belongs to the k-th NCER of the previous NCER run.
//! - A screen (NSCR) uses the nearest preceding NCGR.
//! - Palette = the nearest NCLR by member distance; ties go to the preceding one unless that one is
//!   part of a run of three or more NCLRs (a palette bank, not an owner).
//!
//! What the archives are (found by following `FUN_020490F4(arc, heap)` open calls in the overlays,
//! see `docs/migration/log/A10.md`):
//! - ARC 011 = battle graphics: members 0..=87 are 3D models (stage/field), 88..=221 are the
//!   battle 2D set (HP box / gauge sprites, command-menu pieces, backgrounds, message windows).
//!   Opened by `btlv_input`, `btlv_gauge`, `btlv_b_gauge`, `btlv_timer`, `btlv_finger_cursor` in
//!   overlay 94 with member constants 115..=122, 162, 177..=192 in `FUN_0204A6C8/ABF0/AF28/AEF4`.
//! - ARC 083 = shared "app common" UI (also battle: `FUN_020275E4` returns 83), see `ui.rs`.
//! - ARC 088 = bag (overlay 70), ARC 076 = party list (overlay 91), ARC 078 = summary (overlay
//!   131), ARC 116 (+180 palettes?) = PC box (overlay 170).

use super::items::{atlas_ron, load_narc, pack};
use super::pokemon::{member, Res};
use super::ui::tiles_image;
use nds_rom::nanr::Nanr;
use nds_rom::narc::Narc;
use nds_rom::ncer::Ncer;
use nds_rom::ncgr::Ncgr;
use nds_rom::nclr::Nclr;
use nds_rom::nscr::Nscr;
use nds_rom::render::{render_cell, render_screen, Image};
use std::fmt::Write as _;
use std::ops::RangeInclusive;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// screens use an earlier gfx; palettes are the nearest NCLR
    Default,
    /// screens come before their gfx (ARC 116)
    ScreenFirst,
    /// the i-th NCGR uses the i-th NCLR (ARC 180)
    ParallelPals,
}

pub struct Item {
    pub name: String,
    pub img: Image,
    pub extra: String,
}

pub struct AnimOut {
    pub nanr: usize,
    pub ncer: usize,
    /// (mode, frames of (cell, duration))
    pub seqs: Vec<(u32, Vec<(u16, u16)>)>,
}

#[derive(Default)]
pub struct Walked {
    pub items: Vec<Item>,
    pub anims: Vec<AnimOut>,
    pub failures: Vec<(String, String)>,
    pub members: usize,
}

fn tag(b: &[u8]) -> [u8; 4] {
    b.get(..4).map_or([0; 4], |t| t.try_into().unwrap())
}

/// Nearest NCLR to member `i` (see module docs).
fn nearest_pal(tags: &[[u8; 4]], i: usize, lo: usize, hi: usize) -> Option<usize> {
    let is_pal = |j: usize| tags[j] == *b"RLCN";
    let prev = (lo..=i.min(hi)).rev().find(|&j| is_pal(j));
    let next = (i..=hi).find(|&j| is_pal(j));
    match (prev, next) {
        (Some(p), Some(n)) => {
            if p == n {
                return Some(p);
            }
            let run = (lo..=p).rev().take_while(|&j| is_pal(j)).count().max(1);
            let (dp, dn) = (i - p, n - i);
            if dp < dn || (dp == dn && run < 3) {
                Some(p)
            } else {
                Some(n)
            }
        }
        (p, n) => p.or(n),
    }
}

/// Walk `range` of `narc` and render everything renderable.
pub fn walk(narc: &Narc, range: RangeInclusive<usize>, mode: Mode) -> Walked {
    let hi = (*range.end()).min(narc.len().saturating_sub(1));
    let lo = *range.start();
    let mut w = Walked::default();
    let data: Vec<Vec<u8>> = (0..=hi)
        .map(|i| if i >= lo { member(narc, i) } else { vec![] })
        .collect();
    let tags: Vec<[u8; 4]> = data.iter().map(|d| tag(d)).collect();
    w.members = hi + 1 - lo.min(hi + 1);
    let mut used_gfx = vec![false; hi + 1];
    let pals: Vec<usize> = (lo..=hi).filter(|&j| tags[j] == *b"RLCN").collect();
    let gfxs: Vec<usize> = (lo..=hi).filter(|&j| tags[j] == *b"RGCN").collect();
    let pal_for = |g: usize| -> Option<usize> {
        if mode == Mode::ParallelPals && pals.len() == gfxs.len() {
            return gfxs.iter().position(|&x| x == g).map(|k| pals[k]);
        }
        nearest_pal(&tags, g, lo, hi)
    };
    let mut last_ncer_run: Vec<usize> = vec![];
    let mut i = lo;
    while i <= hi {
        match &tags[i] {
            b"RECN" => {
                let start = i;
                let mut run = vec![];
                while i <= hi && tags[i] == *b"RECN" {
                    run.push(i);
                    i += 1;
                }
                // nearest gfx run before the cell run (skipping palettes, anims, screens)
                let mut g = start;
                while g > lo && tags[g - 1] != *b"RGCN" {
                    g -= 1;
                }
                let mut gfx_run = vec![];
                while g > lo && tags[g - 1] == *b"RGCN" {
                    g -= 1;
                    gfx_run.push(g);
                }
                gfx_run.reverse();
                for (k, &m) in run.iter().enumerate() {
                    let gi = if gfx_run.is_empty() {
                        None
                    } else if gfx_run.len() >= run.len() {
                        Some(gfx_run[gfx_run.len() - run.len() + k])
                    } else {
                        Some(gfx_run[k.min(gfx_run.len() - 1)])
                    };
                    let Some(gi) = gi else {
                        w.failures
                            .push((format!("ncer {m}"), "no NCGR before it".into()));
                        continue;
                    };
                    used_gfx[gi] = true;
                    let Some(pi) = pal_for(gi) else {
                        w.failures.push((format!("ncer {m}"), "no NCLR".into()));
                        continue;
                    };
                    match (
                        Ncer::parse(&data[m]),
                        Ncgr::parse(&data[gi]),
                        Nclr::parse(&data[pi]),
                    ) {
                        (Ok(ce), Ok(gr), Ok(pl)) => {
                            for c in 0..ce.cells.len() {
                                if let Some((im, org)) = render_cell(&ce, c, &gr, &pl) {
                                    if im.opaque_pixels() > 0 {
                                        w.items.push(Item {
                                            name: format!("cell_{m:03}_{c:02}"),
                                            img: im,
                                            extra: format!(
                                                ", ncer: {m}, cell: {c}, ncgr: {gi}, nclr: {pi}, origin: ({}, {})",
                                                org.0, org.1
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                        (a, b, c) => w.failures.push((
                            format!("ncer {m}"),
                            format!(
                                "{:?} {:?} {:?}",
                                a.err().map(|e| e.to_string()),
                                b.err().map(|e| e.to_string()),
                                c.err().map(|e| e.to_string())
                            ),
                        )),
                    }
                }
                last_ncer_run = run;
            }
            b"RNAN" => {
                let start = i;
                let mut run = vec![];
                while i <= hi && tags[i] == *b"RNAN" {
                    run.push(i);
                    i += 1;
                }
                let _ = start;
                // cell runs: previous (already seen) and next one in member order
                let next_run: Vec<usize> = {
                    let mut j = i;
                    while j <= hi && tags[j] != *b"RECN" {
                        j += 1;
                    }
                    let mut r = vec![];
                    while j <= hi && tags[j] == *b"RECN" {
                        r.push(j);
                        j += 1;
                    }
                    r
                };
                let use_next = match (last_ncer_run.last(), next_run.first()) {
                    (Some(&p), Some(&n)) => n - (i - 1) < (i - 1) - p,
                    (None, Some(_)) => true,
                    _ => false,
                };
                let cells: &Vec<usize> = if use_next { &next_run } else { &last_ncer_run };
                for (k, &m) in run.iter().enumerate() {
                    match Nanr::parse(&data[m]) {
                        Ok(a) => {
                            let ncer = cells.get(k).or(cells.last()).copied().unwrap_or(0);
                            w.anims.push(AnimOut {
                                nanr: m,
                                ncer,
                                seqs: a
                                    .sequences
                                    .iter()
                                    .map(|s| {
                                        (
                                            s.mode,
                                            s.frames.iter().map(|f| (f.cell, f.duration)).collect(),
                                        )
                                    })
                                    .collect(),
                            });
                        }
                        Err(e) => w.failures.push((format!("nanr {m}"), e.to_string())),
                    }
                }
            }
            b"RCSN" => {
                let m = i;
                i += 1;
                let need = Nscr::parse(&data[m])
                    .map(|sc| {
                        sc.entries
                            .iter()
                            .map(|e| e.tile as usize + 1)
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                let fits = |j: usize| {
                    Ncgr::parse(&data[j])
                        .map(|g| g.tile_count() >= need)
                        .unwrap_or(false)
                };
                let order: Vec<usize> = if mode == Mode::ScreenFirst {
                    (m + 1..=hi).filter(|&j| tags[j] == *b"RGCN").collect()
                } else {
                    (lo..m).rev().filter(|&j| tags[j] == *b"RGCN").collect()
                };
                let gi = order
                    .iter()
                    .copied()
                    .find(|&j| fits(j))
                    .or_else(|| order.first().copied());
                let pi = gi.and_then(pal_for);
                let (Some(gi), Some(pi)) = (gi, pi) else {
                    w.failures
                        .push((format!("nscr {m}"), "no NCGR/NCLR".into()));
                    continue;
                };
                used_gfx[gi] = true;
                match (
                    Nscr::parse(&data[m]),
                    Ncgr::parse(&data[gi]),
                    Nclr::parse(&data[pi]),
                ) {
                    (Ok(sc), Ok(gr), Ok(pl)) => {
                        let im = render_screen(&gr, &pl, &sc);
                        if im.opaque_pixels() > 0 {
                            w.items.push(Item {
                                name: format!("screen_{m:03}"),
                                img: im,
                                extra: format!(", nscr: {m}, ncgr: {gi}, nclr: {pi}"),
                            });
                        }
                    }
                    (a, b, c) => w.failures.push((
                        format!("nscr {m}"),
                        format!(
                            "{:?} {:?} {:?}",
                            a.err().map(|e| e.to_string()),
                            b.err().map(|e| e.to_string()),
                            c.err().map(|e| e.to_string())
                        ),
                    )),
                }
            }
            _ => i += 1,
        }
    }
    // graphics not used by any cell bank or screen: plain tile sheets
    for g in lo..=hi {
        if tags[g] != *b"RGCN" || used_gfx[g] {
            continue;
        }
        let Some(pi) = pal_for(g) else {
            continue;
        };
        match (Ncgr::parse(&data[g]), Nclr::parse(&data[pi])) {
            (Ok(gr), Ok(pl)) => {
                let wt = gr.tiles_x.map_or(16, usize::from).clamp(1, 32);
                let im = tiles_image(&gr, &pl, wt);
                if im.opaque_pixels() > 0 {
                    w.items.push(Item {
                        name: format!("tiles_{g:03}"),
                        img: im,
                        extra: format!(", ncgr: {g}, nclr: {pi}"),
                    });
                }
            }
            (a, b) => w.failures.push((
                format!("ncgr {g}"),
                format!(
                    "{:?} {:?}",
                    a.err().map(|e| e.to_string()),
                    b.err().map(|e| e.to_string())
                ),
            )),
        }
    }
    w
}

/// Atlas image + RON for a walked archive.
pub fn atlas(arc: u32, name: &str, w: &Walked, note: &str) -> (Image, String) {
    let images: Vec<(String, Image)> = w
        .items
        .iter()
        .map(|i| (i.name.clone(), i.img.clone()))
        .collect();
    let (atlas, rects) = pack(&images, 1024);
    let entries: Vec<_> = w
        .items
        .iter()
        .zip(&rects)
        .map(|(it, r)| (it.name.clone(), *r, it.extra.clone()))
        .collect();
    let mut ron = atlas_ron(&format!("ARC {arc:03}"), name, &atlas, &entries, note);
    // animations section before the closing paren
    let mut a = String::from("    animations: [\n");
    for an in &w.anims {
        let _ = write!(
            a,
            "        (nanr: {}, ncer: {}, sequences: [",
            an.nanr, an.ncer
        );
        for (mode, fr) in &an.seqs {
            let _ = write!(a, "(mode: {mode}, frames: [");
            for (c, d) in fr {
                let _ = write!(a, "({c}, {d}),");
            }
            let _ = write!(a, "]),");
        }
        let _ = writeln!(a, "]),");
    }
    a.push_str("    ],\n");
    if let Some(p) = ron.rfind(')') {
        ron.insert_str(p, &a);
    }
    (atlas, ron)
}

pub struct Report {
    pub images: usize,
    pub files: usize,
    pub failures: Vec<(String, String)>,
}

/// (category, [(arc, member range, file stem, note)])
#[allow(clippy::type_complexity)]
pub fn plan(
    cat: &str,
) -> Option<Vec<(u32, RangeInclusive<usize>, &'static str, &'static str, Mode)>> {
    Some(match cat {
        "battle-hud" => vec![(
            11,
            88..=221,
            "battle_hud",
            "ARC 011 2D set (members 0..=87 are 3D models, not converted); cell/NCGR/NCLR pairing and the role of each piece (HP box, command buttons, backgrounds) UNVERIFIED",
            Mode::Default,
        )],
        "menus" => vec![
            (88, 0..=usize::MAX, "bag_arc088", "bag screen (overlay 70 opens ARC 88); pairing UNVERIFIED", Mode::Default),
            (76, 0..=usize::MAX, "party_arc076", "party list (overlay 91 opens ARC 76); pairing UNVERIFIED", Mode::Default),
            (78, 0..=usize::MAX, "summary_arc078", "summary screen (overlay 131 opens ARC 78); pairing UNVERIFIED", Mode::Default),
            (116, 0..=usize::MAX, "box_arc116", "PC box (overlay 170 opens ARC 116); pairing UNVERIFIED", Mode::ScreenFirst),
            (180, 0..=usize::MAX, "box_arc180", "overlay 170 uses ARC 180 as a palette source; UNVERIFIED", Mode::ParallelPals),
        ],
        _ => return None,
    })
}

/// (file stem, ARC id, walked archive, atlas image, atlas RON)
pub type Built = (String, u32, Walked, Image, String);

pub fn build_all(files: &Path, cat: &str) -> Res<Vec<Built>> {
    let p = plan(cat).ok_or_else(|| format!("unknown category {cat}"))?;
    let mut out = vec![];
    for (arc, range, stem, note, mode) in p {
        let narc = load_narc(files, arc)?;
        let w = walk(&narc, range, mode);
        let (img, ron) = atlas(arc, stem, &w, note);
        out.push((stem.to_string(), arc, w, img, ron));
    }
    Ok(out)
}

pub fn extract(files: &Path, out: &Path, cat: &str) -> Res<Report> {
    let dir = out.join("sprites").join(cat);
    std::fs::create_dir_all(&dir)?;
    let mut rep = Report {
        images: 0,
        files: 0,
        failures: vec![],
    };
    for (stem, arc, w, img, ron) in build_all(files, cat)? {
        std::fs::write(dir.join(format!("{stem}.png")), img.to_png()?)?;
        std::fs::write(dir.join(format!("{stem}.atlas.ron")), ron)?;
        rep.images += w.items.len();
        rep.files += 1;
        rep.failures.extend(
            w.failures
                .into_iter()
                .map(|(k, e)| (format!("ARC {arc} {k}"), e)),
        );
    }
    Ok(rep)
}

/// Thumbnail grid: every image is shrunk by an integer factor to fit `cell` x `cell`, on a grey
/// checker so transparency shows.
pub fn thumb_grid(images: &[&Image], cols: usize, cell: usize) -> Image {
    let rows = images.len().div_ceil(cols).max(1);
    let (gw, gh) = (cols * (cell + 1), rows * (cell + 1));
    let mut g = Image::new(gw, gh);
    for y in 0..gh {
        for x in 0..gw {
            let v = if ((x / 8) + (y / 8)) % 2 == 0 { 70 } else { 90 };
            let o = (y * gw + x) * 4;
            g.rgba[o..o + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    for (n, im) in images.iter().enumerate() {
        let k = im.width.max(im.height).div_ceil(cell).max(1);
        let (ox, oy) = ((n % cols) * (cell + 1), (n / cols) * (cell + 1));
        for y in 0..(im.height / k) {
            for x in 0..(im.width / k) {
                let p = im.pixel(x * k, y * k);
                if p[3] == 0 {
                    continue;
                }
                let o = ((oy + y) * gw + ox + x) * 4;
                g.rgba[o..o + 4].copy_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
    }
    g
}

pub fn contact_sheet(files: &Path, cat: &str) -> Res<Image> {
    let all = build_all(files, cat)?;
    let imgs: Vec<&Image> = all
        .iter()
        .flat_map(|a| a.2.items.iter().map(|i| &i.img))
        .collect();
    Ok(thumb_grid(&imgs, 24, 48))
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
        if f.join("a/0/1/1").exists() {
            Some(f)
        } else {
            nds_rom::testenv::rom_missing("black-rom/files");
            None
        }
    }

    #[test]
    fn battle_hud_converts() {
        let Some(f) = files() else { return };
        let all = build_all(&f, "battle-hud").unwrap();
        let (_, _, w, img, ron) = &all[0];
        assert!(w.failures.is_empty(), "{:?}", w.failures);
        assert!(w.items.len() > 100, "{}", w.items.len());
        assert!(img.opaque_pixels() > 20_000);
        assert!(ron.contains("animations"));
        // HP box graphics 189/190/191 are used together by btlv_b_gauge
        assert!(w
            .items
            .iter()
            .any(|i| i.extra.contains("ncer: 191, cell: 0, ncgr: 189, nclr: 190")));
    }

    #[test]
    fn menus_convert() {
        let Some(f) = files() else { return };
        let all = build_all(&f, "menus").unwrap();
        assert_eq!(all.len(), 5);
        for (stem, _, w, img, _) in &all {
            assert!(w.failures.is_empty(), "{stem}: {:?}", w.failures);
            assert!(!w.items.is_empty(), "{stem}");
            assert!(img.opaque_pixels() > 1_000, "{stem}");
        }
    }
}
