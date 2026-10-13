# fndiff spec: vm.c, the script interpreter (F7-spec), main ARM9 Thumb code 0x020111EC..0x020113D8 (T1 unit unk_020111F8;
# the "vm.c" debug string is the file argument of the allocation in 0x0201121C).  Grammar: svflow_specs/README.md (generic
# mode).  Run: tools/check-fn.sh black-script vm --per-function
#
# VM layout (0x30 + 4 * (stack + regs) bytes; script_sys.c creates it with stack 0x100, regs 0x40 = 0x530 bytes):
#   +0x00 u16 stack words          +0x02 u16 reg words           +0x04 command table (fn(vm, ctx) -> 0 next / 1 yield)
#   +0x08 u32 command count        +0x0C u8 call-stack depth     +0x0D u8 status: 0 stopped, 1 running, 2 waiting
#   +0x0E u8 (cleared by reset)    +0x10 wait callback fn(vm, ctx) -> 1 done   +0x14 PC (byte pointer, 0 = no script)
#   +0x18 -> call stack u32[stack] (= +0x30 + regs * 4)          +0x1C -> regs u32[regs] (= +0x30)
#   +0x20 ctx (the command handlers' second argument)            +0x24 pre-dispatch hook fn(vm, ctx, arg, cmd) -> 0 stops
#   +0x28 hook argument            +0x2C.. owner data (script_sys.c: the script file)
# Step 0x02011298: status 0 -> 0; status 2 -> with a wait callback: call it, 1 -> callback cleared and status 1; return 1
# either way; without one -> status 1 and run; status 1 -> loop { PC == 0 -> stop (status 0), 0; cmd = u16 at PC (PC += 2);
# cmd >= count -> stop, 0; hook set and hook(vm, ctx, arg, cmd) == 0 -> stop, 0; table[cmd](vm, ctx) == 1 -> return 1 };
# any other status -> status 0, return 1.  Script operands are little-endian u16 (0x02011330) / u32 (0x0201134C) at PC.
#
# Not specced: 0x02011264 (`ldr r3,=Heap_Free; bx r3`, the VM free veneer), 0x020111E8 (returns 0x15; no tie to the VM).
# Arena (arg0 of most blocks, 0x800 bytes): VM at 0 (regs 0x30.., call stack 0x130..), script bytes at 0x600, command table
# at 0x700 (0x40 entries, all the fake handler 0x02EE0031); fake wait callback 0x02EE0011, fake hook 0x02EE0021.
area vm
set field

# 020111ec(vm): reset: PC, +0xE, status, wait callback, depth = 0; clear the call stack and the regs (MI_CpuFill8); hook and
# its argument = 0.
fn 020111ec ret=none inputs=200
  nowork
  arg buf 0x600
  fix arg0 0 u16 range 0 0x100
  fix arg0 2 u16 range 0 0x40
  fix arg0 0x18 u32 addrof arg0 0x130
  fix arg0 0x1C u32 addrof arg0 0x30

# 0201121c(heap, init): VM = Heap_AllocDebug(heap, 0x30 + 4 * (regs + stack), 0, "vm.c", 0x60), zeroed; the 12-byte init
# record {u16 stack, u16 regs, table, count} is copied to +0; +0x1C = VM + 0x30, +0x18 = VM + 0x30 + regs * 4; returns VM.
# arg2 = arena the allocation lands in (a register the function does not read).
fn 0201121c ret=ofs:arg2 inputs=200
  nowork
  arg u32 alt range 0 0x10 || any
  arg buf 0x10
  arg buf 0x600
  fix arg1 0 u16 range 0 0x100
  fix arg1 2 u16 range 0 0x40
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v

# 0201126c(vm, ctx): reset (020111ec) and ctx +0x20 = ctx.
fn 0201126c ret=none inputs=100
  nowork
  arg buf 0x600
  arg u32 any
  fix arg0 0 u16 range 0 0x100
  fix arg0 2 u16 range 0 0x40
  fix arg0 0x18 u32 addrof arg0 0x130
  fix arg0 0x1C u32 addrof arg0 0x30
  stub 020111ec ret=none args=v

# 0201127c(vm, pc): reset (020111ec), PC = pc, status 1 (start a script).
fn 0201127c ret=none inputs=100
  nowork
  arg buf 0x600
  arg u32 any
  fix arg0 0 u16 range 0 0x100
  fix arg0 2 u16 range 0 0x40
  fix arg0 0x18 u32 addrof arg0 0x130
  fix arg0 0x1C u32 addrof arg0 0x30
  stub 020111ec ret=none args=v

# 02011290(vm): stop: status 0, PC 0.
fn 02011290 ret=none inputs=100
  nowork
  arg buf 0x40

# 02011298(vm): one step (see the header).
fn 02011298 ret=u32 inputs=600
  nowork
  arg buf 0x800
  fix arg0 4 u32 addrof arg0 0x700
  fix arg0 8 u32 alt range 0 0x40 || range 0x30 0x40
  fix arg0 0xD u8 alt range 0 2 || range 0 3 || any
  fix arg0 0x10 u32 alt const 0 || const 0x02EE0011
  fix arg0 0x14 u32 alt addrof arg0 0x600 || const 0
  fix arg0 0x24 u32 alt const 0 || const 0x02EE0021
  fix arg0 0x600 u16 range 0 0x48 rep=0x40,2
  fix arg0 0x700 u32 const 0x02EE0031 rep=0x40,4
  stub 02ee0010 ret=pick:0/1 args=v,v
  stub 02ee0020 ret=pick:0/1/1/1 args=v,v,v,v
  stub 02ee0030 seq ret=pick:0/0/0/1 args=v,v
  stub 02011330 seq ret=pick:0/1/2/5/0x3F/0x40/0x41/0xFFFF args=v

# 02011324(vm): ctx (+0x20).
fn 02011324 ret=u32 inputs=100
  nowork
  arg buf 0x40

# 02011328(vm, hook, arg): +0x24 = hook, +0x28 = arg.
fn 02011328 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any

# 02011330(vm): u16 LE at PC, PC += 2.
fn 02011330 ret=u32 inputs=200
  nowork
  arg buf 0x100
  fix arg0 0x14 u32 alt addrof arg0 0x40 || addrof arg0 0x41 || addrof arg0 0xFE

# 0201134c(vm): u32 LE at PC (any alignment), PC += 4.
fn 0201134c ret=u32 inputs=200
  nowork
  arg buf 0x100
  fix arg0 0x14 u32 alt addrof arg0 0x40 || addrof arg0 0x41 || addrof arg0 0x43 || addrof arg0 0xFC

# 0201137c(vm, v): push v on the call stack when depth + 1 < stack words (signed); depth += 1.
fn 0201137c ret=none inputs=300
  nowork
  arg buf 0x600
  arg u32 any
  fix arg0 0 u16 alt range 0 0x100 || range 0 4
  fix arg0 0xC u8 alt range 0 0xFF || range 0 5
  fix arg0 0x18 u32 addrof arg0 0x130

# 02011398(vm): pop: depth 0 -> 0; else depth -= 1, returns the call stack word at the new depth.
fn 02011398 ret=u32 inputs=300
  nowork
  arg buf 0x600
  fix arg0 0xC u8 alt range 0 0xFF || range 0 3
  fix arg0 0x18 u32 addrof arg0 0x130

# 020113b0(vm, pc): jump: PC = pc.
fn 020113b0 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any

# 020113b4(vm, pc): call: push the current PC (0201137c), PC = pc.
fn 020113b4 ret=none inputs=200
  nowork
  arg buf 0x600
  arg u32 any
  fix arg0 0 u16 alt range 0 0x100 || range 0 4
  fix arg0 0xC u8 alt range 0 0xFF || range 0 5
  fix arg0 0x18 u32 addrof arg0 0x130
  stub 0201137c ret=none args=v,v

# 020113c4(vm): return: PC = pop (02011398).
fn 020113c4 ret=none inputs=200
  nowork
  arg buf 0x600
  fix arg0 0xC u8 alt range 0 0xFF || range 0 3
  fix arg0 0x18 u32 addrof arg0 0x130
  stub 02011398 ret=u32 args=v

# 020113d0(vm, fn): wait: status 2, wait callback = fn.
fn 020113d0 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any
