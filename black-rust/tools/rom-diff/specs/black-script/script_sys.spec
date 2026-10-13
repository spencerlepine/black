# fndiff spec: script_sys.c + script_work.c (F7-spec), overlay 10 Thumb code 0x02158804..0x021591FE and 0x02159960..0x02159B48,
# plus the scrcmd context getters 0x0215A5CC..0x0215A5FA (they sit in the scrcmd_work.c area of F8 but every command calls them).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script script_sys --per-function
# Layouts were read from the disassembly (Ghidra's field set has no overlay prototypes: it drops arguments and calls
# Heap_AllocDebug 02030734 / Heap_Free 020307B0 "noreturn"; ignore that).
#
# Objects (all sizes from the allocation calls):
#  SYS   "script_sys.c" (02158804, 0x18 bytes, Heap_AllocDebug(heap, 0x18, 1, "script_sys.c", 0xEF)): the script system of one
#        script event.  +0 u16 heap, +2 u8 number of live VMs, +4/+8/+0xC VM pointers (3 slots, 0 = free), +0x10 end callback
#        (fn(arg1, arg)), +0x14 its argument.  021588a4 installs the callback 0x0215889D (a veneer to main 0x0201235C).
#  EVENT WORK of the script event (8 bytes, the data block of the GMEVENT 020122C0 creates in 02158920): +0 SW, +4 SYS.
#        The event function is 021588dc(event, seq, work).
#  SW    SCRIPT_WORK "script_work.c" (02158e7c, 0x14C bytes, line 0xAF): +0 u32 magic 0x0003643F, +4 u16 script id,
#        +8 u32 MMDL (talk target; 02158f94 also stores its OBJ id into var 0x8011), +0xC u16 heap, +0x10 GAMESYS, +0x14 GMEVENT,
#        +0x18 8 bytes {+0 = 02188C54(fieldmap), +4 = fieldmap = 0201210C(gamesys)} (02158e54/02158f58), +0x20 u32 (creator arg),
#        +0x24 u32 "no message resources" flag (= arg != 0), +0x28 u32 that arg, +0x2C WORDSET (0201EC74(0x1C, 0x40, heap)),
#        +0x30/+0x34 two STRBUFs (020457B0(0x500, heap)) (all three 0 when +0x24 != 0), +0x38 u32 (02158f80/84), +0x3C heap block
#        pointer (02158fb4/b8, freed by 02158fbc), +0x40 address getter (02158f88), +0x44 + i*0x1C small records (02159124),
#        +0x7C.. script vars 0x8000.. as u16 ((id - 0x8000) * 2 + 0x7C, 02158fd0; 0x60 vars to +0x13C), +0x13C/+0x140/+0x144 u32
#        get/set pairs, +0x148 -> SCRCMD WORK (0215a514, 0x110 bytes "scrcmd_work.c": +0 SW, +4 GAMESYS, +8 GAMEDATA,
#        +0xC = 02012964(gamedata), +0x104 u8 = 7).
#  VM    "vm.c" (main 0x0201121C(heap, &init {u16 stack_words, u16 reg_words, u32 cmd_table, u32 cmd_count}),
#        0x30 + 4 * (regs + stack) bytes; script_sys uses stack 0x100, regs 0x40 -> 0x530 bytes):
#        +0 u16 stack words, +2 u16 reg words, +4 command table (0x0217062C), +8 command count (*0x021705D8 = 0x261 = 609),
#        +0xC u8 call-stack depth, +0xD u8 status (0 stopped, 1 running, 2 waiting on +0x10), +0xE u8, +0x10 wait callback
#        fn(vm, ctx) (020113D0 sets it and status 2), +0x14 PC (byte pointer into the script), +0x18 call stack (u32[stack]),
#        +0x1C regs (u32[regs], at +0x30), +0x20 ctx = SCRCMD CTX, +0x24 pre-dispatch hook fn(vm, ctx, arg, cmd) (0 = none),
#        +0x28 its argument, +0x2C script file data (ARC 57 file, 02158d78; freed by 02158cb4).
#  CTX   SCRCMD CTX (0215a55c(&init, heap), 0x24 bytes "scrcmd_work.c" line 0x94, allocated with heap id | 0x8000 (top-down)):
#        +0 u16 heap, +4..+0x13 a copy of the 16-byte init record built by 02158bfc {+4 u16 zone id, +6 u16 script id,
#        +8 u32 mode (the creator arg, 0215a5d8; 0/1/2 select the command permission bit, see 02158de8), +0xC u32 (mode != 0)
#        (0215a5d4), +0x10 SW}, +0x14 STRBUF?/+0x1C heap block (freed by 0215a5a4), +0x20 -> SCRCMD WORK (= SW+0x148).
#        Getters: 0215a5cc u16 +0, 0215a5d0 u16 +4, 0215a5d4 +0xC, 0215a5d8 +8, 0215a5f4 work[0] = SW, 0215a5dc work[1] =
#        GAMESYS, 0215a5e4 work[2] = GAMEDATA, 0215a5ec work[3].
#
# Command dispatch (main vm.c 0x02011298, run once per frame per VM by 0215882c): while status is running and PC != 0:
# cmd = u16 at PC (02011330, PC += 2); cmd >= +8 (609) stops the VM; the hook +0x24 (02158de8 here) runs first and may stop the
# VM (it returns 0); then table[cmd](vm, ctx): 1 = yield (VM keeps running next frame), 0 = next command.  The table is at
# **0x0217062C** (609 entries, index = command id), not at 0x021705F0: F1's 624-slot run starts 15 words early, at a separate
# null-terminated 14-entry proc table 0x021705F0 (021591b4, gated by the 14-bit mask at 0x02172BA0: 02159144/68/98), so F1's
# "slot N" is command N - 15 (slot 15 = command 0 = 02159240).  Command ids are not dispatched anywhere else.  The 609-byte
# table 0x02170374 (the "undecoded byte table at 0x021705A0" of F1 is its tail) holds per command a permission mask: bit k is
# required when ctx mode (+8) is k (0..2); a missing bit (or mode >= 3) calls the fatal 02158dbc (endless loop).
# Script file: header = u32 relative offsets per script (02158bfc: PC = data + idx * 4, then PC += u32 at PC); script ids
# map to (ARC 57 file, type, msg) by the 46-entry table 0x021701A8 {u16 lo, hi, file, type, msg} (02158cd0, ids 2000..10524);
# ids 1..1999 are zone-local (file 02013B5C(zone), msg 02013B68(zone), index id - 1); id 0 = file 0x356, msg 0x9E.
#
# Not specced (veneers / fragments, the walk proves they are part of a specced function or a tail jump):
#  02158824 (`ldr r3,=Heap_Free; bx r3`), 0215889c (`ldr r3,=0x0201235D; bx r3`, the SYS end callback), 02159190
#  (`ldr r3,=02159199; bx r3`), fragments 021588f2 (021588dc), 02158e10/18/20/28 (02158de8), 02159970 (02159960),
#  02159994 (02159984), 021599f4 (021599a8), 02159aba (02159a64).  02159204..0215995E (commands 0..63, F8 core) are F8.
#
# Spec conventions (as core4 / B11): every block is `nowork`; the object a function works on is an `arg buf` arena with the
# objects it points to in zones of the same buffer; callees in this area are plain stubs that list the stubs of their own
# callees too (closure: with every function of the area treated as ported, all blocks run without a fault, a stray write or
# an unstubbed callee, so porting order needs no spec edit); main / other overlay callees are plain stubs (they never
# deactivate here).  Thunks run as real code and land on the stub of their target (Heap_Free 020307B0 via 02158824 /
# 0215a554 / 02011264).  MIi_CpuFill32 02082A7C and MIi_CpuCopy16 02082A60 run as real code (svflow ALLOW; ports inline them).
# Stub results that the function dereferences are `ptr:argK:OFS:N` into a zone; values compared against constants are `pick:`.
# Function pointers called through a field are faked at 0x02EE0011 (stub 02ee0010).
area script_sys
set field

# ======================================================================== script_sys.c: SYS (0x18 bytes)

# 02158804(heap): SYS = Heap_AllocDebug(heap, 0x18, 1, "script_sys.c", 0xEF); SYS->heap (u16 +0) = heap; return SYS.
# (Ghidra: false noreturn.)  arg1 = arena the allocation lands in (the function ignores r1).
fn 02158804 ret=ofs:arg1 inputs=100
  nowork
  arg u32 alt range 0 0x10 || any
  arg buf 0x40
  stub 02030734 ret=ptr:arg1:0:1 args=v,v,v,v,v

# 0215882c(sys): for each of the 3 VM slots: a VM whose step 02011298(vm) returns 0 (stopped) is freed (02158cb4), its slot
# cleared and the live count +2 decremented; returns 1 while the count is nonzero.
fn 0215882c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 4 u32 alt const 0 || addrof arg0 0x100
  fix arg0 8 u32 alt const 0 || addrof arg0 0x300
  fix arg0 0xC u32 alt const 0 || addrof arg0 0x500
  fix arg0 2 u8 alt range 0 3 || any
  stub 02011298 ret=pick:0/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02011324 ret=u32 args=v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v

# 0215886c(sys, vm): put vm into the first free slot (+4 + i*4), count +2 += 1; returns the slot or 3 when all are taken.
fn 0215886c ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 any
  fix arg0 4 u32 alt const 0 || any
  fix arg0 8 u32 alt const 0 || any
  fix arg0 0xC u32 alt const 0 || any

# 02158894(sys, i): VM of slot i (u32 at +4 + i*4).
fn 02158894 ret=u32 inputs=100
  nowork
  arg buf 0x40
  arg u32 range 0 7

# 021588a4(sys, arg): end callback +0x10 = 0x0215889D (veneer to 0x0201235C), +0x14 = arg.
fn 021588a4 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u32 any

# 021588b0(sys, x): returns (+0x10)(x, +0x14).
fn 021588b0 ret=u32 inputs=100
  nowork
  arg buf 0x20
  arg u32 any
  fix arg0 0x10 u32 const 0x02EE0011
  stub 02ee0010 ret=u32 args=v,v

# 021588c0(sys): 1 if an end callback is set (+0x10 != 0).
fn 021588c0 ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 0x10 u32 alt const 0 || any

# 021588d0(gamesys): zone id = 02012AC8(020120F8(gamesys)) (u16 at GAMEDATA+0x114).
fn 021588d0 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v

# 021588dc(event, seq, work): the script event's function.  work = {SW, SYS}.  While 0215882c(SYS) reports live VMs it
# returns 0 (continue); then frees SW (02158f18), runs the SYS end callback if one is set (021588b0(SYS, event)), frees SYS
# (Heap_Free via the veneer 02158824) and returns 1 when there was no callback, 0 after the callback (it changed the event).
# arg2 arena: work at 0, SYS at 0x100 (VMs at 0x200/0x300/0x400), SW at 0x600.
fn 021588dc ret=u32 inputs=300
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x800
  fix arg2 0 u32 addrof arg2 0x600
  fix arg2 4 u32 addrof arg2 0x100
  fix arg2 0x102 u8 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || addrof arg2 0x200
  fix arg2 0x108 u32 alt const 0 || addrof arg2 0x300
  fix arg2 0x10C u32 alt const 0 || addrof arg2 0x400
  fix arg2 0x110 u32 alt const 0 || const 0x02EE0011
  fix arg2 0x62C u32 alt const 0 || any
  fix arg2 0x630 u32 alt const 0 || any
  fix arg2 0x634 u32 alt const 0 || any
  stub 0215882c ret=pick:0/1 args=v
  stub 02158f18 ret=none args=v
  stub 021588c0 ret=pick:0/1 args=v
  stub 021588b0 ret=u32 args=v,v
  stub 02ee0010 ret=u32 args=v,v
  stub 02011298 ret=pick:0/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02011324 ret=u32 args=v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v

# 02158920(gamesys, -, script id, mmdl, s0 arg): start a script event: event = 020122C0(gamesys, 0, 021588dd, 8);
# work = 020124E8(event); work[1] = SYS (02158804(4)); work[0] = SW (02158e7c(4, gamesys, event, id, s0, 0));
# 02158f94(SW, mmdl); 021589a0(SW, zone 021588d0(gamesys), id); returns the event.
# arg0 arena (gamesys is only passed on): event work 0x100, SYS 0x200, SW / heap blocks 0x400.., VM 0xA00, ctx 0xC00 (its
# work block 0xD00).  The heap stub hands out blocks anywhere in 0x400..0x500 (two blocks of one run may overlap).
fn 02158920 ret=u32 inputs=300
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 alt range 0 3 || range 1 1999 || range 2000 10524 || any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 020122c0 ret=u32 args=v,v,v,v
  stub 020124e8 ret=ptr:arg0:0x100:1 args=v
  stub 02158804 ret=ptr:arg0:0x200:1 args=v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 02158f94 ret=none args=v,v
  stub 021588d0 ret=u32 args=v
  stub 021589a0 ret=u32 args=v,v,v
  # closure: 02158804 / 02158e7c / 02158f94 / 021588d0 / 021589a0
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 02158fd0 ret=ptr:arg0:0x49e:1 args=v,v
  stub 0216d588 ret=u16 args=v
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 021589dc ret=nullable:4:ptr:arg0:0x200:1 args=v
  stub 02158f50 ret=nullable:4:u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 0215886c ret=pick:0/1/2/3 args=v,v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  global 02172ba0 4

# 0215897c(sw, child): 02012374(SW->event (02158f50), child): call a child event from the script.
fn 0215897c ret=none inputs=100
  nowork
  arg buf 0x200
  arg u32 any
  stub 02158f50 ret=u32 args=v
  stub 02012374 ret=none args=v,v

# 0215898c(sw, arg): if SYS = 021589dc(sw) exists, set its end callback (021588a4(SYS, arg)).
# arg0 arena: SW at 0, event work at 0x400 (+4 -> SYS at 0x500).
fn 0215898c ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x14 u32 alt const 0 || any
  fix arg0 0x404 u32 addrof arg0 0x500
  stub 021589dc ret=nullable:4:ptr:arg0:0x500:1 args=v
  stub 021588a4 ret=none args=v,v
  stub 02158f50 ret=nullable:4:u32 args=v
  stub 020124e8 ret=ptr:arg0:0x400:1 args=v

# 021589a0(sw, zone, script id): SYS = 021589dc(sw); none -> 3; else vm = 02158bfc(SYS->heap, sw, zone, id, 0) and the VM
# goes into a free SYS slot (0215886c; returns the slot or 3).  arg0 arena: SW 0, event work 0x400 (+4 -> SYS 0x500),
# VM 0xA00, ctx 0xC00 (work block 0xD00).
fn 021589a0 ret=u32 inputs=300
  nowork
  arg buf 0x1000
  arg u32 alt range 0 0x30 || any
  arg u32 alt range 0 3 || range 1 1999 || range 2000 10524 || any
  fix arg0 0x14 u32 alt const 0 || any
  fix arg0 0x404 u32 addrof arg0 0x500
  fix arg0 0x504 u32 alt const 0 || any
  fix arg0 0x508 u32 alt const 0 || any
  fix arg0 0x50C u32 alt const 0 || any
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 021589dc ret=nullable:4:ptr:arg0:0x500:1 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 0215886c ret=pick:0/1/2/3 args=v,v
  stub 02158f50 ret=nullable:4:u32 args=v
  stub 020124e8 ret=ptr:arg0:0x400:1 args=v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v

# 021589d0(event): SW = word 0 of the event's work (020124E8(event)).  arg1 = the work block (r1 is not read).
fn 021589d0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  stub 020124e8 ret=ptr:arg1:0:1 args=v

# 021589dc(sw): SYS of the script = word 1 of the event work (020124E8(SW->event)); 0 when SW has no event (+0x14 == 0).
fn 021589dc ret=u32 inputs=200
  nowork
  arg buf 0x200
  fix arg0 0x14 u32 alt const 0 || any
  stub 02158f50 ret=nullable:4:u32 args=v
  stub 020124e8 ret=ptr:arg0:0x180:1 args=v

# 021589f4(event work): clear flags 0..99 (02014368(ew, 0, 99)) and vars 0x4000..0x401E (020143D8(ew, 0x4000, 0x401F)).
fn 021589f4 ret=none inputs=100
  nowork
  arg u32 any
  stub 02014368 ret=none args=v,v,v
  stub 020143d8 ret=none args=v,v,v

# ---------------------------------------------------------------- synchronous scripts

# 02158a34(gamesys, sw or 0, script id, mode): run a script to the end at once: without sw a temporary SW is made
# (02158e7c(4, gamesys, 0, id, 0, mode)); vm = 02158bfc(4, sw, zone 021588d0(gamesys), id, mode); step 02011298 until it
# returns != 1; free the VM (02158cb4) and the temporary SW (02158f18).
# arg0 arena: SW 0x400 (heap blocks 0x400..0x500), VM 0xA00, ctx 0xC00 (work block 0xD00).
fn 02158a34 ret=none inputs=300
  nowork
  arg buf 0x1000
  arg u32 alt const 0 || addrof arg0 0x400
  arg u32 alt range 0 3 || range 1 1999 || range 2000 10524 || any
  arg u32 alt range 0 3 || any
  fix arg0 0xC20 u32 addrof arg0 0xD00
  fix arg0 0x42C u32 alt const 0 || any
  fix arg0 0x430 u32 alt const 0 || any
  fix arg0 0x434 u32 alt const 0 || any
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  # closure
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

# 02158a10(gamesys): 02158a34(gamesys, 0, 9600, 2).  02158a20(gamesys): 02158a34(gamesys, 0, 9601, 2).
fn 02158a10 ret=none inputs=100
  nowork
  arg buf 0x1000
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

fn 02158a20 ret=none inputs=100
  nowork
  arg buf 0x1000
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

# 02158a90(gamesys, -, key, mode): look up the zone's special script `key` in the table 02162450(02012844(GAMEDATA))
# (02158b3c); none (0xFFFF) -> 0; else run it synchronously (02158a34(gamesys, 0, id, mode)) -> 1.
# arg0 arena: special-script records at 0x100 (6 bytes {u16 key, u16 id, u16}, key 0 ends), the 02158a34 zones as there.
fn 02158a90 ret=u32 inputs=300
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 range 0 5
  arg u32 alt range 0 3 || any
  fix arg0 0x100 u16 range 0 5 rep=12,6
  fix arg0 0x148 u16 const 0
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 020120f4 ret=u32 args=v
  stub 02012844 ret=u32 args=v
  stub 02162450 ret=ptr:arg0:0x100:1 args=v
  stub 02158b3c ret=pick:0xFFFF/0x2580/0x7D0/7 args=v,v
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

# 02158ac8(gamesys, x): 02158a90(gamesys, x, 4, 1).  02158ad4: key 3.  (Same closure as 02158a90.)
fn 02158ac8 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0x100 u16 range 0 5 rep=12,6
  fix arg0 0x148 u16 const 0
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 02158a90 ret=u32 args=v,v,v,v
  stub 020120f4 ret=u32 args=v
  stub 02012844 ret=u32 args=v
  stub 02162450 ret=ptr:arg0:0x100:1 args=v
  stub 02158b3c ret=pick:0xFFFF/0x2580/0x7D0/7 args=v,v
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

fn 02158ad4 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0x100 u16 range 0 5 rep=12,6
  fix arg0 0x148 u16 const 0
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 02158a90 ret=u32 args=v,v,v,v
  stub 020120f4 ret=u32 args=v
  stub 02012844 ret=u32 args=v
  stub 02162450 ret=ptr:arg0:0x100:1 args=v
  stub 02158b3c ret=pick:0xFFFF/0x2580/0x7D0/7 args=v,v
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

# 02158ae0(gamesys, x): clear the "first visit" flags/vars of the event work (021589f4(0201296C(GAMEDATA))), then
# 02158a90(gamesys, x, 2, 2).
fn 02158ae0 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0x100 u16 range 0 5 rep=12,6
  fix arg0 0x148 u16 const 0
  fix arg0 0xC20 u32 addrof arg0 0xD00
  stub 020120f4 ret=u32 args=v
  stub 0201296c ret=u32 args=v
  stub 021589f4 ret=none args=v
  stub 02014368 ret=none args=v,v,v
  stub 020143d8 ret=none args=v,v,v
  stub 02158a90 ret=u32 args=v,v,v,v
  stub 02012844 ret=u32 args=v
  stub 02162450 ret=ptr:arg0:0x100:1 args=v
  stub 02158b3c ret=pick:0xFFFF/0x2580/0x7D0/7 args=v,v
  stub 02158a34 ret=none args=v,v,v,v
  stub 02158e7c ret=ptr:arg0:0x400:1 args=v,v,v,v,v,v
  stub 021588d0 ret=u32 args=v
  stub 02158bfc ret=ptr:arg0:0xa00:1 args=v,v,v,v,v
  stub 02011298 seq ret=pick:0/1/1 args=v
  stub 02158cb4 ret=none args=v
  stub 02158f18 ret=none args=v
  stub 02030734 ret=ptr:arg0:0x400:0x40 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  stub 020120f8 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg0:0xa00:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg0:0xc00:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  global 02172ba0 4

# 02158b00(gamesys, x): conditional special script: id = 02158b6c(GAMEDATA, table 02162450(02012844(GAMEDATA)), 1);
# 0xFFFF -> 0; else 02158768(gamesys, id, 0, x) (delivery/IRC side, stub).  arg0 arena: records at 0x100 (see 02158b6c).
fn 02158b00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x100 u16 range 0 3 rep=4,6
  fix arg0 0x102 u32 alt const 0 || const 0xEA
  fix arg0 0x108 u32 alt const 0 || const 0xE4
  fix arg0 0x10E u32 alt const 0 || const 0xDE
  fix arg0 0x114 u32 alt const 0 || const 0xD8
  fix arg0 0x118 u16 const 0
  fix arg0 0x1F0 u16 range 0 3 rep=6,6
  fix arg0 0x1F2 u16 range 0 3 rep=6,6
  fix arg0 0x214 u16 const 0
  fix arg0 0x300 u16 range 0 3 rep=4,4
  stub 020120f4 ret=u32 args=v
  stub 02012844 ret=u32 args=v
  stub 02162450 ret=ptr:arg0:0x100:1 args=v
  stub 02158b6c ret=pick:0xFFFF/0x2580/0x7D0/7 args=v,v,v
  stub 02158768 ret=u32 args=v,v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg0:0x300:4 args=v,v

# 02158b3c(records, key): id of the first 6-byte record {u16 key, u16 id, u16} whose key matches; 0xFFFF at the key-0 end.
fn 02158b3c ret=u32 inputs=300
  nowork
  arg buf 0x80
  arg u32 alt range 0 5 || any
  fix arg0 0 u16 range 0 5 rep=12,6
  fix arg0 0x48 u16 const 0

# 02158b6c(gamedata, records, key): conditional script lookup.  Record {u16 key, u32 rel offset (bytes 2..5)}; the matching
# record's list (record + 6 + offset) holds 6-byte entries {u16 var, u16 value, u16 script id} (var 0 ends): the first entry
# whose event var (*020143CC(0201296C(gamedata), var)) equals value gives the id; 0xFFFF otherwise / offset 0 / no record.
# arg1 layout: 4 records at 0 (offsets point at 0x1F0), list at 0x1F0 (6 entries), var values at 0x300.
fn 02158b6c ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x400
  arg u32 alt range 0 3 || any
  fix arg1 0 u16 range 0 3 rep=4,6
  fix arg1 2 u32 alt const 0 || const 0x1EA
  fix arg1 8 u32 alt const 0 || const 0x1E4
  fix arg1 0xE u32 alt const 0 || const 0x1DE
  fix arg1 0x14 u32 alt const 0 || const 0x1D8
  fix arg1 0x18 u16 const 0
  fix arg1 0x1F0 u16 range 0 3 rep=6,6
  fix arg1 0x1F2 u16 range 0 3 rep=6,6
  fix arg1 0x214 u16 const 0
  fix arg1 0x300 u16 range 0 3 rep=4,4
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg1:0x300:4 args=v,v

# ======================================================================== script_work.c: VM / ctx setup

# 02158bfc(heap, sw, zone, script id, mode): make a VM for one script.  ctx = 0215a55c({u16 zone, u16 id, mode, mode != 0,
# sw}, heap); vm = 0201121C(heap, {0x100, 0x40, 0x0217062C, *0x021705D8}); 0201126C(vm, ctx); idx = 02158cd0(zone, id, &file,
# &type, &msg); vm+0x2C = 02158d78(file, heap) (script file data); with mode 0 the message bank is opened (0215a760(ctx, type,
# msg); the "msg != 0xFFFFFFFF" test compares a u16 and is always true); start the VM at the data (0201127C), then
# PC += idx * 4 and PC += u32 at PC (0201134C); install the dispatch hook (02158e38).  Returns vm.
# arg1 arena: SW 0, VM 0x400, ctx 0x800 (work block 0x900).
fn 02158bfc ret=ofs:arg1 inputs=300
  nowork
  arg u32 alt range 0 0x10 || any
  arg buf 0x1000
  arg u32 alt range 0 0x30 || any
  arg u32 alt range 0 3 || range 1 1999 || range 2000 10524 || any
  arg u32 alt const 0 || range 0 3 || any
  fix arg1 0x820 u32 addrof arg1 0x900
  stub 02158db0 ret=pick:0/1 args=v
  stub 0215a55c ret=u32 args=in:16,v
  stub 0201121c ret=ptr:arg1:0x400:1 args=v,in:12
  stub 0201126c ret=none args=v,v
  stub 02158cd0 ret=u16 args=v,v,out:2,out:2,out:2
  stub 02158d78 ret=u32 args=v,v
  stub 0204a1bc ret=u32 args=v,v,v,v
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0215a760 ret=none args=v,v,v
  stub 0201127c ret=none args=v,v
  stub 0201134c ret=mask:0x3FC args=v
  stub 02158e38 ret=none args=v
  stub 02011324 ret=ptr:arg1:0x800:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v

# 02158cb4(vm): free a script VM: 0215a5a4(ctx = 02011324(vm)), Heap_Free(vm+0x2C file data), Heap_Free(vm) (veneer 02011264).
fn 02158cb4 ret=none inputs=100
  nowork
  arg buf 0x40
  stub 02011324 ret=u32 args=v
  stub 0215a5a4 ret=none args=v
  stub 020307b0 ret=none args=v

# 02158cd0(zone, id, *file, *type, *msg): see the header (table 0x021701A8; an id inside an entry's [lo, ...] range but past
# its hi asserts 02034714; returns the index of the script in its file (u16)).
fn 02158cd0 ret=u32 inputs=600
  nowork
  arg u32 alt range 0 0x30 || any
  arg u32 alt range 0 3 || range 1 1999 || near 2000 3 || range 2000 10524 || near 10524 3 || oneof 2099 2100 10149 10150 10199 10200 0xFFFF || any
  arg buf 4
  arg buf 4
  arg buf 4
  stub 02013b5c ret=u16 args=v
  stub 02013b68 ret=u16 args=v
  stub 02034714 noreturn ret=none args=v,v,v

# 02158d78(file, heap): 0204A1BC(57, file, 0, heap): load the script file (ARC 57).  (The `bne` after `movs r2, 0` never
# branches.)
fn 02158d78 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 0204a1bc ret=u32 args=v,v,v,v

# 02158d90(id): 0 for id 2000, 1 for id < 0x291C, else 0 (no symbol; called from overlay 21 0x02189E18 / 0x0218A1B0 / 0x0218A33C).
fn 02158d90 ret=u32 inputs=200
  nowork
  arg u32 alt near 2000 3 || near 0x291C 3 || range 0 0x3000 || any

# 02158db0(x): x != 0.
fn 02158db0 ret=u32 inputs=100
  nowork
  arg u32 alt const 0 || any

# 02158dbc(): fatal stop: if 0203CD98() { 0203CF04(); 0203CEAC(0); 0203D0D0(); do 0203CF5C() while !0203CDBC() } then
# 020053EC() forever (modelled as noreturn: the run halts there).
fn 02158dbc ret=none inputs=100
  nowork
  stub 0203cd98 ret=pick:0/1 args=
  stub 0203cf04 ret=none args=
  stub 0203ceac ret=none args=v
  stub 0203d0d0 ret=none args=
  stub 0203cf5c ret=none args=
  stub 0203cdbc seq ret=pick:0/0/1 args=
  stub 020053ec noreturn ret=none args=

# 02158de8(vm, ctx, gamesys, cmd): the VM pre-dispatch hook: 0201210C(gamesys) (unused), 0215a5d4(ctx) (unused); mode =
# 0215a5d8(ctx); ok = bit `mode` of the byte 0x02170374[cmd] (0 for mode >= 3); not ok -> 02158dbc (fatal).  Returns ok.
fn 02158de8 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x40
  arg u32 any
  arg u32 alt range 0 608 || range 0 70 || range 300 330
  fix arg1 8 u32 alt range 0 2 || range 0 4 || any
  stub 0201210c ret=u32 args=v
  stub 0215a5d4 ret=u32 args=v
  stub 0215a5d8 ret=pick:0/1/2/0/1/2/3 args=v
  stub 02158dbc noreturn ret=none args=
  stub 0203cd98 ret=pick:0/1 args=
  stub 0203cf04 ret=none args=
  stub 0203ceac ret=none args=v
  stub 0203d0d0 ret=none args=
  stub 0203cf5c ret=none args=
  stub 0203cdbc seq ret=pick:0/0/1 args=
  stub 020053ec noreturn ret=none args=

# 02158e38(vm): 02011328(vm, 0x02158DE9, 0215a5dc(02011324(vm))): pre-dispatch hook 02158de8 with the GAMESYS as its arg.
# arg0 arena: ctx 0x100 (work block 0x200).
fn 02158e38 ret=none inputs=100
  nowork
  arg buf 0x400
  fix arg0 0x120 u32 addrof arg0 0x200
  stub 02011324 ret=ptr:arg0:0x100:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02011328 ret=none args=v,v,v

# 02158e54(dst, gamesys): fm = 0201210C(gamesys); MIi_CpuFill32(0, dst, 8); dst[1] = fm; fm != 0 -> dst[0] = 02188C54(fm).
fn 02158e54 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v

# 02158e7c(heap, gamesys, event, script id, a, b): SW = Heap_AllocDebug(heap, 0x14C, 1, "script_work.c", 0xAF), fields as
# in the header; message resources (wordset, two strbufs) only when b == 0; +0x148 = 0215a514(SW, heap); +0x18 via 02158e54;
# with b == 0 the proc mask is reset (02159130).  arg6 = arena the allocation lands in (a stack word the function never reads).
fn 02158e7c ret=ofs:arg6 inputs=300
  nowork
  arg u32 alt range 0 0x10 || any
  arg u32 any
  arg u32 any
  arg u32 alt range 0 10524 || any
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x200
  stub 02030734 ret=ptr:arg6:0:1 args=v,v,v,v,v
  stub 02158db0 ret=pick:0/1 args=v
  stub 0201ec74 ret=u32 args=v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0215a514 ret=u32 args=v,v
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v
  stub 02159130 ret=none
  global 02172ba0 4

# 02158f18(sw): SW+0 = 0; free the wordset (0201ED04), the two strbufs (02045808) when nonzero, the scrcmd work (+0x148,
# Heap_Free via the veneer 0215a554) and SW itself.
fn 02158f18 ret=none inputs=200
  nowork
  arg buf 0x200
  fix arg0 0x2C u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x34 u32 alt const 0 || any
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  stub 020307b0 ret=none args=v

# ---------------------------------------------------------------- SW accessors

# 02158f50(sw): +0x14 (GMEVENT).
fn 02158f50 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f54(sw): +0x10 (GAMESYS).
fn 02158f54 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f58(sw): refresh the 8-byte field block at +0x18 (02158e54(sw+0x18, sw+0x10)) and return its address.
fn 02158f58 ret=ofs:arg0 inputs=200
  nowork
  arg buf 0x200
  stub 02158e54 ret=none args=v,v
  stub 0201210c ret=nullable:3:u32 args=v
  stub 02188c54 ret=u32 args=v

# 02158f6c(sw): +0x148 (scrcmd work).
fn 02158f6c ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f74(sw): +0x2C (wordset).
fn 02158f74 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f78(sw): +0x30 (strbuf 1; no symbol, 9 callers in overlay 21).
fn 02158f78 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f7c(sw): +0x34 (strbuf 2).
fn 02158f7c ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f80(sw, v): +0x38 = v.
fn 02158f80 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u32 any

# 02158f84(sw): +0x38.
fn 02158f84 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f88(sw): address sw + 0x40.
fn 02158f88 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x200

# 02158f8c(sw): u16 +4 (script id).
fn 02158f8c ret=u16 inputs=100
  nowork
  arg buf 0x200

# 02158f90(sw): +8 (MMDL).
fn 02158f90 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158f94(sw, mmdl): +8 = mmdl; mmdl != 0 -> var 0x8011 (02158fd0) = 0216D588(mmdl) (the OBJ id).
fn 02158f94 ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 alt const 0 || any
  stub 02158fd0 ret=ptr:arg0:0x9e:1 args=v,v
  stub 0216d588 ret=u16 args=v

# 02158fb4(sw): address sw + 0x3C.
fn 02158fb4 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x200

# 02158fb8(sw): +0x3C.
fn 02158fb8 ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02158fbc(sw): +0x3C != 0 -> Heap_Free(+0x3C), +0x3C = 0.
fn 02158fbc ret=none inputs=100
  nowork
  arg buf 0x200
  fix arg0 0x3C u32 alt const 0 || any
  stub 020307b0 ret=none args=v

# 02158fd0(sw, var): address of script var `var` = sw + 0x7C + (var - 0x8000) * 2 (no range check; plain arithmetic).
fn 02158fd0 ret=u32 inputs=200
  nowork
  arg buf 0x200
  arg u32 alt range 0x8000 0x805F || any

# 02158fe0(sw, gamedata, var): address of var `var`: < 0x4000 -> 0; < 0x8000 -> event work var (020143CC(0201296C(gamedata),
# var)); < 0xC000 -> script var (02158fd0); else 0.  arg0 arena: SW 0, event work var zone 0x400.
fn 02158fe0 ret=ofs:arg0 inputs=400
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 alt range 0 0x10 || near 0x4000 4 || range 0x4000 0x7FFF || near 0x8000 4 || range 0x8000 0x805F || near 0xC000 4 || oneof 0xFFFF 0x10000 0x14000 0xFFFFFFFF || any
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg0:0x400:0x100 args=v,v
  stub 02158fd0 ret=u32 args=v,v

# 0215901c(sw, gamedata, var): value of var `var` (u16 behind 02158fe0), or `var` itself when it has no address (a literal).
fn 0215901c ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 alt range 0 0x10 || near 0x4000 4 || range 0x4000 0x7FFF || near 0x8000 4 || range 0x8000 0x805F || range 0xC000 0xC003 || oneof 0xFFFF 0x10000 0xFFFFFFFF
  stub 02158fe0 ret=nullable:3:ptr:arg0:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg0:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg0:0x7c:0x30 args=v,v

# 02159030(sw, var, value): store the u16 value into var `var` (02158fe0(sw, 020120F4(sw+0x10), var)); 0 when it has no address.
fn 02159030 ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg u32 alt range 0 0x10 || near 0x4000 4 || range 0x4000 0x7FFF || near 0x8000 4 || range 0x8000 0x805F || range 0xC000 0xC003 || oneof 0xFFFF 0xFFFFFFFF
  arg u32 any
  stub 020120f4 ret=u32 args=v
  stub 02158fe0 ret=nullable:3:ptr:arg0:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg0:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg0:0x7c:0x30 args=v,v

# 02159058(sw, a, b, c, d): vars 0x8000..0x8003 = a, b, c, (u16) d (stack word) via 02159030.
fn 02159058 ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02159030 ret=pick:0/1 args=v,v,v16
  stub 020120f4 ret=u32 args=v
  stub 02158fe0 ret=nullable:3:ptr:arg0:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg0:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg0:0x7c:0x30 args=v,v

# 0215909c(sw): copy of script vars 0x8020..0x805E: buf = Heap_AllocDebug(sw+0xC heap, 0x7E, 1, "script_work.c", 0x21B);
# MIi_CpuCopy16(var address 0x8020, buf, 0x7E); returns buf.  arg0 arena: SW 0, the copy at 0x200.
fn 0215909c ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x400
  stub 02030734 ret=ptr:arg0:0x200:1 args=v,v,v,v,v
  stub 02158fd0 ret=ptr:arg0:0xbc:1 args=v,v

# 021590d4(sw, buf): restore vars 0x8020..0x805E from buf (MIi_CpuCopy16 0x7E bytes), Heap_Free(buf).
fn 021590d4 ret=none inputs=100
  nowork
  arg buf 0x200
  arg buf 0x80
  stub 02158fd0 ret=ptr:arg0:0xbc:1 args=v,v
  stub 020307b0 ret=none args=v

# 021590f4(sw, v): +0x13C = v.   021590fc(sw): +0x13C.   02159104(sw, v): +0x140 = v (no symbol).   0215910c(sw): +0x140.
# 02159114(sw, v): +0x144 = v.   0215911c(sw): +0x144.
fn 021590f4 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u32 any

fn 021590fc ret=u32 inputs=100
  nowork
  arg buf 0x200

fn 02159104 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u32 any

fn 0215910c ret=u32 inputs=100
  nowork
  arg buf 0x200

fn 02159114 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u32 any

fn 0215911c ret=u32 inputs=100
  nowork
  arg buf 0x200

# 02159124(sw, i): address sw + 0x44 + i * 0x1C (no symbol; plain arithmetic, no range check).
fn 02159124 ret=u32 inputs=100
  nowork
  arg buf 0x200
  arg u32 alt range 0 3 || any

# ---------------------------------------------------------------- init-proc mask (u32 at 0x02172BA0, bits 0..13)

# 02159130(): mask = 0 (MIi_CpuFill32(0, 0x02172BA0, 4), tail jump).
fn 02159130 ret=none inputs=50
  nowork
  global 02172ba0 4

# 02159144(bit): bit < 14 (signed) and not yet set (02159198) -> set it.
fn 02159144 ret=none inputs=200
  nowork
  arg s32 alt range 0 13 || near 14 2 || any
  global 02172ba0 4
  stub 02159198 ret=pick:0/1 args=v

# 02159168(bit): bit < 14 (signed) and set (02159198) -> clear it.
fn 02159168 ret=none inputs=200
  nowork
  arg s32 alt range 0 13 || near 14 2 || any
  global 02172ba0 4
  stub 02159198 ret=pick:0/1 args=v

# 02159198(bit): bit >= 14 (signed) -> 0, else bit `bit` of the mask.
fn 02159198 ret=u32 inputs=200
  nowork
  arg s32 alt range 0 13 || near 14 2 || any
  global 02172ba0 4

# 021591b4(-, -, task): run the init procs of table 0x021705F0 from task+0x10 on: an entry 0 ends (1); a proc whose bit is
# set (02159198) is called as proc(task, &task+0xC) and a 0 result returns 0 (call again later); after each entry the index
# +0x10 grows, +0xC = 0, and index >= +0x14 (signed) returns 1.  The 14 procs are stubs (overlays 10/21; F8/F10 code).
fn 021591b4 ret=u32 inputs=300
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x20
  fix arg2 0x10 u32 range 0 14
  fix arg2 0x14 s32 alt range 0 15 || any
  global 02172ba0 4
  stub 02159198 ret=pick:0/1/1 args=v
  stub 021b52e0 ret=pick:0/1/1 args=v,v
  stub 021b0630 ret=pick:0/1/1 args=v,v
  stub 021b0ee8 ret=pick:0/1/1 args=v,v
  stub 021b10f4 ret=pick:0/1/1 args=v,v
  stub 021b13ec ret=pick:0/1/1 args=v,v
  stub 021b1234 ret=pick:0/1/1 args=v,v
  stub 021b14e8 ret=pick:0/1/1 args=v,v
  stub 021649c8 ret=pick:0/1/1 args=v,v
  stub 021d1bd8 ret=pick:0/1/1 args=v,v
  stub 021b1604 ret=pick:0/1/1 args=v,v
  stub 021af174 ret=pick:0/1/1 args=v,v
  stub 0215f2a0 ret=pick:0/1/1 args=v,v
  stub 021aefa4 ret=pick:0/1/1 args=v,v
  stub 021af1d0 ret=pick:0/1/1 args=v,v

# ======================================================================== script_work.c tail: commands 49, 50, 311 + accessors
# Command handlers are fn(vm, ctx) -> 0 (next command) / 1 (yield).  ctx arena (arg1, 0x800): ctx 0..0x24, work block 0x40
# (+0 -> SW 0x200, +4 GAMESYS, +8 GAMEDATA, +0xC), SW 0x200..0x34C, event work var zone 0x400..0x800.

# 02159960(vm, ctx): wait callback: 1 while (020362DC() & 3) != 0 (the pad/touch state).   02159984: mask 0xF3.
fn 02159960 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 020362dc ret=pick:0/1/2/3/4/8/0x10/0x80 args=

fn 02159984 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 020362dc ret=pick:0/1/2/3/4/8/0x10/0x80/0x0C args=

# 02159974(vm, ctx): command 49: wait (020113D0(vm, 0x02159961)) and yield.   02159998: command 50 with 0x02159985.
fn 02159974 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  stub 020113d0 ret=none args=v,v

fn 02159998 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  stub 020113d0 ret=none args=v,v

# 021599a8(vm, ctx): wait callback of command 311.  fm = 02188CCC(0201210C(GAMESYS)); if !02012E40(GAMEDATA): r = 1 unless
# (021A0F90(fm) == 7 and 021A153C(fm) == 0); else switch 02012DD0(GAMEDATA): 0/1 -> 0, 2 -> var(vm) = 1, 0; 3 -> var(vm) = 0
# (02159ae8 reads the var id from the script), 021A0F90(fm) == 7 -> 021A1548(fm); r = 1; others -> 1.  r == 1 -> x =
# 021590fc(SW); 021D11D8(x); 021D1194(x); 021590f4(SW, 0).  Returns r.
fn 021599a8 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x20 u32 addrof arg1 0x40
  fix arg1 0x40 u32 addrof arg1 0x200
  stub 0215a5e4 ret=u32 args=v
  stub 0215a5f4 ret=ptr:arg1:0x200:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0201210c ret=u32 args=v
  stub 02188ccc ret=u32 args=v
  stub 02012e40 ret=pick:0/1 args=v
  stub 021a0f90 ret=pick:7/7/0/1 args=v
  stub 021a153c ret=pick:0/1 args=v
  stub 02012dd0 ret=pick:0/1/2/3/4 args=v
  stub 02159ae8 ret=ptr:arg1:0x400:0x100 args=v,v
  stub 021a1548 ret=none args=v
  stub 021590fc ret=u32 args=v
  stub 021d11d8 ret=none args=v
  stub 021d1194 ret=none args=v
  stub 021590f4 ret=none args=v,v
  stub 02011330 ret=pick:0x4000/0x4001/0x7FFF/0x8000/0x8001/0x805F/0x8020 args=v
  stub 02158fe0 ret=ptr:arg1:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg1:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg1:0x27c:0x30 args=v,v

# 02159a64(vm, ctx): command 311.  gs = GAMESYS; fm = 0201210C(gs); x = 021D1164(02188C84(fm), fm); 021D11B0(x); with
# 021A0F90(02188CCC(fm)) == 7: 021A1524 / 021A1530 on it; SW+0x13C = x (021590f4); 02008DF0(02012F2C(GAMEDATA), 1);
# 02012DAC(GAMEDATA); wait on 021599a8 (020113D0(vm, 0x021599A9)); returns 1.
fn 02159a64 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x20 u32 addrof arg1 0x40
  fix arg1 0x40 u32 addrof arg1 0x200
  stub 0215a5f4 ret=ptr:arg1:0x200:1 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0201210c ret=u32 args=v
  stub 02188c84 ret=u32 args=v
  stub 02188ccc ret=u32 args=v
  stub 021d1164 ret=u32 args=v,v
  stub 021d11b0 ret=none args=v
  stub 021a0f90 ret=pick:7/0/1 args=v
  stub 021a1524 ret=none args=v
  stub 021a1530 ret=none args=v
  stub 021590f4 ret=none args=v,v
  stub 02012f2c ret=u32 args=v
  stub 02008df0 ret=none args=v,v
  stub 02012dac ret=none args=v
  stub 020113d0 ret=none args=v,v

# 02159ae8(vm, ctx): address of the var whose id is the next u16 of the script (02158fe0(SW, GAMEDATA, 02011330(vm))).
# 280 command handlers use it (F1 hub): `*02159ae8(vm, ctx) = x` stores into a script/event var.
fn 02159ae8 ret=ofs:arg1 inputs=400
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x20 u32 addrof arg1 0x40
  fix arg1 0x40 u32 addrof arg1 0x200
  stub 0215a5e4 ret=u32 args=v
  stub 0215a5f4 ret=ptr:arg1:0x200:1 args=v
  stub 02011330 ret=pick:0/1/0x3FFF/0x4000/0x4001/0x7FFF/0x8000/0x8001/0x805F/0x8020/0xC000/0xFFFF args=v
  stub 02158fe0 ret=nullable:3:ptr:arg1:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg1:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg1:0x27c:0x30 args=v,v

# 02159b10(vm, ctx): value of the next script u16 read as a var or literal (0215901c(SW, GAMEDATA, 02011330(vm))).
fn 02159b10 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x20 u32 addrof arg1 0x40
  fix arg1 0x40 u32 addrof arg1 0x200
  stub 0215a5e4 ret=u32 args=v
  stub 0215a5f4 ret=ptr:arg1:0x200:1 args=v
  stub 02011330 ret=pick:0/1/0x3FFF/0x4000/0x4001/0x7FFF/0x8000/0x8001/0x805F/0x8020/0xC000/0xFFFF args=v
  stub 0215901c ret=u32 args=v,v,v
  stub 02158fe0 ret=nullable:3:ptr:arg1:0x400:0x100 args=v,v,v
  stub 0201296c ret=u32 args=v
  stub 020143cc ret=ptr:arg1:0x400:0x100 args=v,v
  stub 02158fd0 ret=ptr:arg1:0x27c:0x30 args=v,v

# 02159b38(ctx): zone id = 02012AC8(020127A4(GAMEDATA)) (takes the ctx in r0, unlike the command helpers above).
fn 02159b38 ret=u32 inputs=100
  nowork
  arg buf 0x80
  fix arg0 0x20 u32 addrof arg0 0x40
  stub 0215a5e4 ret=u32 args=v
  stub 020127a4 ret=u32 args=v
  stub 02012ac8 ret=u16 args=v

# ---------------------------------------------------------------- scrcmd ctx getters (scrcmd_work.c, F1 hubs)
# 0215a5cc u16 +0 (heap), 0215a5d0 u16 +4 (zone id), 0215a5d4 +0xC (mode != 0), 0215a5d8 +8 (mode), and through the work
# block +0x20: 0215a5f4 [0] SW (325 callers), 0215a5dc [1] GAMESYS (276), 0215a5e4 [2] GAMEDATA (149), 0215a5ec [3].
fn 0215a5cc ret=u16 inputs=100
  nowork
  arg buf 0x40

fn 0215a5d0 ret=u16 inputs=100
  nowork
  arg buf 0x40

fn 0215a5d4 ret=u32 inputs=100
  nowork
  arg buf 0x40

fn 0215a5d8 ret=u32 inputs=100
  nowork
  arg buf 0x40

fn 0215a5dc ret=u32 inputs=100
  nowork
  arg buf 0x80
  fix arg0 0x20 u32 addrof arg0 0x40

fn 0215a5e4 ret=u32 inputs=100
  nowork
  arg buf 0x80
  fix arg0 0x20 u32 addrof arg0 0x40

fn 0215a5ec ret=u32 inputs=100
  nowork
  arg buf 0x80
  fix arg0 0x20 u32 addrof arg0 0x40

fn 0215a5f4 ret=u32 inputs=100
  nowork
  arg buf 0x80
  fix arg0 0x20 u32 addrof arg0 0x40
