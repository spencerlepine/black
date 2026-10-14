# fndiff spec (F11): calender.c (overlay 10 Thumb code 0x02164778..0x02164B6C, F1 row 10.24) minus its script-command handlers.
# File bound: `Heap_AllocDebug(heap, 0x11C, 0, "calender.c", 0x53)` in 021647A0 (string 0x021729B0); eventdata_system.c ends at
# 0x02164768, game_beacon_search.c starts at 0x02164B6C. The four command handlers 458 / 467 / 468 / 469 (02164778, 02164910,
# 02164994, 02164978) and the GMEVENT of 468 (021649E8) are black-script `scrcmd_misc10c` (F8-spec5); 14 functions here.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field calender --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry (19
# entries; 02164868 is not one: the literal word of the thunk 021647E8 decodes as a pc-relative load). Fragments (labels Ghidra lists
# as functions): 02164968 (inside 02164910), 021649B8 (inside 02164994), 021649FE / 02164A12 (inside 021649E8).
#
# CALENDAR (0x11C, 021647A0): +0 u16 heap, +4 GAMEDATA, +8 archive handle of ARC 97 (0x61), +0xC 0x44 records {u16 zone, u16 offset}
# (member 1 of the archive from byte 2, 0x110 bytes). The special-day byte of a zone on a date (02164844): the record of the zone gives
# a base offset into member 0, the date the day of the year (02164CC: the ROM table 0x02171C20 of 13 month lengths {0, 31, 29, 31,
# 30, 31, 30, 31, 31, 30, 31, 30, 31}, so February always has 29 days); a zone without a record (offset 0xFFFF) answers zonedata
# 02013C10(zone) instead. The date: 021690D0 / 021690E0 (GAMEDATA) = month / day of the RTC record (save block 0x25 + 0x10).
# CALENDAR EVENT work (0x1C, 02164B38): +0 GAMESYS, +4 GAMEDATA, +8 save, +0xC..+0x17 the parameter of proc 0xD8 (0x021BDB40):
# {GAMESYS, GAMEDATA, u8 flag}, +0x15 u8 (subscreen reset, read by step 6), +0x18 u8 flag.
#
# All blocks are `nowork`; objects are `arg buf` arenas; out-of-area callees are `force` stubs (021F5578: an overlay outside the field
# set; 021A0F40: field_subscreen.c, U2). Closure stubs (as core4 / F5b): a block lists the stubs of its callees and of the callees of
# its in-area descendants (checked with every function treated as ported: no fault, stray write or unstubbed callee). In-area getters
# whose answers callers compare are `pick` stubs. 021647E8 is a thunk (`ldr r3, =0x0216481D; bx r3`): its target is a stub in the
# plain run.
area calender
set field

# 021647A0(gd, heap) -> CALENDAR: c = Heap_AllocDebug(heap, 0x11C, 0, "calender.c", 0x53); 021647F0(c) (clear); c+0 = (u16) heap;
# c+4 = gd; 021647FC(c) (open ARC 97); 02164890(c) (read the zone records); c. Arena: arg 2 (r2 is overwritten).
fn 021647a0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x200
  stub 02030734 force ret=ptr:arg2:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 021647f0 ret=none args=v  # in-area: clear
  stub 021647fc ret=none args=v  # in-area: open ARC 97
  stub 02164890 ret=none args=v  # in-area: read the zone records
  stub 020490f4 force ret=u32 args=v,v  # archive open (id, heap)
  stub 0204944c force ret=none args=v,v,v,v,out:0x110  # archive read (handle, member, offset, size, dst)

# 021647D8(c): 0216480C(c) (close); Heap_Free(c).
fn 021647d8 ret=none inputs=60
  nowork
  arg buf 0x20
  stub 0216480c ret=none args=v  # in-area: close
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02049238 force ret=none args=v  # archive close (handle)

# 021647E8(c, zone): thunk to 0216481C (the special-day byte of today).
fn 021647e8 ret=u8 inputs=200
  nowork
  arg buf 0x120
  arg u32 alt range 0 8 || any
  fix arg0 0xC u16 alt range 0 8 || any rep=0x44,4
  fix arg0 0xE u16 alt oneof 0xFFFF 0xFFFE 0 || any rep=0x44,4
  stub 0216481c ret=u8 args=v,v  # in-area: special-day byte (c, zone)
  stub 021690d0 force ret=pick:1/2/6/12/13/0 args=v  # GAMEDATA -> month 1..12 (RTC record; 0 / 13: the table ends)
  stub 021690e0 force ret=pick+u32:1/2/28/29/31 args=v  # GAMEDATA -> day (RTC record)
  stub 02164844 ret=u8 args=v,v,v,v  # in-area: special-day byte (c, zone, month, day)
  stub 021648a8 ret=pick:0xFFFF/0/1/0x100/0xFFFE args=v,v  # in-area: zone record offset
  stub 021648cc ret=u16 args=v,v  # in-area: day of the year
  stub 02013c10 force ret=u32 args=v  # zonedata: zone byte (zone)
  stub 0204944c force ret=none args=v,v,v,v,out:1  # archive read (handle, member, offset, size, dst)

# 021647F0(c): c+0 (u16), c+4, c+8 = 0.
fn 021647f0 ret=none inputs=40
  nowork
  arg buf 0x10

# 021647FC(c): c+8 = 020490F4(0x61, (u16) c+0) (open ARC 97 with the calendar's heap).
fn 021647fc ret=none inputs=60
  nowork
  arg buf 0x10
  stub 020490f4 force ret=u32 args=v,v  # archive open (id, heap)

# 0216480C(c): 02049238(c+8) (close); c+8 = 0.
fn 0216480c ret=none inputs=60
  nowork
  arg buf 0x10
  stub 02049238 force ret=none args=v  # archive close (handle)

# 0216481C(c, zone): m = 021690D0(c+4), d = 021690E0(c+4) (month / day of the RTC record); 02164844(c, zone, (u8) m, (u8) d).
fn 0216481c ret=u8 inputs=200
  nowork
  arg buf 0x120
  arg u32 alt range 0 8 || any
  fix arg0 0xC u16 alt range 0 8 || any rep=0x44,4
  fix arg0 0xE u16 alt oneof 0xFFFF 0xFFFE 0 || any rep=0x44,4
  stub 021690d0 force ret=pick:1/2/6/12/13/0 args=v  # GAMEDATA -> month 1..12 (RTC record; 0 / 13: the table ends)
  stub 021690e0 force ret=pick+u32:1/2/28/29/31 args=v  # GAMEDATA -> day (RTC record)
  stub 02164844 ret=u8 args=v,v,v,v  # in-area: special-day byte (c, zone, month, day)
  stub 021648a8 ret=pick:0xFFFF/0/1/0x100/0xFFFE args=v,v  # in-area: zone record offset
  stub 021648cc ret=u16 args=v,v  # in-area: day of the year
  stub 02013c10 force ret=u32 args=v  # zonedata: zone byte (zone)
  stub 0204944c force ret=none args=v,v,v,v,out:1  # archive read (handle, member, offset, size, dst)

# 02164844(c, zone, m, d): o = 021648A8(c, zone); o == 0xFFFF -> 02013C10(zone); else 0204944C(c+8, 0, (u16)(o + 021648CC(m, d)), 1,
# &b) and (u8) b.
fn 02164844 ret=u8 inputs=300
  nowork
  arg buf 0x120
  arg u32 alt range 0 8 || any
  arg u32 alt range 0 13 || oneof 0xFFFFFFFF 0x80000000
  arg u32 alt range 0 31 || any
  fix arg0 0xC u16 alt range 0 8 || any rep=0x44,4
  fix arg0 0xE u16 alt oneof 0xFFFF 0xFFFE 0 || any rep=0x44,4
  stub 021648a8 ret=pick:0xFFFF/0/1/0x100/0xFFFE args=v,v  # in-area: zone record offset
  stub 021648cc ret=u16 args=v,v  # in-area: day of the year
  stub 02013c10 force ret=u32 args=v  # zonedata: zone byte (zone)
  stub 0204944c force ret=none args=v,v,v,v,out:1  # archive read (handle, member, offset, size, dst)

# 02164890(c): 0204944C(c+8, 1, 2, 0x110, c + 0xC) (the zone records: member 1 from byte 2).
fn 02164890 ret=none inputs=60
  nowork
  arg buf 0x120
  stub 0204944c force ret=none args=v,v,v,v,out:0x110  # archive read (handle, member, offset, size, dst)

# 021648A8(c, zone): the offset of the first of the 0x44 records at c + 0xC whose u16 key equals zone (a 32-bit compare), else
# 0xFFFF.
fn 021648a8 ret=u32 inputs=300
  nowork
  arg buf 0x120
  arg u32 alt range 0 8 || oneof 0x10000 0x10001 || any
  fix arg0 0xC u16 alt range 0 8 || any rep=0x44,4

# 021648CC(m, d): the day of the year: the first m (signed; none for m <= 0) of the 13 month lengths at ROM 0x02171C20, summed in
# u16, + (u16)(d - 1). m <= 13 (a larger m reads past the 13-byte stack copy).
fn 021648cc ret=u16 inputs=300
  nowork
  arg u32 alt range 0 13 || oneof 0xFFFFFFFF 0x80000000 0xFFFFFFF3
  arg u32 alt range 0 32 || oneof 0x10000 0x10001 || any

# 021649C8(w): the end callback of command 467: 021F5578(0201210C(w+0)) (overlay 0x52, outside the field set), 02159168(7) (field
# proc flag), 02034A5C(0x52) (unload); 1.
fn 021649c8 ret=u32 inputs=60
  nowork
  arg buf 0x10
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 021f5578 force ret=none args=v  # overlay 0x52 (outside the field set): end (fieldmap)
  stub 02159168 force ret=none args=v  # script_sys: clear field-proc flag (id)
  stub 02034a5c force ret=none args=v  # overlay unload (id)

# 02164A2C(event, seq, W) (CALENDAR EVENT proc): g = W+0, 020120F4(g), fm = 0201210C(g); by *seq: 0 -> child 021BE8D4(g, fm, 0,
# 0) (fade); 1 -> 020144DC(g, fm); 2 -> 020297AC(g, 0x483, 6, 0) (BGM), then falls into 3 (seq += 2); 3 -> W+0xC..+0x17 = {W+0, W+4,
# (u8) W+0x18}, 020120B4(g, 0xD8, 0x021BDB40, W+0xC) (proc), seq + 1, 0; 4 -> 0 while 020120F0(g), else child 02029690(g, 6, 0x3C);
# 5 -> 020145CC(g); 6 -> W+0x15 != 0: 021A0F40(02188CCC(fm), 0) (subscreen); child 021BE92C(g, fm, 0, 0, 1, 0, 0); 7 -> 02029CC0(g);
# 8 -> 1; else 0. A child event: 02012374(event, child), seq + 1, 0.
fn 02164a2c ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 8 || oneof 9 0xFFFFFFFF || any
  fix arg2 0x15 u8 alt const 0 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 021be8d4 force ret=u32 args=v,v,v,v  # fade event (g, fm, 0, 0)
  stub 020144dc force ret=u32 args=v,v  # event (g, fm)
  stub 020297ac force ret=u32 args=v,v,v,v  # BGM event (g, 0x483, 6, 0)
  stub 020120b4 force ret=none args=v,v,v,in:0xC  # proc call (g, 0xD8, proc data, param)
  stub 020120f0 force ret=pick:0/1 args=v  # GAMESYS: a proc is running
  stub 02029690 force ret=u32 args=v,v,v  # BGM event (g, 6, 0x3C)
  stub 020145cc force ret=u32 args=v  # event (g)
  stub 02188ccc force ret=u32 args=v  # FIELDMAP -> subscreen
  stub 021a0f40 force ret=none args=v,v  # field_subscreen (U2): set mode
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # fade event (g, fm, 0, 0, 1, 0, 0)
  stub 02029cc0 force ret=u32 args=v  # BGM event (g)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)

# 02164B38(g, -, flag) -> event = 020122C0(g, 0, 0x02164A2D, 0x1C); W = 020124E8(event): W+4 = 020120F4(g), W+8 = 02012EBC(W+4),
# W+0 = g, (u8) W+0x18 = flag. Arena: arg 3 (W).
fn 02164b38 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
