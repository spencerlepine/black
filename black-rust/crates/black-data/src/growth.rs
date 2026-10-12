//! EXP growth tables (ARC 017) (A7h).
//!
//! Eight tables, one per growth rate, each giving the total EXP needed for levels 0-100
//! (101 little-endian `u32`).

use serde::{Deserialize, Serialize};

/// Levels 0..=100.
pub const LEVELS: usize = 101;

/// Total EXP needed to reach each level (index = level).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GrowthTable(pub Vec<u32>);

/// All growth rates, indexed by the personal record's growth-rate byte.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GrowthTables(pub Vec<GrowthTable>);

impl GrowthTable {
    /// EXP needed to reach `level` (clamped to 100).
    pub fn exp_for_level(&self, level: u8) -> u32 {
        self.0[(level as usize).min(LEVELS - 1)]
    }

    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != LEVELS * 4 {
            return None;
        }
        Some(GrowthTable(
            b.as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c))
                .collect(),
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|v| v.to_le_bytes()).collect()
    }
}

#[cfg(feature = "rom")]
impl GrowthTables {
    /// Converts the ARC 017 NARC.
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| {
                GrowthTable::from_bytes(f)
                    .ok_or_else(|| format!("growth table {i}: {} bytes", f.len()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(GrowthTables)
    }
}

#[cfg(all(test, feature = "rom"))]
mod tests {
    use super::*;

    const ARC: &str =
        "$HOME/workplace/black/black-rom/files/a/0/1/7";

    #[test]
    fn real_rom_growth_tables() {
        let Ok(bytes) = std::fs::read(ARC) else {
            eprintln!("skipping: {ARC} missing");
            return;
        };
        let t = GrowthTables::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 8);
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (g, f) in t.0.iter().zip(narc.files()) {
            assert_eq!(g.to_bytes(), f);
            assert_eq!(g.exp_for_level(1), 0);
            assert!(g.0.windows(2).skip(1).all(|w| w[0] < w[1]));
        }
        // Medium Fast (n^3), Erratic (600,000) and Fluctuating (1,640,000) at level 100.
        let at100: Vec<u32> = t.0.iter().map(|g| g.exp_for_level(100)).collect();
        assert_eq!(at100[0], 1_000_000);
        assert!(at100.contains(&600_000));
        assert!(at100.contains(&1_640_000));
        assert_eq!(t.0[0].exp_for_level(50), 125_000);
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<GrowthTables>(&ron).unwrap(), t);
    }
}
