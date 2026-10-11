//! Error type shared by every module of the crate.

use std::path::PathBuf;

/// Everything that can go wrong while reading, building or comparing ROMs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{what} is truncated: need {need:#x} bytes at {offset:#x}, have {have:#x}")]
    Truncated {
        what: &'static str,
        offset: usize,
        need: usize,
        have: usize,
    },
    #[error("invalid {what}: {detail}")]
    Invalid { what: &'static str, detail: String },
    #[error("corrupt BLZ stream: {0}")]
    Blz(String),
    #[error("no NitroCode magic found, cannot locate ModuleParams")]
    NoModuleParams,
    #[error("autoload list of {0:#x} bytes fits no known entry size")]
    BadAutoloadList(u32),
    #[error("{what} must be placed at {at:#x} but the ROM is already {len:#x} bytes long")]
    Overlap { what: String, at: usize, len: usize },
    #[error("PNG error: {0}")]
    Png(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn invalid(what: &'static str, detail: impl Into<String>) -> Self {
        Error::Invalid {
            what,
            detail: detail.into(),
        }
    }
}

/// Read a whole file, attaching the path to any error.
pub fn read_file(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    std::fs::read(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Write a whole file, attaching the path to any error.
pub fn write_file(path: impl AsRef<std::path::Path>, data: &[u8]) -> Result<()> {
    let path = path.as_ref();
    std::fs::write(path, data).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Create a directory (and parents), attaching the path to any error.
pub fn create_dir(path: impl AsRef<std::path::Path>) -> Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn le32(data: &[u8], off: usize, what: &'static str) -> Result<u32> {
    off.checked_add(4)
        .and_then(|e| data.get(off..e))
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(Error::Truncated {
            what,
            offset: off,
            need: 4,
            have: data.len().saturating_sub(off),
        })
}

pub(crate) fn le16(data: &[u8], off: usize, what: &'static str) -> Result<u16> {
    off.checked_add(2)
        .and_then(|e| data.get(off..e))
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or(Error::Truncated {
            what,
            offset: off,
            need: 2,
            have: data.len().saturating_sub(off),
        })
}

pub(crate) fn slice<'a>(
    data: &'a [u8],
    off: usize,
    len: usize,
    what: &'static str,
) -> Result<&'a [u8]> {
    off.checked_add(len)
        .and_then(|end| data.get(off..end))
        .ok_or(Error::Truncated {
            what,
            offset: off,
            need: len,
            have: data.len().saturating_sub(off),
        })
}
