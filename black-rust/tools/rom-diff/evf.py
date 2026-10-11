"""Differential suite for black-battle::event_factor (event factor list, overlay 93).

Handlers are tiny Thumb routines assembled here; several call back into the original
add/remove/dispatch code so re-entrant behaviour is exercised for real. Mirrors
`event_factor::tests`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from emu import Emu, Fnv, Rng  # noqa: E402
from bpp import mix, random_bpp, SIZE  # noqa: E402

M = 0xFFFFFFFF
G = 0x021F3CB0
POOL = 0x021F4470
EV = 0x021F3ECC
STUBS = 0x02F10000
TABLES = 0x02F20000
MONS = 0x02E30000
CB1 = 0x02F18000
CB2 = 0x02F18010
NONE = 0xFF

H = {name: STUBS + 0x40 * i for i, name in enumerate(
    ["log", "remove", "nest", "add", "work", "reenter", "e73", "nest_reenter"])}
TABLE_DEFS = [
    [(0, "log"), (1, "nest"), (2, "work"), (0x73, "log")],
    [(0, "work"), (2, "remove"), (1, "add"), (3, "log")],
    [(1, "nest_reenter"), (0, "e73"), (2, "log"), (0x73, "work")],
    [(2, "add"), (0, "remove"), (1, "reenter"), (3, "work")],
]
TABLE_ADDR = [TABLES + 0x40 * i for i in range(4)]

ASM = f"""
    .syntax unified
    .thumb
    .text
    .org 0x{H['log'] - STUBS:x}
    bx lr
    .org 0x{H['remove'] - STUBS:x}
    ldr r3, =0x021D7041
    bx r3
    .ltorg
    .org 0x{H['nest'] - STUBS:x}
    push {{r4, lr}}
    adds r0, r1, #0
    movs r1, #2
    ldr r3, =0x021D70F1
    blx r3
    pop {{r4, pc}}
    .ltorg
    .org 0x{H['add'] - STUBS:x}
    push {{r4, lr}}
    sub sp, #12
    movs r0, #3
    str r0, [sp]
    ldr r0, ={TABLE_ADDR[1]:#x}
    str r0, [sp, #4]
    movs r0, #2
    str r0, [sp, #8]
    movs r0, #0
    movs r1, #0x55
    movs r2, #1
    movs r3, #0x80
    ldr r4, =0x021D6F0D
    blx r4
    add sp, #12
    pop {{r4, pc}}
    .ltorg
    .org 0x{H['work'] - STUBS:x}
    ldr r2, [r3]
    adds r2, #1
    str r2, [r3]
    bx lr
    .org 0x{H['reenter'] - STUBS:x}
    ldr r3, =0x021D70C9
    bx r3
    .ltorg
    .org 0x{H['e73'] - STUBS:x}
    ldr r3, =0x021D70BD
    bx r3
    .ltorg
    .org 0x{H['nest_reenter'] - STUBS:x}
    push {{r4, r5, r6, lr}}
    adds r5, r1, #0
    ldr r3, =0x021D70C9
    blx r3
    adds r0, r5, #0
    movs r1, #2
    ldr r3, =0x021D70E5
    blx r3
    pop {{r4, r5, r6, pc}}
    .ltorg
"""


def idx(p):
    return NONE if p == 0 else (p - POOL) // 0x3C


class Ctx:
    def __init__(self):
        e = self.e = Emu()
        e.load_thumb(STUBS, ASM)
        for t, defs in zip(TABLE_ADDR, TABLE_DEFS):
            e.wbytes(t, b"".join(struct.pack("<II", ev, H[h] | 1) for ev, h in defs))
        self.log = []
        self.field = False
        for name, a in H.items():
            e.trace(a, lambda em, r0, r1, r2, r3, a=a: self.log.append(
                (1, a, idx(r0), r1, r2 & 0xFF, idx(r3 - 0x1C))))
        e.stub(0x021C7CF4, self.get_mon)
        e.stub(0x021EF948, self.field_fx)
        e.stub(0x021E03E8, self.f3e8)
        e.stub(CB1, lambda em, r0, r1, r2, r3, sp: self.cb(CB1, r0, r1, r2, r3, sp))
        e.stub(CB2, lambda em, r0, r1, r2, r3, sp: self.cb(CB2, r0, r1, r2, r3, sp))

    def get_mon(self, e, ctx, pid, *_, sp):
        self.log.append((2, ctx, pid, 0, 0, 0))
        return MONS + pid * 0x200 if pid < 4 else 0

    def field_fx(self, e, i, *_, sp):
        self.log.append((3, i, 0, 0, 0, 0))
        return int(self.field)

    def f3e8(self, e, f, w, *_, sp):
        self.log.append((4, idx(f), idx(w - 0x1C), 0, 0, 0))
        return mix(idx(f), 0x3E8) & 1

    def cb(self, which, g, ctx, ftype, event, sp):
        sub = struct.unpack("<I", self.e.rbytes(sp, 4))[0]
        owner = struct.unpack("<I", self.e.rbytes(sp + 4, 4))[0]
        self.log.append((5, which, idx(g), ctx, ftype, event))
        self.log.append((6, sub, owner, 0, 0, 0))
        if which == CB2:
            return 0
        return mix(idx(g) ^ (ftype << 8), event ^ (sub << 8) ^ (owner << 24)) & 1

    def run(self, addr, *args, stack=()):
        return self.e.call(addr, *[a & M for a in args], stack_args=stack)[0]

    def state(self, h):
        e = self.e
        for i in range(0x84):
            b = e.rbytes(POOL + 0x3C * i, 0x3C)
            w = struct.unpack("<15I", b)
            h.w(idx(w[0])); h.w(idx(w[1])); h.w(w[2]); h.w(w[3]); h.w(w[4]); h.w(w[5]); h.w(w[6])
            for k in range(7):
                h.w(w[7 + k])
            h.w(b[0x38] | b[0x39] << 8); h.b(b[0x3A]); h.b(b[0x3B])
        h.w(e.r32(G)); h.w(idx(e.r32(G + 4))); h.w(e.r32(G + 8))
        for i in range(0x84):
            h.w(idx(e.r32(G + 0xC + 4 * i)))
        for x in e.rbytes(EV, 0x5A4):
            h.b(x)
        for ent in self.log:
            for v in ent:
                h.w(v)
        self.log.clear()


def slot(a):
    return POOL + 0x3C * (a % 0x84)


def op(c, o, a, b, d):
    if o == 0:
        c.run(0x021D6EAC); return []
    if o in (1, 2, 3):
        r = c.run(0x021D6F0C, a % 8, b & 0xFFFF, (b >> 16) % 8, d & 0xFFFFFF,
                  stack=((a >> 8) % 6, TABLE_ADDR[(a >> 16) % 4], (a >> 24) % 5))
        return [idx(r)]
    if o == 4:
        c.run(0x021D7040, slot(a)); return []
    if o == 5:
        c.run(0x021D701C); return []
    if o == 6:
        c.run(0x021D6EE8); return []
    ev = 0x73 if a & 0x100 else a % 4
    if o == 7:
        c.run(0x021D70E4, b, ev); return []
    if o == 8:
        c.run(0x021D70F0, b, ev); return []
    if o == 9:
        c.run(0x021D70FC, b, ev, d % 8); return []
    if o == 10:
        c.run(0x021D7320, slot(a), (CB1 if b & 1 else CB2) | 1); return []
    if o == 11:
        c.run(0x021D7324, slot(a)); return []
    if o == 12:
        return [idx(c.run(0x021D732C, a % 8, b % 6)), idx(c.run(0x021D7360, slot(d)))]
    if o == 13:
        c.run(0x021D7394, b % 6); return []
    if o == 14:
        c.run(0x021D73D4, b % 6); return []
    if o == 15:
        c.run(0x021D7404, b % 6, a % 8); return []
    if o == 16:
        return [c.run(0x021D7438, b % 6, a % 8)]
    if o == 17:
        s = slot(a)
        r = [c.run(0x021D70A8, s), c.run(0x021D70AC, s), c.run(0x021D70B4, s, d % 7),
             c.run(0x021D7098, s)]
        k = b % 5
        if k == 0:
            c.run(0x021D70DC, s, d % 7, b)
        elif k == 1:
            c.run(0x021D70BC, s)
        elif k == 2:
            c.run(0x021D70C8, s)
        elif k == 3:
            c.run(0x021D7090, s)
        return r
    if o == 18:
        if a & 1:
            c.run(0x021D74E4)
        else:
            c.run(0x021D74C8)
        return []
    raise ValueError(o)


N_OPS = 19


def suite():
    c = Ctx()
    h = Fnv()
    for seed in range(120):
        r = Rng(0xE7 + seed)
        for i in range(4):
            c.e.wbytes(MONS + i * 0x200, random_bpp(r))
        c.field = bool(r.next() & 1)
        c.e.wbytes(EV, bytes(0x5A4))
        e_sp = r.next() % 3
        c.e.wbytes(EV, struct.pack("<I", e_sp))
        c.run(0x021D6EAC)
        c.log.clear()
        for _ in range(60):
            o = r.next() % N_OPS
            a, b, d = r.next(), r.next(), r.next()
            for v in op(c, o, a, b, d):
                h.w(v)
            c.state(h)
    return h.hex()


if __name__ == "__main__":
    print("event_factor", suite())
