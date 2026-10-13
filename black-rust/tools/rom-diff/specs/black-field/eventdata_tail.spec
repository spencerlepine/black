# fndiff spec: the tail of F1 row 10.23 (overlay 10 Thumb code 0x021643AC..0x02164768; F6a-spec): zone-change hooks (021643AC /
# 021643E8: field_encount_st, flag 0x964, hidden-item respawn), the field-menu entry checks (02164480 / 021644C4, called from
# event_shortcut_menu.c, table ROM 0x021714DC) and the menu event constructors (021644F4, table ROM 0x02171504), two GMEVENTs
# (021645E0 / 021646A0) and the hidden-item flags (021646E8 / 02164708 / 02164728, ROM 0x02171534: 0xB1 records of 10 bytes, u16 +0 =
# flag - 900, u8 +3 = respawns).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field eventdata_tail --per-function
# Set `field` (main 10 11 18 20 21). F1 row 10.23 ("eventdata_system.c", 0x02161D98..0x02164768) holds four files; F6a-spec splits it by
# address into the areas wfbc_data (0x02161D98..0x0216237C), eventdata (0x0216237C..0x02163010), mmdl_move (0x02163010..0x021643AC) and
# eventdata_tail (0x021643AC..0x02164768); calls between them are `force` stubs (one: eventdata 02162970 -> eventdata_tail 02164708).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 197 entries in
# the row, minus 12 veneers (`ldr r3,=T; bx r3`, callers stub T: 02162024 -> 021622AC, 02162038 -> 02162330, 02162040 -> 02162358,
# 02162A64 -> 02162CE8, 02162A6C -> 02162ECC, 021626A8 -> 02162B68, 021626B0 -> 02162BCC, 02164454 / 5C / 64 / 6C -> 0215E34C,
# 02164474 -> 0215E364) and 3 false entries (02161E82, 021634A8 inside 02163464, 02163E72 inside 02163E58: data words in main
# 0x020A3E5A..0x020A481C decode as bl), = 185 specced (wfbc_data 40, eventdata 56, mmdl_move 70, eventdata_tail 19). Ghidra misses 02162370,
# 0216237C, 02162458, 02162478, 021624B0, 02162560, 021625E8, 021621C0, 02164240, 0216458C, 021645C0 (and splits 9 switch cases / tails
# into LAB_ functions). Render-only functions: none (the draw side is reached only through overlay-21 field-effect callees, all stubs).
# Closure stubs (as core4 / F4a): a block lists the stubs of its callees and of the callees of its in-area descendants and the shared fix
# lines, so the area can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed
# callee). Out-of-area callees are `force` stubs (overlay addresses collide with battle PORTED entries; ARM9 / fldmmdl callees are ported
# in other black-field areas). Indirect calls through ROM tables reach in-area stubs (plain) or force stubs at the table targets.
#
# All blocks are `nowork`; objects are `arg buf` arenas (the GMEVENT work that 020124E8 returns is the block's argument 2).
area eventdata_tail
set field

# 021643AC(gsys, fieldmap): 0215E428(0200D418(02012EBC(gsys)), 02188C9C(fieldmap) (zone)); 0215E3A0(gsys);
# clear_flag(0201296C(gsys), 0x964); 0215E33C(fieldmap)
fn 021643ac ret=none inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 0200d418 force ret=u32 args=v  # (gamedata handle) -> encount work
  stub 0201296c force ret=u32 args=v  # GAMESYS / GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201433c force ret=none args=v,v  # event_work clear_flag (ew, flag)
  stub 0215e33c force ret=none args=v  # field_encount_st (fieldmap)
  stub 0215e3a0 force ret=none args=v  # field_encount_st (gsys)
  stub 0215e428 force ret=none args=v,v  # field_encount_st (work, zone / loc)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id

# 021643E8(gsys, loc): fs = 02012B38(gsys); 02013F6C(loc) & 3 == 0: 02014464(fs, 0); 0215E428(0200D418(02012EBC(gsys)), loc);
# 0215E364(gsys); ew = 0201296C(gsys); clear_flag(ew, 0x964); 02164728(ew) (hidden items); 02013FFC(loc) == 0:
# 02008988(02008854(02012EBC(gsys)), 0)
fn 021643e8 ret=none inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 020056ec force ret=lt:0 args=v  # random below n (100)
  stub 02008854 force ret=u32 args=v  # (gsys handle) -> sound / system object
  stub 02008988 force ret=none args=v,v  # (object, 0)
  stub 0200d418 force ret=u32 args=v  # (gamedata handle) -> encount work
  stub 0201296c force ret=u32 args=v  # GAMESYS / GAMEDATA -> EVENT WORK
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> field_status
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02013f6c force ret=u32 args=v  # LOCATION / zone flags
  stub 02013ffc force ret=pick:0/1 args=v  # zone predicate
  stub 0201433c force ret=none args=v,v  # event_work clear_flag (ew, flag)
  stub 0201440c force ret=pick:0/1 args=v  # event_work: read and clear the marker byte +0x3E8 (ew)
  stub 02014464 force ret=none args=v,v  # field_status set (fs, 0)
  stub 0215e364 force ret=none args=v  # field_encount_st (gsys)
  stub 0215e428 force ret=none args=v,v  # field_encount_st (work, zone / loc)
  stub 02164728 ret=none args=v  # in-area

# 0216447C(): empty
fn 0216447c ret=none inputs=100
  nowork

# 02164480(menu, gsys, x): p = 020127A4(020120F4(gsys)); MI_CpuFill8(menu, 0, 0x18); i < 10: 02028748(menu, i, 021644C4(i, gsys, x));
# menu +4 = 02012B28(p)
fn 02164480 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA getter (party / pocket)
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02012b28 force ret=pick:0/1/2 args=v  # GAMEDATA byte getter
  stub 02013c20 force ret=pick:0/1 args=v  # party predicate
  stub 02013c50 force ret=pick:0/1 args=v  # party predicate
  stub 02013d20 force ret=pick:0/1 args=v  # party predicate
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate
  stub 02028748 force ret=none args=v,v,v  # menu entry set (menu, index, enabled)
  stub 021644c4 ret=pick:0/1 args=v,v,v  # in-area
  stub 02164508 ret=pick:0/1 args=v,v,v  # in-area
  stub 02164564 ret=pick:0/1 args=v,v,v  # in-area
  stub 0216458c ret=pick:0/1 args=v,v,v  # in-area
  stub 0216459c ret=pick:0/1 args=v,v,v  # in-area
  stub 021645c0 ret=pick:0/1 args=v,v,v  # in-area
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab4b8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c5c64 force ret=pick:0/1 args=v,v,v  # (a, b, 0)

# 021644C4(i, gsys, x) -> enabled: gd = 020120F4(gsys); p = 020127A4(gd); f = ROM 0x021714DC[i] (10 entries: 02164508, 0, 0,
# 02164564, 0216458C, 0216459C, 021645C0, 0, 0, 021645C0); f == 0: 1; f(gd, x, p)
fn 021644c4 ret=u32 inputs=100
  nowork
  arg u32 range 0 9
  arg u32 any
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA getter (party / pocket)
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02012b28 force ret=pick:0/1/2 args=v  # GAMEDATA byte getter
  stub 02013c20 force ret=pick:0/1 args=v  # party predicate
  stub 02013c50 force ret=pick:0/1 args=v  # party predicate
  stub 02013d20 force ret=pick:0/1 args=v  # party predicate
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate
  stub 02164508 ret=pick:0/1 args=v,v,v  # in-area
  stub 02164564 ret=pick:0/1 args=v,v,v  # in-area
  stub 0216458c ret=pick:0/1 args=v,v,v  # in-area
  stub 0216459c ret=pick:0/1 args=v,v,v  # in-area
  stub 021645c0 ret=pick:0/1 args=v,v,v  # in-area
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab4b8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c5c64 force ret=pick:0/1 args=v,v,v  # (a, b, 0)

# 021644F4(i, a, b) -> ROM 0x02171504[i].fn(b, a) (6 entries of 8 bytes: 02164654, 02164678, 02184E94, 021C6068, 021C5D10,
# 021646C4; the second word is unused here). Arena: argument 2 (the GMEVENT work of the in-area constructors)
fn 021644f4 ret=u32 inputs=100
  nowork
  arg u32 range 0 5
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 02164654 ret=u32 args=v,v  # in-area
  stub 02164678 ret=u32 args=v,v  # in-area
  stub 021646c4 ret=u32 args=v,v  # in-area
  stub 02184e94 force ret=u32 args=v,v  # ov20 event constructor (table 0x02171504 entry 2)
  stub 02185140 force ret=u32 args=v,v,in:12,v  # event_mapchange (ov20): event (gsys, zone 0x117, &pos, 0)
  stub 021c5d10 force ret=u32 args=v,v  # ov21 event constructor (table 0x02171504 entry 4)
  stub 021c6068 force ret=u32 args=v,v  # ov21 event constructor (table 0x02171504 entry 3)

# 02164508(gd, fieldmap, p) -> bool (menu entry 0): q = 02012AC8(p); b = 02012B28(p) (u8); r = 021AB4B8(021AB0F0(021A30C8(
# 02188CB4(fieldmap)))); b == 1: r == 0; else 02013C20(q) != 0 and b == 0 and r == 0
fn 02164508 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02012b28 force ret=pick:0/1/2 args=v  # GAMEDATA byte getter
  stub 02013c20 force ret=pick:0/1 args=v  # party predicate
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab4b8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 02164564(gd, x, p) -> bool: q = 02012AC8(p); 02013E04(q) == 0: 0; 02013C50(q) != 0
fn 02164564 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02013c50 force ret=pick:0/1 args=v  # party predicate
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate

# 0216458C(gd, x, p) -> 02013E04(02012AC8(p))
fn 0216458c ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate

# 0216459C(gd, x, p) -> 02013E04(02012AC8(p)) == 0 ? 0 : 021C5C64(gd, x, 0)
fn 0216459c ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate
  stub 021c5c64 force ret=pick:0/1 args=v,v,v  # (a, b, 0)

# 021645C0(gd, x, p) -> q = 02012AC8(p); 02013D20(q) != 0 ? 0 : 02013E04(q)
fn 021645c0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012ac8 force ret=u32 args=v  # GAMEDATA -> party
  stub 02013d20 force ret=pick:0/1 args=v  # party predicate
  stub 02013e04 force ret=pick:0/1 args=v  # party predicate

# 021645E0(ev, seq, work) -> done (GMEVENT proc 0x021645E1): f = 0201210C(work[0]); 02188C7C(f); seq 0: 02012138(work[0]);
# work[1] += 1; 0215870C(..) == 1 and work[1] < 30: 0; else 02188BF8(f), seq 1 (falls through); seq 1: 021A2F54(02188CB4(f)) == 1:
# seq 2; 0; seq 2: 021A34F8(02188CB4(f)) == 1; else 0
fn 021645e0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 4 u32 alt range 25 31 || any
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012138 force ret=u32 args=v  # GAMESYS getter
  stub 0215870c force ret=pick:0/1 args=v  # script_sys predicate
  stub 02188bf8 force ret=none args=v  # fieldmap (fieldmap)
  stub 02188c7c force ret=u32 args=v  # fieldmap getter
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C
  stub 021a2f54 force ret=pick:0/1/2 args=v  # (object) step
  stub 021a34f8 force ret=pick:0/1/2 args=v  # (object) step

# 02164654(-, gsys) -> GMEVENT: ev = 020122C0(gsys, 0, 0x021645E1, 8); work = 020124E8(ev): work[0] = gsys, work[1] = 0; ev.
# Arena: argument 2 (unused register)
fn 02164654 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)

# 02164678(-, gsys) -> 02185140(gsys, 0x117, &(0x200000, 0x20, 0x1E8000), 0) (ov20: map change event)
fn 02164678 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 02185140 force ret=u32 args=v,v,in:12,v  # event_mapchange (ov20): event (gsys, zone 0x117, &pos, 0)

# 021646A0(ev, seq, work) -> 1 (GMEVENT proc 0x021646A1): o = 02188CCC(work[1]); 021A0F30(o, 021A0F90(o) == 6 ? 0 : 6)
fn 021646a0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 02188ccc force ret=u32 args=v  # fieldmap getter
  stub 021a0f30 force ret=none args=v,v  # (object, mode 0 / 6)
  stub 021a0f90 force ret=pick:0/6/7 args=v  # (object) -> mode

# 021646C4(a, gsys) -> GMEVENT: ev = 020122C0(gsys, 0, 0x021646A1, 8); work = 020124E8(ev): work[0] = gsys, work[1] = a; ev.
# Arena: argument 2 (unused register)
fn 021646c4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)

# 021646E8(id) -> 8000 <= id <= 0x225F ? id - 8000 (u16) : 0 (hidden-item index of a script id)
fn 021646e8 ret=u32 inputs=100
  nowork
  arg u32 alt range 7990 8200 || range 0x2250 0x2270 || any

# 02164708(id) -> (u16) (ROM 0x02171534[021646E8(id)] (10-byte records, u16 +0) + 900) (the hidden-item flag)
fn 02164708 ret=u32 inputs=100
  nowork
  arg u32 alt range 7990 8200 || range 0x2250 0x2270 || any
  stub 021646e8 ret=pick:0/1/2/0xB0 args=v  # in-area

# 02164728(ew): 0201440C(ew) != 0 (once per day marker): for each of the 0xB1 hidden items (ROM 0x02171534, 10 bytes):
# byte +3 != 0 and 020056EC(100) < 20: clear_flag(ew, u16 +0 + 900) (respawn)
fn 02164728 ret=none inputs=60
  nowork
  arg u32 any
  stub 020056ec force ret=lt:0 args=v  # random below n (100)
  stub 0201433c force ret=none args=v,v  # event_work clear_flag (ew, flag)
  stub 0201440c force ret=pick:0/1 args=v  # event_work: read and clear the marker byte +0x3E8 (ew)

