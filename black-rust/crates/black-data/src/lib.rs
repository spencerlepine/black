//! Data types and loaders for the extracted assets (D4).
//!
//! Each table module owns its serde types and, behind the `rom` feature (native only), a
//! converter from the ROM's binary records. Assets are stored as RON under `assets/data/`.

pub mod egg_moves;
pub mod encounters;
pub mod evolution;
#[cfg(feature = "rom")]
pub mod extract;
pub mod growth;
pub mod items;
pub mod learnset;
pub mod loader;
pub mod moves;
pub mod personal;
pub mod tmhm;
pub mod trainers;
pub mod tutors;
pub mod types;

/// Serialises a table as pretty RON (stable output, so re-extraction is diffable).
pub fn to_ron<T: serde::Serialize>(value: &T) -> Result<String, ron::Error> {
    ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
}

/// Parses a table from RON.
pub fn from_ron<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, ron::error::SpannedError> {
    ron::from_str(text)
}
