//! Move data (ARC 021) (A7d).
//!
//! One 36-byte record per move. Offsets are those read by the main-ARM9 getter
//! `FUN_0201BDB4(record, param)` (behind `FUN_0201BD28(move, param)`, the battle code's
//! `FlowHost::move_param`); names come from that getter and from known moves (Growl, Swords
//! Dance, Calm Mind, Absorb, Take Down, Recover, Flamethrower).

use black_core::ids::MoveId;
use serde::{Deserialize, Serialize};

/// Bytes per record.
pub const SIZE: usize = 36;

/// One move record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Move {
    /// +0x00 (getter 0).
    pub ty: u8,
    /// +0x01 (getter 1): effect category (0 damage, 2 stat change, 3 heal, 4 damage + status,
    /// 8 drain…; inferred).
    pub quality: u8,
    /// +0x02 (getter 2): 0 status, 1 physical, 2 special.
    pub class: u8,
    /// +0x03 (getter 3).
    pub power: u8,
    /// +0x04 (getter 4): 101 = never misses (the getter reports it as 100).
    pub accuracy: u8,
    /// +0x05 (getter 5).
    pub pp: u8,
    /// +0x06 (getter 6, signed).
    pub priority: i8,
    /// +0x07: high nibble = max hits (getter 8), low nibble = min hits (getter 9).
    pub hits: u8,
    /// +0x08 (getter 0xB): inflicted condition (4 = burn for Flamethrower).
    pub condition: u16,
    /// +0x0A (getter 0xC): condition chance in percent.
    pub condition_chance: u8,
    /// +0x0B (getter 0xD).
    pub condition_duration: u8,
    /// +0x0C / +0x0D (getters 0xE / 0xF): min/max turns.
    pub min_turns: u8,
    pub max_turns: u8,
    /// +0x0E (getter 7; the getter maps 6 to 0).
    pub crit_stage: u8,
    /// +0x0F (getter 0xA).
    pub flinch: u8,
    /// +0x10 (getter 0x1C): effect sequence.
    pub effect: u16,
    /// +0x12: > 0 drain percent (getter 0x19), < 0 recoil percent (getter 0x1E).
    pub recoil_drain: i8,
    /// +0x13: > 0 heal percent (getter 0x1A).
    pub heal: i8,
    /// +0x14 (getter 0x1B): target (5 all foes, 7 user; inferred).
    pub target: u8,
    /// +0x15..+0x17: stat IDs (1 Atk … 5 Spe; inferred from Growl/Agility/Calm Mind).
    pub stats: [u8; 3],
    /// +0x18..+0x1A: stat stage changes.
    pub stat_amounts: [i8; 3],
    /// +0x1B..+0x1D: stat change chances.
    pub stat_chances: [u8; 3],
    /// +0x1E: always `0x5353` in the data; not read by the getter.
    pub unk_1e: u16,
    /// +0x20: flag bits.
    pub flags: u32,
}

/// Indexed by move ID.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MoveTable(pub Vec<Move>);

impl Move {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        let b: &[u8; SIZE] = b.try_into().ok()?;
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        Some(Move {
            ty: b[0],
            quality: b[1],
            class: b[2],
            power: b[3],
            accuracy: b[4],
            pp: b[5],
            priority: b[6] as i8,
            hits: b[7],
            condition: u16_at(8),
            condition_chance: b[0xA],
            condition_duration: b[0xB],
            min_turns: b[0xC],
            max_turns: b[0xD],
            crit_stage: b[0xE],
            flinch: b[0xF],
            effect: u16_at(0x10),
            recoil_drain: b[0x12] as i8,
            heal: b[0x13] as i8,
            target: b[0x14],
            stats: [b[0x15], b[0x16], b[0x17]],
            stat_amounts: [b[0x18] as i8, b[0x19] as i8, b[0x1A] as i8],
            stat_chances: [b[0x1B], b[0x1C], b[0x1D]],
            unk_1e: u16_at(0x1E),
            flags: u32::from_le_bytes([b[0x20], b[0x21], b[0x22], b[0x23]]),
        })
    }

    pub fn to_bytes(&self) -> [u8; SIZE] {
        let mut b = [0u8; SIZE];
        b[..8].copy_from_slice(&[
            self.ty,
            self.quality,
            self.class,
            self.power,
            self.accuracy,
            self.pp,
            self.priority as u8,
            self.hits,
        ]);
        b[8..10].copy_from_slice(&self.condition.to_le_bytes());
        b[0xA..0x10].copy_from_slice(&[
            self.condition_chance,
            self.condition_duration,
            self.min_turns,
            self.max_turns,
            self.crit_stage,
            self.flinch,
        ]);
        b[0x10..0x12].copy_from_slice(&self.effect.to_le_bytes());
        b[0x12] = self.recoil_drain as u8;
        b[0x13] = self.heal as u8;
        b[0x14] = self.target;
        b[0x15..0x18].copy_from_slice(&self.stats);
        for (d, s) in b[0x18..0x1B].iter_mut().zip(self.stat_amounts) {
            *d = s as u8;
        }
        b[0x1B..0x1E].copy_from_slice(&self.stat_chances);
        b[0x1E..0x20].copy_from_slice(&self.unk_1e.to_le_bytes());
        b[0x20..].copy_from_slice(&self.flags.to_le_bytes());
        b
    }
}

impl MoveTable {
    pub fn get(&self, mv: MoveId) -> Option<&Move> {
        self.0.get(mv as usize)
    }
}

#[cfg(feature = "rom")]
impl MoveTable {
    /// Converts the ARC 021 NARC.
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| Move::from_bytes(f).ok_or_else(|| format!("move {i}: {} bytes", f.len())))
            .collect::<Result<Vec<_>, _>>()
            .map(MoveTable)
    }
}

#[cfg(all(test, feature = "rom"))]
mod tests {
    use super::*;

    #[test]
    fn real_rom_moves() {
        const ARC: &str =
            "$HOME/workplace/black/black-rom/files/a/0/2/1";
        let Ok(bytes) = std::fs::read(ARC) else {
            eprintln!("skipping: {ARC} missing");
            return;
        };
        let t = MoveTable::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 560);
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (m, f) in t.0.iter().zip(narc.files()) {
            assert_eq!(&m.to_bytes()[..], f);
        }
        let tackle = t.get(33).unwrap();
        assert_eq!(
            (
                tackle.ty,
                tackle.class,
                tackle.power,
                tackle.accuracy,
                tackle.pp
            ),
            (0, 1, 50, 100, 35)
        );
        let growl = t.get(45).unwrap();
        assert_eq!((growl.stats[0], growl.stat_amounts[0]), (1, -1));
        let calm_mind = t.get(347).unwrap();
        assert_eq!(
            (calm_mind.stats, calm_mind.stat_amounts),
            ([3, 4, 0], [1, 1, 0])
        );
        assert_eq!(t.get(71).unwrap().recoil_drain, 50);
        assert_eq!(t.get(36).unwrap().recoil_drain, -25);
        assert_eq!(t.get(105).unwrap().heal, 50);
        let flamethrower = t.get(53).unwrap();
        assert_eq!(
            (
                flamethrower.ty,
                flamethrower.power,
                flamethrower.condition,
                flamethrower.condition_chance
            ),
            (9, 95, 4, 10)
        );
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<MoveTable>(&ron).unwrap(), t);
    }
}
