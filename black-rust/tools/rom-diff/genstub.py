#!/usr/bin/env python3
"""Harness stub skeleton generator (MIGRATION-AGENT.md section 3b, Z9).

usage: python3 -I tools/rom-diff/genstub.py <addr> [--set battle] [--tag-base 0x126] [--packet FILE]

Reads the work packet of <addr> (default /tmp/blk/packets/<set>/<addr>.md, built on the fly with
tools/re/packet.py if missing) and prints, for every callee that is not ported to Rust and not
already stubbed in handlers.py/bpp.py:
  1. a Python skeleton: one `st(addr, lambda ...)` line per callee, for Host.__init__ in handlers.py
  2. a Rust skeleton: the FlowHost trait declaration and the TestHost impl, for flow.rs / testkit.rs
Both follow the existing convention: the call and its scalar arguments are logged as
[TAG, args...] (TAG 0x1xx, allocated from the highest one used in handlers.py/testkit.rs) and the
return value is mix(fold(args), addr & 0xFFFF), so Python and Rust agree bit for bit.
Pointer arguments are not logged (emulator addresses are not comparable); a callee that writes
through a pointer, or returns one, is flagged NEEDS-HAND and must be modelled by the coordinator.
The output is generated: porting workers consume it and must not edit it.
Pure Python 3 stdlib.
"""
import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))  # black-rust/
RE_DIR = os.path.join(ROOT, "tools", "re")
HANDLERS_PY = os.path.join(HERE, "handlers.py")
BPP_PY = os.path.join(HERE, "bpp.py")
TESTKIT_RS = os.path.join(ROOT, "crates", "black-battle", "src", "handlers", "testkit.rs")

ROW = re.compile(r"^\| `([0-9a-f]{8})` \| (\S+) \| `(.*?)`(?: \[[^\]]*\])? \| (.*) \|$")
BOOL_T = {"bool"}
B8_T = {"undefined1", "byte", "uchar", "u8", "char", "s8", "sbyte"}
B16_T = {"undefined2", "ushort", "short", "u16", "s16", "word"}
SIGNED_T = {"int", "s32", "long"}


def load_packet_module():
    spec = importlib.util.spec_from_file_location("re_packet", os.path.join(RE_DIR, "packet.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def packet_text(setname, addr, path):
    if path:
        return open(path).read()
    default = os.path.join(os.environ.get("PACKETS_OUT", "/tmp/blk/packets"), setname, addr + ".md")
    if os.path.exists(default):
        return open(default).read()
    return load_packet_module().build(setname, addr)[0]


def parse_callees(text):
    out = []
    in_tbl = False
    for ln in text.split("\n"):
        if ln.startswith("## Callees"):
            in_tbl = True
            continue
        if in_tbl and ln.startswith("## "):
            break
        m = ROW.match(ln) if in_tbl else None
        if m:
            out.append({"addr": m.group(1), "name": m.group(2), "proto": m.group(3), "rust": m.group(4)})
    return out


def kind(ctype):
    """(class, rust type) for a C type string."""
    t = ctype.replace("const", "").strip()
    if "*" in t or "[" in t:
        return "ptr", None
    t = t.split()[-1] if t else "undefined4"
    if t in BOOL_T:
        return "bool", "bool"
    if t in B8_T:
        return "u8", "u8"
    if t in B16_T:
        return "u16", "u16"
    if t in SIGNED_T:
        return "i32", "i32"
    if t == "void":
        return "void", None
    return "u32", "u32"


def parse_proto(proto):
    """(ret_type, [(ctype, name)], guessed). Unknown prototypes fall back to 4 x undefined4."""
    m = re.match(r"^(?P<ret>.*?)\s*\b(?P<name>[A-Za-z_]\w*)\s*\((?P<args>.*)\)\s*$", proto.strip())
    if not m or "(" in m.group("args") or not m.group("ret"):
        return "undefined4", [("undefined4", "param_%d" % (i + 1)) for i in range(4)], True
    args = []
    raw = m.group("args").strip()
    if raw and raw != "void":
        for i, a in enumerate(x.strip() for x in raw.split(",")):
            if a == "...":
                continue
            mm = re.match(r"^(.*?)([A-Za-z_]\w*)((?:\[\d*\])*)$", a)
            if mm and mm.group(1).strip() and mm.group(1).strip() not in ("unsigned", "struct", "const"):
                ctype, nm = mm.group(1).strip() + mm.group(3), mm.group(2)
            else:
                ctype, nm = a, "p%d" % (i + 1)
            args.append((ctype, nm))
    return m.group("ret").strip(), args, False


def used_tags():
    tags = set()
    for p in (HANDLERS_PY, TESTKIT_RS):
        for m in re.finditer(r"log(?:\.extend\(\[|\.append\(|\.push\(| \+= \[)\s*0x(1[0-9A-Fa-f]{2})\b", open(p).read()):
            tags.add(int(m.group(1), 16))
    return tags


def harnessed(addr):
    pat = re.compile(r"\bst(?:ub)?\(\s*0x0*%s\b" % addr.lstrip("0"), re.I)
    return any(pat.search(open(p).read()) for p in (HANDLERS_PY, BPP_PY))


def rust_name(c):
    m = re.search(r"`(?:\w+::)?(\w+)`", c["rust"])
    if c["rust"].startswith("trait stub") and m:
        return m.group(1)
    return "fun_" + c["addr"]


def in_testkit(name):
    return re.search(r"\bfn %s\(" % re.escape(name), open(TESTKIT_RS).read()) is not None


def gen(c, tag):
    ret_t, args, guessed = parse_proto(c["proto"])
    rk, rrust = kind(ret_t)
    key = "0x%04X" % (int(c["addr"], 16) & 0xFFFF)
    flags = ["GUESSED prototype (4 x undefined4)"] if guessed else []
    # Python/Rust argument expressions for the scalar args; pointers are skipped
    py_args, rs_args, rs_sig, pyl, log = [], [], [], [], []
    for i, (ct, nm) in enumerate(args):
        k, rt = kind(ct)
        reg = "r%d" % i if i < 4 else None
        var = re.sub(r"\W", "_", nm)
        if i < 4:
            pyl.append(var)
            src = var
        else:
            src = "em.r32(sp + %d)" % (4 * (i - 4))
        if k == "ptr":
            flags.append("pointer arg %s (%s) is not logged; model any write through it by hand" % (nm, ct))
            continue
        mask = {"u8": " & 0xFF", "u16": " & 0xFFFF", "bool": " & 1"}.get(k, "")
        e = ("(%s%s)" % (src, mask)) if mask else src
        py_args.append(e)
        cast = {"u8": " as u32", "u16": " as u32", "bool": " as u32", "i32": " as u32"}.get(k, "")
        rs_args.append("%s%s" % (var, cast) if cast else var)
        rs_sig.append("%s: %s" % (var, rt))
    # fold: mix(mix(a0, a1), a2) ..., then mix(fold, KEY); no scalar args -> mix(self.wseed, KEY)
    def fold(items, base):
        if not items:
            return base
        acc = items[0]
        for x in items[1:]:
            acc = "mix(%s, %s)" % (acc, x)
        return acc
    py_val = "mix(%s, %s)" % (fold(py_args, "self.wseed"), key)
    rs_val = "mix(%s, %s)" % (fold(rs_args, "self.wseed"), key)
    py_ret = {"bool": " & 1", "u8": " & 0xFF", "u16": " & 0xFFFF"}.get(rk, "")
    if re.search(r"undefined8|long long|\bdouble\b|\b[su]64\b", " ".join([ret_t] + [t for t, _ in args])):
        flags.append("64-bit value in the prototype: r0/r1 pair not modelled")
    if rk == "ptr":
        flags.append("pointer return: Python returns TMP; the Rust side needs a hand-written model")
    nreg = min(len(args), 4)
    names = "em, " + "".join(p + ", " for p in pyl[:nreg]) + ("*_, " if len(args) < 4 else "") + "sp"
    logl = ", ".join(["0x%X" % tag] + py_args)
    if rk == "void":
        body = "self.log.extend([%s])" % logl
        py = "st(0x%s, lambda %s: %s)" % (c["addr"].upper().rjust(8, "0"), names, body)
    elif rk == "ptr":
        py = "st(0x%s, lambda %s: (self.log.extend([%s]), TMP)[1])" % (c["addr"].upper(), names, logl)
    else:
        py = "st(0x%s, lambda %s: (self.log.extend([%s]), %s%s)[1])" % (
            c["addr"].upper(), names, logl, py_val, py_ret)
    if rk == "ptr":
        rs = None
    else:
        ret_sig = "" if rk == "void" else " -> %s" % rrust
        sig = "fn %s(&mut self%s)%s" % (rust_name(c), "".join(", " + s for s in rs_sig), ret_sig)
        lg = "self.log.extend([%s]);" % ", ".join(["0x%X" % tag] + rs_args)
        if rk == "void":
            body = [lg]
        elif rk == "bool":
            body = [lg, "%s & 1 != 0" % rs_val]
        elif rk in ("u8", "u16"):
            body = [lg, "(%s & %s) as %s" % (rs_val, "0xFF" if rk == "u8" else "0xFFFF", rrust)]
        elif rk == "i32":
            body = [lg, "%s as i32" % rs_val]
        else:
            body = [lg, rs_val]
        rs = (sig, body)
    return {"py": py, "rs": rs, "flags": flags, "tag": tag, "ret": ret_t, "args": args}


def main(argv):
    a = argv[1:]
    setname, addr, tag_base, packet = "battle", None, None, None
    i = 0
    while i < len(a):
        if a[i] == "--set":
            setname = a[i + 1]
            i += 2
        elif a[i] == "--tag-base":
            tag_base = int(a[i + 1], 0)
            i += 2
        elif a[i] == "--packet":
            packet = a[i + 1]
            i += 2
        elif a[i] in ("-h", "--help"):
            print(__doc__)
            return 0
        else:
            addr = "%08x" % (int(a[i].lower().replace("0x", ""), 16) & ~1)
            i += 1
    if not addr:
        print(__doc__)
        return 2
    callees = parse_callees(packet_text(setname, addr, packet))
    tag = tag_base if tag_base is not None else max(used_tags() | {0xFF}) + 1
    todo, skipped = [], []
    for c in callees:
        r = c["rust"]
        if r.startswith("ported") or r.startswith("runtime"):
            skipped.append((c, "real code under the emulator (%s)" % r.split(":")[0]))
        elif r.startswith("trait stub") and "flow.rs" not in r:
            skipped.append((c, "trait stub outside FlowHost (%s): real code under the emulator" % r.split("(")[-1].rstrip(")").split(":")[0]))
        elif r.startswith("trait stub") and in_testkit(rust_name(c)):
            skipped.append((c, "FlowHost method %s already in testkit.rs" % rust_name(c)))
        elif harnessed(c["addr"]):
            skipped.append((c, "already stubbed in handlers.py/bpp.py"))
        elif r.startswith("unresolved"):
            skipped.append((c, "unresolved (indirect call or data): model by hand"))
        else:
            todo.append(c)
    out = []
    out.append("# GENERATED by tools/rom-diff/genstub.py %s --set %s -- do not edit" % (addr, setname))
    out.append("# tags start at 0x%X (highest used in handlers.py/testkit.rs + 1); coordinator re-bases with --tag-base on merge" % tag)
    for c, why in skipped:
        out.append("# skip %s %s: %s" % (c["addr"], c["name"], why))
    gens = []
    for c in todo:
        gens.append((c, gen(c, tag)))
        tag += 1
    out.append("")
    out.append("# --- Python: add inside Host.__init__ (tools/rom-diff/handlers.py), `st = e.stub`")
    for c, g in gens:
        note = " -- " + "; ".join(g["flags"]) if g["flags"] else ""
        out.append("# GENERATED %s %s: %s%s" % (c["addr"], c["name"], c["proto"], ("  NEEDS-HAND" + note) if g["flags"] else ""))
        out.append(g["py"])
    if not gens:
        out.append("# (no new stubs needed)")
    out.append("")
    out.append("// --- Rust: declare in trait FlowHost (crates/black-battle/src/flow.rs)")
    for c, g in gens:
        if g["rs"]:
            out.append("/// C: `%s` (0x%s). GENERATED by genstub.py." % (c["name"], c["addr"].upper()))
            out.append(g["rs"][0] + ";")
        else:
            out.append("// NEEDS-HAND %s: %s" % (c["addr"], "; ".join(g["flags"])))
    out.append("")
    out.append("// --- Rust: implement in `impl FlowHost for TestHost` (crates/black-battle/src/handlers/testkit.rs)")
    for c, g in gens:
        if g["rs"]:
            out.append("// GENERATED %s tag 0x%X" % (c["addr"], g["tag"]))
            out.append(g["rs"][0] + " {")
            out.extend("    " + b for b in g["rs"][1])
            out.append("}")
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
