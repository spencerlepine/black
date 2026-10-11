"""Differential hashes for black-battle, computed by running the original overlay 93 code.

Each section mirrors a `*_matches_rom_fuzz` test in crates/black-battle exactly (same
generator, same call order, same hash).
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from emu import Emu, Fnv, Rng  # noqa: E402

M = 0xFFFFFFFF
SCRATCH = 0x02E00000
MONS = 0x02E10000
PARTY = 0x02E80000
EV = 0x021F3ECC
RNGCTX = 0x021F6368
LCG_MUL = 0x5D588B656C078965
LCG_ADD = 0x269EC3


def s32(v):
    v &= M
    return v - (1 << 32) if v & 0x80000000 else v


class Mon:
    def __init__(self, r):
        self.max_hp = r.next() & 0xFFFF
        self.level = 1 + r.next() % 100
        self.form = r.next() % 4
        self.species = r.next() % 650
        self.poke_id = r.next() % 24
        self.t1 = r.next() % 18
        self.t2 = r.next() % 18


e = Emu()
mons = {}
party = {"mons": []}
diag = {"called": False}


def bm_get(em, bpp, id_, *_, sp):
    m = mons[bpp]
    return {0x0E: m.max_hp, 0x0F: m.level, 0x13: m.form}[id_]


def has_type(em, bpp, ty, *_, sp):
    m = mons[bpp]
    return int(ty != 17 and (m.t1 == ty or m.t2 == ty))


e.stub(0x021D59B4, bm_get)
e.stub(0x021D5814, has_type)
e.stub(0x021D5440, lambda em, bpp, *_, sp: mons[bpp].poke_id)
e.stub(0x021D5444, lambda em, bpp, *_, sp: mons[bpp].species)
e.stub(0x02019C90, lambda em, s, f, p, *_, sp: (s * 31 + f * 7 + p) % 256)
e.stub(0x0202A3FC, lambda em, v, *_, sp: v % 5)
e.stub(0x021B9B94, lambda em, *_, sp: len(party["mons"]))
e.stub(0x021B9C00, lambda em, p, i, *_, sp: party["mons"][i])


def on_diag(em, *_, sp):
    diag["called"] = True


e.stub(0x02034714, on_diag)


def add_mon(m, slot):
    addr = MONS + slot * 0x200
    mons[addr] = m
    return addr


def call(addr, *args):
    return e.call(addr, *[a & M for a in args])[0]


# ---- sick_cont ------------------------------------------------------------------------------
def sick_cont():
    h = Fnv()
    r = Rng(0x5C)
    for _ in range(2000):
        a = r.next()
        b = r.next()
        h.w(call(0x021E8940, a))
        h.w(call(0x021E896C, a, b))
        h.w(call(0x021E899C, a))
        h.w(call(0x021E89C0))
        h.w(call(0x021E89D4, a))
        h.w(call(0x021E89F8, a, b))
        h.w(call(0x021E8A28, a, b))
        h.w(call(0x021E8A58))
        h.w(call(0x021E8A78, a))
        h.w(call(0x021E8A78, a & 0x7F))
    return h.hex()


# ---- event_var ------------------------------------------------------------------------------
def event_var():
    h = Fnv()
    for seed in range(300):
        r = Rng(seed)
        e.wbytes(EV, bytes(0x5A4))
        for _ in range(60):
            op = r.next() % 12
            label = 2 + r.next() % 6
            a = r.next()
            v = s32(a) >> (r.next() % 32)
            if op in (0, 1):
                diag["called"] = False
                call(0x021D74F8)
                h.w(int(not diag["called"]))
            elif op == 2:
                call(0x021D7540)
            elif op == 3:
                call(0x021D75A8, label, v)
            elif op == 4:
                call(0x021D75EC, label, v)
            elif op == 5:
                call(0x021D7630, label, v)
            elif op == 6:
                lo = r.next() % 300
                hi = r.next() % 300
                call(0x021D7674, label, v, lo, hi)
            elif op == 7:
                h.w(call(0x021D76BC, label, v))
            elif op == 8:
                call(0x021D7704, label, r.next() % 0x4000)
            elif op == 9:
                h.w(call(0x021D775C, label))
            elif op == 10:
                h.w(call(0x021D77E8, EV, label))
            else:
                e.w32(SCRATCH, 0)
                found = call(0x021D778C, label, SCRATCH)
                h.w(found)
                h.w(e.r32(SCRATCH))
        for x in e.rbytes(EV, 0x5A4):
            h.b(x)
    return h.hex()


# ---- calc: pure -----------------------------------------------------------------------------
def trunc_mod8(v):
    s = s32(v)
    m = abs(s) % 8
    return -m if s < 0 else m


def pure():
    h = Fnv()
    r = Rng(0xCA1C)
    for _ in range(5000):
        a, b, c, d = r.next(), r.next(), r.next(), r.next()
        stage = r.next() % 13
        h.w(call(0x021D78E4, a & 0xFFFF, stage))
        h.w(call(0x021D78E4, a, stage))
        h.w(call(0x021D7908, b & 0xFF, stage))
        h.w(call(0x021D7908, b, stage))
        t1, t2, t3 = c % 18, (c >> 8) % 18, (c >> 16) % 18
        h.w(call(0x021D794C, t1, t2))
        h.w(call(0x021D799C, t1, (t2 << 8) | t3))
        h.w(call(0x021D79E8, d % 6, (d >> 8) % 6))
        e.wbytes(SCRATCH, b"\xff" * 17)
        n = call(0x021D7A9C, (d >> 16) % 17, SCRATCH)
        h.w(n)
        for x in e.rbytes(SCRATCH, 17):
            h.b(x)
        h.w(call(0x021D7A44, a % 251, b % 1000, c % 101, d % 1000))
        h.w(call(0x021D7A44, a, b, c, d))
        h.w(call(0x021D7A44, a % 251, b % 1000, c % 101, 0))
        h.w(call(0x021D7A6C, a, b % 8))
        h.w(call(0x021D7AD4, trunc_mod8(c)))
        h.w(call(0x021D7B10, a, b))
        h.w(call(0x021D7B10, a & 0xFFFF, b & 0x3FFF))
        h.w(call(0x021D7B2C, a & 0xFF, b & 0xFFF))
        h.w(call(0x021D7B3C, a, b))
        h.w(call(0x021D7B3C, a & 0xFFFF, b % 200))
        h.w(call(0x021D7C38, a % 6, b % 18))
        h.w(call(0x021D7EF0, a % 5))
        h.w(call(0x021D7F00, b % 6))
        h.w(call(0x021D7F2C, c % 5, d))
        six = struct.pack("<I", a) + bytes([b & 0xFF, (b >> 8) & 0xFF])
        e.wbytes(SCRATCH, six)
        h.w(call(0x021D806C, SCRATCH))
        e.wbytes(SCRATCH, bytes(6))
        call(0x021D8090, c, SCRATCH)
        for x in e.rbytes(SCRATCH, 6):
            h.b(x)
    return h.hex()


# ---- calc: rng ------------------------------------------------------------------------------
def rng():
    h = Fnv()
    for seed in range(400):
        r = Rng(seed)
        hi = r.next()
        lo = r.next()
        x = (hi << 32) | lo
        e.wbytes(RNGCTX, struct.pack("<QQQ", x, LCG_MUL, LCG_ADD))
        mons.clear()
        mon = add_mon(Mon(r), 0)
        for _ in range(20):
            op = r.next() % 9
            a = r.next()
            b = r.next()
            if op == 0:
                v = call(0x021D78AC, a % 300)
            elif op == 1:
                v = call(0x021D7930, a % 6)
            elif op == 2:
                v = call(0x021D7AF0, a % 120)
            elif op == 3:
                v = call(0x021D7B8C, a % 50, b % 50)
            elif op == 4:
                v = call(0x021D7BA4, a % 7)
            elif op == 5:
                v = call(0x021D7D18, a % 9)
            elif op == 6:
                e.w32(SCRATCH, 0xCCCCCCCC)
                call(0x021D7DC0, (a % 12) - 2, mon, SCRATCH)
                v = e.r32(SCRATCH)
            elif op == 7:
                e.w32(SCRATCH, b)
                call(0x021D7C70, a & 0xFFFF, mon, SCRATCH)
                v = e.r32(SCRATCH)
            else:
                v = call(0x021D78AC, a)
            h.w(v)
            x = struct.unpack("<Q", e.rbytes(RNGCTX, 8))[0]
            h.w(x & M)
            h.w(x >> 32)
    return h.hex()


# ---- calc: hooks ----------------------------------------------------------------------------
def hooks():
    h = Fnv()
    for seed in range(2000):
        r = Rng(seed)
        mons.clear()
        mon = add_mon(Mon(r), 0)
        d = r.next() % 40
        h.w(call(0x021D7B60, mon, d))
        h.w(call(0x021D7B74, mon, d))
        h.w(call(0x021D7BD4, mon, r.next() % 6))
        h.w(call(0x021D7EB4, mon))
        h.w(call(0x021D80A8, mon))
        n = r.next() % 7
        party["mons"] = [add_mon(Mon(r), 1 + i) for i in range(n)]
        h.w(call(0x021D7FF8, r.next() % 12, PARTY))
        v = r.next() % 100
        h.w(call(0x021D7F38, v))
        h.w(call(0x021D7F4C, v))
        h.w(call(0x021D7F60, v))
    return h.hex()


if __name__ == "__main__":
    which = sys.argv[1:] or ["sick_cont", "event_var", "pure", "rng", "hooks"]
    for name in which:
        print(name, globals()[name]())
