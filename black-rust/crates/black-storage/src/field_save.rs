//! Save blocks whose accessors are ported in `black-field` (S5b): models only, so they can live in
//! [`crate::save::SaveData`] (black-storage does not depend on black-field; the field code works on
//! the byte images [`EventWorkSave::to_bytes`] / [`MmdlPositions::to_bytes`] give).
//!
//! * Block 45 (0x3EA bytes, main ARM9 0x020142D0..0x02014428): the event work, accessors in
//!   `black_field::event_work` (flags, vars, the "temp flags cleared" marker). Layout: +0x000 0x13E
//!   `u16` vars, +0x27C 0x16C bytes of flags (bit `id & 7` of byte `id >> 3`), +0x3E8 `u8` marker,
//!   +0x3E9 pad. Init (`sub_020142D8`) = all zero.
//! * Block 51 (0x168 bytes, overlay 10 0x0216E474..0x0216E618): the 30 kept MMDL positions
//!   (VecFx32 each; all 0xFF bytes = no position), accessors in `black_field::fldmmdl`
//!   (`save_work_*`). Init (`FUN_0216E47C`) = all 0xFFFFFFFF.

use serde::{Deserialize, Serialize};

use crate::le::{r16s, r32s, w16s, w32s};

/// Save block number of the event work.
pub const EVENT_WORK_BLOCK_ID: u32 = 45;
/// Bytes of the event work.
pub const EVENT_WORK_SIZE: usize = 0x3EA;
/// Event vars.
pub const VARS: usize = 0x13E;
/// Bytes of event flags.
pub const FLAG_BYTES: usize = 0x16C;
/// Save block number of the MMDL positions.
pub const MMDL_POS_BLOCK_ID: u32 = 51;
/// Bytes of the MMDL positions.
pub const MMDL_POS_SIZE: usize = 0x168;
/// Kept positions.
pub const POSITIONS: usize = 30;

/// Save block 45.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventWorkSave {
    /// +0x000: the vars.
    pub vars: Vec<u16>,
    /// +0x27C: the flags.
    #[serde(with = "crate::save::hex")]
    pub flags: Vec<u8>,
    /// +0x3E8: the "temp flags cleared" marker.
    pub marker: u8,
    /// +0x3E9: pad.
    pub pad: u8,
}

impl Default for EventWorkSave {
    fn default() -> Self {
        EventWorkSave::from_bytes(&[0; EVENT_WORK_SIZE])
    }
}

impl EventWorkSave {
    /// Reads the original 0x3EA-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        EventWorkSave {
            vars: r16s::<VARS>(b, 0).to_vec(),
            flags: b[0x27C..0x27C + FLAG_BYTES].to_vec(),
            marker: b[0x3E8],
            pad: b[0x3E9],
        }
    }

    /// The original 0x3EA-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; EVENT_WORK_SIZE];
        w16s(&mut b, 0, &self.vars);
        b[0x27C..0x27C + FLAG_BYTES].copy_from_slice(&self.flags);
        b[0x3E8] = self.marker;
        b[0x3E9] = self.pad;
        b
    }

    /// Flag `id` (ids below 0x4000 only; the higher ones are not saved).
    pub fn flag(&self, id: u32) -> bool {
        self.flags[(id >> 3) as usize] & (1 << (id & 7)) != 0
    }

    /// Var `id` (0x4000-based ids, as the scripts use them).
    pub fn var(&self, id: u32) -> u16 {
        self.vars[(id - 0x4000) as usize]
    }
}

/// Save block 51.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MmdlPositions {
    /// The kept positions (x, y, z fx32).
    pub positions: Vec<[u32; 3]>,
}

impl MmdlPositions {
    /// Reads the original 0x168-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        MmdlPositions {
            positions: (0..POSITIONS).map(|i| r32s(b, 12 * i)).collect(),
        }
    }

    /// The original 0x168-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; MMDL_POS_SIZE];
        for (i, p) in self.positions.iter().enumerate() {
            w32s(&mut b, 12 * i, p);
        }
        b
    }

    /// The block after its init (`FUN_0216E47C`): every word 0xFFFFFFFF.
    pub fn new_game() -> Self {
        MmdlPositions {
            positions: vec![[u32::MAX; 3]; POSITIONS],
        }
    }
}
