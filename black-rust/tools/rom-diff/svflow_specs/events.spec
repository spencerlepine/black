# svflow spec: events (0x021CA8CC-0x021CC8D8; B10d, drafted by B10-specs). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (see "B10-specs additions": seq, outm, ret=ptr).
#
# P (the first argument of nearly every function here) is the SvFlowWork image: `arg work`. Fields seen: +0 server,
# +4 main module, +8 pokecon (party), +0xC command queue buffer (put_cmd target), +0x10/+0x14/+0x18 u32 state,
# +0x3E0 (event table), +0x1AB8 / +0x1D74 command-queue helper words (`advance_fields` / `set_word` operate there).
# A BTL_POKEPARAM argument is `arg bpp` with `env bpp` (BattleMon accessors 021d5440-021d6e24 run as real code; the
# Pokemon-level API 02017e1c/e40/d30/d70, 02019c90 etc. is the `env bpp` mock).
#
# Rules used by every function below:
#  * callees ported in cmd_queue (021cc8d8, 021cc95c, 021cd4b0, 021cd4fc, 021cd568, 021cd64c, ...) run as REAL code on
#    both sides, so a function that reaches them gets a command queue: `arg buf 0xC00`, `fix work 0xC addrof` it,
#    `fix argK 0 u32 range 0 2900` and `global 021f3b44 0x40`; `stub 02034714 noreturn` is the fatal routine they use;
#  * the event-variable stack (021d74f8 push, 021d75ec, 021d7630, 021d70f0, 021d775c, 021d7540 pop) and the HANDEX
#    pool (021c88a4 push -> record pointer, 021c88c8 pop) are stubbed; the HANDEX string helpers 021c8838/021c8860 and
#    the pure helpers (soft float 0209xxxx, u32_div, cpu_fill8, same_side 021b9a24, poke_side 021b9a40, 021d7f00,
#    021d80bc, 021e8940, BattleMon accessors) run as real code;
#  * flow-level callees (main/party/target queries 021b86b8..021b9bf0, 021cc668, ...) are stubbed; those that return a
#    BattleMon pointer (021b9934, 021b98ac, 021b9c00) use `ret=ptr:argK:0:1` pointing at an `arg bpp` buffer;
#  * 02xxxxxx veneers `bx pc` (021ca8d8, 021ca8e8, 021ca9d4, ..., 021cc758) are ARM trampolines into overlay code that is
#    not loaded; callers stub them by their veneer address (the real targets are the 0x0689xxxx words after them).
# Argument/stub register use follows the disassembly (see server_side.spec for the "r0 passes through" rule).
area events

# ---- tiny buffer helpers (word table of 24 entries, 021cc8a0..8c8) ---------------------------------------------

# p[0..24) = 6
fn 021cc8a0 ret=none
  nowork
  arg buf 0x60

# p[i] = v
fn 021cc8b0 ret=none
  nowork
  arg buf 0x400
  arg u32 range 0 255
  arg u32 any

fn 021cc8b8 ret=u32
  nowork
  arg buf 0x400
  arg u32 range 0 255

fn 021cc8c0 ret=u32
  nowork
  arg buf 0x400
  arg u32 range 0 255

# *p = 0; MI_CpuFill8(p + 4, 0, 500) (tail call, real code)
fn 021cc8c8 ret=none
  nowork
  arg buf 0x200

# ---- slot lookup helpers ---------------------------------------------------------------------------------------

# arg0 = {+0 server-side object (a), +4 main}. Looks up the client slot of the pokemon id from the main object and
# returns b9bf0(slot[4], index). b8e28 writes (client id <= 3, index) as two bytes; the server-side object is arg1.
# B10-spec2: 021bb85c is real code now (server_side), so the server-side object S lives in the same buffer, at 0x100
# (slots at S+0x14, used flag at slot+0xB; the stub's old ptr:arg0 model no longer applies)
fn 021cb174 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x100
  fix arg0 0x11f u8 range 0 254 rep=4,12
  stub 021b8c50 force ret=u32 args=v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021b9bf0 ret=u32 args=v,v

fn 021cb1a4 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x100
  fix arg0 0x11f u8 range 0 254 rep=4,12
  stub 021b8cb0 force ret=u32 args=v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021b9bf0 ret=u32 args=v,v

# ---- event-variable stack users --------------------------------------------------------------------------------

# push(0x277); set vars from (bpp id, p3.u8[6], p3.u16[0], 0x1f, 0); dispatch(P, 0x29); result = get(4) & 0xFF; pop(0x27F)
fn 021cb214 ret=u32
  arg work
  arg bpp
  arg buf 0x10
  env bpp
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=v,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# like cb214, but the value of var 4 is the 4th argument and the result is 0x1f when it did not change
fn 021cb274 ret=u32
  arg work
  arg bpp
  arg buf 0x10
  arg u32 range 0 3
  env bpp
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=v,v
  stub 021d775c ret=mod:4 args=v
  stub 021d7540 ret=none args=v

# cb274 result, bracketed by cc8d8 / cc95c on the P+0x1D74 header word
fn 021cb1d4 ret=u32
  arg work
  arg bpp
  arg buf 0x10
  arg u32 range 0 3
  env bpp
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=v,v
  stub 021d775c ret=mod:4 args=v
  stub 021d7540 ret=none args=v
  stub 021cb274 ret=u32 args=v,v,v,v

# ---- HANDEX record makers (021c88a4 returns the record pointer, 021c88c8 commits it) -------------------------

# side of the pokemon; if cbff8(side, 3, turn(5)) then record kind 4 {+4 str_param(1, 0x88, side)}
fn 021cbfa0 ret=u32
  arg work
  arg bpp
  env bpp
  stub 021cbff8 ret=bool args=v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# revive: only for a fainted pokemon; record kind 0x2C {+4 id, +6 hp (item param 0x3A: 0xFD = max/4, 0xFE = max/2,
# 0xFF = full, else the value)}, str_param(2, 3, id), then 021c84f4(P, id)
fn 021cc008 ret=u32
  arg work
  arg bpp
  arg u32 any
  fix arg1 0xE u16 oneof 50 100 200
  fix arg1 0x10 u16 alt const 0 || oneof 50 100 200 || any
  env bpp
  stub 021d805c ret=pick:0xfd/0xfe/0xff/0x20 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v
  stub 021c84f4 force ret=none args=v,v

# raise stat/flag via put_cmd (real cmd_queue code on the queue in P[0xC]): (P, bpp, r2, amount)
fn 021cc0f8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 range 0 4
  arg buf 0xC00
  fix work 0xC u32 addrof arg4
  fix arg4 0 u32 range 0 2900
  global 021f3b44 0x40
  env bpp
  stub 02034714 noreturn ret=none args=v,v,v

# PP restore of one move slot (5th argument): record kind 9 {+4 pp, +5 id, +6 slot, +7 |= 2, str_param(2, 0x186), id, move id}
fn 021cc178 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  arg u32 range 0 5
  env bpp
  stub 021d805c ret=pick:0x7f/0x05/0x20/0 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# PP restore of every move
fn 021cc214 ret=u32
  arg work
  arg bpp
  arg u32 any
  env bpp
  stub 021d805c ret=pick:0x7f/0x05/0x20/0 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# heal: not at full hp and not fainted; record kind 5 {+4 hp (param 0x3A: FD = max/4, FE = max/2, FF = max-hp stat, else value)}
fn 021cc2c0 ret=u32
  arg work
  arg bpp
  arg u32 any
  fix arg1 0xE u16 oneof 50 100 200
  fix arg1 0x10 u16 alt const 0 || oneof 50 100 200 || any
  env bpp
  stub 021d805c ret=pick:0xfd/0xfe/0xff/0x20 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# cure one status (5th argument = sick id): record kind 0xB {+4 sick, +8 id, +0x14 = 1}
fn 021cc350 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  arg u32 range 0 8
  fix arg1 0xE u16 oneof 50 100 200
  fix arg1 0x10 u16 alt const 0 || oneof 50 100 200 || any
  env bpp
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# cure every status in a veneer-provided list (cc428(i) = i-th sick id, 0 ends it)
fn 021cc398 ret=u32 inputs=300
  arg work
  arg bpp
  fix arg1 0xE u16 oneof 50 100 200
  fix arg1 0x10 u16 alt const 0 || oneof 50 100 200 || any
  env bpp
  stub 021cc418 ret=mod:2 args=v,v
  stub 021cc428 ret=mod:4 args=v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# stat stage change: veneer 021cc49c(P+0x1AB8, id) == 6 blocks it; can_change_stage(bpp, stat (5th), amount (r3))
fn 021cc438 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  arg u32 range 1 7
  fix arg1 0xE u16 oneof 50 100 200
  fix arg1 0x10 u16 alt const 0 || oneof 50 100 200 || any
  env bpp
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# held-item based check: veneer 021cc4f8(item) says "blocked"; else 021c84ac(P, bpp, &flag) -> 1, or 2 if flag is set
fn 021cc4ac ret=u32
  arg work
  arg bpp
  env bpp
  stub 021cc4f8 ret=bool args=v
  stub 021c84ac force ret=bool args=v,v,outm:1:1

# unless the pokemon's stat 0x10 is 150: 021bdbb8(P, bpp); returns 1 when 021c88e4(P) == 2
fn 021cc508 ret=u32
  arg work
  arg bpp
  env bpp
  stub 021bdbb8 force ret=none args=v,v
  stub 021c88e4 force ret=mod:3 args=v

fn 021cc558 ret=u32
  arg work
  arg bpp
  env bpp
  stub 021e869c ret=bool args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc5f4 ret=u32
  arg work
  arg bpp
  env bpp
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# ---- item-effect table wrappers (pool 0x021EFDE4): fixed 5th argument, then the shared body ---------------------

fn 021cbf30 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cbf40 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cbf50 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cbf60 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cbf70 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cbf80 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc350 ret=u32 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# cc398 takes (P, bpp) only; the constant 7 on the stack is dead
fn 021cbf90 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc398 ret=u32 args=v,v
  stub 021cc418 ret=mod:2 args=v,v
  stub 021cc428 ret=mod:4 args=v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc098 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc0a8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc0b8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc0c8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc0d8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

fn 021cc0e8 ret=u32
  arg work
  arg bpp
  arg u32 any
  arg u32 any
  env bpp
  stub 021cc438 ret=u32 args=v,v,v,v,v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v

# ---- who is ahead? (battle-end / forfeit style comparison; cc668 -> cc768 -> cc800) --------------------------------

# a = {u8 x, u8 y} (arg1), b = {u8 x, u8 y} (arg2): compares the differences, then cc800 totals for sides 0 / 1; 1 when the
# comparison result equals `side`, 2 when everything is equal
# B10-spec2: 021cc800 is ported (real code), so arg4 is the BattleMon image 021b9c00 points into and `env bpp` supplies the
# BattleMon API (same model as the 021cc800 spec below)
fn 021cc768 ret=u32
  arg work
  arg buf 4
  arg buf 4
  arg u32 range 0 1
  arg bpp
  fix arg1 0 u8 range 0 3 rep=2,1
  fix arg2 0 u8 range 0 3 rep=2,1
  fix arg4 0xE u16 range 1 300
  fix arg4 0x10 u16 range 0 300
  env bpp
  stub 021b8b9c force ret=bool args=v,v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b94 ret=mod:4 args=v
  stub 021b9c00 ret=ptr:arg4:0:1 args=v,v

# totals of (current hp, max hp) over the party of every client of side `which`: out1 = sum hp, out2 = 100 * hp / max hp.
# arg4 is only the BattleMon image the member lookup (021b9c00) points into
fn 021cc800 ret=none
  arg work
  arg buf 4
  arg buf 4
  arg u32 range 0 1
  arg bpp
  fix arg4 0xE u16 range 1 300
  fix arg4 0x10 u16 range 0 300
  env bpp
  stub 021b8b9c force ret=bool args=v,v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b94 ret=mod:4 args=v
  stub 021b9c00 ret=ptr:arg4:0:1 args=v,v

# per client: party size and alive count by side; the side of 021b8ff4's client is compared with the other side;
# ties go to 021ba534 / the veneer 021cc758 (P+0x3E0 table) / cc768
fn 021cc668 ret=u32 inputs=400
  arg work
  arg bpp
  fix arg1 0xE u16 range 1 300
  fix arg1 0x10 u16 range 0 300
  env bpp
  stub 021b9c00 ret=ptr:arg1:0:1 args=v,v
  stub 021b8b9c force ret=pick:1/1/1/0 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b94 ret=pick:1/2/3 args=v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9b98 ret=pick:0/0/0/1/2/3 args=v
  stub 021b8ff4 force ret=u8 args=v
  stub 021ba534 ret=pick:1/1/1/0 args=v
  stub 021cc758 ret=pick:0x1f/3/7/0/11/12/13 args=v
  # 021cc768 and its callee 021cc800 are ported (real code): arg1 is the BattleMon image 021b9c00 points into

# ---- capture / experience arithmetic and the item-use master (cb2d4 .. cbea8, cba40, cb730) -------------------------

# ball bonus by the main object's value (021b873c(P[4]) thresholds 30/150/300/450/600), fixed-point 4.12
fn 021cbcf4 ret=u32
  arg work
  stub 021b873c force ret=pick:0/20/100/200/400/500/700 args=v

# catch-rate multiplier of the 021cbd48 kind (selector r3 < 16)
fn 021cbea8 ret=u32
  arg work
  arg s32 alt range 0 0x1000 || range 0 0xFF000 || any
  stub 021b873c force ret=pick:0/20/100/200/400/500/700 args=v
  stub 021d78ac ret=pick:0/1/0x7f/0x80/0xff/0x100/0x1ff args=v

# ball multiplier by ball id (r3 = 0..15): fixed values, or from type of the target (has_type), from the main object
# (021ba1a8(main, 1), 021b8714(main) -> struct, 021b8724(main, bpp)), from the target's level (get(0xF)) with soft float, ...
fn 021cbd48 ret=u32 inputs=500
  arg work
  arg u32 any
  arg bpp
  arg u32 range 0 17
  arg buf 0x100
  fix work 0x10 u32 alt range 0 8 || any
  fix arg4 0x40 u32 alt oneof 4 5 6 || range 0 8 rep=8,4
  fix arg2 0x18 u8 alt range 0 40 || const 41 || any
  env bpp
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021b8714 force ret=ptr:arg4:0x40:8 args=v
  stub 021b8724 force ret=pick:0/1 args=v,v
  stub 020113f0 ret=pick:0/1/2/3/4 args=v,v
  stub 021cbe98 ret=pick:0/1 args=v

# experience scale: 2*lvl_b + 10 etc. through soft float and 0207c7b4 (integer sqrt-like), clamped to the exp-for-max-level
# of the pokemon (r0); (bpp, exp, level a, level b)
fn 021cb55c ret=u32
  arg bpp
  arg u32 any
  arg u32 alt range 0 100 || const 0xFFFFFFFB
  arg u32 alt range 0 100 || const 0xFFFFFFFB
  env bpp
  stub 0207c7b4 ret=u32 args=v

# EV style update from the table at 021EFDC0 (6 stats x {u8 param, u16 species}): (bpp of the winner, bpp of the loser, 12-byte entry)
fn 021cb5f0 ret=none
  arg bpp
  arg bpp
  arg buf 0x10
  fix arg0 0x12 u16 alt oneof 215 289 290 291 292 293 294 || any
  env bpp
  stub 021d805c ret=pick:0/1/2/3/5 args=v,v

# experience distribution: out = 6 x {u32 exp, u8 flag, ...} (12 bytes each) from the fainted pokemon (arg2), the party
# `r1` and the participants (faced list of arg2); 021cb55c / 021cb5f0 are the two area helpers
fn 021cb2d4 ret=none inputs=300
  arg work
  arg u32 any
  arg bpp
  arg buf 0x48
  arg bpp
  fix arg4 0x12 u16 alt oneof 216 231 || any
  fix arg4 0x10 u16 alt const 0 || range 1 400
  env bpp
  stub 021b9b94 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021b9c00 ret=ptr:arg4:0:1 args=v,v
  stub 021b9934 ret=ptr:arg4:0:1 args=v,v
  stub 021b871c force ret=u32 args=v
  stub 021b9bf0 ret=ptr:arg4:0:1 args=v,v
  stub 02008564 ret=u32 args=v
  stub 02018480 ret=pick:0/1 args=v,v
  stub 02011168 ret=u32 args=v
  stub 021cb55c ret=u32 args=v,v,v,v
  stub 021cb5f0 ret=none args=v,v,v
  stub 0207c7b4 ret=u32 args=v
  stub 021d805c ret=pick:0/1/2/3/5 args=v,v

# ---- message argument builders (HANDEX string parameters through ARM veneers) -------------------------------------
# The veneers 021ca9d4 .. 021cafc8 are stubbed by address (obj, value); `obj` is the output record pointer, opaque here.

# (P, bpp, -, rec, value, out obj): rec[0xC] = message kind 0..13; returns the number of arguments pushed (0, 1 or 2)
fn 021ca8f8 ret=u32 inputs=400
  arg work
  arg bpp
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || const 0x1f
  arg u32 any
  fix arg3 0xC u32 alt range 0 13 || any
  fix work 0 u32 addrof work 0x1000
  fix work 0x101f u8 range 0 254 rep=4,12
  env bpp
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021cb174 ret=u32 args=v
  stub 021b9934 ret=u32 args=v,v
  stub 021ca9d4 ret=none args=v,v
  stub 021ca9e4 ret=none args=v,v
  stub 021ca9f4 ret=none args=v,v
  stub 021caa04 ret=none args=v,v
  stub 021caa14 ret=none args=v,v
  stub 021b8c50 force ret=u32 args=v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021b9bf0 ret=u32 args=v,v

# target id override for the "Pressure"-like record (rec u16[0] == 0xAE): (P, rec, -, id, expected); ret = id or expected
fn 021caa24 ret=u32
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 range 0 8
  arg u32 range 0 8
  fix arg1 0 u16 alt const 0xAE || any
  fix arg1 0xC u32 alt oneof 0 7 || any
  stub 021d80bc ret=u32 args=v,v,v,v,v

# like 021ca8f8 with the other message kinds; party lookups (021b98ac / 021b9934) return a BattleMon, which feeds the
# real accessors; arg6 is that BattleMon image
fn 021caa58 ret=u32 inputs=600
  arg work
  arg bpp
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || const 0x1f
  arg u32 any
  arg bpp
  fix arg3 0xC u32 alt range 0 13 || any
  fix work 0 u32 addrof work 0x1000
  fix work 0x101f u8 range 0 254 rep=4,12
  env bpp
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021caa24 ret=pick:0/1/2/3/6/0x1f args=v,v,v,v,v
  stub 021b98ac ret=ptr:arg6:0:1 args=v,v
  stub 021b9934 ret=ptr:arg6:0:1 args=v,v
  stub 021d78ac ret=pick:0/1 args=v
  stub 021cb174 ret=nullable:4:ptr:arg6:0:1 args=v
  stub 021cb1a4 ret=nullable:4:ptr:arg6:0:1 args=v
  stub 021cb1d4 ret=pick:0x1f/0/1/2/3 args=v,v,v,v
  stub 021cac24 ret=none args=v,v
  stub 021cac34 ret=none args=v,v
  stub 021cac44 ret=none args=v,v
  stub 021cac54 ret=none args=v,v
  stub 021cac64 ret=none args=v,v
  stub 021cac74 ret=none args=v,v
  stub 021cac84 ret=none args=v,v
  stub 021cac94 ret=none args=v,v
  stub 021caca4 ret=none args=v,v
  stub 021cacb4 ret=none args=v,v
  stub 021cacc4 ret=none args=v,v
  stub 021cacd4 ret=none args=v,v
  stub 021cace4 ret=none args=v,v
  stub 021d80bc ret=u32 args=v,v,v,v,v
  stub 021b8c50 force ret=u32 args=v
  stub 021b8cb0 force ret=u32 args=v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021b9bf0 ret=nullable:4:ptr:arg6:0:1 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=v,v
  stub 021d775c ret=mod:4 args=v
  stub 021d7540 ret=none args=v

# message arguments for the 4-party style (doubles / triples): 021b99e0 returns a list {u16 n0, u16 n1, u8 ids[...]}
# (arg7 holds it); every party lookup returns the BattleMon of arg6
fn 021cacf4 ret=u32 inputs=700
  arg work
  arg bpp
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || const 0x1f
  arg u32 any
  arg bpp
  arg buf 0x40
  fix arg3 0xC u32 alt range 0 13 || any
  fix arg7 0 u16 range 0 6
  fix arg7 2 u16 range 0 6
  fix work 0 u32 addrof work 0x1000
  fix work 0x101f u8 range 0 254 rep=4,12
  env bpp
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b99e0 ret=ptr:arg7:0:1 args=v
  stub 021caa24 ret=pick:0/1/2/3/6/0x1f args=v,v,v,v,v
  stub 021b98ac ret=nullable:5:ptr:arg6:0:1 args=v,v
  stub 021b9934 ret=nullable:5:ptr:arg6:0:1 args=v,v
  stub 021d78ac ret=pick:0/1/2/3 args=v
  stub 021b8c38 force ret=pick:0/1/2 args=v,v
  stub 021cb174 ret=nullable:4:ptr:arg6:0:1 args=v
  stub 021cb1d4 ret=pick:0x1f/0/1/2/3 args=v,v,v,v
  stub 021caf18 ret=none args=v,v
  stub 021caf28 ret=none args=v,v
  stub 021caf38 ret=none args=v,v
  stub 021caf48 ret=none args=v,v
  stub 021caf58 ret=none args=v,v
  stub 021caf68 ret=none args=v,v
  stub 021caf78 ret=none args=v,v
  stub 021caf88 ret=none args=v,v
  stub 021caf98 ret=none args=v,v
  stub 021cafa8 ret=none args=v,v
  stub 021cafb8 ret=none args=v,v
  stub 021cafc8 ret=none args=v,v
  stub 021d80bc ret=u32 args=v,v,v,v,v
  stub 021b8c50 force ret=u32 args=v
  stub 021b8cb0 force ret=u32 args=v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021b9bf0 ret=nullable:4:ptr:arg6:0:1 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=v,v
  stub 021d775c ret=mod:4 args=v
  stub 021d7540 ret=none args=v

# (P, mon id, target bpp, u16 *move, u8 distance, out obj): picks the best of up to 3 candidate allies from 021b8778(main,
# flags, bytes[8]); veneers 021cb134 (obj), 021cb144 (obj, 0) -> BattleMon, 021cb154 (obj, mon), 021cb164 (obj)
fn 021cafd8 ret=u32 inputs=500
  arg work
  arg u32 range 0 4
  arg bpp
  arg buf 4
  arg u32 range 0 8
  arg u32 any
  arg bpp
  arg bpp
  fix arg3 0 u16 any
  fix arg6 0x10 u16 alt const 0 || any
  fix arg7 0x10 u16 alt const 0 || range 1 300
  env bpp
  stub 021cb134 ret=pick:0/1/1 args=v
  stub 021cb144 ret=ptr:arg6:0:1 args=v,v
  stub 021cb154 ret=none args=v,v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021b8778 force ret=pick:0/1/2/3 args=v,v,outm:8:3
  stub 021b98ac ret=ptr:arg7:0:1 args=v,v
  stub 021d78ac ret=pick:0/1 args=v
  stub 021cb164 ret=none args=v

# ---- capture roll and item use (cbb34, cba40, cb730) ---------------------------------------------------------------

# capture check for ball r3 on the target (arg2): out4 = shake count (3 for the master ball, r3 == 1), out5 = flag.
# 020111bc/0207c768/0207c7b4 are ARM9 math helpers, 021d78ac the battle RNG (rand(range)); the 64-bit multiply 0209c054, the
# soft float and u32_div run as real code
fn 021cbb34 ret=u32 inputs=600
  arg work
  arg u32 any
  arg bpp
  arg u32 alt range 0 17 || const 1
  arg buf 4
  arg buf 4
  fix arg2 0xE u16 range 1 300
  fix arg2 0x10 u16 range 1 300
  env bpp
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021cbcf4 ret=pick:0x1000/0x2000/0x3000/0x800 args=v
  stub 021cbd48 ret=pick:0x1000/0x2000/0x3000/0x800/0x4800 args=v,v,v,v
  stub 020111bc ret=pick:0/0x1000/0x80000/0xff000/0x100000 args=v
  stub 021cbea8 ret=pick:0/1/2/3 args=v,v
  stub 0207c768 ret=u32 args=v,v
  stub 0207c7b4 ret=u32 args=v
  stub 021d78ac ret=pick:0/100/0x8000/0xffff args=v
  stub 021b873c force ret=pick:0/20/100/200/400/500/700 args=v
  stub 021b8714 force ret=ptr:arg2:0x40:8 args=v
  stub 021b8724 force ret=pick:0/1 args=v,v
  stub 020113f0 ret=pick:0/1/2/3/4 args=v,v
  stub 021cbe98 ret=pick:0/1 args=v

# throw a ball at the first alive target of the opponent side: (P, thrower bpp, ball); target = arg4's BattleMon
# (b98ac returns it); the put_cmd calls run the real command queue in P[0xC] (arg3)
fn 021cba40 ret=u32 inputs=500
  arg work
  arg bpp
  arg u32 any
  arg buf 0xC00
  arg bpp
  fix work 0xC u32 addrof arg3
  fix arg3 0 u32 range 0 2900
  fix arg4 0x10 u16 alt const 0 || range 1 300
  global 021f3b44 0x40
  env bpp
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8778 force ret=pick:0/1/2/3/4/5/6 args=v,v,out:6
  stub 021b98ac ret=ptr:arg4:0:1 args=v,v
  stub 021b86b8 force ret=pick:0/1 args=v
  stub 021cbb34 ret=pick:0/1 args=v,v,v,v,out:1,out:1
  stub 021b8724 force ret=pick:0/1 args=v,v
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021b873c force ret=pick:0/20/100/200/400/500/700 args=v
  stub 021b8714 force ret=ptr:arg4:0x40:8 args=v
  stub 020113f0 ret=pick:0/1/2/3/4 args=v,v
  stub 021cbe98 ret=pick:0/1 args=v
  stub 020111bc ret=pick:0/0x1000/0x80000/0xff000/0x100000 args=v
  stub 0207c768 ret=u32 args=v,v
  stub 0207c7b4 ret=u32 args=v
  stub 021d78ac ret=pick:0/100/0x8000/0xffff args=v

# use an item on a pokemon: (P, bpp (user), item, -, party slot): dispatches through the 23-entry table 021EFDE4
# (id, flag, handler); handlers 021cbf30.. are `force` stubs by address (P, mon, item, extra, p4): B10-spec2, they are ported and
# verified on their own, and run as real code they would read a null BattleMon whenever slot == 6 (a reachable input: the caller 021bcce0 passes
# (word & 0x7f) >> 4, 0..7, and the code special-cases 6), which no port can reproduce. The slot range stays 0..6. Commands go to the real queue
# (arg5); arg6 is the BattleMon b9bf0 returns
fn 021cb730 ret=u32 inputs=800
  arg work
  arg bpp
  arg u32 alt oneof 0x19 0x1e 0x1f 0x20 0x21 0x22 0x23 0x24 0x265 0x263 0x264 0x266 || range 0 0x270
  arg u32 any
  arg u32 range 0 6
  arg buf 0xC00
  arg bpp
  fix work 0xC u32 addrof arg5
  fix work 0x18 u32 alt const 1 || range 0 3
  fix arg5 0 u32 range 0 2900
  fix arg6 0x10 u16 alt const 0 || range 1 300
  global 021f3b44 0x40
  env bpp
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9aec ret=u32 args=v,v
  stub 021b6994 force ret=pick:1/2/3/4 args=v,v
  stub 021b9bf0 ret=ptr:arg6:0:1 args=v,v
  stub 021cba30 ret=pick:6/0/1/2 args=v,v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021d805c ret=pick:0/4/3/1/2/5 args=v,v
  stub 021cba40 ret=pick:0/1 args=v,v,v
  stub 021b9090 ret=none args=v,v,v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021ba1b4 ret=none args=v,v
  stub 021c8338 force ret=pick:0/1 args=v,v
  stub 021ba250 ret=none args=v,v,v
  stub 021cbf30 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf40 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf50 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf60 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf70 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf80 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbf90 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cbfa0 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc008 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc098 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0a8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0b8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0c8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0d8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0e8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc0f8 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc178 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc214 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc2c0 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc4ac force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc508 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc558 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021cc5f4 force ret=pick:0/1/2/1 args=v,v,v,v,v
  stub 021c88a4 force ret=ptr:work:0x1800:1 args=v,v,v
  stub 021c88c8 force ret=none args=v,v
  stub 021cc418 ret=mod:2 args=v,v
  stub 021cc428 ret=mod:4 args=v
  stub 021cc49c ret=mod:7 args=v,v
  stub 021cbff8 ret=bool args=v,v,v
  stub 021c84f4 force ret=none args=v,v
  stub 021cc4f8 ret=bool args=v
  stub 021c84ac force ret=bool args=v,v,outm:1:1
  stub 021bdbb8 force ret=none args=v,v
  stub 021c88e4 force ret=mod:3 args=v
  stub 021e869c ret=bool args=v,v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8778 force ret=pick:0/1/2/3/4/5/6 args=v,v,out:6
  stub 021b98ac ret=ptr:arg6:0:1 args=v,v
  stub 021b86b8 force ret=pick:0/1 args=v
  stub 021b8724 force ret=pick:0/1 args=v,v
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021b873c force ret=pick:0/20/100/200/400/500/700 args=v
  stub 021b8714 force ret=ptr:arg6:0x40:8 args=v
  stub 020113f0 ret=pick:0/1/2/3/4 args=v,v
  stub 021cbe98 ret=pick:0/1 args=v
  stub 020111bc ret=pick:0/0x1000/0x80000/0xff000/0x100000 args=v
  stub 0207c768 ret=u32 args=v,v
  stub 0207c7b4 ret=u32 args=v
  stub 021d78ac ret=pick:0/100/0x8000/0xffff args=v
