//! Trainer data (ARC 092) and parties (ARC 093) (A7f).
//!
//! ARC 092 records follow the main-ARM9 getter `FUN_0202A230(trainer, param)` (record read
//! through `FUN_0202A3AC`, which loads ARC 0x5C = 92): params 0-3 = bytes +0..+3, 4-7 = items
//! at +4, 8 = u32 at +0xC, 9 = bit 0 of +0x10, 10 = +0x11, 11 = u16 at +0x12. Party entries are
//! 8 bytes plus an item (`format` bit 1) and four moves (`format` bit 0).

use black_core::ids::{ItemId, MoveId, SpeciesId};
use serde::{Deserialize, Serialize};

/// Trainer record (20 bytes; one 16-byte dummy exists).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Trainer {
    /// +0x00 (param 0): party format; bit 0 = entries carry moves, bit 1 = entries carry an item.
    pub format: u8,
    /// +0x01 (param 1): trainer class.
    pub class: u8,
    /// +0x02 (param 2): battle type (1 single, 2 double, 3 triple; inferred).
    pub battle_type: u8,
    /// +0x03 (param 3): party size.
    pub party_size: u8,
    /// +0x04 (params 4-7).
    pub items: [ItemId; 4],
    /// +0x0C (param 8): AI flags.
    pub ai: u32,
    /// +0x10: bit 0 = param 9 (UNVERIFIED meaning: heals party?); other bits kept.
    pub flags: u8,
    /// +0x11 (param 10; UNVERIFIED meaning: prize money multiplier?).
    pub p10: u8,
    /// +0x12 (param 11; UNVERIFIED meaning: prize item?).
    pub p11: u16,
    /// True for the short (16-byte) record, which has no +0x10..+0x13.
    pub short: bool,
}

/// One party member.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartyMon {
    /// +0 (UNVERIFIED meaning: IV/difficulty).
    pub difficulty: u8,
    /// +1 (UNVERIFIED meaning: gender/ability bits).
    pub flags: u8,
    pub level: u8,
    /// +3, always kept.
    pub unk_3: u8,
    pub species: SpeciesId,
    pub form: u16,
    pub item: Option<ItemId>,
    pub moves: Option<[MoveId; 4]>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TrainerEntry {
    pub trainer: Trainer,
    pub party: Vec<PartyMon>,
    /// Party bytes that are not a whole number of entries (only the dummy trainer 0), kept raw.
    pub party_raw: Option<Vec<u8>>,
}

/// Indexed by trainer ID.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TrainerTable(pub Vec<TrainerEntry>);

impl Trainer {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != 20 && b.len() != 16 {
            return None;
        }
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        let short = b.len() == 16;
        Some(Trainer {
            format: b[0],
            class: b[1],
            battle_type: b[2],
            party_size: b[3],
            items: [u16_at(4), u16_at(6), u16_at(8), u16_at(10)],
            ai: u32::from_le_bytes(b[0xC..0x10].try_into().unwrap()),
            flags: if short { 0 } else { b[0x10] },
            p10: if short { 0 } else { b[0x11] },
            p11: if short { 0 } else { u16_at(0x12) },
            short,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![self.format, self.class, self.battle_type, self.party_size];
        for i in self.items {
            b.extend_from_slice(&i.to_le_bytes());
        }
        b.extend_from_slice(&self.ai.to_le_bytes());
        if !self.short {
            b.extend_from_slice(&[self.flags, self.p10]);
            b.extend_from_slice(&self.p11.to_le_bytes());
        }
        b
    }

    /// Bytes per party entry for this trainer's format.
    pub fn entry_size(&self) -> usize {
        8 + if self.format & 2 != 0 { 2 } else { 0 } + if self.format & 1 != 0 { 8 } else { 0 }
    }
}

impl PartyMon {
    fn parse(b: &[u8], format: u8) -> Self {
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        let mut o = 8;
        let item = (format & 2 != 0).then(|| {
            o += 2;
            u16_at(o - 2)
        });
        let moves =
            (format & 1 != 0).then(|| [u16_at(o), u16_at(o + 2), u16_at(o + 4), u16_at(o + 6)]);
        PartyMon {
            difficulty: b[0],
            flags: b[1],
            level: b[2],
            unk_3: b[3],
            species: u16_at(4),
            form: u16_at(6),
            item,
            moves,
        }
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&[self.difficulty, self.flags, self.level, self.unk_3]);
        out.extend_from_slice(&self.species.to_le_bytes());
        out.extend_from_slice(&self.form.to_le_bytes());
        if let Some(i) = self.item {
            out.extend_from_slice(&i.to_le_bytes());
        }
        for m in self.moves.iter().flatten() {
            out.extend_from_slice(&m.to_le_bytes());
        }
    }
}

impl TrainerEntry {
    /// Parses a trainer record and its party member; the party holds `size / entry_size`
    /// entries (the file, not `party_size`, decides).
    pub fn from_bytes(data: &[u8], party: &[u8]) -> Option<Self> {
        let trainer = Trainer::from_bytes(data)?;
        let n = trainer.entry_size();
        if !party.len().is_multiple_of(n) {
            return Some(TrainerEntry {
                trainer,
                party: Vec::new(),
                party_raw: Some(party.to_vec()),
            });
        }
        let party = party
            .chunks(n)
            .map(|c| PartyMon::parse(c, trainer.format))
            .collect();
        Some(TrainerEntry {
            trainer,
            party,
            party_raw: None,
        })
    }

    pub fn party_bytes(&self) -> Vec<u8> {
        if let Some(raw) = &self.party_raw {
            return raw.clone();
        }
        let mut out = Vec::new();
        for m in &self.party {
            m.write(&mut out);
        }
        out
    }
}

#[cfg(feature = "rom")]
impl TrainerTable {
    /// Converts the ARC 092 and ARC 093 NARCs.
    pub fn from_narcs(data: &[u8], parties: &[u8]) -> Result<Self, String> {
        let d = nds_rom::narc::Narc::parse(data).map_err(|e| e.to_string())?;
        let p = nds_rom::narc::Narc::parse(parties).map_err(|e| e.to_string())?;
        if d.len() != p.len() {
            return Err(format!("{} trainers, {} parties", d.len(), p.len()));
        }
        d.files()
            .zip(p.files())
            .enumerate()
            .map(|(i, (a, b))| {
                TrainerEntry::from_bytes(a, b).ok_or_else(|| format!("trainer {i}: bad record"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(TrainerTable)
    }
}

#[cfg(all(test, feature = "rom"))]
mod tests {
    use super::*;

    #[test]
    fn real_rom_trainers() {
        const DIR: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../black-rom/files/a/0/9"
        );
        let (Ok(d), Ok(p)) = (
            std::fs::read(format!("{DIR}/2")),
            std::fs::read(format!("{DIR}/3")),
        ) else {
            eprintln!("skipping: {DIR} missing");
            return;
        };
        let t = TrainerTable::from_narcs(&d, &p).unwrap();
        assert_eq!(t.0.len(), 616);
        let dn = nds_rom::narc::Narc::parse(&d).unwrap();
        let pn = nds_rom::narc::Narc::parse(&p).unwrap();
        for ((e, a), b) in t.0.iter().zip(dn.files()).zip(pn.files()) {
            assert_eq!(e.trainer.to_bytes(), a);
            assert_eq!(e.party_bytes(), b);
        }
        // Trainer 1: one Patrat (504) at level 7.
        let m = &t.0[1].party[0];
        assert_eq!((t.0[1].trainer.party_size, m.species, m.level), (1, 504, 7));
        assert!(t.0[1..]
            .iter()
            .all(|e| e.party_raw.is_none() && e.party.len() == e.trainer.party_size as usize));
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<TrainerTable>(&ron).unwrap(), t);
    }
}
