//! Evolutions (ARC 019) and baby forms (ARC 020) (A7c).
//!
//! ARC 019: one 42-byte member per personal record, seven `(u16 method, u16 param, u16 target)`
//! slots; method 0 = empty. ARC 020: one `u16` per species, the base (baby) species of its
//! family. Method semantics belong to the evolution checks (P5); only the ones seen with
//! unambiguous data are named here.

use black_core::ids::SpeciesId;
use serde::{Deserialize, Serialize};

/// Evolution slots per species.
pub const SLOTS: usize = 7;

/// Method values confirmed from the data.
pub mod method {
    /// Unused slot.
    pub const NONE: u16 = 0;
    /// Level-up at `param` or higher (Snivy: 17 -> Servine).
    pub const LEVEL: u16 = 4;
    /// Use item `param` (Eevee: Thunder/Water/Fire Stone).
    pub const ITEM: u16 = 8;
}

/// One evolution slot.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Evolution {
    pub method: u16,
    pub param: u16,
    pub target: SpeciesId,
}

/// The evolution slots of one personal record (empty slots are kept, in order).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Evolutions(pub [Evolution; SLOTS]);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EvolutionTable {
    /// Indexed like the personal table.
    pub evolutions: Vec<Evolutions>,
    /// Indexed by species: the family's base species.
    pub baby: Vec<SpeciesId>,
}

impl Evolutions {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != SLOTS * 6 {
            return None;
        }
        let w = |i: usize| u16::from_le_bytes([b[2 * i], b[2 * i + 1]]);
        Some(Evolutions(std::array::from_fn(|k| Evolution {
            method: w(3 * k),
            param: w(3 * k + 1),
            target: w(3 * k + 2),
        })))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .flat_map(|e| [e.method, e.param, e.target])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    /// Non-empty slots.
    pub fn iter(&self) -> impl Iterator<Item = &Evolution> {
        self.0.iter().filter(|e| e.method != method::NONE)
    }
}

#[cfg(feature = "rom")]
impl EvolutionTable {
    /// Converts the ARC 019 and ARC 020 NARCs.
    pub fn from_narcs(evo: &[u8], baby: &[u8]) -> Result<Self, String> {
        let e = nds_rom::narc::Narc::parse(evo).map_err(|e| e.to_string())?;
        let b = nds_rom::narc::Narc::parse(baby).map_err(|e| e.to_string())?;
        let evolutions = e
            .files()
            .enumerate()
            .map(|(i, f)| {
                Evolutions::from_bytes(f).ok_or_else(|| format!("evolution {i}: {} bytes", f.len()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let baby = b
            .files()
            .enumerate()
            .map(|(i, f)| {
                <[u8; 2]>::try_from(f)
                    .map(u16::from_le_bytes)
                    .map_err(|_| format!("baby {i}: {} bytes", f.len()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(EvolutionTable { evolutions, baby })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_roundtrip() {
        let mut slots = [Evolution::default(); SLOTS];
        slots[0] = Evolution {
            method: method::LEVEL,
            param: 17,
            target: 496,
        };
        let e = Evolutions(slots);
        assert_eq!(Evolutions::from_bytes(&e.to_bytes()), Some(e.clone()));
        assert_eq!(e.iter().count(), 1);
        assert_eq!(Evolutions::from_bytes(&[0; 40]), None);
    }

    #[cfg(feature = "rom")]
    #[test]
    fn real_rom_evolutions() {
        const DIR: &str =
            "$HOME/workplace/black/black-rom/files/a/0";
        let (Ok(evo), Ok(baby)) = (
            std::fs::read(format!("{DIR}/1/9")),
            std::fs::read(format!("{DIR}/2/0")),
        ) else {
            eprintln!("skipping: {DIR} missing");
            return;
        };
        let t = EvolutionTable::from_narcs(&evo, &baby).unwrap();
        assert_eq!((t.evolutions.len(), t.baby.len()), (668, 650));
        let narc = nds_rom::narc::Narc::parse(&evo).unwrap();
        for (e, f) in t.evolutions.iter().zip(narc.files()) {
            assert_eq!(e.to_bytes(), f);
        }
        assert_eq!(
            t.evolutions[495].0[0],
            Evolution {
                method: method::LEVEL,
                param: 17,
                target: 496
            }
        );
        let eevee: Vec<_> = t.evolutions[133]
            .iter()
            .filter(|e| e.method == method::ITEM)
            .map(|e| e.target)
            .collect();
        assert_eq!(eevee, [135, 134, 136]);
        assert_eq!((t.baby[25], t.baby[26], t.baby[1]), (172, 172, 1));
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<EvolutionTable>(&ron).unwrap(), t);
    }
}
