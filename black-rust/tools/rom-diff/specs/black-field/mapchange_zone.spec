# fndiff spec: event_mapchange.c part 3 (overlay 20 Thumb code 0x02185694..0x02186278; F6b): the special-location event, the zone entry
# (021857C0: LOCATION resolve, PLAYER record, the zone hooks, NPC / WFBC people, weather), the zone data load (02185A68: event data,
# map matrix, the flash flags), the SB list of a zone (ARC 101), the comm map-change events (02185BFC / 02185C4C, procs 02185D39 /
# 02185C8D), the special-location updates, the default record (message 190), the two event_mapchange.c allocations and the WFBC list
# helpers (0218609C..0218625C, used by the game-start events).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field mapchange_zone --per-function
# Function list, closure stubs, `force`, the work M and the arena layout: see the header of mapchange_proc.spec.
area mapchange_zone
set field

# 02185694(ev, seq, W) proc of 0218571C (W 0x24: +0 gsys, +4 gd, +8 LOCATION): 0: 0218562C(gsys); 1: 02012B2C(PLAYER, 0),
# 0202C110(0202B578(02012134(gsys))), call the zone BGM event (s16 W+0xC); 2: call 020145FC(gsys); 3: -> 1
fn 02185694 ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0xC u16 alt range 0 20 || any
  stub 0218562c force ret=none args=v  # zone entry from the special location (gsys)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02012134 force ret=u32 args=v  # GAMESYS getter
  stub 0202b578 force ret=u32 args=v  # (x)
  stub 0202c110 force ret=none args=v  # (x)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02029df4 force ret=u32 args=v,v,v  # BGM of a zone (gd, zone, season)
  stub 020298a0 force ret=u32 args=v,v,v,v  # (gsys, bgm, 0, 60) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020145fc force ret=u32 args=v  # (gsys) -> event

# 0218571C(gsys) -> event: proc 0x02185695, work 0x24 {gsys, gd, LOCATION 02159FA8(02012914(gd), W+8)}. Arena: arg 1.
fn 0218571c ret=u32 inputs=60
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02012914 force ret=u32 args=v  # GAMEDATA getter
  stub 02159fa8 force ret=none args=v,out:0x1C  # (x, &loc)

# 02185758(d) -> 0, 0, 0x8000, 0x4000, 0xC000 for d 0..4, else 0 (direction code -> angle)
fn 02185758 ret=u32 inputs=60
  nowork
  arg u32 alt range 0 5 || any

# 02185788(ed, src, dst): src kind (+0) 1: dst = *src; else 021625FC(ed, dst, u16 src+6, u16 src+0xA) == 0: 02014204(dst, u16
# src+4)
fn 02185788 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x20
  fix arg1 0 u32 alt const 1 || range 0 3 || any
  stub 021625fc force ret=pick:0/1 args=v,outm:0x1C:3/0/0/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v  # eventdata: exit location (ed, &loc, zone, warp): kind 0..3
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)

# 021857C0(gsys, loc): the zone entry: 02014454(02012B38(gd), 1); 02185788(ed, loc, &l); kind 3: 0201289C(gd, 02012870(gd));
# PLAYER record zone / direction (02185758(s16 l+8)) / position (grid 02012A90 or rail 02012AA4 by 020141DC); 0202B544; 02012854(gd,
# &l); 02159FE0 id -> 02012900; 02185B9C; 02168EEC; 021643E8; 0215A014; 02158AE0(gsys, 4); 021858E4(gd, loc); 02185A34; 021859FC(gd,
# &l); 0202C83C; weather. Arena: arg 2.
fn 021857c0 ret=none inputs=800
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg1 0 u32 alt const 1 || range 0 4 || any
  fix arg1 4 u16 alt range 0 3 || range 0 20 || any
  fix arg1 8 u16 alt range 0 5 || any
  fix arg2 0x284 u16 range 0 20
  fix arg2 0x400 u32 range 0 3 rep=8,8
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 02014454 force ret=none args=v,v  # (obj, flag)
  stub 02185788 ret=none args=v,in:0x1C,outm:0x1C:3/0/0/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # LOCATION resolve (ed, src, dst): dst kind 0..3
  stub 02012870 force ret=ptr:arg2:0x280:1 args=v  # GAMEDATA -> LOCATION (+0x164)
  stub 0201289c force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION = *loc
  stub 02012ac4 force ret=none args=v,v  # PLAYER record: zone
  stub 02185758 ret=u32 args=v  # direction code -> angle
  stub 02012acc force ret=none args=v,v  # PLAYER record: direction
  stub 020141dc force ret=pick+u32:0/1 args=in:0x1C  # LOCATION: rail position (+0xC)
  stub 02012a90 force ret=none args=v,in:12  # PLAYER record: position = *v
  stub 02012aa4 force ret=none args=v,in:8  # PLAYER record: rail location = *v
  stub 02012abc force ret=none args=v,v  # PLAYER record: position kind (rec, 0 grid / 1 rail)
  stub 02012134 force ret=u32 args=v  # GAMESYS getter
  stub 0202b544 force ret=none args=v,v  # (x, zone)
  stub 02012854 force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION = *loc
  stub 02159fe0 force ret=pick+u32:0/1/0x10005 args=v  # (zone) -> id
  stub 02012900 force ret=none args=v,v  # GAMEDATA (gd, id)
  stub 02185b9c ret=none args=v,v  # SB list of the zone (gd, zone)
  stub 02168eec force ret=none args=v  # (gd)
  stub 021643e8 force ret=none args=v,v  # (gd, zone)
  stub 0215a014 force ret=none args=v,v  # map_matrix (gd, zone)
  stub 02158ae0 force ret=none args=v,v  # (gsys, 4)
  stub 021858e4 ret=none args=v,in:0x1C  # NPCs of the zone (gd, loc)
  stub 02185a34 ret=none args=v,v  # zone flash flags (gd, zone)
  stub 021859fc ret=none args=v,in:0x1C  # WFBC zone hook (gd, loc)
  stub 0202c83c force ret=none args=v,v  # (gsys, zone)
  stub 0201405c force ret=pick+u32:0/5/15/16/17/0xFFFFFFFF args=v  # zonedata: weather (zone)
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0216e29c force ret=none args=v,v  # fldmmdl (sys, weather)
  stub 0216cccc force ret=none args=v  # fldmmdl (sys)
  stub 021625fc force ret=pick:0/1 args=v,outm:0x1C:3/0/0/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v  # eventdata: exit location (ed, &loc, zone, warp): kind 0..3
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 02012ec4 force ret=u32 args=v  # GAMEDATA -> SB list
  stub 0200b37c force ret=none args=v  # SB list clear (sb)
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap)
  stub 02049250 force ret=ptr:arg2:0x400:1 args=v,v,v  # archive member load (h, 0, heap)
  stub 020493d4 force ret=pick:0/8/16/24/0x40 args=v,v  # archive member size (h, 0)
  stub 0200b394 force ret=none args=v,v  # SB list: add (sb, id)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02049238 force ret=none args=v  # archive close
  stub 02162744 force ret=nullable:2:u32 args=v  # eventdata: NPC list
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02162740 force ret=u32 args=v  # eventdata getter
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # fldmmdl: add OBJs (sys, list, zone, n, ew)
  stub 02013f6c force ret=pick+u32:0/1/2/3 args=v  # zonedata flags (zone)
  stub 02014468 force ret=pick:0/1 args=v  # (obj) flash state
  stub 0201446c force ret=none args=v,v  # (obj, flags)
  stub 02186470 force ret=none args=v  # WFBC local (gd)
  stub 02013dd0 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02185928 ret=none args=v,in:0x1C  # WFBC people of the zone (gd, loc)
  stub 0201444c force ret=pick:0/1 args=v  # (obj) flag
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 021864dc force ret=none args=v  # WFBC local (list)
  stub 020127d0 force ret=ptr:arg2:0x4C0:1 args=v,v  # GAMEDATA -> WFBC list (gd, i)
  stub 02186538 force ret=nullable:2:u32 args=v,v,v  # WFBC local alloc (list, i, 1)
  stub 02186278 force ret=nullable:2:u32 args=v,v  # WFBC local (list, i)
  stub 020127e0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02186854 force ret=u32 args=v  # WFBC local (x)
  stub 021868b8 force ret=nullable:2:u32 args=v,v,v,v  # WFBC local alloc (x, i, n, 1)

# 021858E4(gd, loc): m = 02162744(ed); m != 0: 0216CEAC(MMDLSYS, 02162740(ed), s16 loc+4, m, EVENT WORK)
fn 021858e4 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  fix arg1 4 u16 alt range 0 20 || any
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162744 force ret=nullable:2:u32 args=v  # eventdata: NPC list
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 02162740 force ret=u32 args=v  # eventdata getter
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # fldmmdl: add OBJs (sys, list, zone, n, ew)

# 02185928(gd, loc): 02013DD0(zone) != 0: i = 0201444C(02012B38(gd)); list = i ? 020127D0(gd, 0) : 020127C4(gd); a =
# 02186538(list, i, 1), n = 02186278(list, i): both: 0216CEAC(MMDLSYS, a, s16 loc+4, n, EVENT WORK); free a; b = 021868B8(x =
# 020127E0(gd), i, u16 list+2, 1), n = 02186854(x): both: 0216CEAC; free b. Arena: arg 2.
fn 02185928 ret=none inputs=400
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg1 4 u16 alt range 0 20 || any
  stub 02013dd0 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201444c force ret=pick:0/1 args=v  # (obj) flag
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 020127d0 force ret=ptr:arg2:0x4C0:1 args=v,v  # GAMEDATA -> WFBC list (gd, i)
  stub 02186538 force ret=nullable:2:u32 args=v,v,v  # WFBC local alloc (list, i, 1)
  stub 02186278 force ret=nullable:2:u32 args=v,v  # WFBC local (list, i)
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # fldmmdl: add OBJs (sys, list, zone, n, ew)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 020127e0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02186854 force ret=u32 args=v  # WFBC local (x)
  stub 021868b8 force ret=nullable:2:u32 args=v,v,v,v  # WFBC local alloc (x, i, n, 1)

# 021859F0(gd): 0216CF6C(02012964(gd))
fn 021859f0 ret=none inputs=60
  nowork
  arg u32 any
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0216cf6c force ret=none args=v  # fldmmdl (sys)

# 021859FC(gd, loc): 02186470(gd); 02013DD0(zone) != 0: 02185928(gd, loc), 0201444C(02012B38(gd)) == 0: 021864DC(020127C4(gd)).
# Arena: arg 2.
fn 021859fc ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg1 4 u16 alt range 0 20 || any
  stub 02186470 force ret=none args=v  # WFBC local (gd)
  stub 02013dd0 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02185928 ret=none args=v,in:0x1C  # WFBC people of the zone (gd, loc)
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201444c force ret=pick:0/1 args=v  # (obj) flag
  stub 020127c4 force ret=ptr:arg2:0x480:1 args=v  # GAMEDATA -> WFBC list 1 (thunk to 020127D0(gd, 1))
  stub 021864dc force ret=none args=v  # WFBC local (list)
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020127d0 force ret=ptr:arg2:0x4C0:1 args=v,v  # GAMEDATA -> WFBC list (gd, i)
  stub 02186538 force ret=nullable:2:u32 args=v,v,v  # WFBC local alloc (list, i, 1)
  stub 02186278 force ret=nullable:2:u32 args=v,v  # WFBC local (list, i)
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # fldmmdl: add OBJs (sys, list, zone, n, ew)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 020127e0 force ret=u32 args=v  # GAMEDATA getter (WFBC)
  stub 02186854 force ret=u32 args=v  # WFBC local (x)
  stub 021868b8 force ret=nullable:2:u32 args=v,v,v,v  # WFBC local alloc (x, i, n, 1)

# 02185A34(gd, zone): f = 02013F6C(zone); bit 0 and 02014468(o): f = f & ~1 | 2; 0201446C(o = 02012B38(gd), f)
fn 02185a34 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 02013f6c force ret=pick+u32:0/1/2/3 args=v  # zonedata flags (zone)
  stub 02014468 force ret=pick:0/1 args=v  # (obj) flash state
  stub 0201446c force ret=none args=v,v  # (obj, flags)

# 02185A68(gsys, zone): the zone data load: 02014450(o, 02012B70(gd) == 1 and 02013D5C(zone) == 0); the GAMEDATA kind
# (020129B8: 2 / 5 by zonedata predicates, else 3 / 0 by 02012B70 and 020129AC, then 0 unless 3 / 4 / 6); 02162408(ed, zone, season);
# map matrix 02159B94(mm, 02013B44(zone), zone, 0x8001), 02159CC4(mm, gsys, 0x8001); 02185A34(gd, zone)
fn 02185a68 ret=none inputs=1500
  nowork
  arg u32 any
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012b70 force seq ret=pick+u8:0/1/2 args=v  # GAMEDATA u8 +0x1CC (read twice)
  stub 02013d5c force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 02014450 force ret=none args=v,v  # (obj, flag)
  stub 02013cc8 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02013ce0 force ret=pick+u32:0/1/0 args=v  # zonedata predicate (zone)
  stub 02013df0 force ret=pick+u32:0/1/0/0/0 args=v  # zonedata predicate (zone)
  stub 02013d20 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # zonedata predicate (zone)
  stub 020129ac force seq ret=pick+u32:3/0/4/6/3/2/5 args=v  # GAMEDATA kind getter (read up to three times)
  stub 020129b8 force ret=none args=v,v  # GAMEDATA setter (gd, k)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02162408 force ret=none args=v,v,v  # eventdata load (ed, zone, season)
  stub 02012974 force ret=u32 args=v  # GAMEDATA -> map matrix
  stub 02013b44 force ret=u32 args=v  # zonedata (zone)
  stub 02159b94 force ret=none args=v,v,v,v  # map matrix load (mm, id, zone, heap)
  stub 02159cc4 force ret=none args=v,v,v  # map matrix (mm, gsys, heap)
  stub 02185a34 ret=none args=v,v  # zone flash flags (gd, zone)
  stub 02013f6c force ret=pick+u32:0/1/2/3 args=v  # zonedata flags (zone)
  stub 02014468 force ret=pick:0/1 args=v  # (obj) flash state
  stub 0201446c force ret=none args=v,v  # (obj, flags)

# 02185B70(gsys): 0201210C(gsys); 021859F0(gd)
fn 02185b70 ret=none inputs=60
  nowork
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 021859f0 ret=none args=v  # MMDLSYS (gd)
  stub 02012964 force ret=u32 args=v  # GAMEDATA -> MMDLSYS
  stub 0216cf6c force ret=none args=v  # fldmmdl (sys)

# 02185B88(fm, x): 0216DAC8(021A3074(02188CB4(fm)), x)
fn 02185b88 ret=none inputs=60
  nowork
  arg u32 any
  arg u32 any
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216dac8 force ret=u32 args=v,v  # fldmmdl (mmdl, x)

# 02185B9C(gd, zone): sb = 02012EC4(gd); 0200B37C(sb); h = 020490F4(101, 0x8001); t = 02049250(h, 0, 0x8001); the first of the
# 020493D4(h, 0) / 8 records with t[i].u32 == zone: 0200B394(sb, t[i].u16 +4); free t; close h. Arena: arg 2 (table at +0x400).
fn 02185b9c ret=none inputs=400
  nowork
  arg u32 any
  arg u32 alt range 0 3 || any
  arg buf 0x1000
  fix arg2 0x400 u32 range 0 3 rep=8,8
  stub 02012ec4 force ret=u32 args=v  # GAMEDATA -> SB list
  stub 0200b37c force ret=none args=v  # SB list clear (sb)
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap)
  stub 02049250 force ret=ptr:arg2:0x400:1 args=v,v,v  # archive member load (h, 0, heap)
  stub 020493d4 force ret=pick:0/8/16/24/0x40 args=v,v  # archive member size (h, 0)
  stub 0200b394 force ret=none args=v,v  # SB list: add (sb, id)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02049238 force ret=none args=v  # archive close

# 02185BFC(gsys, fm, loc, a, b) -> event: proc 0x02185D39, work 0x30 {LOCATION, gsys, gd, fm, a, u8 b}. Arena: arg 5.
fn 02185bfc ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x20
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA

# 02185C4C(gsys, fm, loc) -> event: proc 0x02185C8D, work 0x30 {LOCATION, gsys, gd, fm}. Arena: arg 3.
fn 02185c4c ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA

# 02185C8C(ev, seq, W) proc of 02185C4C: 0: SE 0x773, 021CF498(02189A08(fm), ev, 36); 1: call a map-change event (0x02184505,
# M init, M+0x10 = W LOCATION, M+0x2C = M+0x40 = 0); 2: call 021BE9F0(gsys, fm, 0, 0, 2); 3: -> 1. Arena: arg 3.
fn 02185c8c ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02189a08 force ret=u32 args=v  # FIELDMAP getter
  stub 021cf498 force ret=none args=v,v,v  # (x, event, 36)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 02184cec force ret=none args=v,v  # M init (M, gsys)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 021be9f0 force ret=u32 args=v,v,v,v,v  # (gsys, fm, 0, 0, 2) -> event

# 02185D38(ev, seq, W) proc of 02185BFC (comm): 0: script 0x279A / 0x279B (021D2750 and no comm block) or 0x279C / 0x279D,
# seq 1 / 3; 1: wait for the comm (02158590 / 02026454), W+0x28 == 1 and 02158630 != 3: script 0x279F, seq 3; else comm record updates,
# 02012B78(gd, 1), 02012B64(gd, own id), call 02185C4C(gsys, fm, W), seq 2; 2: 0217574C(I, 0), seq 3; 3: -> 1. Arena: arg 3.
fn 02185d38 ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  arg buf 0x1000
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0x28 u32 alt oneof 0 1 || any
  fix arg3 0x186 u16 any
  fix arg3 0xA02 u8 range 1 8
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 02158344 force ret=nullable:3:ptr:arg3:0x100:1 args=v  # comm record I (X1) of I0
  stub 0217c070 force ret=pick+u32:0/1 args=v  # comm (I)
  stub 02158630 force ret=pick+u32:0/1/3 args=v  # comm state (I0)
  stub 021d2750 force ret=pick:0/1 args=v  # (gsys)
  stub 02188c84 force ret=u32 args=v  # FIELDMAP getter
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, x)
  stub 02158590 force ret=pick:0/1 args=v  # (gd)
  stub 02026400 force ret=none args=v  # comm (I0)
  stub 02026454 force ret=pick+u32:0/1/2/3 args=v  # comm state (I0)
  stub 02026484 force ret=none args=v  # comm (I0)
  stub 0217574c force ret=none args=v,v  # comm (I, flag)
  stub 02175710 force ret=none args=v,v  # comm (I, b)
  stub 02175b4c force ret=none args=v,v  # comm (I, I + 0x1AC)
  stub 02012b78 force ret=none args=v,v  # GAMEDATA setter
  stub 02039338 force ret=u32 args=  # own id
  stub 02012b64 force ret=none args=v,v  # GAMEDATA setter
  stub 02185c4c ret=u32 args=v,v,in:0x1C  # comm map change event (gsys, fm, &loc)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA

# 02185EB0(gd, loc): l1 = 02012870(gd), l2 = 0201284C(gd); 02014068(zone) != 0: l2 zone 136 -> 020128C0(gd, l1), else
# 02014068(l2 zone) == 0 -> 020128C0(gd, LOCATION 02014204(&t, 214)); l2 zone 158 and 02013C50(zone) -> 020128C0(gd, l1). Arena: arg 2.
fn 02185eb0 ret=none inputs=400
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg1 4 u16 alt range 0 20 || any
  fix arg2 0x284 u16 range 0 20
  fix arg2 0x244 u16 alt oneof 136 158 || range 0 20
  stub 02012870 force ret=ptr:arg2:0x280:1 args=v  # GAMEDATA -> LOCATION (+0x164)
  stub 0201284c force ret=ptr:arg2:0x240:1 args=v  # GAMEDATA -> LOCATION (+0x160)
  stub 02014068 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 020128c0 force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION (+0x16C) = *loc
  stub 02013c50 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)

# 02185F18(gd, dst): 02013FEC(u16 0201284C(gd)+4) != 0: *dst = *02012894(gd). Arena: arg 2.
fn 02185f18 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x1000
  fix arg2 0x244 u16 alt oneof 136 158 || range 0 20
  fix arg2 0x200 u32 alt range 0 4 || any
  fix arg2 0x204 u16 alt oneof 136 158 || range 0 20
  fix arg2 0x208 u16 range 0 4
  stub 0201284c force ret=ptr:arg2:0x240:1 args=v  # GAMEDATA -> LOCATION (+0x160)
  stub 02013fec force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02012894 force ret=ptr:arg2:0x200:1 args=v  # GAMEDATA -> LOCATION

# 02185F48(gd): PLAYER form 3 -> 2, else -> 0
fn 02185f48 ret=none inputs=60
  nowork
  arg u32 any
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012b28 force ret=pick+u32:0/1/2/3 args=v  # PLAYER record +0x40 (move form)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form

# 02185F6C(gd): a default record from message bank 190 string 59 (name), fields 1 / 3 / 2 / 21, the three ROM strings
# 0x02187DA4 / 0x02187DAC / 0x02187DB4, value 0x58F, stored with 0200903C(02008FEC(gd), 0, 0, rec)
fn 02185f6c ret=none inputs=60
  nowork
  arg u32 any
  stub 02045b38 force ret=u32 args=v,v,v,v  # message data open (0, 2, 190, 1)
  stub 02045ec0 force ret=u32 args=v,v  # message data -> STRBUF (m, id)
  stub 02045a38 force ret=none args=v,out:32,v  # STRBUF -> u16 buffer (b, buf, 16)
  stub 02008f08 force ret=u32 args=v  # MYSTATUS-like record alloc (heap)
  stub 02008f88 force ret=none args=v,in:32  # record: name = buf
  stub 02008fa0 force ret=none args=v,v  # record field set
  stub 02008fa8 force ret=none args=v,v  # record field set
  stub 02008fb4 force ret=none args=v,v  # record field set
  stub 02008fbc force ret=none args=v,v  # record field set
  stub 02008fd8 force ret=none args=v,v,v  # record: entry i = ROM string
  stub 02008fc4 force ret=none args=v,v  # record field set
  stub 02008fec force ret=u32 args=v  # GAMEDATA -> record store
  stub 0200903c force ret=none args=v,v,v,v  # record store: put (store, 0, 0, rec)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02045808 force ret=none args=v  # STRBUF free
  stub 02045c04 force ret=none args=v  # message data close

# 0218601C(gd): sv = 02012EBC(gd); 0216A358(gd, 0201AD0C(sv), 02008B14(sv) + 20, season)
fn 0218601c ret=none inputs=60
  nowork
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 0201ad0c force ret=u32 args=v  # save data getter
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02008b14 force ret=u32 args=v  # save block 0x25 + 0x10
  stub 0216a358 force ret=none args=v,v,v,v  # (gd, x, y, season)

# 0218604C() -> Heap_AllocDebug(0x8004, 0x1000, 0, "event_mapchange.c", 0xE35)
fn 0218604c ret=u32 inputs=20
  nowork
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 0218606C() -> Heap_AllocDebug(0x8004, 0x1000, 0, "event_mapchange.c", 0xE3A)
fn 0218606c ret=u32 inputs=20
  nowork
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 0218608C(): empty
fn 0218608c ret=none inputs=10
  nowork

# 02186090(x) -> 0216D588(x) - 0x80
fn 02186090 ret=u32 inputs=60
  nowork
  arg u32 any
  stub 0216d588 force ret=pick+u32:0x80/0x81/0x7F/0 args=v  # fldmmdl: MMDL OBJ id

# 0218609C(L): 02161E78(L + 8 + 24 i), i 0..9
fn 0218609c ret=none inputs=20
  nowork
  arg u32 any
  stub 02161e78 force ret=none args=v  # WFBC record clear

# 021860B8(L): 02161E78(L + 0xF8 + 24 i), i 0..9
fn 021860b8 ret=none inputs=20
  nowork
  arg u32 any
  stub 02161e78 force ret=none args=v  # WFBC record clear

# 021860D4(L, ms, heap): the WFBC people list: L+0 = 1, L+2 = 0, L+4 = 02008530(ms); h = 021867C4(0, heap & 0x7FFF | 0x8000);
# 10 records of list A (+8) and 3 of list B (+0xF8) from 021869E0(L, 020056EC(budget), h, &k) into 02186B60's free record (02186B84(rec,
# k, ms, A: L+2 ? r+4 : r+6 / B: 0, flag)), budget 0x1CE minus each pick's u16 +44; 02186804(h). Arena: arg 3.
fn 021860d4 ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 02008530 force ret=u32 args=v  # MYSTATUS -> trainer id
  stub 021867c4 force ret=u32 args=v,v  # WFBC local: open (0, heap)
  stub 0218682c force ret=u32 args=v  # WFBC local: data of a handle
  stub 020056ec force ret=lt:0 args=v  # rng: next below n (black-core rng)
  stub 02186b60 force ret=u32 args=v  # WFBC local: free record (list)
  stub 021869e0 force ret=ptr:arg3:0x500:4 args=outk:4:0/0/1/0,v,v,outm:4:0xFF/0/0/0  # WFBC local: pick (list, r, h, &out); may set bit 0 of list+2
  stub 02186b84 force ret=none args=v,v,v,v,v  # WFBC local: record init (rec, b, id, x, flag)
  stub 02186804 force ret=none args=v  # WFBC local: close (handle)

# 021861BC(L) -> 0 when L+0 == 1 and L+2 >= 2, else 1 when every list-A record answers 02161E84
fn 021861bc ret=u32 inputs=200
  nowork
  arg buf 0x200
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 2 u16 alt range 0 3 || any
  stub 02161e84 force seq ret=pick:1/1/1/1/1/1/1/1/1/0 args=v  # WFBC record predicate

# 021861F4(L, ms): L+0 > 1 -> 1; L+0 == 0 and L+2 != 2 -> L+2 = 2; L+0 == 1 and L+2 >= 2 -> L+2 = 021584F4((u8) 0200855C(ms),
# 02008530(ms)) == 21 ? 0 : 1; 021866F8(L + 8 + 24 i), i 0..9
fn 021861f4 ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 any
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 2 u16 alt range 0 3 || any
  stub 0200855c force ret=u32 args=v  # MYSTATUS -> byte (region / version)
  stub 02008530 force ret=u32 args=v  # MYSTATUS -> trainer id
  stub 021584f4 force ret=pick+u32:21/0/1 args=v,v  # (a, b)
  stub 021866f8 force ret=none args=v  # WFBC local record

# 0218625C(L) -> L+0 == 0 ? 0 : 021861BC(L) != 0
fn 0218625c ret=u32 inputs=200
  nowork
  arg buf 0x200
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 2 u16 alt range 0 3 || any
  stub 021861bc ret=pick:0/1 args=v  # WFBC list predicate
  stub 02161e84 force seq ret=pick:1/1/1/1/1/1/1/1/1/0 args=v  # WFBC record predicate
