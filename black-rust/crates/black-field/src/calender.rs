//! `calender.c` (overlay 10, `0x02164778..0x02164B6C`, F1 row 10.24; F11) minus its four script-command handlers and
//! the GMEVENT of command 468 (black-script `scrcmd_misc10c`): the CALENDAR object (the special-day byte of a zone on
//! today's date, from ARC 97) and the CALENDAR EVENT (fade, BGM, the calendar proc 0xD8, fade back).
//!
//! Spec: `tools/rom-diff/specs/black-field/calender.spec` (its header has the records). Verified by
//! `tools/check-fn.sh black-field calender --per-function`. Every callee outside this file (heap, archive, zonedata,
//! GMEVENT, the fade / BGM events, the overlay-0x52 entry, the subscreen) is reached through [`FieldEnv::call`] by
//! address; the date comes from the RTC record of the save data (`FUN_021690D0` / `FUN_021690E0`, month / day).

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_47A0,
    0x0216_47D8,
    0x0216_47E8,
    0x0216_47F0,
    0x0216_47FC,
    0x0216_480C,
    0x0216_481C,
    0x0216_4844,
    0x0216_4890,
    0x0216_48A8,
    0x0216_48CC,
    0x0216_49C8,
    0x0216_4A2C,
    0x0216_4B38,
];

/// Address of the string `"calender.c"` (the file argument of the allocation in [`new`]).
pub const FILE_CALENDER: u32 = 0x0217_29B0;
/// Size of the CALENDAR object.
pub const CALENDAR_SIZE: u32 = 0x11C;
/// The archive of the calendar data (ARC 97).
pub const ARC_CALENDAR: u32 = 0x61;
/// Number of zone records at CALENDAR + 0xC.
pub const ZONE_RECORDS: u32 = 0x44;
/// "No record" offset of [`zone_offset`].
pub const NO_RECORD: u32 = 0xFFFF;
/// ROM table `0x02171C20`: the month lengths [`day_of_year`] sums (index 0 is 0; February always has 29 days).
pub const MONTH_DAYS: [u8; 13] = [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
/// The CALENDAR EVENT proc [`event_proc`] (Thumb address).
pub const EVENT_PROC: u32 = 0x0216_4A2D;
/// The proc 0xD8 the CALENDAR EVENT runs, and its data (`0x021BDB40`, overlay 21).
pub const CALENDAR_PROC_ID: u32 = 0xD8;
pub const CALENDAR_PROC_DATA: u32 = 0x021B_DB40;

/// C: `Heap_AllocDebug(heap, size, flag, file, line)` / `Heap_Free(block)`.
const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// C: `FUN_020490F4(id, heap)` open an archive, `FUN_02049238(handle)` close it, `FUN_0204944C(handle, member, offset,
/// size, dst)` read from a member.
const ARC_OPEN: u32 = 0x0204_90F4;
const ARC_CLOSE: u32 = 0x0204_9238;
const ARC_READ: u32 = 0x0204_944C;
/// C: `FUN_021690D0(gd)` / `FUN_021690E0(gd)`: month / day of the RTC record.
const RTC_MONTH: u32 = 0x0216_90D0;
const RTC_DAY: u32 = 0x0216_90E0;
/// C: `FUN_02013C10(zone)`: zonedata byte of a zone (the answer for a zone without a calendar record).
const ZONE_BYTE: u32 = 0x0201_3C10;
/// GAMESYS getters `FUN_020120F4` (GAMEDATA) / `FUN_0201210C` (FIELDMAP); GAMEDATA -> save `FUN_02012EBC`.
const GAMEDATA: u32 = 0x0201_20F4;
const FIELDMAP: u32 = 0x0201_210C;
const SAVE: u32 = 0x0201_2EBC;
/// GMEVENT create / work / call child.
const EVENT_NEW: u32 = 0x0201_22C0;
const EVENT_WORK: u32 = 0x0201_24E8;
const EVENT_CALL: u32 = 0x0201_2374;

/// C: `FUN_021647A0(gd, heap)`: a new CALENDAR (0x11C bytes on `heap`): cleared, `+0` the heap, `+4` the GAMEDATA, ARC 97
/// opened and its zone records read.
pub fn new<E: FieldEnv>(env: &mut E, gd: u32, heap: u32) -> u32 {
    let c = env.callv(HEAP_ALLOC, &[heap, CALENDAR_SIZE, 0, FILE_CALENDER, 0x53]);
    clear(env, c);
    env.w16(c, heap as u16);
    env.w32(c + 4, gd);
    open(env, c);
    read_records(env, c);
    c
}

/// C: `FUN_021647D8(c)`: close the archive and free the CALENDAR.
pub fn delete<E: FieldEnv>(env: &mut E, c: u32) {
    close(env, c);
    env.callv(HEAP_FREE, &[c]);
}

/// C: `FUN_021647E8(c, zone)`: a thunk to [`today`].
pub fn today_thunk<E: FieldEnv>(env: &mut E, c: u32, zone: u32) -> u32 {
    today(env, c, zone)
}

/// C: `FUN_021647F0(c)`: `+0` (u16), `+4`, `+8` = 0.
pub fn clear<E: FieldEnv>(env: &mut E, c: u32) {
    env.w16(c, 0);
    env.w32(c + 4, 0);
    env.w32(c + 8, 0);
}

/// C: `FUN_021647FC(c)`: `+8` = the handle of ARC 97 opened with the calendar's heap.
pub fn open<E: FieldEnv>(env: &mut E, c: u32) {
    let heap = env.r16(c) as u32;
    let h = env.callv(ARC_OPEN, &[ARC_CALENDAR, heap]);
    env.w32(c + 8, h);
}

/// C: `FUN_0216480C(c)`: close the archive, `+8` = 0.
pub fn close<E: FieldEnv>(env: &mut E, c: u32) {
    let h = env.r32(c + 8);
    env.callv(ARC_CLOSE, &[h]);
    env.w32(c + 8, 0);
}

/// C: `FUN_0216481C(c, zone)`: the special-day byte of `zone` on the date of the RTC record ([`special_day`] with the
/// month and day truncated to bytes).
pub fn today<E: FieldEnv>(env: &mut E, c: u32, zone: u32) -> u32 {
    let gd = env.r32(c + 4);
    let m = env.callv(RTC_MONTH, &[gd]);
    let gd = env.r32(c + 4);
    let d = env.callv(RTC_DAY, &[gd]);
    special_day(env, c, zone, m & 0xFF, d & 0xFF)
}

/// C: `FUN_02164844(c, zone, m, d)`: the byte at offset `(u16)(record offset + day of the year)` of ARC 97 member 0 for a
/// zone with a record, else `FUN_02013C10(zone)`.
pub fn special_day<E: FieldEnv>(env: &mut E, c: u32, zone: u32, m: u32, d: u32) -> u32 {
    let o = zone_offset(env, c, zone);
    if o == NO_RECORD {
        return env.callv(ZONE_BYTE, &[zone]);
    }
    let doy = day_of_year(m, d);
    let off = o.wrapping_add(doy) & 0xFFFF;
    let h = env.r32(c + 8);
    let mut b = [0u8; 1];
    env.call(
        ARC_READ,
        &mut [
            CArg::V(h),
            CArg::V(0),
            CArg::V(off),
            CArg::V(1),
            CArg::OutBuf(&mut b),
        ],
    );
    b[0] as u32
}

/// C: `FUN_02164890(c)`: read the 0x44 zone records (member 1 of ARC 97 from byte 2, 0x110 bytes) to `c + 0xC`.
pub fn read_records<E: FieldEnv>(env: &mut E, c: u32) {
    let h = env.r32(c + 8);
    env.call(
        ARC_READ,
        &mut [
            CArg::V(h),
            CArg::V(1),
            CArg::V(2),
            CArg::V(ZONE_RECORDS * 4),
            CArg::Out(c + 0xC, ZONE_RECORDS * 4),
        ],
    );
}

/// C: `FUN_021648A8(c, zone)`: the offset of the first zone record whose key equals `zone` (compared as 32 bits, so a
/// `zone` above 0xFFFF never matches), else [`NO_RECORD`].
pub fn zone_offset<E: FieldEnv>(env: &mut E, c: u32, zone: u32) -> u32 {
    for i in 0..ZONE_RECORDS {
        let r = c + 0xC + 4 * i;
        if env.r16(r) as u32 == zone {
            return env.r16(r + 2) as u32;
        }
    }
    NO_RECORD
}

/// C: `FUN_021648CC(m, d)`: the day of the year, `(u16)(sum of the first m month lengths (u16) + (u16)(d - 1))`; no month
/// is summed for a signed `m <= 0`. `m` must be at most 13 (the original sums its 13-byte stack copy of [`MONTH_DAYS`];
/// a larger `m` reads past it, UNVERIFIED: this port panics).
pub fn day_of_year(m: u32, d: u32) -> u32 {
    let mut s: u32 = 0;
    let n = m as i32;
    let mut i = 0;
    while i < n {
        s = (s + MONTH_DAYS[i as usize] as u32) & 0xFFFF;
        i += 1;
    }
    (s + (d.wrapping_sub(1) & 0xFFFF)) & 0xFFFF
}

/// C: `FUN_021649C8(w)`: the end callback of script command 467: `FUN_021F5578(fieldmap)` (overlay 0x52, outside the
/// field set), clear field-proc flag 7, unload overlay 0x52; 1.
pub fn cmd467_end<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let g = env.r32(w);
    let fm = env.callv(FIELDMAP, &[g]);
    env.callv(0x021F_5578, &[fm]);
    env.callv(0x0215_9168, &[7]);
    env.callv(0x0203_4A5C, &[0x52]);
    1
}

/// C: `FUN_02164A2C(event, seq, W)`: the CALENDAR EVENT proc (steps in the spec header): fade out, `FUN_020144DC`, BGM
/// 0x483, the calendar proc 0xD8 with `{GAMESYS, GAMEDATA, flag}` (step 2 falls into step 3, so it advances by two),
/// wait for the proc, BGM back, `FUN_020145CC`, fade in (resetting the subscreen when `W + 0x15` is set), `FUN_02029CC0`;
/// step 8 ends the event (1).
pub fn event_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let g = env.r32(w);
    env.callv(GAMEDATA, &[g]);
    let fm = env.callv(FIELDMAP, &[g]);
    let s = env.r32(seq);
    let child = match s {
        0 => env.callv(0x021B_E8D4, &[g, fm, 0, 0]),
        1 => env.callv(0x0201_44DC, &[g, fm]),
        2 | 3 => {
            if s == 2 {
                let c = env.callv(0x0202_97AC, &[g, 0x483, 6, 0]);
                env.callv(EVENT_CALL, &[event, c]);
                let n = env.r32(seq).wrapping_add(1);
                env.w32(seq, n);
            }
            let p = w + 0xC;
            env.mem().fill(p, 0xC, 0);
            let a = env.r32(w);
            env.w32(p, a);
            let b = env.r32(w + 4);
            env.w32(p + 4, b);
            let f = env.r8(w + 0x18);
            env.w8(p + 8, f);
            env.call(
                0x0201_20B4,
                &mut [
                    CArg::V(g),
                    CArg::V(CALENDAR_PROC_ID),
                    CArg::V(CALENDAR_PROC_DATA),
                    CArg::In(p, 0xC),
                ],
            );
            let n = env.r32(seq).wrapping_add(1);
            env.w32(seq, n);
            return 0;
        }
        4 => {
            if env.callv(0x0201_20F0, &[g]) != 0 {
                return 0;
            }
            env.callv(0x0202_9690, &[g, 6, 0x3C])
        }
        5 => env.callv(0x0201_45CC, &[g]),
        6 => {
            if env.r8(w + 0x15) != 0 {
                let sub = env.callv(0x0218_8CCC, &[fm]);
                env.callv(0x021A_0F40, &[sub, 0]);
            }
            env.callv(0x021B_E92C, &[g, fm, 0, 0, 1, 0, 0])
        }
        7 => env.callv(0x0202_9CC0, &[g]),
        8 => return 1,
        _ => return 0,
    };
    env.callv(EVENT_CALL, &[event, child]);
    let n = env.r32(seq).wrapping_add(1);
    env.w32(seq, n);
    0
}

/// C: `FUN_02164B38(gsys, -, flag)`: a new CALENDAR EVENT (work 0x1C: `+0` GAMESYS, `+4` GAMEDATA, `+8` save, `+0x18` the
/// flag byte). The original's second argument is unused.
pub fn event_new<E: FieldEnv>(env: &mut E, g: u32, flag: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[g, 0, EVENT_PROC, 0x1C]);
    let w = env.callv(EVENT_WORK, &[ev]);
    let gd = env.callv(GAMEDATA, &[g]);
    env.w32(w + 4, gd);
    let sv = env.callv(SAVE, &[gd]);
    env.w32(w + 8, sv);
    env.w32(w, g);
    env.w8(w + 0x18, flag as u8);
    ev
}

#[cfg(test)]
mod tests;
