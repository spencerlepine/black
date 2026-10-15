"""Z24: ROM side of the FX / VEC functions that use the DS divider and square-root unit (the generic check-fn emulator leaves
the I/O page 0x04000000 as plain memory, so it cannot run them).

    /tmp/blk/venv/bin/python -I tools/rom-diff/fx_hw_diff.py            # prints "ADDR HASH" per function
    /tmp/blk/venv/bin/python -I tools/rom-diff/fx_hw_diff.py --selftest # divider / sqrt model sanity check

The divider (DIVCNT 0x04000280, DIV_NUMER 0x04000290, DIV_DENOM 0x04000298 -> DIV_RESULT 0x040002A0, DIVREM_RESULT 0x040002A8) and
the square-root unit (SQRTCNT 0x040002B0, SQRT_RESULT 0x040002B4, SQRT_PARAM 0x040002B8) are computed on every read of the register
block 0x04000280..0x040002BF from the register contents at that moment (the busy flags always read 0). Rules, from GBATEK:
truncating signed division; modes 0 (32 / 32), 1 (64 / 32), 2 (64 / 64); a zero divisor gives quotient -1 (numerator >= 0) or +1 with
the remainder = numerator (mode 0: the upper result word inverted); `MIN / -1` wraps (mode 0: 0x80000000, remainder 0); square root
`floor(sqrt)` of the 32-bit (SQRTCNT bit 0 = 0) or 64-bit parameter.

Inputs are generated exactly like `crates/black-core/src/fx/tests.rs::hw_match_rom_model` (xorshift32 seeded with the function
address, 3000 inputs); each result is hashed with FNV-1a over little-endian words. The Rust test holds the recorded hashes.
"""
import math
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from emu import Emu, Rng  # noqa: E402
from unicorn import UC_HOOK_MEM_READ  # noqa: E402

M32 = 0xFFFFFFFF
M64 = 0xFFFFFFFFFFFFFFFF
N_INPUTS = 3000
BUF = 0x02E00000  # input vectors
OUT = 0x02E00100  # output vectors
BOUND = [0, 1, 0xFFFFFFFF, 0x7FFFFFFF, 0x80000000, 0x80000001, 0x1000, 0xFFFFF000, 0x800, 0xFFFFF800, 0x10000, 0xFFFF0000,
         0x40000000, 0xC0000000, 2, 3, 0xFFFFFFFE, 0x1001, 0xFFFFEFFF, 0x7FFF, 0x8000, 0xFFFF8000, 0xFFFF]
FUNCS = [0x0207C768, 0x0207C778, 0x0207C7B4, 0x0207C7E8, 0x0207C958, 0x0207C994, 0x0207CBD0, 0x0207CC30, 0x0207CCA4, 0x0207CDB8,
         0x0207CF3C, 0x0207CFB8]


def s32(v):
    v &= M32
    return v - (1 << 32) if v >> 31 else v


def s64(v):
    v &= M64
    return v - (1 << 64) if v >> 63 else v


def word(r):
    k = r.next() % 8
    if k < 2:
        return BOUND[r.next() % len(BOUND)]
    if k < 5 and k != 4:
        return r.next()
    if k == 4:
        return r.next() % 0x20000
    if k == 5:
        return (-(r.next() % 0x20000)) & M32
    if k == 6:
        return r.next() & 0xFFFFFF
    return (BOUND[r.next() % len(BOUND)] + (r.next() % 9) - 4) & M32


class Hw:
    """The divider / square-root unit, computed on a read of its register block."""

    def __init__(self, e):
        self.e = e
        e.uc.hook_add(UC_HOOK_MEM_READ, self.read, begin=0x04000280, end=0x040002BF)

    def read(self, uc, access, addr, size, value, _):
        e = self.e
        cnt = struct.unpack("<H", e.rbytes(0x04000280, 2))[0] & 3
        num = struct.unpack("<Q", e.rbytes(0x04000290, 8))[0]
        den = struct.unpack("<Q", e.rbytes(0x04000298, 8))[0]
        if cnt == 0:
            n, d = s32(num), s32(den)
        elif cnt == 1:
            n, d = s64(num), s32(den)
        else:
            n, d = s64(num), s64(den)
        if d == 0:
            q, rem = (1 if n < 0 else -1), n
            if cnt == 0:
                q = 0xFFFFFFFF00000001 if n < 0 else 0x00000000FFFFFFFF
        elif cnt == 0 and n == -(1 << 31) and d == -1:
            q, rem = 0x80000000, 0
        else:
            q = abs(n) // abs(d)
            if (n < 0) != (d < 0):
                q = -q
            rem = n - q * d
        e.wbytes(0x040002A0, struct.pack("<QQ", q & M64, rem & M64))
        e.wbytes(0x04000280, struct.pack("<H", struct.unpack("<H", e.rbytes(0x04000280, 2))[0] & 0x7FFF))
        scnt = struct.unpack("<H", e.rbytes(0x040002B0, 2))[0] & 1
        p = struct.unpack("<Q", e.rbytes(0x040002B8, 8))[0]
        if not scnt:
            p &= M32
        e.wbytes(0x040002B4, struct.pack("<I", math.isqrt(p)))
        e.wbytes(0x040002B0, struct.pack("<H", struct.unpack("<H", e.rbytes(0x040002B0, 2))[0] & 0x7FFF))


class Fnv:
    def __init__(self):
        self.h = 14695981039346656037

    def w(self, v):
        for x in struct.pack("<I", v & M32):
            self.h = ((self.h ^ x) * 1099511628211) & M64


def run(e, addr):
    r = Rng(addr)
    h = Fnv()
    for _ in range(N_INPUTS):
        if addr in (0x0207C768, 0x0207C958, 0x0207C994):  # (a, b)
            a, b = word(r), word(r)
            h.w(e.call(addr, a, b, thumb=False)[0])
        elif addr in (0x0207C778, 0x0207C7B4, 0x0207C7E8):  # (x)
            h.w(e.call(addr, word(r), thumb=False)[0])
        elif addr in (0x0207CBD0, 0x0207CC30):  # (v)
            w = [word(r) for _ in range(3)]
            if addr == 0x0207CC30:
                e.wbytes(BUF, struct.pack("<3H", *[x & 0xFFFF for x in w]))
            else:
                e.wbytes(BUF, struct.pack("<3I", *w))
            h.w(e.call(addr, BUF, thumb=False)[0])
        elif addr in (0x0207CCA4, 0x0207CDB8):  # (v, out)
            w = [word(r) for _ in range(3)]
            e.wbytes(OUT, bytes(16))
            if addr == 0x0207CDB8:
                e.wbytes(BUF, struct.pack("<3H", *[x & 0xFFFF for x in w]))
                e.call(addr, BUF, OUT, thumb=False)
                for x in struct.unpack("<3H", e.rbytes(OUT, 6)):
                    h.w(x)
            else:
                e.wbytes(BUF, struct.pack("<3I", *w))
                e.call(addr, BUF, OUT, thumb=False)
                for x in struct.unpack("<3I", e.rbytes(OUT, 12)):
                    h.w(x)
        elif addr == 0x0207CF3C:  # (a, b)
            w = [word(r) for _ in range(6)]
            e.wbytes(BUF, struct.pack("<6I", *w))
            h.w(e.call(addr, BUF, BUF + 12, thumb=False)[0])
        elif addr == 0x0207CFB8:  # (y, x), never i32::MIN (the original reads outside its table)
            y, x = word(r), word(r)
            y = 0x80000001 if y == 0x80000000 else y
            x = 0x80000001 if x == 0x80000000 else x
            h.w(e.call(addr, y, x, thumb=False)[0] & 0xFFFF)
        else:
            raise ValueError(hex(addr))
    return f"{h.h:016x}"


def selftest(e):
    assert e.call(0x0207C768, 3 << 12, 2 << 12, thumb=False)[0] == 0x1800  # FX_Div(3, 2) = 1.5
    assert e.call(0x0207C7B4, 4 << 12, thumb=False)[0] == 2 << 12  # FX_Sqrt(4) = 2
    assert e.call(0x0207C958, (-7) & M32, 2, thumb=False)[0] == (-3) & M32  # CP division truncates
    assert e.call(0x0207C994, (-7) & M32, 2, thumb=False)[0] == (-1) & M32
    e.wbytes(BUF, struct.pack("<3I", 3 << 12, 0, 4 << 12))
    assert e.call(0x0207CBD0, BUF, thumb=False)[0] == 5 << 12  # |(3, 0, 4)| = 5
    print("selftest ok")


def main():
    e = Emu(overlays=[])
    Hw(e)
    if "--selftest" in sys.argv:
        selftest(e)
        return
    for a in FUNCS:
        print("%08x %s" % (a, run(e, a)))


if __name__ == "__main__":
    main()
