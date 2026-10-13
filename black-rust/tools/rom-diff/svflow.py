"""Per-function differential driver (B10 server flow, Z16 any crate). Mirrors
crates/black-fndiff (and the server_flow testkit on top of it); the spec grammar is documented in
tools/rom-diff/svflow_specs/README.md.

  svflow.py AREA                  area hash over every function in PORTED (server_flow/AREA.rs)
  svflow.py AREA --json [--jobs N]   {"module", "hash", "funcs": {addr: {"hash", "n_inputs", "seeds"}}}
  svflow.py AREA --detail ADDR:SEED  one input's section-labelled word stream (JSON)
  svflow.py AREA --all            run every function in the spec, not only the PORTED ones
  svflow.py AREA --fn ADDR        restrict to one function (hex)
  svflow.py AREA --order-check    (Z19) exit 1 if an input's hash changes when the inputs run in reverse order
                                  after junk in r0-r12 / flags / the stack window (combine with --all / --fn)
  svflow.py --selftest            (Z19) reset probes (r3, r4, r12, stack, flags) + order check of server1 021b6848

Generic mode (any crate; tools/check-fn.sh passes these):
  --spec FILE         spec file (default svflow_specs/AREA.spec)
  --ported-src FILE   Rust file whose `pub const PORTED` lists the area's functions (repeatable);
                      without it the area is server_flow/AREA.rs
  --strict-callees    exit 4 if a spec'd function called an address that is not a stub, not PORTED and not the
                      function itself (default: warn on stderr; SVFLOW_ALLOW=hex,hex.. adds addresses)
  --set NAME          memory image: a tools/re/sets.txt set (battle | field | ...); default is the
                      spec's `set` line, else battle. battle = ARM9 main + overlay 93 (as before).

Input generation, the stub model and the hash are specified once (below) and re-implemented in
Rust; any change here must be mirrored in crates/black-fndiff.

Entry state (Z19): every input starts from the same CPU state (Emu.call in emu.py): r0-r12 = 0, then
the declared arguments in r0-r3 and on the stack from sp = 0x02F00000; flags clear; the stack window
0x02EF0000..0x02F01000 zero-filled first. Undeclared registers / stack words read 0 (black_fndiff
`Case::word`). Memory outside the input's regions is restored byte-for-byte after each input.
"""
import json
import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from emu import B, RET, STACK_ZERO_HI, STACK_ZERO_LO, Emu, Rng  # noqa: E402
from bpp import Env, mix, random_bpp, SIZE as BPP_SIZE  # noqa: E402

M = 0xFFFFFFFF
M64 = 0xFFFFFFFFFFFFFFFF
FNV_OFFSET = 14695981039346656037
FNV_PRIME = 1099511628211
WORK = 0x02E00000
WORK_SIZE = 0x212C
BUF_BASE = 0x02E10000  # pointer argument i lives at BUF_BASE + i * BUF_STRIDE
BUF_STRIDE = 0x10000
MAX_PTR_ARGS = 13
REGION_BASE = 0x02F10000  # `region` k lives at REGION_BASE + k * BUF_STRIDE
MAX_REGIONS = 14
SETS_TXT = os.path.join(os.path.dirname(os.path.dirname(HERE)), "tools", "re", "sets.txt")
# Z19: Emu.call zero-fills this stack window and zeroes r0-r12 / the flags before every call (see emu.py),
# so an argument register or stack word the spec does not pass reads 0 on every input, whatever ran before
STACK_LO = STACK_ZERO_LO
STACK_HI = STACK_ZERO_HI
ROOT = os.path.dirname(os.path.dirname(HERE))
SPECS = os.path.join(HERE, "svflow_specs")
FLOW_SRC = os.path.join(ROOT, "crates", "black-battle", "src", "server_flow")
CLIENT_SRC = os.path.join(ROOT, "crates", "black-battle", "src", "client")  # B13-spec1: areas client1.. (btl_client.c), `nowork` svflow areas
SERVER_SRC = os.path.join(ROOT, "crates", "black-battle", "src", "server")  # B11-spec: areas server1/server2 (btl_server.c), `nowork` svflow areas
DEFAULT_INPUTS = 200


# ------------------------------------------------------------------ spec


class Gen:
    """Scalar generator: kind in any/range/oneof/const/mask/near/addrof."""

    def __init__(self, kind, params):
        self.kind, self.p = kind, params

    def draw(self, r, addr_of=None, tgt=None, vals=None):
        k, p = self.kind, self.p
        if k == "sum16":  # Z16/P1: u16 halfword sum of the fix target's bytes [OFS, OFS+SIZE); no draw
            return sum(int.from_bytes(tgt[p[0] + 2 * i:p[0] + 2 * i + 2], "little") for i in range(p[1] // 2)) & 0xFFFF
        if k == "any":
            s = r.next() % 8
            if s == 0:
                return 0
            if s == 1:
                return 1
            if s == 2:
                return 0xFF
            if s == 3:
                return r.next() % 0x100
            if s == 4:
                return r.next() % 0x10000
            return r.next()
        if k == "range":
            return (p[0] + r.next() % (p[1] - p[0] + 1)) & M
        if k == "near":
            return (p[0] - p[1] + r.next() % (2 * p[1] + 1)) & M
        if k == "oneof":
            return p[r.next() % len(p)]
        if k == "const":
            return p[0] & M
        if k == "mask":
            return r.next() & p[0]
        if k == "addrof":
            return addr_of(p[0]) + p[1]
        if k == "alt":
            return p[r.next() % len(p)].draw(r, addr_of, tgt, vals)
        if k == "rel":  # value of the earlier argument K plus a draw (wrapping)
            return (vals[p[0]] + p[1].draw(r, addr_of, tgt, vals)) & M
        if k == "xrel":  # value of the earlier argument K xor a draw
            return vals[p[0]] ^ p[1].draw(r, addr_of, tgt, vals)
        if k == "relm":  # (earlier argument K & MASK) plus a draw (wrapping)
            return ((vals[p[0]] & p[1]) + p[2].draw(r, addr_of, tgt, vals)) & M
        if k == "ifeq":  # P-spec2: earlier argument K == V ? draw THEN : draw ELSE (only the chosen one draws)
            return (p[2] if vals[p[0]] == p[1] else p[3]).draw(r, addr_of, tgt, vals)
        raise ValueError(k)


TYPES = {"u8": (0xFF, False), "u16": (0xFFFF, False), "u32": (M, False),
         "s8": (0xFF, True), "s16": (0xFFFF, True), "s32": (M, True)}


def conv(v, ty):
    mask, signed = TYPES[ty]
    v &= mask
    if signed and v & ((mask + 1) >> 1):
        v |= M ^ mask
    return v & M


def parse_gen(tok, line):
    kind = tok[0]
    if kind == "alt":
        alts, cur = [], []
        for x in tok[1:] + ["||"]:
            if x == "||":
                alts.append(parse_gen(cur, line))
                cur = []
            else:
                cur.append(x)
        return Gen("alt", alts)
    if kind == "relm":
        return Gen(kind, [int(tok[1], 0), int(tok[2], 0), parse_gen(tok[3:], line)])
    if kind == "ifeq":  # ifeq K V THEN-GEN || ELSE-GEN
        i = tok.index("||")
        return Gen(kind, [int(tok[1], 0), int(tok[2], 0), parse_gen(tok[3:i], line), parse_gen(tok[i + 1:], line)])
    if kind in ("rel", "xrel"):
        return Gen(kind, [int(tok[1], 0), parse_gen(tok[2:], line)])
    if kind == "any":
        return Gen("any", [])
    if kind in ("range", "near"):
        return Gen(kind, [int(tok[1], 0), int(tok[2], 0)])
    if kind == "oneof":
        return Gen("oneof", [int(x, 0) for x in tok[1:]])
    if kind in ("const", "mask"):
        return Gen(kind, [int(tok[1], 0)])
    if kind == "addrof":
        return Gen("addrof", [tok[1], int(tok[2], 0) if len(tok) > 2 else 0])
    if kind == "sum16":
        return Gen("sum16", [int(tok[1], 0), int(tok[2], 0)])
    raise ValueError("bad generator: " + line)


class Arg:
    def __init__(self, kind, ty=None, gen=None, n=0, ofs=0, name=None):
        self.kind, self.ty, self.gen, self.n, self.ofs, self.name = kind, ty, gen, n, ofs, name


class Fix:
    def __init__(self, target, ofs, ty, gen, bits=None, crypt=None, ramp=None):
        self.target, self.ofs, self.ty, self.gen, self.bits = target, ofs, ty, gen, bits
        self.ramp = ramp  # P-spec `ramp` directive: element count (a running sum of draws), else None
        # crypt = (size, seed_width, seed_ofs, skip) with skip = None | (width, ofs, mask): P1 `crypt` directive
        self.crypt = crypt


class Stub:
    def __init__(self, addr, ret, args, noreturn, seq=False, force=False):
        self.addr, self.ret, self.args, self.noreturn, self.seq = addr, ret, args, noreturn, seq
        self.force = force  # B10-spec2: stays a stub although the callee is ported (the Rust port calls it through env.call)


class Fn:
    def __init__(self, addr, ret, mode, inputs):
        self.addr, self.ret, self.mode, self.inputs = addr, ret, mode, inputs
        self.args, self.fixes, self.globals, self.stubs = [], [], [], []
        self.env_bpp = False
        self.regions = []  # [(name, size)], declaration order
        self.nowork = False


class Specs(list):
    """The functions of a spec file; `.set` is its `set NAME` line (None = battle)."""

    set = None


def parse_spec(text):
    area, funcs, cur = None, Specs(), None
    for line in text.split("\n"):
        line = line.split("#")[0].strip()
        if not line:
            continue
        t = line.split()
        if t[0] == "area":
            area = t[1]
        elif t[0] == "set":
            funcs.set = t[1]
        elif t[0] == "fn":
            opts = dict(x.split("=", 1) for x in t[2:])
            cur = Fn(int(t[1], 16), opts.get("ret", "none"), opts.get("mode", "thumb"),
                     int(opts.get("inputs", DEFAULT_INPUTS)))
            funcs.append(cur)
        elif cur is None:
            raise ValueError("directive before fn: " + line)
        elif t[0] == "arg":
            if t[1] == "work":
                cur.args.append(Arg("work", ofs=int(t[2], 0) if len(t) > 2 else 0))
            elif t[1] == "buf":
                cur.args.append(Arg("buf", n=int(t[2], 0)))
            elif t[1] == "region":
                cur.args.append(Arg("region", name=t[2], ofs=int(t[3], 0) if len(t) > 3 else 0))
            elif t[1] == "bpp":
                cur.args.append(Arg("bpp", n=BPP_SIZE))
            else:
                cur.args.append(Arg("scalar", ty=t[1], gen=parse_gen(t[2:], line)))
        elif t[0] == "fix":
            rest, bits, rep = t[4:], None, (1, 0)
            while rest and (rest[-1].startswith("bits=") or rest[-1].startswith("rep=")):
                o = rest.pop()
                a, b = o.split("=", 1)[1].split(":" if o.startswith("bits=") else ",")
                if o.startswith("bits="):
                    bits = (int(a, 0), int(b, 0))
                else:
                    rep = (int(a, 0), int(b, 0))
            for i in range(rep[0]):
                cur.fixes.append(Fix(t[1], int(t[2], 0) + i * rep[1], t[3], parse_gen(rest, line), bits))
        elif t[0] == "crypt":
            # crypt TARGET OFS SIZE u16@SEEDOFS|u32@SEEDOFS [skipif=u16@OFS&MASK]
            sw, so = t[4].split("@")
            skip = None
            if len(t) > 5:
                kw, km = t[5].split("=", 1)[1].split("&")
                kw, ko = kw.split("@")
                skip = (2 if kw == "u16" else 4 if kw == "u32" else 1, int(ko, 0), int(km, 0))
            cur.fixes.append(Fix(t[1], int(t[2], 0), "u32", None, None,
                                 (int(t[3], 0), 2 if sw == "u16" else 4, int(so, 0), skip)))
        elif t[0] == "ramp":
            # ramp TARGET OFS COUNT TYPE GEN (P-spec): COUNT consecutive TYPE values from OFS, each the
            # previous one (0 before the first) plus a GEN draw; a non-decreasing table (EXP curve)
            cur.fixes.append(Fix(t[1], int(t[2], 0), t[4], parse_gen(t[5:], line), None, None, int(t[3], 0)))
        elif t[0] == "region":
            cur.regions.append((t[1], int(t[2], 0)))
        elif t[0] == "nowork":
            cur.nowork = True
        elif t[0] == "global":
            cur.globals.append((int(t[1], 16), int(t[2], 0)))
        elif t[0] == "env":
            cur.env_bpp = t[1] == "bpp"
        elif t[0] == "stub":
            opts = dict(x.split("=", 1) for x in t[2:] if "=" in x)
            args = opts.get("args", "")
            cur.stubs.append(Stub(int(t[1], 16), opts.get("ret", "none"),
                                  args.split(",") if args else [], "noreturn" in t[2:], "seq" in t[2:],
                                  "force" in t[2:]))
        else:
            raise ValueError("unknown directive: " + line)
    for f in funcs:
        for i, a in enumerate(f.args):
            if a.kind in ("buf", "bpp") and (i >= MAX_PTR_ARGS or a.n > BUF_STRIDE or a.n % 4):
                raise ValueError("bad buffer arg %d of %08x" % (i, f.addr))
        names = [n for n, _ in f.regions]
        if len(set(names)) != len(names) or len(names) > MAX_REGIONS:
            raise ValueError("bad regions of %08x" % f.addr)
        if any(n > BUF_STRIDE or n % 4 or n == 0 for _, n in f.regions):
            raise ValueError("bad region size in %08x" % f.addr)
        for a in f.args:
            if a.kind == "region" and a.name not in names:
                raise ValueError("arg region %s of %08x is not declared" % (a.name, f.addr))
    return area, funcs


SPEC_PATH = {}  # area -> spec file (--spec); default svflow_specs/AREA.spec
PORTED_SRC = []  # --ported-src files; empty = server_flow/AREA.rs


def load_spec(area):
    text = open(SPEC_PATH.get(area) or os.path.join(SPECS, area + ".spec")).read()
    if "NEEDS-HAND" in text and not os.environ.get("SVFLOW_ALLOW_DRAFT"):
        sys.exit("svflow: %s.spec still has NEEDS-HAND lines (genspec.py draft); resolve them first" % area)
    return parse_spec(text)[1]


def ported_in(path):
    src = open(path).read()
    if "pub const PORTED" not in src:
        return []
    block = src[src.index("pub const PORTED"):]
    block = block[:block.index("];")]
    return [int(x.replace("_", ""), 16) for x in re.findall(r"0x([0-9A-Fa-f]{4}_?[0-9A-Fa-f]{4})", block)]


def ported(area=None):
    """PORTED addresses of one area, or of everything (area=None). With --ported-src the area's
    list comes from those files; the server_flow modules still count for "everything ported"."""
    if PORTED_SRC:
        return [a for p in PORTED_SRC for a in ported_in(p)] + (ported_flow() if not area else [])
    return ported_flow(area)


def crate_ported():
    """Every `pub const PORTED` address of every logic crate (warning check only: a callee ported in another crate or area
    runs as real code on the ROM side and as that Rust port on the other, e.g. poke_crypt 02019a7c from poke_tool; it never
    deactivates a stub, see Z16)."""
    if not CRATE_PORTED:
        for dp, _, fns in os.walk(os.path.join(ROOT, "crates")):
            if os.sep + "src" not in dp:
                continue
            for fn in fns:
                if fn.endswith(".rs"):
                    CRATE_PORTED.update(ported_in(os.path.join(dp, fn)))
        CRATE_PORTED.add(0)
    return CRATE_PORTED


def ported_flow(area=None):
    out = []
    for src in (FLOW_SRC, CLIENT_SRC, SERVER_SRC):
        if not os.path.isdir(src):
            continue
        for fn in sorted(os.listdir(src)):
            if not fn.endswith(".rs") or (area and fn != area + ".rs"):
                continue
            out += ported_in(os.path.join(src, fn))
    return out


# ------------------------------------------------------------------ stub model


def ret_value(kind, val):
    if kind == "none":
        return None
    if kind == "bool":
        return val & 1
    if kind == "u8":
        return val & 0xFF
    if kind == "u16":
        return val & 0xFFFF
    if kind in ("u32", "i32"):
        return val
    if kind.startswith("mod:"):
        return val % int(kind[4:], 0)
    if kind.startswith("mask:"):
        return val & int(kind[5:], 0)
    if kind.startswith("nullable:"):  # B10-specs: `nullable:Z:KIND` = 0 when mix(val, 0x1234) % Z == 0, else KIND
        z, inner = kind[9:].split(":", 1)
        return 0 if mix(val, 0x1234) % int(z, 0) == 0 else ret_value(inner, val)
    if kind.startswith("pick:"):  # B10-specs: one of V1/V2/... chosen by mix(val, 0xC0DE) % count
        vs = [int(x, 0) for x in kind[5:].split("/")]
        return vs[mix(val, 0xC0DE) % len(vs)] & M
    if kind.startswith("ptr:work:"):  # B10-specs: stub result = address inside the work image, ptr:work:OFS:N
        ofs, n = kind[9:].split(":")
        return (WORK + int(ofs, 0) + 4 * (mix(val, 0xC0DE) % int(n, 0))) & M
    if kind.startswith("ptr:arg"):  # B10-specs: stub result = address inside pointer argument K (see README)
        k, ofs, n = kind[4:].split(":")
        return (Runner.buf_addr(int(k[3:])) + int(ofs, 0) + 4 * (mix(val, 0xC0DE) % int(n, 0))) & M
    if kind == "ofs:work":  # B10f-spec: pointer into the work image: compared as an offset from WORK, 0 stays 0
        return val if val == 0 else (val - WORK) & M
    if kind.startswith("ofs:arg"):  # pointer into pointer argument K: compared as an offset, 0 stays 0
        return val if val == 0 else (val - Runner.buf_addr(int(kind[7:]))) & M
    raise ValueError("ret kind " + kind)


class Ctx:
    """Per-run mutable state shared by the stubs."""

    def __init__(self):
        self.log = []
        self.salt = 0
        self.halted = None
        self.counts = {}  # stub address -> calls so far in this run (`seq` stubs)


def make_stub(st, ctx):
    key = st.addr & 0xFFFF

    def f(em, a0, a1, a2, a3, sp):
        raw = [a0, a1, a2, a3] + [em.r32(sp + 4 * (i - 4)) for i in range(4, len(st.args))]
        vals, extra, outs, outps = [], [], [], []
        for i, k in enumerate(st.args):
            if k == "v":
                vals.append(raw[i])
            elif k == "v16":  # B10-spec4: scalar of which only the low halfword is logged / mixed (the callee ignores the rest)
                vals.append(raw[i] & 0xFFFF)
            elif k.startswith("in:"):
                n = int(k[3:], 0)
                # B10-specs: N need not be a multiple of 4; the last word is zero-padded
                extra += list(struct.unpack("<%dI" % ((n + 3) // 4), em.rbytes(raw[i], n).ljust((n + 3) // 4 * 4, b"\0")))
            elif k.startswith("out:"):
                outs.append((raw[i], int(k[4:], 0), [0xFF], 0))
            elif k.startswith("outm:"):  # outm:N:M0/M1/..., B10-specs: byte j is ANDed with M[j % len(M)]
                _, n, m = k.split(":")
                outs.append((raw[i], int(n, 0), [int(x, 0) & 0xFF for x in m.split("/")], 0x100 * (i + 1)))
            elif k.startswith("outn:"):  # outn:N:M.., B10-spec2: `outm` when the pointer is non-null, else the value 0 (a `v` arg)
                if raw[i] == 0:
                    vals.append(0)
                else:
                    _, n, m = k.split(":")
                    outs.append((raw[i], int(n, 0), [int(x, 0) & 0xFF for x in m.split("/")], 0x100 * (i + 1)))
            elif k.startswith("outp:"):  # outp:K:OFS:N, B12-spec: the out word is an address inside pointer argument K
                _, kk, ofs, n = k.split(":")
                outps.append((raw[i], int(kk, 0), int(ofs, 0), int(n, 0), 0x100 * (i + 1)))
        ctx.log += [st.addr] + vals + extra
        if st.noreturn:
            ctx.halted = st.addr
            em.uc.emu_stop()
            return None
        acc = ctx.salt
        if st.seq:  # B10-specs: the call index (0-based) joins the result, so a polled callee changes its answer
            n = ctx.counts.get(st.addr, 0)
            ctx.counts[st.addr] = n + 1
            acc = mix(acc, n)
        for x in vals:
            acc = mix(acc, x)
        for ptr, n, m, salt in outs:
            if salt:  # `outm`: also mixes in its argument position and is mixed twice (one mix() keeps outputs of
                # different calls XOR-correlated), then takes bits 8-15
                data = bytes((mix(mix(acc, key + 0x1000 + salt + j), 0x5EED) >> 8) & 0xFF & m[j % len(m)] for j in range(n))
            else:
                data = bytes(mix(acc, key + 0x1000 + j) & 0xFF for j in range(n))
            em.wbytes(ptr, data)
        for ptr, kk, ofs, n, salt in outps:  # B12-spec: base(K) + OFS + 4 * (mix(mix(acc, key + 0x1000 + salt), 0xC0DE) % N)
            em.wbytes(ptr, struct.pack("<I", (Runner.buf_addr(kk) + ofs + 4 * (mix(mix(acc, key + 0x1000 + salt), 0xC0DE) % n)) & M))
        if st.ret.startswith("seqptr:"):  # P4: `seqptr:argK:OFS:STEP` = base(K) + OFS + n * STEP, n = this stub's 0-based call
            # index in the run (a bump allocator: every call gets its own disjoint block; the value never depends on acc)
            _, kk, ofs, step = st.ret.split(":")
            n = ctx.counts.get(("seqptr", st.addr), 0)
            ctx.counts[("seqptr", st.addr)] = n + 1
            return (Runner.buf_addr(int(kk[3:])) + int(ofs, 0) + n * int(step, 0)) & M
        if st.ret.startswith("lt:"):  # B10-spec4: `lt:K` = mix(...) % argK (0 when argK == 0): a rand_range(n) answer below n
            n = raw[int(st.ret[3:], 0)]
            return mix(acc, key) % n if n else 0
        return ret_value(st.ret, mix(acc, key))

    return f


# ------------------------------------------------------------------ driver


def words(b):
    return struct.unpack("<%dI" % (len(b) // 4), b)


def fill(r, n):
    return bytearray(struct.pack("<%dI" % (n // 4), *[r.next() for _ in range(n // 4)]))


def set_overlays(name):
    """Overlay numbers of a tools/re/sets.txt set; None for battle (the historical image)."""
    if name in (None, "battle"):
        return None
    for line in open(SETS_TXT):
        t = line.split()
        if t and t[0] == name:
            return [int(x) for x in t[1:] if x != "main"]
    sys.exit("svflow: unknown set %r (tools/re/sets.txt)" % name)


SET = [None]  # --set / spec `set` line
CRATE_PORTED = set()
ALLPORTED = set()  # addresses that may run as real ROM code in a callee position (everything ported)
CALLWARN = set()  # (function, callee) pairs already reported: B10f-spec2 unstubbed-callee check
# pure library helpers the ports inline / call as Rust (soft float, below; 0209c2b0 unsigned divide, 0209c054 64-bit multiply (B10i), 0209c074 64-bit left shift (`lsl` by r2 & 0x3f), 02082bcc MI_CpuFill8,
# 02082d44 MI_CpuCopy8, 02082ac0 bulk copy) and the thunk 021b8ddc (its target 021b8d08 is stubbed): real code on the ROM side by design.
# 021cd63c (B10j) is the `ldr r3,[pc]; bx r3` thunk to the ported cmd_queue push_u8 021ccaa8 (core5 5798 calls the thunk).
# 021ba1a8 (B12-spec; `ldr r0,[r0]; bx` to 02013158) and 021d4c58 (to 021d4cec): thunks whose targets are stubbed.
ALLOW = {0x0209C2B0, 0x0209C054, 0x0209C074, 0x02082BCC, 0x02082D44, 0x02082AC0, 0x021B8DDC, 0x021CD63C, 0x021BA1A8, 0x021D4C58,
         # soft float 0209bdbc / be04 / c0a4 / c494 / c95c / cb8c: black-battle `softfloat` (own hash), called directly by the ports
         0x0209BDBC, 0x0209BE04, 0x0209C0A4, 0x0209C494, 0x0209C95C, 0x0209CB8C,
         # black-pokemon `create`: small helpers the ports call as plain Rust (pub fns of create.rs, not in PORTED; create's hash covers
         # them): 020173ec returns 0xDC (PARTY_MON_SIZE), 02017ed8 shiny test, 02017f08 gender, 0201808c / 020181fc move-slot teach/put,
         # 02018430 thunk to 02018438 (level from EXP, calls growth's 0201850c)
         0x020173EC, 0x02017ED8, 0x02017F08, 0x0201808C, 0x020181FC, 0x02018430,
         # P4: 02018438 (level from EXP; `create::level_of`, its callee 0201850c is a stub): real code in waza 02017fb0 by design,
         # the Rust port calls `level_of` (create's hash covers it through the 02018430 thunk)
         0x02018438,
         # black-text `wordset`: message-tag helpers of another file (0201d5c4 / d5cc / d60c / d670 / d688), real code on the ROM side by
         # design (wordset.spec header); private Rust helpers in wordset.rs, covered by the wordset hash
         0x0201D5C4, 0x0201D5CC, 0x0201D60C, 0x0201D670, 0x0201D688,
         # F7-spec: MIi_CpuCopy16 02082a60 (src, dst, size) and MIi_CpuFill32 02082a7c (value, dst, size), pure ARM helpers like
         # MI_CpuFill8 / MI_CpuCopy8 above; the black-script ports inline them
         0x02082A60, 0x02082A7C,
         # B13-spec2: 02082a90 MI_CpuCopy32 (word copy; client4 021d1a74 copies the adapter data into the command queue)
         0x02082A90,
         # F2a-spec: further ARM9 MI/C library helpers the field ports inline: 02082a48 MIi_CpuClear16(v, dst, n),
         # 0203efc8 memcmp(a, b, n) (byte difference or 0)
         0x02082A48, 0x0203EFC8} | {int(x, 16) for x in os.environ.get("SVFLOW_ALLOW", "").replace(",", " ").split()}

# B10-spec3: callees that run as real ROM code by design in a spec with `env bpp` (events; see the header of events.spec).
# The Rust ports reproduce them (inlined or ported elsewhere), so an equal area hash verifies their effect:
#  * BattleMon accessors 021d5440..021d7f00: `battle_mon` (own hash); `env bpp` mocks only the Pokemon-level API
#  * same_side 021b9a24, poke_side 021b9a40, HANDEX string helpers 021c8838 / 021c8860, pure helper 021e8940: inlined in events.rs
ALLOW_BPP = {0x021B9A24, 0x021B9A40, 0x021C8838,
             0x021C8860, 0x021E8940} | set(range(0x021D5440, 0x021D7F04, 2))


class Runner:
    def __init__(self, fs, active_stubs):
        global UC_HOOK_CODE, UC_ARM_REG_LR
        from unicorn import UC_HOOK_MEM_WRITE, UC_HOOK_CODE
        from unicorn.arm_const import UC_ARM_REG_PC, UC_ARM_REG_LR
        self.pc_reg = UC_ARM_REG_PC
        self.fs = fs
        e = self.e = Emu(set_overlays(SET[0]))
        self.ctx = Ctx()
        self.env = Env(e) if fs.env_bpp else None
        self.owned = [(STACK_LO, STACK_HI)]
        if not fs.nowork:
            self.owned.append((WORK, WORK + WORK_SIZE))
        for i, a in enumerate(fs.args):
            if a.kind in ("buf", "bpp"):
                self.owned.append((self.buf_addr(i), self.buf_addr(i) + a.n))
        for k, (_, n) in enumerate(fs.regions):
            self.owned.append((self.region_addr(k), self.region_addr(k) + n))
        for (ga, gn) in fs.globals:
            self.owned.append((ga, ga + gn))
            # P4: a global outside the mapped RAM (an I/O register such as DISPCNT 0x04000000) gets its 4 KiB pages mapped,
            # zero-filled; the random fill then writes it like any other global (no existing spec has one: hash-neutral)
            mapped = [(lo, hi) for lo, hi, _ in e.uc.mem_regions()]
            for pg in range(ga & ~0xFFF, ga + gn, 0x1000):
                if not any(lo <= pg <= hi for lo, hi in mapped):
                    e.uc.mem_map(pg, 0x1000)
                    mapped.append((pg, pg + 0xFFF))
        for st in fs.stubs:
            if st.addr in active_stubs:
                e.stub(st.addr, make_stub(st, self.ctx))
        self.dirty = {}
        e.uc.hook_add(UC_HOOK_MEM_WRITE, self._write)
        # B10f-spec2: a callee with no stub runs as real ROM code here while the Rust port panics ("no stub").
        # A call = the first instruction after a jump whose lr is the end of the previous instruction.
        self.active = set(active_stubs) | set(e.stubs)  # + the `env bpp` mock API (bpp.Env registers its stubs on the emulator)
        self.prev_end = None
        e.uc.hook_add(UC_HOOK_CODE, self._calls)

    def _calls(self, uc, address, size, _):
        pe, self.prev_end = self.prev_end, address + size
        if pe is None or address == pe:
            return
        a = address & ~1
        if uc.reg_read(UC_ARM_REG_LR) & ~1 != pe or a == self.fs.addr or a in self.active or a in ALLPORTED or a in ALLOW or a in crate_ported():
            return
        if a == RET:
            return
        if self.env and a in ALLOW_BPP:
            return
        t = self.thunk_target(uc, a)
        if t is not None and (t in self.active or t in ALLPORTED or t in ALLOW or t in crate_ported()):
            return  # a thunk (`ldr r3,[pc]; bx r3`) that lands on a stub / ported / allowed callee: not a real-code call
        key = (self.fs.addr, a)
        if key not in CALLWARN:
            CALLWARN.add(key)
            print("svflow: warning: %08x calls %08x, which has no active stub and is not PORTED (the ROM side runs it as real code; "
                  "the Rust port would panic): add a stub" % key, file=sys.stderr)

    @staticmethod
    def thunk_target(uc, a):
        """Target of the Thumb thunk `ldr r3,[pc,#imm]; [movs rN,#k;] bx r3` at `a` (the optional `movs` adds an argument,
        e.g. 0200873c = 020071cc(x, 0x1b)), or None."""
        try:
            h = struct.unpack("<HHH", bytes(uc.mem_read(a, 6)))
            if h[0] & 0xFF00 != 0x4B00:
                return None
            if h[1] != 0x4718 and not (h[1] & 0xF800 == 0x2000 and h[2] == 0x4718):
                return None
            return struct.unpack("<I", bytes(uc.mem_read(((a + 4) & ~3) + (h[0] & 0xFF) * 4, 4)))[0] & ~1
        except Exception:
            return None

    @staticmethod
    def buf_addr(i):
        return BUF_BASE + i * BUF_STRIDE

    @staticmethod
    def region_addr(k):
        return REGION_BASE + k * BUF_STRIDE

    def _write(self, uc, access, address, size, value, _):
        for lo, hi in self.owned:
            if lo <= address < hi:
                return
        for k in range(size):
            self.dirty.setdefault(address + k, None)

    def original(self, a):
        """Byte of the unmodified image at `a` (ARM9 main and the set's overlays, else zero)."""
        for base, blob in self.e.images:
            if base <= a < base + len(blob):
                return blob[a - base]
        return 0

    def addr_of(self, name):
        if name == "work":
            return WORK
        if name.startswith("region:"):
            return self.region_addr([n for n, _ in self.fs.regions].index(name[7:]))
        if name.startswith("arg"):
            return self.buf_addr(int(name[3:]))
        if name.startswith("global"):
            return self.fs.globals[int(name[6:])][0]
        raise ValueError(name)

    def one(self, seed, want):
        """Run one input. Returns (hash, stream or None)."""
        fs, e, ctx = self.fs, self.e, self.ctx
        r = Rng(fs.addr ^ ((seed * 0x9E37) & M))
        ctx.salt = r.next()
        ctx.log, ctx.halted, ctx.counts = [], None, {}
        self.prev_end = None
        if self.env:
            self.env.field = bool(r.next() & 1)
            self.env.log.clear()
        work = None if fs.nowork else fill(r, WORK_SIZE)
        vals, bufs = [], {}
        for i, a in enumerate(fs.args):
            if a.kind == "scalar":
                vals.append(conv(a.gen.draw(r, self.addr_of, None, vals), a.ty))
            elif a.kind == "work":
                vals.append(WORK + a.ofs)
            elif a.kind == "region":
                vals.append(self.addr_of("region:" + a.name) + a.ofs)
            elif a.kind == "buf":
                bufs[i] = fill(r, a.n)
                vals.append(self.buf_addr(i))
            else:
                bufs[i] = random_bpp(r)
                vals.append(self.buf_addr(i))
        gl = [fill(r, n) for (_, n) in fs.globals]
        rg = {nm: fill(r, n) for (nm, n) in fs.regions}
        for fx in fs.fixes:
            if fx.target == "work":
                tgt = work
            elif fx.target.startswith("region:"):
                tgt = rg[fx.target[7:]]
            elif fx.target.startswith("arg"):
                tgt = bufs[int(fx.target[3:])]
            else:
                tgt = gl[int(fx.target[6:])]
            if fx.crypt:
                crypt_fix(tgt, fx)
                continue
            w = TYPES[fx.ty][0].bit_length() // 8
            if fx.ramp:
                acc = 0
                for i in range(fx.ramp):
                    acc = (acc + conv(fx.gen.draw(r, self.addr_of, tgt, vals), fx.ty)) & TYPES[fx.ty][0]
                    tgt[fx.ofs + i * w:fx.ofs + (i + 1) * w] = struct.pack("<I", acc)[:w]
                continue
            v = conv(fx.gen.draw(r, self.addr_of, tgt, vals), fx.ty)
            if fx.bits:
                lo, bw = fx.bits
                m = ((1 << bw) - 1) << lo
                cw = int.from_bytes(tgt[fx.ofs:fx.ofs + w], "little")
                v = (cw & ~m) | ((v << lo) & m)
            tgt[fx.ofs:fx.ofs + w] = struct.pack("<I", v & M)[:w]
        if work is not None:
            e.wbytes(WORK, work)
        for k, (nm, _) in enumerate(fs.regions):
            e.wbytes(self.region_addr(k), rg[nm])
        for i, b in bufs.items():
            e.wbytes(self.buf_addr(i), b)
        for (ga, _), b in zip(fs.globals, gl):
            e.wbytes(ga, b)
        self.dirty.clear()  # the stack window and the registers are reset by Emu.call (Z19)
        fault = None
        ret = None
        try:
            ret = e.call(fs.addr, *vals[:4], stack_args=vals[4:], thumb=fs.mode == "thumb")
            if not ctx.halted and (e.uc.reg_read(self.pc_reg) & ~1) != RET:
                fault = 0x7FFFFFFF  # instruction budget exhausted
        except Exception as ex:  # unicorn UcError: bad memory access etc.
            fault = getattr(ex, "errno", 0xFFFFFFFE)
        stray = []
        for a in sorted(self.dirty):
            try:
                b = e.rbytes(a, 1)[0]
            except Exception:  # B10f-spec2: a dirty address that is not mapped (write crossing the end of a mapping)
                if fault is None:
                    fault = 0xFFFFFFFD
                continue
            if b != self.original(a):
                stray += [a, b]
        stream = []

        def sec(name, ws):
            stream.append((name, ws))

        if fault is not None:
            sec("fault", [fault])
        elif ctx.halted:
            sec("halt", [1, ctx.halted])
        else:
            sec("halt", [0])
            rk = fs.ret
            if rk == "u64":
                sec("ret", [ret[0], ret[1]])
            elif rk != "none":
                sec("ret", [ret_value(rk, ret[0])])
        if not fs.nowork:
            sec("work", words(e.rbytes(WORK, WORK_SIZE)))
        for i, a in enumerate(fs.args):
            if a.kind in ("buf", "bpp"):
                sec("arg%d" % i, words(e.rbytes(self.buf_addr(i), a.n)))
        for k, (nm, n) in enumerate(fs.regions):
            sec("region:" + nm, words(e.rbytes(self.region_addr(k), n)))
        for k, (ga, gn) in enumerate(fs.globals):
            sec("global%d" % k, words(e.rbytes(ga, gn)))
        sec("stray", [len(stray) // 2] + stray)
        sec("stublog", [len(ctx.log)] + ctx.log)
        if self.env:
            flat = [v for ent in self.env.log for v in ent]
            sec("env", [len(flat)] + flat)
        for a in sorted(self.dirty):  # restore the shared image for the next input
            try:
                e.wbytes(a, bytes([self.original(a)]))
            except Exception:
                pass
        h = FNV_OFFSET
        for _, ws in stream:
            for w in ws:
                h = ((h ^ (w & M)) * FNV_PRIME) & M64
        flat = [[n, w & M] for n, ws in stream for w in ws] if want else None
        return "%016x" % h, flat


def crypt_fix(tgt, fx):
    """`crypt` directive: XOR halfwords tgt[OFS, OFS+SIZE) with the poke_tool LCG32 stream
    (state = seed; state = state * 0x41C64E6D + 0x6073; key = state >> 16), unless the skipif test hits."""
    size, sw, so, skip = fx.crypt
    if skip and int.from_bytes(tgt[skip[1]:skip[1] + skip[0]], "little") & skip[2]:
        return
    state = int.from_bytes(tgt[so:so + sw], "little")
    for i in range(size // 2):
        state = (state * 0x41C64E6D + 0x6073) & M
        o = fx.ofs + 2 * i
        tgt[o:o + 2] = ((int.from_bytes(tgt[o:o + 2], "little") ^ (state >> 16)) & 0xFFFF).to_bytes(2, "little")


def run_fn(fs, active_stubs, only_seed=None):
    rn = Runner(fs, active_stubs)
    seeds, faults = [], 0
    stream = None
    for seed in range(fs.inputs):
        if only_seed is not None and seed != only_seed:
            continue
        hx, st = rn.one(seed, only_seed is not None)
        if st:
            stream = st
        seeds.append(hx)
    h = FNV_OFFSET
    for s in seeds:
        v = int(s, 16)
        for w in (v & M, v >> 32):
            h = ((h ^ w) * FNV_PRIME) & M64
    return {"hash": "%016x" % h, "n_inputs": len(seeds), "seeds": seeds}, stream


def area_hash(addrs, funcs):
    h = FNV_OFFSET
    for a in addrs:
        for w in [a] + [x for s in funcs["%08x" % a]["seeds"] for x in (int(s, 16) & M, int(s, 16) >> 32)]:
            h = ((h ^ w) * FNV_PRIME) & M64
    return "%016x" % h


def _job(args):
    area, addrs, allported = args
    ALLPORTED.update(allported)
    spec = {f.addr: f for f in load_spec(area)}
    out = {}
    for a in addrs:
        out["%08x" % a] = run_fn(spec[a], allstubs(spec[a], allported))[0]
    out["__warn"] = sorted(CALLWARN)
    return out


def allstubs(fs, allported):
    return {s.addr for s in fs.stubs if s.force or s.addr not in allported}


def poison(e, r):
    """Z19 order check: leave junk in every register, the flags and the stack window, as an
    arbitrary earlier call would. Emu.call must make it invisible."""
    from emu import RESET_REGS
    from unicorn.arm_const import UC_ARM_REG_CPSR
    for reg in RESET_REGS:
        e.uc.reg_write(reg, r.next())
    e.uc.reg_write(UC_ARM_REG_CPSR, e.uc.reg_read(UC_ARM_REG_CPSR) | 0xF80F0000)
    e.wbytes(STACK_LO, fill(r, STACK_HI - STACK_LO))


def order_check(fs, active_stubs):
    """Z19: per-input hashes of one function are the same in seed order on a fresh Runner and in
    reverse seed order on one Runner whose registers / flags / stack window are poisoned before
    every input. Returns the list of seeds that differ."""
    fwd = run_fn(fs, active_stubs)[0]["seeds"]
    rn = Runner(fs, active_stubs)
    r = Rng(0x2019)
    bad = []
    for seed in reversed(range(fs.inputs)):
        poison(rn.e, r)
        if rn.one(seed, False)[0] != fwd[seed]:
            bad.append(seed)
    return sorted(bad)


def selftest():
    """Z19 reset probes on hand-written code, then the order check on the function that found
    the leak (server1 021b6848 reads the caller's saved r3 through a stack buffer) and on the
    selftest area. Prints `selftest ok` or exits 1."""
    from unicorn.arm_const import UC_ARM_REG_CPSR
    e = Emu()
    base = 0x02FE0000  # free RAM below RET, outside every image and harness region
    probes = [  # (name, thumb?, code words): each returns the probed value in r0
        ("r3", True, [0x4618, 0x4770]),  # mov r0, r3; bx lr
        ("r4", True, [0x1C20, 0x4770]),  # adds r0, r4, #0; bx lr (reads r4 before any flag-setting matters)
        ("r12", True, [0x4660, 0x4770]),  # mov r0, r12; bx lr
        ("stack below sp", True, [0xB081, 0x9800, 0xB001, 0x4770]),  # sub sp,#4; ldr r0,[sp]; add sp,#4; bx lr
        ("stack word after the arguments", True, [0x9801, 0x4770]),  # ldr r0, [sp, #4]; bx lr (1 stack arg passed)
        ("cpsr flags", False, [0xE10F0000, 0xE12FFF1E]),  # mrs r0, cpsr; bx lr (ARM)
    ]
    r = Rng(19)
    fails = []
    for k, (name, thumb, code) in enumerate(probes):
        a = base + 0x40 * k
        e.wbytes(a, struct.pack("<%d%s" % (len(code), "H" if thumb else "I"), *code))
        for trial in range(3):
            poison(e, r)
            e.call(a, 0x11, 0x22, 0x33, 0x44, stack_args=(0x55, 0x66), thumb=thumb)  # dirty r0-r3 / stack args
            poison(e, r)
            got = e.call(a, 0x11, stack_args=(0x55,), thumb=thumb)[0]
            if name == "cpsr flags":
                got &= 0xF80F0000
            if got != 0:
                fails.append("%s reads %#x (trial %d)" % (name, got, trial))
    if e.uc.reg_read(UC_ARM_REG_CPSR) & 0x1F == 0:
        fails.append("cpsr mode bits changed")
    # the arguments themselves still arrive
    a = base + 0x40 * len(probes)
    e.wbytes(a, struct.pack("<HHHH", 0x18C0, 0x9900, 0x1840, 0x4770))  # adds r0,r0,r3; ldr r1,[sp]; adds r0,r0,r1; bx lr
    if e.call(a, 5, 0, 0, 7, stack_args=(9,))[0] != 21:
        fails.append("arguments r0/r3/stack word 0 not passed")
    checks = [("server1", 0x021B6848)] + [("selftest", f.addr) for f in load_spec("selftest")]
    for area, addr in checks:
        spec = {f.addr: f for f in load_spec(area)}
        fs = spec[addr]
        fs.inputs = min(fs.inputs, 64)
        bad = order_check(fs, allstubs(fs, set(ported())))
        if bad:
            fails.append("%s %08x: input hash depends on run order / leftover CPU state at seeds %s" % (area, addr, bad[:8]))
    if fails:
        print("selftest FAILED:\n  " + "\n  ".join(fails))
        return 1
    print("selftest ok (%d reset probes, %d order checks)" % (len(probes) + 1, len(checks)))
    return 0


def main(argv):
    args = argv[1:]
    if args[:1] == ["--selftest"]:
        return selftest()
    if not args or args[0].startswith("-"):
        print(__doc__)
        return 2
    area = args[0]
    opt = args[1:]

    def val(flag):
        return opt[opt.index(flag) + 1] if flag in opt else None

    while "--spec" in opt:
        i = opt.index("--spec")
        SPEC_PATH[area] = opt[i + 1]
        del opt[i:i + 2]
    while "--ported-src" in opt:
        i = opt.index("--ported-src")
        PORTED_SRC.append(opt[i + 1])
        del opt[i:i + 2]
    SET[0] = val("--set")
    specs = load_spec(area)
    SET[0] = SET[0] or specs.set
    spec = {f.addr: f for f in specs}
    allported = set(ported())
    ALLPORTED.update(allported)
    addrs = [f.addr for f in specs] if "--all" in opt else ported(area)
    if val("--fn"):
        addrs = [int(val("--fn"), 16)]
    for a in addrs:
        if a not in spec:
            print("svflow: %08x is not in svflow_specs/%s.spec" % (a, area), file=sys.stderr)
            return 3
    if "--order-check" in opt:  # Z19: every input hashes the same in any order / after leftover CPU state
        bad = 0
        for a in addrs:
            seeds = order_check(spec[a], allstubs(spec[a], allported))
            if seeds:
                bad += 1
                print("%s %08x order-dependent at seeds %s" % (area, a, seeds[:16]))
        print("%s order-check: %d/%d functions order-dependent" % (area, bad, len(addrs)))
        return 1 if bad else 0
    if val("--detail"):
        a, sd = val("--detail").split(":")
        a = int(a, 16)
        _, st = run_fn(spec[a], allstubs(spec[a], allported), int(sd))
        print(json.dumps(st))
        return 0
    jobs = int(val("--jobs") or 1)
    if jobs > 1:
        from multiprocessing import Pool
        n = jobs * 4
        with Pool(jobs) as p:
            parts = p.map(_job, [(area, addrs[i::n], allported) for i in range(n)])
        funcs = {}
        for d in parts:
            CALLWARN.update(tuple(k) for k in d.pop("__warn"))
            funcs.update(d)
    else:
        funcs = {"%08x" % a: run_fn(spec[a], allstubs(spec[a], allported))[0] for a in addrs}
    total = area_hash(addrs, funcs)
    if "--strict-callees" in opt and CALLWARN:
        return 4
    if "--json" in opt:
        print(json.dumps({"module": area, "hash": total, "funcs": funcs}))
    else:
        print(area, total)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
