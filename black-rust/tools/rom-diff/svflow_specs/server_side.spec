# svflow spec: server_side (0x021BAA94-0x021BB968; B10e, drafted by B10-specs). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (see "B10-specs additions": seq, outm, ret=ptr).
#
# Every function here works on the SERVER-SIDE object ("S": the btl_server.c client-adapter state, NOT SvFlowWork), so
# each fn says `nowork` and takes S as `arg buf 0x1000`. Layout seen so far (offsets in S):
#   +0x00 state fn pointer   +0x04 substate   +0x0C main ptr   +0x10 ptr   +0x14 4 client slots * 0xC:
#   slot: +0 handle(ptr) +4 u32 +8 u8 +9 u8 +0xA u8 +0xB u8 id (0xFF = slot unused)   +0x44 ptr   +0x48 u32
#   +0x68 BattleRecordBuilder   +0xA8/+0xAA u16   +0xAC ptr   +0xB0 ptr   +0xB4 4 * 3 words   +0xE4 4 counts
#   +0xEC/+0xED bytes   +0xCB0 ptr   +0xCB4 u8[6] list   +0xCBA u8 count   +0xCC0 u16
# Pointer fields that are DEREFERENCED are fixed to `addrof arg0 0xF00` (data zone 0xF00-0xFFF of the same buffer) or
# come from a `ret=ptr:arg0:0xD00:64` stub (word zone 0xD00-0xDFF); every other pointer field is an opaque random
# value that only reaches a stub. A port resolves such an address as `addr - 0x02E1_0000` (arg 0 is at 0x02E10000).
# State functions (S, &state): the pointer to S's substate word is `arg buf 0x10` (index 1).
#
# B10-spec2: `stub ADDR force ...` keeps a stub active although the callee is ported. Used for 021cc668 (events area; it takes
# the flow work (SvFlowWork) that S[0x44] only addresses, so the Rust state functions call it through env.call). `outn:N:M` is
# `outm` for a non-null out pointer and a plain `v` (0) for a null one: 021eeb64 is called with (handle, &size) by 021bb6a8 /
# 021bb628 and with (handle, 0) by 021bb0a0.
# Thunk callees (`ldr r3,[pc]; bx r3` at 021bb9d0, 021c84d8, 021b917c, 021b8de4, 021eeae0) are stubbed at their TARGET
# (021bb9d8, 021cc668, 021ba54c, 021b8cfc, 020307b0), since the thunk itself is real code in the emulator.
# Stub return kinds follow the REGISTER use in the disassembly, not Ghidra's prototype: a callee Ghidra calls void
# but whose r0 reaches a later call or the return value (021b9920, 021ee8d8, 021ee9a4, 021bb650) is `ret=u32`.
area server_side

# ---- small data helpers ------------------------------------------------------------------------------------

# slot[0xB] = 0xFF (marks the slot unused)
fn 021bb828 ret=none
  nowork
  arg buf 0x10

# slot active? (id byte != 0xFF)
fn 021bb830 ret=u32
  nowork
  arg buf 0x10
  fix arg0 0xB u8 alt const 255 || any

# slot = (id, handle, flag)
fn 021bb840 ret=none
  nowork
  arg buf 0x10
  arg u8 any
  arg u32 any
  arg u8 any

# slot.u32 = val; slot.u8[8] = f(val); slot.u8[9] = v
fn 021bb848 ret=none
  nowork
  arg buf 0x10
  arg u32 any
  arg u8 any
  stub 021b9b94 ret=u8 args=v

# &slot(idx) if idx < 4 and the slot is used, else 0 (returned as an offset into arg 0)
fn 021bb85c ret=ofs:arg0
  nowork
  arg buf 0x1000
  arg u32 alt range 0 5 || any
  fix arg0 0x1f u8 alt const 255 || any rep=4,12

# &slot(idx) if the slot is used (no bound check on idx)
fn 021bb878 ret=ofs:arg0
  nowork
  arg buf 0x1000
  arg s32 alt range 0 3 || range 0 200
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xC7 u8 alt const 255 || any rep=64,12

# slot(idx) used? (false for idx >= 4)
fn 021bb890 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 5 || any
  stub 021bb830 ret=bool args=v

# u8 at S[0x30 + i]
fn 021bb8a8 ret=u8
  nowork
  arg buf 0x1000
  arg s32 alt range 0 8 || range 0 0xF00

# word at S + b*12 + c*4
fn 021bb8b0 ret=u32
  nowork
  arg buf 0x1000
  arg s32 alt range 0 3 || range 0 300
  arg s32 range 0 5

# S[0x44]
fn 021bb964 ret=u32
  nowork
  arg buf 0x1000

# S[0xCBA] = 0 and S[0xCBF] = 0
fn 021bb920 ret=none
  nowork
  arg buf 0x1000

# append `val` to the u8 list at S+0xCB4 (count at S+0xCBA) unless already in it
fn 021bb930 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || any
  fix arg0 0xCBA u8 alt range 0 6 || any
  fix arg0 0xCB4 u8 range 0 5 rep=6,1

# *(S+0xB0) -> out (2 words)
fn 021babdc ret=none
  nowork
  arg buf 0x1000
  arg buf 8
  fix arg0 0xB0 u32 addrof arg0 0xF00

# set the state function and clear the substate: p[0] = fn, p[1] = 0
fn 021babec ret=none
  nowork
  arg buf 8
  arg u32 any

# tail call: babec(S, 0x021BACAD)
fn 021babf4 ret=none
  nowork
  arg buf 8
  stub 021babec ret=none args=v,v

# returns 1 (no arguments)
fn 021bae5c ret=u32
  nowork

# empty function (bx lr)
fn 021bb56c ret=none
  nowork

# ---- broadcast helpers over the 4 client slots -------------------------------------------------------------

# reset: handle calls are bracketed by 021eeae8/021eeafc; every used slot gets (code, data, size)
fn 021bb724 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021bb830 ret=bool args=v
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none

# bb724(S, code, 0, 0)
fn 021bb718 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  stub 021bb724 ret=none args=v,v,v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none

# one slot: if used { eeb70(handle); eeb00(handle, code, data, count) }; count is a stack argument
fn 021bb768 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021bb830 ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021eeb00 force ret=none args=v,v,v,v

# all used slots ready? (1 unless some used slot's eeb2c answers 0)
fn 021bb798 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021bb830 ret=bool args=v
  stub 021eeb2c force ret=bool args=v

fn 021bb7d0 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021bb830 ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021bb7fc ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021bb830 ret=bool args=v
  stub 021eead0 force ret=none args=v

# ---- the three slot-setup entry points (called from battle_server.s) --------------------------------------
# idx is a client id: slots are S+0x14+idx*12 for idx 0..=3 (S+0x44 is the next field), the callers in battle_server.s
# pass literals 0/1/3 or a value from `do {..} while (i < 4)` loops (sub_021b7e3c), and 021bb85c/021bb890 reject idx >= 4.

# (S, r1, idx, r3): slot(idx) = (idx, r1, 1), slot data = (b9aec(S[0x10], idx), r3)
fn 021baa94 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 range 0 3
  arg u32 any
  stub 021b9aec ret=u32 args=v,v
  stub 021bb840 ret=none args=v,v,v,v
  stub 021bb848 ret=none args=v,v,v
  stub 021b9b94 ret=u8 args=v

# (S, r1, r2, idx, u8 on the stack): like aa94 but the handle comes from eea58(r2, idx, 1, S.u16[0xCC0])
fn 021baacc ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 range 0 3
  arg u8 any
  stub 021b9aec ret=u32 args=v,v
  stub 021eea58 force ret=u32 args=v,v,v,v
  stub 021bb840 ret=none args=v,v,v,v
  stub 021bb848 ret=none args=v,v,v
  stub 021b9b94 ret=u8 args=v

# (S, idx, u8): slot(idx) = (idx, 0, 0), slot data = (b9aec(S[0x10], idx), u8)
fn 021bab10 ret=none
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u8 any
  stub 021b9aec ret=u32 args=v,v
  stub 021bb840 ret=none args=v,v,v,v
  stub 021bb848 ret=none args=v,v,v
  stub 021b9b94 ret=u8 args=v

# ---- teardown ----------------------------------------------------------------------------------------------

# state = 021BAC05, then free everything. 021bb9d0 is a thunk to 021bb9d8
fn 021bab40 ret=none
  nowork
  arg buf 0x1000
  stub 021babec ret=none args=v,v
  stub 021bb7fc ret=none args=v
  stub 021bb9d8 force ret=none args=v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eead0 force ret=none args=v

# ARM stub 02045808(S[0xAC]) (blx), 021bbb48(S[0x44]), then free every slot handle that is used, nonzero and has flag[0xA] == 0
fn 021bab5c ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0x14 u32 alt const 0 || any rep=4,12
  fix arg0 0x1e u8 alt const 0 || any rep=4,12
  stub 02045808 ret=none args=v
  stub 021bbb48 force ret=none args=v
  stub 020307b0 ret=none args=v

# ---- state functions (S, &state) ---------------------------------------------------------------------------

# run the current state function (indirect call through S[0]); on 1 broadcast (9, S[0xB0], 8) and set S[0xED] = 2;
# when S[0xED] == 2 return bb798(S). The state-function stubs are the ROM's own state functions.
fn 021bab9c ret=u32 inputs=1500
  nowork
  arg buf 0x1000
  # S[0] = any state function of this area (Thumb bit set): the ones the others and pick_next_state select
  fix arg0 0 u32 oneof 0x021BAC05 0x021BACAD 0x021BAE61 0x021BAEF9 0x021BB0E1 0x021BB209 0x021BB281 0x021BB321 0x021BB369 0x021BB3B1 0x021BB3F9 0x021BB43D 0x021BB485
  fix arg0 4 u32 range 0 9
  fix arg0 0xED u8 oneof 0 1 2 2 3
  fix arg0 0x48 u32 range 0 8
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  fix arg0 0xCBA u8 alt range 0 6 || range 0 3
  # callees of all the state functions (union of their specs; one stub line per address)
  stub 021bb718 ret=none args=v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021b86b8 force ret=mod:5 args=v
  stub 021bbb60 force ret=u32 args=v
  stub 021babec ret=none args=v,v
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8d8 ret=u32 args=v,out:4,v
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=nullable:2:u32 args=v,v,out:4
  stub 021ee9a4 ret=u32 args=v,out:4
  stub 021bc390 force ret=bool args=v
  stub 021ba5d8 ret=mod:3 args=v
  stub 021ba618 ret=mod:3 args=v
  stub 021bbc34 force ret=none args=v
  stub 021bbc44 force ret=u32 args=v,v
  stub 021bbd20 force ret=none args=v
  stub 021bbd2c force ret=u32 args=v,v
  stub 021b86c0 force ret=u8 args=v
  stub 021b8588 force ret=mod:2 args=v
  stub 021bc55c force ret=u8 args=v
  stub 021b9120 ret=none args=v,v
  stub 021cc668 force ret=u32 args=v
  stub 021b916c ret=none args=v,v
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,outn:4:0xC/0/0/0
  stub 021b8ff4 force ret=u8 args=v
  stub 021bc360 force ret=bool args=v
  stub 021bbd80 force ret=none args=v
  stub 021bbd8c force ret=u32 args=v,v
  stub 021b85a8 force ret=bool args=v
  stub 021b8cfc force ret=bool args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b98 ret=mod:4 args=v
  stub 021d8454 ret=mod:4 args=v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021d5440 ret=u8 args=v
  stub 021ba54c ret=mod:4 args=v
  stub 021b86dc force ret=u16 args=v
  stub 02005d90 ret=none args=v,v
  stub 021b9114 ret=mod:2 args=v
  stub 020362dc ret=u32
  stub 02035c60 ret=u32

fn 021bac04 ret=u32 inputs=400
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 5
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  stub 021bb718 ret=none args=v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021b86b8 force ret=mod:5 args=v
  stub 021bbb60 force ret=u32 args=v
  stub 021bb5fc ret=u32 args=v,v
  stub 021babec ret=none args=v,v
  stub 021bb724 ret=none args=v,v,v,v
  stub 021babf4 ret=none args=v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8d8 ret=u32 args=v,out:4,v

fn 021bacac ret=u32 inputs=600
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 9
  fix arg0 0x48 u32 range 0 8
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  stub 021bc390 force ret=bool args=v
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb718 ret=none args=v,v
  stub 021bb7d0 ret=none args=v
  stub 021ba5d8 ret=mod:3 args=v
  stub 021ba618 ret=mod:3 args=v
  stub 021babec ret=none args=v,v
  stub 021bb6a8 ret=none args=v
  stub 021bae5c ret=u32
  stub 021bb628 ret=bool args=v,v,v
  stub 021bbc34 force ret=none args=v
  stub 021bbc44 force ret=u32 args=v,v
  stub 021b86c0 force ret=u8 args=v
  stub 021bb56c ret=none args=v
  stub 021babf4 ret=none args=v
  stub 021b86b8 force ret=mod:3 args=v
  stub 021b8588 force ret=mod:2 args=v
  stub 021bc55c force ret=u8 args=v
  stub 021b9120 ret=none args=v,v
  stub 021cc668 force ret=u32 args=v
  stub 021b916c ret=none args=v,v
  # 021bb6a8 is ported (real code): its callees need stubs and a data zone
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,outn:4:0xC/0/0/0
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=nullable:2:u32 args=v,v,out:4

fn 021bae60 ret=u32 inputs=400
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  fix arg0 0xD00 u8 any rep=64,4
  stub 021bb718 ret=none args=v,v
  stub 021bb798 ret=bool args=v
  stub 021b8ff4 force ret=u8 args=v
  stub 021bb7d0 ret=none args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,v
  stub 021bc360 force ret=bool args=v
  stub 021bb724 ret=none args=v,v,v,v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021baef8 ret=u32 inputs=600
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 7
  fix arg0 0x48 u32 range 0 7
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb040 ret=bool args=v
  stub 021bb0a0 ret=u8 args=v
  stub 021bb768 ret=none args=v,v,v,v,v
  stub 021bb7d0 ret=none args=v
  stub 021ba618 ret=mod:3 args=v
  stub 021babec ret=none args=v,v
  stub 021bb6a8 ret=none args=v
  stub 021bb628 ret=bool args=v,v,v
  stub 021bbd80 force ret=none args=v
  stub 021bbd8c force ret=u32 args=v,v
  stub 021cc668 force ret=u32 args=v
  stub 021b916c ret=none args=v,v
  stub 021babf4 ret=none args=v
  # 021bb6a8 is ported (real code): its callees need stubs and a data zone
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,outn:4:0xC/0/0/0
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=nullable:2:u32 args=v,v,out:4
  stub 021b85a8 force ret=bool args=v
  stub 021b8cfc force ret=bool args=v,v
  stub 021b8ff4 force ret=u8 args=v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b98 ret=mod:4 args=v
  stub 021d8454 ret=mod:4 args=v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021d5440 ret=u8 args=v

# the 4 spec'd next-state constants of 021bb208/bb280/bb320 are literals in the ROM (021BB281 ...)
fn 021bb208 ret=u32 inputs=300
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021ee9a4 ret=u32 args=v,out:4
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021bb718 ret=none args=v,v
  stub 021cc668 force ret=u32 args=v
  stub 021b916c ret=none args=v,v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

# poll: first call (state 0) returns 1 when the main object says so; state 1 returns 1
# record-exchange state: id list, collect + flush the clients' records (021bb6a8/021bb628/021bb650 real code), reset the
# header at *(S+0xCB0), 021bbd2c picks the step stored in S[0x48] (0..=5, anything else is the default) -- B10-spec2
fn 021bb0e0 ret=u32 inputs=1200
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 alt range 0 6 || oneof 1 3 4 5
  fix arg0 0x48 u32 alt range 0 6 || oneof 4 5
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  fix arg0 0xCBA u8 any
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021ba618 ret=mod:3 args=v
  stub 021babec ret=none args=v,v
  stub 021bbd20 force ret=none args=v
  stub 021bbd2c force ret=u32 args=v,v
  stub 021cc668 force ret=u32 args=v
  stub 021b916c ret=none args=v,v
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,outn:4:0xC/0/0/0
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=nullable:2:u32 args=v,v,out:4

fn 021bb320 ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 2
  stub 021b9114 ret=mod:2 args=v
  stub 020362dc ret=u32
  stub 02035c60 ret=u32

# like 021bb3b0 with broadcast code 0xE (B10-spec2)
fn 021bb368 ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021bb3b0 ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021bb3f8 ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021bb43c ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

fn 021bb484 ret=u32
  nowork
  arg buf 0x1000
  arg buf 0x10
  fix arg1 0 u32 range 0 3
  stub 021bb724 ret=none args=v,v,v,v
  stub 021bb798 ret=bool args=v
  stub 021bb7d0 ret=none args=v
  stub 021b86c0 force ret=u8 args=v
  stub 021babec ret=none args=v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021eeb2c force ret=bool args=v
  stub 021eeb70 force ret=none args=v

# ---- other S methods ---------------------------------------------------------------------------------------

# S+0xAA = ba54c(S[0xC]) (via thunk 021b917c), S+0xA8 = b8ff4(S[0xC]); next state by b86b8(S[0xC]) and that value
fn 021bb280 ret=u32 inputs=300
  nowork
  arg buf 0x1000
  stub 021ba54c ret=mod:4 args=v
  stub 021b8ff4 force ret=u8 args=v
  stub 021b86b8 force ret=mod:5 args=v
  stub 021b86dc force ret=u16 args=v
  stub 02005d90 ret=none args=v,v
  stub 021babec ret=none args=v,v

# all required party members present? (list at S+0xCB4, count S+0xCBA)
fn 021bb040 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0xCBA u8 alt range 0 6 || range 0 3
  stub 021b85a8 force ret=bool args=v
  stub 021b8cfc force ret=bool args=v,v
  stub 021b8ff4 force ret=u8 args=v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b98 ret=mod:4 args=v

# slot 0 (S+0x20): if used and its record says kind 3, b9920(S[0x10], 1, (word >> 7) & 7) and return d5440(that)
fn 021bb0a0 ret=u8
  nowork
  arg buf 0x1000
  fix arg0 0xD00 u32 any rep=64,4
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,v
  stub 021d8454 ret=mod:4 args=v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021d5440 ret=u8 args=v

# pokemon of bpp -> 02017e1c(pk, 0x73, S[0xAC]), then 02027268(S[0xAC])
fn 021bb8bc ret=none
  nowork
  arg buf 0x1000
  arg buf 0x20
  stub 021d5844 ret=u32 args=v
  stub 02017e1c ret=u32 args=v,v,v
  stub 02027268 ret=none args=v

# pokemon of bpp -> 02017e1c(pk, 5, 0) -> 020271dc(u16), then two ba1b4(S[0xC], 7 / 0x54)
fn 021bb8dc ret=none
  nowork
  arg buf 0x1000
  arg buf 0x20
  stub 021d5844 ret=u32 args=v
  stub 02017e1c ret=u32 args=v,v,v
  stub 020271dc ret=none args=v
  stub 021ba1b4 ret=none args=v,v

# tail call 021b9234(S[0xC]); 021bb914's literal is 021B9235
fn 021bb914 ret=none
  nowork
  arg buf 0x1000
  stub 021b9234 ret=none args=v

# ---- record builder / data exchange ------------------------------------------------------------------------

# broadcast (code 10, builder data) after loading ee8d8; S+0x68 = BattleRecordBuilder. returns 1
fn 021bb5fc ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  stub 021ee8b4 ret=none args=v,v
  stub 021ee8d8 ret=u32 args=v,out:4,v
  stub 021bb724 ret=none args=v,v,v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none

# builder init(tag), per used slot AddClientEntries, Finish -> returns the finish result (r0 of 021ee96c)
fn 021bb650 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg buf 8
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021ee8b4 ret=none args=v,v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,out:4
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=u32 args=v,v,v

# bb650 into a local, then broadcast (10, result, local) when the result is nonzero; returns 1 or 0
fn 021bb628 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  stub 021bb650 ret=mod:2 args=v,v,v,out:4
  stub 021bb724 ret=none args=v,v,v,v
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021eeae8 force ret=none
  stub 021eeb00 force ret=none args=v,v,v,v
  stub 021eeafc force ret=none
  stub 021ee8b4 ret=none args=v,v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,out:4
  stub 021ee8f4 ret=none args=v,v,v,v
  stub 021ee96c ret=nullable:2:u32 args=v,v,out:4

# clear the 2-word header at *(S+0xCB0), fetch the mode's data (modes 1-4), then compare: true (1) unless
# header[0] == size and memcmp(header + 8, data, size) says equal (stubs never change header[0], so only size 0 gets there)
fn 021bb570 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 5
  arg u32 any
  arg u32 alt const 0 || any
  fix arg0 0xCB0 u32 addrof arg0 0xF00
  stub 021bbb60 force ret=u32 args=v
  stub 021bbc34 force ret=none args=v
  stub 021bbc44 force ret=u32 args=v,v
  stub 021bbd80 force ret=none args=v
  stub 021bbd8c force ret=u32 args=v,v
  stub 021bbd20 force ret=none args=v
  stub 021bbd2c force ret=u32 args=v,v
  stub 0203efc8 ret=mod:2 args=v,v,v

# per used slot: S.counts[i] = 0, then copy the slot's record words (size >> 2 of them, at most 3 here) to S+0xB4+i*12
fn 021bb6a8 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x1f u8 alt const 255 || any rep=4,12
  fix arg0 0xD00 u32 any rep=64,4
  stub 021bb830 ret=bool args=v
  stub 021eeb64 force ret=ptr:arg0:0xD00:64 args=v,outm:4:0xC/0/0/0

# load a recorded stream (data, size) into the builder and unpack it into S+0xB4 / S+0xE4 until ReadNext says stop.
# ReadNext (seq: the answer changes per call) fills rp (u32), clientID (<= 3), numEntries (<= 3) and 3 entry words;
# the 5th argument (entries) is a stack word of the call
fn 021bb4e0 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  stub 021ee9b4 ret=none args=v,v,v
  stub 021ee9e4 ret=bool seq args=v,out:4,outm:1:3,outm:1:3,out:12
