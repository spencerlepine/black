# fndiff spec: the head of F1 row 10.23 (overlay 10 Thumb code 0x02161D98..0x0216237C; F6a-spec): Black City / White Forest people data
# (callers in overlay 20 field_wfbc_data_local.c, F1 scope X1: needs the scope decision) and a 0x48-byte COMM record whose updates are
# skipped while 0200FB24 (communication) is active. Logic only; specced because it is in the F6a row.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field wfbc_data --per-function
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
# LIST (0x1E8 bytes; 02161D98 init: +0 = 0, +2 = 2): +0 u16, +2 u16, REC[10] at +8 (list A) and REC[10] at +0xF8 (list B).
# REC (0x18): +0 u8 bits 0..3 kind (0 = empty), bits 4..7 nonzero = the 8 u16 at +4 must hold the end code 0xFFFF (02045A98);
#   +1 u8 id (valid < 30), +2 u8 (valid <= 100), +3 u8 (7 after 02161F90), +4 u16[8] (a name), +0x14..+0x17.
#   02161DC4(list, id, which) finds a valid REC by id (which 0: A, 1: B, 2: both); 021620B0 compacts a list after a delete.
# COMM (0x48; 02161F54 / 02161F68 clear it): +0 u16 own id (02039338), +2 u16 flags (bit i < 3: slot A[i] valid, 8: +0x20 valid,
#   0x10: +0x28 valid, bit i + 5: slot B[i] valid), +4 u16, +8 slot A[3] (8 bytes: u32 id, u16 list id, u16 handler index into ROM
#   0x02172990 = 02162198 / 021621C0 / 021621E8), +0x20 (8), +0x28 (8: u32 = +4, u16, u16), +0x30 slot B[3] (u32 i, u16, u16).
#
# Arena (the `work` image): LIST work+0, COMM work+0x200, scratch records work+0x300 / 0x310 / 0x320, the 0218682C (WFBC data) answer
# work+0x380. 02161F90 needs a valid REC of each id 0..3 in list B (it writes through the 02161DC4 result without a null check).
area wfbc_data
set field

# 02161D98(list): +0 = 0, +2 = 2 (u16); clears the 20 records (02161E78 on +8 + 0x18 i and +0xF8 + 0x18 i, i < 10)
fn 02161d98 ret=none inputs=100
  arg work 0
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02161e78 ret=none args=v  # in-area

# 02161DC4(list, id, which) -> record: i = 0..9: which 0 / 2: list A (+8) record i valid (02161ECC) with id (+1) == id;
# which 1 / 2 (which - 1 < 2 unsigned): list B (+0xF8) record i likewise; returns the record or 0
fn 02161dc4 ret=ofs:work inputs=100
  arg work 0
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 3 || any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area

# 02161E4C(list, id, which): 02161DC4(list, id, which) != 0
fn 02161e4c ret=u32 inputs=100
  arg work 0
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 3 || any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area

# 02161E60(list, which): which == 0: 021620B0(list + 8) else 021620B0(list + 0xF8) (compacts the list)
fn 02161e60 ret=none inputs=100
  arg work 0
  arg u32 alt oneof 0 1 || any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area
  stub 021620b0 ret=none args=v  # in-area

# 02161E78(rec): MI_CpuFill8(rec, 0, 0x18) (tail call)
fn 02161e78 ret=none inputs=100
  arg work 0x20
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02161E84(rec) -> bool: e = 02045A98() (EOS); rec +1 (id) >= 30: 0; rec +2 > 100: 0; rec +0 bits 4..7 != 0: one of the 8 u16 at
# +4 must be e, else 0; 1
fn 02161e84 ret=u32 inputs=100
  arg work 0x20
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)

# 02161ECC(rec) -> bool: rec +0 bits 0..3 == 0: 0; else 02161E84(rec)
fn 02161ecc ret=u32 inputs=100
  arg work 0x20
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161e84 ret=pick:0/1 args=v  # in-area

# 02161EEC(p): 10 u16 = 0xFFFF
fn 02161eec ret=none inputs=100
  arg work 0x300
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02161F00(p): MI_CpuFill8(p, 0, 4); +2 = 10 (u16)
fn 02161f00 ret=none inputs=100
  arg work 0x300
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02161F14(out, r, v): r +2 (u16) == 1: h = 021867C4(r +9 (u8), (v & 0x7FFF) | 0x8000); d = 0218682C(h); out[0] = d +0x24 (u16),
# out[1] = d +8; 02186804(h); else out = (0, 10) (u16 pair)
fn 02161f14 ret=none inputs=100
  arg work 0x310
  arg work 0x320
  arg u32 any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  fix work 0x322 u16 alt const 1 || any
  stub 021867c4 force ret=u32 args=v,v  # field_wfbc_data_local (X1): open (id, x)
  stub 02186804 force ret=none args=v  # field_wfbc_data_local (X1): close (handle)
  stub 0218682c force ret=ptr:work:0x380:1 args=v  # field_wfbc_data_local (X1): data of a handle

# 02161F54(comm): MI_CpuFill8(comm, 0, 0x48); +0 = 02039338() (u16)
fn 02161f54 ret=none inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02039338 force ret=u32 args=  # own id (stored as u16)

# 02161F68(comm): MI_CpuFill8(comm, 0, 0x48) (tail call)
fn 02161f68 ret=none inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02161F74(comm, list, mask): 0216210C() == 0: 02162120(comm, list, mask)
fn 02161f74 ret=none inputs=100
  arg work 0x200
  arg work 0
  arg u32 alt range 0 7 || any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 0200fb24 force ret=pick:0/1 args=  # communication active?
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e60 ret=none args=v,v  # in-area
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area
  stub 021620b0 ret=none args=v  # in-area
  stub 0216210c ret=pick:0/1 args=  # in-area
  stub 02162120 ret=none args=v,v,v  # in-area
  stub 02162198 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 021621c0 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 021621e8 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 02162254 ret=pick:0/1 args=v,v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/1/0  # in-area
  stub 02162278 ret=none args=v,v,v,v  # in-area
  stub 0216230c ret=none args=v,v  # in-area

# 02161F90(-, r, list, x, id, y) (id, y: stack u16): rec = 02161DC4(list, id, 1) (never null here: the spec plants a valid record
# of each id 0..3 in list B); h = 021867C4(id, y); d = 0218682C(h); rec +2 = r +2 == 0 ? d +6 : d +4 (byte); rec +3 = 7; 02186804(h);
# 02186520(r, x, rec); 02161E78(rec); 02161E60(list, 1)
fn 02161f90 ret=none inputs=100
  arg u32 any
  arg work 0x320
  arg work 0
  arg u32 any
  arg u16 range 0 3
  arg u16 any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  fix work 0x322 u16 alt const 0 || any
  fix work 0xF8 u8 range 1 15 rep=4,0x18
  fix work 0xF9 u8 const 0
  fix work 0x111 u8 const 1
  fix work 0x129 u8 const 2
  fix work 0x141 u8 const 3
  fix work 0xFA u8 range 0 100 rep=4,0x18
  stub 02161dc4 ret=ptr:work:0xF8:1 args=v,v,v  # in-area (never null here)
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161e60 ret=none args=v,v  # in-area
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area
  stub 021620b0 ret=none args=v  # in-area
  stub 02186520 force ret=none args=v,v,in:0x18  # field_wfbc_data_local (X1): (p, x, record)
  stub 021867c4 force ret=u32 args=v,v  # field_wfbc_data_local (X1): open (id, x)
  stub 02186804 force ret=none args=v  # field_wfbc_data_local (X1): close (handle)
  stub 0218682c force ret=ptr:work:0x380:1 args=v  # field_wfbc_data_local (X1): data of a handle

# 02161FEC(comm, v): +4 = v (u16)
fn 02161fec ret=none inputs=100
  arg work 0x200
  arg u32 any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02161FF0(comm, rec): 0216210C() == 0: 0216220C(comm, rec)
fn 02161ff0 ret=none inputs=100
  arg work 0x200
  arg work 0x300
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 0200fb24 force ret=pick:0/1 args=  # communication active?
  stub 0216210c ret=pick:0/1 args=  # in-area
  stub 0216220c ret=none args=v,v  # in-area

# 02162008(comm, i, out): 0216210C() == 0: 02162230(comm, i, out)
fn 02162008 ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 0200fb24 force ret=pick:0/1 args=  # communication active?
  stub 0216210c ret=pick:0/1 args=  # in-area
  stub 02162230 ret=none args=v,v,v  # in-area

# 0216202C(comm, out, b): *b = comm +4 (byte); tail 021622D8(comm, out)
fn 0216202c ret=u32 inputs=100
  arg work 0x200
  arg work 0x310
  arg work 0x320
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 021622d8 ret=pick:0/1 args=v,v  # in-area

# 02162048(comm, v, i): 0216210C() == 0: 02162298(comm, v, i); else t = ROM 0x0217144C[i] (u32: 1, 3, 4); t != 4: local
# {u32 comm +0 (u16), u16 v, u16 t}; 0216220C(comm, &local)
fn 02162048 ret=none inputs=100
  arg work 0x200
  arg u32 any
  arg u32 alt range 0 2 || range 0 3
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 0216220c ret=none args=v,in:8  # in-area (stack record)
  stub 0200fb24 force ret=pick:0/1 args=  # communication active?
  stub 0216210c ret=pick:0/1 args=  # in-area
  stub 02162298 ret=none args=v,v,v  # in-area

# 0216208C(comm) -> (+2 & 8) != 0
fn 0216208c ret=u32 inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 0216209C(comm) -> u16 +0x26
fn 0216209c ret=u32 inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 021620A0(comm): 02162358(comm); 021622F4(comm)
fn 021620a0 ret=none inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 021622f4 ret=none args=v  # in-area
  stub 02162358 ret=none args=v  # in-area

# 021620B0(recs): for i < 10: rec i invalid (02161ECC == 0): first valid j > i: MI_CpuCopy8(rec j, rec i, 0x18);
# 02161E78(rec j) (compaction)
fn 021620b0 ret=none inputs=100
  arg work 0x8
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area

# 0216210C() -> 0200FB24() != 0
fn 0216210c ret=u32 inputs=100
  nowork
  stub 0200fb24 force ret=pick:0/1 args=  # communication active?

# 02162120(comm, list, mask): for i < 3: mask bit i clear: 0216230C(comm, i); else 02162254(comm, i, &rec) (8 bytes):
# r = ROM 0x02172990[rec +6](comm, list, &rec, &out) (02162198 / 021621C0 / 021621E8); 0216230C(comm, i); r != 0:
# 02162278(comm, i, out +4, out +6)
fn 02162120 ret=none inputs=100
  arg work 0x200
  arg work 0
  arg u32 alt range 0 7 || any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e60 ret=none args=v,v  # in-area
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area
  stub 021620b0 ret=none args=v  # in-area
  stub 02162198 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 021621c0 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 021621e8 ret=pick:0/1 args=v,v,in:8,out:8  # in-area
  stub 02162254 ret=pick:0/1 args=v,v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/1/0  # in-area
  stub 02162278 ret=none args=v,v,v,v  # in-area
  stub 0216230c ret=none args=v,v  # in-area

# 02162198(comm, list, in, out) -> 1: out = in (u32 +0, u16 +4); out +6 = 02161DC4(list, in +4, 1) != 0 ? 0 : 1
fn 02162198 ret=u32 inputs=100
  arg work 0x200
  arg work 0
  arg work 0x300
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area

# 021621C0(comm, list, in, out) -> 1: as 02162198 with out +6 = found ? 2 : 3
fn 021621c0 ret=u32 inputs=100
  arg work 0x200
  arg work 0
  arg work 0x300
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area

# 021621E8(comm, list, in, out) -> 0: rec = 02161DC4(list, in +4, 1); rec: 02161E78(rec); 02161E60(list, 1)
fn 021621e8 ret=u32 inputs=100
  arg work 0x200
  arg work 0
  arg work 0x300
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any
  stub 02045a98 force ret=pick:0xFFFF args=  # the end-of-string code (u16 0xFFFF at 0x020A7E7C)
  stub 02161dc4 ret=nullable:3:ptr:work:0xF8:1 args=v,v,v  # in-area
  stub 02161e60 ret=none args=v,v  # in-area
  stub 02161e78 ret=none args=v  # in-area
  stub 02161e84 ret=pick:0/1 args=v  # in-area
  stub 02161ecc ret=pick:0/1 args=v  # in-area
  stub 021620b0 ret=none args=v  # in-area

# 0216220C(comm, rec): rec +0 (u32) == comm +0 (u16): MI_CpuCopy8(rec, comm + 0x20, 8); +2 |= 8
fn 0216220c ret=none inputs=100
  arg work 0x200
  arg work 0x300
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162230(comm, i, src): MI_CpuCopy8(src, comm + 8 + 8 i, 8); +2 |= 1 << i
fn 02162230 ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  arg work 0x300
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162254(comm, i, dst) -> bool: +2 & (1 << i): MI_CpuCopy8(comm + 8 + 8 i, dst, 8), 1; else 0
fn 02162254 ret=u32 inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162278(comm, i, a, b): comm + 8 i: +0x30 = i (u32), +0x34 = a, +0x36 = b (u16); +2 |= 1 << (i + 5)
fn 02162278 ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  arg u32 any
  arg u32 any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162298(comm, a, b): +0x2E = b, +0x2C = a (u16), +0x28 = +4 (u16 -> u32); +2 |= 0x10
fn 02162298 ret=none inputs=100
  arg work 0x200
  arg u32 any
  arg u32 any
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 021622AC(comm, i, dst) -> bool: +2 & (1 << (i + 5)): MI_CpuCopy8(comm + 0x30 + 8 i, dst, 8), 1; else 0
fn 021622ac ret=u32 inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 021622D8(comm, dst) -> bool: +2 & 0x10: MI_CpuCopy8(comm + 0x28, dst, 8), 1; else 0
fn 021622d8 ret=u32 inputs=100
  arg work 0x200
  arg work 0x310
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 021622F4(comm): MI_CpuFill8(comm + 0x20, 0, 8); +2 &= ~8
fn 021622f4 ret=none inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 0216230C(comm, i): MI_CpuFill8(comm + 8 + 8 i, 0, 8); +2 &= ~(1 << i)
fn 0216230c ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162330(comm, i): MI_CpuFill8(comm + 0x30 + 8 i, 0, 8); +2 &= ~(1 << (i + 5))
fn 02162330 ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || range 0 15
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162358(comm): MI_CpuFill8(comm + 0x28, 0, 8); +2 &= ~0x10
fn 02162358 ret=none inputs=100
  arg work 0x200
  fix work 0x8 u8 alt range 0 3 || range 0x10 0x13 || any rep=20,0x18
  fix work 0x9 u8 alt range 0 3 || any rep=20,0x18
  fix work 0xA u8 alt range 0 100 || any rep=20,0x18
  fix work 0x14 u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x1A u16 alt const 0xFFFF || any rep=20,0x18
  fix work 0x200 u16 alt range 0 3 || any
  fix work 0x20E u16 range 0 2 rep=3,8
  fix work 0x300 u32 alt range 0 3 || any
  fix work 0x304 u16 alt range 0 3 || any

# 02162370(v) -> v >= 7 (signed)
fn 02162370 ret=u32 inputs=100
  nowork
  arg s32 alt range 0 10 || any

