//! Loaders for the on-disk asset format (RON, one file per table under `assets/data/`).
//!
//! Loaders take bytes (no `std::fs`), so they work on wasm where the asset server hands over
//! the file contents.

use crate::egg_moves::EggMoveTable;
use crate::encounters::EncounterData;
use crate::evolution::EvolutionTable;
use crate::growth::GrowthTables;
use crate::items::ItemTable;
use crate::learnset::LearnsetTable;
use crate::moves::MoveTable;
use crate::personal::PersonalTable;
use crate::tmhm::TmHmTable;
use crate::trainers::TrainerTable;
use crate::tutors::TutorTable;
use crate::types::TypeChart;
use std::fmt;

/// Why an asset failed to load.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    /// The bytes are not valid UTF-8.
    Utf8(String),
    /// The RON text does not match the table's schema.
    Ron(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Utf8(e) => write!(f, "asset is not UTF-8: {e}"),
            LoadError::Ron(e) => write!(f, "asset is not valid RON: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Parses any table from RON bytes.
pub fn load<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, LoadError> {
    let text = std::str::from_utf8(bytes).map_err(|e| LoadError::Utf8(e.to_string()))?;
    crate::from_ron(text).map_err(|e| LoadError::Ron(e.to_string()))
}

/// File names under `assets/data/`, one per table.
pub mod file {
    pub const PERSONAL: &str = "personal.ron";
    pub const LEARNSET: &str = "learnset.ron";
    pub const EVOLUTION: &str = "evolution.ron";
    pub const MOVES: &str = "moves.ron";
    pub const ITEMS: &str = "items.ron";
    pub const TRAINERS: &str = "trainers.ron";
    pub const ENCOUNTERS: &str = "encounters.ron";
    pub const GROWTH: &str = "growth.ron";
    pub const TYPES: &str = "types.ron";
    pub const TMHM: &str = "tmhm.ron";
    pub const EGG_MOVES: &str = "egg_moves.ron";
    pub const TUTORS: &str = "tutors.ron";
}

pub fn load_personal(bytes: &[u8]) -> Result<PersonalTable, LoadError> {
    load(bytes)
}
pub fn load_learnset(bytes: &[u8]) -> Result<LearnsetTable, LoadError> {
    load(bytes)
}
pub fn load_evolution(bytes: &[u8]) -> Result<EvolutionTable, LoadError> {
    load(bytes)
}
pub fn load_moves(bytes: &[u8]) -> Result<MoveTable, LoadError> {
    load(bytes)
}
pub fn load_items(bytes: &[u8]) -> Result<ItemTable, LoadError> {
    load(bytes)
}
pub fn load_trainers(bytes: &[u8]) -> Result<TrainerTable, LoadError> {
    load(bytes)
}
pub fn load_encounters(bytes: &[u8]) -> Result<EncounterData, LoadError> {
    load(bytes)
}
pub fn load_growth(bytes: &[u8]) -> Result<GrowthTables, LoadError> {
    load(bytes)
}
pub fn load_types(bytes: &[u8]) -> Result<TypeChart, LoadError> {
    load(bytes)
}

pub fn load_tmhm(bytes: &[u8]) -> Result<TmHmTable, LoadError> {
    load(bytes)
}
pub fn load_egg_moves(bytes: &[u8]) -> Result<EggMoveTable, LoadError> {
    load(bytes)
}
pub fn load_tutors(bytes: &[u8]) -> Result<TutorTable, LoadError> {
    load(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_input() {
        assert!(matches!(
            load_growth(&[0xFF, 0xFE]),
            Err(LoadError::Utf8(_))
        ));
        assert!(matches!(load_growth(b"nope"), Err(LoadError::Ron(_))));
    }

    #[test]
    fn small_round_trip() {
        let g = GrowthTables(vec![crate::growth::GrowthTable(vec![0, 1, 8])]);
        let text = crate::to_ron(&g).unwrap();
        assert_eq!(load_growth(text.as_bytes()).unwrap(), g);
    }
}
