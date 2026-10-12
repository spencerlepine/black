//! Item data (ARC 024) (A7e).
//!
//! One 36-byte record per item. Offsets follow the main-ARM9 getter `FUN_02021014(record,
//! param)` (behind `FUN_02020fe4`, which battle code reaches through `021d805c` /
//! `FlowHost::item_param`); the getter param number is given per field. Names come from known
//! items (Potion: price 300, fling 30, heals 20 HP; Cheri Berry: Natural Gift Fire 60).

use black_core::ids::ItemId;
use serde::{Deserialize, Serialize};

/// Bytes per record.
pub const SIZE: usize = 36;

/// One item record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// +0x00 (param 0 returns it × 10): price / 10.
    pub price_div10: u16,
    /// +0x02 (param 1): battle hold effect.
    pub hold_effect: u8,
    /// +0x03 (param 2): hold effect parameter.
    pub hold_param: u8,
    /// +0x04 (param 8; UNVERIFIED meaning: Pluck/Bug Bite effect?).
    pub p08: u8,
    /// +0x05 (param 9; UNVERIFIED meaning: Fling effect?).
    pub p09: u8,
    /// +0x06 (param 10): Fling power (Potion 30).
    pub fling_power: u8,
    /// +0x07 (param 11): Natural Gift power (Cheri Berry 60).
    pub natural_gift_power: u8,
    /// +0x08: bit field read by params 0xC (bits 0-4: Natural Gift type), 3 (bit 5), 4 (bit 6),
    /// 5 (bits 7-10), 0xD (bits 11-15).
    pub packed: u16,
    /// +0x0A (param 6).
    pub p06: u8,
    /// +0x0B (param 7).
    pub p07: u8,
    /// +0x0C (param 0xE): usage data kind (0: +0x10 is a single byte, 1: +0x10 is a
    /// party-use parameter block read by `FUN_020210C8`).
    pub usage_kind: u8,
    /// +0x0D..+0x0F (params 0xF, 0x10, 0x11).
    pub p0f: u8,
    pub p10: u8,
    pub p11: u8,
    /// +0x10..+0x23: usage data (see `usage_kind`); kept raw until S4 ports `020210C8`.
    pub usage: [u8; 20],
}

impl Item {
    /// Price in money.
    pub fn price(&self) -> u32 {
        self.price_div10 as u32 * 10
    }

    /// Natural Gift type (param 0xC).
    pub fn natural_gift_type(&self) -> u8 {
        (self.packed & 0x1F) as u8
    }

    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        let b: &[u8; SIZE] = b.try_into().ok()?;
        Some(Item {
            price_div10: u16::from_le_bytes([b[0], b[1]]),
            hold_effect: b[2],
            hold_param: b[3],
            p08: b[4],
            p09: b[5],
            fling_power: b[6],
            natural_gift_power: b[7],
            packed: u16::from_le_bytes([b[8], b[9]]),
            p06: b[0xA],
            p07: b[0xB],
            usage_kind: b[0xC],
            p0f: b[0xD],
            p10: b[0xE],
            p11: b[0xF],
            usage: b[0x10..].try_into().unwrap(),
        })
    }

    pub fn to_bytes(&self) -> [u8; SIZE] {
        let mut b = [0u8; SIZE];
        b[..2].copy_from_slice(&self.price_div10.to_le_bytes());
        b[2..8].copy_from_slice(&[
            self.hold_effect,
            self.hold_param,
            self.p08,
            self.p09,
            self.fling_power,
            self.natural_gift_power,
        ]);
        b[8..10].copy_from_slice(&self.packed.to_le_bytes());
        b[0xA..0x10].copy_from_slice(&[
            self.p06,
            self.p07,
            self.usage_kind,
            self.p0f,
            self.p10,
            self.p11,
        ]);
        b[0x10..].copy_from_slice(&self.usage);
        b
    }
}

/// Indexed by item ID.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemTable(pub Vec<Item>);

impl ItemTable {
    pub fn get(&self, item: ItemId) -> Option<&Item> {
        self.0.get(item as usize)
    }
}

#[cfg(feature = "rom")]
impl ItemTable {
    /// Converts the ARC 024 NARC.
    pub fn from_narc(bytes: &[u8]) -> Result<Self, String> {
        let narc = nds_rom::narc::Narc::parse(bytes).map_err(|e| e.to_string())?;
        narc.files()
            .enumerate()
            .map(|(i, f)| Item::from_bytes(f).ok_or_else(|| format!("item {i}: {} bytes", f.len())))
            .collect::<Result<Vec<_>, _>>()
            .map(ItemTable)
    }
}

#[cfg(all(test, feature = "rom"))]
mod tests {
    use super::*;

    #[test]
    fn real_rom_items() {
        const ARC: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../black-rom/files/a/0/2/4"
        );
        let Ok(bytes) = std::fs::read(ARC) else {
            eprintln!("skipping: {ARC} missing");
            return;
        };
        let t = ItemTable::from_narc(&bytes).unwrap();
        assert_eq!(t.0.len(), 627);
        let narc = nds_rom::narc::Narc::parse(&bytes).unwrap();
        for (it, f) in t.0.iter().zip(narc.files()) {
            assert_eq!(&it.to_bytes()[..], f);
        }
        let potion = t.get(17).unwrap();
        assert_eq!(
            (potion.price(), potion.fling_power, potion.usage[0xD]),
            (300, 30, 20)
        );
        assert_eq!(t.get(1).unwrap().price(), 0);
        let cheri = t.get(149).unwrap();
        assert_eq!(
            (cheri.natural_gift_type(), cheri.natural_gift_power),
            (9, 60)
        );
        let ron = crate::to_ron(&t).unwrap();
        assert_eq!(crate::from_ron::<ItemTable>(&ron).unwrap(), t);
    }
}
