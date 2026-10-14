#!/usr/bin/env python3
"""Draft svflow spec generator (docs/migration/checklist/B.md, B10h).

usage: python3 -I tools/rom-diff/genspec.py <addr>... [--set battle] [--area NAME] [--packets-dir DIR]

Reads the work packet of each address (default /tmp/blk/packets/<set>/<addr>.md, built with
tools/re/packet.py if missing) and prints a draft `fn` block of tools/rom-diff/svflow_specs/AREA.spec:
  - `ret=` from the Ghidra return type,
  - one `arg` per parameter: integer types -> `u8|u16|u32|s16 any`; a first integer parameter that is
    dereferenced at fixed offsets -> `arg work` (guess); pointer parameters -> `arg buf N` sized from the
    largest offset used (guess),
  - one `stub` per callee that is not ported and not a runtime helper (scalar -> `v`, pointer -> `p`;
    `noreturn` when the pseudo-C marks the call as "does not return").
Everything uncertain is tagged `NEEDS-HAND`; the harness job checks it against the disassembly
(`rizin -qc 'aa; pdf @ 0xADDR' /tmp/blk/re/battle/battle.elf`) and tightens inputs with `fix`/`near`/`range`
lines before the spec is accepted. Ghidra prototypes are often wrong (register-passed extra args, varargs
prologues such as `push {r0-r3}`), so argument counts of callees are flagged VERIFY.
Pure Python 3 stdlib.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import genstub as G  # noqa: E402


def split_pseudo(text):
    m = re.search(r"```c\n(.*?)```", text, re.S)
    return m.group(1) if m else ""


def header(code):
    """Collapsed function header (everything before the first `{`)."""
    h = code.split("{", 1)[0]
    h = re.sub(r"/\*.*?\*/", " ", h, flags=re.S)
    return re.sub(r"\s+", " ", h).strip()


def ret_kind(ret_t):
    k, _ = G.kind(ret_t)
    if re.search(r"undefined8|long long|\b[su]64\b", ret_t):
        return "u64"
    return {"void": "none", "ptr": "u32", "bool": "bool", "u8": "u8", "u16": "u16", "i32": "i32"}.get(k, "u32")


def max_offset(code, name):
    """Largest byte offset used through parameter `name` (0 if only dereferenced as *p)."""
    best = 0
    for m in re.finditer(r"\b%s\s*\+\s*(0x[0-9a-fA-F]+|\d+)\b" % re.escape(name), code):
        best = max(best, int(m.group(1), 0))
    for m in re.finditer(r"\b%s\[(0x[0-9a-fA-F]+|\d+)\]" % re.escape(name), code):
        best = max(best, 4 * int(m.group(1), 0))
    return best


def used_as_base(code, nm):
    """True if integer parameter `nm` is used as a base address (`nm + x`, `*(T *)(nm + x)`)."""
    return re.search(r"\b%s\s*\+" % re.escape(nm), code) is not None


def arg_lines(args, code):
    out = []
    for i, (ct, nm) in enumerate(args):
        k, _ = G.kind(ct)
        if k == "ptr":
            n = (max_offset(code, nm) + 8 + 3) // 4 * 4
            out.append("  arg buf 0x%X   # NEEDS-HAND: pointer parameter %s (%s); size/layout guessed, add `fix` lines" % (max(n, 16), nm, ct))
        elif k in ("u32", "i32") and used_as_base(code, nm):
            if i == 0:
                out.append("  arg work   # NEEDS-HAND: guess, %s is used as a base address (work pointer? a sub-struct needs `arg work OFS`)" % nm)
            else:
                n = (max_offset(code, nm) + 8 + 3) // 4 * 4
                out.append("  arg buf 0x%X   # NEEDS-HAND: guess, %s is used as a base address; size/layout guessed, add `fix` lines" % (max(n, 0x100), nm))
        else:
            ty = {"u8": "u8", "bool": "u8", "u16": "u16", "i32": "s32"}.get(k, "u32")
            gen = "range 0 1" if k == "bool" else "any"
            out.append("  arg %s %s" % (ty, gen))
    return out


def stub_lines(callees, code):
    out, skipped = [], []
    for c in callees:
        r = c["rust"]
        if r.startswith("ported") or r.startswith("runtime"):
            skipped.append("# skip %s %s: runs as real code (%s)" % (c["addr"], c["name"], r.split(":")[0]))
            continue
        if r.startswith("unresolved"):
            skipped.append("# skip %s %s: unresolved (indirect call or data): model by hand" % (c["addr"], c["name"]))
            continue
        ret_t, args, guessed = G.parse_proto(c["proto"])
        site = re.search(r"%s\s*\(([^;]*?)\)\s*;" % re.escape(c["name"]), code, re.S)
        n_site = len([x for x in site.group(1).split(",") if x.strip()]) if site else None
        if guessed or (not args and n_site):
            kinds = ["v"] * (n_site or 0)
            note = "VERIFY: prototype unknown, %s args at the first call site" % n_site
        else:
            kinds = ["p" if G.kind(t)[0] == "ptr" else "v" for t, _ in args]
            note = "VERIFY arg count (prototype has %d%s)" % (len(args), "" if n_site is None else ", call site passes %d" % n_site)
        nr = " noreturn" if re.search(r"does not return \*/\s*%s\s*\(" % re.escape(c["name"]), code) else ""
        rk = ret_kind(ret_t)
        notes = [note]
        if any(G.kind(t)[0] == "ptr" for t, _ in args):
            notes.append("pointer args are ignored by the stub; add out:N / in:N if the callee writes/reads through one")
        if G.kind(ret_t)[0] == "ptr":
            notes.append("pointer return not modelled")
        out.append("  stub %s%s ret=%s args=%s   # %s: %s" % (c["addr"], nr, rk, ",".join(kinds), "NEEDS-HAND", "; ".join(notes)))
    return out, skipped


def spec_for(addr, setname, packets_dir):
    path = os.path.join(packets_dir, setname, addr + ".md") if packets_dir else None
    text = G.packet_text(setname, addr, path if path and os.path.exists(path) else None)
    code = split_pseudo(text)
    ret_t, args, guessed = G.parse_proto(header(code))
    lines = []
    if guessed:
        lines.append("# NEEDS-HAND: header not parsed, arguments are a 4 x undefined4 placeholder")
    lines.append("fn %s ret=%s   # NEEDS-HAND: argument list is Ghidra's prototype; check the register args in the disassembly" % (addr, ret_kind(ret_t)))
    lines += arg_lines(args, code)
    st, skipped = stub_lines(G.parse_callees(text), code)
    lines += st
    lines += ["  " + s for s in skipped]
    return "\n".join(lines)


def main(argv):
    a = argv[1:]
    setname, area, packets_dir, addrs = "battle", None, None, []
    i = 0
    while i < len(a):
        if a[i] == "--set":
            setname = a[i + 1]
            i += 2
        elif a[i] == "--area":
            area = a[i + 1]
            i += 2
        elif a[i] == "--packets-dir":
            packets_dir = a[i + 1]
            i += 2
        elif a[i] in ("-h", "--help"):
            print(__doc__)
            return 0
        else:
            addrs.append("%08x" % (int(a[i].lower().replace("0x", ""), 16) & ~1))
            i += 1
    if not addrs:
        print(__doc__)
        return 2
    if area:
        print("area %s\n" % area)
    for ad in addrs:
        print(spec_for(ad, setname, packets_dir))
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
