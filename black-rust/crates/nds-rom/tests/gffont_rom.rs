//! NFTR text fonts (a/0/2/3) against the real ROM. Skipped when the ROM files are absent.

use nds_rom::gffont::{fonts_from_narc, GfFont};
use nds_rom::narc::Narc;
use nds_rom::sha1_hex;
use std::path::PathBuf;

fn narc() -> Option<Narc> {
    let root = std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"));
    let bytes = std::fs::read(root.join("files/a/0/2/3")).ok()?;
    Some(Narc::parse(&bytes).unwrap())
}

#[test]
fn parses_all_text_fonts() {
    let Some(n) = narc() else {
        eprintln!("skipping: a/0/2/3 not found");
        return;
    };
    let fonts = fonts_from_narc(&n);
    assert_eq!(
        fonts.iter().map(|f| f.0).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );
    let dims: Vec<_> = fonts
        .iter()
        .map(|(_, f)| (f.len(), f.code_count(), f.cell_w, f.cell_h, f.bpp))
        .collect();
    assert_eq!(
        dims,
        [
            (5064, 5064, 12, 15, 2),
            (5064, 5064, 10, 10, 2),
            (5064, 5064, 11, 13, 2),
            (32, 32, 13, 8, 2),
            (41, 41, 12, 12, 2)
        ]
    );
    // Advances of 'A', ' ', 'i', 'W' in the three text fonts.
    let w = |f: &GfFont| ['A', ' ', 'i', 'W'].map(|c| f.char_width(c as u16).unwrap_or(0));
    assert_eq!(w(&fonts[0].1), [6, 4, 3, 6]);
    assert_eq!(w(&fonts[1].1), [5, 2, 3, 5]);
    assert_eq!(w(&fonts[2].1), [5, 2, 3, 5]);
    let atlas = fonts[0].1.atlas(64);
    assert_eq!((atlas.width, atlas.height), (768, 1200));
    assert_eq!(
        sha1_hex(&atlas.rgba),
        "8f524e6a69800b7f187848e1a0a4e7f2992ce238"
    );
}
