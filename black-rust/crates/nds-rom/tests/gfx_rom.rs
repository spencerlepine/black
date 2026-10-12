//! 2D graphics readers against every NARC of the real ROM. Skipped when the ROM files are absent.

use nds_rom::nanr::Nanr;
use nds_rom::narc::Narc;
use nds_rom::ncer::Ncer;
use nds_rom::ncgr::Ncgr;
use nds_rom::nclr::Nclr;
use nds_rom::nscr::Nscr;
use nds_rom::{lz, render, sha1_hex};
use std::path::PathBuf;

fn files_dir() -> PathBuf {
    std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
        .join("files")
}

fn narc_path(id: u32) -> PathBuf {
    files_dir().join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10))
}

fn member(narc: &Narc, i: usize) -> Vec<u8> {
    let f = narc.file(i);
    lz::decompress_any(f).unwrap_or_else(|| f.to_vec())
}

#[test]
fn parses_every_graphics_file() {
    let dir = files_dir().join("a");
    if !dir.exists() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let mut counts = [0usize; 5];
    let mut failures = Vec::new();
    for id in 0..1000u32 {
        let Ok(bytes) = std::fs::read(narc_path(id)) else {
            continue;
        };
        let Ok(narc) = Narc::parse(&bytes) else {
            continue;
        };
        for i in 0..narc.len() {
            let d = member(&narc, i);
            let (kind, r) = match d.get(..4) {
                Some(b"RLCN") => (0, Nclr::parse(&d).map(|_| ())),
                Some(b"RGCN") => (1, Ncgr::parse(&d).map(|_| ())),
                Some(b"RCSN") => (2, Nscr::parse(&d).map(|_| ())),
                Some(b"RECN") => (3, Ncer::parse(&d).map(|_| ())),
                Some(b"RNAN") => (4, Nanr::parse(&d).map(|_| ())),
                _ => continue,
            };
            counts[kind] += 1;
            if let Err(e) = r {
                failures.push(format!("arc {id} member {i}: {e}"));
            }
        }
    }
    eprintln!(
        "parsed NCLR {} NCGR {} NSCR {} NCER {} NANR {}",
        counts[0], counts[1], counts[2], counts[3], counts[4]
    );
    assert!(
        failures.is_empty(),
        "{} failures: {:?}",
        failures.len(),
        &failures[..failures.len().min(10)]
    );
    assert!(counts.iter().all(|&c| c > 0));
}

fn open(id: u32) -> Option<Narc> {
    Narc::parse(&std::fs::read(narc_path(id)).ok()?).ok()
}

/// Bulbasaur's block of ARC 004: 29 tiled 12x12 4bpp, 31 scanned 32x16 4bpp sheet, 33 cell bank
/// (2D mapping, 5 cells), 34 animations, 38 palette (16 colours).
#[test]
fn golden_pokemon_sprite() {
    let Some(narc) = open(4) else {
        eprintln!("skipping: ARC 004 not found");
        return;
    };
    let pal = Nclr::parse(&member(&narc, 38)).unwrap();
    assert_eq!(pal.colors.len(), 16);
    assert_eq!(pal.colors[1], 32767);
    assert_eq!(pal.rgba(0, 1), [255, 255, 255, 255]);
    assert_eq!(pal.rgba(0, 0)[3], 0);

    let g = Ncgr::parse(&member(&narc, 31)).unwrap();
    assert_eq!(
        (g.tiles_x, g.tiles_y, g.bpp, g.scanned),
        (Some(32), Some(16), 4, true)
    );
    assert_eq!(g.tile_count(), 512);
    let img = render::render_tiles(&g, &pal, 0);
    assert_eq!((img.width, img.height), (256, 128));
    assert_eq!(img.opaque_pixels(), 1333);
    assert_eq!(
        sha1_hex(&img.rgba),
        "05f81ebe2526cb29e8591c1e1838c76bbcd86593"
    );

    let tiled = Ncgr::parse(&member(&narc, 29)).unwrap();
    assert_eq!(
        (tiled.tiles_x, tiled.scanned, tiled.tile_count()),
        (Some(12), false, 144)
    );

    let cells = Ncer::parse(&member(&narc, 33)).unwrap();
    assert_eq!((cells.cells.len(), cells.mapping_mode), (5, 4));
    assert_eq!(cells.cells[0].oams.len(), 2);
    assert!(cells.cells[0]
        .oams
        .iter()
        .all(|o| o.rotscale && o.double_size));
    let (head, origin) = render::render_cell(&cells, 0, &g, &pal).unwrap();
    assert_eq!((head.width, head.height, origin), (32, 24, (-16, -16)));
    assert_eq!(head.opaque_pixels(), 349);
    assert_eq!(
        sha1_hex(&head.rgba),
        "ac77c70ca8f4ce5ad5e6bc3000f7bad71c363b74"
    );

    let anim = Nanr::parse(&member(&narc, 34)).unwrap();
    assert!(!anim.sequences.is_empty());
    assert_eq!(anim.sequences[0].frames.len(), 12);
    assert!(anim
        .sequences
        .iter()
        .flat_map(|s| &s.frames)
        .all(|f| (f.cell as usize) < 5));

    let png = head.to_png().unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

/// A system NARC with 256-colour palettes, 8bpp tiles and 256x256 screens (ARC 040).
#[test]
fn golden_screen() {
    let Some(narc) = open(40) else {
        eprintln!("skipping: ARC 040 not found");
        return;
    };
    let scr = Nscr::parse(&member(&narc, 34)).unwrap();
    assert_eq!((scr.width, scr.height, scr.entries.len()), (256, 256, 1024));
    let g = Ncgr::parse(&member(&narc, 32)).unwrap();
    assert_eq!((g.bpp, g.tile_count()), (8, 512));
    let pal = Nclr::parse(&member(&narc, 0)).unwrap();
    assert_eq!(pal.colors.len(), 256);
    let img = render::render_screen(&g, &pal, &scr);
    assert_eq!(img.opaque_pixels(), 48128);
    assert_eq!(
        sha1_hex(&img.rgba),
        "4165cdf8d83a25f7c6d6d566b4f396ce63783ce1"
    );
}
