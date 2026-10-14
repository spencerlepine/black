# fndiff spec (F14): `field_task.c` + `field_task_manager.c` (F1 row 21.75, first part; overlay 21 of the field set,
# 0x021C6C70..0x021C6F00). Run: tools/check-fn.sh black-field field_task --per-function
#
# FIELD TASK (0xC, `Heap_AllocDebug(heap, 0xC, 0, "field_task.c", 0x3B)` in 021C6C70): +0 u32 state (0 idle, 1 running,
# 2 done), +4 the work block (allocated with the task, size given by the creator), +8 the proc `int proc(work)` (1 = done).
# FIELD TASK MANAGER (0xC, "field_task_manager.c"): +0 u8 slot count, +4 -> task[count], +8 -> wait[count] (the task a
# slot waits for before it starts; 0 = start at once).
#
# Conventions (headers of the F5b / F6a specs): all blocks `nowork`; objects are `arg buf` arenas; the manager arena
# (arg0, 0x200 bytes) holds the manager at 0, task[8] at 0x20, wait[8] at 0x40 and eight tasks at 0x80 + 0x10 * k
# (state 0..2 or noise, work, proc = FAKE callee 0x02EE0011, stubbed at 02EE0010). Closure stubs: a block lists the
# stubs of its callees and of the callees of all its in-area descendants, so the area can be ported in any order (checked
# with every function treated as ported: no fault, stray write or unstubbed callee). Heap_AllocDebug 02030734 is a
# `seqptr` bump allocator into a trailing arena the function never reads; Heap_Free 020307B0 is a `force` stub.
# The two veneers 021C6CBC (-> 021C6CF4) and 021C6CC4 (-> 021C6CE4) are not specced: callers stub their targets.
area field_task
set field

# FUN_021C6C70(heap, size, proc): create a task (state 0, work block of `size` bytes). arg3 = r3 (overwritten) = arena.
fn 021c6c70 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 alt range 0 0x40 || any
  arg u32 any
  arg buf 0x100
  stub 02030734 force ret=seqptr:arg3:0:0x40 args=v,v,v,v,v

# FUN_021C6CA8(task): free the work block, then the task.
fn 021c6ca8 ret=none inputs=50
  nowork
  arg buf 0x10
  stub 020307b0 force ret=none args=v

# FUN_021C6CCC(task): the work block.
fn 021c6ccc ret=u32 inputs=30
  nowork
  arg buf 0x10

# FUN_021C6CD0(task): state == 2 (done).
fn 021c6cd0 ret=u32 inputs=60
  nowork
  arg buf 0x10
  fix arg0 0 u32 alt range 0 3 || any

# FUN_021C6CE0(task): the state.
fn 021c6ce0 ret=u32 inputs=30
  nowork
  arg buf 0x10

# FUN_021C6CE4(task): start (state 1).
fn 021c6ce4 ret=none inputs=30
  nowork
  arg buf 0x10

# FUN_021C6CEC(task): end (state 2).
fn 021c6cec ret=none inputs=30
  nowork
  arg buf 0x10

# FUN_021C6CF4(task): one frame: a running task calls its proc; the proc's 1 ends the task.
fn 021c6cf4 ret=none inputs=120
  nowork
  arg buf 0x10
  fix arg0 0 u32 alt range 0 2 || any
  fix arg0 8 u32 const 0x02EE0011
  stub 02ee0010 force ret=pick+u32:0/1/2 args=v
  stub 021c6cec ret=none args=v

# FUN_021C6D10(count, heap): create a manager with `count` slots. arg2 (r2, overwritten) = arena.
fn 021c6d10 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 8 || range 9 16
  arg u32 any
  arg buf 0x200
  stub 02030734 force ret=seqptr:arg2:0:0x48 args=v,v,v,v,v

# FUN_021C6D5C(mgr): remove every slot (freeing its task), free the arrays and the manager.
fn 021c6d5c ret=none inputs=150
  nowork
  arg buf 0x200
  fix arg0 0 u8 range 0 8
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 0x02E100C0 0x02E100D0 0x02E100E0 0x02E100F0 rep=8,4
  fix arg0 0x40 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  stub 021c6e74 ret=none args=v,v
  stub 021c6ca8 ret=none args=v
  stub 020307b0 force ret=none args=v

# FUN_021C6D90(mgr): one frame of every slot: run the tasks, start the tasks whose wait task is done, drop finished ones.
fn 021c6d90 ret=none inputs=300
  nowork
  arg buf 0x200
  fix arg0 0 u8 range 0 8
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 0x02E100C0 0x02E100D0 0x02E100E0 0x02E100F0 rep=8,4
  fix arg0 0x40 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  fix arg0 0x80 u32 alt range 0 2 || any rep=8,0x10
  fix arg0 0x88 u32 const 0x02EE0011 rep=8,0x10
  stub 021c6e94 ret=none args=v,v
  stub 021c6ea8 ret=none args=v,v
  stub 021c6edc ret=none args=v,v
  stub 021c6cf4 ret=none args=v
  stub 021c6cec ret=none args=v
  stub 021c6ce0 ret=pick+u32:0/1/2 args=v
  stub 021c6ce4 ret=none args=v
  stub 021c6cd0 ret=pick:0/1 args=v
  stub 021c6e74 ret=none args=v,v
  stub 021c6ca8 ret=none args=v
  stub 02ee0010 force ret=pick+u32:0/1/2 args=v
  stub 020307b0 force ret=none args=v

# FUN_021C6DE0(mgr, task, wait): put the task in the first free slot (and start it when `wait` is 0); no free slot:
# free the task. The task is one of the arena's tasks (its state is written / its blocks freed by the descendants).
fn 021c6de0 ret=none inputs=300
  nowork
  arg buf 0x200
  arg u32 oneof 0x02E100C0 0x02E100D0 0x02E100E0 0x02E100F0
  arg u32 alt const 0 || oneof 0x02E10080 0x02E10090
  fix arg0 0 u8 range 0 8
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  stub 021c6e5c ret=none args=v,v,v,v
  stub 021c6ce4 ret=none args=v
  stub 021c6ca8 ret=none args=v
  stub 020307b0 force ret=none args=v

# FUN_021C6E20(mgr): number of occupied slots (u8).
fn 021c6e20 ret=u8 inputs=150
  nowork
  arg buf 0x200
  fix arg0 0 u8 range 0 8
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 0x20 u32 alt const 0 || any rep=8,4

# FUN_021C6E48(mgr): no slot occupied.
fn 021c6e48 ret=u32 inputs=150
  nowork
  arg buf 0x200
  fix arg0 0 u8 range 0 8
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 0x20 u32 alt const 0 || const 0 || any rep=8,4
  stub 021c6e20 ret=pick+u8:0/1/2 args=v

# FUN_021C6E5C(mgr, i, task, wait): fill slot i when it is free.
fn 021c6e5c ret=none inputs=100
  nowork
  arg buf 0x200
  arg u8 range 0 7
  arg u32 any
  arg u32 any
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || any rep=8,4

# FUN_021C6E74(mgr, i): free slot i's task and clear the slot.
fn 021c6e74 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u8 range 0 7
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  stub 021c6ca8 ret=none args=v
  stub 020307b0 force ret=none args=v

# FUN_021C6E94(mgr, i): run slot i's task for one frame (through the veneer 021C6CBC).
fn 021c6e94 ret=none inputs=100
  nowork
  arg buf 0x200
  arg u8 range 0 7
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  fix arg0 0x80 u32 alt range 0 2 || any rep=8,0x10
  fix arg0 0x88 u32 const 0x02EE0011 rep=8,0x10
  stub 021c6cf4 ret=none args=v
  stub 021c6cec ret=none args=v
  stub 02ee0010 force ret=pick+u32:0/1/2 args=v

# FUN_021C6EA8(mgr, i): start slot i's task when it is idle and its wait task is done (start through the veneer
# 021C6CC4).
fn 021c6ea8 ret=none inputs=300
  nowork
  arg buf 0x200
  arg u8 range 0 7
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  fix arg0 0x40 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  fix arg0 0x80 u32 alt range 0 2 || any rep=8,0x10
  stub 021c6ce0 ret=pick+u32:0/1/2 args=v
  stub 021c6ce4 ret=none args=v

# FUN_021C6EDC(mgr, i): remove slot i when its task is done.
fn 021c6edc ret=none inputs=150
  nowork
  arg buf 0x200
  arg u8 range 0 7
  fix arg0 4 u32 addrof arg0 0x20
  fix arg0 8 u32 addrof arg0 0x40
  fix arg0 0x20 u32 alt const 0 || oneof 0x02E10080 0x02E10090 0x02E100A0 0x02E100B0 rep=8,4
  fix arg0 0x80 u32 alt range 0 2 || any rep=8,0x10
  stub 021c6cd0 ret=pick:0/1 args=v
  stub 021c6e74 ret=none args=v,v
  stub 021c6ca8 ret=none args=v
  stub 020307b0 force ret=none args=v
