# fndiff spec: the head of event_battle.c (overlay 10 Thumb code 0x0216E62C..0x0216E954; F1 counted it in 10.38 fldmmdl.c, but the
# functions are battle-event glue: GMEVENT procs / constructors calling btl_setup and the event_battle work init 0216F02C; F5b owns
# event_battle.c from 0x0216E95C). F4a-spec. Grammar: svflow_specs/README.md. Run: tools/check-fn.sh black-field event_battle_head
# All blocks are `nowork`; objects are `arg buf 0x100` arenas (the GMEVENT work that 020124E8 returns is the arena named in that
# block's stub, `ptr:argK:0:1`). Every callee is a `force` stub (black-field btl_setup 02013090 / 02013144 are ported elsewhere).
# GMEVENT (game_event.c, black-field game_data): 020122C0(gsys, parent, proc, work size) creates, 020124E8(ev) = its work,
# 02012374(ev, child) calls, 0201235C(ev, next) changes. Battle call work (0x10, built by 0216E7C8): +0 gsys, +4 BATTLE_SETUP,
# +8 flag (1 = return at once after the battle), +0xC. Child work (0x28, proc 0x0216EB99, init 0216F02C): +0x14, +0x18, +0x1C,
# +0x20, +0x24 (9 in 0216E8CC / 0216E910), +8 -> object whose +0x18 u16 is set to 0x46B by 0216E910.
area event_battle_head
set field

# 0216E62C(x, mon, bg): dex = 0200CA1C(x); species = PokeParam(mon, 5); ROM 0x02172554 (25 x {u16 bg, u16 form}): first entry
# with bg: species 0x19C (Burmy): set form (02017988), register (0200CFA0); no call reaches this function (dead code?)
fn 0216e62c ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 26 || any
  stub 0200ca1c force ret=u32 args=v  # (x) -> pokedex/save object
  stub 0200cfa0 force ret=none args=v,v  # pokedex register (dex, mon)
  stub 02017988 force ret=none args=v,v  # PokeParam set form (mon, form)
  stub 02017e1c force ret=pick:0x19c/0x19d/1 args=v,v,v  # PokeParam get (mon, 5 = species, 0)

# 0216E680(f): f != 0: f(); returns ~f (callback at 0x0216EDD8)
fn 0216e680 ret=u32 inputs=40
  nowork
  arg u32 alt const 0 || const 0x02EE0011
  stub 02ee0010 force ret=none args=v  # FAKE callback

# 0216E68C(event, seq, w) GMEVENT proc (wild/trainer battle call): w +0 gsys, +4 BATTLE_SETUP, +8 flag, +0xC;
# seq 0: child = GMEVENT(gsys, 0, 0x0216EB99, 0x28), 0216F02C(child work, gsys, setup), work+0x14 = 1, species of party mon 0 ...,
# 021CF5D8(..), call child, seq++; 1: flag 1: end (1); else 0216EFA8(..) ? change to 0215F6E8(gsys) : call 021BE92C(..), seq++;
# 2: 0202E818(..) == 1: call 02181D8C(gsys, 4); seq++; 3: end (1); other: 0
fn 0216e68c ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x100
  arg buf 0x100
  arg buf 0x100
  arg buf 0x100
  arg buf 0x100
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 4 u32 addrof arg3 0
  fix arg2 8 u32 alt oneof 0 1 || any
  fix arg3 4 u32 alt const 1 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 0201235c force ret=none args=v,v  # GMEVENT change (event, next)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02012a44 force ret=u32 args=v  # GAMEDATA getter
  stub 02012f20 force ret=u32 args=v  # GAMEDATA getter
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 02017e1c force ret=pick:0x19c/0x19d/1 args=v,v,v  # PokeParam get (mon, 5 = species, 0)
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, 0)
  stub 0202e79c force ret=none args=v
  stub 0202e818 force ret=pick:0/1 args=v
  stub 0215f6e8 force ret=u32 args=v  # (gsys) -> event
  stub 0216efa8 force ret=pick:0/1 args=v,v
  stub 0216f02c force ret=none args=v,v,v  # event_battle (ov10, next area): battle event work init (work, gsys, setup)
  stub 02181d8c force ret=u32 args=v,v  # (gsys, 4) -> event
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fieldmap, 0, 0, 1, 0, 0) -> event
  stub 021cf5d8 force ret=none args=v,v,v,v,v,v

# 0216E7C8(gsys, fieldmap, x, y, z): ev = GMEVENT(gsys, 0, 0x0216E68D, 0x10); w = work: +0 gsys, +4 x, +8 y, +0xC z;
# 0216EE40(x, 0); 02008DF0(GAMEDATA record, 5); 021AAABC(02188CE8(fieldmap)); returns ev
fn 0216e7c8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02012f2c force ret=u32 args=v  # GAMEDATA getter
  stub 0216ee40 force ret=none args=v,v
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP getter
  stub 021aaabc force ret=none args=v

# 0216E818(gsys, fieldmap, a, b, c, d, e): ev = GMEVENT(.., 0x0216EB99, 0x28); setup = btl_setup alloc(4); 021AA5CC(..);
# 0216EE20((u16) c, 0); records 6 and 0x56; 0216F02C(w, gsys, setup); w+0x14 = 1; e & 1: or_flags(setup, 0x100), w+0x18 = 1;
# 021CF748(c, fieldmap, w+0x24, *(w+8) + 0x18); 021AAABC(..); returns ev
fn 0216e818 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt oneof 0 1 || any
  arg buf 0x100
  fix arg7 8 u32 any
  stub 020124e8 force ret=ptr:arg7:0:1 args=v  # GMEVENT work (event)
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 02012f2c force ret=u32 args=v  # GAMEDATA getter
  stub 02013090 force ret=u32 args=v  # btl_setup alloc (heap 4) (black-field btl_setup::alloc)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, 0x100) (black-field btl_setup::or_flags)
  stub 0216ee20 force ret=none args=v,v
  stub 0216f02c force ret=none args=v,v,v  # event_battle (ov10, next area): battle event work init (work, gsys, setup)
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP getter
  stub 021aa5cc force ret=none args=v,v,v,v,v,v,v
  stub 021aaabc force ret=none args=v
  stub 021cf748 force ret=none args=v,v,v,v

# 0216E8CC(gsys, fieldmap, setup): ev = GMEVENT(.., 0x0216EB99, 0x28); 0216F02C(w, gsys, setup); w +0x14 = 0, +0x18 = 1,
# +0x20 = 1, +0x24 = 9; 021AAABC(02188CE8(fieldmap)); returns ev
fn 0216e8cc ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  arg buf 0x100
  arg buf 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 0216f02c force ret=none args=v,v,v  # event_battle (ov10, next area): battle event work init (work, gsys, setup)
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP getter
  stub 021aaabc force ret=none args=v

# 0216E910(gsys, fieldmap, setup): as 0216E8CC with +0x1C = 1 (not +0x20); *(w+8) +0x18 = 0x46B (u16)
fn 0216e910 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  arg buf 0x100
  arg buf 0x100
  fix arg5 8 u32 addrof arg5 0x80
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 0216f02c force ret=none args=v,v,v  # event_battle (ov10, next area): battle event work init (work, gsys, setup)
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP getter
  stub 021aaabc force ret=none args=v

