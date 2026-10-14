# fndiff spec: event_battle.c after its head (overlay 10 Thumb code 0x0216E954..0x0216F088, F1 row 10.39; F5b): the battle child
# GMEVENT (proc 0x0216EB99: fade, battle overlays 0x5D / 0x5C, return to the field), the capture-demo battle, the special-encounter
# override of 0216E818's foe, the post-battle processing (last result into GAMEDATA, roamer update, beacon notifications, record 0x4F)
# and the battle event work init. File bound: `Heap_AllocDebug(heap, 0xDC, 1, "event_battle.c", 0x1DA)` in 0216E95C (string
# 0x02172B54); the head 0x0216E62C..0x0216E954 is F4a's area `event_battle_head`; trcard_sys.c starts at 0x0216F088.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field event_battle --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 17 entries
# (0216EB98 is a `.byte` block Ghidra misses; 0x0216EBBC / 0x0216EBE0 are labels inside it; 0x0216EE4E is a data word at 0x0216DE4C that
# decodes as `bl`). Render-only functions: none.
# Closure stubs (as core4 / F4a / F5a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force` (overlay addresses collide with battle PORTED entries; ARM9 / other-area callees are ported elsewhere).
#
# BATTLE EVENT work W (0x28, 0216F02C): +0 GAMESYS, +4 GAMEDATA, +8 BATTLE_SETUP, +0xC / +0x10 the (GAMEDATA, setup) pair handed to the
# battle overlay 0x5C, +0x14 1 = end right after the battle (no return fade), +0x18 1 = never "lost" (no white-out check), +0x1C 1 =
# record the battle (02168D30), +0x20 1 = keep the setup (0216F054 frees it otherwise), +0x24 (021CF5D8 / 021CF498 cookie).
# BATTLE_SETUP fields read here: +0 battle type (0..4: column of the ROM table 0x021725D8), +4, +0x18 u16 (02029BCC: the battle BGM /
# fade), +0x28 the foe party, +0x4C -> u16 (trainer id), +0x8C battle result (0..6: row of 0x021725D8).
# ROM tables: 0x021725B8 u8[8] (0216EFCC; 0xFF -> 1), 0x021725C0 3 x 2 function pointers (0216EE20: beacon notifications by the
# 0216EDFC class, overlay-10 0x02165174..0x0216528C = game_beacon (X1), force stubs), 0x021725D8 u8[7][5] (0216EFA8: "lost" by result /
# type). Integrity checks in the proc (cases 6 / 7): ARM entries 021882DC / 02188354 / 02188390 of an overlay that is not part of the
# field set (the field set has ov21 Thumb code there) get (0x0216E681, callback, 0) and should answer ~callback; a sum that starts at
# 0x013A1AB5 (divisible by 0x1933) gains 0x3D1 / 0x5D1 on a mismatch and 0x9D on the third match; a nonzero remainder (the ARM9 divide
# 0209C2B0, real code) calls 020056A0(0x02011D35, 0, 0x7F) and counts the global 0x0214624C.
# All blocks are `nowork`; objects are `arg buf` arenas or stub answers.
area event_battle
set field

# 0216E95C(gsys, a1, heap) -> event: the capture demo. hid = 0x8000 | heap & 0x7FFF; gd = 020120F4(gsys); setup = 02013090(heap);
# p1 = 0201A920(hid), p2 = 0201A920(hid); mon = Heap_AllocDebug(hid, 0xDC, 1, "event_battle.c", 0x1DA); 0201756C(mon, 0x23C, 7, 0, 0)
# (species 572, level 7), moves 1 / 0x2D / 0 / 0 (02017E40 ids 0x36..0x39), 0201A98C(p1, mon); 0201756C(mon, 0x1F8, 2, 0, 0), moves
# 0x21 / 0x2B / 0 / 0, 0201A98C(p2, mon); 021AA304(&sit, gd, a1), sit +0 = 0, +4 = 5; 020135C0(setup, gd, p1, p2, &sit, heap); free
# mon, p2, p1; ev = 020122C0(gsys, 0, 0x0216EB99, 0x28); W = 020124E8(ev); 0216F02C(W, gsys, setup); W+0x14 = 1;
# 021CF5D8(0x1F8, 5, 0, a1, W+0x24, *(W+8) + 0x18); W+0x24 = 0; ev
fn 0216e95c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02013090 force ret=u32 args=v  # btl_setup alloc (heap) (black-field btl_setup)
  stub 0201a920 force seq ret=u32 args=v  # party alloc (heap)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0201756c force ret=none args=v,v,v,v,v  # PokeParam init (mon, species, level, ...)
  stub 02017e40 force ret=none args=v,v,v  # PokeParam set (mon, id, value)
  stub 0201a98c force ret=none args=v,v  # party add (party, mon)
  stub 021aa304 force ret=none args=out:0x10,v,v  # field_encount: FIELD SITUATION (sit, gd, a1) (black-field encount)
  stub 020135c0 force ret=none args=v,v,v,v,in:0x10,v  # btl_setup: wild setup from two parties (setup, gd, p1, p2, &sit, heap)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 0216f02c ret=none args=v,v,v  # in-area: W init (W, gsys, setup)
  stub 021cf5d8 force ret=none args=v,v,v,v,v,v  # (species, 5, 0, a1, &W+0x24, *(W+8) + 0x18)

# 0216EAB0(gsys, fm, a, b, c, d, e, f) -> event: S = 02012FD8(020120F4(gsys)); ev = 0216E818(gsys, fm, a, b, c, d, e); 0200E740(S, f)
# != 0: mon = 0201AB28(*(*(W+8) + 0x28), 0) (W = 020124E8(ev)), sp = 02017E1C(mon, 5, 0); 02017588(mon, (u16) 0200E788(S, f, 21),
# 0200E74C(S, f), 0200E7B4(S, f), 0, -1, 0200E7A4(S, f), 0); 0200E76C(S, f) != 0: 02017E40(mon, 10, 0200E780(S, f)); 02017E40(mon,
# 0x74, 0200E7AC(S, f)); 020178D8(mon, (u16) sp); 02017FA8(mon); ev. Arena: arg 8 (W at 0, setup at 0x100).
fn 0216eab0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x200
  fix arg8 8 u32 addrof arg8 0x100
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012fd8 force ret=u32 args=v  # GAMEDATA -> save record S
  stub 0216e818 force ret=u32 args=v,v,v,v,v,v,v  # event_battle_head: battle event (gsys, fm, a, b, c, d, e)
  stub 0200e740 force ret=pick:0/1 args=v,v  # S entry f present
  stub 020124e8 force ret=ptr:arg8:0:1 args=v  # GMEVENT work (event)
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, 0)
  stub 02017e1c force ret=u32 args=v,v,v  # PokeParam get (mon, 5 = species, 0)
  stub 0200e788 force ret=u32 args=v,v,v  # S entry f field (S, f, 21)
  stub 0200e74c force ret=u32 args=v,v  # S entry f field
  stub 0200e7b4 force ret=u32 args=v,v  # S entry f field
  stub 0200e7a4 force ret=u32 args=v,v  # S entry f field
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v  # PokeParam re-init (mon, species, level, id, 0, -1, x, 0)
  stub 0200e76c force ret=pick:0/1 args=v,v  # S entry f has an item
  stub 0200e780 force ret=u32 args=v,v  # S entry f item
  stub 02017e40 force ret=none args=v,v,v  # PokeParam set (mon, id, value)
  stub 0200e7ac force ret=u32 args=v,v  # S entry f field 0x74
  stub 020178d8 force ret=none args=v,v  # PokeParam (mon, species)
  stub 02017fa8 force ret=none args=v  # PokeParam recalc (mon)

# 0216EB98(ev, seq, W) GMEVENT proc 0x0216EB99 (12 steps; gsys = W+0, gd = W+4, fm = 0201210C(gsys); every step but 8 / 11 moves on
# (*seq + 1) and returns 0 through the integrity tail): 0: call 02029BCC(gsys, setup u16 +0x18); 1: 021CF498(02189A08(fm), ev, W+0x24);
# 2: call 020144DC(gsys, fm); 3: 0201447C(02012B38(gd), 1), 0202E79C(02012F20(gd)), 02012444(ev, 0x5D, 0x021EFDAC, setup); 4: W+0x10 =
# setup, W+0xC = gd, 02012444(ev, 0x5C, 0x021B9D94, W+0xC); 5: 02034AC4(0xE6), 0201447C(.., 0), 0202E80C(02012F20(gd)); 6: check 1,
# 0216EFF0(W, gd), W+0x1C: 02168D30(*02012F50(gd), setup), seq + 1, check 2; 7: check 3, W+0x18 == 0 and 0216EFDC(W) == 1 ? call
# 02029B28(gsys, 30) : call 02029C00(gsys), 02034A5C(0xE6); 8: W+0x18 == 0 and 0216EFDC(W) == 1: 0216F054(W), W+0x14 == 1 -> 1, else
# change to 0215F6E8(gsys) -> 0; otherwise call 020145CC(gsys); 9: call 02029CC0(gsys); 10: W+0x14 == 0: call 021BE92C(gsys, fm, 0, 0,
# 1, 0, 0); 11: 0216F054(W) -> 1; > 11: tail. Arenas: arg 2 = W (+0xF0 = the 02012F50 record), arg 3 = setup (+0x100 = the 0x4C word).
fn 0216eb98 ret=u32 inputs=800
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x200
  fix arg1 0 u32 alt range 0 11 || range 0 11 || any
  fix arg2 8 u32 addrof arg3 0
  fix arg2 0x14 u32 alt oneof 0 1 || any
  fix arg2 0x18 u32 alt const 0 || oneof 0 1 || any
  fix arg2 0x1C u32 alt oneof 0 1 || any
  fix arg2 0x20 u32 alt oneof 0 1 || any
  fix arg3 0 u32 range 0 4
  fix arg3 0x8C u32 range 0 6
  fix arg3 0x4C u32 addrof arg3 0x100
  global 0214624c 4
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02029bcc force ret=u32 args=v,v  # (gsys, bgm) -> event
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02189a08 force ret=u32 args=v  # FIELDMAP getter
  stub 021cf498 force ret=none args=v,v,v  # (x, event, W+0x24)
  stub 020144dc force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201447c force ret=none args=v,v  # (x, flag)
  stub 02012f20 force ret=u32 args=v  # GAMEDATA getter
  stub 0202e79c force ret=none args=v
  stub 0202e80c force ret=none args=v
  stub 02012444 force ret=none args=v,v,v,v  # app proc call (event, overlay, proc data, param)
  stub 02034ac4 force ret=none args=v  # overlay 0xE6 (ARM)
  stub 02034a5c force ret=none args=v  # overlay 0xE6 (ARM)
  stub 021882dc force ret=pick+u32:0xFDE90F9A/0xFDE90F9A/0 args=v,v,v  # integrity check 1 (ARM; ~0x0216F065 = pass)
  stub 02188354 force ret=pick+u32:0xFDE90F8E/0xFDE90F8E/0 args=v,v,v  # integrity check 2 (ARM; ~0x0216F071 = pass)
  stub 02188390 force ret=pick+u32:0xFDE90F82/0xFDE90F82/0 args=v,v,v  # integrity check 3 (ARM; ~0x0216F07D)
  stub 020056a0 force ret=none args=v,v,v  # (0x02011D35, 0, 0x7F): the failed-check reaction
  stub 02012f50 force ret=ptr:arg2:0xF0:1 args=v  # GAMEDATA -> record (gd + 0x6A4)
  stub 02168d30 force ret=none args=v,v  # battle record (x, setup)
  stub 02029b28 force ret=u32 args=v,v  # (gsys, 30) -> event
  stub 02029c00 force ret=u32 args=v  # (gsys) -> event
  stub 02029cc0 force ret=u32 args=v  # (gsys) -> event
  stub 020145cc force ret=u32 args=v  # (gsys) -> event (white-out)
  stub 0215f6e8 force ret=u32 args=v  # (gsys) -> event
  stub 0201235c force ret=none args=v,v  # GMEVENT change (event, next)
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, 0, 0, 1, 0, 0) -> event (fade in)
  stub 0216efdc ret=pick:0/1 args=v  # in-area: lost (W)
  stub 0216eff0 ret=none args=v,v  # in-area: after the battle (W, gd)
  stub 0216f054 ret=none args=v  # in-area: free the setup (W)
  # descendants (0216EFF0 -> 0216EE7C / 0216EF84 -> 0216EE20 / 0216EE40 -> 0216EDFC; 0216EFDC -> 0216EFA8)
  stub 0216ee7c ret=none args=v  # in-area
  stub 0216ef84 ret=none args=v  # in-area
  stub 0216ee20 ret=none args=v,v  # in-area
  stub 0216ee40 ret=none args=v,v  # in-area
  stub 0216edfc ret=pick:0/1/2 args=v  # in-area
  stub 0216efa8 ret=pick:0/1 args=v,v  # in-area
  stub 02012a4c force ret=none args=v,v  # GAMEDATA +0x1BC = result
  stub 0215e5b0 force ret=none args=v,v  # move_pokemon: roamer after the battle (gd, setup)
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER
  stub 02012ac8 force ret=u16 args=v  # PLAYER u16 +0
  stub 0202c83c force ret=none args=v,v  # (gsys, x)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012f2c force ret=u32 args=v  # GAMEDATA -> record block
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)
  stub 02012934 force ret=u32 args=v  # GAMEDATA -> party
  stub 0201ab18 force ret=u16 args=v  # party: lead index
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, i)
  stub 020457b0 force ret=u32 args=v,v  # STRBUF alloc (12, heap)
  stub 02017e1c force ret=pick+u32:0/1/2/3/4/0x10000/0x10001 args=v,v,v  # PokeParam get (mon, id, x)
  stub 021654ac force ret=none args=v  # game_beacon (X1): fainted
  stub 02165424 force ret=none args=v  # game_beacon (X1): low HP
  stub 02165468 force ret=none args=v  # game_beacon (X1): no PP
  stub 021654f0 force ret=none args=v  # game_beacon (X1): status
  stub 02045808 force ret=none args=v  # STRBUF free
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 0216505c force ret=none args=v  # game_beacon (X1)
  stub 02165078 force ret=none args=v  # game_beacon (X1)
  stub 021653f4 force ret=none args=v  # game_beacon (X1)
  stub 0202a230 force ret=u32 args=v,v  # (trainer id, 1) -> trainer data
  stub 0202a3fc force ret=pick+u32:2/9/12/0/1/3 args=v  # trainer class kind
  stub 02165174 force ret=none args=v  # game_beacon (X1) (table 0x021725C0)
  stub 021651ac force ret=none args=v
  stub 021651e4 force ret=none args=v
  stub 0216521c force ret=none args=v
  stub 02165254 force ret=none args=v
  stub 0216528c force ret=none args=v
  stub 020130b4 force ret=none args=v  # btl_setup free (black-field btl_setup)

# 0216EDFC(x) -> class: k = 0202A3FC(0202A230(x, 1)); k == 12 or 9 -> 0; k == 2 -> 1; else 2
fn 0216edfc ret=u32 inputs=200
  nowork
  arg u32 any
  stub 0202a230 force ret=u32 args=v,v  # (trainer id, 1) -> trainer data
  stub 0202a3fc force ret=pick+u32:2/9/12/0/1/3 args=v  # trainer class kind

# 0216EE20(x, i): k = 0216EDFC(x); ROM 0x021725C0[k][i](x) (3 x 2 game_beacon calls)
fn 0216ee20 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 range 0 1
  stub 0216edfc ret=pick:0/1/2 args=v  # in-area
  stub 0202a230 force ret=u32 args=v,v  # (trainer id, 1) -> trainer data
  stub 0202a3fc force ret=pick+u32:2/9/12/0/1/3 args=v  # trainer class kind
  stub 02165174 force ret=none args=v  # game_beacon (X1) (table 0x021725C0)
  stub 021651ac force ret=none args=v
  stub 021651e4 force ret=none args=v
  stub 0216521c force ret=none args=v
  stub 02165254 force ret=none args=v
  stub 0216528c force ret=none args=v

# 0216EE40(setup, i): sp = (u16) 02017E1C(0201AB28(0201313C(setup, 1), 0), 5, 0); i 0: 0216505C(sp), 1: 02165078(sp), 2: 021653F4(sp)
fn 0216ee40 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 3 || any
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, 0)
  stub 02017e1c force ret=pick+u32:0x281/0x282/0x10001 args=v,v,v  # PokeParam get (mon, 5 = species, 0)
  stub 0216505c force ret=none args=v  # game_beacon (X1)
  stub 02165078 force ret=none args=v  # game_beacon (X1)
  stub 021653f4 force ret=none args=v  # game_beacon (X1)

# 0216EE7C(W): mon = party member 0201AB18(P) of P = 02012934(W+4); result r = setup +0x8C; r != 0 and r != 2: s = 020457B0(12,
# 0x8004), 02017E1C(mon, 0x73, s) (nickname), hp = (u16) get 0xA0, max = (u16) get 0xA1: hp == 0 -> 021654AC(s), else 2 hp <= max ->
# 02165424(s); first i < 4 with move 0x36+i != 0 and PP 0x3A+i == 0 -> 02165468(s); status 0x9D != 0 -> 021654F0(s); 02045808(s).
# Type t = setup +0: t 0: r 3 -> 0216EE40(setup, 2), r 1 -> 0216EE40(setup, 1); t 1, r 1: 0216EE20((u16) *(setup +0x4C), 1).
# Arenas: arg 0 = W, arg 1 = setup (+0x100 = the 0x4C word).
fn 0216ee7c ret=none inputs=600
  nowork
  arg buf 0x100
  arg buf 0x200
  fix arg0 8 u32 addrof arg1 0
  fix arg1 0 u32 alt range 0 4 || range 0 1
  fix arg1 0x8C u32 alt range 0 6 || range 0 3
  fix arg1 0x4C u32 addrof arg1 0x100
  stub 02012934 force ret=u32 args=v  # GAMEDATA -> party
  stub 0201ab18 force ret=u16 args=v  # party: lead index
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, i)
  stub 020457b0 force ret=u32 args=v,v  # STRBUF alloc (12, heap)
  stub 02017e1c force ret=pick+u32:0/1/2/3/4/0x10000/0x10001 args=v,v,v  # PokeParam get (mon, id, x)
  stub 021654ac force ret=none args=v  # game_beacon (X1): fainted
  stub 02165424 force ret=none args=v  # game_beacon (X1): low HP
  stub 02165468 force ret=none args=v  # game_beacon (X1): no PP
  stub 021654f0 force ret=none args=v  # game_beacon (X1): status
  stub 02045808 force ret=none args=v  # STRBUF free
  stub 0216ee20 ret=none args=v,v  # in-area
  stub 0216ee40 ret=none args=v,v  # in-area
  stub 0216edfc ret=pick:0/1/2 args=v  # in-area
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 0216505c force ret=none args=v  # game_beacon (X1)
  stub 02165078 force ret=none args=v  # game_beacon (X1)
  stub 021653f4 force ret=none args=v  # game_beacon (X1)
  stub 0202a230 force ret=u32 args=v,v  # (trainer id, 1) -> trainer data
  stub 0202a3fc force ret=pick+u32:2/9/12/0/1/3 args=v  # trainer class kind
  stub 02165174 force ret=none args=v  # game_beacon (X1) (table 0x021725C0)
  stub 021651ac force ret=none args=v
  stub 021651e4 force ret=none args=v
  stub 0216521c force ret=none args=v
  stub 02165254 force ret=none args=v
  stub 0216528c force ret=none args=v

# 0216EF84(W): rec = 02012F2C(020120F4(W+0)); setup +0 == 0 and +0x8C == 4: 02008DF0(rec, 0x4F) (record counter 0x4F + 1)
fn 0216ef84 ret=none inputs=200
  nowork
  arg buf 0x100
  arg buf 0x200
  fix arg0 8 u32 addrof arg1 0
  fix arg1 0 u32 alt const 0 || range 0 4
  fix arg1 0x8C u32 alt const 4 || range 0 6
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012f2c force ret=u32 args=v  # GAMEDATA -> record block
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)

# 0216EFA8(r, t) -> lost: t == 4 -> 0; ROM 0x021725D8[5 r + t] == 0 -> 1, else 0
fn 0216efa8 ret=u32 inputs=200
  nowork
  arg u32 range 0 6
  arg u32 range 0 4

# 0216EFCC(i) -> ROM 0x021725B8[i] (u8), 0xFF -> 1
fn 0216efcc ret=u32 inputs=60
  nowork
  arg u32 range 0 7

# 0216EFDC(W) -> 0216EFA8(setup +0x8C, setup +0) (tail call through a literal)
fn 0216efdc ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x200
  fix arg0 8 u32 addrof arg1 0
  fix arg1 0 u32 range 0 4
  fix arg1 0x8C u32 range 0 6
  stub 0216efa8 ret=pick:0/1 args=v,v  # in-area

# 0216EFF0(W, gd): 02012A4C(gd, setup +0x8C); 0215E5B0(gd, setup); 0216EE7C(W); 0216EF84(W); 0202C83C(W+0, 02012AC8(020127A4(gd)))
fn 0216eff0 ret=none inputs=400
  nowork
  arg buf 0x100
  arg u32 any
  arg buf 0x200
  fix arg0 8 u32 addrof arg2 0
  fix arg2 0 u32 alt range 0 1 || range 0 4
  fix arg2 0x8C u32 alt range 0 6 || oneof 1 3 4
  fix arg2 0x4C u32 addrof arg2 0x100
  stub 02012a4c force ret=none args=v,v  # GAMEDATA +0x1BC = result
  stub 0215e5b0 force ret=none args=v,v  # move_pokemon: roamer after the battle (gd, setup)
  stub 0216ee7c ret=none args=v  # in-area
  stub 0216ef84 ret=none args=v  # in-area
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER
  stub 02012ac8 force ret=u16 args=v  # PLAYER u16 +0
  stub 0202c83c force ret=none args=v,v  # (gsys, x)
  # descendants
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012f2c force ret=u32 args=v  # GAMEDATA -> record block
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)
  stub 02012934 force ret=u32 args=v  # GAMEDATA -> party
  stub 0201ab18 force ret=u16 args=v  # party: lead index
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, i)
  stub 020457b0 force ret=u32 args=v,v  # STRBUF alloc (12, heap)
  stub 02017e1c force ret=pick+u32:0/1/2/3/4/0x10000/0x10001 args=v,v,v  # PokeParam get (mon, id, x)
  stub 021654ac force ret=none args=v  # game_beacon (X1): fainted
  stub 02165424 force ret=none args=v  # game_beacon (X1): low HP
  stub 02165468 force ret=none args=v  # game_beacon (X1): no PP
  stub 021654f0 force ret=none args=v  # game_beacon (X1): status
  stub 02045808 force ret=none args=v  # STRBUF free
  stub 0216ee20 ret=none args=v,v  # in-area
  stub 0216ee40 ret=none args=v,v  # in-area
  stub 0216edfc ret=pick:0/1/2 args=v  # in-area
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 0216505c force ret=none args=v  # game_beacon (X1)
  stub 02165078 force ret=none args=v  # game_beacon (X1)
  stub 021653f4 force ret=none args=v  # game_beacon (X1)
  stub 0202a230 force ret=u32 args=v,v  # (trainer id, 1) -> trainer data
  stub 0202a3fc force ret=pick+u32:2/9/12/0/1/3 args=v  # trainer class kind
  stub 02165174 force ret=none args=v  # game_beacon (X1) (table 0x021725C0)
  stub 021651ac force ret=none args=v
  stub 021651e4 force ret=none args=v
  stub 0216521c force ret=none args=v
  stub 02165254 force ret=none args=v
  stub 0216528c force ret=none args=v

# 0216F02C(W, gsys, setup): MIi_CpuClear32(0, W, 0x28); W+0 = gsys, W+4 = 020120F4(gsys), W+8 = setup, W+0x14 = W+0x18 = W+0x1C = 0
fn 0216f02c ret=none inputs=60
  nowork
  arg buf 0x100
  arg u32 any
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA

# 0216F054(W): W+0x20 == 0: 020130B4(W+8) (free the setup)
fn 0216f054 ret=none inputs=60
  nowork
  arg buf 0x100
  fix arg0 0x20 u32 alt const 0 || any
  stub 020130b4 force ret=none args=v  # btl_setup free (black-field btl_setup)

# 0216F064 / 0216F070 / 0216F07C: the integrity-check callbacks: 0203F0A8() twice (OS tick / random source; result unused by the
# callers, which only compare the callback address)
fn 0216f064 ret=none inputs=20
  nowork
  stub 0203f0a8 force ret=u32 args=

fn 0216f070 ret=none inputs=20
  nowork
  stub 0203f0a8 force ret=u32 args=

fn 0216f07c ret=none inputs=20
  nowork
  stub 0203f0a8 force ret=u32 args=
