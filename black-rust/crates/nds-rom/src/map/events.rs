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

/// Background event (sign, hidden item), 20 bytes, as `eventdata_system.c` reads it (overlay 10 `FUN_02162970` /
/// `FUN_02162E48` / `FUN_02162E9C` / `FUN_02162A74`; F6a, layout in `black-field`'s README).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgEvent {
    /// Script id (for a hidden item `8000 + i`, item `i` of the hidden-item table).
    pub script: u16,
    /// Kind: 2 = hidden item (present while its event flag `table[i] + 900` is clear); other kinds use `facing`.
    pub kind: u16,
    /// Facing rule (kinds other than 2): the player's direction (0 up, 1 down, 2 left, 3 right) that triggers it:
    /// 0: dir 0, 1: dir 3, 2: dir 2, 3: dir 1, 4: any, 5: dir 2 or 3, 6: dir 0 or 1; other values never.
    pub facing: u16,
    /// Position kind: 0 grid, else rail.
    pub pos_kind: u16,
    /// Grid: tile x. Rail: low half = line, high half = front.
    pub x: u32,
    /// Grid: tile z. Rail: low half = side (s16).
    pub z: u32,
    /// Grid: height in `fx32 >> 12` units (16 per tile; the event is hit for `y - 2 <= pos.y >> 12 < y + 2`).
    pub y: i32,
}

impl BgEvent {
    /// Rail line (`+8`, rail records).
    pub fn rail_line(&self) -> u16 {
        self.x as u16
    }
    /// Rail front (`+0xA`, rail records).
    pub fn rail_front(&self) -> u16 {
        (self.x >> 16) as u16
    }
    /// Rail side (`+0xC`, rail records).
    pub fn rail_side(&self) -> i16 {
        self.z as u16 as i16
    }
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

/// Warp, 20 bytes, as `eventdata_system.c` reads it (overlay 10 `FUN_021625FC` / `FUN_02162D2C` / `FUN_02162E00` /
/// `FUN_02162B68` / `FUN_02162BCC` / `FUN_02162C84` / `FUN_02162D90`; F6a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warp {
    /// Destination zone id (0xFFFF in 6 warps: disabled / dynamic target). Confirmed in Nuvema Town: 390/392/394/396 = its four house interiors.
    pub dest_zone: u16,
    /// Destination warp index in the destination zone; 0x100 makes the LOCATION kind 3 (else 2); 0xFFFF (here or in
    /// `dest_zone`) = disabled.
    pub dest_warp: u16,
    /// Exit direction code: 1..4 = direction 0..3 (up, down, left, right); anything else counts as 1 (down).
    pub exit_dir: u8,
    /// Door / transition kind (read by `FUN_021626E8`).
    pub door: u8,
    /// Position kind: 0 grid, 1 rail.
    pub pos_kind: u16,
    /// Grid: x in `fx32 >> 12` units (16 per tile; a door's centre is `16 t + 8`). Rail: line.
    pub x: i16,
    /// Grid: y (height) in units. Rail: front start.
    pub y: i16,
    /// Grid: z in units. Rail: side start.
    pub z: i16,
    /// Grid: width in tiles (the warp spans `x ..= x + 16 (w - 1)`). Rail: front length.
    pub width: u16,
    /// Grid: depth in tiles. Rail: side length.
    pub depth: u16,
    /// Rail: flip mode of the exit offset (0: front, 1: side, 2: both). Grid: unread.
    pub flip: u16,
}

/// Trigger (coordinate script trigger), 22 bytes, as `eventdata_system.c` reads it (overlay 10 `FUN_021627C8` /
/// `FUN_02162F0C` / `FUN_02162FC8` / `FUN_02162ECC`; F6a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    /// Script id.
    pub script: u16,
    /// The value `var` must hold.
    pub value: u16,
    /// Event var (0 = unconditional, else the trigger fires while `var == value`).
    pub var: u16,
    /// Kind: 1..4 = only when facing direction 0..3, 6 = the separate kind-6 lookup, 0 = others.
    pub kind: u16,
    /// Position kind: 0 grid, else rail.
    pub pos_kind: u16,
    /// Grid: tile x. Rail: line.
    pub x: u16,
    /// Grid: tile z (the horizontal row). Rail: front start.
    pub z: u16,
    /// Grid: width in tiles. Rail: side start (s16).
    pub width: u16,
    /// Grid: depth in tiles. Rail: front length.
    pub depth: u16,
    /// Grid: height in `fx32 >> 12` units (hit for `y - 2 <= pos.y >> 12 < y + 2`). Rail: side length.
    pub y: i16,
    /// Not read by the code (0 in every ROM record).
    pub unused: u16,
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
                kind: r.u16()?,
                facing: r.u16()?,
                pos_kind: r.u16()?,
                x: r.u32()?,
                z: r.u32()?,
                y: r.u32()? as i32,
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
                dest_warp: r.u16()?,
                exit_dir: r.u8()?,
                door: r.u8()?,
                pos_kind: r.u16()?,
                x: r.u16()? as i16,
                y: r.u16()? as i16,
                z: r.u16()? as i16,
                width: r.u16()?,
                depth: r.u16()?,
                flip: r.u16()?,
            });
        }
        for _ in 0..nt {
            e.triggers.push(Trigger {
                script: r.u16()?,
                value: r.u16()?,
                var: r.u16()?,
                kind: r.u16()?,
                pos_kind: r.u16()?,
                x: r.u16()?,
                z: r.u16()?,
                width: r.u16()?,
                depth: r.u16()?,
                y: r.u16()? as i16,
                unused: r.u16()?,
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
            for v in [b.script, b.kind, b.facing, b.pos_kind] {
                o.extend(v.to_le_bytes());
            }
            for v in [b.x, b.z, b.y as u32] {
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
            o.extend(w.dest_warp.to_le_bytes());
            o.extend([w.exit_dir, w.door]);
            for v in [
                w.pos_kind, w.x as u16, w.y as u16, w.z as u16, w.width, w.depth, w.flip,
            ] {
                o.extend(v.to_le_bytes());
            }
        }
        for t in &self.triggers {
            for v in [
                t.script, t.value, t.var, t.kind, t.pos_kind, t.x, t.z, t.width, t.depth,
                t.y as u16, t.unused,
            ] {
                o.extend(v.to_le_bytes());
            }
        }
        o.extend(&self.tail);
        o
    }
}
