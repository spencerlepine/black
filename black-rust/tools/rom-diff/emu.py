"""Run original Pokemon Black ARM9 + overlay 93 machine code under Unicorn.

Reads the decomp's build products read-only; nothing is written back.
"""
import os
import struct
import sys
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE
from unicorn.arm_const import *

B = os.environ.get("BLACK_ROM_BUILD", os.path.join(os.path.dirname(__file__), "../../../black-rom/build/black.us"))
RET = 0x02FFF000
REGS = [UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3]


def overlay_base(n):
    """Load address of overlay `n` (first code section `OVY_<n>` of the build's main.elf)."""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "re"))
    import mkelf
    _, _, secs = mkelf.read_elf(f"{B}/main.elf")
    return next(s["addr"] for s in secs if s["name"] == "OVY_%d" % n and s["type"] == mkelf.SHT_PROGBITS)


class Emu:
    """`overlays=None`: ARM9 main + overlay 93 (battle), as always. `overlays=[10, 11, ...]`:
    ARM9 main + those overlays at their link addresses (a `tools/re/sets.txt` set); `ovy` is empty."""

    def __init__(self, overlays=None):
        uc = Uc(UC_ARCH_ARM, UC_MODE_ARM)
        uc.mem_map(0x02000000, 0x01000000)  # main RAM (generous)
        main = open(f"{B}/main.sbin", "rb").read()
        uc.mem_write(0x02004000, main)
        self.images = [(0x02004000, main)]  # (base, bytes) of everything loaded from the build
        if overlays is None:
            self.ovy = open(f"{B}/OVY_93.sbin", "rb").read()
            uc.mem_write(0x021B6100, self.ovy)
            self.images.append((0x021B6100, self.ovy))
        else:
            self.ovy = b""
            for n in overlays:
                blob = open(f"{B}/OVY_{n}.sbin", "rb").read()
                base = overlay_base(n)
                uc.mem_write(base, blob)
                self.images.append((base, blob))
        self.uc = uc
        self.stubs = {}
        self.traces = {}
        uc.hook_add(UC_HOOK_CODE, self._hook)

    def _hook(self, uc, addr, size, _):
        if addr == RET:
            uc.emu_stop()
            return
        tr = self.traces.get(addr)
        if tr is not None:
            tr(self, *[uc.reg_read(r) for r in REGS])
        fn = self.stubs.get(addr)
        if fn is not None:
            args = [uc.reg_read(r) for r in REGS]
            sp = uc.reg_read(UC_ARM_REG_SP)
            r = fn(self, *args, sp=sp)
            if r is not None:
                uc.reg_write(UC_ARM_REG_R0, r & 0xFFFFFFFF)
            lr = uc.reg_read(UC_ARM_REG_LR)
            # Unicorn selects Thumb from bit 0 of the value written to PC (interworking return).
            uc.reg_write(UC_ARM_REG_PC, lr)

    def trace(self, addr, fn):
        """Observe `addr` without replacing it: fn(emu, r0, r1, r2, r3)."""
        self.traces[addr & ~1] = fn

    def load_thumb(self, addr, source):
        """Assemble Thumb `source` with arm-none-eabi-as and load it at `addr`."""
        import subprocess
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            open(f"{d}/a.s", "w").write(source)
            subprocess.run(["arm-none-eabi-as", "-mthumb", "-march=armv5te", "-o", f"{d}/a.o", f"{d}/a.s"], check=True)
            subprocess.run(["arm-none-eabi-objcopy", "-O", "binary", "-j", ".text", f"{d}/a.o", f"{d}/a.bin"], check=True)
            self.uc.mem_write(addr, open(f"{d}/a.bin", "rb").read())

    def stub(self, addr, fn):
        self.stubs[addr & ~1] = fn

    def call(self, addr, *args, stack_args=(), thumb=True):
        uc = self.uc
        sp = 0x02F00000
        for i, a in enumerate(stack_args):
            uc.mem_write(sp + 4 * i, struct.pack("<I", a & 0xFFFFFFFF))
        for r, a in zip(REGS, args):
            uc.reg_write(r, a & 0xFFFFFFFF)
        uc.reg_write(UC_ARM_REG_SP, sp)
        uc.reg_write(UC_ARM_REG_LR, RET)
        uc.emu_start(addr | (1 if thumb else 0), RET, count=2_000_000)
        return uc.reg_read(UC_ARM_REG_R0), uc.reg_read(UC_ARM_REG_R1)

    def r32(self, a):
        return struct.unpack("<I", self.uc.mem_read(a, 4))[0]

    def w32(self, a, v):
        self.uc.mem_write(a, struct.pack("<I", v & 0xFFFFFFFF))

    def rbytes(self, a, n):
        return bytes(self.uc.mem_read(a, n))

    def wbytes(self, a, b):
        self.uc.mem_write(a, bytes(b))


class Fnv:
    def __init__(self):
        self.h = 14695981039346656037

    def b(self, v):
        self.h ^= v & 0xFF
        self.h = (self.h * 1099511628211) & 0xFFFFFFFFFFFFFFFF

    def w(self, v):
        for x in struct.pack("<I", v & 0xFFFFFFFF):
            self.b(x)

    def hex(self):
        return f"{self.h:016x}"


class Rng:
    """Same xorshift32 as black-core's differential tests."""

    def __init__(self, seed):
        s = (seed * 2654435761 + 1) & 0xFFFFFFFF
        self.s = s if s else 1

    def next(self):
        x = self.s
        x ^= (x << 13) & 0xFFFFFFFF
        x ^= x >> 17
        x ^= (x << 5) & 0xFFFFFFFF
        self.s = x
        return x
