//! Zone events (ARC 125, member = zone id; 428 members for 427 zones).
//!
//! Layout (confirmed by an exact size match on all 428 members):
//! `size:u32` (= 4 + the record bytes), `n_bg:u8 n_npc:u8 n_warp:u8 n_trigger:u8`, then
//! `n_bg` x 20-byte [`BgEvent`], `n_npc` x 36-byte [`Npc`], `n_warp` x 20-byte [`Warp`],
//! `n_trigger` x 22-byte [`Trigger`], then a variable `tail` (4 bytes `00000000` when empty;
//! otherwise a list of small `(kind:u16, value:u16, ...)` entries, kept raw).
//! A member with `size == 0` (zone 427) has no counts and no records.

use super::Rd;
use crate::error::{Error, Result};

/// Background (sign / hidden item style) event, 20 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgEvent {
    /// Script id (UNVERIFIED).
    pub script: u16,
    pub unk_02: u16,
    pub unk_04: u32,
    /// Looks like tile x (e.g. 0x307 in Nuvema Town). UNVERIFIED.
    pub unk_08: u32,
    /// Looks like tile y/z. UNVERIFIED.
    pub unk_0c: u32,
    pub unk_10: u32,
}

/// Overworld NPC, 36 bytes: the MMDL HEADER that `fldmmdl.c` creates an object from (overlay 10 `FUN_0216CFC4` /
/// `FUN_0216D040` / `FUN_0216D0F8`; F4a-spec, layout in `black-field`'s README).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Npc {
    /// OBJ id (local object id, 0.. in file order; 0xFF = the player).
    pub id: u16,
    /// OBJ code (sprite; 0xA2..=0xB1 are read from event vars 0x4020.., 0xB9 takes its size from `params`).
    pub obj_code: u16,
    /// Move code (index of the move-proc table, ROM `0x021D5E04`).
    pub move_code: u16,
    /// Event type.
    pub event_type: u16,
    /// Event flag (the object is not created while it is set; for an alias: a zone id).
    pub event_flag: u16,
    /// Event id (script; 0xFFFF = alias).
    pub script: u16,
    /// Direction.
    pub dir: u16,
    /// Params 0..2 (OBJ code 0xB9: params 0 / 1 = size).
    pub params: [u16; 3],
    /// Move limit x.
    pub limit_x: i16,
    /// Move limit z.
    pub limit_z: i16,
    /// Position kind: 0 grid, 1 rail (`x` / `z` / `height` are then rail coordinates, not decoded).
    pub kind: u32,
    /// Grid x.
    pub x: u16,
    /// Grid z (the horizontal row; the height is separate).
    pub z: u16,
    /// Height y, fx32 (`n << 16`: 0x30000 = 3 tiles up).
    pub height: u32,
}

/// Warp, 20 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    /// Destination zone id (0xFFFF in 6 warps: special/dynamic target). Confirmed in Nuvema Town: 390/392/394/396 = its four house interiors.
    pub dest_zone: u16,
    pub unk_02: u16,
    pub unk_04: u32,
    /// Fixed-point-looking source position (UNVERIFIED).
    pub unk_08: u32,
    pub unk_0c: u32,
    pub unk_10: u32,
}

/// Trigger (coordinate script trigger), 22 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    /// Script id (UNVERIFIED).
    pub script: u16,
    pub unk_02: u16,
    pub unk_04: [u8; 6],
    /// Looks like tile x. UNVERIFIED.
    pub x: u16,
    /// Looks like tile y. UNVERIFIED.
    pub y: u16,
    pub unk_0e: [u8; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ZoneEvents {
    /// `false` for the one member with `size == 0`.
    pub has_records: bool,
    pub bg: Vec<BgEvent>,
    pub npcs: Vec<Npc>,
    pub warps: Vec<Warp>,
    pub triggers: Vec<Trigger>,
    pub tail: Vec<u8>,
}

impl ZoneEvents {
    pub fn parse(d: &[u8]) -> Result<ZoneEvents> {
        let mut r = Rd::new(d, "zone events");
        let size = r.u32()? as usize;
        if size == 0 {
            return Ok(ZoneEvents {
                tail: r.rest().to_vec(),
                ..Default::default()
            });
        }
        let counts: [u8; 4] = r.arr()?;
        let [nb, nn, nw, nt] = counts.map(usize::from);
        let want = 4 + 20 * nb + 36 * nn + 20 * nw + 22 * nt;
        if want != size {
            return Err(Error::invalid(
                "zone events",
                format!("size field {size:#x} != computed {want:#x}"),
            ));
        }
        let mut e = ZoneEvents {
            has_records: true,
            ..Default::default()
        };
        for _ in 0..nb {
            e.bg.push(BgEvent {
                script: r.u16()?,
                unk_02: r.u16()?,
                unk_04: r.u32()?,
                unk_08: r.u32()?,
                unk_0c: r.u32()?,
                unk_10: r.u32()?,
            });
        }
        for _ in 0..nn {
            e.npcs.push(Npc {
                id: r.u16()?,
                obj_code: r.u16()?,
                move_code: r.u16()?,
                event_type: r.u16()?,
                event_flag: r.u16()?,
                script: r.u16()?,
                dir: r.u16()?,
                params: [r.u16()?, r.u16()?, r.u16()?],
                limit_x: r.u16()? as i16,
                limit_z: r.u16()? as i16,
                kind: r.u32()?,
                x: r.u16()?,
                z: r.u16()?,
                height: r.u32()?,
            });
        }
        for _ in 0..nw {
            e.warps.push(Warp {
                dest_zone: r.u16()?,
                unk_02: r.u16()?,
                unk_04: r.u32()?,
                unk_08: r.u32()?,
                unk_0c: r.u32()?,
                unk_10: r.u32()?,
            });
        }
        for _ in 0..nt {
            e.triggers.push(Trigger {
                script: r.u16()?,
                unk_02: r.u16()?,
                unk_04: r.arr()?,
                x: r.u16()?,
                y: r.u16()?,
                unk_0e: r.arr()?,
            });
        }
        e.tail = r.rest().to_vec();
        Ok(e)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        if !self.has_records {
            o.extend(0u32.to_le_bytes());
            o.extend(&self.tail);
            return o;
        }
        let size = 4
            + 20 * self.bg.len()
            + 36 * self.npcs.len()
            + 20 * self.warps.len()
            + 22 * self.triggers.len();
        o.extend((size as u32).to_le_bytes());
        o.extend([
            self.bg.len() as u8,
            self.npcs.len() as u8,
            self.warps.len() as u8,
            self.triggers.len() as u8,
        ]);
        for b in &self.bg {
            o.extend(b.script.to_le_bytes());
            o.extend(b.unk_02.to_le_bytes());
            for v in [b.unk_04, b.unk_08, b.unk_0c, b.unk_10] {
                o.extend(v.to_le_bytes());
            }
        }
        for n in &self.npcs {
            for v in [
                n.id,
                n.obj_code,
                n.move_code,
                n.event_type,
                n.event_flag,
                n.script,
                n.dir,
                n.params[0],
                n.params[1],
                n.params[2],
                n.limit_x as u16,
                n.limit_z as u16,
            ] {
                o.extend(v.to_le_bytes());
            }
            o.extend(n.kind.to_le_bytes());
            o.extend(n.x.to_le_bytes());
            o.extend(n.z.to_le_bytes());
            o.extend(n.height.to_le_bytes());
        }
        for w in &self.warps {
            o.extend(w.dest_zone.to_le_bytes());
            o.extend(w.unk_02.to_le_bytes());
            for v in [w.unk_04, w.unk_08, w.unk_0c, w.unk_10] {
                o.extend(v.to_le_bytes());
            }
        }
        for t in &self.triggers {
            o.extend(t.script.to_le_bytes());
            o.extend(t.unk_02.to_le_bytes());
            o.extend(t.unk_04);
            o.extend(t.x.to_le_bytes());
            o.extend(t.y.to_le_bytes());
            o.extend(t.unk_0e);
        }
        o.extend(&self.tail);
        o
    }
}
