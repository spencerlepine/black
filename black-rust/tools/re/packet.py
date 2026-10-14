#!/usr/bin/env python3
"""Work packet for one function (docs/migration/ENV.md "RE workflow", Z8).

usage: python3 -I tools/re/packet.py <addr> [--set battle] [--out DIR]

Writes <DIR>/<addr>.md (default /tmp/blk/packets/<set>/) and prints the path. Contents: header,
typed pseudo-C, callee table (prototype, Rust item if ported / "trait stub"), cluster membership
with already-ported siblings as a template hint, and rizin disassembly only when the pseudo-C
has decompiler warnings. Shares its ported-set logic with tools/re/queue.py.
"""
import importlib.util
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("re_queue", os.path.join(HERE, "queue.py"))
Q = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(Q)

# harmless warnings listed in section 3b / seen on every typed handler; they do not trigger asm
HARMLESS = ("Unknown calling convention", "Globals starting with", "Subroutine does not return",
            "Treating indirect jump as call")
WARN = re.compile(r"WARNING|Could not recover|\bin_r\d|unaff_|Bad instruction")


def has_warnings(src):
    hits = []
    for ln in src.split("\n"):
        if WARN.search(ln) and not any(h in ln for h in HARMLESS):
            hits.append(ln.strip())
    return hits


def load_signatures(path):
    """({addr: proto}, {name: proto}, [(lo, hi, proto)])"""
    by_addr, by_name, pools = {}, {}, []
    try:
        fh = open(path)
    except IOError:
        return by_addr, by_name, pools
    with fh:
        for ln in fh:
            ln = ln.rstrip("\n")
            if not ln.strip() or ln.startswith("#"):
                continue
            f = ln.split("\t")
            if f[0] == "@pool" and len(f) >= 4:
                pools.append((int(f[1], 16), int(f[2], 16), f[3]))
            elif len(f) >= 2:
                k = f[0].strip()
                if re.match(r"^[0-9a-fA-F]{8}$", k):
                    by_addr[k.lower()] = f[1]
                else:
                    by_name[k] = f[1]
    return by_addr, by_name, pools


def proto_for(addr, name, sigs, ix):
    by_addr, by_name, pools = sigs
    if addr in by_addr:
        return by_addr[addr], "signatures.tsv"
    if name in by_name:
        return by_name[name], "signatures.tsv"
    a = int(addr, 16)
    for lo, hi, p in pools:
        if lo <= a < hi:
            return p, "signatures.tsv (@pool)"
    return None, None


def ghidra_header(ix, addr):
    r = ix.rows.get(addr)
    if not r:
        return ""
    path = os.path.join(ix.pc, "%s_%s.c" % (addr, r["name"]))
    try:
        parts = []
        with open(path, errors="replace") as fh:
            for ln in fh:
                t = ln.strip()
                if not t or t.startswith("/*"):
                    continue
                parts.append(t)
                if ")" in t or len(parts) > 6:
                    break
        return " ".join(parts)
    except IOError:
        pass
    return ""


def rust_ref(st, addr):
    r = st.rust.get(addr)
    if not r:
        return None
    return r


def fmt_rust(r):
    return "`%s` (%s:%d)" % (r["item"], r["file"], r["line"])


def disasm(ix, addr, size):
    elf = os.path.join(ix.dir, ix.setname + ".elf")
    try:
        out = subprocess.run(
            ["rizin", "-e", "scr.color=0", "-qc", "aa; pD %d @ 0x%s" % (size, addr), elf],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=120, universal_newlines=True)
        return out.stdout.rstrip()
    except (OSError, subprocess.SubprocessError) as e:
        return "(rizin failed: %s)" % e


def build(setname, addr):
    addr = ("%08x" % (int(addr, 16) & ~1))
    st = Q.Status(setname)
    ix = st.ix
    r = ix.rows.get(addr)
    if r is None:
        raise SystemExit("no function at %s in set %s" % (addr, setname))
    sigs = load_signatures(os.path.join(HERE, "signatures.tsv"))
    src_path = os.path.join(ix.pc, "%s_%s.c" % (addr, r["name"]))
    with open(src_path, errors="replace") as fh:
        pseudo = fh.read().rstrip()
    warns = has_warnings(pseudo)
    callers = sorted(set(ix.callers.get(addr, [])))
    of, members = ix.clusters()
    cid = of.get(addr)

    L = []
    me = st.rust.get(addr)
    L.append("# Packet %s %s" % (addr, r["name"]))
    L.append("")
    L.append("- set: `%s`  address: `0x%s`  size: %d bytes  decompile ok: %s" % (setname, addr, r["size"], r["ok"]))
    L.append("- status in Rust: %s" % (
        "**%s** %s" % (me["status"], fmt_rust(me)) if me else "not ported"))
    L.append("- callers: %d (index says %d)" % (len(callers), r["callers"]))
    for c in callers[:10]:
        cr = ix.rows[c]
        s = st.state(c)
        L.append("  - `%s` %s%s" % (c, cr["name"], " [%s]" % s if s else ""))
    if len(callers) > 10:
        L.append("  - ... %d more" % (len(callers) - 10))
    L.append("- cluster: %s" % ("#%d (%d members)" % (cid, len(members[cid])) if cid else "none"))
    L.append("")
    L.append("## Pseudo-C (Ghidra, a reading aid; only the differential test proves anything)")
    L.append("")
    L.append("```c")
    L.append(pseudo)
    L.append("```")
    L.append("")
    L.append("## Callees")
    L.append("")
    seen = set()
    rows = []
    for name, a in ix.callee_addrs(r):
        key = a or name
        if key in seen:
            continue
        seen.add(key)
        rows.append((name, a))
    if not rows:
        L.append("(none: leaf)")
    else:
        L.append("| addr | name | prototype | Rust |")
        L.append("|---|---|---|---|")
        for name, a in rows:
            if a is None:
                L.append("| ? | %s | | unresolved |" % name)
                continue
            rr = st.rust.get(a)
            p, src = proto_for(a, ix.rows[a]["name"], sigs, ix)
            if p is None:
                p = ghidra_header(ix, a)
                src = "ghidra"
            if rr and rr["status"] == "ported":
                rs = "ported: " + fmt_rust(rr)
            elif rr and rr["status"] == "stub":
                rs = "trait stub: " + fmt_rust(rr)
            elif rr:
                rs = "documented only: " + fmt_rust(rr)
            else:
                cl = st.callee_class(name, a, 0, 0)  # range-free: runtime or unported
                rs = "runtime helper" if cl == "runtime" else "NOT PORTED (add a trait method; never invent behaviour)"
            L.append("| `%s` | %s | `%s`%s | %s |" % (
                a, ix.rows[a]["name"], p.replace("|", "\\|"), "" if src == "ghidra" else " [%s]" % src, rs))
    L.append("")
    L.append("## Cluster")
    L.append("")
    if not cid:
        L.append("Not in a shape cluster (unique shape).")
    else:
        mem = members[cid]
        L.append("Cluster #%d: %d functions with the same shape (names and constants masked)." % (cid, len(mem)))
        done = [(m, st.rust[m]) for m in mem if m != addr and st.state(m) == "ported"]
        if done:
            L.append("")
            L.append("Already ported siblings (template hint; copy the pattern, change the constants):")
            for m, rr in done[:10]:
                L.append("- `%s` -> %s" % (m, fmt_rust(rr)))
            if len(done) > 10:
                L.append("- ... %d more" % (len(done) - 10))
        else:
            L.append("No sibling is ported yet: this one would become the template.")
        L.append("")
        shown = [m for m in mem if m != addr]
        L.append("Siblings (%d): %s%s" % (len(shown), " ".join(shown[:40]), " ..." if len(shown) > 40 else ""))
    L.append("")
    if warns:
        L.append("## Disassembly (rizin; the pseudo-C has decompiler warnings)")
        L.append("")
        L.append("Warnings:")
        for w in warns[:8]:
            L.append("- `%s`" % w)
        L.append("")
        L.append("```")
        L.append(disasm(ix, addr, r["size"]))
        L.append("```")
    else:
        L.append("## Disassembly")
        L.append("")
        L.append("Omitted: the pseudo-C has no decompiler warnings.")
    L.append("")
    return "\n".join(L), addr


def main(argv):
    a = argv[1:]
    setname, out, addr = "battle", None, None
    i = 0
    while i < len(a):
        if a[i] == "--set":
            setname = a[i + 1]
            i += 2
        elif a[i] == "--out":
            out = a[i + 1]
            i += 2
        elif a[i] in ("-h", "--help"):
            print(__doc__)
            return 0
        else:
            addr = a[i].lower().replace("0x", "")
            i += 1
    if not addr:
        print(__doc__)
        return 2
    text, addr = build(setname, addr)
    out = out or os.path.join(Q.PACKETS_OUT, setname)
    os.makedirs(out, exist_ok=True)
    path = os.path.join(out, addr + ".md")
    with open(path, "w") as fh:
        fh.write(text)
    print(path)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
