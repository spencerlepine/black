//! TM/HM move table (A7j): 101 x u16 move ids in main ARM9 at 0x0209EAA0 (T3 findings).
//!
//! Storage order: `[0..92]` TM01..TM92, `[92..98]` HM01..HM06, `[98..101]` TM93..TM95. The same
//! indices are the bit numbers of the per-species learnability field (`Personal::tm_hm`).

use crate::personal::Personal;
use black_core::ids::MoveId;
use serde::{Deserialize, Serialize};

/// Number of TM/HM slots.
pub const COUNT: usize = 101;
/// File offset of the table inside `main.sbin` (0x0209EAA0 - 0x02004000).
pub const MAIN_OFFSET: usize = 0x9AAA0;

/// Move id per TM/HM slot, in storage order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TmHmTable(pub Vec<MoveId>);

impl TmHmTable {
    /// Reads the table out of `main.sbin`; `None` if the image is too short.
    pub fn from_main(main_sbin: &[u8]) -> Option<Self> {
        let b = main_sbin.get(MAIN_OFFSET..MAIN_OFFSET + COUNT * 2)?;
        Some(TmHmTable(
            b.as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .collect(),
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|m| m.to_le_bytes()).collect()
    }

    /// Storage index for an item id (TM/HM items 328..=425 and 618..=620).
    pub fn index_for_item(item: u16) -> Option<usize> {
        match item {
            328..=425 => Some(usize::from(item - 328)),
            618..=620 => Some(usize::from(item - 520)),
            _ => None,
        }
    }

    /// Move taught by a TM/HM item.
    pub fn move_for_item(&self, item: u16) -> Option<MoveId> {
        Self::index_for_item(item).and_then(|i| self.0.get(i).copied())
    }

    /// Moves the species can learn from TMs/HMs.
    pub fn learnable_moves(&self, p: &Personal) -> Vec<MoveId> {
        self.0
            .iter()
            .enumerate()
            .filter(|(i, _)| p.can_learn_tm_hm(*i))
            .map(|(_, &m)| m)
            .collect()
    }

    /// Whether the species can learn `mv` from a TM/HM.
    pub fn can_learn(&self, p: &Personal, mv: MoveId) -> bool {
        self.0
            .iter()
            .enumerate()
            .any(|(i, &m)| m == mv && p.can_learn_tm_hm(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_mapping() {
        assert_eq!(TmHmTable::index_for_item(328), Some(0));
        assert_eq!(TmHmTable::index_for_item(420), Some(92));
        assert_eq!(TmHmTable::index_for_item(618), Some(98));
        assert_eq!(TmHmTable::index_for_item(620), Some(100));
        assert_eq!(TmHmTable::index_for_item(327), None);
        assert_eq!(TmHmTable::index_for_item(621), None);
        assert!(TmHmTable::from_main(&[0; 16]).is_none());
    }

    #[test]
    fn real_main_table() {
        let Some(dir) = std::env::var_os("BLACK_ROM_BUILD") else {
            eprintln!("skipping: BLACK_ROM_BUILD unset");
            return;
        };
        let path = std::path::Path::new(&dir).join("main.sbin");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} missing", path.display());
            return;
        };
        let t = TmHmTable::from_main(&bytes).unwrap();
        assert_eq!(t.0.len(), COUNT);
        assert_eq!(t.to_bytes(), bytes[MAIN_OFFSET..MAIN_OFFSET + COUNT * 2]);
        assert_eq!(t.0[0], 468); // TM01
        assert_eq!(t.move_for_item(328), Some(468));
        assert_eq!(t.0[92], 15); // HM01 = Cut
        assert_eq!(t.move_for_item(420), Some(15));
        assert_eq!(&t.0[98..], [528, 249, 555]);
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<TmHmTable>(&ron).unwrap(), t);
    }
}
