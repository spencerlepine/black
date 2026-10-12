//! Type chart (A7i).
//!
//! The 17 x 17 chart `[attack][defend]` lives in overlay 93 read-only data at 0x021F047C (read by
//! `FUN_overlay_d_93__021d794c`); entries are 0 immune, 2 half, 4 neutral, 8 double. Type IDs
//! follow `black_core::ids::types`.

use black_core::ids::types;
use serde::{Deserialize, Serialize};

/// RAM address of the chart in overlay 93.
pub const CHART_ADDR: u32 = 0x021F_047C;
/// Chart rows/columns.
pub const N: usize = types::COUNT as usize;

/// Effectiveness of one attack type against one defend type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Immune,
    Half,
    Neutral,
    Double,
}

impl Effect {
    /// The multiplier in quarters (0, 2, 4, 8), as stored in the ROM.
    pub fn quarters(self) -> u8 {
        match self {
            Effect::Immune => 0,
            Effect::Half => 2,
            Effect::Neutral => 4,
            Effect::Double => 8,
        }
    }

    pub fn from_quarters(q: u8) -> Option<Self> {
        Some(match q {
            0 => Effect::Immune,
            2 => Effect::Half,
            4 => Effect::Neutral,
            8 => Effect::Double,
            _ => return None,
        })
    }
}

/// `chart[attack][defend]`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TypeChart(pub Vec<Vec<Effect>>);

impl TypeChart {
    /// Parses the 289 chart bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != N * N {
            return None;
        }
        b.chunks(N)
            .map(|row| {
                row.iter()
                    .map(|&q| Effect::from_quarters(q))
                    .collect::<Option<Vec<_>>>()
            })
            .collect::<Option<Vec<_>>>()
            .map(TypeChart)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flatten().map(|e| e.quarters()).collect()
    }

    /// Effectiveness of `attack` against `defend`; types outside the chart (e.g. `types::NONE`)
    /// are neutral, as in the original.
    pub fn get(&self, attack: u8, defend: u8) -> Effect {
        self.0
            .get(attack as usize)
            .and_then(|r| r.get(defend as usize))
            .copied()
            .unwrap_or(Effect::Neutral)
    }

    /// Reads the chart out of the decompressed overlay-93 image loaded at `base`.
    pub fn from_overlay93(image: &[u8], base: u32) -> Option<Self> {
        let o = CHART_ADDR.checked_sub(base)? as usize;
        Self::from_bytes(image.get(o..o + N * N)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_lookup() {
        let mut b = vec![4u8; N * N];
        b[types::FIRE as usize * N + types::WATER as usize] = 2;
        b[types::FIRE as usize * N + types::GRASS as usize] = 8;
        b[types::NORMAL as usize * N + types::GHOST as usize] = 0;
        let c = TypeChart::from_bytes(&b).unwrap();
        assert_eq!(c.to_bytes(), b);
        assert_eq!(c.get(types::FIRE, types::GRASS), Effect::Double);
        assert_eq!(c.get(types::FIRE, types::NONE), Effect::Neutral);
        assert!(TypeChart::from_bytes(&[4; 10]).is_none());
        assert!(TypeChart::from_bytes(&[3; N * N]).is_none());
    }

    #[cfg(feature = "rom")]
    #[test]
    fn real_rom_chart() {
        const OVY: &str =
            "$HOME/workplace/black/black-rom/build/black.us/OVY_93.sbin";
        let Ok(image) = std::fs::read(OVY) else {
            eprintln!("skipping: {OVY} missing");
            return;
        };
        let c = TypeChart::from_overlay93(&image, 0x021B_6100).unwrap();
        assert_eq!(c.get(types::NORMAL, types::GHOST), Effect::Immune);
        assert_eq!(c.get(types::FIGHTING, types::NORMAL), Effect::Double);
        assert_eq!(c.get(types::WATER, types::FIRE), Effect::Double);
        assert_eq!(c.get(types::ELECTRIC, types::GROUND), Effect::Immune);
        assert_eq!(c.get(types::GHOST, types::NORMAL), Effect::Immune);
        assert_eq!(c.get(types::GRASS, types::STEEL), Effect::Half);
        let ron = crate::to_ron(&c).unwrap();
        assert_eq!(crate::from_ron::<TypeChart>(&ron).unwrap(), c);
    }
}
