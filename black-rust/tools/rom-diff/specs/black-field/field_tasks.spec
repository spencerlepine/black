# fndiff spec (F14): the concrete field tasks after `field_task_manager.c` (F1 row 21.75, second part; overlay 21 of the
# field set, 0x021C6F00..0x021C7DAA; no `__FILE__` string, named by role) plus the compiler-runtime float compare
# 0x0209BBB0 they call. Run: tools/check-fn.sh black-field field_tasks --per-function
#
# Every task is a constructor `X(fm, ...)` -> FIELD TASK (field_task 021C6C70(heap = 02188C84(fm), size, proc); work =
# 021C6CCC(task)) and its proc `int proc(work)` (1 = done). Tasks: camera distance 021C6F00 / 021C6F30; camera angle
# 021C701C / 021C7058 / 021C71A4 (021C7094 init, 021C70DC step); OBJ spin 021C71D4 / 021C7220 / 021C7260 / 021C72AC with
# the direction curves 021C72F8 / 021C734C / 021C73B4 and procs 021C7418 / 021C748C / 021C7500; player move 021C754C /
# 021C7584; OBJ shake 021C7684 / 021C76C4 / 021C7730; wait 021C77D8 / 021C77FC; deferred call 021C7814 / 021C7844 (calls
# 0204E720 with four words); OBJ offset 021C7858 / 021C7878 / 021C78DC / 021C7940 / 021C7A20 / 021C7A7C; camera target
# 021C7AE4 / 021C7B1C; camera position 021C7C10 / 021C7C74; periodic call 021C7D28 / 021C7D78.
#
# Conventions (as F5b / F13): all blocks `nowork`; a constructor writes the work block that the forced stub 021C6CCC
# answers into a trailing `arg buf` arena the function never reads as a register (`ptr:argK:0:1`); a proc's work is arg0.
# Closure stubs (a block lists the stubs of its callees and of all its in-area descendants; checked with every function
# treated as ported: no fault, stray write or unstubbed callee). Every out-of-area stub is `force`.
# Soft float: 0209BDBC i2f, 0209BE04 u2f, 0209C494 f2i, 0209C95C fadd, 0209CB8C fsub run as real code (svflow ALLOW, the
# ports call black_core::softfloat); fmul 0209C77C, fdiv 0209CE10, f2u 0209C4C8 and the double helpers 0209BD38 /
# 0209C5E0 / 0209B64C are `force` stubs (pure helpers, not ported anywhere). 0209BBB0 (a > b on two floats, 0 for NaN;
# the callers branch on the C flag it sets) cannot be a stub (stubs set no flags): it is specced and ported here so that
# it runs as real code in every block. 0209C054 (64-bit multiply) and 0209C0A4 (signed divide) are ALLOW.
# The direction curves read a 4-entry u16 table copied to the stack at index f2i(x) % 4: a negative index reads below the
# copy (stack garbage); the float that reaches f2i is always >= 0 in the game (products of u2f values), so the last fmul /
# the double-to-float helper answer closed `pick:` lists of non-negative floats (a negative answer would compare stack
# garbage).
area field_tasks
set field

# FUN_0209BBB0(a, b) (ARM, runtime): 1 when float a > float b (signed-magnitude compare, -0 == +0), 0 when either is NaN.
fn 0209bbb0 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 0x80000000 0x7F800000 0xFF800000 0x7FC00000 0x7F800001 0xFFC00000 0xFF800001 0x3F800000 0xBF800000 1 0x80000001 0x7F7FFFFF 0xFF7FFFFF || any
  arg u32 alt rel 0 oneof 0 1 0xFFFFFFFF || xrel 0 oneof 0x80000000 0x80000001 || oneof 0 0x80000000 0x7F800000 0xFF800000 0x7FC00000 0x7F800001 0x3F800000 0xBF800000 || any

# FUN_021C6F00(fm, frames, target): camera distance task, work 0x14 {fm, u8 seq, u16 count, u16 frames, start, target}.
fn 021c6f00 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v

# FUN_021C6F30(work): seq 0 reads the start distance; seq 1 interpolates start -> target over `frames` and sets it.
fn 021c6f30 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 4 u8 alt range 0 2 || any
  fix arg0 6 u16 alt range 0 12 || any
  fix arg0 8 u16 alt range 0 12 || any
  stub 02188c58 force ret=u32 args=v
  stub 0218ed54 force ret=u32 args=v
  stub 0218e138 force ret=u32 args=v
  stub 0218e13c force ret=none args=v,v
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0218ed58 force ret=none args=v,v

# FUN_021C701C(fm, frames, target) / FUN_021C7058: camera angle task (axis 0 / 1), work 0x1C {fm, cam, axis, u8 seq,
# u16 count, u16 frames, start, target}.
fn 021c701c ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v
  stub 02188c58 force ret=u32 args=v

fn 021c7058 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v
  stub 02188c58 force ret=u32 args=v

# FUN_021C7094(work): read the current angle of the axis and unwrap start / target to the short way round.
fn 021c7094 ret=none inputs=400
  nowork
  arg buf 0x40
  fix arg0 8 u32 alt range 0 2 || any
  fix arg0 0x18 u32 alt oneof 0 0x4000 0x7FFF 0x8000 0x8001 0xC000 0xFFFF 0x10000 0x18000 || any
  stub 0218ed40 force ret=pick+u32:0/0x4000/0x8000/0xC000/0xFFFF/0x10000/0x18000 args=v
  stub 0218ed2c force ret=pick+u32:0/0x4000/0x8000/0xC000/0xFFFF/0x10000/0x18000 args=v

# FUN_021C70DC(work): one step of the smoothstep angle interpolation.
fn 021c70dc ret=none inputs=300
  nowork
  arg buf 0x40
  fix arg0 8 u32 alt range 0 2 || any
  fix arg0 0xE u16 alt range 0 12 || any
  fix arg0 0x10 u16 alt range 0 12 || any
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0209c4c8 force ret=u32 args=v
  stub 0218e138 force ret=u32 args=v
  stub 0218e13c force ret=none args=v,v
  stub 0218ed48 force ret=none args=v,v
  stub 0218ed34 force ret=none args=v,v

# FUN_021C71A4(work): the camera angle proc (seq 0: 021C7094, seq 1: 021C70DC until count >= frames).
fn 021c71a4 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 8 u32 alt range 0 2 || any
  fix arg0 0xC u8 alt range 0 2 || any
  fix arg0 0xE u16 alt range 0 12 || any
  fix arg0 0x10 u16 alt range 0 12 || any
  fix arg0 0x18 u32 alt oneof 0 0x4000 0x8000 0x8001 0xC000 0x10000 || any
  stub 021c7094 ret=none args=v
  stub 021c70dc ret=none args=v
  stub 0218ed40 force ret=pick+u32:0/0x4000/0x8000/0xC000/0xFFFF/0x10000/0x18000 args=v
  stub 0218ed2c force ret=pick+u32:0/0x4000/0x8000/0xC000/0xFFFF/0x10000/0x18000 args=v
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0209c4c8 force ret=u32 args=v
  stub 0218e138 force ret=u32 args=v
  stub 0218e13c force ret=none args=v,v
  stub 0218ed48 force ret=none args=v,v
  stub 0218ed34 force ret=none args=v,v

# FUN_021C71D4(fm, frames, turns): player-OBJ spin task (proc 021C7418), work 0x10 {fm, obj, u16 count, u16 frames,
# u8 turns, u16 start dir}.
fn 021c71d4 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v
  stub 021a3074 force ret=u32 args=v
  stub 0216d5f8 force ret=u32 args=v

# FUN_021C7220(fm, frames, turns, obj): the same spin for a given OBJ. arg4 (a stack word, never read) = arena.
fn 021c7220 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 0216d5f8 force ret=u32 args=v

# FUN_021C7260(fm, frames, turns): player-OBJ spin task with the curve 021C734C (proc 021C748C).
fn 021c7260 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v
  stub 021a3074 force ret=u32 args=v
  stub 0216d5f8 force ret=u32 args=v

# FUN_021C72AC(fm, frames, turns, end dir): player-OBJ spin task with the curve 021C73B4 (proc 021C7500). arg4 = arena.
fn 021c72ac ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 021a3074 force ret=u32 args=v

# FUN_021C72F8(count, frames, turns) (floats): direction ROM 0x021DAAC0[f2i(count / frames * 4 turns) % 4].
fn 021c72f8 ret=u16 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0209c77c force ret=nth:u32;pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C734C(count, frames, turns): the same with count^2 / frames^2 (ROM 0x021DAAB8).
fn 021c734c ret=u16 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0209c77c force ret=nth:u32;u32;u32;pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C73B4(count, frames, turns): the same through a square root in double precision (ROM 0x021DAAC8).
fn 021c73b4 ret=u16 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0209c77c force ret=u32 args=v,v
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209bd38 force ret=u64 args=v
  stub 0209c5e0 force ret=u64 args=v,v
  stub 0209b64c force ret=pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v

# FUN_021C7418(work): spin proc: set the curve's direction; after `frames` steps set the final one (start dir 0..3, else 0).
fn 021c7418 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 8 u16 alt range 0 12 || any
  fix arg0 0xA u16 alt range 0 12 || any
  fix arg0 0xE u16 alt range 0 5 || any
  stub 021c72f8 ret=pick+u16:0/1/2/3 args=v,v,v
  stub 0216d410 force ret=none args=v,v
  stub 0209c77c force ret=nth:u32;pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C748C(work): the same with 021C734C.
fn 021c748c ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 8 u16 alt range 0 12 || any
  fix arg0 0xA u16 alt range 0 12 || any
  fix arg0 0xE u16 alt range 0 5 || any
  stub 021c734c ret=pick+u16:0/1/2/3 args=v,v,v
  stub 0216d410 force ret=none args=v,v
  stub 0209c77c force ret=nth:u32;u32;u32;pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C7500(work): the same with 021C73B4; the last step sets the work's end dir (0216D5E0).
fn 021c7500 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 8 u16 alt range 0 12 || any
  fix arg0 0xA u16 alt range 0 12 || any
  stub 021c73b4 ret=pick+u16:0/1/2/3 args=v,v,v
  stub 0216d410 force ret=none args=v,v
  stub 0216d5e0 force ret=none args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209bd38 force ret=u64 args=v
  stub 0209c5e0 force ret=u64 args=v,v
  stub 0209b64c force ret=pick:0/0x3F800000/0x40000000/0x40400000/0x40800000/0x40A00000/0x41200000/0x3F000000/0x4B000000/0x4F800000 args=v,v

# FUN_021C754C(fm, frames, &delta): player move task, work 0x28 {u8 seq, fm, count, frames, start pos (0x10), delta}.
fn 021c754c ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v

# FUN_021C7584(work): seq 0 reads the player position; seq 1 sets start * (1 - t) + delta * t, t = count / frames (fx32).
fn 021c7584 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt range 0 2 || any
  fix arg0 8 u32 alt range 0 12 || any
  fix arg0 0xC u32 alt range 0 12 || any
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 021a3038 force ret=none args=v,out:12
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 020520a4 force ret=none args=in:12,v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 021a3044 force ret=none args=v,in:12
  stub 02164010 force ret=none args=v

# FUN_021C7684(fm, obj, frames, amplitude): OBJ shake task, work 0x14 {u8 seq, fm, u16 count, u16 frames, i2f(amplitude),
# obj}. arg4 = arena.
fn 021c7684 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v

# FUN_021C76C4(&vec, count, frames, amplitude) (floats): vec = {0, f2i(round(4096 * (a - a * (c / f^2) * c))), 0}.
fn 021c76c4 ret=none inputs=300
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0209c77c force ret=u32 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C7730(work): shake proc: offsets the OBJ by 021C76C4 each frame (sound 0x681 at the start, 0x682 at the end).
fn 021c7730 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt range 0 2 || any
  fix arg0 8 u16 alt range 0 12 || any
  fix arg0 0xA u16 alt range 0 12 || any
  stub 02188cb4 force ret=u32 args=v
  stub 021c76c4 ret=none args=out:12,v,v,v
  stub 0216d8c0 force ret=none args=v,in:12
  stub 020061e4 force ret=none args=v
  stub 02188cd0 force ret=u32 args=v
  stub 021ac528 force ret=none args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0209ce10 force ret=u32 args=v,v

# FUN_021C77D8(fm, frames): wait task, work 4 {u16 count, u16 frames}. arg2 (r2, overwritten) = arena.
fn 021c77d8 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg2:0:1 args=v

# FUN_021C77FC(work): count up; done when count >= frames.
fn 021c77fc ret=u32 inputs=200
  nowork
  arg buf 0x10
  fix arg0 0 u16 alt range 0 12 || any
  fix arg0 2 u16 alt range 0 12 || any

# FUN_021C7814(fm, a, b, c, d): deferred call task, work 0x14 {fm, a, b, c, d}. arg5 = arena.
fn 021c7814 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg5:0:1 args=v

# FUN_021C7844(work): 0204E720(a, b, c, d); done.
fn 021c7844 ret=u32 inputs=30
  nowork
  arg buf 0x20
  stub 0204e720 force ret=none args=v,v,v,v

# FUN_021C7858(fm, frames, &delta): OBJ offset task (021C7878) for the player's OBJ. arg3 (r3, overwritten) = arena.
fn 021c7858 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 alt range 0 20 || any
  arg buf 0x10
  arg buf 0x40
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 021c7878 ret=u32 args=v,v,v,v
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v
  stub 0216d88c force ret=none args=v,out:12

# FUN_021C7878(fm, frames, &delta, obj): OBJ offset task (proc 021C7A20), work 0x2C {u8 seq, fm, reverse, u16 count,
# u16 frames, delta, base offset (0x1C), obj}; a negative `frames` runs backwards. arg4 = arena.
fn 021c7878 ret=u32 inputs=80
  nowork
  arg u32 any
  arg u32 alt range 0 20 || any
  arg buf 0x10
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 0216d88c force ret=none args=v,out:12

# FUN_021C78DC: the same with the proc 021C7A7C.
fn 021c78dc ret=u32 inputs=80
  nowork
  arg u32 any
  arg u32 alt range 0 20 || any
  arg buf 0x10
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 0216d88c force ret=none args=v,out:12

# FUN_021C7940(&out, count, &delta, frames): out = delta * round(4096 * count / frames) (fx32 multiply per axis).
fn 021c7940 ret=none inputs=300
  nowork
  arg buf 0x10
  arg u16 alt range 0 20 || any
  arg buf 0x10
  arg u16 alt range 0 20 || any
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v

# FUN_021C7A20(work): offset proc: sets the OBJ offset delta * count / frames (reverse: (frames - count) / frames).
fn 021c7a20 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt const 0 || any
  fix arg0 8 u32 alt range 0 2 || any
  fix arg0 0xC u16 alt range 0 12 || any
  fix arg0 0xE u16 alt range 0 12 || any
  stub 021c7940 ret=none args=out:12,v,in:12,v
  stub 0216d8c0 force ret=none args=v,in:12
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v

# FUN_021C7A7C(work): the same added to the base offset, set as the OBJ position offset (0216D89C).
fn 021c7a7c ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt const 0 || any
  fix arg0 8 u32 alt range 0 2 || any
  fix arg0 0xC u16 alt range 0 12 || any
  fix arg0 0xE u16 alt range 0 12 || any
  stub 021c7940 ret=none args=out:12,v,in:12,v
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 0216d89c force ret=none args=v,in:12
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v

# FUN_021C7AE4(fm, frames, &delta): camera target move task (proc 021C7B1C), work 0x28 as 021C754C.
fn 021c7ae4 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg3:0:1 args=v

# FUN_021C7B1C(work): as 021C7584 on the camera target (0218ECC4 get / 0218ECD4 set).
fn 021c7b1c ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt range 0 2 || any
  fix arg0 8 u32 alt range 0 12 || any
  fix arg0 0xC u32 alt range 0 12 || any
  stub 02188c58 force ret=u32 args=v
  stub 0218ecc4 force ret=none args=v,out:12
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 020520a4 force ret=none args=in:12,v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 0218ecd4 force ret=none args=v,in:12

# FUN_021C7C10(fm, frames, &from, &to): camera position task (proc 021C7C74), work 0x34 {u8 seq, fm, u16 count,
# u16 frames, from (0xC), to (0x18), to - from (0x24), saved word (0x30)}. arg4 = arena.
fn 021c7c10 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 02188c58 force ret=u32 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12

# FUN_021C7C74(work): seq 0 saves and clears a camera word; seq 1 sets from + (to - from) * count / frames.
fn 021c7c74 ret=u32 inputs=300
  nowork
  arg buf 0x40
  fix arg0 0 u8 alt range 0 2 || any
  fix arg0 8 u16 alt range 0 12 || any
  fix arg0 0xA u16 alt range 0 12 || any
  stub 02188c58 force ret=u32 args=v
  stub 0218e138 force ret=u32 args=v
  stub 0218e13c force ret=none args=v,v
  stub 0209ce10 force ret=u32 args=v,v
  stub 0209c77c force ret=u32 args=v,v
  stub 0207cee4 force ret=none args=v,in:12,in:12,out:12
  stub 0218ec90 force ret=none args=v,in:12

# FUN_021C7D28(fm, x, frames, period): periodic task (proc 021C7D78), work 0x14 {fm, 02188CD0(fm), player OBJ, u8 stop,
# u16 count, u16 frames, u16 period}. arg4 = arena.
fn 021c7d28 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02188c84 force ret=u16 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021c6c70 force ret=u32 args=v,v,v
  stub 021c6ccc force ret=ptr:arg4:0:1 args=v
  stub 02188cd0 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v

# FUN_021C7D78(work): every `period` frames 021ADD7C(obj, x); done after `frames`. The period is never 0 here (a 0
# period divides by zero in 0209C0A4).
fn 021c7d78 ret=u32 inputs=300
  nowork
  arg buf 0x20
  fix arg0 0xC u8 alt const 0 || any
  fix arg0 0xE u16 alt range 0 20 || any
  fix arg0 0x10 u16 alt range 0 20 || any
  # V-0034: period 0 drawn too (the runtime divide returns remainder 0 for a zero divisor)
  fix arg0 0x12 u16 alt const 0 || range 1 4 || range 1 0xFFFF
  stub 021add7c force ret=none args=v,v
