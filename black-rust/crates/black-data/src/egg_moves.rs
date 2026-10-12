//! Egg moves (ARC 123 = `a/1/2/3`) (A7j).
//!
//! 650 members indexed by species id: `u16 count, u16 moves[count]`.

use black_core::ids::MoveId;
use serde::{Deserialize, Serialize};

/// Egg moves per species (index = species id; empty for species with none).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EggMoveTable(pub Vec<Vec<MoveId>>);

/// Parses one member; `None` unless it is exactly `count` moves long.
pub fn parse_member(b: &[u8]) -> Option<Vec<MoveId>> {
    let words: Vec<u16> = b
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    if b.is_empty() {
        return Some(vec![]); // species 0 dummy member
    }
    if !b.len().is_multiple_of(2) || usize::from(words[0]) != words.len() - 1 {
        return None;
    }
    Some(words[1..].to_vec())
}

pub fn member_bytes(moves: &[MoveId]) -> Vec<u8> {
    std::iter::once(moves.len() as u16)
        .chain(moves.iter().copied())
        .flat_map(u16::to_le_bytes)
        .collect()
}

impl EggMoveTable {
    pub fn get(&self, species: usize) -> &[MoveId] {
        self.0.get(species).map_or(&[], Vec::as_slice)
    }

    #[cfg(feature = "rom")]
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| parse_member(f).ok_or_else(|| format!("egg moves {i}: bad member")))
            .collect::<Result<Vec<_>, _>>()
            .map(EggMoveTable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_roundtrip_and_garbage() {
        let m = vec![10, 20, 30];
        assert_eq!(parse_member(&member_bytes(&m)), Some(m));
        assert_eq!(parse_member(&[0, 0]), Some(vec![]));
        assert_eq!(parse_member(&[2, 0, 1, 0]), None);
        assert_eq!(parse_member(&[]), Some(vec![]));
        assert_eq!(parse_member(&[1]), None);
    }

    #[cfg(feature = "rom")]
    #[test]
    fn real_rom_egg_moves() {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        let path = root.join("files/a/1/2/3");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} missing", path.display());
            return;
        };
        let t = EggMoveTable::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 650);
        assert_eq!(t.0.iter().filter(|m| m.is_empty()).count(), 390); // 389 two-byte members + the 0-byte member 0
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (m, f) in t.0.iter().zip(narc.files()).skip(1) {
            assert_eq!(member_bytes(m), f);
        }
        // Bulbasaur has 14 egg moves.
        assert_eq!(t.get(1).len(), 14);
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<EggMoveTable>(&ron).unwrap(), t);
    }
}
