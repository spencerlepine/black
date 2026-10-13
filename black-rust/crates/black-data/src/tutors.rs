//! Move tutors (A7j): per-species tutor bits (personal +0x38, bits 0..=6) and the fixed
//! tutor move map, which is code in overlay 21 (`FUN_021B5F24`), not data (T3 findings).

use crate::personal::PersonalTable;
use black_core::ids::MoveId;
use serde::{Deserialize, Serialize};

/// Mask of the used tutor bits.
pub const BITS_MASK: u32 = 0x7F;
/// Fallback move when no bit matches in tutors 1 and 2.
pub const FALLBACK_MOVE: MoveId = 307;

/// Tutor bits (0..=6) per personal record (0 for the record without a tutor word).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TutorTable(pub Vec<u8>);

impl TutorTable {
    /// Builds from the personal table; `None` if any record has bits outside 0..=6.
    pub fn from_personal(p: &PersonalTable) -> Option<Self> {
        p.0.iter()
            .map(|r| {
                let v = r.tutors.unwrap_or(0);
                (v & !BITS_MASK == 0).then_some(v as u8)
            })
            .collect::<Option<Vec<_>>>()
            .map(TutorTable)
    }

    pub fn bits(&self, record: usize) -> u8 {
        self.0.get(record).copied().unwrap_or(0)
    }
}

/// Move offered by `tutor` (0..=2) to a record with tutor `bits`. Bits 0-2 pick the tutor 2
/// move, bits 3-5 the tutor 1 move. Bit 6 is UNVERIFIED (set on dragon-type families; the code
/// T3 read never tests it), so tutor 0 ignores it.
pub fn tutor_move(tutor: u8, bits: u8) -> Option<MoveId> {
    let b = |n: u8| bits >> n & 1 != 0;
    match tutor {
        0 => Some(434),
        1 => Some(if b(3) {
            338
        } else if b(4) {
            307
        } else if b(5) {
            308
        } else {
            FALLBACK_MOVE
        }),
        2 => Some(if b(0) {
            520
        } else if b(1) {
            519
        } else if b(2) {
            518
        } else {
            FALLBACK_MOVE
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_map() {
        assert_eq!(tutor_move(0, 0), Some(434));
        assert_eq!(tutor_move(1, 1 << 3), Some(338));
        assert_eq!(tutor_move(1, 1 << 4), Some(307));
        assert_eq!(tutor_move(1, 1 << 5), Some(308));
        assert_eq!(tutor_move(1, 0), Some(307));
        assert_eq!(tutor_move(2, 1), Some(520));
        assert_eq!(tutor_move(2, 2), Some(519));
        assert_eq!(tutor_move(2, 4), Some(518));
        assert_eq!(tutor_move(3, 0), None);
    }

    #[cfg(feature = "rom")]
    #[test]
    fn real_rom_tutors() {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let Ok(bytes) = std::fs::read(root.join("files/a/0/1/6")) else {
            nds_rom::testenv::rom_missing("personal missing");
            return;
        };
        let p = PersonalTable::from_narc(&bytes).unwrap();
        let t = TutorTable::from_personal(&p).unwrap();
        assert_eq!(t.0.len(), p.0.len());
        let count = |n: u8| t.0.iter().filter(|&&b| b >> n & 1 != 0).count();
        assert_eq!([count(0), count(1), count(2)], [15, 15, 15]);
        assert_eq!([count(3), count(4), count(5)], [5, 5, 5]);
        assert!(count(6) >= 30); // 30 species (T3) plus alternate forms
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<TutorTable>(&ron).unwrap(), t);
    }
}
