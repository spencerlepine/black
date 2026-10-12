//! Level-up learnsets (ARC 018) (A7b).
//!
//! One member per personal record: `(u16 move, u16 level)` pairs in level order, ended by
//! `0xFFFF 0xFFFF`.

use black_core::ids::MoveId;
use serde::{Deserialize, Serialize};

/// A move learned at a level.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelMove {
    pub level: u16,
    pub mv: MoveId,
}

/// Learnset of one personal record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Learnset(pub Vec<LevelMove>);

/// Learnsets indexed like the personal table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LearnsetTable(pub Vec<Learnset>);

const END: u16 = 0xFFFF;

impl Learnset {
    /// Parses one member; `None` if it is not a whole number of pairs ending in the terminator.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        let words: Vec<u16> = b
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        if !b.len().is_multiple_of(4) || words.len() < 2 {
            return None;
        }
        let (body, end) = words.split_at(words.len() - 2);
        if end != [END, END] {
            return None;
        }
        let moves: Vec<LevelMove> = body
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| LevelMove {
                mv: p[0],
                level: p[1],
            })
            .collect();
        if moves.iter().any(|m| m.mv == END) {
            return None;
        }
        Some(Learnset(moves))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .flat_map(|m| [m.mv, m.level])
            .chain([END, END])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    /// Moves learned exactly at `level`.
    pub fn at_level(&self, level: u16) -> impl Iterator<Item = MoveId> + '_ {
        self.0
            .iter()
            .filter(move |m| m.level == level)
            .map(|m| m.mv)
    }
}

#[cfg(feature = "rom")]
impl LearnsetTable {
    /// Converts the ARC 018 NARC.
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| {
                Learnset::from_bytes(f).ok_or_else(|| format!("learnset {i}: bad member"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(LearnsetTable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_roundtrip_and_rejects_garbage() {
        let l = Learnset(vec![
            LevelMove { mv: 33, level: 1 },
            LevelMove { mv: 45, level: 3 },
        ]);
        assert_eq!(Learnset::from_bytes(&l.to_bytes()), Some(l));
        assert_eq!(Learnset::from_bytes(&[0xFF; 4]), Some(Learnset::default()));
        assert_eq!(Learnset::from_bytes(&[1, 0, 1, 0]), None);
        assert_eq!(Learnset::from_bytes(&[0xFF; 6]), None);
    }

    #[cfg(feature = "rom")]
    #[test]
    fn real_rom_learnsets() {
        const ARC: &str =
            "$HOME/workplace/black/black-rom/files/a/0/1/8";
        let Ok(bytes) = std::fs::read(ARC) else {
            eprintln!("skipping: {ARC} missing");
            return;
        };
        let t = LearnsetTable::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 668);
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (l, f) in t.0.iter().zip(narc.files()) {
            assert_eq!(l.to_bytes(), f);
        }
        // Bulbasaur: Tackle 1, Growl 3, Leech Seed 7, Vine Whip 9.
        let b = &t.0[1].0;
        assert_eq!(
            b[..4].iter().map(|m| (m.mv, m.level)).collect::<Vec<_>>(),
            [(33, 1), (45, 3), (73, 7), (22, 9)]
        );
        assert!(t.0[1..650].iter().all(|l| !l.0.is_empty()));
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<LearnsetTable>(&ron).unwrap(), t);
    }
}
