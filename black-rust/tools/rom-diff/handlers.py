"""Generic differential driver for battle event handlers. Mirrors handlers/testkit.rs.

Usage: python -I handlers.py abilities
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
NMONS = 8
HANDEX_SIZE = 0x80
LABELS = [2, 3, 4, 5, 6, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 27, 29, 30, 31, 32, 33,
          34, 36, 37, 38, 39, 40, 42, 43, 44, 45, 46, 48, 49, 50, 51, 53, 55, 56, 57, 58, 62, 63,
          64, 65, 66, 68, 69, 70, 71, 72, 75, 77, 81, 83]


class Host:
    def __init__(self, e):
        self.e = e
        self.log = []
        self.nbuf = 0
        self.wseed = 0
        st = e.stub
        st(0x021C88A4, self.push)
        st(0x021C88C8, self.pop)
        st(0x021C88B4, self.push_pop)
        st(0x021C7D00, self.poke_pos)
        st(0x021C8120, lambda em, *a, sp: (em.wbytes(TMP, bytes(16)), TMP)[1])
        st(0x021C7F68, self.expand)
        st(0x021C8288, self.weather)
        st(0x0201C1E8, self.category)
        st(0x021C7CF4, lambda em, wk, i, *a, sp: MONS_ADDR + 0x200 * i if i < NMONS else 0)

    def push(self, e, wk, ty, poke, *_, sp):
        self.log += [0x100, ty, poke]
        a = HX + 0x100 * self.nbuf
        self.nbuf += 1
        e.wbytes(a, bytes(HANDEX_SIZE))
        return a

    def pop(self, e, wk, a, *_, sp):
        self.log.append(0x101)
        self.log += list(struct.unpack(f"<{HANDEX_SIZE // 4}I", e.rbytes(a, HANDEX_SIZE)))

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
        return mix(mv, 0xC1E8) % 3


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


def run(addrs):
    e = Emu()
    e.uc.mem_map(0, 0x10000)
    env = Env(e)
    host = Host(e)
    h = Fnv()
    for addr in addrs:
        for seed in range(200):
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
                struct.pack_into("<I", ev, 0xC4 + 4 * s, value_for(r, me) & M)
                k = [0, 1, 2, 3, 3, 3, 4, 0][r.next() % 8]
                ev[0x544 + s] = k
                if k == 3:
                    struct.pack_into("<I", ev, 0xC4 + 4 * s, 0x100 + r.next() % 0x10000)
                    if r.next() & 1:
                        struct.pack_into("<I", ev, 0x3C4 + 4 * s, r.next() % 0x40)
                        struct.pack_into("<I", ev, 0x244 + 4 * s, r.next() % 0x10000)
            e.wbytes(EV, ev)
            hi, lo = r.next(), r.next()
            e.wbytes(RNGCTX, struct.pack("<QQQ", (hi << 32) | lo, 0x5D588B656C078965, 0x269EC3))
            fac = bytearray(0x3C)
            for k in range(7):
                struct.pack_into("<I", fac, 0x1C + 4 * k, r.next())
            e.wbytes(POOL, fac)
            e.call(addr, POOL, FLOW, me, POOL + 0x1C)
            for w in words(e.rbytes(EV, 0x5A4)):
                h.w(w)
            for i in range(NMONS):
                for w in words(e.rbytes(MONS_ADDR + 0x200 * i, SIZE)):
                    h.w(w)
            for w in words(e.rbytes(0, SIZE)):
                h.w(w)
            for w in words(e.rbytes(POOL + 0x1C, 0x1C)):
                h.w(w)
            x = struct.unpack("<Q", e.rbytes(RNGCTX, 8))[0]
            h.w(x & M); h.w(x >> 32)
            for v in host.log:
                h.w(v)
            for ent in env.log:
                for v in ent:
                    h.w(v)
            host.log.clear(); env.log.clear(); host.nbuf = 0
    return h.hex()


def ported(module):
    src = open(os.path.join(os.path.dirname(__file__), "../../crates/black-battle/src/handlers",
                            module + ".rs")).read()
    block = src[src.index("pub const PORTED"):]
    block = block[:block.index("];")]
    return [int(x, 16) for x in re.findall(r"0x([0-9A-Fa-f]{8})", block)]


if __name__ == "__main__":
    for mod in sys.argv[1:] or ["abilities"]:
        print(mod, run(ported(mod)))
