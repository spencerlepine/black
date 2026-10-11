//! Reading, building and comparing Nintendo DS / DSi ROMs.
//!
//! The crate is a Rust port of the ROM tooling of the Pokemon Black decompilation
//! (`make_rom.py`, `blz.py`, `compress_modules.py`, `banner.py` and the `compare_*` scripts).

pub mod banner;
pub mod blz;
pub mod build;
pub mod compare;
pub mod crc;
pub mod error;
pub mod fnt;
pub mod header;
pub mod module;
pub mod overlay;
pub mod pack;
pub mod rom;

pub use error::{Error, Result};
pub use header::Header;
pub use rom::Rom;

/// Lower-case hex SHA-1 of `data`.
pub fn sha1_hex(data: &[u8]) -> String {
    sha1_smol::Sha1::from(data).digest().to_string()
}
