# fndiff spec (F11): weather.c (overlay 21 Thumb code 0x021A19E8..0x021A1DB4, the first part of F1 row 21.18; 16 functions, all
# specced): the weather manager. It owns two weather tasks (weather_task.c, area `weather_task`), the overlay of the current weather
# and two 3D backgrounds, and changes the weather: a plain change (stop the old task, swap the overlay, start the new one) or a
# cross-fade between rain-like weathers (1 <-> 4 / 5, 4 <-> 5: the second task starts while the first stops).
# File bound: `Heap_AllocDebug(heap, 0x30, 1, "weather.c", 0x158)` in 021A19E8 (string 0x021DDE74); field_3dbg.c ends at 0x021A19DA,
# weather_task.c starts at 0x021A1DB4 (its allocation in 021A1DB4 names "weather_task.c", 0x021DDE80).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field weather --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers (the proc table 0x021DDE6C = {021A1C69, 021A1D05}, the
# TCB callback 021A1C1D), the Ghidra index and a Thumb walk from every entry (16; F1 counts weather.c as 16). 021A1BDC is a thunk to
# 021A1C34 (`ldr r3, =0x021A1C35; bx r3`), 021A1BF8 stores and tail-calls 02034AC4 (overlay load).
#
# WEATHER (0x30, 021A19E8): +0 / +4 two 3D backgrounds (field_3dbg 021A15A4 with the ROM parameters 0x021D8850 + 12 i), +8 u16 the change
# proc (0 plain 021A1C68, 1 cross-fade 021A1D04; 021A1D78 picks it), +0xA u16 the change step (0 idle), +0xC u16 the current weather,
# +0xE u16 the next (0xFFFF none), +0x10 a flag passed to the new task (1 after a change; 021A1BE4 clears it), +0x14 u16 a weather asked
# for during a change (0xFFFF none), +0x18 the sprite system (0204B100(64, 8, heap)), +0x1C its resources (0204B034(0x021D8840, 1,
# heap)), +0x20 / +0x24 the two weather tasks (+0x20 the active one), +0x28 the loaded weather overlay (-1 none), +0x2C the TCB of the
# V-blank callback 021A1C1C (020056A0(0x021A1C1D, W, 0x40)). The weather table (ROM 0x021D8868, 15 entries of {task data (in the
# weather's overlay), overlay id 0x18..0x1C}) is indexed by the weather 0..14.
#
# Callees in weather_task.c (021A1DB4 new, 021A1DF8 delete, 021A1E08 frame, 021A1FEC start, 021A2050 stop, 021A20C8 force end,
# 021A21B0 status (thunk to 021A273C)), field_3dbg.c (G8) and the sprite / TCB / overlay system are `force` stubs.
# All blocks are `nowork`; the WEATHER is an `arg buf 0x100` arena (current / next weather 0..14, the proc kind 0..1, the change
# step 0..5). Closure stubs (as core4 / F5b): a block lists the stubs of its callees and of the callees of its in-area descendants
# (checked with every function treated as ported: no fault, stray write or unstubbed callee); the proc-table call of 021A1AF4 lands on
# the in-area stubs 021A1C68 / 021A1D04.
area weather
set field

# 021A19E8(a, b, c, d, e, f, heap) -> WEATHER: W = Heap_AllocDebug(heap, 0x30, 1, "weather.c", 0x158); W+0 / +4 = 021A15A4(0x021D8850 +
# 12 i, heap); +8 = +0xA = 0; +0xC = +0xE = +0x14 = 0xFFFF; +0x28 = -1; +0x18 = 0204B100(64, 8, heap); +0x1C = 0204B034(0x021D8840, 1,
# heap); 0204B258(+0x18, +0x1C); +0x20 / +0x24 = 021A1DB4(+0x18, a, b, c, d, W, e, f, heap); +0x2C = 020056A0(0x021A1C1D, W, 0x40).
# Arena: arg 7 (a stack word the function never reads).
fn 021a19e8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 02030734 force ret=ptr:arg7:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 021a15a4 force ret=u32 args=v,v  # field_3dbg (G8): new (ROM parameters, heap)
  stub 0204b100 force ret=u32 args=v,v,v  # sprite system: new (64, 8, heap)
  stub 0204b034 force ret=u32 args=v,v,v  # sprite resources: new (ROM parameters, 1, heap)
  stub 0204b258 force ret=none args=v,v  # sprite system: set resources
  stub 021a1db4 force ret=u32 args=v,v,v,v,v,v,v,v,v  # weather_task: new
  stub 020056a0 force ret=u32 args=v,v,v  # TCB add (callback, work, priority)

# 021A1AAC(W): 02030EAC(+0x2C) (TCB delete); 021A1DF8(task i); 021A1C04(W) (unload the overlay); 0204B084(+0x1C); 0204B1CC(+0x18);
# 021A15D4(3D background i); Heap_Free(W).
fn 021a1aac ret=none inputs=60
  nowork
  arg buf 0x100
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 02030eac force ret=none args=v  # TCB delete
  stub 021a1df8 force ret=none args=v  # weather_task: delete
  stub 021a1c04 ret=none args=v  # in-area: unload the overlay
  stub 0204b084 force ret=none args=v  # sprite resources: delete
  stub 0204b1cc force ret=none args=v  # sprite system: delete
  stub 021a15d4 force ret=none args=v  # field_3dbg (G8): delete
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02034a5c force ret=none args=v  # overlay unload (id)

# 021A1AF4(W, x) (one frame): the change proc of +8 (ROM table 0x021DDE6C: 021A1C68 / 021A1D04)(W, x); no change running and a weather
# asked for (+0x14 != 0xFFFF) -> 021A1BDC(W, it) (request), +0x14 = 0xFFFF; 021A1E08(task i, x).
fn 021a1af4 ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a1bdc ret=u32 args=v,v  # in-area: request (thunk)
  stub 021a1e08 force ret=none args=v,v  # weather_task: frame (task, x)
  stub 021a1c68 ret=none args=v,v  # in-area: plain change proc
  stub 021a1d04 ret=none args=v,v  # in-area: cross-fade proc
  stub 021a1c34 ret=pick:0/1 args=v,v  # in-area: request a change (W, w)
  stub 021a2050 force ret=none args=v,v,v  # weather_task: stop (task, a, kind)
  stub 021a21b0 force ret=pick:0/1/2/3/4/5 args=v  # weather_task: status (thunk to 021A273C)
  stub 021a1c04 ret=none args=v  # in-area: unload the overlay
  stub 021a1bf8 ret=none args=v,v  # in-area: load the overlay (W, id)
  stub 021a1fec force ret=none args=v,v,v,v,v,v,v  # weather_task: start (task, data, a, b, kind, flag, x)
  stub 021a1d78 ret=pick:0/1 args=v,v  # in-area: cross-fade? (cur, next)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 02034ac4 force ret=none args=v  # overlay load (id)

# 021A1B38(W): W != 0 -> 021A15F4(3D background i) (i = 0, 1). W may be null.
fn 021a1b38 ret=none inputs=60
  nowork
  arg u32 alt const 0 || const 0x02E20000
  arg buf 0x10
  stub 021a15f4 force ret=none args=v  # field_3dbg (G8): draw

# 021A1B50(W): W != 0 and a task whose status (021A21B0) is 1 -> 1; else 0. W may be null.
fn 021a1b50 ret=u32 inputs=100
  nowork
  arg u32 alt const 0 || const 0x02E20000
  arg buf 0x100
  stub 021a21b0 force ret=pick:0/1/2/3/4/5 args=v  # weather_task: status (thunk to 021A273C)

# 021A1B74(W, w, x) (set the weather at once): w != +0xC or w == 0 -> 021A20C8(task i) (end both), 021A1C04(W), +0xA = 0, 021A1BF8(W,
# overlay of w), 021A1FEC(task 0, data of w, 0, 0, 1, 1, x), +0xC = w, +0xE = 0xFFFF.
fn 021a1b74 ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 alt const 0 || range 0 14
  arg u32 any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a20c8 force ret=none args=v  # weather_task: force end
  stub 021a1c04 ret=none args=v  # in-area: unload the overlay
  stub 021a1bf8 ret=none args=v,v  # in-area: load the overlay (W, id)
  stub 021a1fec force ret=none args=v,v,v,v,v,v,v  # weather_task: start (task, data, a, b, kind, flag, x)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 02034ac4 force ret=none args=v  # overlay load (id)

# 021A1BDC(W, w): thunk to 021A1C34.
fn 021a1bdc ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg u32 alt range 0 2 || range 0 14 || any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a1c34 ret=pick:0/1 args=v,v  # in-area: request a change (W, w)
  stub 021a1d78 ret=pick:0/1 args=v,v  # in-area: cross-fade? (cur, next)

# 021A1BE4(W, w): 021A1C34(W, w); +0x10 = 0.
fn 021a1be4 ret=none inputs=100
  nowork
  arg buf 0x100
  arg u32 alt range 0 2 || range 0 14 || any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a1c34 ret=pick:0/1 args=v,v  # in-area: request a change (W, w)
  stub 021a1d78 ret=pick:0/1 args=v,v  # in-area: cross-fade? (cur, next)

# 021A1BF4(W): +0xC (u16, the current weather).
fn 021a1bf4 ret=u16 inputs=40
  nowork
  arg buf 0x100

# 021A1BF8(W, ovl): +0x28 = ovl; tail call 02034AC4(ovl) (overlay load).
fn 021a1bf8 ret=none inputs=60
  nowork
  arg buf 0x100
  arg u32 any
  stub 02034ac4 force ret=none args=v  # overlay load (id)

# 021A1C04(W): +0x28 != -1 -> 02034A5C(+0x28) (unload), +0x28 = -1.
fn 021a1c04 ret=none inputs=60
  nowork
  arg buf 0x100
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 02034a5c force ret=none args=v  # overlay unload (id)

# 021A1C1C(tcb, W) (V-blank TCB): 021A1670(3D background i).
fn 021a1c1c ret=none inputs=40
  nowork
  arg u32 any
  arg buf 0x10
  stub 021a1670 force ret=none args=v  # field_3dbg (G8): V-blank

# 021A1C34(W, w) (request a change): a change running -> +0x14 = w, 0; w == +0xC and w != 0 -> 0; else +0xE = w, +0x10 = 1, +8 =
# 021A1D78(+0xC, w), +0xA = 1, 1.
fn 021a1c34 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg u32 alt range 0 2 || range 0 14 || any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a1d78 ret=pick:0/1 args=v,v  # in-area: cross-fade? (cur, next)

# 021A1C68(W, x) (plain change): step 1: 021A2050(task 0, 1, 1) (stop), 2; 2: task 0 stopped (021A21B0 == 0) -> 021A1C04(W), 3; 3:
# 021A1BF8(W, overlay of +0xE), 4; 4: 021A1FEC(task 0, data of +0xE, 1, 1, 1, +0x10, x), +0x10 = 1, +0xC = +0xE, +0xE = 0xFFFF, 0.
fn 021a1c68 ret=none inputs=300
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a2050 force ret=none args=v,v,v  # weather_task: stop (task, a, kind)
  stub 021a21b0 force ret=pick:0/1/2/3/4/5 args=v  # weather_task: status (thunk to 021A273C)
  stub 021a1c04 ret=none args=v  # in-area: unload the overlay
  stub 021a1bf8 ret=none args=v,v  # in-area: load the overlay (W, id)
  stub 021a1fec force ret=none args=v,v,v,v,v,v,v  # weather_task: start (task, data, a, b, kind, flag, x)
  stub 02034a5c force ret=none args=v  # overlay unload (id)
  stub 02034ac4 force ret=none args=v  # overlay load (id)

# 021A1D04(W, x) (cross-fade): step 1: 021A1FEC(task 1, data of +0xE, 1, 1, 2, +0x10, x), 021A2050(task 0, 1, 2), 2; 2: task 0 stopped
# -> swap the tasks, +0xC = +0xE, +0xE = 0xFFFF, +0x10 = 1, 0.
fn 021a1d04 ret=none inputs=300
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 8 u16 range 0 1
  fix arg0 0xA u16 alt const 0 || range 0 5 || any
  fix arg0 0xC u16 alt range 0 2 || range 0 14 || const 0xFFFF
  fix arg0 0xE u16 range 0 14
  fix arg0 0x10 u32 alt range 0 1 || any
  fix arg0 0x14 u16 alt range 0 14 || const 0xFFFF
  fix arg0 0x28 u32 alt const 0xFFFFFFFF || range 0x18 0x1C
  stub 021a1fec force ret=none args=v,v,v,v,v,v,v  # weather_task: start (task, data, a, b, kind, flag, x)
  stub 021a2050 force ret=none args=v,v,v  # weather_task: stop (task, a, kind)
  stub 021a21b0 force ret=pick:0/1/2/3/4/5 args=v  # weather_task: status (thunk to 021A273C)

# 021A1D78(cur, next): 1 for a cross-fade (cur 1 -> next 4 / 5, cur 4 -> 1 / 5, cur 5 -> 4 / 1; next compared as u16 for cur 1 and
# as 32 bits otherwise), else 0.
fn 021a1d78 ret=u32 inputs=300
  nowork
  arg u32 alt range 0 6 || oneof 0x10001 0x10004 || any
  arg u32 alt range 0 6 || oneof 0x10004 0x10005 0x10001 || any
