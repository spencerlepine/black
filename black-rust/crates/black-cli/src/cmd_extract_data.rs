//! `black extract-data <out>`: ROM tables -> `<out>/data/*.ron` (A12).

use black_data::extract::{extract_data, RomSource};
use std::path::Path;

/// `rom_root` is the `black-rom` directory (holding `files/` and `build/black.us/`).
pub fn run(rom_root: &Path, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for p in extract_data(&RomSource::from_root(rom_root), out)? {
        println!("wrote {}", p.display());
    }
    Ok(())
}
