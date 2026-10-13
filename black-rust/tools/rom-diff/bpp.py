"""Differential suites for black-battle::battle_mon (BTL_POKEPARAM, asm/battle_pokeparam.s).

Mirrors `battle_mon::tests` exactly: same generator, same op table, same mock environment.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from emu import Emu, Fnv, Rng  # noqa: E402

M = 0xFFFFFFFF
BPP = 0x02E20000
BPP2 = 0x02E21000
SCR = 0x02E00000
SIZE = 0x1F8


def wide(a, b):
    return [b % 64, b % 0x400, b % 0x20000, b][(a >> 28) & 3]


def mix(a, b):
    x = ((a * 0x9E3779B1) ^ (b * 0x85EBCA77)) & M
    return (x ^ (x >> 15)) & M


class Env:
    def __init__(self, e):
        self.log = []
        self.field = False
        st = e.stub
        st(0x02017E1C, self.get)
        st(0x02017E40, self.put)
        st(0x02017D30, self.fast_begin)
        st(0x02017D70, self.fast_end)
        st(0x020181F4, self.set_move)
        st(0x02017F00, self.f00)
        st(0x02017F90, self.status)
        st(0x0201C158, self.max_pp)
        st(0x02019C90, self.personal)
        st(0x021EF948, self.field_fx)
        st(0x02018584, self.exp_for_level)
        st(0x020185C4, self.recalc)
        st(0x02017F9C, self.set_status)
        st(0x02017988, self.set_form)
        st(0x0202142C, self.f42c)
        e.uc.mem_map(0x06898000, 0x8000)  # ARM autoload region holding FUN_0689C868
        st(0x0689C868, self.passable)
        st(0x02030734, self.heap_alloc)

    def get(self, e, pp, i, *_, sp):
        self.log.append((1, pp, i, 0))
        return mix(pp, i)

    def put(self, e, pp, i, v, *_, sp):
        self.log.append((2, pp, i, v))

    def fast_begin(self, e, pp, *_, sp):
        self.log.append((3, pp, 0, 0))
        return mix(pp, 0xFB) & 1

    def fast_end(self, e, pp, t, *_, sp):
        self.log.append((4, pp, t, 0))

    def set_move(self, e, pp, mv, slot, *_, sp):
        self.log.append((5, pp, mv, slot))

    def f00(self, e, pp, *_, sp):
        self.log.append((6, pp, 0, 0))
        return mix(pp, 0xF00)

    def status(self, e, pp, *_, sp):
        self.log.append((7, pp, 0, 0))
        return mix(pp, 0xF90) % 7

    def max_pp(self, e, mv, ups, *_, sp):
        self.log.append((8, mv, ups, 0))
        return mix(mv, ups) & 0x3F

    def personal(self, e, s, f, p, *_, sp):
        self.log.append((9, s, f, p))
        return mix(s ^ (f << 16), p)

    def exp_for_level(self, e, s, f, l, *_, sp):
        self.log.append((11, s, f, l))
        return mix(s ^ (f << 16), l) & 0x1FFFF

    def recalc(self, e, pp, *_, sp):
        self.log.append((12, pp, 0, 0))

    def set_status(self, e, pp, st, *_, sp):
        self.log.append((13, pp, st, 0))

    def set_form(self, e, pp, f, *_, sp):
        self.log.append((14, pp, f, 0))

    def f42c(self, e, v, *_, sp):
        self.log.append((15, v, 0, 0))
        return v % 3

    def passable(self, e, s, *_, sp):
        self.log.append((16, s, 0, 0))
        return mix(s, 0x99) & 1

    def heap_alloc(self, e, heap, size, *_, sp):
        e.wbytes(BPP, bytes(SIZE))
        return BPP

    def field_fx(self, e, i, *_, sp):
        self.log.append((10, i, 0, 0))
        return int(self.field)


def random_bpp(r):
    b = bytearray()
    for _ in range(SIZE // 4):
        b += struct.pack("<I", r.next())
    b[0x140] %= 5
    b[0x1EF] %= 3
    for i in range(3):
        b[0x1EC + i] %= 7
    b[0xD0] %= 25
    return b


class Ctx:
    def __init__(self):
        self.e = Emu()
        self.env = Env(self.e)

    def run(self, addr, *args, stack=()):
        r0, _ = self.e.call(addr, *[a & M for a in args], stack_args=stack)
        return r0

    def state(self, h, extra=()):
        for x in self.e.rbytes(BPP, SIZE):
            h.b(x)
        for ent in self.env.log:
            for v in ent:
                h.w(v)
        self.env.log.clear()


def slot_ptr(s):
    return BPP + 0x104 + 14 * s


def ops_a(c, op, a, b, d):
    """One operation of suite A. Returns values to hash (function results)."""
    e = c.e
    s = a % 4
    if op == 0:
        c.run(0x021D51CC, slot_ptr(s)); return []
    if op == 1:
        c.run(0x021D5734, BPP, s, 0 if (b >> 28) == 0 else b & 0xFFFF, d & 0xFF, stack=(a >> 8 & 1,)); return []
    if op == 2:
        return [c.run(0x021D5244, slot_ptr(s), b, s)]
    if op == 3:
        return [c.run(0x021D4F84, BPP, b, a >> 8 & 1)]
    if op == 4:
        c.run(0x021D505C, BPP); return []
    if op == 5:
        c.run(0x021D50B4, BPP); return []
    if op == 6:
        c.run(0x021D5150, BPP); return []
    if op == 7:
        return [c.run(0x021D5448, BPP), c.run(0x021D5450, BPP), c.run(0x021D5478, BPP),
                c.run(0x021D54B4, BPP)]
    if op == 8:
        return [c.run(0x021D54C0, BPP, s), c.run(0x021D54D0, BPP, s), c.run(0x021D54E0, BPP, s)]
    if op == 9:
        e.wbytes(BPP2, bytes(SIZE))
        c.run(0x021D54F4, BPP, BPP2)
        return list(e.rbytes(BPP2, SIZE))
    if op == 10:
        e.wbytes(SCR, bytes(8))
        r = c.run(0x021D5528, BPP, s, SCR, SCR + 4)
        return [r, e.rbytes(SCR, 1)[0], e.rbytes(SCR + 4, 1)[0]]
    if op == 11:
        return [c.run(0x021D554C, BPP, s), c.run(0x021D5568, BPP, s), c.run(0x021D5584, BPP, a % 6)]
    if op == 12:
        mv = c.e.rbytes(slot_ptr(s) + 6, 2)
        mv = mv[0] | mv[1] << 8 if b & 1 else b & 0xFFFF
        return [c.run(0x021D55A0, BPP, mv), c.run(0x021D5750, BPP, mv), c.run(0x021D5778, BPP, mv)]
    if op == 13:
        return [c.run(0x021D55D8, BPP, s, b & 1)]
    if op == 14:
        c.run(0x021D5608, BPP, s, wide(a, b)); return []
    if op == 15:
        c.run(0x021D5640, BPP, s, wide(a, b)); return []
    if op == 16:
        return [c.run(0x021D5674, BPP, s, wide(a, b))]
    if op == 17:
        return [c.run(0x021D56A0, BPP, s, wide(a, b))]
    if op == 18:
        c.run(0x021D56CC, BPP, s); return []
    if op == 19:
        c.run(0x021D5364, BPP); return []
    if op == 20:
        c.run(0x021D539C, BPP); return []
    if op == 21:
        c.run(0x021D5F08, BPP); return []
    if op == 22:
        return [c.run(0x021D5D68, BPP), c.run(0x021D5EFC, BPP), c.run(0x021D5D68, BPP)]
    if op == 23:
        st = 1 + a % 7
        delta = (b % 65) - 32
        return [c.run(0x021D5D14, BPP, st, delta), c.run(0x021D5D50, BPP, st)]
    if op == 24:
        return [c.run(0x021D5DC0, BPP, a % 10, wide(a, b))]
    if op == 25:
        return [c.run(0x021D5E38, BPP, a % 10, wide(a, b))]
    if op == 26:
        c.run(0x021D5EA8, BPP, a % 10, wide(a, b)); return []
    if op == 27:
        return [c.run(0x021D5F14, BPP), c.run(0x021D5F38, BPP, (b % 201) - 100)]
    if op == 28:
        c.run(0x021D5F90, BPP, wide(a, b)); return []
    if op == 29:
        c.run(0x021D5FA4, BPP, wide(a, b)); return []
    if op == 30:
        c.run(0x021D5FB8, BPP); return []
    if op == 31:
        r = [c.run(0x021D5B40, BPP), c.run(0x021D5BA4, BPP)]
        c.run(0x021D5B44, BPP, b & 0xFFFF)
        return r
    if op == 32:
        n = b % 16
        return [c.run(0x021D5BB0, BPP, n), c.run(0x021D5BDC, BPP, n)]
    if op == 33:
        n = b % 16
        c.run([0x021D5FC0, 0x021D5FE4, 0x021D6008][a % 3], BPP, n); return []
    if op == 34:
        return [c.run(0x021D5C08, BPP), c.run(0x021D5C30, BPP)]
    if op == 35:
        r = [c.run(0x021D5844, BPP), c.run(0x021D5864, BPP)]
        if a & 1:
            c.run(0x021D5848, BPP, b)
        else:
            c.run(0x021D5854, BPP)
        return r
    raise ValueError(op)


N_OPS_A = 36


def suite_a():
    c = Ctx()
    h = Fnv()
    for seed in range(150):
        r = Rng(0xB0 + seed)
        c.e.wbytes(BPP, random_bpp(r))
        c.env.field = bool(r.next() & 1)
        for _ in range(40):
            op = r.next() % N_OPS_A
            a, b, d = r.next(), r.next(), r.next()
            for v in ops_a(c, op, a, b, d):
                h.w(v)
            c.state(h)
    return h.hex()


RNGCTX = 0x021F6368
LCG_MUL = 0x5D588B656C078965
LCG_ADD = 0x269EC3


def r16(e, a):
    v = e.rbytes(a, 2)
    return v[0] | v[1] << 8


def ops_b(c, op, a, b, d):
    e = c.e
    if op == 0:
        if a & 3 == 0:
            # V-0012 hint: grounded (sick 0x18) with Flying in either slot (or both)
            c.run(0x021D6034, BPP, 0x18, d | 1)
            c.run(0x021D66B0, BPP, [0x0211, 0x1102, 0x0202, (b % 18) << 8 | 2][(a >> 2) & 3])
        e.wbytes(SCR, bytes(2))
        c.run(0x021D57A8, BPP, SCR, SCR + 1)
        t = list(e.rbytes(SCR, 2))
        return t + [c.run(0x021D57FC, BPP), c.run(0x021D5814, BPP, b % 32)]
    if op == 1:
        pair = 0x1111 if (b >> 16) & 3 == 0 else (0x1102 if (b >> 16) & 3 == 1 else b & 0xFFFF)
        c.run(0x021D66B0, BPP, pair)
        c.run(0x021D66D0, BPP, 0x30 if (d >> 16) & 3 == 0 else d & 0xFFFF)
        return []
    if op == 2:
        return [c.run(0x021D59B4, BPP, a % 23), c.run(0x021D58A8, BPP, b % 14),
                c.run(0x021D5AD8, BPP, 6 + d % 8)]
    if op == 3:
        e.wbytes(SCR, bytes(14))
        c.run(0x021D590C, BPP, SCR)
        return list(e.rbytes(SCR, 14))
    if op == 4:
        c.run(0x021D5970, BPP, 7 + a % 7, b & 0xFFFF); return []
    if op == 5:
        if a & 3 == 1:
            e.wbytes(BPP + 0x10, e.rbytes(BPP + 0x0E, 2))
        elif a & 3 == 2:
            e.wbytes(BPP + 0x10, bytes(2))
        return [c.run(0x021D5B48, BPP), c.run(0x021D5B68, BPP), c.run(0x021D5B7C, BPP)]
    if op == 6:
        mx = (b & 0xFFFF) >> ((a >> 20) & 15)
        hp = 0 if (a >> 28) & 7 == 0 else (a & 0xFFFF) >> ((a >> 24) & 15)
        e.wbytes(BPP + 0x0E, struct.pack("<HH", mx, hp))
        return [c.run(0x021D5C44, BPP)]
    if op == 7:
        s_ = a % 36
        return [c.run(0x021D62C4, BPP, s_), c.run(0x021D6314, BPP, s_), c.run(0x021D631C, BPP, s_),
                c.run(0x021D62D8, BPP, s_), c.run(0x021D62A8, BPP)]
    if op == 8:
        c.run(0x021D6034, BPP, a % 36, b); return []
    if op == 9:
        if a & 1:
            c.run(0x021D61F0, BPP)
        else:
            c.run(0x021D6238, BPP, b % 36)
        return []
    if op == 10:
        c.run(0x021D6264, BPP, b % 40); return []
    if op == 11:
        if (a >> 8) & 3 == 0:
            # V-0012 hint: a fresh kind-1 cont with a count-up limit in bits 3-8 (turns 0 < limit)
            c.run(0x021D6034, BPP, a % 36, (d & ~0x1FF & M) | (((b >> 2) % 63 + 1) << 3) | 1)
        e.wbytes(SCR, struct.pack("<II", 0xCCCCCCCC, 0xCCCCCCCC))
        p1 = SCR if b & 1 else 0
        p2 = SCR + 4 if b & 2 else 0
        r = c.run(0x021D6064, BPP, a % 36, p1, p2)
        return [r, e.r32(SCR), e.r32(SCR + 4)]
    if op == 12:
        if a & 3 == 0:
            c.run(0x021D66D0, BPP, 0x30)  # V-0012 hint: Early Bird
        return [c.run(0x021D6130, BPP), c.run(0x021D61A8, BPP)]
    if op == 13:
        return [c.run(0x021D6324, BPP, a % 12)]
    if op == 14:
        c.run(0x021D63B8, BPP, a & 1); return []
    if op == 15:
        c.run(0x021D4EFC, BPP, b, a & 1, (a >> 1) & 1); return []
    if op == 16:
        c.run(0x021D52B4, BPP, b, a & 1); return []
    if op == 17:
        c.run(0x021D533C, BPP); return []
    if op == 18:
        c.run([0x021D646C, 0x021D64F0, 0x021D6580][a % 3], BPP); return []
    if op == 19:
        c.run(0x021D65C8, BPP, BPP2)
        return list(e.rbytes(BPP2, SIZE))
    if op == 20:
        if b & 1:
            e.wbytes(BPP + 0x0C, struct.pack("<H", 0x1EC))
        c.run(0x021D66D8, BPP, a % 32); return []
    if op == 21:
        c.run(0x021D6BFC, BPP); return []
    if op == 22:
        c.run(0x021D6B84, BPP, a & 1); return []
    if op == 23:
        c.run(0x021D6C3C, BPP, b)
        c.run(0x021D6E24, BPP, d)
        c.run(0x021D6EA0, BPP)
        return []
    if op == 24:
        if b & 1:
            e.wbytes(BPP2 + 0x1F2, bytes(2))
        if b & 2:
            t = e.rbytes(BPP2 + 0x1B, 1)[0] & 0xDF
            e.wbytes(BPP2 + 0x1B, bytes([t]))
            t = e.rbytes(BPP + 0x1B, 1)[0] & 0xDF
            e.wbytes(BPP + 0x1B, bytes([t]))
        return [c.run(0x021D6C50, BPP, BPP2)]
    if op == 25:
        mv = r16(e, BPP + 0x14C) if (b >> 16) & 1 else (a >> 8) % 4
        c.run(0x021D6744, BPP, a & 3, b & 1, (b >> 8) & 0x1F, stack=(mv, d & 0xFFFF)); return []
    if op == 26:
        r = [c.run(0x021D679C, BPP), c.run(0x021D67A8, BPP), c.run(0x021D67B0, BPP),
             c.run(0x021D67B8, BPP), c.run(0x021D67C4, BPP), c.run(0x021D67D0, BPP),
             c.run(0x021D67E8, BPP)]
        c.run(0x021D67D8, BPP, b % 3)
        return r
    if op == 27:
        c.run(0x021D63EC, BPP, b & 0xFFFF); return []
    if op == 28:
        c.run(0x021D6408, BPP); return []
    if op == 29:
        c.run(0x021D6440, BPP, b % 16); return []
    if op == 30:
        r = [c.run(0x021D6740, BPP)]
        if a & 1:
            c.run(0x021D6730, BPP, b & 0xFFFF)
        else:
            c.run(0x021D6738, BPP)
        return r
    if op == 31:
        c.run(0x021D67F4 if a & 1 else 0x021D6824, BPP); return []
    if op == 32:
        e.wbytes(SCR, struct.pack("<II", a, b))
        c.run(0x021D6848, BPP, SCR); return []
    if op == 33:
        e.wbytes(SCR, bytes(8))
        r = c.run(0x021D6940, BPP, a % 4, b % 8, SCR)
        return [c.run(0x021D6920, BPP, a % 4), r] + list(e.rbytes(SCR, 8))
    if op == 34:
        c.run(0x021D69A0, BPP, a % 5, b & 0xFF)
        return [c.run(0x021D69AC, BPP, d % 5)]
    if op == 35:
        c.run(0x021D6E40, BPP, a % 0x20, b & 0xFFFF)
        e.wbytes(SCR, bytes(4))
        r = c.run(0x021D6E50, BPP, SCR, SCR + 2)
        return [r, e.rbytes(SCR, 1)[0], r16(e, SCR + 2), c.run(0x021D6E74, BPP)]
    if op == 36:
        c.run(0x021D6E88, BPP)
        return [c.run(0x021D6E74, BPP)]
    if op == 37:
        if a & 1:
            c.run(0x021D6D5C, BPP, b & 0xFFFF)
        else:
            c.run(0x021D6D70, BPP)
        return [c.run(0x021D6D7C, BPP), c.run(0x021D6D90, BPP)]
    if op == 38:
        e.wbytes(SCR, struct.pack("<H", b & 0xFFFF))
        r = c.run(0x021D6D9C, BPP, SCR)
        return [r, r16(e, SCR)]
    if op == 39:
        if a & 1:
            c.run(0x021D6DC4, BPP)
        c.run(0x021D6DCC, BPP, b % 30)
        return [c.run(0x021D6E04, BPP), c.run(0x021D6E0C, BPP, d % 30)]
    if op == 40:
        e.wbytes(BPP + 8, struct.pack("<I", wide(d, a)))
        e.wbytes(BPP + 0x18, bytes([(a >> 16) & 0xFF]))
        e.wbytes(SCR, struct.pack("<I", wide(a, b)) + bytes(16))
        r = c.run(0x021D69B8, BPP, SCR, SCR + 4)
        return [r, e.r32(SCR)] + list(e.rbytes(SCR + 4, 14)) + [c.run(0x021D6B6C, BPP)]
    if op == 41:
        e.wbytes(RNGCTX, struct.pack("<QQQ", (a << 32) | b, LCG_MUL, LCG_ADD))
        res = c.run(0x021D4D84, b, a & 0xFF, 0)
        x = struct.unpack("<Q", e.rbytes(RNGCTX, 8))[0]
        return [res == BPP, x & M, x >> 32]
    raise ValueError(op)


N_OPS_B = 42


def suite_b():
    c = Ctx()
    h = Fnv()
    for seed in range(200):
        r = Rng(0xB000 + seed)
        c.e.wbytes(BPP, random_bpp(r))
        c.e.wbytes(BPP2, random_bpp(r))
        c.env.field = bool(r.next() & 1)
        for _ in range(30):
            op = r.next() % N_OPS_B
            a, b, d = r.next(), r.next(), r.next()
            for v in ops_b(c, op, a, b, d):
                h.w(v)
            c.state(h)
    return h.hex()


def tagged():
    c = Ctx()
    h = Fnv()
    r = Rng(0x4D64)
    for _ in range(300):
        ln = r.next() % 0x40
        hdr = (ln << 8) | (r.next() & 0xFF)
        src = bytes(r.next() & 0xFF for _ in range(0x40))
        tag = r.next() & 0xFF
        c.e.wbytes(SCR, struct.pack("<I", hdr) + bytes(0x44))
        c.e.wbytes(SCR + 0x100, src)
        c.run(0x021D4D64, SCR, SCR + 0x100, tag)
        for x in c.e.rbytes(SCR, 0x48):
            h.b(x)
    return h.hex()


if __name__ == "__main__":
    which = sys.argv[1:] or ["suite_a", "suite_b", "tagged"]
    for name in which:
        print(name, globals()[name]())
