//! `icons` (type / contest / damage-category / ball icons) and `battle-ui` (cell-based UI pieces:
//! HP bar, status labels, markings, ...) from ARC 083 and ARC 055.
//!
//! ARC 083 (172 members; found by rendering, no loader traced: UNVERIFIED which screen owns it,
//! likely the summary screen, which also shows status/ball/type labels):
//! - members 34..=50: 32x16 English type labels in type-id order (Normal..Dark; order UNVERIFIED),
//!   51..=55: contest labels (Cool, Beauty, Cute, Smart, Tough), 56..=58: damage-category icons.
//!   Palette = member 33.
//! - members 112..=136: 25 ball icons, 16x24, palette = member 111.
//! - the rest: `[NCLR.., NCGR.., NCER.., NANR..]` groups; `battle-ui` renders every cell of every
//!   NCER with the NCGR paired by position in the group (pairing UNVERIFIED) and the group's first
//!   NCLR.
//!
//! ARC 055 (29 members): 26 Japanese 32x16 type/contest labels (members 0..=22, 26..=28; palette
//! member 25). A10d found the in-battle archives: ARC 083 is the shared "app common" UI (the
//! accessors at 0x020275E4..0x02027780 return its member ids; `btlv_gauge` opens ARC 83 with
//! palette 11 / gfx 12 / cells 13+n / anims 16+n), and the battle HUD sprites live in ARC 011
//! members 88..=221 (see `gfxarc.rs`, `black extract-sprites battle-hud`).

use super::items::{atlas_ron, load_narc, pack};
use super::pokemon::{member, Res};
use nds_rom::narc::Narc;
use nds_rom::ncer::Ncer;
use nds_rom::ncgr::Ncgr;
use nds_rom::nclr::{bgr555_to_rgba, Nclr};
use nds_rom::render::{render_cell, Image};
use std::path::Path;

pub struct Report {
    pub images: usize,
    pub failures: Vec<(String, String)>,
}

/// Render sequential 8x8 tiles `wt` tiles wide with palette row 0.
pub fn tiles_image(g: &Ncgr, pal: &Nclr, wt: usize) -> Image {
    let nt = g.tile_count();
    let ht = nt.div_ceil(wt);
    let mut img = Image::new(wt * 8, ht * 8);
    for t in 0..nt {
        let Some(px) = g.tile_pixels(t) else { continue };
        let (bx, by) = ((t % wt) * 8, (t / wt) * 8);
        for y in 0..8 {
            for x in 0..8 {
                let i = px[y * 8 + x] as usize;
                if i == 0 {
                    continue;
                }
                let c = bgr555_to_rgba(pal.colors.get(i).copied().unwrap_or(0), true);
                let o = ((by + y) * img.width + bx + x) * 4;
                img.rgba[o..o + 4].copy_from_slice(&c);
            }
        }
    }
    img
}

fn tile_set(
    narc: &Narc,
    pal: usize,
    range: Vec<usize>,
    wt: usize,
    name: impl Fn(usize) -> String,
    rep: &mut Report,
    out: &mut Vec<(String, Image, usize)>,
) {
    let p = match Nclr::parse(&member(narc, pal)) {
        Ok(p) => p,
        Err(e) => {
            rep.failures.push((format!("pal {pal}"), e.to_string()));
            return;
        }
    };
    for i in range {
        match Ncgr::parse(&member(narc, i)) {
            Ok(g) => out.push((name(i), tiles_image(&g, &p, wt), i)),
            Err(e) => rep.failures.push((format!("member {i}"), e.to_string())),
        }
    }
}

const CONTEST: [&str; 5] = ["cool", "beauty", "cute", "smart", "tough"];

pub fn icons(files: &Path) -> Res<(Image, String, Report)> {
    let a83 = load_narc(files, 83)?;
    let a55 = load_narc(files, 55)?;
    let mut rep = Report {
        images: 0,
        failures: vec![],
    };
    let mut v: Vec<(String, Image, usize)> = vec![];
    tile_set(
        &a83,
        33,
        (34..=58).collect(),
        4,
        |i| match i {
            34..=50 => format!("type_{:02}", i - 34),
            51..=55 => format!("contest_{}", CONTEST[i - 51]),
            _ => format!("category_{}", i - 56),
        },
        &mut rep,
        &mut v,
    );
    tile_set(
        &a83,
        111,
        (112..=136).collect(),
        2,
        |i| format!("ball_{:02}", i - 112),
        &mut rep,
        &mut v,
    );
    let jp: Vec<usize> = (0..=22).chain(26..=28).collect();
    tile_set(
        &a55,
        25,
        jp,
        4,
        |i| format!("type_jp_{i:02}"),
        &mut rep,
        &mut v,
    );
    rep.images = v.len();
    let images: Vec<(String, Image)> = v.iter().map(|(n, i, _)| (n.clone(), i.clone())).collect();
    let (atlas, rects) = pack(&images, 512);
    let entries: Vec<_> = v
        .iter()
        .zip(&rects)
        .map(|((n, _, m), r)| {
            let arc = if n.ends_with(|c: char| c.is_ascii_digit()) && n.starts_with("type_jp") {
                55
            } else {
                83
            };
            (n.clone(), *r, format!(", arc: {arc}, member: {m}"))
        })
        .collect();
    let ron = atlas_ron(
        "ARC 083 / ARC 055",
        "icons",
        &atlas,
        &entries,
        "type_NN order = type id (UNVERIFIED); category_0..2 order UNVERIFIED; ball_NN -> item mapping UNVERIFIED",
    );
    Ok((atlas, ron, rep))
}

/// Cell-based UI pieces of ARC 083.
pub fn battle_ui(files: &Path) -> Res<(Image, String, Report)> {
    let narc = load_narc(files, 83)?;
    let mut rep = Report {
        images: 0,
        failures: vec![],
    };
    let tag = |i: usize| {
        narc.file(i)
            .get(..4)
            .map(|t| t.to_vec())
            .unwrap_or_default()
    };
    // member ranges handled by `icons`
    let skip = |i: usize| (33..=58).contains(&i) || (111..=136).contains(&i);
    let mut pal: Option<usize> = None;
    let mut gfx: Vec<usize> = vec![];
    let mut cells_run = 0usize;
    let mut images: Vec<(String, Image)> = vec![];
    let mut meta = vec![];
    let n = narc.len();
    let mut i = 0;
    while i < n {
        let t = tag(i);
        if skip(i) {
            i += 1;
            continue;
        }
        match t.as_slice() {
            b"RLCN" => {
                if i == 0 || tag(i - 1) != b"RLCN" {
                    pal = Some(i);
                    gfx.clear();
                    cells_run = 0;
                }
            }
            b"RGCN" => {
                if cells_run > 0 {
                    gfx.clear();
                    cells_run = 0;
                }
                gfx.push(i);
            }
            b"RECN" => {
                let g = gfx.get(cells_run).or(gfx.last()).copied();
                if let (Some(g), Some(p)) = (g, pal) {
                    match (
                        Ncer::parse(&member(&narc, i)),
                        Ncgr::parse(&member(&narc, g)),
                        Nclr::parse(&member(&narc, p)),
                    ) {
                        (Ok(ce), Ok(gr), Ok(pl)) => {
                            for c in 0..ce.cells.len() {
                                if let Some((im, _)) = render_cell(&ce, c, &gr, &pl) {
                                    if im.opaque_pixels() > 0 {
                                        images.push((format!("ui_{i:03}_{c:02}"), im));
                                        meta.push((i, c, g, p));
                                    }
                                }
                            }
                        }
                        (e1, e2, e3) => rep.failures.push((
                            format!("member {i}"),
                            format!(
                                "{:?} {:?} {:?}",
                                e1.err().map(|e| e.to_string()),
                                e2.err().map(|e| e.to_string()),
                                e3.err().map(|e| e.to_string())
                            ),
                        )),
                    }
                }
                cells_run += 1;
            }
            _ => {}
        }
        i += 1;
    }
    rep.images = images.len();
    let (atlas, rects) = pack(&images, 512);
    let entries: Vec<_> = images
        .iter()
        .zip(&rects)
        .zip(&meta)
        .map(|(((nm, _), r), (m, c, g, p))| {
            (
                nm.clone(),
                *r,
                format!(", ncer: {m}, cell: {c}, ncgr: {g}, nclr: {p}"),
            )
        })
        .collect();
    let ron = atlas_ron(
        "ARC 083",
        "battle_ui",
        &atlas,
        &entries,
        "every cell of every NCER; NCGR/NCLR pairing by position is UNVERIFIED; ARC/screen ownership UNVERIFIED",
    );
    Ok((atlas, ron, rep))
}

pub fn extract(files: &Path, out: &Path, which: &str) -> Res<Report> {
    let (dir, name) = match which {
        "icons" => ("icons", "icons"),
        _ => ("battle-ui", "battle_ui"),
    };
    let d = out.join("sprites").join(dir);
    std::fs::create_dir_all(&d)?;
    let (atlas, ron, rep) = if which == "icons" {
        icons(files)?
    } else {
        battle_ui(files)?
    };
    std::fs::write(d.join(format!("{name}.png")), atlas.to_png()?)?;
    std::fs::write(d.join(format!("{name}.atlas.ron")), ron)?;
    Ok(rep)
}

/// Contact sheet: item atlas, icon atlas and battle-UI atlas stacked vertically.
pub fn contact_sheet(files: &Path) -> Res<Image> {
    let parts = [
        super::items::build(files)?.0,
        icons(files)?.0,
        battle_ui(files)?.0,
    ];
    let w = parts.iter().map(|p| p.width).max().unwrap_or(1);
    let h: usize = parts.iter().map(|p| p.height + 4).sum();
    let mut img = Image::new(w, h);
    let mut y = 0;
    for p in &parts {
        for yy in 0..p.height {
            let d = ((y + yy) * w) * 4;
            img.rgba[d..d + p.width * 4]
                .copy_from_slice(&p.rgba[yy * p.width * 4..(yy + 1) * p.width * 4]);
        }
        y += p.height + 4;
    }
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd_extract_sprites::pokemon::image_hash;

    fn files() -> Option<std::path::PathBuf> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let f = root.join("files");
        f.join("a/0/8/3").exists().then_some(f)
    }

    #[test]
    fn converts_icons_and_ui() {
        let Some(f) = files() else { return };
        let (a, ron, rep) = icons(&f).unwrap();
        assert!(rep.failures.is_empty(), "{:?}", rep.failures);
        assert_eq!(rep.images, 25 + 25 + 26);
        assert!(a.opaque_pixels() > 5_000);
        assert!(ron.contains("type_10"));
        let (a, _, rep) = battle_ui(&f).unwrap();
        assert!(rep.failures.is_empty(), "{:?}", rep.failures);
        assert!(rep.images > 40, "{}", rep.images);
        assert!(a.opaque_pixels() > 5_000);
    }

    #[test]
    fn type_icon_golden() {
        let Some(f) = files() else { return };
        let a83 = load_narc(&f, 83).unwrap();
        let pal = Nclr::parse(&member(&a83, 33)).unwrap();
        // type id 9 = Fire
        let g = Ncgr::parse(&member(&a83, 34 + 9)).unwrap();
        let im = tiles_image(&g, &pal, 4);
        assert_eq!((im.width, im.height), (32, 16));
        assert_eq!(image_hash(&im), GOLDEN_FIRE, "Fire type icon");
    }
    const GOLDEN_FIRE: &str = "0e9cf0ec956fed7d670799512f3ab807420d8799";
}
