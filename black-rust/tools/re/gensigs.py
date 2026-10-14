#!/usr/bin/env python3
"""gensigs.py: generate candidate Ghidra signatures and the struct header (Z10).

  python3 -I tools/re/gensigs.py [--index /tmp/blk/re/battle/pc/index.jsonl]
                                 [--set "main 93 94 95 96"]

Writes (next to this script):
  signatures.generated.tsv       candidate lines, same format as signatures.tsv
  signatures.generated.report    what was skipped / conflicted, per-source counts
  types.h                        structs from black-rom/include + BTL_POKEPARAM layout
signatures.tsv itself is never touched (coordinator merges). Sources:
  include  prototypes in black-rom/include/*.h, symbol -> address via ndsdisasm_config/*.cfg
  rust     `/// C:` doc comments on Rust fns whose param/return types map unambiguously
  runtime  compiler/SDK helpers from docs/migration/NOTES.md "Asm reading notes"
Later sources never override earlier ones; disagreements are listed in the report.
"""
import argparse
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))  # black-rust
ROM = os.path.abspath(os.path.join(ROOT, "..", "black-rom"))

PRIM = {
    "u8": "unsigned char", "u16": "unsigned short", "u32": "unsigned int",
    "i8": "signed char", "i16": "short", "i32": "int",
    "bool": "int", "usize": "unsigned int",
}
CTYPE = {  # include/types.h typedefs -> canonical C (keeps the .tsv independent of types.h)
    "u8": "unsigned char", "u16": "unsigned short", "u32": "unsigned int", "u64": "unsigned long long",
    "s8": "signed char", "s16": "short", "s32": "int", "s64": "long long",
    "BOOL": "int", "bool8": "unsigned char", "bool32": "unsigned int",
}
STRUCT_PTR = {  # include/Rust struct -> type defined in types.h
    "BattleRecordReader": "BattleRecordReader", "BattleRecordBuilder": "BattleRecordBuilder",
    "UNK_021D845C": "UNK_021D845C",
}
OPAQUE = {"BattleRecord": "void"}

# section 7 "Asm reading notes" (the ones not already in signatures.tsv are new; the rest dedupe).
RUNTIME = [
    ("0209c2b0", "unsigned int _u32_div_f(unsigned int a, unsigned int b)"),
    ("0209c0a4", "int _s32_div_f(int a, int b)"),
    ("0209c054", "long long _ll_mul(long long a, long long b)"),
    ("0209b750", "double _i2d(int a)"),
    ("0209b790", "double _u2d(unsigned int a)"),
    ("02082d44", "void MI_CpuCopy8(void *src, void *dst, unsigned int n)"),
    ("02082bcc", "void MI_CpuFill8(void *dst, int v, unsigned int n)"),
    ("02030734", "void *Heap_AllocDebug(unsigned int heapID, unsigned int size, int unk2, const char *file, unsigned int line)"),
    ("020307b0", "void Heap_Free(void *ptr)"),
]

# BTL_POKEPARAM (0x1F8): offsets from crates/black-battle/src/battle_mon.rs (doc table + accessors).
BPP_SIZE = 0x1F8
BPP_FIELDS = [  # (offset, c decl)
    (0x000, "void *pp"), (0x004, "void *illusion_pp"), (0x008, "u32 exp"),
    (0x00C, "u16 species"), (0x00E, "u16 max_hp"), (0x010, "u16 hp"), (0x012, "u16 item"),
    (0x016, "u16 ability_loaded"), (0x018, "u8 level"), (0x019, "u8 poke_id"),
    (0x01A, "u8 personal_1"), (0x01B, "u8 form_flags"),
    (0x01C, "u32 sick_cont[36]"), (0x0AC, "u8 sick_turns[36]"),
    (0x0EE, "u16 stats[5]"),
    (0x0F8, "u8 type1"), (0x0F9, "u8 type2"), (0x0FA, "u8 pp_fa"),
    (0x0FC, "s8 stages[7]"),
    (0x104, "BtlMoveSlot moves[4]"),
    (0x13C, "u16 ability"), (0x13E, "u16 personal_26"),
    (0x140, "u8 move_count"), (0x141, "u8 form"), (0x142, "u8 crit_boost"), (0x143, "u8 moves_used"),
    (0x144, "u8 reset_144"),
    (0x146, "u16 field_146"), (0x14A, "u16 field_14a"), (0x14C, "u16 field_14c"), (0x14E, "u16 field_14e"),
    (0x153, "u8 flags_a[2]"), (0x155, "u8 flags_b[2]"), (0x157, "u8 flags_c[5]"),
]
BPP_FIELD_SIZE = {  # sizes in bytes for gap computation
    "void *": 4, "u32": 4, "u16": 2, "u8": 1, "s8": 1, "BtlMoveSlot": 14,
}
MOVE_SLOT = """typedef struct {
    u16 orig_id;
    u8 orig_pp;
    u8 orig_max_pp;
    u8 orig_pp_ups;
    u8 orig_flags;
    u16 cur_id;
    u8 cur_pp;
    u8 cur_max_pp;
    u8 cur_pp_ups;
    u8 cur_flags;
    u8 linked;
    u8 pad_0d;
} BtlMoveSlot;
"""


def decl_size(decl):
    m = re.match(r"(.+?)\s*(\*?)\s*(\w+)(?:\[(\d+)\])?$", decl)
    base, star, _, n = m.groups()
    sz = 4 if star else BPP_FIELD_SIZE[base.strip()]
    return sz * (int(n) if n else 1)


def bpp_struct():
    out, pos, gap = ["typedef struct {"], 0, 0
    for off, decl in BPP_FIELDS:
        assert off >= pos, (hex(off), hex(pos))
        if off > pos:
            out.append("    u8 unk_%03X[%d];" % (pos, off - pos))
        out.append("    %s; // +0x%03X" % (decl, off))
        pos = off + decl_size(decl)
    assert pos <= BPP_SIZE
    if pos < BPP_SIZE:
        out.append("    u8 unk_%03X[%d];" % (pos, BPP_SIZE - pos))
    out.append("} BTL_POKEPARAM; // 0x%X bytes" % BPP_SIZE)
    return "\n".join(out) + "\n"


def strip_comments(s):
    s = re.sub(r"/\*.*?\*/", "", s, flags=re.S)
    return re.sub(r"//[^\n]*", "", s)


def read(p):
    with open(p, encoding="utf-8", errors="replace") as f:
        return f.read()


def gen_types():
    inc = os.path.join(ROM, "include")
    parts = ["// GENERATED by tools/re/gensigs.py. Do not edit.\n"
             "// Parsed by ApplySignatures.java (Ghidra CParser). #pragma pack keeps offsets exact.\n"
             "#pragma pack(1)\n"]
    t = read(os.path.join(inc, "types.h"))
    parts.append("\n".join(l for l in strip_comments(t).splitlines()
                           if l.startswith("typedef") or l.startswith("#define TRUE")
                           or l.startswith("#define FALSE")) + "\n")
    for h in ("heap.h", "battle_common.h", "battle_record.h"):
        s = strip_comments(read(os.path.join(inc, h)))
        for m in re.finditer(r"^#define\s+\w+\s+\w+\s*$", s, flags=re.M):
            parts.append(m.group(0) + "\n")
        for m in re.finditer(r"typedef\s+(?:struct|enum)\s*\{.*?\}\s*\w+;", s, flags=re.S):
            parts.append(m.group(0) + "\n")
        for m in re.finditer(r"^enum\s+\w+\s*\{.*?\};", s, flags=re.S | re.M):
            parts.append(m.group(0) + "\n")
    parts.append(MOVE_SLOT)
    parts.append(bpp_struct())
    return "\n".join(parts)


def norm_c_type(t):
    t = re.sub(r"\s+", " ", t.strip())
    t = re.sub(r"\s*\*\s*", " *", t)
    t = re.sub(r"\*(?=\*)", "* ", t)
    return t.strip()


def canon_type(t):
    """include type text (no param name) -> canonical C type for the .tsv"""
    t = norm_c_type(t)
    t = re.sub(r"^enum \w+$", "unsigned int", t)
    star = ""
    while t.endswith("*"):
        star += "*"
        t = t[:-1].strip()
    const = ""
    if t.startswith("const "):
        const, t = "const ", t[6:]
    t = CTYPE.get(t, t)
    if t in OPAQUE:
        t = OPAQUE[t]
    return (const + t + (" " + star if star else "")).replace(" *", " *").strip().replace("* *", "**")


def split_param(p):
    p = norm_c_type(p)
    if p in ("void", ""):
        return None
    m = re.match(r"^(.*?[\s*])(\w+)$", p)
    if m and m.group(1).strip():
        ty, name = m.group(1), m.group(2)
        if ty.strip() in ("unsigned", "signed", "struct", "enum", "const"):  # unnamed
            return p, ""
        return ty, name
    return p, ""


def fmt_proto(ret, name, params):
    ps = ", ".join((t.strip() + ("" if t.rstrip().endswith("*") else " ") + n).strip() if n
                   else t.strip() for t, n in params) or "void"
    r = ret if not ret.endswith("*") else ret
    return "%s%s%s(%s)" % (r, "" if r.endswith("*") else " ", name, ps)


def norm_sig(proto):
    """comparable form: no names, no const"""
    m = re.match(r"^(.*?)\b(\w+)\s*\((.*)\)\s*$", proto)
    ret, args = m.group(1), m.group(3)
    out = [norm_c_type(ret).replace("const ", "")]
    for a in args.split(","):
        sp = split_param(a)
        if sp:
            out.append(norm_c_type(sp[0]).replace("const ", ""))
    return tuple(out)


def load_index(path):
    idx = {}
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            idx.setdefault(r["addr"].lower(), r["name"])
    return idx


def load_cfg(mods):
    names = {}
    files = []
    for m in mods:
        files.append(os.path.join(ROM, "ndsdisasm_config",
                                  "arm9_config.cfg" if m == "main" else "overlay_%s.cfg" % m))
    for p in files:
        if not os.path.exists(p):
            continue
        for l in read(p).splitlines():
            m = re.match(r"^(?:thumb_func|arm_func|arm_func_end)?\s*0x([0-9a-fA-F]+)\s+(\w+)", l)
            m = re.match(r"^(?:thumb_func|arm_func)\s+0x([0-9a-fA-F]+)\s+(\w+)", l)
            if m:
                names.setdefault(m.group(2), "%08x" % int(m.group(1), 16))
    return names


def hex_in(s):
    m = re.search(r"(?:0x|_)([0-9a-fA-F]{8})\b", s)
    return m.group(1).lower() if m else None


def resolve_name(tok, cfg, idx_by_name):
    """C symbol (as written in include/doc) -> 8-hex address or None"""
    base = re.sub(r"\(.*$", "", tok).strip()
    if re.fullmatch(r"(FUN_overlay_d_\d+__|FUN_|sub_|thunk_FUN_)[0-9a-fA-F]{8}", base):
        return base[-8:].lower()
    if base in cfg:
        return cfg[base]
    return idx_by_name.get(base)


# ---------------------------------------------------------------- include

def include_protos():
    out = []  # (name, ret, params)
    for h in sorted(glob.glob(os.path.join(ROM, "include", "*.h"))):
        s = strip_comments(read(h))
        s = re.sub(r"static\s+inline[^{;]*\{(?:[^{}]|\{[^{}]*\})*\}", "", s)  # drop inline bodies
        s = re.sub(r"typedef\s+(?:struct|enum)\s*\{.*?\}\s*\w+;", "", s, flags=re.S)
        s = re.sub(r"^enum\s+\w+\s*\{.*?\};", "", s, flags=re.S | re.M)
        for m in re.finditer(r"^([A-Za-z_][\w\s\*]*?)\b(\w+)\s*\(([^;{}]*)\)\s*;", s, flags=re.M):
            ret, name, args = m.groups()
            if ret.strip().startswith(("typedef", "#")):
                continue
            ps = [split_param(a) for a in args.split(",")] if args.strip() else []
            ps = [p for p in ps if p]
            out.append((name, canon_type(ret), [(canon_type(t), n) for t, n in ps], os.path.basename(h)))
    return out


# ---------------------------------------------------------------- rust

FN_RE = re.compile(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+(\w+)\s*(<[^>(]*>)?\s*\(")


def rust_files():
    return sorted(glob.glob(os.path.join(ROOT, "crates", "*", "src", "**", "*.rs"), recursive=True))


def parse_rust(cfg, idx_by_name, stats):
    """yield dicts: addr, cname, ret, params, rustfn, where"""
    res = []
    for path in rust_files():
        lines = read(path).splitlines()
        impl_ty, in_trait = None, False
        i = 0
        rel = os.path.relpath(path, ROOT)
        while i < len(lines):
            ln = lines[i]
            m = re.match(r"^(?:pub\s+)?impl(?:<[^>]*>)?\s+(?:[\w:<>, ]+\s+for\s+)?(\w+)", ln)
            if m:
                impl_ty, in_trait = m.group(1), False
            elif re.match(r"^(?:pub\s+)?trait\s+\w+", ln):
                impl_ty, in_trait = None, True
            elif re.match(r"^\S", ln) and not ln.startswith(("}", "//", "#", "///")):
                if not re.match(r"^(?:pub\s+)?impl|^(?:pub\s+)?trait", ln):
                    impl_ty, in_trait = None, False
            if not re.match(r"^\s*///", ln):
                i += 1
                continue
            j = i
            while j < len(lines) and re.match(r"^\s*(///|#\[)", lines[j]):
                j += 1
            block = lines[i:j]
            doc = " ".join(re.sub(r"^\s*///\s?", "", b) for b in block if b.lstrip().startswith("///"))
            fm = FN_RE.match(lines[j]) if j < len(lines) else None
            cm = re.search(r"\bC:\s*(.*)", doc)
            i = j
            if not cm or not fm:
                continue
            stats["rust_cdoc_fn"] += 1
            # signature text up to '{' or ';'
            k, sig = j, ""
            while k < len(lines):
                sig += " " + lines[k].strip()
                if "{" in lines[k] or lines[k].rstrip().endswith(";"):
                    break
                k += 1
            sig = sig.split("{")[0].strip().rstrip(";")
            ctext = cm.group(1)
            toks = re.findall(r"`([^`]+)`", ctext)
            addr = cname = None
            for t in toks:
                a = resolve_name(t, cfg, idx_by_name)
                if a:
                    addr, cname = a, re.sub(r"\(.*$", "", t).strip()
                    break
            if not addr:
                a = hex_in(ctext.split(":")[0] if False else ctext[:200])
                if a and toks:
                    addr, cname = a, re.sub(r"\(.*$", "", toks[0]).strip()
            res.append(dict(addr=addr, cname=cname, sig=sig, fn=fm.group(1), impl=impl_ty,
                            trait=in_trait, where="%s:%d" % (rel, j + 1), generic=fm.group(2)))
        # next file
    return res


def split_args(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return [a.strip() for a in out]


def rust_to_proto(it):
    """-> (ret, [(type,name)]) or reason string"""
    m = re.search(r"\bfn\s+\w+\s*(?:<[^>(]*>)?\s*\(", it["sig"])
    if not m:
        return "unparsed"
    depth, k = 1, m.end()
    while k < len(it["sig"]) and depth:
        depth += {"(": 1, ")": -1}.get(it["sig"][k], 0)
        k += 1
    args = split_args(it["sig"][m.end():k - 1])
    rm = re.match(r"\s*->\s*(.+?)(?:\s+where\b.*)?$", it["sig"][k:])
    ret = rm.group(1).strip() if rm else ""
    params = []
    for a in args:
        if re.fullmatch(r"&\s*(?:mut\s+)?self", a) or a == "self":
            if it["trait"]:
                continue  # trait hooks: self is the environment, not a C argument
            if it["impl"] == "BattleMon":
                params.append(("BTL_POKEPARAM *", "bpp"))
                continue
            if it["impl"] in STRUCT_PTR:
                params.append((STRUCT_PTR[it["impl"]] + " *", "self_"))
                continue
            if it["impl"] in OPAQUE:
                params.append(("void *", "self_"))
                continue
            return "self of %s" % it["impl"]
        pm = re.match(r"^(?:mut\s+)?(\w+)\s*:\s*(.+)$", a)
        if not pm:
            return "param %r" % a
        name, ty = pm.groups()
        ty = ty.strip()
        if re.fullmatch(r"&\s*mut\s+E|&\s*mut\s+dyn\s+\w+", ty) and name in ("env", "e"):
            continue  # Rust-side environment, not a C argument
        mm = re.fullmatch(r"&\s*(?:mut\s+)?(\w+)", ty)
        if mm and mm.group(1) == "BattleMon":
            params.append(("BTL_POKEPARAM *", name))
        elif ty in PRIM:
            if name in ("pp", "party", "party_pp") and ty == "u32":
                params.append(("void *", name))
            else:
                params.append((PRIM[ty], name))
        else:
            return "param type %s" % ty
    if ret == "":
        r = "void"
    elif ret in PRIM:
        r = PRIM[ret]
    else:
        return "return type %s" % ret
    return r, params


# ---------------------------------------------------------------- main

def existing_sigs(path, cfg, idx_by_name):
    sigs = {}
    for l in read(path).splitlines():
        if not l.strip() or l.startswith("#"):
            continue
        p = l.split("\t")
        if p[0] == "@pool":
            continue
        a = p[0].lower().replace("0x", "")
        if not re.fullmatch(r"[0-9a-f]{8}", a):
            a = resolve_name(p[0], cfg, idx_by_name) or p[0]
        sigs[a] = p[1]
    return sigs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--index", default="/tmp/blk/re/battle/pc/index.jsonl")
    ap.add_argument("--set", default="main 93 94 95 96")
    ap.add_argument("--out", default=os.path.join(HERE, "signatures.generated.tsv"))
    a = ap.parse_args()

    idx = load_index(a.index)
    idx_by_name = {}
    for ad, nm in idx.items():
        idx_by_name.setdefault(nm, ad)
    cfg = load_cfg(a.set.split())
    existing = existing_sigs(os.path.join(HERE, "signatures.tsv"), cfg, idx_by_name)

    chosen = {}      # addr -> (proto, source, where)
    conflicts, skipped = [], []
    counts = {"include": 0, "rust": 0, "runtime": 0}

    def offer(addr, proto, source, where):
        if addr not in idx:
            skipped.append("%s %s: address %s not in index (other overlay set?)" % (source, where, addr))
            return
        if addr in existing:
            if norm_sig(existing[addr]) != norm_sig(proto):
                conflicts.append("%s %s @%s\n    existing: %s\n    %-8s: %s" % (
                    source, where, addr, existing[addr], source, proto))
            return
        if addr in chosen:
            if norm_sig(chosen[addr][0]) != norm_sig(proto):
                conflicts.append("%s %s @%s\n    kept %s (%s): %s\n    dropped %s: %s" % (
                    source, where, addr, chosen[addr][1], chosen[addr][2], chosen[addr][0], source, proto))
            return
        chosen[addr] = (proto, source, where)
        counts[source] += 1

    # (a) include/
    for name, ret, ps, h in include_protos():
        ad = resolve_name(name, cfg, idx_by_name)
        if not ad:
            skipped.append("include %s:%s: no address for symbol" % (h, name))
            continue
        offer(ad, fmt_proto(ret, name, ps), "include", "%s:%s" % (h, name))

    # (c) runtime helpers (ordered before rust so the curated forms win)
    for ad, proto in RUNTIME:
        offer(ad, proto, "runtime", ad)

    # (b) rust
    stats = {"rust_cdoc_fn": 0}
    for it in parse_rust(cfg, idx_by_name, stats):
        if not it["addr"]:
            skipped.append("rust %s: no resolvable C symbol/address" % it["where"])
            continue
        r = rust_to_proto(it)
        if isinstance(r, str):
            skipped.append("rust %s (%s @%s): %s" % (it["where"], it["fn"], it["addr"], r))
            continue
        ret, params = r
        cn = re.sub(r"\W", "_", it["cname"] or it["fn"])
        if not re.match(r"[A-Za-z_]", cn):
            cn = "sub_" + it["addr"]
        offer(it["addr"], fmt_proto(ret, cn, params), "rust", it["where"])

    # informational: compare declared arity with what Ghidra inferred in the existing pseudo-C
    arity = []
    pcdir = os.path.join(os.path.dirname(a.index))
    for ad in sorted(chosen):
        hits = glob.glob(os.path.join(pcdir, ad + "_*.c"))
        if not hits:
            continue
        head = [l for l in read(hits[0]).splitlines() if l.strip()][:1]
        m = re.match(r"^[^(]*\((.*)\)\s*$", head[0]) if head else None
        if not m:
            continue
        g = 0 if m.group(1).strip() in ("void", "") else len(m.group(1).split(","))
        pm = re.match(r"^.*?\((.*)\)\s*$", chosen[ad][0])
        c = 0 if pm.group(1).strip() in ("void", "") else len(pm.group(1).split(","))
        if g != c:
            arity.append("%s %s: declared %d, ghidra %d (%s)" % (ad, chosen[ad][1], c, g, chosen[ad][2]))

    with open(a.out, "w") as f:
        f.write("# GENERATED by tools/re/gensigs.py from include/, Rust `/// C:` items and section 7. Do not edit;\n"
                "# the coordinator merges wanted lines into signatures.tsv. Applied after it (later wins).\n")
        for src in ("runtime", "include", "rust"):
            f.write("# --- %s\n" % src)
            for ad in sorted(chosen):
                if chosen[ad][1] == src:
                    f.write("%s\t%s\t# %s\n" % (ad, chosen[ad][0], chosen[ad][2]) if False else
                            "%s\t%s\n" % (ad, chosen[ad][0]))
    with open(os.path.join(HERE, "types.h"), "w") as f:
        f.write(gen_types())
    rep = os.path.join(HERE, "signatures.generated.report")
    with open(rep, "w") as f:
        f.write("generated lines: %s (total %d)\n" % (counts, sum(counts.values())))
        f.write("rust /// C: items followed by a fn: %d\n" % stats["rust_cdoc_fn"])
        f.write("\n== conflicts (%d) ==\n" % len(conflicts))
        f.write("\n".join(conflicts) + "\n")
        f.write("\n== arity differs from existing pseudo-C (%d; review before merging) ==\n" % len(arity))
        f.write("\n".join(arity) + "\n")
        f.write("\n== skipped (%d) ==\n" % len(skipped))
        f.write("\n".join(skipped) + "\n")
    print("generated %s; conflicts %d; skipped %d; report %s" % (counts, len(conflicts), len(skipped), rep))


if __name__ == "__main__":
    sys.exit(main())
