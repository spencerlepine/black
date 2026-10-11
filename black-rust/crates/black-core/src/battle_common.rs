//! C: include/battle_common.h

/// C: `BTL_CLIENT_MAX`
pub const BTL_CLIENT_MAX: usize = 4;

/// C: `UNK_021D845C` (4 bytes, little-endian on disk).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct BattleEntry {
    pub unk_00: u32,
}

const _: () = assert!(core::mem::size_of::<BattleEntry>() == 4);

impl BattleEntry {
    pub const SIZE: usize = 4;

    pub fn from_bytes(b: &[u8; 4]) -> Self {
        Self {
            unk_00: u32::from_le_bytes(*b),
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.unk_00.to_le_bytes()
    }
}

/// Undecompiled overlay 93 routines that initialise a [`BattleEntry`].
pub trait EntryHooks {
    /// C: `FUN_overlay_d_93__021d845c` - used for the `TAG_TYPE_UNK_3` record.
    fn init_unk3(&self, entry: &mut BattleEntry);
    /// C: `FUN_overlay_d_93__021d8470` - used for the end-of-record entry.
    fn init_end(&self, entry: &mut BattleEntry);
}
