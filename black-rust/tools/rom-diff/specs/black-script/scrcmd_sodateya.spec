# fndiff spec: overlay-10 script commands of scrcmd_sodateya.c (F8-spec3), Thumb code 0x0215CE98..0x0215D34C: F1-SPLIT row 10.16
# (0x0215CF00..) plus the tail of row 10.15 (commands 235..237 at 0x0215CE98..0x0215CF00).  The function at 0x0215D34C (cmd 359)
# allocates with __FILE__ "scrcmd_musical.c", so scrcmd_musical.c starts there (not at F1's 0x0215D3EE); it is in
# scrcmd_misc10.spec.  Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_sodateya
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  235 0215ce98  236 0215cebc  237 0215cee8  238 0215cfb4  239 0215cfd8  240 0215cf54  241 0215cf84  242 0215cffc
#  243 0215d034  244 0215d0a4  245 0215d0d4  246 0215d110  247 0215cf00  248 0215d06c  593 0215d14c
# 18 functions specced (15 command ids).
#  * The daycare object is 02188D50(field map) (overlay-21 sodateya.c, F13); slot getters 021C34F8(daycare, slot) -> PokeParam,
#    021C3400 / 340C / 34C4 (counts / state), 021C341C / 3490 / 34B4 (per-slot values), 021C3204 / 3270 / 32F4 / 335C (deposit /
#    withdraw / egg actions with the party).  PokeParam ids: 5 species, 0x6E gender (0 M, 1 F, 2 genderless), 0x6F form, 0x75
#    (nickname flag?, UNVERIFIED).  Names of the overlay-21 calls are inferred from the command order (UNVERIFIED; F13).
#  * 247 runs the party-select event 0x0215D2A4 / GMEVENT 0x0215D1E4 (seq 0..5: field fade 021BE8D4, app event 020144DC,
#    02014910, 020145CC, return 021BE92C; result A+0x3C -> var).
#  * 593: the gender mark of a daycare mon's name (0x0215D198: 0 none, 1 male, 2 female; Nidoran without nickname: none).
# Not specced: 0215ceb2 (a false `bl` target decoded from data inside 0215ce98); LAB_ 0215d190 / d222 are inside 0215d14c / d1e4.
#
# Read from the disassembly of the raw OVY_10.sbin (rizin `-a arm -b 16 -m 0x02155160`; rizin on field.elf mis-disassembles
# mixed ARM / Thumb and Ghidra's field set drops the arguments of overlay calls).  Function list: command-table targets + init-
# proc table entries (0x021705F0) + `bl` targets from the whole field set + odd code pointers + labels, walked as Thumb from every
# entry (only 2-byte padding and literal pools stay uncovered).
#
# Spec conventions (as scrcmd_core.spec / scrcmd_work.spec): every block is `nowork`.  A command handler `fn(vm, ctx)` gets two
# arenas: arg0 = the VM (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, PC +0x14, compare byte +0xE in 0..2), arg1 = the
# scrcmd ctx (`arg buf 0x800`: CTX at 0, SCRCMD WORK at 0x100, zone A 0x300 (field block, SW records), balloon work 0x1D4, zone B
# 0x400, vars 0x600..0x7FF where `02159ae8` points).  A helper taking the ctx in r0 gets the ctx arena as arg0.  A GMEVENT function
# `(event, seq, work)` gets `arg u32, arg buf 0x10 (seq, bounded), arg buf 0x40 (work)`; an init proc `(task, task + 0xC)` gets
# `arg buf 0x100, arg buf 0x10`.  Script-operand readers (vm.c 02011330 / 0201134c, script_sys 02159ae8 / 02159b10) are `seq`
# stubs.  Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee).
# **Every stub outside the area carries `force`** (overlay addresses collide with battle `PORTED` entries and the callees in
# script_sys / scrcmd_work are ported; the F8 areas call each other only through forced stubs, so each is verified on its own).
# Indirect calls (`blx rN` through a fn argument) go to the fake callback 0x02EE0011 (stub 02ee0010).  UI callees are `ScriptHost`
# requests in the port (crates/black-script/src/host.rs); results that steer a branch are `pick:` stubs.
# F8-spec3 additions: Heap_AllocDebug (02030734) is a `seqptr` bump allocator into the ctx arena (blocks of a block never
# overlap); its 4th argument is the __FILE__ string (0x021728F0 "scrcmd_proc.c", 0x02172900 "scrcmd_sodateya.c", 0x02172914
# "scrcmd_musical.c"), which places the file boundaries.  Callees of other overlay-10 files, ARM9 main and overlays 11 / 21 / 37
# are force stubs; overlay 37 (0x021F3xxx, outside the field set) is loaded by 02034AC4(0x25) around the call.  An unset callee-
# saved register or stack word that a path reads is 0 under the Z19 entry state; the header of the block says so.
area scrcmd_sodateya
set field

# cmd 235: var = 021C3400(daycare); 0.
fn 0215ce98 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c3400 force ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02188d50 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 236: 021C32F4(daycare, party); 0.
fn 0215cebc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02188d50 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 021c32f4 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 237: 021C335C(daycare); 0.
fn 0215cee8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188d50 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 021c335c force ret=none args=v

# cmd 247: var; 02012934(GAMEDATA) (unused); 0200BDB0(SAVE); event 0215d2a4(GAMESYS, field map, var); call it; 1.
fn 0215cf00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012ebc force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 0215d2a4 ret=u32 args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 0200bdb0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0215d2a4
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x400:1 args=v
  stub 0200ca38 force ret=u16 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0xA0 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c force ret=u32 args=v

# cmd 240: v = value; 021C3204(daycare, party, v); 0.
fn 0215cf54 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 021c3204 force ret=none args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 241: v = value; 021C3270(daycare, v, party); 0.
fn 0215cf84 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c3270 force ret=none args=v,v,v
  stub 02012934 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 238: var = 021C340C(daycare); 0.
fn 0215cfb4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 021c340c force ret=u16 args=v
  stub 02188d50 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 239: var = 021C34C4(daycare); 0.
fn 0215cfd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c34c4 force ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02188d50 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 242: var, slot; var = PokeParam get(021C34F8(daycare, slot), 5 (species), 0); 0.
fn 0215cffc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 243: var, slot; var = PokeParam get(021C34F8(daycare, slot), 0x6F, 0); 0.
fn 0215d034 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 248: var, slot; var = PokeParam get(021C34F8(daycare, slot), 0x6E (gender), 0); 0.
fn 0215d06c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 244: var, slot; var = 021C341C(daycare, slot); 0.
fn 0215d0a4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 021c341c force ret=u16 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 245: var, slot; 021C34F8(daycare, slot) (unused); var = 021C3490(daycare, slot); 0.
fn 0215d0d4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c3490 force ret=u16 args=v,v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 246: var, slot; 021C34F8(daycare, slot) (unused); var = 021C34B4(daycare, slot); 0.
fn 0215d110 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 021c34b4 force ret=u16 args=v,v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 593: var, m, slot; mon = 021C34F8(daycare, slot); var = m == 0 ? 0215d198(mon) : 0; 0.
fn 0215d14c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215d198 ret=pick:0/1/2 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02188d50 force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2 args=v,v seq
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0215d198
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# 0215d198(mon): the gender mark of a name: g = PokeParam get(mon, 0x6E); species 0x1D / 0x20 (Nidoran F / M) with get(mon,
# 0x75) == 0 -> 0; else g 0 -> 1, 1 -> 2, other 0.
fn 0215d198 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# GMEVENT fn of cmd 247 (event, seq, W = {GAMESYS, field map, A (0x98), B (0x28), var}): seq 0: call 021BE8D4(GAMESYS,
# field map, 0, 0), seq 1; 1: call 020144DC(GAMESYS, field map), A+0x3C = B+0xF byte, seq 2; 2: call 02014910(GAMESYS, field map, A,
# B), seq 3; 3: call 020145CC(GAMESYS), seq 4; 4: call 021BE92C(GAMESYS, field map, 0, 0, 1, 0, 0), seq 5; 5: *var = A+0x3C,
# Heap_Free(A), Heap_Free(B), done (1); other seq: 0.
fn 0215d1e4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 6 || any
  fix arg2 8 u32 addrof arg2 0x40
  fix arg2 0xC u32 addrof arg2 0xC0
  fix arg2 0x10 u32 addrof arg2 0xF0
  stub 020145cc force ret=u32 args=v
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v
  stub 020307b0 force ret=none args=v
  stub 02014910 force ret=u32 args=v,v,v,v
  stub 02012374 force ret=none args=v,v
  stub 021be8d4 force ret=u32 args=v,v,v,v
  stub 020144dc force ret=u32 args=v,v

# 0215d2a4(GAMESYS, field map, var) -> GMEVENT: A = Heap_AllocDebug(4, 0x98, 0, "scrcmd_sodateya.c", 0x22B) = {party, ..,
# +0x34 = 0x12, +0x38 = 0, +0x3C = 0}; B = alloc(0x28) = {party, A+0xC, GAMEDATA, b0xC 1, b0xD 0, b0xE = 0201AA18(party),
# b0xF 0, b0x10 0, +0x18 = 0200CA38(0200CA1C(GAMEDATA))}; event = 020122C0(GAMESYS, 0, 0x0215D1E5, 0x14); W = {GAMESYS,
# field map, A, B, var}.
fn 0215d2a4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x400
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0x200:1 args=v
  stub 0200ca38 force ret=u16 args=v
  stub 02030734 force ret=seqptr:arg3:0:0xA0 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c force ret=u32 args=v
