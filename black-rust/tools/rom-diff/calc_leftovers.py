"""Differential hashes for black-battle::{calc_target, calc_action} (battle_calc.s leftovers).

Mirrors the `*_matches_rom_fuzz` tests in crates/black-battle/src/calc_target.rs and
calc_action.rs exactly. Reuses the emulator and shared stubs from gen.py.

    /tmp/blk/venv/bin/python -I tools/rom-diff/calc_leftovers.py [target|speed|id_set|action]
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen import e, add_mon, mons, call, Fnv, Rng, M, SCRATCH, RNGCTX, LCG_MUL, LCG_ADD  # noqa: E402

BPP = 0x02E20000
BPP_SIZE = 0x1F8
SAVED = 0x02E30000
MAIN = 0x02E40000


def mix(a, b):
    x = ((a * 0x9E3779B1) ^ (b * 0x85EB_CA77)) & M
    return (x ^ (x >> 15)) & M


class TMon:
    def __init__(self, poke_id, t1, t2):
        self.poke_id = poke_id
        self.t1 = t1
        self.t2 = t2
        self.species = 0
        self.max_hp = 0
        self.level = 0
        self.form = 0


# ---- targeting ------------------------------------------------------------------------------
tlog = []
cfg = {"rule": 0}


def s_rule(em, *_, sp):
    tlog.append((1, 0, 0, 0))
    return cfg["rule"]


def s_8d8c(em, main, arg1, a, *_, sp):
    tlog.append((2, arg1, a, 0))
    return mix(arg1, a) % 8


def s_waza_param(em, waza, i, *_, sp):
    tlog.append((3, waza, i, 0))
    return mix(waza, i) % 16


def s_waza_flag(em, waza, i, *_, sp):
    tlog.append((4, waza, i, 0))
    return mix(waza, i)


def s_8c50(em, main, pos, a, *_, sp):
    tlog.append((5, pos, a, 0))
    return mix(pos, a + 5)


def s_8d38(em, main, arg1, code, out, *_, sp):
    tlog.append((6, arg1, code, 0))
    n = mix(arg1, code) % 5
    em.wbytes(out, bytes(mix(code, i) & 0xFF for i in range(n)))
    return n


def s_8778(em, main, code, out, *_, sp):
    tlog.append((7, code, 0, 0))
    n = mix(code, 0x77) % 5
    em.wbytes(out, bytes(mix(code, 0x100 + i) & 0xFF for i in range(n)))
    return n


def s_82b8(em, flow, a, *_, sp):
    tlog.append((8, a, 0, 0))
    return mix(a, 0x82) & 1


e.stub(0x021B8588, s_rule)
e.stub(0x021B8D8C, s_8d8c)
e.stub(0x0201BD28, s_waza_param)
e.stub(0x0201BEB8, s_waza_flag)
e.stub(0x021B8C50, s_8c50)
e.stub(0x021B8D38, s_8d38)
e.stub(0x021B8778, s_8778)
e.stub(0x021C82B8, s_82b8)


def target():
    h = Fnv()
    for seed in range(400):
        r = Rng(seed)
        hi = r.next()
        lo = r.next()
        x = (hi << 32) | lo
        e.wbytes(RNGCTX, struct.pack("<QQQ", x, LCG_MUL, LCG_ADD))
        mons.clear()
        mon = add_mon(TMon(r.next() % 24, r.next() % 18, r.next() % 18), 0)
        for _ in range(20):
            op = r.next() % 4
            a = r.next()
            b = r.next()
            cfg["rule"] = r.next() % 3
            arg1 = a % 8
            waza = 0xAE if b % 6 == 0 else (b >> 8) % 300
            if op in (0, 1):
                v = call(0x021D80BC, MAIN, arg1, mon, waza)
            elif op == 2:
                v = call(0x021D85C0, MAIN, a % 120)
            else:
                v = e.call(0x021D81F8, MAIN, arg1, mon, waza, stack_args=(SAVED,))[0]
            h.w(v)
            xx = struct.unpack("<Q", e.rbytes(RNGCTX, 8))[0]
            h.w(xx & M)
            h.w(xx >> 32)
            if op == 3:
                for byte in e.rbytes(SAVED, 24):
                    h.b(byte)
            for ent in tlog:
                for val in ent:
                    h.w(val)
            tlog.clear()
    return h.hex()


# ---- unstaged speed (021d85b0) --------------------------------------------------------------
def speed():
    h = Fnv()
    slog = []
    field = {"v": False}

    def field_fx(em, i, *_, sp):
        slog.append((10, i, 0, 0))
        return int(field["v"])

    e.stub(0x021EF948, field_fx)
    for seed in range(500):
        r = Rng(0x5D00 + seed)
        b = bytearray()
        for _ in range(BPP_SIZE // 4):
            b += struct.pack("<I", r.next())
        b[0x140] %= 5
        b[0x1EF] %= 3
        for i in range(3):
            b[0x1EC + i] %= 7
        b[0xD0] %= 25
        e.wbytes(BPP, bytes(b))
        field["v"] = r.next() & 1 != 0
        h.w(call(0x021D85B0, BPP))
        for ent in slog:
            for val in ent:
                h.w(val)
        slog.clear()
    return h.hex()


# ---- id set (021d8230..021d8270) ------------------------------------------------------------
def id_set():
    h = Fnv()
    r = Rng(0x1D5E)
    for _ in range(3000):
        call(0x021D8230, SCRATCH)
        for _ in range(r.next() % 7):
            id_ = r.next()
            call(0x021D8244, SCRATCH, (id_ & 0xFF) if r.next() % 4 == 0 else id_ % 5)
        h.w(call(0x021D826C, SCRATCH))
        for byte in e.rbytes(SCRATCH + 4, 4):
            h.b(byte)
        client = r.next() % 6
        lim = r.next() % 4
        h.w(call(0x021D8270, SCRATCH, client, lim))
        h.w(call(0x021B902C, r.next() % 6, client))
        a, b = r.next(), r.next()
        h.w(call(0x021B902C, a, b))
        h.w(call(0x021B902C, a & 3, b))
    return h.hex()


# ---- action records -------------------------------------------------------------------------
def action():
    h = Fnv()
    r = Rng(0xAC71)
    for _ in range(6000):
        w = r.next()
        if r.next() % 2 == 0:
            w = (w & ~0xF & M) | r.next() % 10
        a, b, c = r.next(), r.next(), r.next()

        def run(addr, *args):
            e.w32(SCRATCH, w)
            call(addr, SCRATCH, *args)
            return e.r32(SCRATCH)

        e.w32(SCRATCH, w)
        h.w(call(0x021D8454, SCRATCH))
        h.w(call(0x021D832C, SCRATCH))
        h.w(call(0x021D8340, SCRATCH))
        h.w(call(0x021D83E0, SCRATCH))
        call(0x021D82FC, SCRATCH, 6 if a % 3 == 0 else b)
        h.w(e.r32(SCRATCH))
        call(0x021D8318, SCRATCH)
        h.w(e.r32(SCRATCH))
        h.w(run(0x021D82D0, a, b))
        h.w(run(0x021D8358, a, b, c))
        h.w(run(0x021D8394, a, b))
        h.w(run(0x021D83C4))
        h.w(run(0x021D83FC, a))
        h.w(run(0x021D8414))
        h.w(run(0x021D8424))
        h.w(run(0x021D8434))
        h.w(run(0x021D8444))
    return h.hex()


# ---- leftovers2: 021d7d6c, 021d7e10, 021d7e90, 021d7f88, 021d7fbc (calc_leftovers2.rs) -------
def leftovers2():
    h = Fnv()
    st = {"s": 0, "s2": 0}
    lg = []
    PA, PB = SCRATCH + 0x100, SCRATCH + 0x104
    pmem = {}

    def s_entry(em, i, *_, sp):
        lg.append((10, i, 0, 0))
        m = mix(i, st["s"])
        return 0 if m % 4 == 0 else m % 40 + 1

    def s_has(em, bpp, sick, *_, sp):
        lg.append((11, sick, 0, 0))
        return mix(sick, st["s2"]) & 1

    def s_aa18(em, item, *_, sp):
        lg.append((1, item, 0, 0))
        return mix(item, 0xAA)

    def s_ab28(em, item, idx, *_, sp):
        lg.append((2, item, idx, 0))
        return mix((item + idx) & M, 0xAB)

    def s_a230(em, v, k, *_, sp):
        lg.append((3, v, k, 0))
        return mix(v, k) % 1000

    def s_pgv(em, hd, id_, arg, *_, sp):
        lg.append((4, hd, id_, arg))
        return mix(hd, id_) % 500

    e.stub(0x021D7E34, s_entry)
    e.stub(0x021D62C4, s_has)
    e.stub(0x0201AA18, s_aa18)
    e.stub(0x0201AB28, s_ab28)
    e.stub(0x0202A230, s_a230)
    e.stub(0x02017E1C, s_pgv)
    r = Rng(0xB1B2)
    for _ in range(3000):
        v = r.next()
        v = v if r.next() % 2 == 0 else ((v % 12) - 3) & M
        h.w(call(0x021D7D6C, v))
        count = r.next() % 8
        lst = [r.next() % 12 for _ in range(8)]
        value = r.next() % 12
        e.wbytes(SCRATCH, b"".join(struct.pack("<H", x) for x in lst))
        h.w(call(0x021D7E90, SCRATCH, count, value))

        st["s"] = r.next()
        st["s2"] = r.next()
        h.w(call(0x021D7E10, BPP))
        for ent in lg:
            for val in ent:
                h.w(val)
        lg.clear()

        p = [r.next() % 5 for _ in range(24)]
        p[0] = r.next() % 3
        p[0x13] = PA if r.next() % 4 != 0 else 0
        p[0x15] = PB if r.next() % 4 != 0 else 0
        e.w32(PA, r.next())
        e.w32(PB, r.next())
        e.wbytes(SAVED, struct.pack("<24I", *p))
        h.w(call(0x021D7F88, SAVED))
        for ent in lg:
            for val in ent:
                h.w(val)
        lg.clear()
    return h.hex()


if __name__ == "__main__":
    which = sys.argv[1:] or ["target", "speed", "id_set", "action", "leftovers2"]
    for name in which:
        print(name, globals()[name]())
