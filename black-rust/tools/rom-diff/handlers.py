"""Generic differential driver for battle event handlers. Mirrors handlers/testkit.rs.

Usage: python -I handlers.py abilities
       python -I handlers.py items [--list]
       python -I handlers.py moves [--list]
       python -I handlers.py effects [--list]   (B7h: overlay-95 field / position / side effect handlers)
"""
import os
import re
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from emu import Emu, Fnv, Rng  # noqa: E402
from bpp import Env, mix, random_bpp, SIZE  # noqa: E402

M = 0xFFFFFFFF
EV = 0x021F3ECC
POOL = 0x021F4470
RNGCTX = 0x021F6368
FLOW = 0x02E50000
MONS_ADDR = 0x02E30000
TMP = 0x02E60000
HX = 0x02E70000
PARTY = 0x02E80000
EXCL = 0x02E90000
SCRATCH = 0x02EA0000
SCRATCH_PTR = 0x021F6364
STRP = 0x02EB0000  # string parameter buffer; event variable 0x3F holds its address (moves)
STRP_SIZE = 0x40
NMONS = 8
HANDEX_SIZE = 0x80
ITEM_REGISTRY = 0x021F1EA4
MOVE_REGISTRY = 0x021F3030
MOVE_COUNT = 258
OVY_BASE = 0x021B6100
TYPE_ITEM = 5
TYPE_MOVE = 0
LABELS = [2, 3, 4, 5, 6, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 27, 29, 30, 31, 32, 33,
          34, 36, 37, 38, 39, 40, 42, 43, 44, 45, 46, 48, 49, 50, 51, 53, 55, 56, 57, 58, 62, 63,
          64, 65, 66, 68, 69, 70, 71, 72, 75, 77, 81, 83]


def item_registry(ovy):
    """(keys, by_fn): the item IDs of the registry at 0x021F1EA4 in registry order, and for each
    handler address (without the Thumb bit) the item IDs using it in registry order, then table
    order. Every getter is `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr` (checked)."""
    keys, by_fn, events = [], {}, {}
    i = 0
    while True:
        k, g = struct.unpack_from("<II", ovy, ITEM_REGISTRY - OVY_BASE + 8 * i)
        if k == 0 and g == 0:
            return keys, by_fn, events
        i += 1
        o = (g & ~1) - OVY_BASE
        mov, st, ldr, bx = struct.unpack_from("<4H", ovy, o)
        assert mov >> 8 == 0x21 and st == 0x6001 and ldr >> 8 == 0x48 and bx == 0x4770, hex(g)
        table = struct.unpack_from("<I", ovy, ((o + 8) & ~3) + (ldr & 0xFF) * 4)[0]
        keys.append(k)
        for j in range(mov & 0xFF):
            ev, fn = struct.unpack_from("<II", ovy, table - OVY_BASE + 8 * j)
            fn &= ~1
            if k not in by_fn.setdefault(fn, []):
                by_fn[fn].append(k)
            events.setdefault(fn, set()).add(ev)


def move_registry(ovy):
    """(keys, by_fn, events) like `item_registry` for the move registry at 0x021F3030: 258
    `(u16 move ID, pad, getter)` entries, stride 8. The 4 getters of moves 87, 100, 217 and 542
    have the same `movs/str/ldr/bx` shape (their `ldr` offset is 0, the table is just not a
    pool-aligned literal), so one decoder covers all (checked)."""
    keys, by_fn, events = [], {}, {}
    for i in range(MOVE_COUNT):
        k, g = struct.unpack_from("<HxxI", ovy, MOVE_REGISTRY - OVY_BASE + 8 * i)
        o = (g & ~1) - OVY_BASE
        mov, st, ldr, bx = struct.unpack_from("<4H", ovy, o)
        assert mov >> 8 == 0x21 and st == 0x6001 and ldr >> 8 == 0x48 and bx == 0x4770, hex(g)
        table = struct.unpack_from("<I", ovy, ((o + 8) & ~3) + (ldr & 0xFF) * 4)[0]
        keys.append(k)
        for j in range(mov & 0xFF):
            ev, fn = struct.unpack_from("<II", ovy, table - OVY_BASE + 8 * j)
            fn &= ~1
            if k not in by_fn.setdefault(fn, []):
                by_fn[fn].append(k)
            events.setdefault(fn, set()).add(ev)
    return keys, by_fn, events


# B7h: overlay-95 effect handlers (`handlers.py effects`). Overlay 95 is linked at 0x06898020
# (`OVY_95` of main.elf), so its code is loaded there next to overlay 93.
OVY95_BASE = 0x06898020
# (kind, registry address, entries, stride, factor type): `field` = btl_field.c slots (factor type
# 3, 8 entries `(u32 key, getter)`), `pos` = position effects (type 1, 5 entries), `side` = side
# effects (type 2, 14 entries `(u32 key, getter, u32 max layers)`; located by the literal at
# 0x06898cb8, the table `FUN_06898c10` reads).
FX_REGISTRIES = [("field", 0x0689D650, 8, 8, 3), ("pos", 0x0689D6C8, 5, 8, 1), ("side", 0x0689D780, 14, 12, 2)]
FX_TYPE = {kind: ty for kind, _, _, _, ty in FX_REGISTRIES}
FX_ALT_BD28 = 0x068993B0  # effect handler whose hint makes 0201bd28 answer 4 / 5 (the plain stub does so only for 1 key in 4, see wide_param)
AB_ALT_89B8 = 0x021D89B8  # V-0015: Slow Start; its hint keys 021c804c on poke ^ wseed (by poke ID alone only poke 0 is found, with action kind 3)
FX_STUB_DE0 = 0x06898DE0  # side-effect layer count (reads the 0x0689E7E0 state array): stubbed, tag 0x155


def wide(x, salt, small, mask, signed=False):
    """V-0007: return domain of the stubbed black-pokemon `waza` getters (0201BD28 and its wrappers). 3 of 4
    keys keep the small, branch-friendly answer; the 4th gets any value of the real record field (`mask`
    bits, sign-extended when `signed`), so handler branches on rarer values are reachable without hints."""
    if mix(x, salt ^ 0x10000) % 4 != 0:
        return small
    v = mix(x, salt ^ 0x20000) & mask
    return (v - (mask + 1) if signed and v > mask >> 1 else v) & M


def wide_param(mv, prm):
    """0201BD28(mv, prm): the domain of each parameter as `waza::record_param` reads it from the record."""
    small = mix(mv, prm ^ 0xBD28) % 3
    if prm in (6, 17, 20, 23):
        return wide(mv, prm ^ 0xBD28, small, 0xFF, signed=True)
    if prm in (8, 9):
        return wide(mv, prm ^ 0xBD28, small, 0xF)
    if prm in (11, 28):
        return wide(mv, prm ^ 0xBD28, small, 0xFFFF)
    if prm in (25, 26):
        return wide(mv, prm ^ 0xBD28, small, 0x7F)
    if prm in (30, 31):
        return wide(mv, prm ^ 0xBD28, small, 0xFF) % 129
    if prm == 29 or prm > 31:
        return 0
    v = wide(mv, prm ^ 0xBD28, small, 0xFF)
    return 100 if prm == 4 and v == 101 else 0 if prm == 7 and v == 6 else v


def load_ovy95():
    return open(os.path.join(os.environ.get(
        "BLACK_ROM_BUILD", os.path.join(os.path.dirname(__file__), "../../../black-rom/build/black.us")),
        "OVY_95.sbin"), "rb").read()


def fx_registry(ovy95):
    """(by_fn, keys, events, maxes) of the overlay-95 effect registries, like `move_registry`.
    `by_fn[fn]` = (kind, [keys using it in registry order, then table order]); `keys[kind]` = the
    keys of the registry in order; `events[fn]` = set of event IDs; `maxes[key]` = max layers of
    the side effect (the third field of a side registry entry). Every getter is
    `movs r1,#n; str r1,[r0]; ldr r0,[pc,#k]; bx lr` (asserted)."""
    def r32(a):
        return struct.unpack_from("<I", ovy95, a - OVY95_BASE)[0]

    by_fn, keys, events, maxes = {}, {}, {}, {}
    for kind, reg, n, stride, _ in FX_REGISTRIES:
        keys[kind] = []
        for i in range(n):
            k, g = r32(reg + stride * i), r32(reg + stride * i + 4)
            if kind == "side":
                maxes[k] = r32(reg + stride * i + 8)
            o = (g & ~1) - OVY95_BASE
            mov, st, ldr, bx = struct.unpack_from("<4H", ovy95, o)
            assert mov >> 8 == 0x21 and st == 0x6001 and ldr >> 8 == 0x48 and bx == 0x4770, hex(g)
            table = r32(((g & ~1) + 8 & ~3) + (ldr & 0xFF) * 4)
            keys[kind].append(k)
            for j in range(mov & 0xFF):
                ev, fn = struct.unpack_from("<II", ovy95, table - OVY95_BASE + 8 * j)
                fn &= ~1
                ent = by_fn.setdefault(fn, (kind, []))
                assert ent[0] == kind, hex(fn)  # no handler is shared between registries
                if k not in ent[1]:
                    ent[1].append(k)
                events.setdefault(fn, set()).add(ev)
    return by_fn, keys, events, maxes


class Host:
    def __init__(self, e, moves=False, items=False, effects=False):
        self.e = e
        self.log = []
        self.nbuf = 0
        self.nparty = 0
        self.wseed = 0
        st = e.stub
        st(0x021C88A4, self.push)
        st(0x021C88C8, self.pop)
        st(0x021C88B4, self.push_pop)
        st(0x021C7D00, self.poke_pos)
        st(0x021C8120, lambda em, wk, size, *a, sp: (em.wbytes(TMP, bytes(max(16, size))), TMP)[1])
        st(0x021C7F68, self.expand)
        st(0x021C8288, self.weather)
        st(0x0201C1E8, self.category)
        st(0x021C804C, self.find_action)
        st(0x0689C9B0, lambda em, ab, *a, sp: (self.log.extend([0x115, ab]), mix(ab, 0xC9B0) & 1)[1])
        st(0x0689C8EC, lambda em, mv, *a, sp: (self.log.extend([0x116, mv]), int(mix(mv, 0xC8EC) % 4 == 0))[1])
        st(0x021C8290, lambda em, wk, at, de, mv, sp: (self.log.extend([0x117, at, de, mv]), int(mix(at ^ (de << 8), mv ^ 0x8290) % 4 == 0))[1])
        st(0x021C7D54, self.opponents)
        st(0x0201C1D8, lambda em, mv, *a, sp: (self.log.extend([0x111, mv]), wide(mv, 0xC1D8, [0, 1, 1, 40, 60, 90, 120, 250][mix(mv, 0xC1D8) % 8], 0xFF))[1])
        st(0x0201C1BC, lambda em, mv, *a, sp: (self.log.extend([0x112, mv]), wide(mv, 0xC1BC, mix(mv, 0xC1BC) % 12, 0xFF))[1])
        st(0x0201C314, lambda em, mv, *a, sp: (self.log.extend([0x113, mv]), mix(mv, 0xC314) & 1)[1])
        st(0x0201C1C8, lambda em, mv, *a, sp: (self.log.extend([0x114, mv]), wide(mv, 0xC1C8, mix(mv, 0xC1C8) % 18, 0xFF))[1])
        st(0x021C83D4, lambda em, *a, sp: (self.log.append(0x10C), mix(self.wseed, 0x83D4) & 1)[1])
        st(0x0689C944, lambda em, ab, *a, sp: (self.log.extend([0x10D, ab]), int(mix(ab, 0xC944) % 4 == 0))[1])
        st(0x021C815C, lambda em, *a, sp: (self.log.append(0x10E), mix(self.wseed, 0x815C) % 4)[1])
        st(0x021B9A6C, lambda em, x, y, *a, sp: (self.log.extend([0x10F, x, y]), mix(x ^ (y << 8), 0x9A6C) & 1)[1])
        st(0x021C82B8, lambda em, wk, x, *a, sp: (self.log.extend([0x10B, x]), mix(self.wseed, 0x82B8 ^ x) & 1)[1])
        st(0x0201BD28, lambda em, mv, prm, *a, sp: (self.log.extend([0x109, mv, prm]), 4 + mix(mv, prm ^ 0xBD29) % 2 if self.alt_hint == FX_ALT_BD28 else wide_param(mv, prm))[1])
        st(0x0201BEB8, lambda em, mv, fl, *a, sp: (self.log.extend([0x10A, mv, fl]), mix(mv, fl ^ 0xBEB8) & 1)[1])
        st(0x021C8010, self.pos_in_battle)
        st(0x021C7CF4, lambda em, wk, i, *a, sp: MONS_ADDR + 0x200 * i if i < NMONS else 0)
        st(0x021C8234, lambda em, wk, p, *a, sp: (self.log.extend([0x118, p]), mix(p, 0x8236 if self.alt_hint == 0x021E7634 else 0x8234) & 1)[1])
        st(0x0689C8DC, lambda em, mv, *a, sp: (self.log.extend([0x119, mv]), mix(mv, 0xC8DC) & 1)[1])
        st(0x0689C8CC, lambda em, mv, *a, sp: (self.log.extend([0x11A, mv]), mix(mv, 0xC8CC) & 1)[1])
        st(0x0689C980, lambda em, it, *a, sp: (self.log.extend([0x11B, it]), mix(it, 0xC980) & 1)[1])
        st(0x0689C990, lambda em, it, *a, sp: (self.log.extend([0x11C, it]), mix(it, 0xC990) & 1)[1])
        st(0x021C80AC, lambda em, wk, p, *a, sp: (self.log.extend([0x11D, p]), mix(self.wseed ^ p, 0x80AC) & 1)[1])
        st(0x021C8168, lambda em, *a, sp: (self.log.append(0x11E), mix(self.wseed, 0x8168) & 1)[1])
        st(0x021B8DDC, lambda em, p, *a, sp: (self.log.extend([0x11F, p]), mix(p, 0x8DDC) % 3)[1])
        st(0x021C8434, lambda em, wk, p, at, *a, sp: self.log.extend([0x120, p, at]))
        st(0x020213C0, lambda em, it, *a, sp: (self.log.extend([0x121, it]), mix(it, 0x13C0) & 1)[1])
        st(0x0201C020, lambda em, mv, *a, sp: (self.log.extend([0x122, mv]), mix(mv, 0xC020) % 4)[1])
        st(0x0201C0BC, self.mv_entry)
        st(0x021B8C5C, lambda em, rule, pos, *a, sp: (self.log.extend([0x124, rule, pos]), mix(rule ^ (pos << 8), 0x8C5C) % 6)[1])
        st(0x021C8020, lambda em, wk, pos, *a, sp: (self.log.extend([0x125, pos]), mix(pos, 0x8020) % NMONS)[1])
        # held-item handlers (handlers.py items); tags 0x126+
        st(0x021C7EC4, lambda em, wk, *a, sp: (self.log.append(0x126), mix(self.wseed, 0x7EC4) & 1)[1])
        st(0x021C7F24, lambda em, wk, p, *a, sp: (self.log.extend([0x127, p]), mix(p, 0x7F26 if self.alt_dfe28 else 0x7F24) & 1)[1])
        st(0x021C818C, lambda em, wk, p, *a, sp: (self.log.extend([0x128, p]), mix(p, 0x818C) & 1)[1])
        self.alt_hint = 0  # Z18c: address of the move handler whose hint swaps a stub salt (0 = none): 021e7634 -> 021c8234 uses 0x8236 (with 0x8234 it always agrees with 021c8374); 021e6e38 -> 021c804c's result uses 0x804E (with 0x804D it is the opposite of 021c814c)
        self.alt_dfe28 = False  # harness hint for item handler 021dfe28 (see run()): other salts for 021c830c / 021c7f24
        st(0x021C830C, lambda em, wk, *a, sp: (self.log.append(0x129), mix(self.wseed, 0x830D if self.alt_dfe28 else 0x830C) & 1)[1])
        st(0x021C8374, lambda em, wk, p, *a, sp: (self.log.extend([0x12A, p]), mix(p, 0x8374) & 1)[1])
        st(0x021C8408, lambda em, wk, p, *a, sp: self.log.extend([0x12B, p]))
        st(0x021D805C, lambda em, it, prm, *a, sp: (self.log.extend([0x12C, it, prm]), mix(it, prm ^ 0x805C) & 0xFF)[1])
        st(0x0689C9E4, lambda em, i, *a, sp: (self.log.extend([0x12D, i]), mix(i, 0xC9E4) % 0x20 if i < 6 else 0)[1])
        st(0x020189D0, lambda em, pp, x, *a, sp: (self.log.extend([0x12E, pp, x]), M if mix(pp, x ^ 0x89D0) % 3 == 0 else mix(pp, x ^ 0x89D1) % 5)[1])
        if moves or effects:
            self.move_stubs()
        if effects:
            # B7h: callee of two side-effect handlers (tag 0x155; meaning guessed, see the Rust doc)
            st(FX_STUB_DE0, self.layer_count)
        elif items and not moves:
            # also a move stub (tag 0x154); item handler 021dfe28 calls it
            st(0x021C7F4C, lambda em, wk, *a, sp: (self.log.append(0x154), mix(self.wseed, 0x7F4C) & 1)[1])

    def move_stubs(self):
        """Callees of the move handlers (handlers.py moves); tags 0x12F+. Only installed in move
        mode, so the abilities and items runs keep executing the real code at these addresses."""
        st = self.e.stub
        lg = self.log.extend
        w = lambda: self.wseed
        # flow work queries (r0 = flow work, not logged)
        st(0x021C7D34, lambda em, wk, p, *a, sp: (lg([0x12F, p]), mix(p, 0x7D34) % 7)[1])
        st(0x021C7D44, lambda em, wk, *a, sp: (lg([0x130]), 0x1F if mix(w(), 0x7D44) % 4 == 0 else mix(w(), 0x7D45) % NMONS)[1])
        st(0x021C7ED8, lambda em, wk, p, *a, sp: (lg([0x131, p]), mix(p, 0x7ED8) % 4)[1])
        st(0x021C7FCC, self.party)
        st(0x021C7FE0, self.partner_party)
        st(0x021C8030, lambda em, wk, p, x, *a, sp: (lg([0x134, p, x]), mix(p, x ^ 0x8030) % 7)[1])
        st(0x021C8114, lambda em, wk, *a, sp: (lg([0x135]), 0 if mix(w(), 0x8114) % 4 == 0 else mix(w(), 0x8115) % 0x230)[1])
        st(0x021C8138, lambda em, wk, *a, sp: (lg([0x136]), mix(w(), 0x8138) % 16)[1])
        st(0x021C814C, lambda em, wk, p, *a, sp: (lg([0x137, p]), mix(p, 0x814C) & 1)[1])
        st(0x021C8174, lambda em, wk, *a, sp: (lg([0x138]), mix(w(), 0x8174) % 17)[1])
        st(0x021C8180, lambda em, wk, *a, sp: (lg([0x139]), mix(w(), 0x8180) % 4)[1])
        st(0x021C81C8, self.mon_stat)
        st(0x021C824C, lambda em, wk, p, *a, sp: (lg([0x13B, p]), mix(p, 0x824C) & 1)[1])
        st(0x021C82C4, lambda em, wk, side, kind, *a, sp: (lg([0x13C, side, kind]), mix(side, kind ^ 0x82C4) & 1)[1])
        st(0x021C8300, lambda em, wk, x, *a, sp: (lg([0x13D, x]), mix(x, 0x8300) % 5)[1])
        st(0x021C831C, lambda em, wk, p, *a, sp: lg([0x13E, p & 0xFF]))
        st(0x021C83A0, lambda em, wk, p, *a, sp: (lg([0x13F, p]), 1 + mix(p, 0x83A0) % 3000)[1])
        st(0x021C83DC, lambda em, wk, v, *a, sp: lg([0x140, v & 0xFF]))
        st(0x021C83E8, lambda em, wk, n, *a, sp: (lg([0x141, n]), mix(n, 0x83E8) & 1)[1])
        st(0x021C846C, self.fun_021c846c)
        st(0x021C8494, lambda em, wk, p, *a, sp: lg([0x143, p]))
        st(0x021C2E4C, lambda em, wk, *a, sp: (lg([0x144]), mix(w(), 0x2E4C))[1])
        st(0x021C2AB0, self.weather)
        st(0x021C7F4C, lambda em, wk, *a, sp: (lg([0x154]), mix(w(), 0x7F4C) & 1)[1])
        # overlay 95 queries (no flow work; ring pointer arguments are not logged)
        st(0x06898CF4, lambda em, side, kind, *a, sp: (lg([0x145, side, kind]), mix(side, kind ^ 0x8CF4) & 1)[1])
        st(0x0689C8AC, lambda em, mv, *a, sp: (lg([0x146, mv]), mix(mv, 0xC8AC) & 1)[1])
        st(0x0689C8BC, lambda em, mv, *a, sp: (lg([0x147, mv]), mix(mv, 0xC8BC) & 1)[1])
        st(0x0689C91C, lambda em, ab, *a, sp: (lg([0x148, ab]), mix(ab, 0xC91C) & 1)[1])
        st(0x0689C93C, lambda em, ab, *a, sp: (lg([0x149, ab]), mix(ab, 0xC93C) & 1)[1])
        st(0x0689CA1C, lambda em, mv, *a, sp: (lg([0x14A, mv]), mix(mv, 0xCA1C) & 1)[1])
        st(0x0689CA3C, lambda em, mv, *a, sp: (lg([0x14B, mv]), mix(mv, 0xCA3C) & 1)[1])
        st(0x0689CA5C, lambda em, mv, *a, sp: (lg([0x14C, mv]), mix(mv, 0xCA5C) & 1)[1])
        st(0x0689CA7C, lambda em, mv, *a, sp: (lg([0x14D, mv]), mix(mv, 0xCA7C) & 1)[1])
        st(0x0689CA8C, self.excluded_moves)
        st(0x0689D064, lambda em, ring, mv, i, *a, sp: (lg([0x14F, mv, i]), mix(mv, i ^ 0xD064) & 1)[1])
        st(0x0689D098, lambda em, ring, mv, t, *a, sp: (lg([0x150, mv, t]), mix(mv, t ^ 0xD098) % 4)[1])
        st(0x0689D0C8, self.last_move)
        st(0x0689D174, lambda em, ring, i, *a, sp: (lg([0x152, i]), mix(i, 0xD174) % 7 if i < 4 else 0)[1])
        st(0x0689D184, lambda em, ring, i, j, *a, sp: (lg([0x153, i, j]), (0x1F if mix(i, j ^ 0xD184) % 4 == 0 else mix(i, j ^ 0xD185) % 12) if i < 4 else 0x1F)[1])

    def layer_count(self, e, fac, side, *_, sp):
        """06898de0(factor, side): the layer count of the factor's side effect on `side`
        (low byte of the 0x0689E7E0 + side * 0xE0 + key * 0x10 state word; 0 past key 14 / side 1).
        Stub result: 0..3 from the sub ID (u16 at factor + 0x38) and the side."""
        sub = struct.unpack("<H", e.rbytes(fac + 0x38, 2))[0]
        self.log += [0x155, sub, side]
        return mix(sub ^ (side << 16), 0x8DE0) % 4 if side < 2 else 0

    def party(self, e, wk, poke, *_, sp):
        """021c7fcc: the party of the Pokemon's side: a struct of mon pointers (+0..) and a count
        byte at +0x18, read by the real 021b9b94 / 021b9c00. Members are mons 0..7."""
        self.log += [0x132, poke]
        return self.write_party(e, mix(poke, 0x7FCC) % 7, poke, 0x7FCD)

    def partner_party(self, e, wk, poke, *_, sp):
        """021c7fe0: like `party`, or null (a third of the time)."""
        self.log += [0x133, poke]
        if mix(poke, 0x7FE0) % 3 == 0:
            return 0
        return self.write_party(e, mix(poke, 0x7FE1) % 7, poke, 0x7FE2)

    def write_party(self, e, n, poke, salt):
        a = PARTY + 0x40 * self.nparty
        self.nparty += 1
        b = bytearray(0x20)
        for i in range(n):
            struct.pack_into("<I", b, 4 * i, MONS_ADDR + 0x200 * (mix(poke, salt + i) % NMONS))
        b[0x18] = n
        e.wbytes(a, bytes(b))
        return a

    def mon_stat(self, e, wk, mon, x, *_, sp):
        """021c81c8(flow, bpp, 0): a stat of a battle Pokemon (speed-like, 1..2048)."""
        pid = (mon - MONS_ADDR) // 0x200 if MONS_ADDR <= mon < MONS_ADDR + NMONS * 0x200 else 0xFF
        self.log += [0x13A, pid, x]
        return 1 + mix(pid, x ^ 0x81C8) % 0x800

    def fun_021c846c(self, e, wk, poke, tgt, out, *_, sp):
        self.log += [0x142, poke, tgt]
        e.w32(out, mix(poke, tgt ^ 0x846D) & 1)
        return mix(poke, tgt ^ 0x846C) & 1

    def excluded_moves(self, e, out, *_, sp):
        """0689ca8c(out): 0x29 move IDs (a list of moves, e.g. excluded from a random pick)."""
        self.log.append(0x14E)
        e.w32(out, 0x29)
        e.wbytes(EXCL, b"".join(struct.pack("<H", mix(i, 0xCA8C) % 0x230) for i in range(0x29)))
        return EXCL

    def last_move(self, e, ring, t, *_, sp):
        """0689d0c8(ring, turn): the move used on that turn (0 = none)."""
        self.log += [0x151, t]
        k = mix(t, 0xD0C8) % 4
        return [0, 558, 559, mix(t, 0xD0C9) % 0x230][k]

    def push(self, e, wk, ty, poke, *_, sp):
        self.log += [0x100, ty, poke]
        a = HX + 0x100 * self.nbuf
        self.nbuf += 1
        e.wbytes(a, bytes(HANDEX_SIZE))
        return a

    def pop(self, e, wk, a, *_, sp):
        self.log.append(0x101)
        self.log += list(struct.unpack(f"<{HANDEX_SIZE // 4}I", e.rbytes(a, HANDEX_SIZE)))

    def mv_entry(self, e, mv, index, out, *_, sp):
        self.log += [0x123, mv, index]
        e.w32(out, wide(mv ^ (index << 16), 0xC0BC, ((mix(mv, index ^ 0xC0BC) % 5) - 1) & M, 0xFF, signed=True))

    def push_pop(self, e, wk, ty, poke, *_, sp):
        self.log += [0x102, ty, poke]

    def poke_pos(self, e, wk, poke, *_, sp):
        self.log += [0x103, poke]
        return mix(poke, 0x7D00) % 6

    def expand(self, e, wk, code, buf, *_, sp):
        self.log += [0x104, code]
        n = mix(code, 0x7F68) % 7
        e.wbytes(buf, bytes(mix(code, i) % NMONS for i in range(n)))
        return n

    def weather(self, e, wk, *_, sp):
        self.log.append(0x105)
        return mix(self.wseed, 0x2AB0) % 5

    def category(self, e, mv, *_, sp):
        self.log += [0x106, mv]
        return wide(mv, 0xC1E8, mix(mv, 0xC1E8) % 3, 0xFF)


def _find_action(self, e, wk, poke, out, *_, sp):
    self.log += [0x107, poke]
    if self.alt_hint == AB_ALT_89B8:
        k = (poke ^ self.wseed) & M
        e.w32(out, mix(k, 0x89BA))
        return mix(k, 0x89B9) & 1
    e.w32(out, mix(poke, 0x804C))
    return mix(poke, 0x804E if self.alt_hint == 0x021E6E38 else 0x804D) & 1


def _pos_in_battle(self, e, wk, poke, *_, sp):
    self.log += [0x108, poke]
    return mix(poke, 0x8010) % 7


def _opponents(self, e, wk, poke, buf, *_, sp):
    self.log += [0x110, poke]
    n = mix(poke, 0x7D54) % 4
    e.wbytes(buf, bytes(mix(poke, 0x7D55 + i) % NMONS for i in range(n)))
    return n


Host.opponents = _opponents
Host.find_action = _find_action
Host.pos_in_battle = _pos_in_battle


def T(me):
    return (me + 1) % NMONS


K = 0x55  # a move ID used by the Z18c hints
# Z18c: poke IDs >= 12 are on the far side (021b9a24 compares `id >= 12`), but the harness has only
# NMONS = 8 mons (7cf4 returns 0 for the rest): FAR is such an ID, whose mon is the "null mon" at
# address 0 (hint mon index NULLMON), so its fields can be written (HP != 0).
FAR = 12
NULLMON = -1


def _val(salt, want):
    """Smallest x >= 1 with mix(x, salt) & 1 == want (a stub argument that makes the stub answer)."""
    return next(x for x in range(1, 100) if mix(x, salt) & 1 == want)


def _pick(me, conds):
    """The first mon after `me` (wrapping, `me` last) with mix(mon, salt) & 1 == want for every
    (salt, want) in `conds`; T(me) if none."""
    for i in range(1, NMONS + 1):
        p = (me + i) % NMONS
        if all(mix(p, salt) & 1 == want for salt, want in conds):
            return p
    return T(me)


def _knows(mon, mv, pp):
    """Mon bytes: `mon` has one move `mv` (slot 0) with `pp` PP."""
    return [(mon, 0x140, 1, 1), (mon, 0x10A, 2, mv), (mon, 0x10C, 1, pp)]


def _partner(mv, slot):
    """A move that 021e8384 pairs with `mv` (0x206 + 0x208 in slot 3, else 0x207 / 0x206)."""
    if slot == 3 and mv in (0x206, 0x208):
        return 0x206 + 0x208 - mv
    return 0x206 if mv == 0x207 else 0x207


def _targets(me):
    """Event target IDs for vars 6, 7, 8: the next three mons after `me`."""
    return [(6, (me + 1) % NMONS), (7, (me + 2) % NMONS), (8, (me + 3) % NMONS)]


# Z18 harness hints for move handlers whose tails are gated on state the random inputs almost
# never produce (found with coverage.py). Applied to 3 of 4 seeds (seed % 4 != 0; the rest keep
# the plain random inputs), after every draw (no extra draws), identically in
# handlers/testkit.rs (`move_hint`). Each entry returns (event variables forced as (label, value),
# kind 0; mon bytes written as (mon, offset, width, value)) from `me` (the factor owner), `mv`
# (the factor's move ID) and `slot` (seed % 4, 1..3).
MOVE_HINTS = {
    # target (var 6) = the next mon; `me` knows the factor's move in a slot
    0x021E0998: lambda me, mv, slot: ([(3, me), (6, (me + 1) % NMONS)], [(me, 0x10A + 14 * slot, 2, mv)]),
    0x021E0A80: lambda me, mv, slot: ([(3, me), (6, (me + 1) % NMONS)], [(me, 0x10A + 14 * slot, 2, mv)]),
    # `me` has type 7 (0xF8 = type 1)
    0x021E0E28: lambda me, mv, slot: ([(3, me)], [(me, 0xF8, 1, 7)]),
    # var 0x47 == 0
    0x021E11DC: lambda me, mv, slot: ([(3, me), (0x47, 0)], []),
    # species (0x0C) == 0x1B9
    0x021E12A4: lambda me, mv, slot: ([(3, me)], [(me, 0x0C, 2, 0x1B9)]),
    # var 0x1D == 8
    0x021E1354: lambda me, mv, slot: ([(3, me), (0x1D, 8)], []),
    # the mon's byte 0x157 (u8_157(0)) < 3
    0x021E1668: lambda me, mv, slot: ([(3, me)], [(me, 0x157, 1, slot - 1)]),
    # Z18c. T = the next mon (var 6 / the target); K = a move ID (0x55) that T knows in slot 0 with
    # PP (move count 0x140, slot-0 move ID 0x10A, PP 0x10C); held item 0x12; HP 0x10, max HP 0x0E;
    # condition u32s at 0x1C + 4 * sick; flag A byte 0x153; work[0] = 0.
    0x021E36F4: lambda me, mv, slot: ([(3, me), (6, T(me))], [(me, 0x12, 2, 0), (T(me), 0x12, 2, 0x10)], [(0, 0)]),
    0x021E6728: lambda me, mv, slot: (
        [(4, me), (3, FAR), (0x12, _val(0xC8CC, 0))], [(me, 0x10, 2, 0), (NULLMON, 0x10, 2, 1)]),
    0x021E5B78: lambda me, mv, slot: ([(3, me)], []),
    0x021E20E0: lambda me, mv, slot: ([(4, me), (0x46, 0)], [(me, 0x10, 2, 1)]),
    0x021E6840: lambda me, mv, slot: (
        [(4, me), (3, T(me)), (0x12, _val(0xC8CC, 0)), (0x14, K)],
        [(me, 0x10, 2, 0)] + _knows(T(me), K + (slot == 3), 5)),
    0x021E8190: lambda me, mv, slot: ([(3, me)], [(me, 0x0C, 2, 0x288)]),
    0x021E3F44: lambda me, mv, slot: ([(3, me)], [(me, 0x12, 2, 0)]),
    0x021E4B00: lambda me, mv, slot: (
        [(3, me), (6, T(me))], [(T(me), 0x14A, 2, K)] + _knows(T(me), K, 2 * slot)),
    0x021E1EB8: lambda me, mv, slot: ([(2, me), (0x12, mv)], []),
    # 021c8374(t) false and 021c8234(t) true; condition 0x21 of t clear; 0x1E / 0x20 vary by slot
    0x021E7634: lambda me, mv, slot: (
        [(3, me), (6, _pick(me, [(0x8374, 0), (0x8236, 1)]))],
        [(_pick(me, [(0x8374, 0), (0x8236, 1)]), 0x1C + 4 * 0x21, 4, 0)]
        + [(_pick(me, [(0x8374, 0), (0x8236, 1)]), 0x1C + 4 * k, 4, 1 if (slot == 2 and k == 0x1E) or (slot == 3 and k == 0x20) else 0) for k in (0x1E, 0x20)]),
    0x021E25FC: lambda me, mv, slot: (
        [(2, me)], [(me, 0x1C + 4 * 0x19, 4, 0), (me, 0x1C + 4 * 2, 4, 0)], [(6, 0)]),
    0x021E4408: lambda me, mv, slot: ([(2, me), (0x22, 0)], []),
    0x021E2E28: lambda me, mv, slot: ([(3, me)], [(me, 0x14C, 2, mv), (me, 0x14E, 2, slot)]),
    0x021E4688: lambda me, mv, slot: (
        [(3, me), (6, T(me))],
        [(T(me), 0x14A, 2, K), (T(me), 0x1C + 4 * 0x17, 4, 0), (T(me), 0x153, 1, 2 * (slot == 2))]
        + _knows(T(me), K, 5)),
    0x021E4828: lambda me, mv, slot: (
        [(3, me), (6, T(me))],
        [(T(me), 0x14A, 2, K), (T(me), 0x1C + 4 * 0xD, 4, 0), (T(me), 0x153, 1, 2 * (slot == 2))]
        + _knows(T(me), K, 5)),
    # 021e8384(mv, u16 at 0x1F4) pairs 0x206/0x207, 0x208/0x207, 0x206/0x208 (the fusion moves)
    0x021E83CC: lambda me, mv, slot: (
        [(3, me)], [(me, 0x1F6, 1, slot), (me, 0x1F4, 2, _partner(mv, slot))]),
    0x021E7924: lambda me, mv, slot: ([(3, me), (0x3C, 0xB)], []),
    0x021E5DAC: lambda me, mv, slot: ([(3, me), (0x46, 0), (0x47, 0)], [(me, 0x12, 2, 0x10 * slot)]),
    0x021E2D5C: lambda me, mv, slot: ([(3, me)], _knows(me, mv + (slot == 3), 2 * slot - 1)),
    # var 0xE = 1 expands to mon 7 (alt salt for 021c804c: see Host.alt_hint); its flag A bit 3 clear
    0x021E6E38: lambda me, mv, slot: ([(2, me), (0xE, 6 if slot == 3 else 1)], [(7, 0x153, 1, 0)]),
    # Z18c: caps for the loops that drew a huge var 5 (target count) or work word. The targets are
    # position IDs in vars 6..11 (label 12 is a different variable), so at most 6; the factors are
    # only ever pushed for a handful of targets: 1..3 here, with the targets in slots 6, 7, 8.
    0x021E732C: lambda me, mv, slot: ([(3, me), (5, slot)] + _targets(me), []),
    0x021E7388: lambda me, mv, slot: ([(3, me), (5, slot)] + _targets(me), []),
    0x021E7420: lambda me, mv, slot: ([(3, me), (5, slot)] + _targets(me), []),
    # work[0] = hits so far (< work[1] = the cap, 3..5 for the multi-hit power doubling)
    0x021E2700: lambda me, mv, slot: ([(3, me)], [], [(0, slot - 1), (1, 2 + 2 * (slot == 2) + (slot == 3))]),
}


# Z18c: the handlers whose loop bound (var 5 = target count, or work word 0 for 021e2700) was a
# 32-bit draw that ran the original into the emulator's instruction cap. Their bound is capped on
# every seed: the hint for seed % 4 != 0, and for the plain seeds (seed % 4 == 0) the drawn value
# reduced modulo 4 (var 5) or 8 (work word 0).
CAP_ALL = (0x021E732C, 0x021E7388, 0x021E7420, 0x021E2700)


def apply_move_hint(e, addr, seed, me, mv):
    if addr in CAP_ALL and seed % 4 == 0:
        if addr == 0x021E2700:
            e.w32(POOL + 0x1C, e.r32(POOL + 0x1C) % 8)
        else:
            a = EV + 0xC4 + 4 * (LABELS.index(5) + 1)
            e.w32(a, e.r32(a) % 4)
        return
    if addr not in MOVE_HINTS or seed % 4 == 0:
        return
    h = MOVE_HINTS[addr](me, mv, seed % 4)
    vs, ms = h[0], h[1]
    extra = []  # Z18c: labels absent from LABELS get the next free slots (in order of appearance)
    for lab, v in vs:
        if lab in LABELS:
            s = LABELS.index(lab) + 1
        else:
            if lab not in extra:
                extra.append(lab)
                e.wbytes(EV + 4 + 2 * (len(LABELS) + len(extra)), struct.pack("<H", lab))
            s = len(LABELS) + 1 + extra.index(lab)
        e.w32(EV + 0xC4 + 4 * s, v)
        e.wbytes(EV + 0x544 + s, b"\0")
    for mon, off, width, v in ms:
        e.wbytes((0 if mon == NULLMON else MONS_ADDR + 0x200 * mon) + off, v.to_bytes(width, "little"))
    for k, v in (h[2] if len(h) > 2 else []):
        e.w32(POOL + 0x1C + 4 * k, v)


# Z18b harness hints for ability and item handlers whose branches are gated on state the random
# inputs almost never produce (found with coverage.py). Same rules as MOVE_HINTS (3 of 4 seeds,
# after every draw, no extra draws, only the listed handler addresses) and mirrored exactly in
# handlers/testkit.rs: `tools/rom-diff/genhints.py` prints the Rust table (`HINT_TABLE`) from
# HINTS. Moves keep MOVE_HINTS and never use these.
# Entry: {"vars": [(label, value)], "mons": [(who, offset, width, value)], "work": [(k, value)],
# "wseed": value}; `who` 2 = every mon (applied in list order, so later entries override).
# A value is an int or a tuple of alternatives picked by slot = seed % 4 (index (slot - 1) % len).
# Variables are forced as kind-0 values; `who` 0 = the factor owner `me`, 1 = mon (me + 1) % 8;
# `work` sets word k of the factor's work area (0..6).
ME, NXT = 0x7FFF0001, 0x7FFF0002  # variable values: the owner / the next mon
REL = 0x7FFF0010  # REL + k = mon (me + k) % 8


def _first(pred):
    return next(m for m in range(1, 0x230) if pred(m))


# Move IDs for which a stubbed move query returns the wanted value (the stubs are salted mixes).
MV_BEB8_1 = _first(lambda m: mix(m, 0xBEB8) & 1)  # 0201beb8(mv, 0) != 0 (item handlers; abilities too)
MV_C8CC_0 = _first(lambda m: not mix(m, 0xC8CC) & 1)  # 0689c8cc(mv) == 0


# Overlay-95 effect handlers (B7h): host.wseed values for the stubs 021c8138 / 021c83d4.
W_WISH = _first(lambda w: not mix(w, 0x83D4) & 1 and mix(w, 0x8138) % 16 > 3)
W_DELAYED = _first(lambda w: not mix(w, 0x83D4) & 1 and mix(w, 0x8138) % 16 > 3)
MV_C314_1 = _first(lambda m: mix(m, 0xC314) & 1)  # 0201c314(mv) != 0


W_WEATHER1 = _first(lambda w: mix(w, 0x2AB0) % 5 == 1)  # host.wseed with weather() == 1
IT_13C0 = _first(lambda it: mix(it, 0x13C0) & 1)  # 020213c0(item) != 0


def mv_cat(c):  # 0201c1e8(mv) == c
    return _first(lambda m: mix(m, 0xC1E8) % 3 == c)


def mv_cat_c8cc_0(c):
    return _first(lambda m: mix(m, 0xC1E8) % 3 == c and not mix(m, 0xC8CC) & 1)


def _sp(species, **kw):
    d = {"vars": [(3, ME)], "mons": [(0, 0x0C, 2, species)]}
    d.update(kw)
    return d


HINTS = {
    # --- item handlers ---
    0x021DED6C: {"vars": [(3, ME), (0x4D, 0), (0x32, 160)]},
    0x021DFABC: {"vars": [(4, ME), (0x46, 0), (0x12, MV_BEB8_1)]},
    # var 0x12 is the holder's last move (+0x14c) with a streak (+0x14e) >= 1
    0x021DEC70: {"vars": [(3, ME), (0x12, MV_C8CC_0)],
                 "mons": [(0, 0x14C, 2, MV_C8CC_0), (0, 0x14E, 2, (1, 0x40, 3))]},
    0x021DF9C8: _sp(0x1E3, vars=[(3, ME), (0x16, (0xF, 8, 0xF))]),
    0x021DF974: _sp(0x1E4, vars=[(3, ME), (0x16, (0xF, 10, 0xF))]),
    0x021DE74C: _sp(0x16E, vars=[(3, ME), (0x12, mv_cat(2))]),
    0x021DE8FC: {"vars": [(3, ME), (0x12, mv_cat(1))], "mons": [(0, 0x0C, 2, (0x68, 0x69, 0x68))]},
    0x021DF438: _sp(0x1E7, vars=[(3, ME), (0x16, (0xF, 7, 0xF))]),
    0x021DFD20: {"vars": [(4, ME), (0x46, 0), (0x16, 10)]},
    0x021DFDA4: {"vars": [(4, ME), (0x46, 0), (0x16, 0xC)]},
    0x021DED28: {"vars": [(3, ME), (0x1D, 8)]},
    0x021DE7E4: {"vars": [(4, ME), (0x1A, 1)], "mons": [(0, 0x0C, 2, 0x84)]},
    0x021DE598: _sp(0x71),
    0x021DE5DC: _sp(0x53),
    0x021DE8B4: {"vars": [(4, ME), (0x1A, 2)], "mons": [(0, 0x0C, 2, (0x17C, 0x17D, 0x17C))]},
    0x021DE79C: {"vars": [(4, ME), (0x1A, 2)], "mons": [(0, 0x0C, 2, 0x16E)]},
    0x021DE834: {"vars": [(2, ME)], "mons": [(0, 0x0C, 2, 0x84)]},
    # type 3 as the owner's first or second type
    0x021DEFC4: {"vars": [(2, ME)], "mons": [(0, 0xF8, 1, (3, 0, 3)), (0, 0xF9, 1, (0, 3, 3))]},
    # --- ability handlers ---
    # 021da8a8 gate: var 4 = owner, var 3 = another mon, var 0x16 (move type) = n
    0x021DAF58: {"vars": [(4, ME), (3, NXT), (0x16, 9)]},
    0x021DAB24: {"vars": [(4, ME), (3, NXT), (0x16, 0xC)]},
    0x021DB430: {"vars": [(4, ME), (3, NXT), (0x16, 10)]},
    0x021DB4C4: {"vars": [(4, ME), (3, NXT), (0x16, 0xC)]},
    0x021DCA68: {"vars": [(4, ME), (3, NXT), (0x16, 0xB)]},
    # every mon has the flag (+0x154 bit 0) and a held item (+0x14), the owner has none (+0x12)
    0x021DBBD8: {"vars": [(2, ME)],
                 "mons": [(2, 0x154, 1, 1), (2, 0x14, 2, 0x30), (0, 0x12, 2, 0)]},
    0x021DC9C8: {"vars": [(4, ME), (0x46, 0), (0x12, MV_BEB8_1), (3, (NXT, 12, 12))]},  # side = id > 11
    0x021DBD9C: {"vars": [(4, ME), (0x46, 0), (3, NXT), (0x14, MV_C8CC_0)]},
    0x021DC4D4: {"vars": [(3, ME), (0x46, 0), (0x47, 0), (0x12, MV_BEB8_1), (4, NXT)]},
    0x021DBE6C: {"vars": [(4, ME), (0x1A, 1), (0x46, 0)]},
    0x021DC738: {"vars": [(3, ME)], "mons": [(0, 0x0C, 2, 0x22B)]},
    0x021DC7BC: {"vars": [(2, ME)], "mons": [(0, 0x0C, 2, 0x22B)]},
    0x021D9F1C: {"vars": [(4, ME), (3, NXT), (0x12, MV_BEB8_1)], "mons": [(0, 0x10, 2, 0)]},
    0x021D89B8: {"vars": [(2, ME)], "work": [(2, 0), (1, (5, 4, 7))]},
    # V-0016: pinch boosts (helper 021d8cac): the attacker (var 3) is the owner and the move type
    # (var 0x16) is the boosted one (Blaze 9, Torrent 10, Overgrow 11; Swarm's 6 comes up anyway)
    0x021D8C3C: {"vars": [(3, ME), (0x16, 9)]},
    0x021D8C58: {"vars": [(3, ME), (0x16, 10)]},
    0x021D8C74: {"vars": [(3, ME), (0x16, 11)]},
    0x021DCC80: {"work": [(0, 0)]},
    0x021DB6A4: {"work": [(0, 0)]},
    0x021D8FCC: {"vars": [(3, NXT), (0x12, mv_cat(1))], "mons": [(0, 0x0C, 2, 0x1A5)], "wseed": W_WEATHER1},
    0x021D901C: {"vars": [(4, NXT), (0x1A, 2)], "mons": [(0, 0x0C, 2, 0x1A5)], "wseed": W_WEATHER1},
    0x021D8E1C: {"mons": [(0, 0x0C, 2, 0x1A5)]},
    0x021D8EDC: {"work": [(0, 1)], "mons": [(0, 0x0C, 2, 0x1A5)]},
    0x021D8F84: {"vars": [(2, ME)], "work": [(0, 1)], "mons": [(0, 0x0C, 2, 0x1A5)]},
    0x021DCB38: {"vars": [(2, ME)], "mons": [(0, 0x14, 2, IT_13C0), (0, 0x12, 2, 0)]},
    0x021D95CC: {"vars": [(4, ME), (0x12, 0x1BD)]},
    0x021DB7E8: {"vars": [(4, ME), (0x12, (0x10F, 0x19F, 0x10F))]},
    0x021DC908: {"vars": [(4, ME), (0x46, 0), (0x16, 0x10)]},
    0x021DC960: {"vars": [(4, ME), (0x46, 0), (0x16, (0x10, 6, 7))]},
    0x021DAE04: {"vars": [(0x12, (0x99, 0x78, 0x99)), (0x22, 0)]},
    0x021DB734: {"vars": [(2, ME), (0x12, 0x16B), (0x22, 0)]},
    0x021DBB98: {"vars": [(2, ME)], "work": [(0, 1)], "mons": [(0, 0x12, 2, (0, 0, 5))]},
    0x021DBB64: {"vars": [(2, ME), (0x2D, 0)], "mons": [(0, 0x12, 2, (5, 5, 0))]},
    # some mon other than the owner has ability 0x39 / 0x3A (+0x13C) and no sick-cont 0x10 bits
    0x021D8D40: {"vars": [(3, ME), (0x12, mv_cat(2))],
                 "mons": [(2, 0x13C, 2, (0x39, 0x3A, 0x39)), (2, 0x5C, 4, 0)]},
    0x021DBCEC: {"vars": [(5, 1), (6, ME), (3, NXT), (0x12, MV_BEB8_1)],
                 "mons": [(0, 0x12, 2, 0), (1, 0x12, 2, 5)]},
    # --- overlay-95 effect handlers (B7h; owner = 0 / me % 6 / me & 1, see run()) ---
    # field slots 4 / 5: the move type (var 0x16) is 9 / 12
    0x06898110: {"vars": [(0x16, 9)]},
    0x06898158: {"vars": [(0x16, 0xC)]},
    # position effects: var 2 == 0x1F; work 0 = flag (set / not), 1 = turn stored; 6 = user ID
    0x06898328: {"vars": [(2, 0x1F)], "work": [(0, (1, 0, 1)), (1, 0)], "wseed": W_WISH},
    0x068989CC: {"vars": [(2, 0x1F)], "work": [(0, 0), (6, 0xFF)], "wseed": W_DELAYED},
    # side effects keys 0 / 1 (shared helper 06898e54): the mover (var 4) is on the owner's side,
    # damage class (var 0x1A) = 1 / 2, var 0x45 == 0
    0x06898E28: {"vars": [(4, ME), (0x1A, 1), (0x45, 0)]},
    0x06898E44: {"vars": [(4, ME), (0x1A, 2), (0x45, 0)]},
    # 0201bd28(mv, 0x1b) must answer 4 or 5 (FX_ALT_BD28 swaps the stub), 0201c314(mv) != 0
    FX_ALT_BD28: {"vars": [(4, ME), (0x12, MV_C314_1)]},
}


def _alt(v, slot):
    return v[(slot - 1) % len(v)] if isinstance(v, tuple) else v


def hint_value(v, slot, me):
    v = _alt(v, slot)
    if REL <= v < REL + NMONS:
        return (me + v - REL) % NMONS
    return me if v == ME else (me + 1) % NMONS if v == NXT else v


def apply_hint(e, host, addr, seed, me):
    if addr not in HINTS or seed % 4 == 0:
        return
    h, slot = HINTS[addr], seed % 4
    for lab, v in h.get("vars", []):
        s = LABELS.index(lab) + 1
        e.w32(EV + 0xC4 + 4 * s, hint_value(v, slot, me))
        e.wbytes(EV + 0x544 + s, b"\0")
    for who, off, width, v in h.get("mons", []):
        for mon in range(NMONS) if who == 2 else [me if who == 0 else (me + 1) % NMONS]:
            e.wbytes(MONS_ADDR + 0x200 * mon + off, hint_value(v, slot, me).to_bytes(width, "little"))
    for k, v in h.get("work", []):
        e.w32(POOL + 0x1C + 4 * k, hint_value(v, slot, me))
    if "wseed" in h:
        host.wseed = _alt(h["wseed"], slot)


def value_for(r, me):
    s = r.next() % 8
    if s <= 2:
        return me
    if s <= 4:
        return r.next() % NMONS
    if s == 5:
        return r.next() % 0x200
    if s == 6:
        return 0x800 + r.next() % 0x1800
    return r.next()


def words(b):
    return struct.unpack(f"<{len(b) // 4}I", b)


def run(addrs, funcs=None, detail=None, items=False, hook=None, moves=False, cov=None, effects=False):
    """Hash `addrs`. `funcs` (dict) receives {fn_addr: {"hash", "n_inputs", "seeds"}}; `detail`
    is (addr, seed): returns the section-labelled word stream of that one input instead.
    `items`: held-item handlers (factor type 5, sub ID = a registry item ID).
    `moves`: move handlers (factor type 0, sub ID = a registry move ID).
    `effects`: overlay-95 effect handlers (B7h; factor type 3 field / 1 position / 2 side effect,
    sub ID = a registry key, owner = 0 / position / side; overlay 95 is loaded at 0x06898020).
    `cov` (dict, used by coverage.py): receives {handler address: set of executed instruction
    addresses inside overlays 93 and (effects) 95}; it does not change the hashes."""
    e = Emu()
    e.uc.mem_map(0, 0x10000)
    env = Env(e)
    host = Host(e, moves, items, effects)
    if hook:
        hook(e)
    cur = [None]
    if effects:
        ovy95 = load_ovy95()
        e.uc.mem_write(OVY95_BASE, ovy95)
        fx_by_fn, fx_keys, _, _ = fx_registry(ovy95)
    if cov is not None:
        from unicorn import UC_HOOK_CODE
        e.uc.hook_add(UC_HOOK_CODE, lambda uc, a, size, _: cur[0].add(a), None, OVY_BASE, OVY_BASE + len(e.ovy))
        if effects:
            e.uc.hook_add(UC_HOOK_CODE, lambda uc, a, size, _: cur[0].add(a), None, OVY95_BASE, OVY95_BASE + len(ovy95))
    if items:
        item_keys, item_by_fn, _ = item_registry(e.ovy)
    if moves:
        move_keys, move_by_fn, _ = move_registry(e.ovy)
        # Scratch buffer pointer global used by 021d7e44 (random move pick); never hashed.
        e.w32(SCRATCH_PTR, SCRATCH)
    h = Fnv()
    verbose = bool(os.environ.get("HANDLER_VERBOSE"))
    sd = os.environ.get("HANDLER_SEEDS")
    seed_dump = int(sd, 16) if sd else None
    stream = []
    sec = [""]
    for addr in addrs:
        ha = Fnv()
        seeds = []
        if cov is not None:
            cur[0] = cov.setdefault(addr, set())

        def W(v, h=h, ha=ha):
            h.w(v)
            ha.w(v)
            hs.w(v)
            if want:
                stream.append((sec[0], v & M))

        for seed in range(200):
            hs = Fnv()
            want = detail == (addr, seed)
            r = Rng(addr ^ ((seed * 0x9E37) & M))
            for i in range(NMONS):
                e.wbytes(MONS_ADDR + 0x200 * i, random_bpp(r))
            e.wbytes(0, bytes(SIZE))
            env.field = bool(r.next() & 1)
            host.wseed = r.next()
            me = r.next() % NMONS
            ev = bytearray(0x5A4)
            struct.pack_into("<I", ev, 0, 1)
            struct.pack_into("<H", ev, 4, 1)
            for i, lab in enumerate(LABELS):
                s = i + 1
                struct.pack_into("<H", ev, 4 + 2 * s, lab)
                v = value_for(r, me)
                struct.pack_into("<I", ev, 0xC4 + 4 * s, (v % 8 if lab == 5 else v) & M)
                k = [0, 1, 2, 3, 3, 3, 4, 0][r.next() % 8]
                ev[0x544 + s] = k
                if k == 3:
                    struct.pack_into("<I", ev, 0xC4 + 4 * s, 0x100 + r.next() % 0x10000)
                    if r.next() & 1:
                        struct.pack_into("<I", ev, 0x3C4 + 4 * s, r.next() % 0x40)
                        struct.pack_into("<I", ev, 0x244 + 4 * s, r.next() % 0x10000)
            host.alt_dfe28 = items and addr == 0x021DFE28
            host.alt_hint = addr if (moves and addr in (0x021E7634, 0x021E6E38) or effects and addr == FX_ALT_BD28
                                     or not (moves or items or effects) and addr == AB_ALT_89B8) and seed % 4 != 0 else 0
            if items and addr == 0x021DFE28 and seed % 4:
                # Harness hint (no extra draws): 021dfe28 is gated on var 0x4D == 0 and the holder
                # being an event target (var 5 = count, var 6 = ID); `alt_dfe28` picks other
                # salts for 021c830c / 021c7f24 (with the usual ones 021c830c is always the opposite
                # of 021c83d4, and 021c7f24 and 021c8374 cannot both pass). Without both hints the
                # final 021c7f4c gate is never reached.
                for lab, v in ((0x4D, 0), (5, 1), (6, me)):
                    s = LABELS.index(lab) + 1
                    struct.pack_into("<I", ev, 0xC4 + 4 * s, v)
                    ev[0x544 + s] = 0
            if moves or effects:
                # Variable 0x3F is a pointer to the message (string parameter) buffer.
                struct.pack_into("<I", ev, 0xC4 + 4 * (LABELS.index(0x3F) + 1), STRP)
                e.wbytes(STRP, bytes(STRP_SIZE))
            e.wbytes(EV, ev)
            hi, lo = r.next(), r.next()
            e.wbytes(RNGCTX, struct.pack("<QQQ", (hi << 32) | lo, 0x5D588B656C078965, 0x269EC3))
            fac = bytearray(0x3C)
            for k in range(7):
                struct.pack_into("<I", fac, 0x1C + 4 * k, r.next())
            if items:
                # Work words hold "already used" flags and counters, so make 0 and small values
                # common instead of 32-bit noise (which would skip those branches).
                for k in range(7):
                    sel = r.next() % 4
                    if sel < 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, 0)
                    elif sel == 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, r.next() % 4)
                # Type 5 factor owned by `me`; the sub ID is mostly one of the items using this
                # handler (the handler may look up its own item), else any registry item.
                own = item_by_fn.get(addr, [])
                pick, which = r.next(), r.next()
                item = own[which % len(own)] if own and pick % 4 else item_keys[which % len(item_keys)]
                struct.pack_into("<I", fac, 0x10, TYPE_ITEM)
                struct.pack_into("<H", fac, 0x38, item)
                fac[0x3A] = fac[0x3B] = me
            if moves:
                # Same work-word distribution as items (flags and counters), drawn after the
                # ability draws. Type 0 factor owned by `me`; the sub ID is mostly one of the
                # moves using this handler, else any registry move.
                for k in range(7):
                    sel = r.next() % 4
                    if sel < 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, 0)
                    elif sel == 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, r.next() % 4)
                own = move_by_fn.get(addr, [])
                pick, which = r.next(), r.next()
                mv = own[which % len(own)] if own and pick % 4 else move_keys[which % len(move_keys)]
                struct.pack_into("<I", fac, 0x10, TYPE_MOVE)
                struct.pack_into("<H", fac, 0x38, mv)
                fac[0x3A] = fac[0x3B] = me
            owner = me
            if effects:
                # Same work-word distribution as moves. The factor is of the handler's registry
                # kind; the sub ID is mostly a key using this handler, else any key of the kind.
                # The owner (`factor + 0x3A`, passed to the handler) is 0 for field effects, a
                # position (0..5) for position effects and a side (0/1) for side effects;
                # `+0x3B` is 0x1F for all three (set by the adders `FUN_0689827C` / `021d6f0c`).
                for k in range(7):
                    sel = r.next() % 4
                    if sel < 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, 0)
                    elif sel == 2:
                        struct.pack_into("<I", fac, 0x1C + 4 * k, r.next() % 4)
                kind, own = fx_by_fn[addr]
                pick, which = r.next(), r.next()
                key = own[which % len(own)] if pick % 4 else fx_keys[kind][which % len(fx_keys[kind])]
                owner = {"field": 0, "pos": me % 6, "side": me & 1}[kind]
                struct.pack_into("<I", fac, 0x10, FX_TYPE[kind])
                struct.pack_into("<H", fac, 0x38, key)
                fac[0x3A] = owner
                fac[0x3B] = 0x1F
            e.wbytes(POOL, fac)
            if moves:
                apply_move_hint(e, addr, seed, me, mv)
            else:
                apply_hint(e, host, addr, seed, me)
            e.call(addr, POOL, FLOW, owner, POOL + 0x1C)
            sec[0] = "ev"
            for w in words(e.rbytes(EV, 0x5A4)):
                W(w)
            for i in range(NMONS):
                sec[0] = "mon%d" % i
                for w in words(e.rbytes(MONS_ADDR + 0x200 * i, SIZE)):
                    W(w)
            sec[0] = "null_mon"
            for w in words(e.rbytes(0, SIZE)):
                W(w)
            sec[0] = "work"
            for w in words(e.rbytes(POOL + 0x1C, 0x1C)):
                W(w)
            sec[0] = "skip_check"
            W(e.r32(POOL + 0xC))
            sec[0] = "flags"
            W(e.r32(POOL + 0x18))
            x = struct.unpack("<Q", e.rbytes(RNGCTX, 8))[0]
            sec[0] = "rng"
            W(x & M); W(x >> 32)
            sec[0] = "hostlog"
            for v in host.log:
                W(v)
            sec[0] = "envlog"
            for ent in env.log:
                for v in ent:
                    W(v)
            if moves or effects:
                sec[0] = "strparam"
                for w in words(e.rbytes(STRP, STRP_SIZE)):
                    W(w)
            host.log.clear(); env.log.clear(); host.nbuf = 0; host.nparty = 0
            seeds.append(hs.hex())
            if seed_dump == addr:
                print("seed", seed, hs.hex())
        if funcs is not None:
            funcs["%08x" % addr] = {"hash": ha.hex(), "n_inputs": len(seeds), "seeds": seeds}
        if verbose:
            print(hex(addr), ha.hex())
    if detail:
        return stream
    return h.hex()


def ported(module):
    src = open(os.path.join(os.path.dirname(__file__), "../../crates/black-battle/src/handlers",
                            module + ".rs")).read()
    block = src[src.index("pub const PORTED"):]
    block = block[:block.index("];")]
    return [int(x, 16) for x in re.findall(r"0x([0-9A-Fa-f]{8})", block)]


def move_stub_check():
    """Self-test of the move-mode stubs (tags 0x12F-0x153 and `weather` at 021c2ab0): calls each
    stub with random arguments through the emulator and hashes result, log and out-buffers.
    Mirrors `move_stubs_match_python` in handlers/moves/tests.rs."""
    e = Emu()
    e.uc.mem_map(0, 0x10000)
    Env(e)
    host = Host(e, True)
    h = Fnv()
    r = Rng(0x57B5)
    host.wseed = r.next()
    RING = FLOW + 0x1C
    # (address, flow work first, number of scalar args, kind)
    specs = [
        (0x021C7D34, 1, 1, "ret"), (0x021C7D44, 1, 0, "ret"), (0x021C7ED8, 1, 1, "ret"),
        (0x021C7FCC, 1, 1, "party"), (0x021C7FE0, 1, 1, "partner"), (0x021C8030, 1, 2, "ret"),
        (0x021C8114, 1, 0, "ret"), (0x021C8138, 1, 0, "ret"), (0x021C814C, 1, 1, "ret"),
        (0x021C8174, 1, 0, "ret"), (0x021C8180, 1, 0, "ret"), (0x021C81C8, 1, 2, "mon"),
        (0x021C824C, 1, 1, "ret"), (0x021C82C4, 1, 2, "ret"), (0x021C8300, 1, 1, "ret"),
        (0x021C831C, 1, 1, "void"), (0x021C83A0, 1, 1, "ret"), (0x021C83DC, 1, 1, "void"),
        (0x021C83E8, 1, 1, "ret"), (0x021C846C, 1, 2, "out"), (0x021C8494, 1, 1, "void"),
        (0x021C2E4C, 1, 0, "ret"), (0x021C2AB0, 1, 0, "ret"),
        (0x06898CF4, 0, 2, "ret"), (0x0689C8AC, 0, 1, "ret"), (0x0689C8BC, 0, 1, "ret"),
        (0x0689C91C, 0, 1, "ret"), (0x0689C93C, 0, 1, "ret"), (0x0689CA1C, 0, 1, "ret"),
        (0x0689CA3C, 0, 1, "ret"), (0x0689CA5C, 0, 1, "ret"), (0x0689CA7C, 0, 1, "ret"),
        (0x0689CA8C, 0, 0, "excl"), (0x0689D064, 2, 2, "ret"), (0x0689D098, 2, 2, "ret"),
        (0x0689D0C8, 2, 1, "ret"), (0x0689D174, 2, 1, "ret"), (0x0689D184, 2, 2, "ret"),
        (0x021C7F4C, 1, 0, "ret"),
    ]

    def pick():
        k = r.next() % 6
        if k == 0:
            return r.next() % 8
        if k == 1:
            return 0x1F
        if k == 2:
            return r.next() % 0x240
        if k == 3:
            return r.next() % 16
        if k == 4:
            return r.next() % 2
        return r.next()

    for addr, wk, n, kind in specs:
        for _ in range(40):
            args = [pick() for _ in range(n)]
            host.nparty = 0
            call = [FLOW] if wk == 1 else ([RING] if wk == 2 else [])
            if kind == "mon":
                pid = args[0] % 9
                call += [MONS_ADDR + 0x200 * pid if pid < NMONS else 0x1234] + args[1:]
                args[0] = pid if pid < NMONS else 0xFF
            elif kind == "excl":
                call = [TMP]
            elif kind == "out":
                call += args + [TMP]
            else:
                call += args
            ret = e.call(addr, *call)[0]
            if kind in ("ret", "mon", "out"):
                h.w(ret)
            elif kind in ("party", "partner"):
                if ret == 0:
                    h.w(0xFFFFFFFF)
                else:
                    cnt = e.rbytes(ret + 0x18, 1)[0]
                    h.w(cnt)
                    for i in range(cnt):
                        h.w((e.r32(ret + 4 * i) - MONS_ADDR) // 0x200)
            elif kind == "excl":
                h.w(e.r32(TMP))
                for w in struct.unpack("<%dH" % 0x29, e.rbytes(ret, 0x52)):
                    h.w(w)
            if kind == "out":
                h.w(e.r32(TMP))
            for v in host.log:
                h.w(v)
            host.log.clear()
    return h.hex()


def effect_stub_check():
    """Self-test of the effects-mode stub (`06898de0`, tag 0x155): calls it through the emulator
    with random arguments and hashes result and log. Mirrors `effect_stubs_match_python` in
    handlers/effects/tests.rs."""
    e = Emu()
    e.uc.mem_map(0, 0x10000)
    Env(e)
    host = Host(e, False, False, True)
    h = Fnv()
    r = Rng(0xDE0)
    for _ in range(200):
        sub = r.next() % 0x20 if r.next() & 1 else r.next() & 0xFFFF
        side = [0, 1, 2, r.next() % 8, r.next()][r.next() % 5]
        e.wbytes(TMP, bytes(0x40))
        e.wbytes(TMP + 0x38, struct.pack("<H", sub))
        h.w(sub)
        h.w(side)
        h.w(e.call(FX_STUB_DE0, TMP, side)[0])
        for v in host.log:
            h.w(v)
        host.log.clear()
    return h.hex()


def skips():
    """Skip-check callbacks with random battle Pokemon, factor owner and arguments."""
    e = Emu()
    e.uc.mem_map(0, 0x10000)
    env = Env(e)
    host = Host(e)
    h = Fnv()
    r = Rng(0x5C1B)
    ovy = e.ovy
    for addr in [0x021DAEC4, 0x021DB1A8, 0x021DB650, 0x021DC820, 0x021DCCEC]:
        for _ in range(300):
            for i in range(NMONS):
                e.wbytes(MONS_ADDR + 0x200 * i, random_bpp(r))
            fac = bytearray(0x3C)
            fac[0x3B] = r.next() % 24  # both sides occur (poke_side is id >= 12; V-0017)
            e.wbytes(POOL, fac)
            ftype, ev = r.next() % 8, r.next() % 0x90
            k = r.next() % 4
            if k == 0:
                sub = 106
            elif k == 1:
                sub = r.next() % 600
            elif k == 3:
                sub = r.next() % 6  # 021dc820 tests sub_id <= 3
            else:
                o = 0x021F0B78 - 0x021B6100 + 2 * (r.next() % 13)
                sub = ovy[o] | ovy[o + 1] << 8
            owner = fac[0x3B] if r.next() % 4 == 0 else r.next() % 24  # 021db650 needs owner == the factor's Pokemon
            h.w(e.call(addr, POOL, FLOW, ftype, ev, stack_args=(sub, owner))[0])
            for v in host.log:
                h.w(v)
            for ent in env.log:
                for v in ent:
                    h.w(v)
            host.log.clear(); env.log.clear()
    return h.hex()


def registration():
    """Ability registration (0x021D8484-0x021D8564); mirrors handlers/registry/tests.rs."""
    G = 0x021F3CB0
    NONE = 0xFF
    e = Emu()
    env = Env(e)
    h = Fnv()
    base = 0x021B6100

    def idx(p):
        return NONE if p == 0 else (p - POOL) // 0x3C

    abilities = [struct.unpack_from("<I", e.ovy, 0x021F0E74 - base + 8 * i)[0] for i in range(0x9E)]
    for seed in range(120):
        r = Rng(0x8484 + seed)
        for i in range(NMONS):
            e.wbytes(MONS_ADDR + 0x200 * i, random_bpp(r))
        for i in range(NMONS):
            m = MONS_ADDR + 0x200 * i
            if r.next() % 4 == 0:
                ability = r.next() & 0xFFFF
            else:
                ability = abilities[r.next() % len(abilities)]
            e.wbytes(m + 0x13C, struct.pack("<H", ability))
            e.wbytes(m + 0x19, bytes([r.next() % 4]))
        env.field = bool(r.next() & 1)
        e.call(0x021D6EAC)
        env.log.clear()
        for _ in range(30):
            o = r.next() % 5
            a = r.next() % NMONS
            b = r.next() % NMONS
            ma, mb = MONS_ADDR + 0x200 * a, MONS_ADDR + 0x200 * b
            if o == 0:
                h.w(idx(e.call(0x021D8484, ma)[0]))
            elif o == 1:
                e.call(0x021D8510, ma)
            elif o == 2:
                e.call(0x021D8538, ma)
            elif o == 3:
                e.call(0x021D8548, ma)
            else:
                x, y = e.rbytes(ma + 0x13C, 2), e.rbytes(mb + 0x13C, 2)
                e.wbytes(ma + 0x13C, y)
                e.wbytes(mb + 0x13C, x)
                e.call(0x021D8564, ma, mb)
            for i in range(0x84):
                bb = e.rbytes(POOL + 0x3C * i, 0x3C)
                w = struct.unpack("<15I", bb)
                h.w(idx(w[0])); h.w(idx(w[1])); h.w(w[2]); h.w(w[3]); h.w(w[4]); h.w(w[5]); h.w(w[6])
                for k in range(7):
                    h.w(w[7 + k])
                h.w(bb[0x38] | bb[0x39] << 8); h.b(bb[0x3A]); h.b(bb[0x3B])
            h.w(e.r32(G)); h.w(idx(e.r32(G + 4))); h.w(e.r32(G + 8))
            for i in range(0x84):
                h.w(idx(e.r32(G + 0xC + 4 * i)))
            for ent in env.log:
                for v in ent:
                    h.w(v)
            env.log.clear()
    return h.hex()


def _chunk(arg):
    addrs, items, moves, effects = arg
    funcs = {}
    run(addrs, funcs=funcs, items=items, moves=moves, effects=effects)
    return funcs


def run_parallel(addrs, jobs, items=False, moves=False, effects=False):
    """Per-function results only (the module hash is sequential, so it is not produced)."""
    from multiprocessing import Pool
    n = jobs * 4
    with Pool(jobs) as p:
        parts = p.map(_chunk, [(addrs[i::n], items, moves, effects) for i in range(n)])
    funcs = {}
    for d in parts:
        funcs.update(d)
    return {"%08x" % a: funcs["%08x" % a] for a in addrs}


if __name__ == "__main__":
    import json
    args = sys.argv[1:]
    jobs = 1
    if "--jobs" in args:  # --jobs N: with --json, run handlers in N processes (no module hash)
        jobs = int(args[args.index("--jobs") + 1])
        args = [a for i, a in enumerate(args) if not (a == "--jobs" or args[i - 1] == "--jobs")]
    as_json = "--json" in args
    detail = None
    only = None
    if "--detail" in args:  # --detail ADDR:SEED -> JSON [[section, word], ...] for that input
        a, sd = args[args.index("--detail") + 1].split(":")
        only = int(a, 16) & ~1
        detail = (only, int(sd))
    args0 = args
    args = [a for i, a in enumerate(args) if not a.startswith("--") and args[i - 1] != "--detail"]
    for mod in args or ["abilities"]:
        if mod == "skips":
            print(mod, skips())
        elif mod == "effectstubs":
            print(mod, effect_stub_check())
        elif mod == "registration":
            print(mod, registration())
        elif mod == "movestubs":
            print(mod, move_stub_check())
        elif mod == "items" and "--list" in args0:
            keys, by_fn, events = item_registry(Emu().ovy)
            done = set(ported(mod))
            print("items %d registry keys, %d handlers, %d ported" % (len(keys), len(by_fn), len(done)))
            for a in sorted(by_fn):
                print("%08x%s items=%s events=%s" % (
                    a, " ported" if a in done else "", ",".join(map(str, by_fn[a])),
                    ",".join("%#x" % x for x in sorted(events[a]))))
        elif mod == "moves" and "--list" in args0:
            keys, by_fn, events = move_registry(Emu().ovy)
            done = set(ported(mod))
            print("moves %d registry keys, %d handlers, %d ported" % (len(keys), len(by_fn), len(done)))
            for a in sorted(by_fn):
                print("%08x%s moves=%s events=%s" % (
                    a, " ported" if a in done else "", ",".join(map(str, by_fn[a])),
                    ",".join("%#x" % x for x in sorted(events[a]))))
        elif mod == "effects" and "--list" in args0:
            by_fn, keys, events, maxes = fx_registry(load_ovy95())
            done = set(ported(mod))
            print("effects %d registry keys (%s), %d handlers, %d ported" % (
                sum(map(len, keys.values())), ", ".join("%s %d" % (k, len(v)) for k, v in keys.items()),
                len(by_fn), len(done)))
            for a in sorted(by_fn):
                kind, ks = by_fn[a]
                print("%08x%s %s keys=%s events=%s%s" % (
                    a, " ported" if a in done else "", kind, ",".join(map(str, ks)),
                    ",".join("%#x" % x for x in sorted(events[a])),
                    " max=" + ",".join(str(maxes[k]) for k in ks) if kind == "side" else ""))
        elif detail:
            print(json.dumps(run([detail[0]], detail=detail, items=mod == "items", moves=mod == "moves", effects=mod == "effects")))
        elif as_json and jobs > 1:
            print(json.dumps({"module": mod, "hash": None, "funcs": run_parallel(ported(mod), jobs, mod == "items", mod == "moves", mod == "effects")}))
        elif as_json:
            funcs = {}
            total = run(ported(mod), funcs=funcs, items=mod == "items", moves=mod == "moves", effects=mod == "effects")
            print(json.dumps({"module": mod, "hash": total, "funcs": funcs}))
        else:
            print(mod, run(ported(mod), items=mod == "items", moves=mod == "moves", effects=mod == "effects"))
