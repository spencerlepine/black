//! `save_control.c` (S5): the save-control object of the original (main ARM9 0x02006D3C..
//! 0x02007454). Most of it is DS save plumbing that the port replaces: block lookup
//! (`FUN_020071CC(sav, id)` = `FUN_020319A4(ctrl->backup, id)`, here field access on
//! [`crate::save::SaveData`]), the flash / extra-area load and save sequence (`FUN_02007084`,
//! `FUN_02007150`, `FUN_02007168`, `FUN_020071A8`, `FUN_0200723C`, `FUN_02007248`,
//! `FUN_0200725C`, `FUN_020072A4`, `FUN_020072CC`, `FUN_0200734C`, `FUN_02007374`,
//! `FUN_020073B0`, `FUN_020073D8`, `FUN_02007400`, `FUN_02007428`) and the global instance
//! (`FUN_020070A4`), all replaced by [`crate::save::Storage`]. Its first two bytes are the
//! status flags read below; verified per function with `tools/check-fn.sh black-storage
//! save_control` (spec `tools/rom-diff/specs/black-storage/save_control.spec`).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0200_71E4, 0x0200_71EC, 0x0200_71F0];

/// The status bytes at +0 / +1 of the save control (`FUN_02007168` clears +0 and sets +1 when
/// the backup reports state 2 and the block-27 flag is 0). Roles UNVERIFIED: +0 "a save
/// exists", +1 "the save is complete".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveStatus {
    /// +0.
    pub exists: u8,
    /// +1.
    pub complete: u8,
}

impl SaveStatus {
    /// C: `FUN_020071E4`: byte +0.
    pub fn exists(&self) -> u32 {
        self.exists as u32
    }

    /// C: `FUN_020071EC`: byte +1.
    pub fn complete(&self) -> u32 {
        self.complete as u32
    }

    /// C: `FUN_020071F0`: both bytes are 1.
    pub fn usable(&self) -> bool {
        self.exists() == 1 && self.complete() == 1
    }
}

#[cfg(test)]
mod tests;
