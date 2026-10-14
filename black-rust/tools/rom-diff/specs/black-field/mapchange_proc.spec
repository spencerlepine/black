# fndiff spec: event_mapchange.c part 1 (overlay 20 Thumb code 0x02183B60..0x02184CEC; F1 row 20.00; F6b): the game-start events
# (new game / continue / the WFBC reset), the MAPCHANGE work helpers (season check, the door / transition sub-record at M + 0x4C), the
# zone-change child event (proc 0x021841C1, overlay 0xE6 around the zone switch) and the map-change GMEVENT procs (door, warp, stairs,
# fly, teleport, dig, escape rope, fall, rail, union room). File bound: `Heap_AllocDebug(0x8004, 0x1000, 0, "event_mapchange.c", ...)`
# in 0218604C (string 0x02187EA0); field_wfbc_data_local.c starts at 0x02186278.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field mapchange_proc --per-function
#
# Function list (all three mapchange areas, 86 entries): bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a
# Thumb walk from every entry; the 6 F1 extra entries are switch-case labels (02183BF8, 02183D0A, 02183D5A, 02183E04, 021841E2, 02184542,
# 021845BA, 021845DE, 02184636, 021856FA are Ghidra `LAB_` pieces). Render-only functions: none (the VRAM clears 0208068C / 02082AC0 /
# 02080B98 are D3 hooks, stubbed).
# Closure stubs (as core4 / F4a / F5b): a block lists the stubs of its callees and of the callees of its in-area descendants, so the
# area can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force` (also between the three mapchange areas).
#
# MAPCHANGE work M (0x9C, 02184CEC): +0 GAMESYS, +4 GAMEDATA, +8 FIELDMAP, +0xC u16 the zone being left (02188C9C), +0x10 the
# destination LOCATION (0x1C: +0 kind, +4 u16 zone, +6 u16 warp, +8 s16 direction, +0xC rail flag, +0x10 position), +0x2C the door
# kind (021626E8), +0x30 u8 the zone-change kind (0..4, 021841C0), +0x34 a 12-byte position (02184E40), +0x40 / +0x44 the season
# change flags, +0x48 / +0x4A u16 the new / old season (0218403C), +0x4C the door / transition sub-record (0218409C / 02184120: +0 the
# event, +4 / +8 / +0xC M+0 / +4 / +8, +0x10 M+0x2C, +0x14 door kind of the exit warp, +0x18 u16 M+0xC, +0x1C the LOCATION copy,
# +0x38 / +0x3C the fade kinds, +0x40 the season flag, +0x44 / +0x45 the season bytes, +0x48 / +0x4C 0).
# Blocks are `nowork`: a GMEVENT proc is (event, seq buf, work buf); the last `buf 0x1000` argument is the arena that the stubs' pointer
# answers point into (GMEVENT work at +0, comm record I at +0x100, LOCATIONs at +0x200..+0x300, positions +0x340 / +0x360, event-work
# vars +0x380, archive table +0x400, WFBC lists +0x480 / +0x4C0, WFBC records +0x500); the function never reads it as a register.
area mapchange_proc
set field

# 02183B60(f) -> ~f: calls f() first when f != 0 (the integrity-check callback invoker; f = fake callee 0x02EE0011)
fn 02183b60 ret=u32 inputs=60
  nowork
  arg u32 alt const 0 || const 0x02EE0011
  stub 02ee0010 ret=none args=  # fake callback

# 02183B6C(a, k) -> *k 0: 02183C34(a, k), 1: 02183F5C(a, k), 2: 02183D68(a, k), else a (the game-start event by kind)
fn 02183b6c ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 3 || any
  stub 02183c34 ret=u32 args=v,v  # event A new (gsys, x)
  stub 02183f5c ret=u32 args=v,v  # event C new (gsys, p)
  stub 02183d68 ret=u32 args=v,v  # event B new (gsys, x)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 0201433c force ret=none args=v,v  # event_work clear_flag (ew, flag)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 02008b08 force ret=u32 args=v  # save block 0x25 (thunk to 020071CC(x, 0x25))
  stub 0201ad0c force ret=u32 args=v  # save data getter
  stub 02012f08 force ret=u32 args=v  # GAMEDATA -> save block
  stub 02008b4c force ret=pick:0/1 args=v  # RTC record predicate
  stub 02008b78 force ret=pick:0/1 args=v  # RTC record predicate
  stub 02008b14 force ret=u32 args=v  # save block 0x25 + 0x10
  stub 02008c00 force ret=none args=v  # RTC record init (rec)
  stub 0216a390 force ret=none args=v,v  # (gd, x)
  stub 0203f524 force ret=u64 args=  # RTC: ticks (64-bit)
  stub 0200c384 force ret=none args=v,v,v  # (rec, tick lo, tick hi)
  stub 02008b20 force ret=none args=v  # RTC record: store the date (rec)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 0216aae4 force ret=none args=v,v,v  # (gd, x, season)
  stub 02183dd4 ret=none args=v  # WFBC post-process (gsys)
  stub 02021444 force ret=none args=  # reset (no arguments)
  stub 0208068c force ret=none args=v  # GX VRAM bank to LCDC (D3)
  stub 02082ac0 force ret=none args=v,v,v  # MI_CpuClearFast(0, VRAM, size) (D3)
  stub 02080b98 force ret=none args=  # GX VRAM LCDC off (D3)
  stub 02014264 force ret=none args=out:0x1C  # LOCATION clear
  stub 02183c58 ret=none args=v  # WFBC reset (gsys)
  stub 02185f6c force ret=none args=v  # default record (gd)
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02161d64 force ret=none args=v  # WFBC list post-process
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 021860d4 force ret=none args=v,v,v  # WFBC list generate (list, mystatus, heap)
  stub 020127f0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02161f14 force ret=none args=v,v,v  # WFBC (a, b, 1)

# 02183B90(ev, seq, W) proc of 02183C34 (W: +0 gsys, +4 x): 0: VRAM clear (0208068C(0x1FF), 02082AC0(0, 0x06800000, 0xA4000),
# 02080B98()); 1: 0202E8CC(gd, (u16) 0202E888()), call 02187808(gsys, s, s) (s = season); 2: call 02161B90(gsys, ev, 5, 0, 1);
# 3: change to 02183D68(gsys, W+4). Always 0. Arena: arg 3.
fn 02183b90 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0208068c force ret=none args=v  # GX VRAM bank to LCDC (D3)
  stub 02082ac0 force ret=none args=v,v,v  # MI_CpuClearFast(0, VRAM, size) (D3)
  stub 02080b98 force ret=none args=  # GX VRAM LCDC off (D3)
  stub 0202e888 force ret=u32 args=  # RTC: time slot
  stub 0202e8cc force ret=none args=v,v  # (gd, slot)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02187808 force ret=u32 args=v,v,v  # el_scoreboard (gsys, season, season) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02161b90 force ret=u32 args=v,v,v,v,v  # (gsys, ev, 5, 0, 1) -> event
  stub 02183d68 ret=u32 args=v,v  # event B new (gsys, x)
  stub 0201235c force ret=none args=v,v  # GMEVENT change (event, next)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 02014264 force ret=none args=out:0x1C  # LOCATION clear
  stub 02183c58 ret=none args=v  # WFBC reset (gsys)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 02008b08 force ret=u32 args=v  # save block 0x25 (thunk to 020071CC(x, 0x25))
  stub 02008b20 force ret=none args=v  # RTC record: store the date (rec)
  stub 02021444 force ret=none args=  # reset (no arguments)
  stub 02185f6c force ret=none args=v  # default record (gd)
  stub 020127c4 force ret=ptr:arg3:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 021860d4 force ret=none args=v,v,v  # WFBC list generate (list, mystatus, heap)
  stub 02161d64 force ret=none args=v  # WFBC list post-process
  stub 020127f0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02161f14 force ret=none args=v,v,v  # WFBC (a, b, 1)

# 02183C34(gsys, x) -> event: proc 0x02183B91, work 8 {gsys, x}
fn 02183c34 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)

# 02183C58(gsys): gd; 021860D4(020127C4(gd), 02012944(gd), 1); 02161D64(020127C4(gd)); 02161F14(020127F0(gd), 020127C4(gd), 1)
fn 02183c58 ret=none inputs=60
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127c4 force ret=ptr:arg1:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 021860d4 force ret=none args=v,v,v  # WFBC list generate (list, mystatus, heap)
  stub 02161d64 force ret=none args=v  # WFBC list post-process
  stub 020127f0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02161f14 force ret=none args=v,v,v  # WFBC (a, b, 1)

# 02183C9C(ev, seq, W) proc of 02183D68 (W 0x24: +0 gsys, +4 gd, +8 LOCATION): 0: 02158A10(gsys), 02185A68(gsys, u16 W+0xC),
# 021857C0(gsys, W+8); 1: call 020298A0(gsys, 02029DF4(gd, s16 W+0xC, season), 0, 60); 2: call 020145FC(gsys); 3: call 021BE92C(gsys,
# fm, 1, 0, 1, 0, 0); 4: place name 021BC578(02188D20(fm), s16 W+0xC) when 02188D20(fm) != 0 -> 1. Steps 0..3 move on. Arena: arg 3.
fn 02183c9c ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02158a10 force ret=none args=v  # (gsys)
  stub 02185a68 force ret=none args=v,v  # zone data load (gsys, zone)
  stub 021857c0 force ret=none args=v,in:0x1C  # zone entry (gsys, loc)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02029df4 force ret=u32 args=v,v,v  # BGM of a zone (gd, zone, season)
  stub 020298a0 force ret=u32 args=v,v,v,v  # (gsys, bgm, 0, 60) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020145fc force ret=u32 args=v  # (gsys) -> event
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc578 force ret=none args=v,v  # place name (sys, zone)

# 02183D68(gsys, x) -> event (new game): proc 0x02183C9D, work 0x24 {gsys, gd, LOCATION 02014264}; 02183C58(gsys); RTC record
# 02008B20(02008B08(02012EBC(gd))); 02021444(); 02185F6C(gd); VRAM clear
fn 02183d68 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02014264 force ret=none args=out:0x1C  # LOCATION clear
  stub 02183c58 ret=none args=v  # WFBC reset (gsys)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 02008b08 force ret=u32 args=v  # save block 0x25 (thunk to 020071CC(x, 0x25))
  stub 02008b20 force ret=none args=v  # RTC record: store the date (rec)
  stub 02021444 force ret=none args=  # reset (no arguments)
  stub 02185f6c force ret=none args=v  # default record (gd)
  stub 0208068c force ret=none args=v  # GX VRAM bank to LCDC (D3)
  stub 02082ac0 force ret=none args=v,v,v  # MI_CpuClearFast(0, VRAM, size) (D3)
  stub 02080b98 force ret=none args=  # GX VRAM LCDC off (D3)
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 021860d4 force ret=none args=v,v,v  # WFBC list generate (list, mystatus, heap)
  stub 02161d64 force ret=none args=v  # WFBC list post-process
  stub 020127f0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02161f14 force ret=none args=v,v,v  # WFBC (a, b, 1)

# 02183DD4(gsys): 02161D64(020127C4(gd))
fn 02183dd4 ret=none inputs=60
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127c4 force ret=ptr:arg1:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02161d64 force ret=none args=v  # WFBC list post-process

# 02183DE4(ev, seq, W) proc of 02183F5C (W 0x10: +0 gsys, +4 gd, +8 u16 zone, +0xC flag 0x966 was set): 0: 0201445C(02012B38(gd),
# 1); W+0xC: zone = LOCATION(02012894) zone, 02185B70, 02185A68, 0218601C, 021857C0(gsys, loc), 0202735C; else 02185A68, 0218601C, 02168EEC,
# 02164474, 0202C860, weather 0201405C (< 16: 0216E29C, else 0216CCCC); 1: call the zone BGM event; 2: call 020145FC(gsys); 3: W+0xC and
# var 0x4041 != 0: script 0x83B, else call 021BE92C(gsys, fm, 3, 0, 0, s, s); 4: place name -> 1. Arena: arg 3.
fn 02183de4 ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 8 u16 alt range 0 20 || any
  fix arg2 0xC u32 alt oneof 0 1 || any
  fix arg3 0x200 u32 alt range 0 4 || any
  fix arg3 0x204 u16 alt oneof 136 158 || range 0 20
  fix arg3 0x208 u16 range 0 4
  fix arg3 0x380 u16 alt const 0 || any
  fix arg3 0x384 u16 alt const 0 || any
  fix arg3 0x388 u16 alt const 0 || any
  fix arg3 0x38C u16 alt const 0 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201445c force ret=none args=v,v  # (obj, flag)
  stub 02012894 force ret=ptr:arg3:0x200:1 args=v  # GAMEDATA -> LOCATION
  stub 02185b70 force ret=none args=v  # zone leave (gsys)
  stub 02185a68 force ret=none args=v,v  # zone data load (gsys, zone)
  stub 0218601c force ret=none args=v  # (gd)
  stub 021857c0 force ret=none args=v,in:0x1C  # zone entry (gsys, loc)
  stub 0202735c force ret=none args=v,v  # (zone, gd)
  stub 02168eec force ret=none args=v  # (gd)
  stub 02164474 force ret=none args=v,v  # eventdata_tail zone hook (gd, zone)
  stub 0202c860 force ret=none args=v,v  # (gd, zone)
  stub 0201405c force ret=pick+u32:0/5/15/16/17/0xFFFFFFFF args=v  # zonedata: weather (zone)
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0216e29c force ret=none args=v,v  # fldmmdl (sys, weather)
  stub 0216cccc force ret=none args=v  # fldmmdl (sys)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02029df4 force ret=u32 args=v,v,v  # BGM of a zone (gd, zone, season)
  stub 020298a0 force ret=u32 args=v,v,v,v  # (gsys, bgm, 0, 60) -> event
  stub 020145fc force ret=u32 args=v  # (gsys) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020143cc force ret=ptr:arg3:0x380:4 args=v,v  # event_work var address (ew, var)
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, x)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc578 force ret=none args=v,v  # place name (sys, zone)

# 02183F5C(gsys, p) -> event (continue): proc 0x02183DE5, work 0x10 {gsys, gd, u16 p+0x10, flag 0x966}; clears flag 0x966; RTC
# record checks (02008B4C / 02008B78) -> reinit + 0216A390 + 0200C384(rec, ticks); 0216AAE4; 02183DD4; 02021444(); VRAM clear
fn 02183f5c ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 0201433c force ret=none args=v,v  # event_work clear_flag (ew, flag)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 02008b08 force ret=u32 args=v  # save block 0x25 (thunk to 020071CC(x, 0x25))
  stub 0201ad0c force ret=u32 args=v  # save data getter
  stub 02012f08 force ret=u32 args=v  # GAMEDATA -> save block
  stub 02008b4c force ret=pick:0/1 args=v  # RTC record predicate
  stub 02008b78 force ret=pick:0/1 args=v  # RTC record predicate
  stub 02008b14 force ret=u32 args=v  # save block 0x25 + 0x10
  stub 02008c00 force ret=none args=v  # RTC record init (rec)
  stub 0216a390 force ret=none args=v,v  # (gd, x)
  stub 0203f524 force ret=u64 args=  # RTC: ticks (64-bit)
  stub 0200c384 force ret=none args=v,v,v  # (rec, tick lo, tick hi)
  stub 02008b20 force ret=none args=v  # RTC record: store the date (rec)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 0216aae4 force ret=none args=v,v,v  # (gd, x, season)
  stub 02183dd4 ret=none args=v  # WFBC post-process (gsys)
  stub 02021444 force ret=none args=  # reset (no arguments)
  stub 0208068c force ret=none args=v  # GX VRAM bank to LCDC (D3)
  stub 02082ac0 force ret=none args=v,v,v  # MI_CpuClearFast(0, VRAM, size) (D3)
  stub 02080b98 force ret=none args=  # GX VRAM LCDC off (D3)
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 02161d64 force ret=none args=v  # WFBC list post-process

# 0218403C(M): 0202E848(gd, &a, &b); rec = 0201407C(02188C84(fm), 02013B38(u16 M+0x14), 0); f = 020140D8(rec); free rec; a != b and
# f: M+0x44 = 1, M+0x48 = a, M+0x4A = b
fn 0218403c ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 0x14 u16 alt range 0 20 || any
  stub 0202e848 force ret=none args=v,outv:2:0/1/2/3,outv:2:0/1/2/3  # (x, &a, &b)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02013b38 force ret=u32 args=v  # zonedata (zone)
  stub 0201407c force ret=u32 args=v,v,v  # record load (a, b, 0)
  stub 020140d8 force ret=pick+u32:0/1 args=v  # record predicate
  stub 020140b4 force ret=none args=v  # Heap_Free veneer

# 0218409C(M, ev): the door sub-record at M+0x4C (see the header); M+0x4C+0x48 = +0x4C = 0
fn 0218409c ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 0xC u16 alt range 0 20 || any
  fix arg0 0x10 u32 alt oneof 0 1 || any
  fix arg0 0x14 u16 alt range 0 20 || any
  fix arg0 0x30 u8 range 0 5
  fix arg0 0x40 u32 alt oneof 0 1 || any
  fix arg0 0x44 u32 alt oneof 0 1 || any
  stub 021be89c force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be8b8 force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 0202e8a4 force ret=u32 args=v  # time slot -> code

# 02184120(M, ev): as 0218409C; +0x14 = M+0x10 == 1 ? 0 : 021626E8(021625CC(02012844(gd), u16 M+0x16))
fn 02184120 ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 0xC u16 alt range 0 20 || any
  fix arg0 0x10 u32 alt oneof 0 1 || any
  fix arg0 0x14 u16 alt range 0 20 || any
  fix arg0 0x30 u8 range 0 5
  fix arg0 0x40 u32 alt oneof 0 1 || any
  fix arg0 0x44 u32 alt oneof 0 1 || any
  stub 021be89c force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be8b8 force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 0202e8a4 force ret=u32 args=v  # time slot -> code
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (ed, idx)
  stub 021626e8 force ret=pick+u32:0/1/2/3/5/6/7 args=v  # eventdata: warp door kind

# 021841C0(ev, seq, W) proc 0x021841C1 of 02184360 (W: +0 M): 0: 0201447C(02012B38(gd), 2), call 02014554(gsys, fm); 1: overlay 0xE6 load,
# 02013D34(zone) == 1 and 02026454(I0) in 1..2: 02026400(I0), seq 2, else seq 3; 2: wait for 02026454(I0) == 0, seq 3; 3: integrity
# calls 021882A0 / 02188318 / 021883CC (cb 0x02183B61), 02185B70, season change (0202E8CC, 0216AAE4, 0216A358), 02185A68, 021857C0,
# 0202735C unless kind 4, zone hook by M+0x30 (1: 02164454, 2: 02164464, 3: 0216445C), overlay 0xE6 unload, 0215874C(I0); 4: call
# 020145FC; 5: call 02029C64; 6: -> 1. Arena: arg 4 (M in arg 3).
fn 021841c0 ret=u32 inputs=800
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 7 || any
  fix arg2 0 u32 addrof arg3 0
  fix arg3 0xC u16 alt range 0 20 || any
  fix arg3 0x10 u32 alt oneof 0 1 || any
  fix arg3 0x14 u16 alt range 0 20 || any
  fix arg3 0x30 u8 range 0 5
  fix arg3 0x40 u32 alt oneof 0 1 || any
  fix arg3 0x44 u32 alt oneof 0 1 || any
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201447c force ret=none args=v,v  # (obj, flag)
  stub 02014554 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02034ac4 force ret=none args=v  # overlay load (id)
  stub 02013d34 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02026454 force ret=pick+u32:0/1/2/3 args=v  # comm state (I0)
  stub 02026400 force ret=none args=v  # comm (I0)
  stub 021882a0 force ret=none args=v,v,v  # integrity check (ARM, other overlay)
  stub 02185b70 force ret=none args=v  # zone leave (gsys)
  stub 0202e8cc force ret=none args=v,v  # (gd, slot)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 02008b14 force ret=u32 args=v  # save block 0x25 + 0x10
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02012934 force ret=u32 args=v  # GAMEDATA -> party
  stub 0216aae4 force ret=none args=v,v,v  # (gd, x, season)
  stub 0216a358 force ret=none args=v,v,v,v  # (gd, x, y, season)
  stub 02188318 force ret=none args=v,v,v  # integrity check (ARM, other overlay)
  stub 02185a68 force ret=none args=v,v  # zone data load (gsys, zone)
  stub 021857c0 force ret=none args=v,in:0x1C  # zone entry (gsys, loc)
  stub 0202735c force ret=none args=v,v  # (zone, gd)
  stub 021883cc force ret=none args=v,v,v  # integrity check (ARM, other overlay)
  stub 02164454 force ret=none args=v,v  # eventdata_tail zone hook (gd, zone)
  stub 02164464 force ret=none args=v,v  # eventdata_tail zone hook (gd, zone)
  stub 0216445c force ret=none args=v,v  # eventdata_tail zone hook (gd, zone)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 0215874c force ret=none args=v  # (I0)
  stub 020145fc force ret=u32 args=v  # (gsys) -> event
  stub 02029c64 force ret=u32 args=v  # (gsys) -> event

# 02184360(M, kind) -> event: proc 0x021841C1, work 4 {M}; M+0x30 = kind (u8)
fn 02184360 ret=u32 inputs=60
  nowork
  arg buf 0x100
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)

# 02184388(ev, seq, M) proc (door): 0: 0218403C, MMDLs pause, 0218409C(M, ev), call 021BD648(M+0x4C), -> 33; 1: call
# 02184360(M, 0); 2: 02184120(M, ev), call 021BD9BC(M+0x4C); 3: -> 1. Arena: arg 3.
fn 02184388 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 0218403c ret=none args=v  # M: season change check (M)
  stub 02188c7c force ret=u32 args=v  # FIELDMAP -> MMDLSYS
  stub 0216d9a0 force ret=none args=v  # fldmmdl: MMDLSYS pause all
  stub 0218409c ret=none args=v,v  # M: door sub-record (M, ev)
  stub 021bd648 force ret=u32 args=v  # door / transition event (sub)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 02184120 ret=none args=v,v  # M: door sub-record, exit (M, ev)
  stub 021bd9bc force ret=u32 args=v  # door / transition event (sub)
  stub 0202e848 force ret=none args=v,outv:2:0/1/2/3,outv:2:0/1/2/3  # (x, &a, &b)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02013b38 force ret=u32 args=v  # zonedata (zone)
  stub 0201407c force ret=u32 args=v,v,v  # record load (a, b, 0)
  stub 020140d8 force ret=pick+u32:0/1 args=v  # record predicate
  stub 020140b4 force ret=none args=v  # Heap_Free veneer
  stub 021be89c force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be8b8 force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 0202e8a4 force ret=u32 args=v  # time slot -> code
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (ed, idx)
  stub 021626e8 force ret=pick+u32:0/1/2/3/5/6/7 args=v  # eventdata: warp door kind

# 02184410(ev, seq, M) proc (door, comm): I = 02158344(02012138(gsys)); 0: MMDLs pause, seq 5 if 02013DD0(zone) == 1 else 1; 1:
# 0218409C, call 021BD648 -> 33; 2: call 02184360(M, 0); 3: 02184120, call 021BD9BC; 4: -> 1; 5: I null: seq 7, 021746AC(I,
# 0215841C(gd)) == 1: seq 6; 6: I null: seq 7, 02158428(I) == 1: seq 1; 7: MMDLs resume -> 1. Arena: arg 3.
fn 02184410 ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 8 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  fix arg3 0x186 u16 any
  fix arg3 0xA02 u8 range 1 8
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 02158344 force ret=nullable:3:ptr:arg3:0x100:1 args=v  # comm record I (X1) of I0
  stub 02188c7c force ret=u32 args=v  # FIELDMAP -> MMDLSYS
  stub 0216d9a0 force ret=none args=v  # fldmmdl: MMDLSYS pause all
  stub 02013dd0 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 0218409c ret=none args=v,v  # M: door sub-record (M, ev)
  stub 021bd648 force ret=u32 args=v  # door / transition event (sub)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 02184120 ret=none args=v,v  # M: door sub-record, exit (M, ev)
  stub 021bd9bc force ret=u32 args=v  # door / transition event (sub)
  stub 0215841c force ret=u32 args=v  # (gd)
  stub 021746ac force ret=pick+u32:0/1 args=v,v  # comm (I, x)
  stub 02158428 force ret=pick+u32:0/1 args=v  # comm (I) byte
  stub 0216d9e4 force ret=none args=v  # fldmmdl: MMDLSYS resume all
  stub 021be89c force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be8b8 force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 0202e8a4 force ret=u32 args=v  # time slot -> code
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (ed, idx)
  stub 021626e8 force ret=pick+u32:0/1/2/3/5/6/7 args=v  # eventdata: warp door kind

# 02184504(ev, seq, M) proc (warp): P = 02012A30(gd); 0: call 02161824(gsys, fm); 1: BGM 02029D44(P, gd, u16 M+0x14, u8
# M+0x4A), 02029D7C(P); 2: call 02184360(M, M+0x30); 3: -> 1. Arena: arg 3.
fn 02184504 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184584(ev, seq, M) proc: 0: 0218403C, call 02161824; 1: call fade out 021BE8D4(gsys, fm, season change ? 0 :
# 021BE89C(u16 M+0xC, u16 M+0x14), 0); 2: call 02184360(M, M+0x30); 3: call fade in 021BE92C (season change: (3, 0, 0, u8 M+0x48,
# u8 M+0x4A), else (021BE8B8(..), 0, 1, 0, 0)); 4: -> 1. Arena: arg 3.
fn 02184584 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 0218403c ret=none args=v  # M: season change check (M)
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021be89c force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be8d4 force ret=u32 args=v,v,v,v  # (gsys, fm, kind, 0) -> event (fade out)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021be8b8 force ret=u32 args=v,v  # fade kind (zone a, zone b)
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 0202e848 force ret=none args=v,outv:2:0/1/2/3,outv:2:0/1/2/3  # (x, &a, &b)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02013b38 force ret=u32 args=v  # zonedata (zone)
  stub 0201407c force ret=u32 args=v,v,v  # record load (a, b, 0)
  stub 020140d8 force ret=pick+u32:0/1 args=v  # record predicate
  stub 020140b4 force ret=none args=v  # Heap_Free veneer
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 0218466C(ev, seq, M) proc: 0: call 02161824; 1..7: wait; 8: call 021BE8D4(gsys, fm, 2, 0); 9: BGM; 10: call 02184360(M,
# M+0x30); 11: call 021BE92C(gsys, fm, 2, 0, 0, 0, 0); 12: place name 021BC550(02188D20(fm), s16 M+0x14) -> 1; > 12: seq + 1. Arena: arg
# 3.
fn 0218466c ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 13 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 021be8d4 force ret=u32 args=v,v,v,v  # (gsys, fm, kind, 0) -> event (fade out)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184730(ev, seq, M) proc: 0: call 02161824; 1: call 021BFD34(ev, gsys, fm, M+0x34); 2: BGM; 3: call 02184360(M, 0); 4:
# call 021BF58C(ev, gsys, fm); 5: -> 1. Arena: arg 3.
fn 02184730 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 6 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021bfd34 force ret=u32 args=v,v,v,v  # transition event (ev, gsys, fm, x)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021bf58c force ret=u32 args=v,v,v  # transition event (ev, gsys, fm)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 021847CC(ev, seq, M) proc: 0: 0218403C, call 02161824; 1: call 021BFD70(ev, gsys, fm, M+0x44); 2: 02185F48(gd), BGM; 3:
# call 02184360(M, 2); 4: call 021BF5B4(ev, gsys, fm, M+0x44, u16 M+0x48, u16 M+0x4A); 5: place name -> 1. Arena: arg 3.
fn 021847cc ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 6 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 0218403c ret=none args=v  # M: season change check (M)
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021bfd70 force ret=u32 args=v,v,v,v  # transition event (ev, gsys, fm, x)
  stub 02185f48 force ret=none args=v  # player form reset (gd)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021bf5b4 force ret=u32 args=v,v,v,v,v,v  # transition event (ev, gsys, fm, x, a, b)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 0202e848 force ret=none args=v,outv:2:0/1/2/3,outv:2:0/1/2/3  # (x, &a, &b)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02013b38 force ret=u32 args=v  # zonedata (zone)
  stub 0201407c force ret=u32 args=v,v,v  # record load (a, b, 0)
  stub 020140d8 force ret=pick+u32:0/1 args=v  # record predicate
  stub 020140b4 force ret=none args=v  # Heap_Free veneer
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184894(ev, seq, M) proc: as 021847CC with 021BFDA0 / 021BF5E8. Arena: arg 3.
fn 02184894 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 6 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 0218403c ret=none args=v  # M: season change check (M)
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021bfda0 force ret=u32 args=v,v,v,v  # transition event (ev, gsys, fm, x)
  stub 02185f48 force ret=none args=v  # player form reset (gd)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021bf5e8 force ret=u32 args=v,v,v,v,v,v  # transition event (ev, gsys, fm, x, a, b)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 0202e848 force ret=none args=v,outv:2:0/1/2/3,outv:2:0/1/2/3  # (x, &a, &b)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02013b38 force ret=u32 args=v  # zonedata (zone)
  stub 0201407c force ret=u32 args=v,v,v  # record load (a, b, 0)
  stub 020140d8 force ret=pick+u32:0/1 args=v  # record predicate
  stub 020140b4 force ret=none args=v  # Heap_Free veneer
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 0218495C(ev, seq, M) proc: 0: call 02161824; 1: call 021BFDD0(ev, gsys, fm, 1); 2: 02012B2C(PLAYER, 0), BGM; 3: call
# 02184360(M, 3); 4: call 021BF61C(ev, gsys, fm); 5: place name -> 1. Arena: arg 3.
fn 0218495c ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 6 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021bfdd0 force ret=u32 args=v,v,v,v  # transition event (ev, gsys, fm, x)
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021bf61c force ret=u32 args=v,v,v  # transition event (ev, gsys, fm)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184A14(ev, seq, M) proc: x = 02188D38(fm); 0: call 021BE8D4(gsys, fm, 1, 0); 1: wait for 021C6E48(x), call 02161824; 2:
# call 02184360(M, 0); 3: call 021BE92C(gsys, fm, 1, 0, 1, 0, 0); 4: place name -> 1. Arena: arg 3.
fn 02184a14 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02188d38 force ret=u32 args=v  # FIELDMAP getter
  stub 021be8d4 force ret=u32 args=v,v,v,v  # (gsys, fm, kind, 0) -> event (fade out)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021c6e48 force ret=pick:0/1 args=v  # (x) done
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184AB4(ev, seq, M) proc: 0: call 02161824; 1: call 021BE8D4(gsys, fm, 0, 0); 2: call a copy of M (event 0x02184505,
# 0x9C bytes); 3: call 021BE92C(gsys, fm, 0, 0, 1, 0, 0); 4: place name -> 1. Arena: arg 3.
fn 02184ab4 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021be8d4 force ret=u32 args=v,v,v,v  # (gsys, fm, kind, 0) -> event (fade out)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, kind, 0, a, b, c) -> event (fade in)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)

# 02184B54(ev, seq, M) proc: 0: call 02161824; 1: call 021BFE00(ev, gsys, fm); 2: BGM, call 02184360(M, 0); 3: call
# 021BF644(ev, gsys, fm); 4: place name -> 1. Arena: arg 3.
fn 02184b54 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021bfe00 force ret=u32 args=v,v,v  # transition event (ev, gsys, fm)
  stub 02029d44 force ret=none args=v,v,v,v  # BGM (P, gd, zone, season)
  stub 02029d7c force ret=none args=v  # BGM (P)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 021bf644 force ret=u32 args=v,v,v  # transition event (ev, gsys, fm)
  stub 02188d20 force ret=nullable:2:u32 args=v  # FIELDMAP -> place-name system
  stub 021bc550 force ret=none args=v,v  # place name (sys, zone)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184BF8(ev, seq, M) proc (to the union room): 0: call 021BFDD0(ev, gsys, fm, 0); 1: call 02184360(M, 0); 2: overlay 0xC
# unload, 0xB load, script 0x83A; 3: -> 1. Arena: arg 3.
fn 02184bf8 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 021bfdd0 force ret=u32 args=v,v,v,v  # transition event (ev, gsys, fm, x)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 02034ac4 force ret=none args=v  # overlay load (id)
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, x)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)

# 02184C78(ev, seq, M) proc (from the union room): 0: call 02161824; 1: overlay 0xB unload, 0xC load, call 02184360(M, 4);
# 2: 02173744(gsys); 3: call 021BF66C(ev, gsys, fm); 4: -> 1. Arena: arg 3.
fn 02184c78 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u16 alt range 0 20 || any
  fix arg2 0x10 u32 alt oneof 0 1 || any
  fix arg2 0x14 u16 alt range 0 20 || any
  fix arg2 0x30 u8 range 0 5
  fix arg2 0x40 u32 alt oneof 0 1 || any
  fix arg2 0x44 u32 alt oneof 0 1 || any
  stub 02161824 force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 02034ac4 force ret=none args=v  # overlay load (id)
  stub 02184360 ret=u32 args=v,v  # zone change event (M, kind)
  stub 02173744 force ret=none args=v  # overlay 0xC code (gsys)
  stub 021bf66c force ret=u32 args=v,v,v  # transition event (ev, gsys, fm)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
