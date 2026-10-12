//! `black font2 <narc> <out_dir>`: render every NFTR font of the text-font NARC (a/0/2/3).

use nds_rom::error::{create_dir, read_file, write_file};
use nds_rom::{gffont, narc::Narc};
use std::path::Path;

pub fn run(narc: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let narc = Narc::parse(&read_file(narc)?)?;
    create_dir(out_dir)?;
    for (i, f) in gffont::fonts_from_narc(&narc) {
        let img = f.atlas(64);
        let out = out_dir.join(format!("font{i}.png"));
        write_file(&out, &img.to_png()?)?;
        let w = |c: char| f.char_width(c as u16).map_or("-".into(), |w| w.to_string());
        println!(
            "member {i}: {} glyphs, {} codes, cell {}x{}, {}bpp, line feed {}; widths A={} ' '={} i={} W={} -> {}",
            f.len(),
            f.code_count(),
            f.cell_w,
            f.cell_h,
            f.bpp,
            f.line_feed,
            w('A'),
            w(' '),
            w('i'),
            w('W'),
            out.display()
        );
    }
    Ok(())
}
