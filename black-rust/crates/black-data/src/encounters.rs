//! Wild encounters (ARC 126) (A7g).
//!
//! A member is one 232-byte table, or four (one per season, 928 bytes). Table layout: 8 rate
//! bytes, then 56 slots of `(u16 species | form << 11, u8 min level, u8 max level)`: three
//! 12-slot land tables, then four 5-slot water/fishing tables.
//!
//! Rate <-> group mapping (verified from the encounter code, F5a-spec): `rates[t]` is the rate of
//! slot group `t` in file order, `t` being the encounter type of `field_encount.c` (`0x021AA3E0` reads
//! `table[t]`, `0x021A9A5C` copies group `t`): 0 grass, 1 dark grass (double battles), 2 shaking
//! grass, 3 surf, 4 rippling water, 5 fishing, 6 fishing on a ripple. `rates[7]` is not a rate: it
//! is the zone's effect-encounter kind 0..=2 (`0x021AA7DC`: step interval / probability of shaking
//! grass etc., 20 steps 10 % for kinds 0 / 1, 5 steps 15 % for kind 2); the game sets its bit 7 in
//! the RAM copy as the "table loaded" flag. In all 148 tables of the ROM a rate is nonzero exactly
//! when its group has species, and `rates[7]` is 0 (115 tables), 1 (31) or 2 (2). A form of 0x1F
//! means a random form. The game loads table `season` (0..=3, `0x0215E2B8`, called by
//! `eventdata_system.c` `0x02162438` with the season it is given) of a four-table member, and the
//! only table of a one-table member in every season (which season number is which: F11).

use black_core::ids::SpeciesId;
use serde::{Deserialize, Serialize};

/// Bytes per table.
pub const TABLE_SIZE: usize = 232;
/// Slot counts per group, in file order.
pub const GROUP_SLOTS: [usize; 7] = [12, 12, 12, 5, 5, 5, 5];

/// One encounter slot.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Slot {
    pub species: SpeciesId,
    pub form: u8,
    pub min_level: u8,
    pub max_level: u8,
}

/// One table: rates + slot groups.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EncounterTable {
    /// `rates[0..7]`: rate of group `t` (encounter type `t`, see the module doc); `rates[7]`: the
    /// effect-encounter kind (0..=2), not a rate.
    pub rates: [u8; 8],
    /// `GROUP_SLOTS` groups in file order (encounter types 0..=6).
    pub groups: Vec<Vec<Slot>>,
}

/// One zone's encounters: one table, or one per season.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ZoneEncounters(pub Vec<EncounterTable>);

/// Indexed by encounter ID (zone headers refer to it).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EncounterData(pub Vec<ZoneEncounters>);

impl EncounterTable {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != TABLE_SIZE {
            return None;
        }
        let mut ofs = 8;
        let groups = GROUP_SLOTS
            .iter()
            .map(|&n| {
                (0..n)
                    .map(|_| {
                        let w = u16::from_le_bytes([b[ofs], b[ofs + 1]]);
                        let s = Slot {
                            species: w & 0x7FF,
                            form: (w >> 11) as u8,
                            min_level: b[ofs + 2],
                            max_level: b[ofs + 3],
                        };
                        ofs += 4;
                        s
                    })
                    .collect()
            })
            .collect();
        Some(EncounterTable {
            rates: b[..8].try_into().unwrap(),
            groups,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.rates.to_vec();
        for s in self.groups.iter().flatten() {
            out.extend_from_slice(&(s.species | (s.form as u16) << 11).to_le_bytes());
            out.extend_from_slice(&[s.min_level, s.max_level]);
        }
        out
    }
}

impl ZoneEncounters {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.is_empty() || !b.len().is_multiple_of(TABLE_SIZE) {
            return None;
        }
        b.chunks(TABLE_SIZE)
            .map(EncounterTable::from_bytes)
            .collect::<Option<Vec<_>>>()
            .map(ZoneEncounters)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|t| t.to_bytes()).collect()
    }
}

#[cfg(feature = "rom")]
impl EncounterData {
    /// Converts the ARC 126 NARC.
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| {
                ZoneEncounters::from_bytes(f)
                    .ok_or_else(|| format!("encounters {i}: {} bytes", f.len()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(EncounterData)
    }
}

#[cfg(all(test, feature = "rom"))]
mod tests {
    use super::*;

    #[test]
    fn real_rom_encounters() {
        const ARC: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../black-rom/files/a/1/2/6"
        );
        let Ok(bytes) = std::fs::read(ARC) else {
            nds_rom::testenv::rom_missing(&format!("{ARC} missing"));
            return;
        };
        let t = EncounterData::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 112);
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (z, f) in t.0.iter().zip(narc.files()) {
            assert_eq!(z.to_bytes(), f);
            assert!(z.0.len() == 1 || z.0.len() == 4);
        }
        assert!(t.0.iter().any(|z| z.0.len() == 4));
        // Every non-empty slot is a valid species with min <= max.
        for s in
            t.0.iter()
                .flat_map(|z| &z.0)
                .flat_map(|tb| tb.groups.iter().flatten())
        {
            if s.species != 0 {
                assert!(s.species <= 649 && s.min_level <= s.max_level && s.min_level >= 1);
            }
        }
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<EncounterData>(&ron).unwrap(), t);
    }
}
