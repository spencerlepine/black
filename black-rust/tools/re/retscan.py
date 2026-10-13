"""retscan.py (V-0022 audit): find spec'd functions declared `ret=none` whose result some caller uses.

`ret=none` in a spec hides whatever the function leaves in r0, so a port that drops a real return value still
matches. For every `fn ... ret=none` of tools/rom-diff/svflow_specs/*.spec and tools/rom-diff/specs/*/*.spec:
  * callers = every Thumb BL/BLX (also inside `$d` byte blocks, marked `?`) and ARM BL/BLX in main + every overlay
    of the build (main.elf sections, OVY_n.sbin bytes) that targets the function; a caller in an overlay that covers
    the target address, or one that cannot be co-resident with the target's overlay (tools/re/sets.proposed.txt), is
    ignored;
  * after each call the caller is walked (both arms of conditional branches, <= 60 instructions per path) until r0 is
    read (READ), overwritten (clobber), passed on into the next call, or returned on (then the caller's own callers
    are checked, depth 3, same image or main only);
  * literal-pool references (`ldr rX, =fn; bx rX` tail jumps / thunks, `blx rX` calls) are followed the same way.
A READ / CHAIN line is a candidate: decide from the disassembly (rizin `pd`) whether the value is real, then fix the
spec kind, the port and the callers. Needs capstone (not in the Unicorn venv): `python3.12 -m venv V &&
V/bin/pip install capstone`, then `V/bin/python -I tools/re/retscan.py [OUT.json]`. Reads $BLACK_ROM_BUILD,
$SPEC_ROOT (default: this checkout) and the decompile indexes /tmp/blk/re/{battle,field}/pc/index.jsonl.
"""
import json
import os
import re
import struct
import sys
from collections import defaultdict

import capstone
from capstone import arm_const as A

B = os.environ.get("BLACK_ROM_BUILD", os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../../black-rom/build/black.us"))
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
sys.path.insert(0, ROOT + "/tools/re")
import mkelf  # noqa

d, _, secs = mkelf.read_elf(B + "/main.elf")
images = {}  # name -> (addr, blob)
secimg = {}  # section index -> image name
for i, s in enumerate(secs):
    if s["type"] != mkelf.SHT_PROGBITS:
        continue
    n = s["name"]
    if n == "main" or re.fullmatch(r"OVY_\d+", n):
        if n not in images:
            p = B + "/%s.sbin" % n
            if not os.path.exists(p):
                continue
            images[n] = (s["addr"], open(p, "rb").read())
        if s["addr"] >= images[n][0] and s["addr"] < images[n][0] + len(images[n][1]):
            secimg[i] = n
symtab = next(s for s in secs if s["type"] == mkelf.SHT_SYMTAB)
strtab = secs[symtab["link"]]
maps = defaultdict(list)  # image -> [(addr, 'a'|'t'|'d')]
funcs = defaultdict(set)  # image -> addrs of symbols (function starts)
for k in range(1, symtab["size"] // 16):
    nm, val, size, info, oth, shn = struct.unpack_from("<IIIBBH", d, symtab["off"] + 16 * k)
    if shn not in secimg:
        continue
    o = strtab["off"] + nm
    name = d[o:d.index(b"\0", o)].decode(errors="replace")
    im = secimg[shn]
    if name in ("$a", "$t", "$d", "$b"):
        maps[im].append((val & ~1, "t" if name == "$b" else name[1]))
    elif name and not name.startswith("$"):
        funcs[im].add(val & ~1)
for im in maps:
    maps[im].sort()
    funcs[im] = sorted(funcs[im])
import bisect


def mode_at(im, a):
    m = maps[im]
    i = bisect.bisect_right(m, (a, "~")) - 1
    return m[i][1] if i >= 0 else "d"


def rd16(im, a):
    base, blob = images[im]
    return struct.unpack_from("<H", blob, a - base)[0]


def inimg(im, a):
    base, blob = images[im]
    return base <= a < base + len(blob)


def image_of(a, prefer):
    for im in prefer:
        if im in images and inimg(im, a):
            return im
    return None


# ---- callers: sequential decode of every $t / $a region
calls = defaultdict(list)  # target -> [(src image, call addr, mode, kind)]
for im, (base, blob) in images.items():
    m = maps[im]
    for idx, (a0, k) in enumerate(m):
        a1 = m[idx + 1][0] if idx + 1 < len(m) else base + len(blob)
        a1 = min(a1, base + len(blob))
        if k in ("t", "d"):
            a = (a0 + 1) & ~1
            while a + 4 <= a1:
                h1 = rd16(im, a)
                if (h1 & 0xF800) == 0xF000:
                    h2 = rd16(im, a + 2)
                    if (h2 & 0xE800) == 0xE800:
                        off = ((h1 & 0x7FF) << 12) | ((h2 & 0x7FF) << 1)
                        if off & 0x400000:
                            off -= 0x800000
                        t = a + 4 + off
                        if (h2 & 0xF800) == 0xE800:
                            t &= ~3
                            calls[t].append((im, a, "t", "blx" + ("?" if k == "d" else "")))
                        else:
                            calls[t].append((im, a, "t", "bl" + ("?" if k == "d" else "")))
                        a += 4
                        continue
                a += 2
        elif k == "a":
            a = (a0 + 3) & ~3
            while a + 4 <= a1:
                w = struct.unpack_from("<I", blob, a - base)[0]
                if (w >> 24) in (0xFA, 0xFB):  # BLX imm
                    off = (w & 0xFFFFFF) << 2
                    if off & 0x2000000:
                        off -= 0x4000000
                    t = a + 8 + off + (2 if (w >> 24) == 0xFB else 0)
                    calls[t].append((im, a, "a", "blx"))
                elif (w & 0x0F000000) == 0x0B000000 and (w >> 28) != 0xF:
                    off = (w & 0xFFFFFF) << 2
                    if off & 0x2000000:
                        off -= 0x4000000
                    calls[a + 8 + off].append((im, a, "a", "bl"))
                a += 4

cs_t = capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_THUMB)
cs_a = capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_ARM)
for c in (cs_t, cs_a):
    c.detail = True


def insn_at(im, a, mode):
    base, blob = images[im]
    cs = cs_t if mode == "t" else cs_a
    code = blob[a - base:a - base + 4]
    for i in cs.disasm(code, a):
        return i
    return None


def regsets(i):
    try:
        r, w = i.regs_access()
    except Exception:
        r, w = list(i.regs_read), list(i.regs_write)
    return set(r), set(w)


R0 = A.ARM_REG_R0


def is_ret(i):
    m = i.mnemonic
    if m.startswith("pop") or m.startswith("ldm"):
        return "pc" in i.op_str
    if m.startswith("bx") and "lr" in i.op_str:
        return True
    if m.startswith("mov") and i.op_str.startswith("pc"):
        return True
    return False


def follow(im, a, mode, budget=60):
    """Walk from a (the instruction after the call). Returns set of outcomes."""
    out = set()
    stack = [(a, budget)]
    seen = set()
    while stack:
        a, b = stack.pop()
        while True:
            if b <= 0 or (a, mode) in seen or not inimg(im, a):
                out.add(("budget", a, ""))
                break
            seen.add((a, mode))
            i = insn_at(im, a, mode)
            if i is None:
                out.add(("undecodable", a, ""))
                break
            b -= 1
            txt = "%08x %s %s" % (i.address, i.mnemonic, i.op_str)
            m = i.mnemonic
            if m in ("bl", "blx"):
                if m == "blx" and not i.op_str.startswith("#"):
                    r, _ = regsets(i)
                    if R0 in r or i.op_str.strip() == "r0":
                        out.add(("READ", a, txt))
                        break
                out.add(("passes-into-call", a, txt))
                break
            r, w = regsets(i)
            if is_ret(i):
                if "r0" in i.op_str and m.startswith("pop"):
                    out.add(("clobber", a, txt))
                else:
                    out.add(("passes-on-return", a, txt))
                break
            if R0 in r:
                out.add(("READ", a, txt))
                break
            if R0 in w:
                out.add(("clobber", a, txt))
                break
            # branches
            if m.startswith("b") and not m.startswith("bic") and i.operands and i.operands[0].type == A.ARM_OP_IMM:
                t = i.operands[0].imm
                cond = i.cc not in (A.ARM_CC_AL, A.ARM_CC_INVALID)
                if cond:
                    stack.append((t, b))
                    a = a + i.size
                    continue
                a = t
                continue
            if m.startswith("bx"):  # pop {..}; pop {r3}; add sp; bx r3 (varargs epilogue) = a return
                out.add(("passes-on-return", a, txt))
                break
            if (m.startswith("mov") and i.op_str.startswith("pc")) or (m.startswith("ldr") and i.op_str.startswith("pc")):
                out.add(("indirect-jump", a, txt))
                break
            a += i.size
    return out


GSIZE = {}
GIM = {"battle": ["main", "OVY_93", "OVY_94", "OVY_95", "OVY_96"], "field": ["main", "OVY_10", "OVY_11", "OVY_18", "OVY_20", "OVY_21"]}
GSIZE_IM = defaultdict(dict)
for st in ("battle", "field"):
    for line in open("/tmp/blk/re/%s/pc/index.jsonl" % st):
        j = json.loads(line)
        a = int(j["addr"], 16)
        for im in GIM[st]:
            if im in images and inimg(im, a):
                GSIZE_IM[im][a] = j["size"]
                if im == "main" or st == "battle" or a not in GSIZE:
                    GSIZE[a] = j["size"]


def fn_end(im, a):
    if a in GSIZE_IM.get(im, {}):
        return a + GSIZE_IM[im][a]
    f = funcs[im]
    i = bisect.bisect_right(f, a)
    return f[i] if i < len(f) else images[im][0] + len(images[im][1])


def callee_shape(im, a, mode):
    """Linear sweep of the fn body: for each return point, the last instruction that wrote r0 (linear, approximate)."""
    end = min(fn_end(im, a), a + 0x2000)
    x = a
    last = "entry(arg0)"
    shapes = []
    while x < end:
        i = insn_at(im, x, mode)
        if i is None:
            break
        m = i.mnemonic
        txt = "%s %s" % (m, i.op_str)
        if m in ("bl", "blx"):
            last = "call:" + i.op_str
        else:
            r, w = regsets(i)
            if is_ret(i):
                shapes.append(last if not ("r0" in i.op_str and m.startswith("pop")) else "pop r0")
                x += i.size
                continue
            if R0 in w:
                last = "set:" + txt
            if m.startswith("b") and not m.startswith("bic") and i.operands and i.operands[0].type == A.ARM_OP_IMM \
                    and i.cc in (A.ARM_CC_AL, A.ARM_CC_INVALID):
                t = i.operands[0].imm
                if not (a <= t < end):
                    shapes.append("tailjump:%x(after %s)" % (t, last))
        x += i.size
    return shapes


# ---- specs
SETS = {"battle": ["main", "OVY_93", "OVY_94", "OVY_95", "OVY_96"], "battle+96": ["main", "OVY_93", "OVY_96"],
        "field": ["main", "OVY_10", "OVY_11", "OVY_18", "OVY_20", "OVY_21"]}
import glob

SR = os.environ.get("SPEC_ROOT", ROOT)
specs = sorted(glob.glob(SR + "/tools/rom-diff/svflow_specs/*.spec") + glob.glob(SR + "/tools/rom-diff/specs/*/*.spec"))
fns = []  # (spec, set, addr, ret, mode)
stubs = defaultdict(list)  # addr -> [(spec, owner fn, ret, force)]
for sp in specs:
    st = "battle"
    owner = None
    for line in open(sp):
        line = line.split("#")[0].rstrip()
        if not line.strip():
            continue
        t = line.split()
        if t[0] == "set" and not line.startswith(" "):
            st = t[1]
        if t[0] == "fn":
            owner = t[1]
            ret = next((x[4:] for x in t if x.startswith("ret=")), "?")
            mode = "a" if "mode=arm" in t else "t"
            fns.append((os.path.relpath(sp, SR), st, int(t[1], 16), ret, mode))
        elif t[0] == "stub":
            ret = next((x[4:] for x in t if x.startswith("ret=")), "?")
            stubs[int(t[1], 16)].append((os.path.relpath(sp, SR), owner, ret, "force" in t))


GSTARTS_IM = {im: sorted(v) for im, v in GSIZE_IM.items()}


def fn_start(im, a, mode="t"):
    f = funcs[im]
    i = bisect.bisect_right(f, a) - 1
    best = f[i] if i >= 0 else None
    gs = GSTARTS_IM.get(im, [])
    j = bisect.bisect_right(gs, a) - 1
    if j >= 0 and gs[j] + GSIZE_IM[im][gs[j]] > a:
        best = max(best or 0, gs[j])
    if mode == "t":
        x = a - 2
        lo = max(images[im][0], a - 0x800, best or 0)
        while x >= lo:
            h = rd16(im, x)
            if (h & 0xFF00) == 0xB500:  # push {.., lr}
                best = max(best or 0, x)
                break
            x -= 2
    return best


def entry_reads_r0(im, a, mode):
    outs = follow(im, a, mode, budget=30)
    return [o[2] for o in outs if o[0] == "READ"]


GROUPS = []
for line in open(ROOT + "/tools/re/sets.proposed.txt"):
    if line.strip() and not line.startswith("#"):
        GROUPS.append(set("main" if x == "main" else "OVY_" + x for x in line.split()[1:]))
GROUPS.append({"main", "OVY_93", "OVY_94", "OVY_95", "OVY_96", "OVY_10", "OVY_11"})


def compat(src, tgt, strict=True):
    if src == tgt or tgt == "main":
        return True
    if src == "main":
        return not strict  # main calling into an overlay address: only for direct call sites
    return any(src in g and tgt in g for g in GROUPS)


def sites_of(addr, tim, strict=True):
    for (sim, ca, cmode, kind) in calls.get(addr, []):
        if sim != tim and sim != "main" and inimg(sim, addr):
            continue
        if not compat(sim, tim, strict):
            continue
        yield sim, ca, cmode, kind


def chain(addr, tim, depth, seen, strict=True):
    """Evidence that a caller of addr uses r0 (directly, via returning it on, or by passing it to a callee that reads it)."""
    ev = []
    if (addr, tim) in seen:
        return ev
    seen.add((addr, tim))
    for sim, ca, cmode, kind in sites_of(addr, tim, strict):
        for o in follow(sim, ca + 4, cmode):
            if o[0] == "READ":
                ev.append("%s@%08x%s: %s" % (sim, ca, "?" if kind.endswith("?") else "", o[2]))
            elif o[0] == "passes-into-call":
                i = insn_at(sim, o[1], cmode)
                t = i.operands[0].imm if i.operands and i.operands[0].type == A.ARM_OP_IMM else None
                if t is not None:
                    tm = "a" if (i.mnemonic == "blx") != (cmode == "a") else "t"
                    cim = sim if inimg(sim, t) else image_of(t, ["main"] + list(images))
                    if cim:
                        rr = entry_reads_r0(cim, t, tm)
                        if rr:
                            ev.append("%s@%08x: passed as arg0 to %08x which reads it (%s)" % (sim, ca, t, rr[0]))
            elif o[0] == "passes-on-return" and depth > 0:
                c = fn_start(sim, ca, cmode)
                if c is not None:
                    sub = chain(c, sim, depth - 1, seen)
                    if sub:
                        ev.append("%s@%08x: returned on by %08x, whose caller %s" % (sim, ca, c, sub[0]))
    return ev


res = []
for sp, st, addr, ret, mode in fns:
    if ret != "none":
        continue
    tim = image_of(addr, SETS.get(st, SETS["battle"]))
    if tim is None:
        res.append(dict(spec=sp, fn="%08x" % addr, err="no image"))
        continue
    md = mode
    sites = []
    for (sim, ca, cmode, kind) in sites_of(addr, tim, False):
        outs = follow(sim, ca + 4, cmode)
        sites.append(dict(kind=kind, img=sim, at="%08x" % ca, mode=cmode, out=sorted(set((o[0], o[2]) for o in outs))))
    shape = callee_shape(tim, addr, md)
    # pointer refs (indirect use)
    word = struct.pack("<I", addr | (1 if md == "t" else 0))
    prefs = 0
    for im, (base, blob) in images.items():
        prefs += blob.count(word)
    ev = chain(addr, tim, 3, set(), False)
    # literal-pool references: ldr rX,=fn; bx rX (thunk / tail jump) or blx rX (call), else a plain pointer
    lit = dict(tail=0, blxreg=0, ptr=0)
    for im, (base, blob) in images.items():
        if not compat(im, tim) and im != tim:
            continue
        o = blob.find(word)
        while o >= 0:
            L = base + o
            used = False
            if L % 4 == 0:
                for I in range(max(base, L - 1020), L, 2):
                    h = rd16(im, I)
                    if (h & 0xF800) == 0x4800 and ((I + 4) & ~3) + (h & 0xFF) * 4 == L:
                        rx = (h >> 8) & 7
                        for k in range(1, 4):
                            if not inimg(im, I + 2 * k + 2):
                                break
                            h2 = rd16(im, I + 2 * k)
                            if h2 == (0x4700 | (rx << 3)):  # bx rX
                                used = True
                                lit["tail"] += 1
                                c = fn_start(im, I, "t")
                                if c is not None:
                                    for e2 in chain(c, im, 2, set()):
                                        ev.append("%s@%08x: tail jump (ldr r%d; bx r%d) from %08x, whose caller %s" % (im, I, rx, rx, c, e2))
                                break
                            if h2 == (0x4780 | (rx << 3)):  # blx rX
                                used = True
                                lit["blxreg"] += 1
                                for oo in follow(im, I + 2 * k + 2, "t"):
                                    if oo[0] == "READ":
                                        ev.append("%s@%08x: blx r%d then %s" % (im, I + 2 * k, rx, oo[2]))
                                break
                            if (h2 >> 8) & 0xFF in (0x47,) or (h2 & 0xF800) in (0xE000, 0xF000):
                                break
            if not used:
                lit["ptr"] += 1
            o = blob.find(word, o + 1)
    res.append(dict(spec=sp, set=st, fn="%08x" % addr, img=tim, mode=md, shape=shape, sites=sites, ptrrefs=prefs, lit=lit, ev=ev,
                    stubs=[list(s) for s in stubs.get(addr, [])]))
json.dump(res, open(sys.argv[1] if len(sys.argv) > 1 else "/dev/null", "w"), indent=1)
# summary
nflag = 0
for r in res:
    if "err" in r:
        print("ERR", r)
        continue
    kinds = [o[0] for s in r["sites"] for o in s["out"]]
    reads = [o[1] for s in r["sites"] for o in s["out"] if o[0] == "READ"]
    weak = [k for k in kinds if k in ("passes-into-call", "passes-on-return")]
    tail = any(x.startswith("call:") or x.startswith("tailjump") for x in r["shape"])
    stub_rets = sorted(set(s[2] for s in r["stubs"]))
    flag = "READ" if reads else ("CHAIN" if r["ev"] else ("weak" if weak else "-"))
    if r["ev"]:
        nflag += 1
        reads = r["ev"]
    print("%s %-8s %-4s sites=%d ptr=%s tail=%d shape=%s stubs=%s %s" % (
        r["fn"], flag, r["spec"].split("/")[-1][:-5], len(r["sites"]), "%d/%d/%d" % (r["lit"]["tail"], r["lit"]["blxreg"], r["lit"]["ptr"]), tail,
        ",".join(sorted(set(x[:40] for x in r["shape"]))), stub_rets, ("| " + " ; ".join(sorted(set(reads)))[:300]) if reads else ""))
print("flagged READ:", nflag, "of", len(res))
