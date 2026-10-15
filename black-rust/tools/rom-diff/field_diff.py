"""F12: the field session of `crates/black-field` (`integration::session`) entirely in the original code under
Unicorn, for the differential against the ports.

  field_diff.py --keys RUST_TRACE [--out ROM_TRACE] [--text TEXT_OUT] [--dump FRAME,FILE] [--frames N]

RUST_TRACE is the trace the Rust test writes (`F12_TRACE_OUT=<file> cargo test -p black-field f12_field_session`):
its `k<keys>` column gives the keys of every frame, which this script replays. It writes ROM_TRACE in the same
format (one line per frame, see `Session::trace_line` in session.rs) and prints `trace <fnv64>` (and the hash of the
printed text); TEXT_OUT gets the printed STRBUFs' units (the format of the Rust side's F12_TEXT_OUT).
tools/check-field-diff.sh runs both sides and compares the traces line by line.

The machine is the one of `crates/black-field/src/integration` (mod.rs / host.rs / view.rs), with the same host models
at the same addresses: the system boundary (heap, file system below the gfl archive code, overlay loader, OS tick,
RTC seed, keys through KEYINPUT and the ARM7 key word, touch panel, VBlank wait, the NNS sound library and the gfl
sound module, the cartridge integrity checks, the backup-device probe, the boot's comm / ARM7 service inits, the
per-frame render functions of fieldmap.c, the entrance camera, the headless message windows). Where the Rust side
runs a port, this side runs the original function; everything else is original code on both sides.
"""
import argparse
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from emu import B  # noqa: E402
from unicorn import (  # noqa: E402
    Uc, UcError, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE, UC_HOOK_MEM_UNMAPPED, UC_HOOK_MEM_WRITE,
    UC_HOOK_MEM_READ, UC_HOOK_INTR,
)
from unicorn.arm_const import (  # noqa: E402
    UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3, UC_ARM_REG_R4, UC_ARM_REG_R5,
    UC_ARM_REG_R6, UC_ARM_REG_R7, UC_ARM_REG_R8, UC_ARM_REG_R9, UC_ARM_REG_R10, UC_ARM_REG_R11,
    UC_ARM_REG_R12, UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_PC, UC_ARM_REG_CPSR,
)

ROOT = os.path.dirname(os.path.dirname(HERE))
FILES = os.environ.get("BLACK_ROM_FILES", os.path.join(ROOT, "..", "black-rom", "files"))
M32 = 0xFFFFFFFF
REGS = [UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3]
ALL_REGS = REGS + [UC_ARM_REG_R4, UC_ARM_REG_R5, UC_ARM_REG_R6, UC_ARM_REG_R7, UC_ARM_REG_R8,
                   UC_ARM_REG_R9, UC_ARM_REG_R10, UC_ARM_REG_R11, UC_ARM_REG_R12, UC_ARM_REG_LR]

MAIN_BASE = 0x02004000
MAIN_CODE_SIZE = 0xA5E80
RET = 0x02FFF000
STACK_TOP = 0x02F00000
HEAP_BASE = 0x02500000
HEAP_END = 0x02E00000
HEAP_ALLOC = 0x02030734
HOST_PAGE = 0x021FF000
ALLOCATOR_ALLOC = HOST_PAGE
ALLOCATOR_FREE = HOST_PAGE + 0x10
ALLOCATOR_FUNCS = HOST_PAGE + 0x100
RNG_SEED = 0x0123456789ABCDEF
BACKUP_STATE = 0x02146994
GAMESYS = 0x02146248
GFL_SYS = 0x020AA194
RTC = 0x021469FC
SCRIPT_EVENT = 0x021588DD

# (lo, hi, answer): every entry answers the constant (host.rs RANGES)
RANGES = [(0x020726D0, 0x02075C00, 0), (0x020057AC, 0x020067E4, 0)]
RENDER = [0x02188DB0, 0x02188DF0, 0x02188E38, 0x02189084, 0x021890AC, 0x021890C8, 0x02189108, 0x02189140,
          0x02189154, 0x021891C0, 0x021897DC, 0x02189810, 0x02189998, 0x021899C4]
SOUND0 = [0x02005CA4, 0x02005D28, 0x02005D90, 0x02005DF8, 0x02005E30, 0x02006734, 0x02006740, 0x02005E50,
          0x020057AC]
INTEGRITY = [0x021882A0, 0x021882DC, 0x02188318, 0x02188354, 0x02188390, 0x021883CC]
# view.rs: the headless windows (overlay 21)
WINDOWS = {
    0x021904DC: None, 0x02190708: "new", 0x02190728: None, 0x02190758: ("board", 3), 0x02190788: None,
    0x021909BC: None, 0x02190BE4: "new", 0x02190C78: None, 0x02190FA0: "new", 0x02191008: None,
    0x02191084: ("talk", 3), 0x021910C0: 1, 0x02191134: None, 0x02191164: None, 0x02191184: ("talk", 3),
    0x021911B4: "new", 0x0219129C: ("balloon", 3, "new"), 0x021912D8: None, 0x021912F8: 1, 0x0219131C: None,
    0x02191348: ("balloon", 1), 0x02191388: 1, 0x021914CC: None, 0x02191558: "new", 0x021915B4: None,
    0x021915EC: None, 0x021916A8: ("system", 3), 0x021916E0: 1, 0x02191794: None, 0x02191830: "size",
    0x021918DC: None, 0x0219196C: None, 0x02191978: None, 0x021919AC: None, 0x02191A04: "new",
    0x02191A10: "new", 0x02191B14: None, 0x02191BE8: ("window 67", 1, 1), 0x02192570: "new", 0x02192628: None,
    0x0219269C: ("window 69", 3), 0x021926D8: 1, 0x0219270C: None, 0x0219272C: None,
}


class FieldError(Exception):
    pass


def fnv64(data):
    h = 0xCBF29CE484222325
    for x in data:
        h ^= x
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def mapping_symbols():
    """`$a` mapping symbols of main.elf by section key (None = main / autoloads, n = overlay n)."""
    d = open(os.path.join(B, "main.elf"), "rb").read()
    shoff, = struct.unpack_from("<I", d, 0x20)
    es, n, si = struct.unpack_from("<HHH", d, 0x2E)
    secs = [struct.unpack_from("<10I", d, shoff + i * es) for i in range(n)]

    def cstr(o):
        return d[o:d.index(b"\0", o)].decode()
    names = [cstr(secs[si][4] + s[0]) for s in secs]
    out = {}
    sym = next(s for s in secs if s[1] == 2)
    stroff = secs[sym[6]][4]
    for i in range(sym[5] // 16):
        nm, val, _sz, _info, _oth, shndx = struct.unpack_from("<IIIBBH", d, sym[4] + 16 * i)
        if shndx == 0 or shndx >= len(secs):
            continue
        s = cstr(stroff + nm)
        if s not in ("$a", "$t", "$d"):
            continue
        sec = names[shndx]
        if sec.startswith("OVY_"):
            key = int(sec[4:])
        elif sec in ("main", "ITCM", "DTCM", "MAINEX", "VRAM"):
            key = None
        else:
            continue
        out.setdefault(key, []).append((val, s == "$a"))
    for v in out.values():
        v.sort()
    return out


TICKS_PER_FRAME = 8753  # OS_GetTick (host.rs)
ROOT_HEAP = 0xFFFFFFFF
HEAP_HEADER = 0x40


class Arena:
    """host.rs Arena: one heap; first fit in address order over the freed spans (coalesced), else the top."""

    def __init__(self, lo, end, parent=None):
        self.lo = lo
        self.top = lo
        self.end = end
        self.used = {}
        self.free = {}
        self.parent = parent

    def alloc(self, size, align):
        span = ((max(size, 1) + 3) & ~3) + 0x10

        def up(a):
            return (a + align - 1) & ~(align - 1)
        hit = None
        for a in sorted(self.free):
            s = self.free[a]
            if up(a) + span <= a + s:
                hit = (a, s)
                break
        if hit:
            a, s = hit
            del self.free[a]
            base = up(a)
            if base > a:
                self.free[a] = base - a
            end = a + s
            if end > base + span:
                self.free[base + span] = end - base - span
        else:
            base = up(self.top)
            if base + span > self.end:
                return None
            if base > self.top:
                self.free[self.top] = base - self.top
            self.top = base + span
        self.used[base] = [size, span]
        return base

    def release(self, p):
        if p not in self.used:
            return False
        span = self.used.pop(p)[1]
        a, s = p, span
        prev = [k for k in self.free if k < p]
        if prev:
            pa = max(prev)
            if pa + self.free[pa] == p:
                s += self.free.pop(pa)
                a = pa
        if a + s in self.free:
            s += self.free.pop(a + s)
        if a + s == self.top:
            self.top = a
        else:
            self.free[a] = s
        return True


class HostHeap:
    """host.rs HostHeap: one arena per gfl heap id (Heap_Create carves it out of its parent)."""

    def __init__(self):
        self.arenas = {ROOT_HEAP: Arena(HEAP_BASE, HEAP_END)}

    def arena(self, heap):
        return self.arenas.get(heap, self.arenas[ROOT_HEAP])

    def alloc(self, heap, size, align):
        return self.arena(heap).alloc(size, align)

    def release(self, p):
        return any(a.release(p) for a in list(self.arenas.values()))

    def block(self, p):
        for a in self.arenas.values():
            if p in a.used:
                return a.used[p]
        return None

    def create(self, parent, id_, size):
        if id_ in self.arenas:
            raise FieldError("heap %#x created twice" % id_)
        b = self.alloc(parent, size, 16)
        if b is None:
            return 0
        parent = parent if parent in self.arenas else ROOT_HEAP
        self.arenas[id_] = Arena(b + HEAP_HEADER, b + size, (parent, b))
        return 1

    def create_at(self, addr, size, id_):
        if id_ in self.arenas:
            raise FieldError("heap %#x created twice" % id_)
        self.arenas[id_] = Arena(addr, addr + size)
        return 1

    def destroy(self, id_):
        a = self.arenas.pop(id_, None)
        if a is not None and a.parent is not None and a.parent[0] in self.arenas:
            self.arenas[a.parent[0]].release(a.parent[1])
        return 0

    def free_size(self, heap):
        a = self.arena(heap)
        return a.end - a.lo - sum(v[1] for v in a.used.values())


class Field:
    def __init__(self):
        uc = Uc(UC_ARCH_ARM, UC_MODE_ARM)
        self.uc = uc
        uc.mem_map(0x01FF8000, 0x8000)
        uc.mem_map(0x02000000, 0x01000000)
        uc.mem_map(0x04000000, 0x2000)
        uc.mem_map(0x04004000, 0x1000)
        uc.mem_map(0x04100000, 0x1000)
        uc.mem_map(0x05000000, 0x1000)
        for base, size in ((0x06000000, 0x80000), (0x06200000, 0x20000), (0x06400000, 0x40000),
                           (0x06600000, 0x20000), (0x06800000, 0xA4000)):
            uc.mem_map(base, size)
        uc.mem_map(0x07000000, 0x1000)
        main = open(f"{B}/main.sbin", "rb").read()
        uc.mem_write(MAIN_BASE, main[:MAIN_CODE_SIZE])
        o, tab = MAIN_CODE_SIZE, MAIN_CODE_SIZE + 0x900
        for i in range(4):
            dst, size, _sinit, _bss = struct.unpack_from("<IIII", main, tab + 16 * i)
            uc.mem_write(dst, main[o:o + size])
            o += size
        t = open(f"{B}/main_table.sbin", "rb").read()
        self.ovtab = {}
        for i in range(len(t) // 32):
            e = struct.unpack_from("<8I", t, 32 * i)
            self.ovtab[e[0]] = e[1:6]
        self.ovimg = {}
        self.loaded = set()
        self.arm = mapping_symbols()
        self.heap = HostHeap()
        self.files = {}
        self.tick = 0
        self.frame = 0
        self.printed = []
        self.err = None
        self.sp = STACK_TOP - 0x100
        self.hosts = self._hosts()
        self.hook_addrs = set(self.hosts) | set(WINDOWS)
        for a in self.hook_addrs:
            uc.hook_add(UC_HOOK_CODE, self._hook, None, a, a)
        for lo, hi, _ in RANGES:
            uc.hook_add(UC_HOOK_CODE, self._range_hook, None, lo, hi - 1)
        uc.hook_add(UC_HOOK_MEM_UNMAPPED, self._unmapped)
        uc.hook_add(UC_HOOK_MEM_WRITE, self._io_write, None, 0x04000000, 0x04001FFF)
        uc.hook_add(UC_HOOK_MEM_READ, self._io_read, None, 0x04000200, 0x04000217)
        uc.hook_add(UC_HOOK_MEM_READ, self._dma_read, None, 0x040000B0, 0x040000DF)
        uc.hook_add(UC_HOOK_INTR, self._intr)

    # ------------------------------------------------------------------ memory
    def r8(self, a):
        return self.uc.mem_read(a, 1)[0]

    def r16(self, a):
        return struct.unpack("<H", self.uc.mem_read(a, 2))[0]

    def r32(self, a):
        return struct.unpack("<I", self.uc.mem_read(a, 4))[0]

    def w8(self, a, v):
        self.uc.mem_write(a, bytes([v & 0xFF]))

    def w16(self, a, v):
        self.uc.mem_write(a, struct.pack("<H", v & 0xFFFF))

    def w32(self, a, v):
        self.uc.mem_write(a, struct.pack("<I", v & M32))

    def wb(self, a, b):
        self.uc.mem_write(a, bytes(b))

    def alloc(self, size, align=4, heap=ROOT_HEAP):
        if size >= 0x400000:
            raise FieldError("allocation of %#x bytes" % size)
        p = self.heap.alloc(heap, size, align)
        if p is None:
            raise FieldError("heap %#x exhausted by %#x bytes" % (heap, size))
        if os.environ.get("F12_HEAPLOG"):
            print("HEAP f%d %x %x %x %08x lr %08x" % (self.frame, heap, size, align, p, self.uc.reg_read(UC_ARM_REG_LR)),
                  file=sys.stderr)
        self.wb(p, bytes(size))
        return p

    # ------------------------------------------------------------------ overlays
    def overlay_at(self, a):
        for k in self.loaded:
            base, size = self.ovtab[k][0], self.ovtab[k][1]
            if base <= a < base + size:
                return k
        return None

    def load_overlay(self, n):
        base, size, bss, s0, s1 = self.ovtab[n]
        end = base + size + bss
        for k in list(self.loaded):
            b2, sz2, bss2 = self.ovtab[k][0], self.ovtab[k][1], self.ovtab[k][2]
            if b2 < end and base < b2 + sz2 + bss2:
                self.loaded.discard(k)
        img = open(f"{B}/OVY_{n}.sbin", "rb").read()
        self.wb(base, img)
        self.wb(base + size, bytes(bss))
        self.loaded.add(n)
        for a in range(s0, s1, 4):
            f = struct.unpack_from("<I", img, a - base)[0]
            if f:
                self.call(f)

    def entry_is_arm(self, a):
        if a & 3:
            return False
        v = self.arm.get(self.overlay_at(a), [])
        lo, hi = 0, len(v)
        while lo < hi:
            mid = (lo + hi) // 2
            if v[mid][0] <= a:
                lo = mid + 1
            else:
                hi = mid
        if lo and v[lo - 1][1]:
            return True
        w = self.r32(a)
        low, high = w & 0xFFFF, w >> 16
        if low >> 9 == 0x5A or (low >> 11 == 0x1E and high >> 11 in (0x1D, 0x1F)):
            return False
        return w >> 28 == 0xE

    # ------------------------------------------------------------------ calls
    def call(self, addr, *args):
        """A call from the host side (r0-r3, then stack words); r0 | r1 << 32."""
        a = addr & ~1
        if a in self.hook_addrs and self._active(a):
            return self._host(a, list(args) + [0] * 12)
        for lo, hi, ans in RANGES:
            if lo <= a < hi:
                return ans
        uc = self.uc
        saved = [uc.reg_read(r) for r in ALL_REGS] + [uc.reg_read(UC_ARM_REG_SP), uc.reg_read(UC_ARM_REG_PC),
                                                     uc.reg_read(UC_ARM_REG_CPSR)]
        sp = self.sp & ~7
        extra = list(args[4:])
        sp -= (4 * len(extra) + 7) & ~7
        for k, x in enumerate(extra):
            self.w32(sp + 4 * k, x)
        old_sp = self.sp
        self.sp = sp
        for r in ALL_REGS:
            uc.reg_write(r, 0)
        for r, x in zip(REGS, args):
            uc.reg_write(r, x & M32)
        uc.reg_write(UC_ARM_REG_SP, sp)
        uc.reg_write(UC_ARM_REG_LR, RET)
        thumb = not self.entry_is_arm(a)
        try:
            uc.emu_start(a | (1 if thumb else 0), RET, count=50_000_000)
        except UcError as e:
            raise FieldError("call %08x: %s at pc %08x lr %08x%s" % (
                a, e, uc.reg_read(UC_ARM_REG_PC), uc.reg_read(UC_ARM_REG_LR),
                "" if self.err is None else ": " + self.err))
        if self.err is not None:
            raise FieldError("call %08x: %s" % (a, self.err))
        if uc.reg_read(UC_ARM_REG_PC) != RET:
            raise FieldError("call %08x: no return (pc %08x lr %08x)" % (
                a, uc.reg_read(UC_ARM_REG_PC), uc.reg_read(UC_ARM_REG_LR)))
        r = uc.reg_read(UC_ARM_REG_R0) | uc.reg_read(UC_ARM_REG_R1) << 32
        self.sp = old_sp
        for r_, v in zip(ALL_REGS + [UC_ARM_REG_SP, UC_ARM_REG_PC, UC_ARM_REG_CPSR], saved):
            uc.reg_write(r_, v)
        return r

    def cv(self, addr, *args):
        return self.call(addr, *args) & M32

    def _active(self, a):
        ov = {0x021DFC20: 228, 0x021DFDB8: 228}.get(a)
        if a in INTEGRITY:
            ov = 230
        elif a in RENDER or a in WINDOWS or a == 0x021BDCC8:
            ov = 21
        return ov is None or ov in self.loaded

    def _hook(self, uc, addr, size, _):
        if not self._active(addr):
            return
        sp = uc.reg_read(UC_ARM_REG_SP)
        w = [uc.reg_read(r) for r in REGS] + [self.r32(sp + 4 * k) for k in range(8)]
        saved_sp = self.sp
        self.sp = sp & ~7
        try:
            r = self._host(addr, w)
        except FieldError as e:
            self.err = "%08x: %s" % (addr, e)
            uc.emu_stop()
            return
        self.sp = saved_sp
        uc.reg_write(UC_ARM_REG_R0, r & M32)
        uc.reg_write(UC_ARM_REG_R1, (r >> 32) & M32)
        uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))

    def _range_hook(self, uc, addr, size, _):
        if addr in self.hook_addrs:
            return  # a host model of its own inside the range (host.rs: the models come first)
        for lo, hi, ans in RANGES:
            if lo <= addr < hi:
                uc.reg_write(UC_ARM_REG_R0, ans & M32)
                uc.reg_write(UC_ARM_REG_R1, 0)
                uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))
                return

    def _unmapped(self, uc, access, addr, size, value, _):
        self.err = "unmapped access %08x+%d (pc %08x, lr %08x)" % (
            addr, size, uc.reg_read(UC_ARM_REG_PC), uc.reg_read(UC_ARM_REG_LR))
        return False

    def _intr(self, uc, intno, _):
        pc = uc.reg_read(UC_ARM_REG_PC)
        thumb = uc.reg_read(UC_ARM_REG_CPSR) & 0x20
        n = self.r16(pc - 2) & 0xFF if thumb else (self.r32(pc - 4) >> 16) & 0xFF
        if n in (3, 4, 5, 6):
            return
        if n == 9:
            a = uc.reg_read(UC_ARM_REG_R0)
            b = uc.reg_read(UC_ARM_REG_R1)
            a = a - (1 << 32) if a >> 31 else a
            b = b - (1 << 32) if b >> 31 else b
            q = abs(a) // abs(b)
            if (a < 0) != (b < 0):
                q = -q
            uc.reg_write(UC_ARM_REG_R0, q & M32)
            uc.reg_write(UC_ARM_REG_R1, (a - q * b) & M32)
            uc.reg_write(UC_ARM_REG_R3, abs(q) & M32)
            return
        if n == 0xD:
            import math
            uc.reg_write(UC_ARM_REG_R0, math.isqrt(uc.reg_read(UC_ARM_REG_R0)))
            return
        self.err = "BIOS call %#x at %08x" % (n, pc)
        uc.emu_stop()

    # ------------------------------------------------------------------ I/O (host.rs io_*)
    def _io_read(self, uc, access, addr, size, value, _):
        if addr in (0x04000208, 0x04000210, 0x04000214):
            uc.mem_write(addr, bytes(size))

    def _dma_read(self, uc, access, addr, size, value, _):
        # a memory-write hook runs before the store, so the store sets the enable bit again after _dma ran;
        # the transfer is complete by the next read (host.rs: DMA at once)
        cnt = 0x040000B8 + 12 * ((addr - 0x040000B0) // 12)
        uc.mem_write(cnt + 3, bytes([uc.mem_read(cnt + 3, 1)[0] & 0x7F]))

    def _io_write(self, uc, access, addr, size, value, _):
        uc.mem_write(addr, (value & ((1 << (8 * size)) - 1)).to_bytes(size, "little"))
        if 0x04000280 <= addr < 0x040002C0:
            self._math()
        elif 0x040000B0 <= addr < 0x040000E0:
            self._dma((addr - 0x040000B0) // 12)

    def _math(self):
        import math

        def s64(v):
            return v - (1 << 64) if v >> 63 else v

        def s32(v):
            v &= M32
            return v - (1 << 32) if v >> 31 else v
        cnt = self.r16(0x04000280) & 3
        num = struct.unpack("<Q", self.uc.mem_read(0x04000290, 8))[0]
        den = struct.unpack("<Q", self.uc.mem_read(0x04000298, 8))[0]
        if cnt == 0:
            n, d = s32(num), s32(den)
        elif cnt == 1:
            n, d = s64(num), s32(den)
        else:
            n, d = s64(num), s64(den)
        if d == 0:
            q, rem = (-1 if n >= 0 else 1), n
        else:
            q = abs(n) // abs(d)
            if (n < 0) != (d < 0):
                q = -q
            rem = n - q * d
        self.uc.mem_write(0x040002A0, struct.pack("<Q", q & ((1 << 64) - 1)))
        self.uc.mem_write(0x040002A8, struct.pack("<Q", rem & ((1 << 64) - 1)))
        self.w16(0x04000280, self.r16(0x04000280) & ~0x8000)
        scnt = self.r16(0x040002B0) & 1
        p = struct.unpack("<Q", self.uc.mem_read(0x040002B8, 8))[0]
        if not scnt:
            p &= M32
        self.w32(0x040002B4, math.isqrt(p))
        self.w16(0x040002B0, self.r16(0x040002B0) & ~0x8000)

    def _dma(self, ch):
        base = 0x040000B0 + 12 * ch
        cnt = self.r32(base + 8)
        if not cnt >> 31:
            return
        src, dst = self.r32(base), self.r32(base + 4)
        n = cnt & 0x1FFFFF or 0x200000
        unit = 4 if cnt & (1 << 26) else 2

        def step(ctl):
            return {1: -unit, 2: 0}.get(ctl, unit)
        ds, ss = step((cnt >> 21) & 3), step((cnt >> 23) & 3)
        if not 0x04000000 <= dst < 0x05000000:
            for _ in range(n):
                self.uc.mem_write(dst, bytes(self.uc.mem_read(src, unit)))
                src = (src + ss) & M32
                dst = (dst + ds) & M32
        self.w32(base + 8, cnt & 0x7FFFFFFF)

    # ------------------------------------------------------------------ host models (host.rs / view.rs)
    def _hosts(self):
        h = {}

        def const(v):
            return lambda w: v
        h[0x0203064C] = lambda w: self.heap.create(w[0], w[1], w[2])
        h[0x02030678] = lambda w: self.heap.create_at(w[0], w[1], w[2])
        h[0x020306F0] = lambda w: self.heap.destroy(w[0])
        h[0x02030734] = lambda w: self.alloc(w[1], 4, w[0])
        h[0x020307B0] = self._free
        h[0x020305C4] = lambda w: self.alloc(w[0])
        h[0x020305E4] = lambda w: (self.heap.release(w[0]), 1)[1]
        h[0x020307C8] = self._allocator_init
        h[ALLOCATOR_ALLOC] = lambda w: self.alloc(w[1], max(abs(self._s32(self.r32(w[0] + 8))), 4),
                                                   self.r32(w[0] + 4))
        h[ALLOCATOR_FREE] = lambda w: (self.heap.release(w[1]), 0)[1]
        h[0x020307E0] = self._resize
        h[0x02030830] = lambda w: self.heap.free_size(w[0])
        h[0x02034AC4] = self._ov_load
        h[0x02034A5C] = lambda w: (self.loaded.discard(w[0] & 0xFFFF), 1)[1]
        h[0x02035C60] = const(0)
        for a in (0x0208A924, 0x0208A998, 0x0208AA2C, 0x0208AB1C, 0x0203CB34, 0x020107CC, 0x021BDCC8,
                  0x02087AE4, 0x02006300):
            h[a] = const(0)
        h[0x0208ABB8] = lambda w: (self.wb(w[0], bytes(8)), 0)[1]
        h[0x0208BFD0] = lambda w: (self.w32(w[1], 0), 0)[1]
        h[0x020870D0] = self._tick
        h[0x0203F258] = self._rtc_seed
        h[0x02048DDC] = self._fs_open
        h[0x02020DEC] = self._fs_open
        h[0x02078B44] = self._fs_seek
        h[0x02078B70] = self._fs_read
        h[0x02078A80] = lambda w: (self.files.pop(w[0], None), 1)[1]
        h[0x02078C28] = lambda w: self.r32(w[0] + 0x28) - self.r32(w[0] + 0x24)
        h[0x021DFC20] = lambda w: self.alloc(0x400)
        h[0x021DFDB8] = lambda w: (self.heap.release(w[0]), 0)[1]
        for a in INTEGRITY:
            h[a] = lambda w: ~w[1] & M32
        h[0x020063B4] = lambda w: 1 if w[2] != 1 else 0
        for a in RENDER + SOUND0:
            h[a] = const(0)
        h[0x020061E4] = const(0)
        h[0x020057C4] = const(1)
        return h

    def _host(self, a, w):
        if a in WINDOWS:
            return self._window(a, w)
        r = self.hosts[a](w)
        return r & 0xFFFFFFFFFFFFFFFF

    @staticmethod
    def _s32(v):
        return v - (1 << 32) if v >> 31 else v

    def _free(self, w):
        p = w[0]
        if p and not self.heap.release(p):
            raise FieldError("Heap_Free of unknown block %08x" % p)
        return 0

    def _allocator_init(self, w):
        a = w[0]
        self.w32(ALLOCATOR_FUNCS, ALLOCATOR_ALLOC | 1)
        self.w32(ALLOCATOR_FUNCS + 4, ALLOCATOR_FREE | 1)
        self.w32(a, ALLOCATOR_FUNCS)
        self.w32(a + 4, w[1])
        self.w32(a + 8, w[2])
        self.w32(a + 12, 0)
        return 0

    def _resize(self, w):
        p, n = w[0], w[1]
        b = self.heap.block(p)
        if b is None or n > b[0]:
            raise FieldError("resize %08x to %#x" % (p, n))
        b[0] = n
        return 1

    def _ov_load(self, w):
        n = w[0] & 0xFFFF
        if n not in self.loaded:
            self.load_overlay(n)
        return 1

    def _tick(self, w):
        self.tick = max(self.tick + 1, self.frame * TICKS_PER_FRAME)
        return self.tick

    def _rtc_seed(self, w):
        self.wb(w[0], struct.pack("<QQQ", RNG_SEED, 0x5D588B656C078965, 0x00269EC3))
        return 0

    def _fs_open(self, w):
        f, arc = w[0], w[1]
        path = os.path.join(FILES, "a", str(arc // 100), str(arc // 10 % 10), str(arc % 10))
        data = open(path, "rb").read()
        self.wb(f, bytes(0x48))
        self.w32(f + 0x28, len(data))
        self.files[f] = data
        return 1

    def _fs_seek(self, w):
        f, off, whence = w[0], self._s32(w[1]), w[2]
        if f not in self.files:
            raise FieldError("FS_SeekFile on a closed file")
        top, bottom, pos = self.r32(f + 0x24), self.r32(f + 0x28), self.r32(f + 0x2C)
        p = {0: top, 1: pos, 2: bottom}[whence] + off
        if p < top or p > bottom:
            return 0
        self.w32(f + 0x2C, p)
        return 1

    def _fs_read(self, w):
        f, dst, n = w[0], w[1], w[2]
        pos, bottom = self.r32(f + 0x2C), self.r32(f + 0x28)
        n = min(n, bottom - pos)
        self.wb(dst, self.files[f][pos:pos + n])
        self.w32(f + 0x2C, pos + n)
        return n

    def _window(self, a, w):
        m = WINDOWS[a]
        if m == "new":
            return self.alloc(0x40)
        if m == "size":
            self.w8(w[2], 1)
            self.w8(w[3], 2)
            return 0
        if isinstance(m, int):
            return m
        if isinstance(m, tuple):
            text = w[m[1]]
            n = self.r16(text + 2)
            units = [self.r16(text + 8 + 2 * i) for i in range(n)]
            self.printed.append((self.frame, m[0], units))
            if len(m) > 2:
                return self.alloc(0x40) if m[2] == "new" else m[2]
            return 0
        return 0


# ---------------------------------------------------------------------- the session (session.rs)
def boot_system(e):
    e.call(0x0203EF5C, 0)
    e.call(0x02034E24, 0, 0)
    e.call(0x0203129C, 0)
    sys_ = e.cv(HEAP_ALLOC, 0, 0x28, 0, 0, 0)
    e.w32(GFL_SYS + 0x10, sys_)
    for n, mem, task in ((4, 0x1C, 0x18), (0x10, 0x24, 0x20)):
        size = e.cv(0x02030CB0, n)
        m = e.cv(HEAP_ALLOC, 0, size, 0, 0, 0)
        e.w32(sys_ + mem, m)
        e.w32(sys_ + task, e.cv(0x02030CBC, n, m))
    e.call(0x020056D0)
    e.call(0x0204E5E8, 0)
    e.call(0x0204E130, 0)
    e.call(0x0200566C)
    e.call(0x0204E6F4, 2)
    for o, v in ((0x10, 10), (0x14, 9), (0x18, 18), (0x1C, 6), (0x20, 12), (0x24, 0), (0x28, 0)):
        e.w32(RTC + o, v)


def boot(e):
    boot_system(e)
    e.load_overlay(10)
    z = e.cv(HEAP_ALLOC, 0, 4, 0, 0, 0)
    e.w32(BACKUP_STATE, z)
    e.call(0x0200523C)
    proc_ = e.cv(HEAP_ALLOC, 0, 0x40, 0, 0, 0)
    param = e.cv(HEAP_ALLOC, 0, 0x20, 0, 0, 0)
    e.w32(param, 1)
    e.w16(param + 0x10, 389)
    e.call(0x02011D9C, proc_, 0, param)
    return e.r32(GAMESYS)


def start_state(e, gs, zone, x, z):
    ev = e.r32(gs + 0x18)
    w = e.r32(ev + 0xC)
    e.w32(w + 0xC, 1)
    gd = e.cv(0x020120F4, gs)
    loc = e.cv(0x02012894, gd)
    e.call(0x02014184, loc, zone, 1, x << 16 | 0x8000, 0, z << 16 | 0x8000)
    ew = e.cv(0x0201296C, gd)
    v = e.cv(0x020143CC, ew, 0x4080)
    e.w16(v, 3)
    my = e.cv(0x02012944, gd)
    for i, c in enumerate([ord(ch) for ch in "Hilbert"] + [0xFFFF]):
        e.w16(my + 2 * i, c)
    mon = e.cv(0x020173F0, 495, 5, 0, 0, 0)
    party = e.cv(0x02012934, gd)
    e.call(0x0201A98C, party, mon)


def trace_line(e, gs, held):
    fm = e.r32(gs + 0x20)
    x = z = d = 0
    if fm and e.r32(fm + 0xC0):
        pl = e.r32(fm + 0x8C)
        if pl:
            core = e.r32(pl + 4)
            m = e.r32(core + 0x1C)
            if m:
                x, z, d = e.r16(m + 0x3C), e.r16(m + 0x40), e.r16(m + 0x18)
    p = e.cv(0x020120F8, gs)
    zone = e.r16(p)
    ev = e.r32(gs + 0x18)
    s = "f%d k%03x z%d p%d,%d,%d" % (e.frame, held, zone, x, z, d)
    if ev:
        proc_ = e.r32(ev + 4)
        s += " e%08x:%d" % (proc_, e.r32(ev + 8))
        if proc_ == SCRIPT_EVENT:
            w = e.r32(ev + 0xC)
            sys_ = e.r32(w + 4)
            for k in range(1, 4):
                vm = e.r32(sys_ + 4 * k)
                if vm:
                    s += " s%08x/%d/%08x" % (e.r32(vm + 0x14), e.r8(vm + 0xD), e.r32(vm + 0x10))
    if fm:
        msys = e.r32(fm + 0x38)
        if msys:
            n = e.r16(msys + 4)
            arr = e.r32(msys + 0x18)
            s += " m["
            for i in range(n):
                m = arr + 0x100 * i
                if e.r32(m) & 1:
                    s += "%d:%d,%d,%d " % (e.r16(m + 8), e.r16(m + 0x3C), e.r16(m + 0x40), e.r16(m + 0x18))
            s += "]"
    rs = e.r32(GFL_SYS + 0x10)
    s += " r%08x%08x" % (e.r32(rs + 4), e.r32(rs))
    return s


def step(e, gs, held):
    e.frame += 1
    e.w16(0x04000130, ~held & 0x3FF)
    e.w16(0x02FFFFA8, (~held & 0x0C00) | 0x2000)
    e.call(0x02034F5C)
    e.call(0x02011FCC, gs)
    e.call(0x0200567C)
    return trace_line(e, gs, held)


def text_of(units):
    """view.rs Printed::text (black_text::token): characters, \\n, {group:cmd args}."""
    out = []
    i = 0
    if units and units[0] == 0xF100:
        raise FieldError("compressed message (not decoded here)")
    while i < len(units):
        u = units[i]
        if u == 0xFFFF:
            break
        if u == 0xFFFE:
            out.append("\n")
            i += 1
        elif u == 0xF000:
            if i + 2 >= len(units) or i + 3 + units[i + 2] > len(units):
                out.append("{truncated %s}" % [hex(x) for x in units[i:]])
                break
            idn, n = units[i + 1], units[i + 2]
            args = units[i + 3:i + 3 + n]
            out.append("{%x:%x%s}" % (idn >> 8, idn & 0xFF, "".join(" %x" % a for a in args)))
            i += 3 + n
        else:
            out.append(chr(u))
            i += 1
    return "".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--keys", required=True, help="the Rust trace whose k column is replayed")
    ap.add_argument("--out", default=None)
    ap.add_argument("--text", default=None)
    ap.add_argument("--dump", default=None, help="FRAME,FILE: main RAM 0x02000000..0x02700000 after that frame "
                    "(diagnostics; the Rust side's F12_DUMP)")
    ap.add_argument("--frames", type=int, default=None, help="stop after this many frames")
    a = ap.parse_args()
    keys = []
    for line in open(a.keys):
        f = line.split()
        if len(f) > 1 and f[1].startswith("k"):
            keys.append(int(f[1][1:], 16))
    e = Field()
    gs = boot(e)
    start_state(e, gs, 389, 785, 750)
    lines = []
    out = open(a.out, "w") if a.out else None
    try:
        for held in keys:
            if a.frames is not None and e.frame >= a.frames:
                break
            line = step(e, gs, held)
            if a.dump and int(a.dump.split(",")[0]) == e.frame:
                open(a.dump.split(",", 1)[1], "wb").write(bytes(e.uc.mem_read(0x02000000, 0x700000)))
            lines.append(line)
            if out:
                out.write(line + "\n")
                out.flush()
    except FieldError as err:
        print("ROM run failed at frame %d: %s" % (e.frame, err))
        sys.exit(1)
    texts = [text_of(u) for _, _, u in e.printed]
    if a.text:
        with open(a.text, "w") as f:
            for fr, win, units in e.printed:
                f.write("f%d %s:%s\n" % (fr, win, "".join(" %04x" % u for u in units)))
    print("trace %016x" % fnv64("\n".join(lines).encode()))
    print("text %016x" % fnv64("\n".join(texts).encode()))


if __name__ == "__main__":
    main()
