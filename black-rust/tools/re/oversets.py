#!/usr/bin/env python3
"""Infer which NDS overlays are co-resident and propose decompile sets (Z12).

usage: python3 -I oversets.py <build/black.us> [--sets sets.proposed.txt] [--md OVERLAY-SETS.md]
                              [--results results.tsv] [--mkelf mkelf.py] [--report]
Stdlib only (python 3.7+). Inputs (read-only): pokeblack.us.nds (ARM9 overlay table "y9"),
main.elf (decomp symbols; sections OVY_<n> carry the labels), main.sbin, OVY_<n>.sbin.

Evidence, strongest first:
 1. Overlay load calls. main has one overlay manager (all addresses are main's):
        02034AC4  overlay_load(id)    -> checks 02034BB0 (panics if [addr, addr+size) overlaps a
                                         loaded overlay of the same region), records {id, 1} in an
                                         8-byte-slot table, calls FS_LoadOverlay (02079264)
        02034A5C  overlay_unload(id)  -> finds the slot, FS_UnloadOverlay (020792B8)
    Call sites pass the id as `movs r0,#id` or `ldr r0,=id`. M loading K means M and K are
    resident together. Dynamic ids (r0 from a table) are counted but not resolved.
 2. Address ranges: y9 gives [addr, addr+size+bss). Overlays that overlap are never co-resident
    (the loader panics), so every set is checked against that.
 3. Call edges: a BL/BLX in M targeting a labelled function start of exactly one non-overlapping
    overlay K (noisy: missing labels; used only to rank).
 4. Proc-table references: a literal pool of M holds the address of a 3-pointer GFL_PROC table
    that lives in K (M starts K's proc; NOT proof of co-residency: the proc often starts after M
    unloads).
 5. Layout parent: each overlay's base equals the end (incl. BSS) of exactly one other overlay;
    the linker placed K after its dependencies, base = max(end(deps)). Bounds what may coexist.
 6. Debug source-file strings in each overlay's data label it (field, bag, box, ...).
Every set is re-checked against mkelf.py's rule (no overlap of [addr, addr+blob)) and the
stricter one that includes BSS.
"""
import argparse
import collections
import os
import re
import struct
import sys

MAIN_BASE = 0x02004000
STRONG = 3  # min number of distinct call sites for a strong edge
OVL_LOAD = 0x02034AC4
OVL_UNLOAD = 0x02034A5C

# overlay families that are out of scope (D9): networking, Wi-Fi, C-Gear, IR, musicals, ...
NETWORK_KEYS = (
    "net_", "dwc", "nhttp", "ghttp", "gds_", "gtsnego", "wifi", "wireless", "whpipe", "wih.c", "irc",
    "union", "intrude", "colosseum", "mystery", "gsync", "pdwacc", "worldtrade", "pokemontrade",
    "championship", "livebattlematch", "wifibattlematch", "musical", "mus_", "sta_act", "c_gear",
    "no_gear", "beacon", "br_", "mb_", "guru2", "ctvt", "local_tvt", "oekaki", "mictest", "delivery",
    "comm_btl", "vs_multi", "wf2dmap", "event_gtsnego", "event_wificlub", "event_wifibattlematch",
    "cg_help", "event_comm", "script_command.c", "dup_fitting", "research",
)
# first match wins; keyword -> short label
LABELS = (
    ("pokemontrade", "trade"), ("fieldmap.c", "field"), ("script_sys.c", "field-script"),
    ("event_mapchange", "field-mapchange"), ("field_gimmick", "field-gimmick"), ("sp_poke_gimmick", "field-gimmick"),
    ("rail_slipdown", "field-gimmick"), ("fld_faceup", "field-faceup"), ("field_fk_sound", "field-gimmick"),
    ("dowsing", "field-dowsing"), ("report.c", "field-report"), ("btl_server", "battle"), ("btlv_stage", "battle-view"),
    ("btlv_scd", "battle-view"), ("tr_ai", "battle-ai"), ("b_bag_main", "battle-bag"), ("b_plist", "battle-party"),
    ("b_app_tool", "battle-app"), ("itemmenu", "bag"), ("pokelist", "party"), ("p_status", "summary"),
    ("box2_", "box"), ("startmenu", "startmenu"), ("townmap", "townmap"), ("ui_scene", "ui-common"),
    ("zukan_info", "pokedex-info"), ("zukan_detail", "pokedex-detail"), ("zknsearch", "pokedex-search"),
    ("zukan.c", "pokedex"), ("zukan_toroku", "pokedex-register"), ("namein", "naming"), ("shinka_demo", "evolution"),
    ("egg_demo", "egg-hatch"), ("intro_", "intro"), ("demo3d", "demo3d"), ("psel", "starter-select"),
    ("the_end", "ending"), ("award", "awards"), ("pms_input", "pms-input"), ("mailtool", "mail"),
    ("save_outside", "save"), ("event_battle_return", "battle-return"), ("event_pdc_return", "pdc-return"),
    ("pdc.c", "pdc"), ("manual_", "manual"), ("subway_map", "subway-map"), ("cdemo", "cdemo"),
    ("monolith_tool", "monolith"), ("un_select", "un-select"), ("battle_rec", "battle-rec"),
    ("btl_rec_sel", "battle-rec-select"), ("research", "research"),
)


def die(msg):
    sys.exit("oversets: " + msg)


# ---------------------------------------------------------------- inputs
def read_y9(build):
    """ARM9 overlay table from the .nds header (offset/size at 0x50/0x54). 32-byte entries."""
    p = os.path.join(build, "pokeblack.us.nds")
    with open(p, "rb") as f:
        hdr = f.read(0x200)
        off, size = struct.unpack_from("<II", hdr, 0x50)
        f.seek(off)
        t = f.read(size)
    ov = {}
    for i in range(0, len(t), 32):
        oid, addr, rsize, bss, sinit0, sinit1, fid, _ = struct.unpack_from("<8I", t, i)
        ov[oid] = dict(id=oid, addr=addr, size=rsize, bss=bss, sinit=(sinit0, sinit1), file=fid)
    return ov


def read_elf_symbols(path):
    """{section-name: [(value, name, type, size)]} from main.elf (decomp symbols)."""
    d = open(path, "rb").read()
    (shoff,) = struct.unpack_from("<I", d, 0x20)
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", d, 0x2E)
    secs = []
    for i in range(shnum):
        f = struct.unpack_from("<10I", d, shoff + i * shentsize)
        secs.append(dict(name=f[0], type=f[1], off=f[4], size=f[5], link=f[6]))
    ss = secs[shstrndx]
    for s in secs:
        o = ss["off"] + s["name"]
        s["name"] = d[o:d.index(b"\0", o)].decode()
    sym = next(s for s in secs if s["type"] == 2)
    st = secs[sym["link"]]
    out = collections.defaultdict(list)
    for i in range(1, sym["size"] // 16):
        nm, val, size, info, _, shndx = struct.unpack_from("<IIIBBH", d, sym["off"] + i * 16)
        if shndx >= len(secs):
            continue
        o = st["off"] + nm
        out[secs[shndx]["name"]].append((val, d[o:d.index(b"\0", o)].decode("latin1"), info & 0xF, size))
    return out


class Rom:
    def __init__(self, build):
        self.build = build
        self.ov = read_y9(build)
        self.blob = {}
        for k in self.ov:
            self.blob[k] = open(os.path.join(build, "OVY_%d.sbin" % k), "rb").read()
        self.main = open(os.path.join(build, "main.sbin"), "rb").read()
        self.main_end = MAIN_BASE + len(self.main)
        syms = read_elf_symbols(os.path.join(build, "main.elf"))
        self.fstart = collections.defaultdict(set)  # addr -> overlays with a labelled function start
        self.mode = {}  # overlay -> sorted [(addr, 'a'|'t'|'d')]
        for k in self.ov:
            m = []
            for val, name, typ, size in syms.get("OVY_%d" % k, []):
                if name[:2] in ("$a", "$t", "$d") and (len(name) == 2 or not name[2].isalnum() or name[2] in ".$"):
                    m.append((val & ~1, name[1]))
                elif name and name[0] != "$" and not name.startswith("_@") and typ in (0, 2):
                    self.fstart[val & ~1].add(k)
            self.mode[k] = sorted(m)
        for k, o in self.ov.items():
            o["end"] = o["addr"] + o["size"] + o["bss"]
            o["blobend"] = o["addr"] + len(self.blob[k])
            o["strings"] = self.source_strings(self.blob[k])
            o["label"] = label_of(o["strings"])

    @staticmethod
    def source_strings(b):
        names = []
        for m in re.finditer(rb"[A-Za-z0-9_./\\-]{3,64}\.(?:c|h|cpp|cc)\x00", b):
            s = m.group(0)[:-1].decode()
            if s not in names:
                names.append(s)
        return names

    def code_ranges(self, k):
        """[(lo, hi, 'a'|'t')] code ranges of overlay k from mapping symbols."""
        o = self.ov[k]
        m = self.mode[k]
        out = []
        for i, (a, c) in enumerate(m):
            hi = m[i + 1][0] if i + 1 < len(m) else o["blobend"]
            if c in ("a", "t") and hi > a:
                out.append((a, hi, c))
        return out

    def modules(self):
        return [("main", MAIN_BASE, self.main)] + [(k, self.ov[k]["addr"], self.blob[k]) for k in sorted(self.ov)]


def label_of(strings):
    for key, lab in LABELS:
        if any(key in s for s in strings):
            return lab
    return ""


def is_network(o):
    """majority of the debug source names match a D9 keyword (overlay 10 has a few event_irc*/musical_*
    names among 36 but is the field script core)"""
    st = o["strings"]
    if not st:
        return o.get("net_by_edges", False)
    return sum(any(k in s for k in NETWORK_KEYS) for s in st) * 2 > len(st)


# ---------------------------------------------------------------- analysis
def thumb_calls(addr, b, lo=0, hi=None):
    """(site, target) for every Thumb BL/BLX pair in b[lo:hi]."""
    hi = len(b) if hi is None else hi
    for i in range(lo & ~1, hi - 3, 2):
        h1, h2 = struct.unpack_from("<HH", b, i)
        if (h1 & 0xF800) == 0xF000 and (h2 & 0xF800) in (0xF800, 0xE800):
            off = ((h1 & 0x7FF) << 12) | ((h2 & 0x7FF) << 1)
            if off & 0x400000:
                off -= 0x800000
            t = addr + i + 4 + off
            if (h2 & 0xF800) == 0xE800:
                t &= ~3
            yield addr + i, t, i


def arm_calls(addr, b, lo=0, hi=None):
    hi = len(b) if hi is None else hi
    for i in range((lo + 3) & ~3, hi - 3, 4):
        (w,) = struct.unpack_from("<I", b, i)
        if (w >> 24) in (0xEB, 0xFA, 0xFB):
            off = w & 0xFFFFFF
            if off & 0x800000:
                off -= 0x1000000
            yield addr + i, addr + i + 8 + off * 4, i


def call_edges(rom):
    """edges[M][K] = set of call-site addresses in M that call a labelled function of K."""
    edges = collections.defaultdict(lambda: collections.defaultdict(set))
    for m, o in rom.ov.items():
        b = rom.blob[m]
        base = o["addr"]
        for lo, hi, c in rom.code_ranges(m):
            gen = thumb_calls if c == "t" else arm_calls
            for site, t, _ in gen(base, b, lo - base, min(hi - base, len(b))):
                _edge(rom, edges, m, site, t)
    return edges


def _edge(rom, edges, m, site, t):
    o = rom.ov[m]
    if t < rom.main_end or o["addr"] <= t < o["blobend"]:
        return
    ks = rom.fstart.get(t & ~1)
    if ks:
        ks = [k for k in ks if k != m and not overlaps(rom.ov[k], o)]
        if len(ks) == 1:
            edges[m][ks[0]].add(site)


def load_sites(rom):
    """Call sites of the overlay manager (see module docstring).
    Returns (loads, unloads, dyn): loads[(M, K)] = [site, ...] for static ids; dyn[M] = [site, ...]
    for load calls whose id is not an immediate in the preceding instructions."""
    loads = collections.defaultdict(list)
    unloads = collections.defaultdict(list)
    dyn = collections.defaultdict(list)
    for name, base, b in rom.modules():
        n = len(b)
        for site, t, i in thumb_calls(base, b):
            tt = t & ~1
            if tt not in (OVL_LOAD, OVL_UNLOAD):
                continue
            val = None
            for d in range(2, 40, 2):
                p = i - d
                if p < 0:
                    break
                (h,) = struct.unpack_from("<H", b, p)
                if (h & 0xF800) == 0x2000 and ((h >> 8) & 7) == 0:  # movs r0,#imm
                    val = h & 0xFF
                    break
                if (h & 0xF800) == 0x4800 and ((h >> 8) & 7) == 0:  # ldr r0,[pc,#imm]
                    po = ((base + p + 4) & ~3) + (h & 0xFF) * 4 - base
                    if 0 <= po <= n - 4:
                        val = struct.unpack_from("<I", b, po)[0]
                    break
                if (h & 7) == 0 and (h & 0xF800) in (0x1800, 0x1C00, 0x0000, 0x4600):  # adds/lsls/mov into r0
                    break
            tab = loads if tt == OVL_LOAD else unloads
            if val is not None and val in rom.ov:
                tab[(name, val)].append(site)
            elif tt == OVL_LOAD:
                dyn[name].append(site)
    return loads, unloads, dyn


def proc_spawns(rom):
    """{(M, K): {table: movs_seen}}: M's literal pool holds the address of a 3-pointer table
    (GFL_PROC init/main/exit) that lives in K. movs_seen = a `movs rX,#K` sits within 96 bytes of
    the load; without it the reference is accepted only when exactly one overlay has such a table
    at that address."""
    def in_k(k, w):
        o = rom.ov[k]
        return w != 0 and o["addr"] <= (w & ~1) < o["blobend"]

    tabs = {}
    for k, o in rom.ov.items():
        b = rom.blob[k]
        for i in range(0, len(b) - 11, 4):
            t = struct.unpack_from("<3I", b, i)
            if all(in_k(k, w) and w & 1 for w in t):
                if i >= 4 and in_k(k, struct.unpack_from("<I", b, i - 4)[0]):
                    continue
                if i + 16 <= len(b) and in_k(k, struct.unpack_from("<I", b, i + 12)[0]):
                    continue
                tabs.setdefault(o["addr"] + i, set()).add(k)
    out = collections.defaultdict(dict)
    for name, addr, b in rom.modules():
        n = len(b)
        for i in range(0, n - 1, 2):
            (h,) = struct.unpack_from("<H", b, i)
            if h >> 11 != 9:  # ldr rX,[pc,#imm]
                continue
            po = ((addr + i + 4) & ~3) + (h & 0xFF) * 4 - addr
            if not 0 <= po <= n - 4:
                continue
            (w,) = struct.unpack_from("<I", b, po)
            if w not in tabs:
                continue
            for k in tabs[w]:
                if k == name or (name != "main" and overlaps(rom.ov[k], rom.ov[name])):
                    continue
                seen = False
                for d in range(-96, 98, 2):
                    p = i + d
                    if 0 <= p < n - 1:
                        (h2,) = struct.unpack_from("<H", b, p)
                        if h2 >> 11 == 4 and (h2 & 0xFF) == k:
                            seen = True
                            break
                if seen or len(tabs[w]) == 1:
                    out[(name, k)][w] = out[(name, k)].get(w, False) or seen
    return out


# ---------------------------------------------------------------- sets
def overlaps(a, b, strict=True):
    ea = a["end"] if strict else a["blobend"]
    eb = b["end"] if strict else b["blobend"]
    return a["addr"] < eb and b["addr"] < ea


def conflicts(rom, mods):
    """(blob_overlaps, bss_only_overlaps): the first is mkelf.py's refusal rule."""
    ks = sorted(mods, key=lambda k: rom.ov[k]["addr"])
    blob, bss = [], []
    for i, a in enumerate(ks):
        for b in ks[i + 1:]:
            if overlaps(rom.ov[a], rom.ov[b], False):
                blob.append((a, b))
            elif overlaps(rom.ov[a], rom.ov[b], True):
                bss.append((a, b))
    return blob, bss


def layout_parents(rom, k):
    return sorted(p for p, q in rom.ov.items() if p != k and q["end"] == rom.ov[k]["addr"] and q["size"] > 32)


def infer_network(rom, edges, spawns, loads):
    """Unlabelled overlays (no debug source names): decide network vs not from loaders, call edges,
    same-base siblings and layout parent. Labelled overlays use the NETWORK_KEYS majority rule."""
    for k, o in sorted(rom.ov.items()):
        o["net_by_edges"] = False
        if o["strings"] or o["size"] <= 32:
            continue
        starters = {m for (m, kk) in list(spawns) + list(loads) if kk == k and m != "main"}
        if any(rom.ov[m]["strings"] and not is_network(rom.ov[m]) for m in starters):
            continue  # started/loaded by a labelled non-network overlay (field, battle, ...)
        net = sum(len(v) for j, v in edges.get(k, {}).items() if rom.ov[j]["strings"] and is_network(rom.ov[j]))
        oth = sum(len(v) for j, v in edges.get(k, {}).items() if not (rom.ov[j]["strings"] and is_network(rom.ov[j])))
        sib = [j for j, q in rom.ov.items() if j != k and q["addr"] == o["addr"] and q["strings"]]
        par = [j for j in layout_parents(rom, k) if rom.ov[j]["strings"]]
        if net > oth:
            o["net_by_edges"] = True
        elif oth < STRONG and sib and all(is_network(rom.ov[j]) for j in sib):
            o["net_by_edges"] = True
        elif oth < STRONG and o["addr"] < 0x021B6100 and par and all(is_network(rom.ov[j]) for j in par):
            o["net_by_edges"] = True


# Runtime sets. "main" is implicit. 10+11 are loaded together by main at boot (02005240) and stay resident
# in normal (non-Wi-Fi) mode; apps are placed above them. Members beyond 10/11 come from load sites; see note.
PRIMARY = (
    ("field", (10, 11, 18, 20, 21),
     "main loads 10+11 (02005240); 10 loads 20 (0215FA08); 21 loads 18 at 021881A6 and unloads it at 02188B48; 21<->10 call each other (~2,400 / ~250 sites)"),
    ("bag", (10, 11, 70), "bag app; 10 holds its proc table and loads nothing else here; 21 is unloaded (ranges overlap)"),
    ("party-summary", (10, 11, 90, 91, 131, 134),
     "134 loads 91, 131, 90 together (021DC9C8/021DCA92/021DCC36); 90 is 91's layout parent"),
    ("box", (10, 11, 67, 131, 170), "170 loads 67 (021E1E88); 131 is 170's layout parent and 170 references its proc table"),
    ("startmenu", (10, 11, 67, 88, 204, 228), "88 loads 204 and 228 (021BC1B4/021BC35A); 204 loads 67 at entry"),
    ("demo3d", (10, 11, 67, 203), "203 loads 67 at entry (021BDA48)"),
    ("townmap", (10, 11, 67, 72), "72 loads 67 at entry (021B95AA)"),
    ("pokedex", (10, 11, 67, 206, 208, 212),
     "208 loads 67 and 206 at entry (021BEA0C/021BEA12); 212 is 208's layout child (209/210 reference it)"),
    ("pokedex-area", (10, 11, 67, 72, 208, 212), "208 loads 72 (021BF79A) after unloading 206 (overlap), reusing the town map"),
    ("pokedex-search", (10, 11, 67, 206, 210, 212), "210 loads 67 (021BCF92); 210 is an alternative to 208 at 0x021BC240"),
    ("evolution", (10, 11, 67, 131, 195, 196, 198),
     "195 loads 131, 196, 198, 67 (0220473A/022048D0/022048F4?/02204C3C); 198 is an alternative to 197"),
    ("naming-egg", (10, 11, 194, 215), "215 is 194's layout child (egg hatch then nickname); both start from 10/21 proc tables"),
    ("nick-register", (10, 11, 67, 94, 206, 207), "207 loads 67, 206 (0220AF4A/0220AF50) and 94 (0220B95C)"),
    ("battle-bag", (93, 94, 95, 96, 196, 197), "93 loads 94, 95, 96 (021E8DCC/DD8/DE6), 196 and 197 (021E9EBA/EDE); same as `battle` + 196/197"),
    ("battle-party", (93, 94, 95, 96, 196, 198), "93 loads 196 and 198 (021E9B6A/9B94); 197 and 198 are alternatives at 0x0220B120"),
    ("pdc", (10, 11, 94, 98, 196, 197), "98 (pdc_main) loads 94, 196, 197 (021B6164/021B65EC/021B6604); no 93"),
)


def family(rom, k):
    o = rom.ov[k]
    return "field-child" if o["addr"] >= 0x021EE740 and not any(
        overlaps(o, rom.ov[c]) for c in (10, 11, 20, 21)) else "app"


def edge_count(edges, a, b):
    return len(edges.get(a, {}).get(b, ()))


def evidence(rom, edges, loads, spawns, members):
    """short per-member justification strings"""
    out = []
    ms = [m for m in members if m not in (10, 11)]
    for k in ms:
        bits = []
        ld = sorted({m for (m, kk) in loads if kk == k and (m in ms or m == "main")}, key=str)
        if ld:
            bits.append("loaded by " + ",".join(map(str, ld)))
        e = [(j, edge_count(edges, k, j)) for j in ms if j != k and edge_count(edges, k, j) >= STRONG]
        if e:
            bits.append("calls " + ",".join("%d(x%d)" % x for x in e))
        sp = sorted({m for (m, kk) in spawns if kk == k and (m in ms or m == "main")}, key=str)
        if sp:
            bits.append("proc table ref by " + ",".join(map(str, sp)))
        par = [p for p in layout_parents(rom, k) if p in ms]
        if par:
            bits.append("layout parent " + ",".join(map(str, par)))
        if bits:
            out.append("%d: %s" % (k, "; ".join(bits)))
    return out


def missing_loads(rom, loads, mods, net_ok=False):
    """static load targets of members that are neither in the set nor overlapping a member (nor stub/network)"""
    miss = []
    for (m, k), sites in sorted(loads.items(), key=lambda kv: (str(kv[0][0]), kv[0][1])):
        if m not in mods or m in (10, 11) or k in mods or rom.ov[k]["size"] <= 32:
            continue
        if any(overlaps(rom.ov[k], rom.ov[j]) for j in mods):
            continue
        miss.append((m, k, is_network(rom.ov[k])))
    return miss


def build_sets(rom, edges, loads, spawns):
    sets = []
    covered = set()
    for name, mods, note in PRIMARY:
        sets.append(dict(name=name, mods=list(mods), main=True, note=note, kind="runtime"))
        covered.update(mods)
    eligible = [k for k, o in sorted(rom.ov.items()) if o["size"] > 32 and not is_network(o)]
    left = [k for k in eligible if k not in covered]
    for fam, tag in (("field-child", "fld"), ("app", "app")):
        ps = []
        for k in sorted((k for k in left if family(rom, k) == fam), key=lambda k: -rom.ov[k]["size"]):
            for p in ps:
                if not any(overlaps(rom.ov[k], rom.ov[j]) for j in p):
                    p.append(k)
                    break
            else:
                ps.append([k])
        for i, p in enumerate(ps, 1):
            sets.append(dict(
                name="pack-%s-%02d" % (tag, i), mods=sorted(p, key=lambda k: rom.ov[k]["addr"]), main=False, kind="pack",
                note="NOT co-resident at runtime: non-overlapping %s overlays packed for decompile only; lite (no main)" % fam))
            covered.update(p)
    return sets, covered, eligible


def mkelf_check(path_mkelf, build, mods, out):
    import subprocess
    r = subprocess.run([sys.executable, "-I", path_mkelf, build, out] + [str(m) for m in mods],
                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True)
    return r.returncode, (r.stdout.strip().splitlines() or [""])[-1]


def set_line(s):
    return " ".join([s["name"]] + (["main"] if s["main"] else []) + [str(m) for m in s["mods"]])


def addr_range(rom, mods):
    lo = min(rom.ov[m]["addr"] for m in mods)
    hi = max(rom.ov[m]["end"] for m in mods)
    return "0x%08X-0x%08X" % (lo, hi)


def label_of_set(rom, mods):
    return " ".join("%d:%s" % (m, rom.ov[m]["label"] or "-") for m in sorted(mods, key=lambda k: rom.ov[k]["addr"]))


def read_results(path):
    res = {}
    if path:
        for line in open(path):
            f = line.rstrip("\n").split("\t")
            if len(f) >= 4 and f[0] != "name":
                res[f[0]] = f[1:]
    return res


def write_md(path, rom, edges, loads, spawns, sets, results, net, stubs):
    L = []
    L.append("# Overlay co-residency sets (Z12)\n")
    L.append("Generated by `tools/re/oversets.py` (inputs: ROM y9 overlay table, `main.elf` labels, overlay binaries). "
             "Proposed sets: `tools/re/sets.proposed.txt` (same format as `sets.txt`; the coordinator merges).\n")
    L.append("## Overlay load API (main)\n")
    L.append("- `0x02034AC4` `overlay_load(id)`: `FUN_02034BB0` first checks `[addr, addr+size)` of the new overlay against every loaded "
             "overlay in the same RAM region and panics (`FUN_02034714`) on overlap, then records `{id, 1}` in an 8-byte-slot table "
             "(slot count per region from `FUN_02034D08`) and calls `FS_LoadOverlay` (`FUN_02079264`).")
    L.append("- `0x02034A5C` `overlay_unload(id)`: finds the slot and calls `FS_UnloadOverlay` (`FUN_020792B8`).")
    L.append("- Call sites pass the id as `ldr r0,=id` / `movs r0,#id`. The loader's overlap check means **the set of loaded overlays is always "
             "non-overlapping**, and *M loads K* means M and K are resident together. `main` loads `10`+`11` at boot (`0x02005240`).\n")
    L.append("## How the sets were inferred\n")
    L.append("1. **Load sites** (strongest): every call to `overlay_load` with an immediate id; a module that loads K is co-resident with K.")
    L.append("2. **Address ranges** from y9: `[addr, addr+size+bss)`. 237 overlays share ~35 base addresses, so most are alternatives.")
    L.append("3. **Call edges**: Thumb/ARM `BL`/`BLX` targeting a labelled function of exactly one non-overlapping overlay (noisy; ranking only).")
    L.append("4. **Proc-table references**: literal-pool pointers to 3-word GFL_PROC tables in another overlay (who starts whom; not proof of co-residency).")
    L.append("5. **Layout parent**: base equals the end (incl. BSS) of exactly one overlay (`base = max(end(deps))` in the original linker script); "
             "bounds what may coexist but is not what is resident (e.g. every app is placed after the Wi-Fi base 9+13).")
    L.append("6. **Labels** from debug source-file strings in each overlay's data.\n")
    L.append("## Findings\n")
    L.append("- **Two base modes.** Normal: `main` loads **10 + 11** (`script_sys`/`scrcmd_*`/`fldmmdl` + `intrude_comm`). Wi-Fi: `main` loads 9 (+13/14/15/114). They overlap, so never together. 10+11 end at `0x02177020`; every app lives above it.")
    L.append("- **`field` = 10 + 11 + 18 + 20 + 21.** 10 loads 20 (`event_mapchange`); 21 loads 18 (`intrude_field`, 021881A6) early and unloads it at 02188B48; "
             "21 and 10 call each other ~2,400 / ~250 times. Field children at base >= `0x021EE740` (gimmicks 36-65, 73-83, ...; 21 loads 29-35, 63, 74 plus table-driven ids) coexist with it.")
    L.append("- **The field overlay is unloaded for apps**: bag/party/summary/box/startmenu/pokedex all sit at 0x021B6100-0x021EE740 and overlap 21 (`0x02187F00-0x021EE740`).")
    L.append("- **Apps load `67` (ui-common) at their entry** (203, 204, 68, 72, 110, 120, 128, 135, 170, 171, 181, 207-210, 213, 214, 216, 218, 225, ...); "
             "`67` ends exactly at `0x021B95A0` where the 0x021B95A0 apps start.")
    L.append("- **Battle** (93) loads 94, 95, 96 and on demand 196, then 197 or 198; `10` and `11` load 93 as well, so they are resident during battle (`battle` in `sets.txt` omits them; add if battle code calls into 10/11). "
             "`tr_ai` 96 sits at 0x02187F00, overlapping 21: the field is unloaded during battle.")
    L.append("- %d overlays are **32-byte stubs**; %d more are network/Wi-Fi/union/C-Gear/IR/musical (see Not covered).\n" % (len(stubs), len(net)))
    L.append("## Sets\n")
    L.append("`functions`/`time`/`failures` come from `decompile.sh` with a private `RE_OUT`; `-` = checked with `mkelf.py` only (not decompiled). "
             "Evidence cells list load sites (`loaded by`), strong call edges (>=3 sites), proc-table references and layout parents between members.\n")
    L.append("| set | overlays | address range | evidence | functions | time | failures | labels |")
    L.append("|---|---|---|---|---|---|---|---|")
    for s in sets:
        r = results.get(s["name"], ["-", "-", "-"])
        ev = s["note"]
        ex = evidence(rom, edges, loads, spawns, s["mods"])
        if ex and s["kind"] == "runtime":
            ev += ". " + " | ".join(ex)
        L.append("| `%s` | %s%s | %s | %s | %s | %s | %s | %s |" % (
            s["name"], "main " if s["main"] else "", " ".join(map(str, s["mods"])), addr_range(rom, s["mods"]),
            ev.replace("|", "/"), r[0], r[1], r[2], label_of_set(rom, s["mods"])))
    nrt = sum(1 for s in sets if s["kind"] == "runtime")
    L.append("\nThe first %d rows are runtime scenarios (10+11 included because `main` keeps them resident). `pack-*` rows only pack "
             "non-overlapping leftovers so every non-network overlay is decompiled at least once (alternatives at one base cannot share an ELF); "
             "they have no `main` to stay cheap, so calls into main/10/21 stay unresolved.\n" % nrt)
    L.append("Overlay 21's table-driven loads (field gimmicks, 18 other `None` ids) are not resolved statically; each gimmick overlay's runtime partner is 21 + 10 + 11.\n")
    L.append("## Not covered\n")
    L.append("- **Stubs (%d overlays, 32 bytes each):** %s" % (len(stubs), " ".join(map(str, stubs))))
    L.append("- **Network / D9 out of scope (%d overlays, not in any set unless a runtime set needs them: 11, 18):**" % len(net))
    for k in net:
        o = rom.ov[k]
        L.append("  - %d @0x%08X %d B: %s" % (k, o["addr"], o["size"], ", ".join(o["strings"][:3]) or "(no strings; network inferred from loaders/calls/siblings)"))
    L.append("")
    with open(path, "w") as f:
        f.write("\n".join(L))


def report(rom, edges, loads, unloads, dyn, spawns):
    print("\n# layout (id addr end size bss label net | layout-parent)")
    for k, o in sorted(rom.ov.items(), key=lambda kv: (kv[1]["addr"], kv[0])):
        print(k, hex(o["addr"]), hex(o["end"]), o["size"], o["bss"], o["label"] or "-", "net" if is_network(o) else "",
              layout_parents(rom, k))
    print("\n# static overlay_load sites: loader -> loaded [sites]")
    for (m, k), s in sorted(loads.items(), key=lambda kv: (str(kv[0][0]), kv[0][1])):
        print(m, "->", k, " ".join("%08x" % x for x in s))
    print("\n# dynamic-id overlay_load sites (id not an immediate)")
    for m, s in sorted(dyn.items(), key=lambda kv: str(kv[0])):
        print(m, " ".join("%08x" % x for x in s))
    print("\n# strong call edges (>=%d sites)" % STRONG)
    for m in sorted(edges):
        s = {k: len(v) for k, v in edges[m].items() if len(v) >= STRONG}
        if s:
            print(m, s)
    print("\n# proc-table references (spawner -> spawned) [* = movs #id seen]")
    for (m, k), t in sorted(spawns.items(), key=lambda kv: (str(kv[0][0]), kv[0][1])):
        print(m, "->", k, " ".join("%s0x%x" % ("*" if v else "", a) for a, v in sorted(t.items())))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--sets")
    ap.add_argument("--md")
    ap.add_argument("--results")
    ap.add_argument("--report", action="store_true", help="print layout, load sites, edges, spawns")
    ap.add_argument("--mkelf", help="path to mkelf.py: run it on every set (overlap check) into a temp file")
    a = ap.parse_args()
    rom = Rom(a.build)
    edges = call_edges(rom)
    loads, unloads, dyn = load_sites(rom)
    spawns = proc_spawns(rom)
    infer_network(rom, edges, spawns, loads)
    print("overlays: %d, labelled function starts: %d, static load sites: %d, dynamic: %d" % (
        len(rom.ov), len(rom.fstart), sum(len(v) for v in loads.values()), sum(len(v) for v in dyn.values())))
    if a.report:
        report(rom, edges, loads, unloads, dyn, spawns)
    sets, covered, eligible = build_sets(rom, edges, loads, spawns)
    stubs = [k for k, o in sorted(rom.ov.items()) if o["size"] <= 32]
    net = [k for k, o in sorted(rom.ov.items()) if o["size"] > 32 and is_network(o)]
    bad = 0
    for s in sets:
        blob, bss = conflicts(rom, s["mods"])
        status = "ok"
        if blob:
            status = "OVERLAP(mkelf) %s" % blob
            bad += 1
        elif bss:
            status = "ok (BSS-only overlap %s)" % bss
        if a.mkelf and not blob:
            rc, msg = mkelf_check(a.mkelf, a.build, s["mods"], "/tmp/oversets-check.elf")
            if rc:
                status = "mkelf FAILED: " + msg
                bad += 1
        if s["kind"] == "runtime":
            miss = missing_loads(rom, loads, s["mods"])
            if miss:
                status += "  static loads not in set: " + " ".join("%d->%d%s" % (m, k, "(net)" if n else "") for m, k, n in miss)
        print("%-16s %s  [%s]  %s" % (s["name"], set_line(s), addr_range(rom, s["mods"]), status))
    miss = [k for k in eligible if k not in covered]
    print("stubs %d, network %d, eligible %d, uncovered eligible: %s" % (len(stubs), len(net), len(eligible), miss))
    if a.sets:
        with open(a.sets, "w") as f:
            f.write("\n".join(set_line(s) for s in sets) + "\n")
    if a.md:
        write_md(a.md, rom, edges, loads, spawns, sets, read_results(a.results), net, stubs)
    if bad:
        sys.exit(1)


if __name__ == "__main__":
    main()
