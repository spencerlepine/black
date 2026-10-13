//! `gfl_font.dat` against the real ROM. Skipped when the ROM files are absent.

use nds_rom::font::Font;
use nds_rom::sha1_hex;
use std::path::PathBuf;

fn path() -> PathBuf {
    std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
        .join("files/gfl_font.dat")
}

#[test]
fn parses_gfl_font() {
    let Ok(bytes) = std::fs::read(path()) else {
        nds_rom::testenv::rom_missing("gfl_font.dat not found");
        return;
    };
    let f = Font::parse(&bytes).unwrap();
    assert_eq!((f.len(), f.width, f.height), (288, 8, 8));
    let atlas = f.atlas(32);
    assert_eq!((atlas.width, atlas.height), (256, 72));
    assert!(f.glyph_width(0).is_some() && f.glyph_width(288).is_none());
    let widths: Vec<_> = (0..6).map(|g| f.glyph_width(g).unwrap()).collect();
    assert_eq!(widths, [6, 8, 6, 8, 6, 8]);
    assert_eq!(
        sha1_hex(&atlas.rgba),
        "a3ecd0237d7c43803e54871976903e0140f63c83"
    );
}
