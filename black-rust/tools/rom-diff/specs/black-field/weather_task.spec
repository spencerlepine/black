# fndiff spec (F11): weather_task.c (overlay 21 Thumb code 0x021A1DB4..0x021A2D88, the second part of F1 row 21.18) minus its particle
# system: the weather task. A weather (overlay 0x18..0x1C) is a data block of callbacks the task runs as a state machine (start ->
# resources -> fade in -> run -> fade out -> end), plus the API the weather overlays call: the light (field_light), the fog
# (field_fog), the zone fog, the 3D backgrounds, a value stepper with a callback, and the weather sound effect.
# File bound: `Heap_AllocDebug(heap, 0x784, 1, "weather_task.c", 0x17E / 0x874)` in 021A1DB4 / 021A28CC (string 0x021DDE80);
# weather.c ends at 0x021A1DB4, fieldmap_ctrl_grid.c starts at 0x021A2D88. Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-field weather_task --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry: 92
# functions (most API entries are called only from the weather overlays, outside the field set; 26 are thunks `ldr r3, =X; bx r3`,
# some after loading their first argument). 61 are specced. Not specced (31, the particle system: D3, U-phase / G8; reached from the
# specced functions only as in-area stubs): the particle pool (32 x 0x38 at +0x74, list head +0x774, count +0x778) 021A21BC count,
# 021A21C8 new, 021A21E0 delete, 021A2254 burst, 021A274C / 021A277C / 021A27B8 / 021A27F8 pool and per-frame update, 021A2BB8 /
# 021A2BC4 / 021A2BD4 / 021A2C10 / 021A2C24 / 021A2C28 / 021A2630 / 021A2638 sprite objects, 021A2640 / 021A2654 / 021A26C4 positions,
# 021A2424 / 021A244C / 021A2564 / 021A2D30 the camera scroll of the particles (fx32); their sprite resources 021A2978 / 021A2994 /
# 021A29A8 / 021A29F4 (OAM character / palette / cell of ARC +0x20->+0), and the 3D-background resources 021A2860 / 021A28A0 /
# 021A2A18 / 021A2A68 (field_3dbg, G8).
#
# TASK (0x784, 021A1DB4): +0 the sprite system, +4 the FIELD FOG, +8 the LIGHT (field_light.c), +0xC the sound handle, +0x10 the ZONE
# FOG, +0x14 the camera, +0x18 the WEATHER (its two 3D backgrounds at +0 / +4), +0x1C u16 heap, +0x20 the weather data (in the weather
# overlay: +2 u16 particles on, +4 u16 3D-background count, +0x28 work size, +0x2C..+0x44 the state callbacks start / fade in /
# fade in (no particles) / run / fade out 1 / fade out 2 / end, +0x48 the particle callback), +0x24 u8 {low nibble kind, high nibble
# fade-out flag}, +0x25 u8 start mode (0 all at once, 1 in steps), +0x26 u8 state 0..12 (021A273C: status by state, ROM 0x021D88E8 =
# {0, 1, 1, 1, 1, 2, 2, 1, 3, 4, 4, 5, 5, 0}), +0x27 u8 resources ready, +0x28 the sprite resources (0x14), +0x3C the weather's work
# (021A28CC), +0x40 SOUND {u16 state (1 playing, 2 waiting for the BGM system), u16 SE, +8 handle mode}, +0x4C STEPPER (0x1C: +0
# object, +4 s16 value, +6 s16 countdown, +8 s16 period, +0xA s16 count, +0xC s16 value end, +0xE s16 period end, +0x10 s16 period
# step, +0x12 s16 count period, +0x14 s32 value step, +0x18 callback(object, value, x)), +0x74 the particles, +0x77C FOG FADE {u8 on,
# u8 a, u8 frames / 2, u32 b}.
#
# Out-of-area callees are `force` stubs: field_light (021A0590 / 021A05D8 / 021A0618 / 021A063C / 021A065C), field_fog (G8: 021A01A4
# on, 021A01AC / 021A01B4 / 021A01BC / 021A01CC settings, 021A01D4 a byte, 021A01D8 apply, 021A01FC / 021A0224 fades, 021A0244 busy),
# field_zonefog (021A0354, 021A0380 / 021A0390 / 021A03A4 / 021A03B8 / 021A03CC), field_3dbg setters (021A16D4 / 021A16DC / 021A16E0 /
# 021A16E4 / 021A16E8), the camera 0218E188, the sound system (02005C64 BGM busy, 020061E4 / 020061F0 / 02006148 / 020061F8 /
# 02028A7C / 02028ACC / 02028B50). The weather callbacks (data +0x2C..+0x44, called through 021A2724) are FAKE code at 0x02EE0011 (or
# 0, no callback) with a stub at 0x02EE0010; the stepper callback (+0x18) is FAKE 0x02EE0021 (stub 0x02EE0020).
# All blocks are `nowork`; a TASK is `arg buf 0x800` with the WEATHER's background array at +0x790 (+0x18 points there) and the
# weather data at +0x7A0 (+0x20 points there); state 0..13. Closure stubs (as core4 / F5b): a block lists the stubs of its callees and
# of the callees of its in-area descendants (checked with every specced function treated as ported: no fault, stray write or
# unstubbed callee).
area weather_task
set field

# 021A1DB4(spr, cam?, light, fog, zonefog, W, snd, heap16, heap) -> TASK: T = Heap_AllocDebug((u16) heap, 0x784, 1, "weather_task.c",
# 0x17E); +0 spr, +0x14 the 2nd argument, +8 light, +4 fog, +0x10 zonefog, +0x18 W, +0xC snd, +0x1C (u16) the 8th argument. Arena: arg 9.
fn 021a1db4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x800
  stub 02030734 force ret=ptr:arg9:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 021A1DF8(T): 021A20C8(T) (end); Heap_Free(T).
fn 021a1df8 ret=none inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a20c8 ret=none args=v  # in-area: end now
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 021a2724 ret=pick:0/1 args=v,v,v  # in-area: callback (T, cb, x)
  stub 021a2924 ret=none args=v  # in-area: free work + particles
  stub 021a28a0 ret=none args=v  # in-area, not specced: 3D-background resources free
  stub 021a284c ret=none args=v  # in-area: sprite resources free
  stub 021a26d8 ret=none args=v  # in-area: reset
  stub 02ee0010 force ret=pick:0/1 args=v,v,v  # FAKE weather callback (T, kind, x)
  stub 021a2c10 ret=none args=v  # in-area, not specced: particle sprite delete
  stub 021a2bb8 ret=none args=v  # in-area, not specced: particle clear
  stub 021a25cc ret=none args=v  # in-area: SE stop (T)
  stub 021a29f4 ret=none args=v  # in-area, not specced: sprite resources free (res)
  stub 021a2994 ret=none args=v  # in-area, not specced: sprite archive close (res)
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A1E08(T, heap) (one frame): a pending fog fade (+0x77C) with the fog idle (021A0244 == 0) -> 021A2D04(T), 021A2CEC(T). By state:
# 1 -> 021A282C (sprite resources), 021A2860 (3D backgrounds), 021A28CC (work + particles), ready flag, 5; 2 / 3 / 4 the same in three
# frames, then 5; 5 -> callback +0x2C (start) done -> 6 (fade-out flag) / 7; 6 / 7 -> callback +0x30 / +0x34 (fade in) done -> 8; 8 ->
# callback +0x38 (run); 9 / 10 -> callbacks +0x3C / +0x40 (fade out) done -> 10 / 11; 11 -> callback +0x44 (end) when ready, 12; 12 ->
# 021A2924 / 021A28A0 / 021A284C (free), 021A26D8 (reset), 0. States 6 / 8 / 9 / 10 update the particles (021A27F8); 5..10 resume the
# sound (021A2C98).
fn 021a1e08 ret=none inputs=600
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  fix arg0 0x26 u8 alt range 0 13 || any
  stub 021a2cf8 ret=pick:0/1 args=v  # in-area: fog fade pending
  stub 021a0244 force ret=pick:0/1 args=v  # field_fog (G8): busy
  stub 021a2d04 ret=none args=v  # in-area: fog fade apply
  stub 021a2cec ret=none args=v  # in-area: fog fade clear
  stub 021a282c ret=none args=v,v  # in-area: sprite resources (T, heap)
  stub 021a2860 ret=none args=v,v  # in-area, not specced: 3D-background resources (T, heap)
  stub 021a28cc ret=none args=v,v  # in-area: work + particles (T, heap)
  stub 021a2968 ret=none args=v  # in-area: ready flag
  stub 021a2724 ret=pick:0/1 args=v,v,v  # in-area: callback (T, cb, x)
  stub 021a2c98 ret=none args=v,v  # in-area: SE resume (SND, h)
  stub 021a27f8 ret=none args=v  # in-area, not specced: particles frame
  stub 021a2970 ret=pick:0/1 args=v  # in-area: ready?
  stub 021a2924 ret=none args=v  # in-area: free work + particles
  stub 021a28a0 ret=none args=v  # in-area, not specced: 3D-background resources free
  stub 021a284c ret=none args=v  # in-area: sprite resources free
  stub 021a26d8 ret=none args=v  # in-area: reset
  stub 021a0224 force ret=none args=v,v,v,v,v,v  # field_fog (G8): fade (F, a, b, c, d, e)
  stub 021a2978 ret=none args=v,v,v  # in-area, not specced: sprite archive open (res, data, heap)
  stub 021a29a8 ret=none args=v,v,v  # in-area, not specced: sprite resources load (res, data, heap)
  stub 02030734 force ret=ptr:arg0:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 021a2bd4 ret=none args=v,v,v,v,v  # in-area, not specced: particle sprite new (p, T, res, spr, heap)
  stub 02ee0010 force ret=pick:0/1 args=v,v,v  # FAKE weather callback (T, kind, x)
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028a7c force ret=none args=v,v  # sound: play SE on a handle (h, se)
  stub 020061e4 force ret=none args=v  # sound: play SE
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 021a2c10 ret=none args=v  # in-area, not specced: particle sprite delete
  stub 021a2bb8 ret=none args=v  # in-area, not specced: particle clear
  stub 021a25cc ret=none args=v  # in-area: SE stop (T)
  stub 021a29f4 ret=none args=v  # in-area, not specced: sprite resources free (res)
  stub 021a2994 ret=none args=v  # in-area, not specced: sprite archive close (res)
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A1FEC(T, data, mode, fadeout, kind, snd mode, -): +0x20 = data, +0x24 = (fadeout & 0xF) << 4 | (kind & 0xF), +0x25 = mode; mode 0
# -> state 1, 1 -> 2; 021A2C2C(T+0x40, snd mode).
fn 021a1fec ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 alt range 0 2 || any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2c2c ret=none args=v,v  # in-area: SND mode

# 021A2050(T, fadeout, kind) (stop): status s = 021A273C(T); state 0 or s 4 / 5 (stopping) -> nothing; else +0x24 = (fadeout & 0xF)
# << 4 | (kind & 0xF); state <= 5 -> 11; else fade-out flag -> 9, else 11.
fn 021a2050 ret=none inputs=300
  nowork
  arg buf 0x800
  arg u32 alt range 0 2 || any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a273c ret=pick:0/1/2/3/4/5 args=v  # in-area: status

# 021A20C8(T) (end now): state 0 -> nothing; state <= 11 -> callback +0x44 (021A2724(T, cb, 0)); state <= 12 -> 021A2924, 021A28A0,
# 021A284C; 021A26D8 (reset).
fn 021a20c8 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  fix arg0 0x26 u8 alt range 0 13 || any
  stub 021a2724 ret=pick:0/1 args=v,v,v  # in-area: callback (T, cb, x)
  stub 021a2924 ret=none args=v  # in-area: free work + particles
  stub 021a28a0 ret=none args=v  # in-area, not specced: 3D-background resources free
  stub 021a284c ret=none args=v  # in-area: sprite resources free
  stub 021a26d8 ret=none args=v  # in-area: reset
  stub 02ee0010 force ret=pick:0/1 args=v,v,v  # FAKE weather callback (T, kind, x)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 021a2c10 ret=none args=v  # in-area, not specced: particle sprite delete
  stub 021a2bb8 ret=none args=v  # in-area, not specced: particle clear
  stub 021a25cc ret=none args=v  # in-area: SE stop (T)
  stub 021a29f4 ret=none args=v  # in-area, not specced: sprite resources free (res)
  stub 021a2994 ret=none args=v  # in-area, not specced: sprite archive close (res)
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A2108(T, arc, member): field_light 021A0590(T+8, arc, member, 60).
fn 021a2108 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0590 force ret=none args=v,v,v,v  # field_light: change table (L, arc, member, frames)

# 021A2114(T, arc, member): field_light 021A0590(T+8, arc, member, 1).
fn 021a2114 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0590 force ret=none args=v,v,v,v  # field_light: change table (L, arc, member, frames)

# 021A2120(T, flag): flag == 1 -> field_light 021A05D8(T+8) (back to the default table).
fn 021a2120 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 alt range 0 2 || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a05d8 force ret=none args=v  # field_light: restore the default table

# 021A2130(T, colour, out, back): field_light 021A0618(T+8, colour, out, back).
fn 021a2130 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0618 force ret=none args=v,v,v,v  # field_light: fade request (L, colour, out, back)

# 021A213C(T): thunk to field_light 021A065C(T+8) (a light fade is set).
fn 021a213c ret=u32 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a065c force ret=pick:0/1 args=v  # field_light: fade set

# 021A2148(T): thunk to 0218E188(T+0x14) (camera getter).
fn 021a2148 ret=u32 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 0218e188 force ret=u32 args=v  # camera getter (+0x10)

# 021A2154(T, colour, out): thunk to field_light 021A063C(T+8, colour, out).
fn 021a2154 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a063c force ret=none args=v,v,v  # field_light: fade request once (L, colour, out)

# 021A2160(T, v, i): thunk to field_3dbg 021A16D4 (+8)(3D background i of the WEATHER, v).
fn 021a2160 ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 range 0 1
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a16d4 force ret=none args=v,v  # field_3dbg (G8): +8 = v

# 021A2170(T, v, i): thunk to field_3dbg 021A16E4 (+0x14)(3D background i of the WEATHER, v).
fn 021a2170 ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 range 0 1
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a16e4 force ret=none args=v,v  # field_3dbg (G8): +0x14 = v

# 021A2180(T, v, i): thunk to field_3dbg 021A16E8 (+0x18)(3D background i of the WEATHER, v).
fn 021a2180 ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 range 0 1
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a16e8 force ret=none args=v,v  # field_3dbg (G8): +0x18 = v

# 021A2190(T, v, i): thunk to field_3dbg 021A16DC (+0xC)(3D background i of the WEATHER, v).
fn 021a2190 ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 range 0 1
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a16dc force ret=none args=v,v  # field_3dbg (G8): +0xC = v

# 021A21A0(T, v, i): thunk to field_3dbg 021A16E0 (+0x10)(3D background i of the WEATHER, v).
fn 021a21a0 ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 range 0 1
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a16e0 force ret=none args=v,v  # field_3dbg (G8): +0x10 = v

# 021A21B0(T): thunk to 021A273C (status).
fn 021a21b0 ret=u8 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a273c ret=pick:0/1/2/3/4/5 args=v  # in-area: status

# 021A21B8(T): +0x3C (the weather work).
fn 021a21b8 ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A21F8(T, value, period, value end, period end, period step, count period, value step, callback): 021A2A84(T+0x4C, T, period,
# value, value end, period end, period step, count period, value step, callback) (stepper init).
fn 021a21f8 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2a84 ret=none args=v,v,v,v,v,v,v,v,v,v  # in-area: stepper init

# 021A2228(T, value end, period end, period step, value step): 021A2AAC(T+0x4C, ...) (stepper targets).
fn 021a2228 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2aac ret=none args=v,v,v,v,v  # in-area: stepper targets

# 021A223C(T, x): thunk to 021A2AB8(T+0x4C, x) (stepper step).
fn 021a223c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  fix arg0 0x52 s16 alt range 0 2 || any
  stub 021a2ab8 ret=pick:0/1 args=v,v  # in-area: stepper step
  stub 02ee0020 force ret=none args=v,v,v  # FAKE stepper callback (obj, value, x)

# 021A2248(T, x): thunk to 021A2B8C(T+0x4C, x) (plain stepper step).
fn 021a2248 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  fix arg0 0x52 s16 alt range 0 2 || any
  stub 021a2b8c ret=pick:0/1 args=v,v  # in-area: plain stepper step
  stub 02ee0020 force ret=none args=v,v,v  # FAKE stepper callback (obj, value, x)

# 021A22E0(T, a, b, flag): flag == 1 -> fog F = T+4: 021A01AC(F, 0), 021A01CC(F, 31), 021A01B4(F, a), 021A01BC(F, b), 021A01D8(F),
# 021A01A4(F, 1) (fog on).
fn 021a22e0 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 alt range 0 2 || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a01ac force ret=none args=v,v  # field_fog (G8): +1 = v
  stub 021a01cc force ret=none args=v,v  # field_fog (G8): +3 = v
  stub 021a01b4 force ret=none args=v,v  # field_fog (G8): +2 = v
  stub 021a01bc force ret=none args=v,v  # field_fog (G8): +4 = v
  stub 021a01d8 force ret=none args=v  # field_fog (G8): apply
  stub 021a01a4 force ret=none args=v,v  # field_fog (G8): on (F, flag)

# 021A231C(T, flag): flag == 1 -> 021A01A4(T+4, 0) (fog off).
fn 021a231c ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 alt range 0 2 || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a01a4 force ret=none args=v,v  # field_fog (G8): on (F, flag)

# 021A232C(T, a, b, n, mode): mode 1 -> 021A01FC(F, b, a, n); mode 2 -> a == 021A01D4(F) ? 021A01FC(F, b, a, n) : 021A01FC(F, 0x7FDF,
# 021A01D4(F), n / 2) and 021A2CC8(T, a, b, n / 2) (a second phase after the first, signed n / 2).
fn 021a232c ret=none inputs=300
  nowork
  arg buf 0x800
  arg u32 alt range 0 2 || any
  arg u32 any
  arg u32 alt range 0 9 || oneof 0xFFFFFFFF 0xFFFFFFFE 0x80000001 || any
  arg u32 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a01fc force ret=none args=v,v,v,v  # field_fog (G8): fade (F, a, b, n)
  stub 021a01d4 force ret=pick:0/1/2/0x1F args=v  # field_fog (G8): byte +2
  stub 021a2cc8 ret=none args=v,v,v,v  # in-area: fog fade request

# 021A2390(T, a, b, flag): flag == 1 -> 021A01FC(F, a, 021A01D4(F), b).
fn 021a2390 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 alt range 0 2 || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a01d4 force ret=pick:0/1/2/0x1F args=v  # field_fog (G8): byte +2
  stub 021a01fc force ret=none args=v,v,v,v  # field_fog (G8): fade (F, a, b, n)

# 021A23B0(T): fog busy (021A0244(F)) -> 0; no pending fog fade (021A2CF8) -> 1; else 0.
fn 021a23b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0244 force ret=pick:0/1 args=v  # field_fog (G8): busy
  stub 021a2cf8 ret=pick:0/1 args=v  # in-area: fog fade pending

# 021A23D4(T): zone fog 021A0354(T+0x10) == 1.
fn 021a23d4 ret=u32 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0354 force ret=pick:0/1/2 args=v  # field_zonefog (G8): state byte

# 021A23E8(T): thunk to field_zonefog 021A0380(T+0x10).
fn 021a23e8 ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0380 force ret=pick:0/1 args=v  # field_zonefog (G8): query

# 021A23F4(T): thunk to field_zonefog 021A0390(T+0x10).
fn 021a23f4 ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0390 force ret=pick:0/1 args=v  # field_zonefog (G8): query

# 021A2400(T): thunk to field_zonefog 021A03A4(T+0x10).
fn 021a2400 ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a03a4 force ret=u32 args=v  # field_zonefog (G8): getter

# 021A240C(T): thunk to field_zonefog 021A03B8(T+0x10).
fn 021a240c ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a03b8 force ret=u32 args=v  # field_zonefog (G8): getter

# 021A2418(T): thunk to field_zonefog 021A03CC(T+0x10).
fn 021a2418 ret=u32 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a03cc force ret=u32 args=v  # field_zonefog (G8): getter

# 021A25BC(T, se): thunk to 021A2C30(T+0x40, T+0xC, se) (start the weather SE).
fn 021a25bc ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2c30 ret=none args=v,v,v  # in-area: SE start (SND, h, se)
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028a7c force ret=none args=v,v  # sound: play SE on a handle (h, se)
  stub 020061e4 force ret=none args=v  # sound: play SE
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A25CC(T): thunk to 021A2C6C(T+0x40, T+0xC) (stop it).
fn 021a25cc ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A25DC(T, se): BGM system idle (02005C64 == 0) -> T+0x48 ? 02028A7C(T+0xC, se) : 020061E4(se) (play an SE).
fn 021a25dc ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028a7c force ret=none args=v,v  # sound: play SE on a handle (h, se)
  stub 020061e4 force ret=none args=v  # sound: play SE

# 021A2604(T, se, v): BGM system idle -> T+0x48 ? 02028ACC(T+0xC, se, v) : 020061F0(se, v).
fn 021a2604 ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028acc force ret=none args=v,v,v  # sound: play SE on a handle (h, se, v)
  stub 020061f0 force ret=none args=v,v  # sound: play SE (se, v)

# 021A26D8(T) (reset): the TASK zeroed (0x784) except +0..+0x1B and the u16 +0x1C.
fn 021a26d8 ret=none inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2724(T, cb, x): cb ? cb(T, +0x24 & 0xF, x) : 1.
fn 021a2724 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg u32 alt const 0 || const 0x02EE0011
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 02ee0010 force ret=pick:0/1 args=v,v,v  # FAKE weather callback (T, kind, x)

# 021A273C(T): the status of state +0x26 (ROM 0x021D88E8, 14 bytes).
fn 021a273c ret=u8 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A282C(T, heap): 021A2978(T+0x28, data, heap), 021A29A8(T+0x28, data, heap) (sprite resources).
fn 021a282c ret=none inputs=60
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a2978 ret=none args=v,v,v  # in-area, not specced: sprite archive open (res, data, heap)
  stub 021a29a8 ret=none args=v,v,v  # in-area, not specced: sprite resources load (res, data, heap)

# 021A284C(T): 021A29F4(T+0x28), 021A2994(T+0x28).
fn 021a284c ret=none inputs=60
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a29f4 ret=none args=v  # in-area, not specced: sprite resources free (res)
  stub 021a2994 ret=none args=v  # in-area, not specced: sprite archive close (res)

# 021A28CC(T, heap): +0x3C = Heap_AllocDebug(heap, data+0x28, 1, "weather_task.c", 0x874); particles on (data+2) ->
# 021A2BD4(particle i, T, T+0x28, +0, heap), i < 32.
fn 021a28cc ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 02030734 force ret=ptr:arg0:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 021a2bd4 ret=none args=v,v,v,v,v  # in-area, not specced: particle sprite new (p, T, res, spr, heap)

# 021A2924(T): +0x3C -> Heap_Free, 0; particles on -> 021A2C10 / 021A2BB8(particle i), i < 32; 021A25CC(T) (stop the SE).
fn 021a2924 ret=none inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 021a2c10 ret=none args=v  # in-area, not specced: particle sprite delete
  stub 021a2bb8 ret=none args=v  # in-area, not specced: particle clear
  stub 021a25cc ret=none args=v  # in-area: SE stop (T)
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A2968(T): +0x27 = 1.
fn 021a2968 ret=none inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2970(T): +0x27.
fn 021a2970 ret=u8 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2A84(S, obj, period, value, value end, period end, period step, count period, value step, callback): S = {obj, value, 0,
# period, 0, value end, period end, period step, count period, value step, callback} (u16 / s32 stores).
fn 021a2a84 ret=none inputs=60
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any

# 021A2AAC(S, value end, period end, period step, value step): S+0xC / +0xE / +0x10 (u16), S+0x14.
fn 021a2aac ret=none inputs=60
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any

# 021A2AB8(S, x) (stepper step): period step 0 or the period at its end (by the step's sign; clamped) -> flag 1; value step 0 or the
# value at its end -> flag 2; countdown - 1 <= 0 -> callback(obj, value, x), countdown = period, then without flag 1 period += period
# step, without flag 2 every count period frames value += value step. Answers flag 1.
fn 021a2ab8 ret=u32 inputs=600
  nowork
  arg buf 0x20
  arg u32 any
  fix arg0 4 s16 alt range 0 3 || any
  fix arg0 6 s16 alt range 0 2 || any
  fix arg0 8 s16 alt range 0 3 || any
  fix arg0 0xA s16 alt range 0 3 || any
  fix arg0 0xC s16 alt range 0 3 || any
  fix arg0 0xE s16 alt range 0 3 || any
  fix arg0 0x10 u16 alt oneof 0 1 2 0xFFFF 0xFFFE || any
  fix arg0 0x12 s16 alt range 0 3 || any
  fix arg0 0x14 u32 alt oneof 0 1 2 0xFFFFFFFF 0xFFFFFFFE || any
  fix arg0 0x18 u32 const 0x02EE0021
  stub 02ee0020 force ret=none args=v,v,v  # FAKE stepper callback (obj, value, x)

# 021A2B8C(S, x): countdown - 1 <= 0 -> callback(obj, value, x), countdown = period, 1; else 0.
fn 021a2b8c ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 any
  fix arg0 4 s16 alt range 0 3 || any
  fix arg0 6 s16 alt range 0 2 || any
  fix arg0 8 s16 alt range 0 3 || any
  fix arg0 0xA s16 alt range 0 3 || any
  fix arg0 0xC s16 alt range 0 3 || any
  fix arg0 0xE s16 alt range 0 3 || any
  fix arg0 0x10 u16 alt oneof 0 1 2 0xFFFF 0xFFFE || any
  fix arg0 0x12 s16 alt range 0 3 || any
  fix arg0 0x14 u32 alt oneof 0 1 2 0xFFFFFFFF 0xFFFFFFFE || any
  fix arg0 0x18 u32 const 0x02EE0021
  stub 02ee0020 force ret=none args=v,v,v  # FAKE stepper callback (obj, value, x)

# 021A2C2C(SND, mode): SND+8 = mode.
fn 021a2c2c ret=none inputs=40
  nowork
  arg buf 0x10
  arg u32 any

# 021A2C30(SND, h, se): 021A2C6C(SND, h) (stop); BGM system busy -> {2, se} (wait); else {1, se}, play (SND+8 ? 02028A7C(h, se) :
# 020061E4(se)).
fn 021a2c30 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any
  fix arg0 0 u16 alt range 0 3 || any
  fix arg0 8 u32 alt const 0 || any
  stub 021a2c6c ret=none args=v,v  # in-area: SE stop (SND, h)
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028a7c force ret=none args=v,v  # sound: play SE on a handle (h, se)
  stub 020061e4 force ret=none args=v  # sound: play SE
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A2C6C(SND, h): playing (1) -> 0, stop (SND+8 ? 02028B50(h, se) : 020061F8(02006148(se))).
fn 021a2c6c ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  fix arg0 0 u16 alt range 0 3 || any
  fix arg0 8 u32 alt const 0 || any
  stub 02028b50 force ret=none args=v,v  # sound: stop SE on a handle (h, se)
  stub 02006148 force ret=u32 args=v  # sound: SE player of an SE
  stub 020061f8 force ret=none args=v  # sound: stop a player

# 021A2C98(SND, h): waiting (2) and the BGM system idle -> 1, play.
fn 021a2c98 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  fix arg0 0 u16 alt range 0 3 || any
  fix arg0 8 u32 alt const 0 || any
  stub 02005c64 force ret=pick:0/1 args=  # sound: BGM system busy
  stub 02028a7c force ret=none args=v,v  # sound: play SE on a handle (h, se)
  stub 020061e4 force ret=none args=v  # sound: play SE

# 021A2CC8(T, a, b, n): fog fade +0x77C = {1, (u8) a, (u8)(n / 2)}, +0x780 = b.
fn 021a2cc8 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 alt range 0 9 || oneof 0xFFFFFFFF 0xFFFFFFFE || any
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2CEC(T): +0x77C = 0.
fn 021a2cec ret=none inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2CF8(T): +0x77C.
fn 021a2cf8 ret=u8 inputs=40
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4

# 021A2D04(T): 021A0224(F, 0x7FDF, +0x780, +0x77D, +0x77D, +0x77E) (the second fog fade).
fn 021a2d04 ret=none inputs=60
  nowork
  arg buf 0x800
  fix arg0 0x18 u32 addrof arg0 0x790
  fix arg0 0x20 u32 addrof arg0 0x7A0
  fix arg0 0x24 u8 alt range 0 0x22 || any
  fix arg0 0x25 u8 alt range 0 2 || any
  fix arg0 0x26 u8 range 0 13
  fix arg0 0x27 u8 alt const 0 || any
  fix arg0 0x3C u32 alt const 0 || any
  fix arg0 0x40 u16 alt range 0 3 || any
  fix arg0 0x48 u32 alt const 0 || any
  fix arg0 0x64 u32 const 0x02EE0021
  fix arg0 0x77C u8 alt const 0 || any
  fix arg0 0x7A2 u16 alt const 0 || any
  fix arg0 0x7CC u32 alt const 0 || const 0x02EE0011 rep=7,4
  stub 021a0224 force ret=none args=v,v,v,v,v,v  # field_fog (G8): fade (F, a, b, c, d, e)
