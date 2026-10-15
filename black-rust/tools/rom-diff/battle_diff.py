"""B19: a whole battle in the original code under Unicorn (ARM9 main + overlays 93-96 + 231), for
the differential against the B16 integration (`integration::b19` in crates/black-battle).

  battle_diff.py [BATTLE ...] [--out DIR] [--arm9-funcs]
  battle_diff.py --record crates/black-battle/src/integration/b19.rs [--out DIR]

BATTLE is a name from `BATTLES` (default: all; mirrored by `integration::b19::BATTLES`). For each
one the script writes `DIR/<name>.trace` (default DIR = $BLK_OUT/b19, else /tmp/blk/b19) in the
trace format the Rust side emits (`integration::b19::trace`, format in its module doc) and prints
`<name> <fnv64 of the trace>`. `--record` writes those hashes into the Rust RECORDED table, only
when every ROM trace equals the Rust trace next to it (`<name>.rust.trace`); use
tools/check-battle-diff.sh, which runs both sides. `--arm9-funcs` lists the ARM9-main functions
that ran as original code. `B19_WATCH=addr,..` logs the calls of those addresses (diagnostics).

The emulated machine (`Battle`):
- boot: main.sbin at 0x02004000 with its autoload blocks copied to ITCM / DTCM / MAINEX / VRAM (the
  table after them), overlays 93 / 94 / 95 / 96 / 231 at their link addresses (`overlay_base`), BSS
  zero; the gfl std block (`0203EF5C`: Mersenne Twister state, CRC table) and the system Mersenne
  Twister seeded (`0203F034`) as the integration seeds its port; the personal-data archive handle
  opened (`0201AD88`).
- host hooks (`Battle._install_shims`), each the counterpart of the integration's host model of
  the same address (crates/black-battle/src/integration/{arm9,view,ai}.rs): heap (create / alloc
  zeroed / free / destroy), GameProc work, the file system under the archive layer (open an ARC by
  id, FS_SeekFile / FS_ReadFile / FS_CloseFile over black-rom/files), the save-data block table,
  OS_GetTick, the RTC-based RNG seed, key input, the overlay loader, sound, the Pokedex / friendship
  updates (save / field data), the cartridge IR probe, the headless battle view and overlay 94 (the
  player's script answers the move menu).
- the NDS division / square-root coprocessor (0x04000280-0x040002BF) is emulated; any other I/O
  access is reported (and fails the run), as is any access to unmapped memory.
Everything else — the battle overlays, btl_setup.c, vm.c, poke_tool / pokeparty / waza / personal,
strbuf, the gfl archive code, the MI / FX helpers — runs as original machine code.
"""
import argparse
import math
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from emu import B, overlay_base  # noqa: E402
from unicorn import (  # noqa: E402
    Uc, UcError, UC_ARCH_ARM, UC_MODE_ARM, UC_HOOK_CODE, UC_HOOK_BLOCK, UC_HOOK_MEM_UNMAPPED,
    UC_HOOK_MEM_WRITE, UC_HOOK_MEM_READ, UC_HOOK_INTR,
)
from unicorn.arm_const import (  # noqa: E402
    UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3, UC_ARM_REG_R4, UC_ARM_REG_R5,
    UC_ARM_REG_R6, UC_ARM_REG_R7, UC_ARM_REG_R8, UC_ARM_REG_R9, UC_ARM_REG_R10, UC_ARM_REG_R11,
    UC_ARM_REG_R12, UC_ARM_REG_SP, UC_ARM_REG_LR, UC_ARM_REG_PC,
)

ROOT = os.path.dirname(os.path.dirname(HERE))
FILES = os.environ.get("BLACK_ROM_FILES", os.path.join(ROOT, "..", "black-rom", "files"))
M32 = 0xFFFFFFFF
REGS = [UC_ARM_REG_R0, UC_ARM_REG_R1, UC_ARM_REG_R2, UC_ARM_REG_R3]
ALL_REGS = REGS + [UC_ARM_REG_R4, UC_ARM_REG_R5, UC_ARM_REG_R6, UC_ARM_REG_R7, UC_ARM_REG_R8,
                   UC_ARM_REG_R9, UC_ARM_REG_R10, UC_ARM_REG_R11, UC_ARM_REG_R12]

MAIN_BASE = 0x02004000
MAIN_CODE_SIZE = 0xA5E80  # main.sbin: code + data, then the autoload blocks, then their table
RET = 0x02FFF000  # return address of every top-level call (emulation stops there)
STACK_TOP = 0x02F00000
HEAP_BASE = 0x02500000  # shim heap (above the overlays / BSS and MAINEX, below the stack)
HEAP_END = 0x02E00000
MT_PTR = 0x021469F8  # pointer to the system Mersenne Twister state (0x9C4 bytes)
MT_INIT_FN = 0x0203EF5C
MT_SEED_FN = 0x0203F034
MT_NEXT = 0x0203F0A8  # the system Mersenne Twister's next output
MT_SEED = 0x89ABCDEF  # integration `Env::mt` seed
RNG_SEED = 0x0123456789ABCDEF  # integration `Env::seed` (0203F258)
CALC_RNG = 0x021F6368  # battle_calc globals 0x021F6360 + 8: the battle RNG (MathRandContext32)
OVERLAYS = [93, 94, 95, 96, 231]

PROC_INIT, PROC_MAIN, PROC_EXIT = 0x021B6100, 0x021B6434, 0x021B6500
READ_CMD = 0x021CD5E0
ADAPTER_SEND = 0x021EEB00  # (channel, code, data, size): message 8 = a command frame
AI_MASK_FN = 0x021B9FBC

# BattleMon (crates/black-battle battle_mon.rs): HP, max HP, poke id; move slot k's id at
# BPP_MOVES + BPP_MOVE_SIZE * k + 6
BPP_HP, BPP_MAXHP, BPP_ID = 0x10, 0x0E, 0x19
BPP_MOVES, BPP_MOVE_SIZE = 0x104, 0xE

PARTY_SIZE = 0x534
# save-data blocks the battle reaches (`020319A4` shim; `integration::arm9::SAVE_BLOCK_SIZES`): 0x24 the
# Chatot voice (perapvoice.c), 0x34 the player's money (u32 at +0, `0200C274`; size: what the readers touch)
SAVE_BLOCK_SIZES = {0x24: 0x3EC, 0x34: 0x10}


def fnv64(data):
    h = 14695981039346656037
    for b in data:
        h ^= b
        h = (h * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return h


class Narc:
    """Nitro archive: member i = GMIF data [start_i, end_i) of the BTAF table."""

    def __init__(self, data):
        hsize = struct.unpack_from("<H", data, 0xC)[0]
        o, secs = hsize, {}
        while o + 8 <= len(data):
            magic, size = data[o:o + 4], struct.unpack_from("<I", data, o + 4)[0]
            secs[magic] = o
            o += size
        fat = secs.get(b"BTAF", secs.get(b"FATB"))
        img = secs.get(b"GMIF", secs.get(b"FIMG"))
        n = struct.unpack_from("<H", data, fat + 8)[0]
        self.files = []
        for i in range(n):
            s, e = struct.unpack_from("<II", data, fat + 12 + 8 * i)
            self.files.append(data[img + 8 + s:img + 8 + e])


_narcs = {}


def arc_member(arc, k):
    if arc not in _narcs:
        path = os.path.join(FILES, "a", str(arc // 100), str(arc // 10 % 10), str(arc % 10))
        _narcs[arc] = Narc(open(path, "rb").read())
    return _narcs[arc].files[k]


class BattleError(Exception):
    pass


class Battle:
    """One battle in the original code. `party0` / `party1`: lists of (species, level, pid);
    `script`: the move slot the player picks at each move-menu opening (the last one repeats;
    `Env::script`); `switch_to`: the party slot it picks at each party-list opening (the last one
    repeats; `Env::switch_script`); `ai_mask`: forces `021b9fbc` (the AI client's script mask), as
    `Env::ai_mask` does."""

    def __init__(self, party0, party1, script=(0,), switch_to=(1,), ai_mask=None, arm9_funcs=False):
        self.party0, self.party1, self.script, self.ai_mask = party0, party1, list(script), ai_mask
        self.switch_to = list(switch_to)
        self.switches = []
        uc = Uc(UC_ARCH_ARM, UC_MODE_ARM)
        self.uc = uc
        uc.mem_map(0x01FF8000, 0x8000)  # ITCM
        uc.mem_map(0x02000000, 0x01000000)  # main RAM (+ DTCM 0x02FE0000, stack, shim heap)
        uc.mem_map(0x06880000, 0x40000)  # LCDC VRAM: overlay 95 (0x06898020) + its BSS
        uc.mem_map(0x04000000, 0x1000)  # I/O (division / square-root coprocessor; others reported)
        main = open(f"{B}/main.sbin", "rb").read()
        uc.mem_write(MAIN_BASE, main[:MAIN_CODE_SIZE])
        # autoload table (16-byte entries: dest, size, sinit, bss) after the blocks
        o, tab = MAIN_CODE_SIZE, MAIN_CODE_SIZE + 0x900
        for i in range(4):
            dst, size, _sinit, bss = struct.unpack_from("<IIII", main, tab + 16 * i)
            uc.mem_write(dst, main[o:o + size])
            o += size
        assert o == tab, "autoload layout"
        for n in OVERLAYS:
            uc.mem_write(overlay_base(n), open(f"{B}/OVY_{n}.sbin", "rb").read())
        self.heap_next = HEAP_BASE
        self.blocks = {}
        self.io_unknown = {}
        self.files = {}
        self.save_blocks = {}
        self.ui_fight = None
        self.choices = []
        self.ui_log = []
        self.cmd_frames = []  # (main frame, bytes)
        self.frame = -1
        self.mt_calls = 0
        self.calls = {}
        self.arm9_blocks = set() if arm9_funcs else None
        self.tick = 0
        self.shim = {}
        self._install_shims()
        for a, fn in self.shim.items():
            uc.hook_add(UC_HOOK_CODE, self._stub_hook, fn, a, a)
        # the headless view / overlay-94 ranges: any entry there is a call into the view
        uc.hook_add(UC_HOOK_CODE, self._view_range_hook, None, 0x021E8F20, 0x021EAFFF)
        uc.hook_add(UC_HOOK_CODE, self._view_range_hook, None, 0x021F7000, 0x021F9FFF)
        uc.hook_add(UC_HOOK_CODE, self._trace_send, None, ADAPTER_SEND, ADAPTER_SEND)
        uc.hook_add(UC_HOOK_CODE, self._count_mt, None, MT_NEXT, MT_NEXT)
        uc.hook_add(UC_HOOK_MEM_UNMAPPED, self._unmapped)
        uc.hook_add(UC_HOOK_MEM_WRITE, self._io_write, None, 0x04000000, 0x04000FFF)
        uc.hook_add(UC_HOOK_MEM_READ, self._io_read, None, 0x04000000, 0x04000FFF)
        uc.hook_add(UC_HOOK_INTR, self._intr)
        if arm9_funcs:
            uc.hook_add(UC_HOOK_BLOCK, self._arm9_block, None, 0x01FF8000, 0x020A9E7F)
        for a in os.environ.get("B19_WATCH", "").split(","):
            if a:
                uc.hook_add(UC_HOOK_CODE, self._watch, None, int(a, 16), int(a, 16))
        self.err = None
        # boot: the gfl "std" block (`0203EF5C(heap)`: allocates 0xBCC bytes at *0x021469F8: the
        # Mersenne Twister state, mti = 0x271 (unseeded), mag01 = {0, 0x9908B0DF}, a CRC table),
        # then the Mersenne Twister seeded (`0203F034`) with the integration's seed
        self.call(MT_INIT_FN, 0, thumb=False)
        self.call(MT_SEED_FN, MT_SEED, thumb=False)
        # boot: the personal-data archive handle (*0x0214652C, `0201AD88(heap)`)
        self.call(0x0201AD88, 0)

    # ---------------------------------------------------------------- memory
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

    def rb(self, a, n):
        return bytes(self.uc.mem_read(a, n))

    def wb(self, a, b):
        self.uc.mem_write(a, bytes(b))

    def alloc(self, size):
        """A zeroed block (like the integration's `Objects::alloc`: 4-aligned, 16-byte gaps)."""
        base = self.heap_next
        self.heap_next = base + ((max(size, 1) + 3) & ~3) + 0x10
        if self.heap_next >= HEAP_END:
            raise BattleError("shim heap exhausted")
        self.wb(base, bytes(size))
        self.blocks[base] = size
        return base

    def free(self, p):
        if p not in self.blocks:
            raise BattleError("free of unknown block %08x" % p)
        del self.blocks[p]

    # ---------------------------------------------------------------- calls
    def call(self, addr, *args, thumb=True):
        """A top-level call (r0-r3 = args, lr = RET)."""
        uc = self.uc
        for r in ALL_REGS:
            uc.reg_write(r, 0)
        for r, a in zip(REGS, args):
            uc.reg_write(r, a & M32)
        uc.reg_write(UC_ARM_REG_SP, STACK_TOP)
        uc.reg_write(UC_ARM_REG_LR, RET)
        try:
            uc.emu_start(addr | (1 if thumb else 0), RET, count=200_000_000)
        except UcError as e:
            raise BattleError("call %08x: %s at pc %08x (lr %08x)%s; stack %s" % (
                addr, e, uc.reg_read(UC_ARM_REG_PC), uc.reg_read(UC_ARM_REG_LR),
                "" if self.err is None else ": " + self.err, self.stack_scan()))
        if self.err is not None:
            raise BattleError("call %08x: %s; stack %s" % (addr, self.err, self.stack_scan()))
        if uc.reg_read(UC_ARM_REG_PC) & ~1 != RET:
            raise BattleError("call %08x did not return (pc %08x)" % (addr, uc.reg_read(UC_ARM_REG_PC)))
        return uc.reg_read(UC_ARM_REG_R0)

    def stack_scan(self):
        """Words on the stack that look like return addresses (diagnostics only)."""
        sp = self.uc.reg_read(UC_ARM_REG_SP)
        out = []
        for a in range(sp, min(sp + 0x800, STACK_TOP), 4):
            v = self.r32(a)
            if (0x02004000 <= v < 0x020A9E80 or 0x02180000 <= v < 0x0220B000 or 0x06898000 <= v < 0x068A0000) \
                    and (v & 1 or v & 3 == 0):
                out.append("%08x" % v)
        return " ".join(out[:24])

    def _ret(self, uc, r):
        if isinstance(r, tuple):
            uc.reg_write(UC_ARM_REG_R0, r[0] & M32)
            uc.reg_write(UC_ARM_REG_R1, r[1] & M32)
        elif r is not None:
            uc.reg_write(UC_ARM_REG_R0, r & M32)
        uc.reg_write(UC_ARM_REG_PC, uc.reg_read(UC_ARM_REG_LR))

    def _fail(self, uc, msg):
        if self.err is None:
            self.err = msg
        uc.emu_stop()

    def _stub_hook(self, uc, addr, size, fn):
        args = [uc.reg_read(r) for r in REGS]
        self.calls[addr] = self.calls.get(addr, 0) + 1
        try:
            r = fn(*args)
        except BattleError as e:
            self._fail(uc, "%08x: %s" % (addr, e))
            return
        self._ret(uc, r)

    def _view_range_hook(self, uc, addr, size, _):
        if addr in self.shim:
            return  # its own hook answers
        if 0x021E8F20 <= addr <= 0x021EAFFF:
            # any other battle-view entry: an effect / window / message starts, or a "finished?"
            # query: headless, everything is done at once (view.rs)
            self.ui_log.append(addr)
            r = 1
        else:
            r = 0  # overlay 94 sound / effects: nothing plays
        self.calls[addr] = self.calls.get(addr, 0) + 1
        self._ret(uc, r)

    def _trace_send(self, uc, addr, size, _):
        _ch, code, data, n = [uc.reg_read(r) for r in REGS]
        if code == 8:
            b = self.rb(data, n) if n else b""
            if not self.cmd_frames or self.cmd_frames[-1][1] != b:
                self.cmd_frames.append((self.frame, b))

    def _count_mt(self, uc, addr, size, _):
        self.mt_calls += 1

    def _unmapped(self, uc, access, addr, size, value, _):
        self.err = "unmapped access %08x+%d (pc %08x, lr %08x)" % (
            addr, size, uc.reg_read(UC_ARM_REG_PC), uc.reg_read(UC_ARM_REG_LR))
        return False

    def _intr(self, uc, intno, _):
        self._fail(uc, "interrupt %d at pc %08x" % (intno, uc.reg_read(UC_ARM_REG_PC)))

    def _watch(self, uc, addr, size, _):
        """`B19_WATCH=addr,..`: logs each call (frame, args, return address) to stderr."""
        a = [uc.reg_read(r) for r in REGS]
        sys.stderr.write("watch f%d %08x(%s) lr %08x stack %s\n" % (
            self.frame, addr, ",".join("%x" % v for v in a), uc.reg_read(UC_ARM_REG_LR), self.stack_scan()))

    def _arm9_block(self, uc, addr, size, _):
        self.arm9_blocks.add(addr)

    # the NDS math coprocessor: results are computed when an operand / control register is written
    def _io_write(self, uc, access, addr, size, value, _):
        if 0x04000280 <= addr < 0x040002C0:
            # apply the write first, then recompute
            uc.mem_write(addr, (value & ((1 << (8 * size)) - 1)).to_bytes(size, "little"))
            self._math()
            return
        if addr in (0x04000208, 0x04000210, 0x04000214):  # IME / IE / IF: no interrupts here
            return
        self.io_unknown[addr] = self.io_unknown.get(addr, 0) + 1

    def _io_read(self, uc, access, addr, size, value, _):
        if 0x04000280 <= addr < 0x040002C0 or addr in (0x04000208, 0x04000210, 0x04000214):
            return
        self.io_unknown[addr] = self.io_unknown.get(addr, 0) + 1

    def _math(self):
        """DIVCNT / DIV_NUMER / DIV_DENOM -> DIV_RESULT / DIVREM_RESULT, SQRTCNT / SQRT_PARAM -> SQRT_RESULT (both always
        ready), computed on a write. Z25: the one shared model, emu.py `div_unit` / `sqrt_unit` (GBATEK edge cases: division
        by zero, MIN / -1, the inverted upper word in 32/32 mode)."""
        from emu import div_unit, sqrt_unit
        cnt = self.r16(0x04000280)
        num, den = struct.unpack("<QQ", bytes(self.uc.mem_read(0x04000290, 16)))
        q, rem = div_unit(cnt, num, den)
        self.uc.mem_write(0x040002A0, struct.pack("<QQ", q, rem))
        self.w16(0x04000280, cnt & ~0x8000)
        scnt = self.r16(0x040002B0)
        p = struct.unpack("<Q", bytes(self.uc.mem_read(0x040002B8, 8)))[0]
        self.w32(0x040002B4, sqrt_unit(scnt, p))
        self.w16(0x040002B0, scnt & ~0x8000)

    # ---------------------------------------------------------------- shims
    def _install_shims(self):
        s = self.shim

        def const(v):
            return lambda *a: v
        # ---- heap / GameProc (gfl heap.c: the battle's heaps are one shim heap) ----
        s[0x0203064C] = const(1)  # Heap_Create(parent, id, size)
        s[0x02030734] = lambda heap, size, *a: self.alloc(size)  # Heap_AllocDebug(heap, size, ..)
        s[0x020307B0] = lambda p, *a: (self.free(p) if p else None) or 0  # Heap_Free
        s[0x020306F0] = const(0)  # Heap_Destroy

        def proc_alloc(proc, size, *a):
            w = self.alloc(size)
            self.w32(proc + 0x14, w)
            return w

        def proc_free(proc, *a):
            w = self.r32(proc + 0x14)
            if w in self.blocks:
                self.free(w)
            self.w32(proc + 0x14, 0)
            return 0
        s[0x0203159C] = proc_alloc  # GameProc_AllocWork(proc, size, heap)
        s[0x020315D4] = proc_free  # GameProc_FreeWork(proc)
        # ---- system ----
        s[0x020362DC] = const(1)  # keys pressed this frame: the scripted player presses A
        s[0x02035C60] = const(0)  # touch: none
        s[0x02034AC4] = const(1)  # overlay load (everything is loaded)
        s[0x02034A5C] = const(1)  # overlay unload
        s[0x020870D0] = self._get_tick  # OS_GetTick: the integration's deterministic clock
        s[0x0203F258] = self._rtc_seed  # MathRand seed from the RTC: the integration's fixed seed
        # main-module sound layer (fades / BGM / SE / the VBlank sound callback)
        for a in (0x02005CA4, 0x02005D28, 0x02005D90, 0x02005DF8, 0x02005E30, 0x02006734, 0x02006740,
                  0x02005E50, 0x020061E4, 0x020057AC):
            s[a] = const(0)
        s[0x020057C4] = const(1)
        # the save data's block table (`020319A4(mgr, id)`, ARM; `020071CC(save, id)` reaches it
        # with mgr = save[0x10]): one zeroed block per id (the setup's save data is empty)
        s[0x020319A4] = self._save_block
        # 0200CFA0(dex, mon): Pokedex "seen" registration in the save data (the setup has no save
        # data; the integration does not model it either)
        s[0x0200CFA0] = const(0)
        # 0201B7A0(pp, event, zone, heap): friendship change when the player's Pokemon faints; it
        # reads the field's zone table (02013E4C), which a battle without the field does not have
        s[0x0201B7A0] = const(0)
        # ---- file system: the archive layer (gfl arc) runs as original code on top of these ----
        s[0x02048DDC] = self._fs_open_arc  # (FSFile, arc id): FS_InitFile + open through the arc file-id table
        s[0x02078B44] = self._fs_seek  # FS_SeekFile(file, offset, whence)
        s[0x02078B70] = self._fs_read  # FS_ReadFile(file, dst, len)
        s[0x02078A80] = self._fs_close  # FS_CloseFile(file)
        # ---- overlay 231: the cartridge IR probe (genuine hardware) ----
        s[0x021809D8] = const(1)
        # ---- headless battle view (view.rs) ----
        s[0x021E8DC4] = const(0)
        s[0x021E8DF8] = const(0)
        s[0x021E88CC] = const(0)
        s[0x021E88F8] = const(0)
        s[0x021E8F20] = lambda *a: self.alloc(0x40)
        s[0x021E9034] = lambda v, *a: self.free(v) or 0
        s[0x021E908C] = const(0)
        for a in (0x021E90A0, 0x021E979C, 0x021E9760, 0x021E9808, 0x021E9858, 0x021E98AC, 0x021E9920,
                  0x021E9954, 0x021E9964, 0x021E9B14, 0x021E9E70, 0x021EA058, 0x021EA614, 0x021EA65C,
                  0x021EA7FC, 0x021EAA5C, 0x021EAE24, 0x021EAE44, 0x021EAEA8, 0x021EA824):
            s[a] = self._ui_req(a)
        for a in (0x021EAF18, 0x021EAF50, 0x021EAF78):
            s[a] = const(0)
        for a in (0x021E90D0, 0x021E9974, 0x021E9B20, 0x021E9E80, 0x021EA710, 0x021EA720, 0x021EA814,
                  0x021EAE34, 0x021EAE54):
            s[a] = const(1)
        s[0x021E97A4] = self._action_menu
        s[0x021E97BC] = self._move_menu_open
        s[0x021E98B4] = self._move_menu_answer
        s[0x021E98E8] = const(1)
        s[0x021E9A78] = self._party_list
        s[0x021EAC54] = self._yes_no
        for a in (0x021F7998, 0x021F84E8, 0x021F8504, 0x021F87EC, 0x021F88BC, 0x021F8B10, 0x021F7EB4,
                  0x021F8774, 0x021F8678):
            s[a] = const(0)
        s[0x021F8524] = const(1)
        if self.ai_mask is not None:
            s[AI_MASK_FN] = self._ai_mask

    def _ai_mask(self, m, cid, *a):
        return self.ai_mask

    def _get_tick(self, *a):
        """OS_GetTick (u64): 0x1000 more per call (`integration::ai`, `Env::tick`); the AI's time
        budget never runs out."""
        self.tick += 0x1000
        return (self.tick & M32, self.tick >> 32)

    def _rtc_seed(self, ctx, *a):
        self.wb(ctx, struct.pack("<QQQ", RNG_SEED, 0x5D588B656C078965, 0x00269EC3))
        return 0

    def _save_block(self, mgr, bid, *a):
        k = (mgr, bid)
        if k not in self.save_blocks:
            sizes = SAVE_BLOCK_SIZES
            if bid not in sizes:
                raise BattleError("save block %#x not modelled" % bid)
            self.save_blocks[k] = self.alloc(sizes[bid])
        return self.save_blocks[k]

    # FSFile (NitroSDK, 0x48 bytes): prop.file.top / bottom / pos at +0x24 / +0x28 / +0x2C. The
    # file's bytes are host-side (`self.files`); top = 0, so positions are file offsets.
    def _fs_open_arc(self, f, arc, *a):
        path = os.path.join(FILES, "a", str(arc // 100), str(arc // 10 % 10), str(arc % 10))
        data = open(path, "rb").read()
        self.wb(f, bytes(0x48))
        self.files[f] = (arc, data)
        self.w32(f + 0x24, 0)
        self.w32(f + 0x28, len(data))
        self.w32(f + 0x2C, 0)
        return 1

    def _fs_file(self, f):
        if f not in self.files:
            raise BattleError("FS call on a file that is not open (%08x)" % f)
        return self.files[f][1]

    def _fs_seek(self, f, off, whence, *a):
        self._fs_file(f)
        off = off - (1 << 32) if off >> 31 else off
        top, bottom, pos = self.r32(f + 0x24), self.r32(f + 0x28), self.r32(f + 0x2C)
        base = {0: top, 1: pos, 2: bottom}.get(whence)
        if base is None:
            raise BattleError("FS_SeekFile whence %d" % whence)
        p = base + off
        if p < top or p > bottom:
            return 0
        self.w32(f + 0x2C, p)
        return 1

    def _fs_read(self, f, dst, n, *a):
        data = self._fs_file(f)
        pos, bottom = self.r32(f + 0x2C), self.r32(f + 0x28)
        n = min(n, bottom - pos)
        self.wb(dst, data[pos:pos + n])
        self.w32(f + 0x2C, pos + n)
        return n

    def _fs_close(self, f, *a):
        self._fs_file(f)
        del self.files[f]
        return 1

    def _ui_req(self, a):
        def f(*args):
            self.ui_log.append(a)
            return 0
        return f

    def _action_menu(self, *a):
        self.ui_log.append(0x021E97A4)
        return 1  # Fight

    def _move_menu_open(self, view, mon, rec, *a):
        self.ui_fight = (rec, mon)
        return 0

    def _move_menu_answer(self, *a):
        if self.ui_fight is None:
            raise BattleError("move menu was not opened")
        rec, mon = self.ui_fight
        self.ui_fight = None
        k = min(len(self.choices), len(self.script) - 1)
        slot = self.script[k]
        mv = self.r16(mon + BPP_MOVES + BPP_MOVE_SIZE * slot + 6)
        self.w32(rec, 1 | (1 << 4) | ((mv & 0xFFFF) << 7))
        self.choices.append(mv)
        return 1

    def _yes_no(self, view, out, *a):
        """021eac54(view, &answer): the yes / no window has an answer: 0 (the first choice: "yes",
        e.g. send out the next Pokemon)."""
        self.ui_log.append(0x021EAC54)
        self.w32(out, 0)
        return 1

    def _party_list(self, view, block, pos, flag):
        """021e9a78(view, block, pos, flag, sel): the party list; the scripted player picks
        `switch_to[k]` for `pos` as the list's "add" does (overlay 95 `0689CAEC(sel, pos, slot)`)."""
        sel = self.r32(self.uc.reg_read(UC_ARM_REG_SP))  # the fifth argument
        slot = self.switch_to[min(len(self.switches), len(self.switch_to) - 1)]
        self.switches.append(slot)
        self.ui_log.append(0x021E9A78)
        n, mx = self.r8(sel + 6), self.r8(sel + 7)
        if n < mx:
            self.w8(sel + n, slot)
            self.w8(sel + 3 + n, pos)
            self.w8(sel + 6, n + 1)
        return 0

    # ---------------------------------------------------------------- setup
    def make_pokemon(self, species, level, pid):
        """`integration::tests::make_pokemon` in the original code: 02017528 (clear), 02017E40
        (set), 02018A7C (recalc stats); data from 02019C90 (personal), 02019BA4 (EXP table),
        0201C158 (max PP) and the learnset archive (a/0/1/8)."""
        m = self.alloc(0xDC)
        self.call(0x02017528, m)

        def put(i, v):
            self.call(0x02017E40, m, i, v)
        put(0, pid)
        put(5, species)
        put(7, 0x00012345)
        curve = self.call(0x02019C90, species, 0, 0x17)
        put(8, self.call(0x02019BA4, curve, level))
        put(9, self.call(0x02019C90, species, 0, 0x16))
        put(0xA, self.call(0x02019C90, species, 0, 0x18))
        put(0xC, 2)
        for i in range(0x46, 0x4C):
            put(i, 15)
        put(0x70, pid % 25)
        ls = arc_member(18, species)
        moves = []
        for o in range(0, len(ls) - 3, 4):
            mv, lv = struct.unpack_from("<HH", ls, o)
            if mv == 0xFFFF:
                break
            if lv <= level:
                moves.append(mv)
        moves = moves[-4:]
        for i, mv in enumerate(moves):
            put(0x36 + i, mv)
            put(0x3A + i, self.call(0x0201C158, mv, 0))
        put(0x9E, level)
        self.call(0x02018A7C, m)
        assert self.call(0x02017E1C, m, 5, 0) == species
        return m

    def make_party(self, mons):
        p = self.alloc(PARTY_SIZE)
        self.call(0x0201A954, p, 6)
        for spec in mons:
            m = self.make_pokemon(*spec)
            self.call(0x0201A98C, p, m)
            self.free(m)
        return p

    def build_setup(self, player, foe):
        """`integration::tests::build_setup`: a local wild single battle (type 0, rule 0)."""
        setup = self.alloc(0xF0)
        config = self.alloc(4)
        self.w32(setup + 0x5C, config)
        self.w32(setup + 0x24, player)
        self.w32(setup + 0x28, foe)
        ms = self.alloc(0x20)
        self.w32(setup + 0x34, ms)
        for i, c in enumerate((0x0130, 0x0131, 0x0132, 0x0133, 0xFFFF)):
            self.w16(ms + 2 * i, c)
        self.w32(ms + 0x10, 0x00012345)
        records = self.alloc(0x400)
        self.w32(setup + 0x6C, records)
        save = self.alloc(0x20)
        game = self.alloc(4)
        self.w32(game, save)
        self.w32(setup + 0x58, game)
        return setup

    # ---------------------------------------------------------------- the battle
    def hp_line(self, m):
        pc = m + 0xC8
        ids = []
        for i in range(24):
            b = self.r32(pc + 0x84 + 4 * i)
            if b:
                ids.append("%d:%d/%d" % (self.r8(b + BPP_ID), self.r16(b + BPP_HP), self.r16(b + BPP_MAXHP)))
        x = struct.unpack("<Q", self.rb(CALC_RNG, 8))[0]
        return "hp %s rng %016x" % (" ".join(ids), x)

    def run(self):
        player = self.make_party(self.party0)
        foe = self.make_party(self.party1)
        party_hash = fnv64(self.rb(player, PARTY_SIZE) + self.rb(foe, PARTY_SIZE))
        setup = self.build_setup(player, foe)
        proc, seq = self.alloc(0x20), self.alloc(4)
        self.out = out = ["parties %016x" % party_hash]
        n = 0
        while True:
            m = self.r32(proc + 0x14)
            if self.call(PROC_INIT, proc, seq, setup, m):
                break
            n += 1
            if n >= 1000:
                raise BattleError("proc_init does not finish")
        out.append("init_frames %d" % n)
        m = self.r32(proc + 0x14)
        self.frame = 0
        while True:
            done = self.call(PROC_MAIN, proc, seq, setup, m)
            out.append("f %d %s" % (self.frame, self.hp_line(m)))
            if done:
                break
            self.frame += 1
            if self.frame >= 2000:
                raise BattleError("the battle does not end")
        out.append("main_frames %d" % self.frame)
        hp_end = self.hp_line(m)
        n = 0
        self.frame = -2
        while True:
            if self.call(PROC_EXIT, proc, seq, setup, m):
                break
            n += 1
            if n >= 100:
                raise BattleError("proc_exit does not finish")
        out.append("exit_frames %d" % n)
        out.append("end %s" % hp_end)
        out.append("choices %s" % " ".join("%d" % c for c in self.choices))
        out.append("switches %s" % " ".join("%d" % c for c in self.switches))
        for k, (f, b) in enumerate(self.cmd_frames):
            out.append("frame %d at %d len %d %s" % (k, f, len(b), b.hex()))
            for cid, args in self.decode(b):
                out.append("  cmd %02x %s" % (cid, " ".join("%x" % a for a in args)))
        out.append("mt_calls %d" % self.mt_calls)
        return out

    def decode(self, data):
        """`integration::tests::decode_frame` with the original reader 021cd5e0."""
        size = len(data)
        q = self.alloc(8 + size + 0x40)
        o = self.alloc(0x40)
        self.w32(q, size)
        self.wb(q + 8, data)
        cmds = []
        while True:
            cur = self.r32(q + 4)
            if cur == size:
                break
            if cur > size:
                raise BattleError("frame: read past the end")
            self.wb(o, bytes(0x40))
            cid = self.call(READ_CMD, q, o)
            if cid == 0x5E:
                break
            args = list(struct.unpack("<16I", self.rb(o, 0x40)))
            if cid == 0x33:
                self.w32(q + 4, self.r32(q + 4) + (args[0] & 0xFFFF))
            cmds.append((cid, args))
        return cmds


BATTLES = {
    # B16: Snivy lv5 (player, Fight slot 0 every turn) vs a wild Patrat lv5
    "b16": dict(party0=[(495, 5, 0x12345678)], party1=[(504, 5, 0x87654321)]),
    # the same with trainer-AI scripts (mask 7: the basic / strong / expert scripts, B17)
    "trainer_ai": dict(party0=[(495, 5, 0x12345678)], party1=[(504, 5, 0x87654321)], ai_mask=0x7),
    # status moves: Snivy uses Leer (slot 1) twice, then Tackle
    "status": dict(party0=[(495, 5, 0x12345678)], party1=[(504, 5, 0x87654321)], script=(1, 1, 0)),
    # a switch: Snivy (Tackle) faints in turn 3, the player sends out Tepig lv5 (party slot 1)
    "switch": dict(party0=[(495, 5, 0x12345678), (498, 5, 0x0BADBEEF)], party1=[(504, 5, 0x87654321)],
                   switch_to=(1,)),
}


def arm9_functions(blocks):
    """The ARM9-main functions (ELF symbols of main.elf) containing the executed block starts."""
    import bisect
    sys.path.insert(0, os.path.join(ROOT, "tools", "re"))
    import mkelf
    d, _, secs = mkelf.read_elf(f"{B}/main.elf")
    st = next(x for x in secs if x["name"] == ".symtab")
    strt = secs[st["link"]]
    syms = {}
    for k in range(st["size"] // 16):
        nm, val, _size, info, _o, _shn = struct.unpack_from("<IIIBBH", d, st["off"] + 16 * k)
        o = strt["off"] + nm
        name = d[o:d.index(b"\0", o)].decode(errors="replace")
        # function labels (`FUN_...` / named ones); not the `$a/$t/$d` mapping symbols, the `_ADDR`
        # local labels or the per-file section symbols (`unk_...`)
        if 0x01FF8000 <= val < 0x020AA000 and name[:1] not in ("$", "_") and not name.startswith("unk_"):
            syms.setdefault(val & ~1, name)
    addrs = sorted(syms)
    out = {}
    for b in blocks:
        k = bisect.bisect_right(addrs, b) - 1
        if k >= 0:
            out[addrs[k]] = syms[addrs[k]]
    return out


def record(rs, out_dir):
    """Writes the RECORDED table of `rs` (integration/b19.rs) from the ROM traces in `out_dir`,
    only when every battle's ROM trace equals the Rust trace next to it (`NAME.rust.trace`)."""
    rows = []
    for name in BATTLES:
        rom = open(os.path.join(out_dir, name + ".trace"), "rb").read()
        try:
            rust = open(os.path.join(out_dir, name + ".rust.trace"), "rb").read()
        except OSError:
            rust = None
        if rom != rust:
            print("%s: the Rust trace differs from (or is missing next to) the ROM trace: not recorded" % name)
            return 1
        rows.append('    ("%s", "%016x"),' % (name, fnv64(rom)))
    src = open(rs).read()
    a = src.index("// b19-recorded-begin\n") + len("// b19-recorded-begin\n")
    b = src.index("// b19-recorded-end")
    table = "pub const RECORDED: &[(&str, &str)] = &[\n" + "\n".join(rows) + "\n];\n"
    open(rs, "w").write(src[:a] + table + src[b:])
    print("recorded %d battles in %s" % (len(rows), rs))
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("battles", nargs="*")
    ap.add_argument("--out", default=os.path.join(os.environ.get("BLK_OUT", "/tmp/blk"), "b19"))
    ap.add_argument("--arm9-funcs", action="store_true", help="list the ARM9-main functions that ran as original code")
    ap.add_argument("--record", metavar="RS", help="write the RECORDED table of RS from the traces in --out (no run)")
    a = ap.parse_args()
    if a.record:
        return record(a.record, a.out)
    os.makedirs(a.out, exist_ok=True)
    rc = 0
    for name in a.battles or list(BATTLES):
        if name not in BATTLES:
            print("unknown battle %s (known: %s)" % (name, " ".join(BATTLES)))
            return 2
        bt = Battle(arm9_funcs=a.arm9_funcs, **BATTLES[name])
        try:
            lines = bt.run()
        except BattleError as e:
            print("%s ERROR %s" % (name, e))
            # the trace so far (diagnostics; the Rust side never matches an `error` line)
            part = getattr(bt, "out", []) + ["error at main frame %d: %s" % (bt.frame, e)]
            for k, (f, b) in enumerate(bt.cmd_frames):
                part.append("frame %d at %d len %d %s" % (k, f, len(b), b.hex()))
            open(os.path.join(a.out, name + ".trace"), "w").write("\n".join(["b19 %s" % name] + part) + "\n")
            rc = 1
            continue
        text = "\n".join(["b19 %s" % name] + lines) + "\n"
        open(os.path.join(a.out, name + ".trace"), "w").write(text)
        print("%s %016x" % (name, fnv64(text.encode())))
        if bt.io_unknown:
            print("%s: other I/O accesses: %s" % (name, " ".join("%08x x%d" % kv for kv in sorted(bt.io_unknown.items()))))
            rc = 1
        if a.arm9_funcs:
            # a host hook's block is entered (the hook then returns at once): not original code
            fs = arm9_functions(bt.arm9_blocks - set(bt.shim))
            print("%s: %d ARM9-main functions ran as original code:" % (name, len(fs)))
            print(" ".join("%08x" % f for f in sorted(fs)))
    return rc


if __name__ == "__main__":
    sys.exit(main())
