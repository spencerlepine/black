#!/usr/bin/env python3
"""F1: split the field overlays (10, 11, 18, 20, 21) into source-file areas.

Mechanical part of F1-SPLIT.md. Everything here is computed from the ROM overlay bytes and the
Ghidra index; the judgement labels (logic / render / out of scope, which F/G/U item) live in
tools/re/F1-LABELS.tsv and are joined by source-file name (the script prints unlabeled areas).

Inputs (read-only):
  $BLACK_ROM_BUILD/OVY_<n>.sbin           overlay bytes (default: main checkout build)
  $RE_OUT/field/pc/index.jsonl, pc/*.c    Ghidra index + pseudo-C of set `field` (main 10 11 18 20 21)
  tools/re/T1-TRIAGE.tsv                  ARM9 address bands -> subsystem (render vs logic callees)

Method
  1. Debug-name anchors. Each overlay carries a table of `*.c` names (arguments of Heap_AllocDebug /
     assert calls). A Thumb `ldr rX,[pc,#imm]` whose literal-pool word points at one marks a function
     of that source file; anchors in address order give "file runs" (files link in source order).
  2. Cuts between two runs: the function boundary between the last function of run A and the first of
     run B that minimises call edges crossing it (calls to hubs, fns with >= HUB_CALLERS callers, ignored).
  3. Script-command table. The longest run of {0, odd code pointer} words in OVY_10 data is the scrcmd
     table (index = command id). Handlers are clustered by address (<= CLUSTER_GAP fns apart, >= CLUSTER_MIN
     handlers) and extended over static helpers (fns whose callers are all inside the cluster). Each
     cluster is cut out of the anchored run it sits in and reported as `scrcmd[lo-hi]`.
  4. Per function: render score = share of main callees that sit in a render band of T1-TRIAGE
     (gflib graphics, NNS, MCSS, sound, UI toolkit, SDK, ICA, input), MMIO constants in the pseudo-C,
     and DS-only names. A function is `render` when the score is >= 0.5 with >= 2 main callees, or it
     touches MMIO; else `logic`. Area class = majority class by function count (labels override).
  5. Output: <out>/areas.tsv, <out>/fns.tsv, <out>/anchors.tsv and a text summary (or `--md` rows).

usage:  f1split.py [--out DIR] [--md]       DIR default $BLK_OUT/f1 (/tmp/blk/out-F1/f1)
"""
import array
import bisect
import collections
import json
import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD = os.environ.get(
    "BLACK_ROM_BUILD",
    os.path.expanduser("~/workplace/black/black-rom/build/black.us"),
)
RE_OUT = os.environ.get("RE_OUT", "/tmp/blk/re")
OVL = {10: 0x02155160, 11: 0x02172BC0, 18: 0x0217AE20, 20: 0x02183B60, 21: 0x02187F00}
OVL_END = {10: 0x02172BC0, 11: 0x0217AE20, 18: 0x02183B60, 20: 0x02187F00, 21: 0x021EE740}
HUB_CALLERS = 25
CLUSTER_GAP = 14
CLUSTER_MIN = 8
RENDER_SUBS = ("gflib graphics", "NNS", "MCSS", "sound", "UI toolkit", "NitroSDK", "ICA", "input", "gflib 3D")
NAME_RX = re.compile(rb"[A-Za-z0-9_]{3,}\.[ch]\x00")
MMIO_RX = re.compile(r"\b0x0?[46][0-9a-fA-F]{6}\b|DAT_0[46]0")


def load_index():
    rows = [json.loads(l) for l in open(os.path.join(RE_OUT, "field", "pc", "index.jsonl"))]
    rows.sort(key=lambda r: r["addr"])
    for r in rows:
        r["a"] = int(r["addr"], 16)
    by_name = {r["name"]: r["a"] for r in rows}
    for r in rows:
        r["callee_a"] = sorted({by_name[c] for c in r["callees"] if c in by_name})
        for c in r["callees"]:
            if c not in by_name:
                m = re.search(r"([0-9A-Fa-f]{8})$", c)
                if m:
                    r["callee_a"].append(int(m.group(1), 16))
    return rows


def load_bands():
    out = []
    for l in open(os.path.join(HERE, "T1-TRIAGE.tsv")):
        c = l.rstrip("\n").split("\t")
        if len(c) > 12 and c[1] == "arm9":
            out.append((int(c[4], 16), int(c[5], 16), c[11]))
    out.sort()
    return out


def band_of(bands, a):
    k = bisect.bisect_right([b[0] for b in bands], a) - 1
    if k >= 0 and a < bands[k][1]:
        return bands[k][2]
    return None


def find_anchors(n, starts):
    """[(use_addr, source_name, pool_addr, fn_start)] for each ldr of a `*.c` name pointer."""
    base = OVL[n]
    d = open(os.path.join(BUILD, "OVY_%d.sbin" % n), "rb").read()
    names = {base + m.start(): m.group()[:-1].decode() for m in NAME_RX.finditer(d)}
    w = array.array("I")
    w.frombytes(d[: len(d) // 4 * 4])
    pool = {}
    for i, v in enumerate(w):
        if v in names:
            pool[base + 4 * i] = names[v]
    out = []
    for p, nm in sorted(pool.items()):
        users = [
            ia
            for ia in range(max(base, p - 1024), p, 2)
            if struct.unpack_from("<H", d, ia - base)[0] >> 11 == 9
            and ((ia + 4) & ~3) + (struct.unpack_from("<H", d, ia - base)[0] & 0xFF) * 4 == p
        ]
        ua = users[-1] if users else p
        k = bisect.bisect_right(starts, ua) - 1
        out.append((ua, nm, p, starts[k] if k >= 0 else 0))
    return out


def find_cmd_table():
    """Longest run of {0, odd code pointer} words in OVY_10 (>= 300 entries) = scrcmd table."""
    base = OVL[10]
    d = open(os.path.join(BUILD, "OVY_10.sbin"), "rb").read()
    w = array.array("I")
    w.frombytes(d[: len(d) // 4 * 4])

    def ok(v):
        return v == 0 or ((v & 1) and OVL[10] <= v < 0x021F8000)

    best = (0, 0)
    i = 0
    while i < len(w):
        if not ok(w[i]) or w[i] == 0:
            i += 1
            continue
        j = i
        while j < len(w) and ok(w[j]):
            j += 1
        while j > i and w[j - 1] == 0:
            j -= 1
        if j - i > best[1] - best[0]:
            best = (i, j)
        i = j + 1
    i, j = best
    return base + 4 * i, {w[k] & ~1: k - i for k in range(i, j) if w[k]}


def hubs_of(fns, thr):
    cnt = collections.Counter(a for r in fns for a in set(r["callee_a"]))
    return {a for a, c in cnt.items() if c >= thr}


def min_cut(lo, hi, edges):
    """Index c in (lo, hi] minimising edges crossing between [lo, c) and [c, hi]; ties: later c."""
    best = None
    for c in range(lo + 1, hi + 1):
        cross = sum(1 for i in range(lo, c) for j in edges.get(i, ()) if c <= j <= hi)
        key = (cross, -c)
        if best is None or key < best[0]:
            best = (key, c)
    return best[1]


def subcuts(lo, hi, edges, min_side=15, win=12, max_cross=2):
    """Natural cut points inside [lo, hi): local minima of crossing call edges (hubs excluded)."""
    diff = collections.Counter()
    for i in range(lo, hi):
        for j in edges.get(i, ()):
            if i < j < hi:
                diff[i + 1] += 1
                diff[j + 1] -= 1
    cross, run = {}, 0
    for c in range(lo + 1, hi):
        run += diff[c]
        cross[c] = run
    out = []
    for c in range(lo + min_side, hi - min_side):
        v = cross[c]
        if v <= max_cross and all(v <= cross.get(k, 99) for k in range(c - win, c + win + 1)):
            if not out or c - out[-1][0] >= win:
                out.append((c, v))
    return out


def clusters_for(fns, idx_of, cmd, callers_in):
    hs = sorted(idx_of[a] for a in cmd if a in idx_of)
    groups = []
    for i in hs:
        if groups and i - groups[-1][-1] <= CLUSTER_GAP:
            groups[-1].append(i)
        else:
            groups.append([i])
    out = []
    for g in groups:
        if len(g) < CLUSTER_MIN:
            continue
        lo, hi = g[0], g[-1]
        inside = lambda i: lo <= i <= hi
        while hi + 1 < len(fns):
            cs = callers_in.get(hi + 1, set())
            if cs and all(inside(c) or c == hi + 1 for c in cs):
                hi += 1
            else:
                break
        while lo - 1 >= 0:
            cs = callers_in.get(lo - 1, set())
            if cs and all(inside(c) or c == lo - 1 for c in cs):
                lo -= 1
            else:
                break
        out.append((lo, hi, len(g)))
    return out


ARC_RX = re.compile(r"FUN_(020490F4|0204A1BC|02048F60|02048F74|020490A8|02048F4C|02045B38)\(([^;]*?)\);")
ARC_FNS = {}


def arcs_in(text):
    """ARC ids / message banks passed as literals to the ROM loaders (see T1 'ARC users')."""
    out = set()
    for m in ARC_RX.finditer(text):
        args = [a.strip() for a in m.group(2).split(",")]
        lit = lambda x: int(x, 0) if re.fullmatch(r"0x[0-9a-fA-F]+|\d+", x) else None
        f = m.group(1).upper()
        if f == "02048F4C" and len(args) > 1 and lit(args[1]) is not None:
            out.add("arc%d" % lit(args[1]))
        elif f == "02045B38" and len(args) > 2 and lit(args[2]) is not None:
            out.add("msg%d" % lit(args[2]))
        elif f != "02048F4C" and args and lit(args[0]) is not None:
            out.add("arc%d" % lit(args[0]))
    return out


def classify_fns(fns, bands, pcdir):
    res = {}
    for r in fns:
        main_callees = [a for a in r["callee_a"] if a < OVL[10]]
        ren = sum(1 for a in main_callees if (band_of(bands, a) or "").startswith(RENDER_SUBS))
        score = ren / len(main_callees) if main_callees else 0.0
        mmio = False
        p = os.path.join(pcdir, "%s_%s.c" % (r["addr"], r["name"]))
        if os.path.exists(p):
            txt = open(p, errors="replace").read()
            mmio = bool(MMIO_RX.search(txt))
            ARC_FNS[r["a"]] = arcs_in(txt)
        seed = mmio or (score >= 0.5 and len(main_callees) >= 2)
        res[r["a"]] = ["render" if seed else "logic", score, len(main_callees), mmio]
    # propagate: a fn whose callees are mostly render (and >= 2) is render too (monotonic, to a fixed point)
    changed = True
    while changed:
        changed = False
        for r in fns:
            if res[r["a"]][0] == "render" or len(r["callee_a"]) < 2:
                continue
            ren = sum(1 for a in r["callee_a"] if res.get(a, ["logic"])[0] == "render")
            if ren * 2 >= len(r["callee_a"]):
                res[r["a"]][0] = "render"
                changed = True
    return {a: tuple(v) for a, v in res.items()}


def split_overlay(n, rows, anchors, cmd):
    fns = [r for r in rows if OVL[n] <= r["a"] < OVL_END[n]]
    addrs = [r["a"] for r in fns]
    idx_of = {a: i for i, a in enumerate(addrs)}
    hubs = hubs_of(fns, HUB_CALLERS)
    edges = collections.defaultdict(set)
    callers_in = collections.defaultdict(set)
    for i, r in enumerate(fns):
        for a in r["callee_a"]:
            j = idx_of.get(a)
            if j is not None and j != i:
                callers_in[j].add(i)
                if a not in hubs:
                    edges[i].add(j)
                    edges[j].add(i)
    pts = []
    for ua, nm, p, fs in sorted(anchors):
        pts.append((idx_of.get(fs, max(bisect.bisect_right(addrs, fs) - 1, 0)), nm))
    runs = []
    for i, nm in pts:
        if runs and runs[-1][0] == nm:
            runs[-1][2] = max(runs[-1][2], i)
        else:
            runs.append([nm, i, i])
    cut = [0]
    for a, b in zip(runs, runs[1:]):
        lo, hi = a[2], b[1]
        cut.append(min_cut(lo, hi, edges) if hi > lo else max(hi, cut[-1] + 1))
    cut.append(len(fns))
    seg = []  # (lo, hi_excl, name, kind)
    for k, r in enumerate(runs):
        seg.append([cut[k] if k else 0, cut[k + 1], r[0], "anchor"])
    # scrcmd clusters carve out of the run they sit in
    cl = clusters_for(fns, idx_of, cmd, callers_in) if n in (10, 21) else []
    for lo, hi, nh in cl:
        new = []
        for s in seg:
            a, b, nm, kd = s
            if kd in ("anchor", "tail") and not nm.startswith("scrcmd_") and a <= hi and lo < b:
                if a < lo:
                    new.append([a, lo, nm, kd])
                new.append([max(a, lo), min(b, hi + 1), "scrcmd (unanchored)", "cluster"])
                if hi + 1 < b:
                    new.append([hi + 1, b, nm if nm.endswith("+") else nm + "+", "tail"])
            else:
                new.append(s)
        seg = new
    seg.sort()
    # drop empties, merge touching same-name
    merged = []
    for s in seg:
        if s[1] <= s[0]:
            continue
        if merged and merged[-1][2] == s[2] and merged[-1][1] == s[0]:
            merged[-1][1] = s[1]
        else:
            merged.append(s)
    return fns, merged, hubs, edges


def main():
    out_dir = None
    if "--out" in sys.argv:
        out_dir = sys.argv[sys.argv.index("--out") + 1]
    out_dir = out_dir or os.path.join(os.environ.get("BLK_OUT", "/tmp/blk/out-F1"), "f1")
    os.makedirs(out_dir, exist_ok=True)
    rows = load_index()
    starts = [r["a"] for r in rows]
    bands = load_bands()
    tbl_addr, cmd = find_cmd_table()
    print("scrcmd table at %08x: %d entries" % (tbl_addr, len(set(cmd.values()))))
    pcdir = os.path.join(RE_OUT, "field", "pc")
    by_addr = {r["a"]: r for r in rows}
    ext_callers = collections.defaultdict(set)
    for r in rows:
        for a in r["callee_a"]:
            ext_callers[a].add(r["a"])
    fa = open(os.path.join(out_dir, "areas.tsv"), "w")
    ff = open(os.path.join(out_dir, "fns.tsv"), "w")
    fa.write("area\tovl\tlo\thi\tfns\tbytes\tsource\tkind\tanchors\tcmds\tcmd_range\trender\tmmio\text_in\thubs\tentries\tsubcuts\tarcs\n")
    ff.write("addr\tarea\tclass\trender_score\tmain_callees\tmmio\tcmd\tcallers\n")
    an_all = open(os.path.join(out_dir, "anchors.tsv"), "w")
    outrows = []
    cls = classify_fns([r for r in rows if OVL[10] <= r["a"] < OVL_END[21]], bands, pcdir)
    for n in OVL:
        an = find_anchors(n, starts)
        for ua, nm, p, fs in an:
            an_all.write("%d\t%08x\t%s\t%08x\t%08x\n" % (n, ua, nm, p, fs))
        fns, segs, hubs, edges = split_overlay(n, rows, an, cmd)
        anch_count = collections.Counter(nm for _, nm, _, _ in an)
        for si, (lo, hi, nm, kd) in enumerate(segs):
            aid = "%d.%02d" % (n, si)
            sub = fns[lo:hi]
            inset = {r["a"] for r in sub}
            cmds = [cmd[r["a"]] for r in sub if r["a"] in cmd]
            ncls = collections.Counter(cls[r["a"]][0] for r in sub)
            nmmio = sum(1 for r in sub if cls[r["a"]][3])
            ext = collections.Counter()
            for r in sub:
                for c in ext_callers[r["a"]]:
                    if c not in inset:
                        ext[r["a"]] += 1
            ext_in = sum(1 for r in sub if ext[r["a"]])
            hub = [(a, c) for a, c in ext.most_common(4)]
            arcs = collections.Counter(x for r in sub for x in ARC_FNS.get(r["a"], ()))
            ent = sorted(r["a"] for r in sub if (not ext_callers[r["a"]]) and r["a"] not in cmd)
            size = sum(r["size"] for r in sub)
            sc = subcuts(lo, hi, edges) if len(sub) >= 100 else []
            sc_s = " ".join("%08x/%d" % (fns[c]["a"], v) for c, v in sc)
            fa.write(
                "\t".join(
                    [
                        aid,
                        str(n),
                        "%08x" % sub[0]["a"],
                        "%08x" % (sub[-1]["a"] + sub[-1]["size"]),
                        str(len(sub)),
                        str(size),
                        nm,
                        kd,
                        str(anch_count.get(nm.rstrip("+"), 0) if kd != "cluster" else 0),
                        str(len(cmds)),
                        ("%d-%d" % (min(cmds), max(cmds))) if cmds else "",
                        str(ncls["render"]),
                        str(nmmio),
                        str(ext_in),
                        " ".join("%08x:%d" % h for h in hub),
                        str(len(ent)),
                        sc_s,
                        " ".join("%s:%d" % kv for kv in sorted(arcs.items())),
                    ]
                )
                + "\n"
            )
            for r in sub:
                c = cls[r["a"]]
                ff.write(
                    "%s\t%s\t%s\t%.2f\t%d\t%d\t%s\t%d\n"
                    % (r["addr"], aid, c[0], c[1], c[2], c[3], cmd.get(r["a"], ""), len(ext_callers[r["a"]]))
                )
            outrows.append(
                dict(
                    id=aid, ovl=n, lo=sub[0]["a"], hi=sub[-1]["a"] + sub[-1]["size"], fns=len(sub), bytes=size,
                    src=nm, kind=kd, anchors=anch_count.get(nm.rstrip("+"), 0) if kd != "cluster" else 0,
                    arcs=arcs, cmds=cmds, render=ncls["render"], mmio=nmmio, ext_in=ext_in, hubs=hub, subcuts=sc_s,
                )
            )
    for f in (fa, ff, an_all):
        f.close()
    labels = load_labels()
    if "--md" in sys.argv:
        print_md(outrows, labels)
        print()
        print_hubs(rows, outrows)
        print()
        print_gaps(rows)
        return
    miss = sorted({r["src"].rstrip("+") for r in outrows if r["src"].rstrip("+") not in labels})
    if miss:
        print("UNLABELED sources:", ", ".join(miss))
    print("%-6s %-8s %-8s %5s  %-30s %-7s %5s %s" % ("area", "lo", "hi", "fns", "source", "kind", "cmds", "render/mmio"))
    for r in outrows:
        print("%-6s %08x %08x %5d  %-30s %-7s %5d %d" % (r["id"], r["lo"], r["hi"], r["fns"], r["src"], r["kind"], len(r["cmds"]), r["render"]))


def load_labels():
    """F1-LABELS.tsv: source, group, class, feeds, what (class: logic|render|mixed|oos)."""
    out = {}
    p = os.path.join(HERE, "F1-LABELS.tsv")
    if os.path.exists(p):
        for l in open(p):
            c = l.rstrip("\n").split("\t")
            if len(c) >= 5 and not l.startswith("#"):
                out[c[0]] = dict(group=c[1] or c[0], cls=c[2], feeds=c[3], what=c[4])
    return out


def print_md(outrows, labels):
    """Merge adjacent areas that share a label group, then print the per-area markdown table."""
    merged = []
    for r in outrows:
        lb = labels.get(r["src"].rstrip("+"), dict(group=r["src"], cls="?", feeds="?", what=""))
        g = lb["group"] if r["kind"] != "cluster" else "scrcmd (unanchored)"
        if merged and merged[-1]["group"] == g and merged[-1]["ovl"] == r["ovl"] and g != "scrcmd (unanchored)":
            m = merged[-1]
            m["hi"], m["fns"], m["bytes"] = r["hi"], m["fns"] + r["fns"], m["bytes"] + r["bytes"]
            m["cmds"] += r["cmds"]; m["render"] += r["render"]; m["mmio"] += r["mmio"]
            m["anchors"] += r["anchors"]; m["hubs"] += r["hubs"]; m["arcs"] = m["arcs"] + r["arcs"]
            if r["src"] not in m["srcs"]:
                m["srcs"].append(r["src"])
        else:
            m = dict(r); m["group"] = g; m["srcs"] = [r["src"]]; m["lb"] = lb
            merged.append(m)
    tot = collections.Counter()
    feeds = collections.Counter()
    logic_est = collections.Counter()
    for m in merged:
        tot[m["lb"]["cls"]] += m["fns"]
        feeds[m["lb"]["feeds"]] += m["fns"]
        logic_est[m["lb"]["feeds"]] += m["fns"] - m["render"]
    print("class totals (fns):", dict(tot))
    print("by feeding item (fns):", dict(sorted(feeds.items())))
    print("same, minus fns classified render:", dict(sorted(logic_est.items())))
    print()
    print("| area | range | fns | source | evidence | class | key entry/hub fns | feeds |")
    print("|---|---|---|---|---|---|---|---|")
    for m in merged:
        ev = []
        if m["anchors"]:
            ev.append("%d name-anchors" % m["anchors"])
        if m["cmds"]:
            ev.append("%d scrcmd handlers" % len(m["cmds"]))
        ev.append("render %d/%d" % (m["render"], m["fns"]))
        if m["arcs"]:
            ev.append("loads " + ",".join(k for k, _ in m["arcs"].most_common(6)))
        hubs = sorted(m["hubs"], key=lambda h: -h[1])[:3]
        print("| %s | %08X-%08X | %d | %s | %s | %s | %s | %s |" % (
            m["id"], m["lo"], m["hi"], m["fns"], ", ".join(x.rstrip("+") if i == 0 else x for i, x in enumerate(m["srcs"])) if len(m["srcs"]) < 3 else m["group"],
            "; ".join(ev), m["lb"]["cls"], " ".join("%08x(%d)" % h for h in hubs), m["lb"]["feeds"]))


def print_hubs(rows, outrows, n=25):
    """Most-called functions of the field set (callers counted over all fns incl. main) with their area."""
    cnt = collections.Counter(a for r in rows for a in set(r["callee_a"]))
    own = {}
    for o in outrows:
        for a in range(0):
            pass
    print("| fn | callers | area |")
    print("|---|---|---|")
    n_ = 0
    for a, c in cnt.most_common(400):
        if not (OVL[10] <= a < OVL_END[21]):
            continue
        area = next((o for o in outrows if o["lo"] <= a < o["hi"]), None)
        print("| %08X | %d | %s |" % (a, c, "%s %s" % (area["id"], area["src"]) if area else "?"))
        n_ += 1
        if n_ >= n:
            break


def print_gaps(rows):
    """Non-code gaps (>= 512 B between Ghidra functions) and the data tail of each overlay."""
    prev = None
    for r in rows:
        if r["a"] < OVL[10]:
            continue
        if prev is not None and r["a"] - prev >= 512:
            print("- %08X-%08X (%d B) no functions: rodata/tables/strings" % (prev, r["a"], r["a"] - prev))
        prev = max(prev or 0, r["a"] + r["size"])


if __name__ == "__main__":
    main()
