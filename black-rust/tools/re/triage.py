#!/usr/bin/env python3
"""T1: mechanical triage of the unidentified ARM9 / overlay asm files.

usage: python3 -I tools/re/triage.py [--out tools/re/T1-TRIAGE.tsv] [--md tools/re/T1-TRIAGE.md]
                                     [--features FEATURES.json]

Inputs (read-only):
  $BLACK_ROM_BUILD/main.sbin, OVY_<n>.sbin     bytes (default black-rom/build/black.us in the main checkout)
  black-rom/main.lsf, black-rom/asm/*.s        ARM9 object order, overlay load addresses, asm function counts
  tools/re/T1-unidentified.tsv                 the file list (the old status table's "ARM9 (unidentified)" /
                                               "unidentified overlay" rows, recovered from git c3b839c0)
  tools/re/sets.txt + sets.proposed.txt        overlay co-residency sets (Z12)
  $RE_OUT/<set>/pc/index.jsonl + *.c           Ghidra pseudo-C (RE_OUT=/tmp/blk/re; Z12 sets in /tmp/blk/re-z12)
  tools/re/T1-overrides.tsv                    hand labels for what the script cannot decide (the only LLM step)

Per file it computes: address range, overlay/set, function counts (asm labels and Ghidra), debug source-file
strings (literal-pool pointers into string data, plus strings inside the file), hardware-register pool words,
ARC IDs passed to the verified archive loaders, message banks passed to the msgdata creator, Heap_AllocDebug sizes,
named / hub callees and callers, overlay load sites, neighbour files. Then a rule engine maps source-file anchors
to a subsystem / scope / phase item (SRC_MAP below). Output: T1-TRIAGE.tsv and T1-TRIAGE.md.
Pure Python 3 stdlib (written for 3.7).
"""
import array
import bisect
import collections
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN_CHECKOUT = "$HOME/workplace/black"
ROM = os.environ.get("BLACK_ROM", MAIN_CHECKOUT + "/black-rom")
BUILD = os.environ.get("BLACK_ROM_BUILD", ROM + "/build/black.us")
RE_OUT = os.environ.get("RE_OUT", "/tmp/blk/re")
RE_Z12 = os.environ.get("RE_Z12", "/tmp/blk/re-z12")
MAIN_BASE = 0x02004000
OVL_MIN = 0x02100000

# ---------------------------------------------------------------- knowledge (verified in this repo / T1)
HUBS = {
    0x02030734: "Heap_AllocDebug", 0x020307B0: "Heap_Free", 0x02082D44: "MI_CpuCopy8", 0x02082BCC: "MI_CpuFill8",
    0x02034AC4: "overlay_load", 0x02034A5C: "overlay_unload", 0x0209C2B0: "_u32_div_f", 0x0209C0A4: "_s32_div_f",
    0x0209C054: "_ll_mul", 0x02079264: "FS_LoadOverlay", 0x020792B8: "FS_UnloadOverlay", 0x02034714: "panic",
    0x02034734: "assert_fail", 0x02048CEC: "ArcSys_Init", 0x020490F4: "ArcLoadMalloc(arc,dat)",
    0x02048F4C: "ArcLoadData(dst,arc,dat)", 0x0204A1BC: "ArcLoadMallocEx(arc,dat,..)", 0x020457B0: "StrBuf_Create(size,heap)",
    0x02045B38: "MsgData_Create(type,arc,bank,heap)", 0x02048F60: "ArcLoadMallocB(arc,dat,..)", 0x02048F74: "ArcLoadDataOfs(dst,arc,dat,ofs,n)",
    0x020490A8: "ArcMemberSize(arc,dat)", 0x02045CAC: "MsgData_GetStr", 0x02045EC0: "StrBuf_ReadNum",
}
# verified by caller inspection (FUN_0201AD88 ArcLoadMalloc(16,..), FUN_0201ADD0 ArcLoadData(dst,18,..), FUN_02020F34
# FUN_02048F60(0x18,..)): callee -> index of the ARC id argument
ARC_LOADERS = {0x020490F4: 0, 0x0204A1BC: 0, 0x02048F4C: 1, 0x02048F60: 0, 0x02048F74: 1, 0x020490A8: 0}
MSG_CREATE = 0x02045B38  # (type, arc 2|3, bank, heap)
ALLOC = 0x02030734       # (heap, size, align, "file.c", line)
OVL_LOAD = 0x02034AC4

GENERIC = re.compile(r"^(FUN|sub|LAB|thunk|caseD|PTR|DAT|default|switch)")
SRCRE = re.compile(r"^[A-Za-z0-9_]+\.(c|h|cpp)$")


# ---------------------------------------------------------------- source file -> (subsystem, scope, phase)
# scope: in | out (D9) | sdk (X3).  Subsystem names group the summary table.
def _m(sub, scope, phase):
    return (sub, scope, phase)


SRC_MAP = {}
for names, v in [
    ("gfl_use.c", _m("boot / system init (gfl_use)", "in", "G1/G2")),
    ("pm_voice.c perapvoice.c snd_mic.c bgm_info.c player_volume_fader.c field_sound_system.c ringtone_sys.c iss_city_sys.c iss_city_unit.c iss_road_sys.c iss_dungeon_sys.c iss_sys.c iss_3ds_sys.c iss_zone_sys.c iss_switch_sys.c iss_switch_set.c iss_switch.c", _m("sound / ISS", "in", "A9/H")),
    ("save_control.c backup_system.c flash_access.c savedata.c", _m("save system (savedata)", "in", "S5")),
    ("box_savedata.c", _m("PC box data", "in", "S2")),
    ("myitem_savedata.c bag_param.c", _m("bag / items", "in", "S3")),
    ("config.c mystatus.c playtime.c", _m("save: misc blocks", "in", "S5")),
    ("mail.c regulation.c poke_regulation.c symbol_save.c bsubway_savedata.c battle_examination.c battle_rec.c tpoke_data.c tr_tool.c", _m("save: misc blocks / trainer data", "in", "S5/B")),
    ("wifilist.c mystery_data.c musical_dist_save.c gds_profile.c wih_dwc.c net_save.c net_err.c net_system.c net_handle.c net_command.c net_state.c net.c net_irc_wireless.c net_lower_data.c net_ring_buff.c net_queue.c game_comm.c game_beacon.c beacon_status.c comm_player_support.c wm_icon.c", _m("networking / C-Gear / minigames (D9)", "out", "X1")),
    ("vm.c", _m("script VM", "in", "F7")),
    ("ica_anime.c", _m("ICA animation", "in", "U")),
    ("game_event.c game_data.c field_status.c", _m("game data / event flags", "in", "F")),
    ("btl_setup.c", _m("battle setup", "in", "B")),
    ("mcss.c", _m("MCSS (multi-cell sprites)", "in", "G/U")),
    ("zonedata.c", _m("zone headers", "in", "F2")),
    ("poke_tool.c", _m("poke_tool (PokeParam / BoxPokemon)", "in", "P1")),
    ("pokeparty.c", _m("party", "in", "S1")),
    ("poke_personal.c", _m("poke_personal", "in", "P2")),
    ("shinka_check.c", _m("evolution (shinka_check)", "in", "P5")),
    ("waza_tool.c", _m("waza_tool (moves)", "in", "P4")),
    ("printsys.c gf_font.c systemfont.c wordset.c strbuf.c msgdata.c textprint.c pmsi_param.c pms_word.c pms_draw.c", _m("text / message system", "in", "F9")),
    ("bmp_menuwork.c bmp_menu.c bmp_menulist.c bmp_cursor.c infowin.c cursor_move.c app_taskmenu.c app_keycursor.c bmp_oam.c bmp.c bmp_win.c bgwinfrm.c", _m("UI toolkit (bmp / menus)", "in", "G")),
    ("file_util.c arc_tool.c arc_util.c gf_overlay.c gf_standard.c procsys.c vman.c areaman.c", _m("gflib core (arc / overlay / proc / heap)", "in", "G/Z")),
    ("palanm.c wipe.c wipe_sub.c blink_palanm.c time_icon.c screentex.c fade.c double3Ddisp.c billboard.c billboard_act.c particle.c g3d_system.c g3d_util.c g3d_camera.c g3d_light.c clact.c actor_tool.c bg_sys.c pokegra.c codein_param.c button_man.c tcbl.c", _m("gflib graphics (G2/G3, actors, fades)", "in", "G")),
    ("ui.c touchpanel.c key.c", _m("input (key / touch)", "in", "G")),
]:
    for n in names.split():
        SRC_MAP[n] = v

# overlay-resident source files seen in overlay strings / OVERLAY-SETS
OVL_SRC_PREFIX = [
    (r"^(net_|dwc_|wifi|wifi_|wificlub|event_gtsnego|event_wifi|union_|colosseum|intrude_|event_comm|c_gear|no_gear|beacon_|mb_|mbp|nhttp|ghttp|gds_|gsync|gtsnego|mystery|worldtrade|guru2|irc_|ircbattle|compatible_irc|vs_multi|musical|mus_|pokemontrade|wih|livebattle|battle_championship|digitalcard|br_|research_|ctvt|local_tvt|camera_system|draw_system|dup_|sta_|cg_help|wf2dmap|wifi_2dchar|oekaki|mictest|comm_btl|wifibattlematch)", ("networking / C-Gear / minigames (D9)", "out", "X1/X2")),
]


def sub_for_src(s):
    if s in SRC_MAP:
        return SRC_MAP[s]
    for pat, v in OVL_SRC_PREFIX:
        if re.match(pat, s):
            return v
    return None


# ---------------------------------------------------------------- inputs
def read_main():
    return open(os.path.join(BUILD, "main.sbin"), "rb").read()


class Img(object):
    def __init__(self, name, base, data):
        self.name, self.base, self.data = name, base, data
        self.end = base + len(data)
        self.words = array.array("I")
        n = len(data) // 4
        self.words.frombytes(data[: n * 4])

    def cstr(self, addr):
        o = addr - self.base
        d = self.data
        if o <= 0 or o >= len(d) or (d[o - 1] != 0 and o % 4 != 0):
            return None
        e = d.find(b"\0", o)
        if e < 0 or e - o < 4 or e - o > 120:
            return None
        s = d[o:e]
        if all(32 <= c < 127 for c in s):
            return s.decode()
        return None


def parse_lsf():
    objs = []
    for l in open(os.path.join(ROM, "main.lsf")):
        if "Autoload" in l:
            break
        m = re.match(r"\s*Object\s+(\S+)", l)
        if m:
            objs.append(m.group(1))
    out = []
    for o in objs:
        m = re.search(r"unk_([0-9A-F]{8})\.o$", o)
        out.append((int(m.group(1), 16), os.path.basename(o)[:-2] + ".s", o.startswith("src/")))
    return out


def asm_fn_count(path):
    n = 0
    rx = re.compile(r"^\s*(non_word_aligned_)?(thumb|arm)_func_start\b")
    for l in open(path, errors="replace"):
        if rx.match(l):
            n += 1
    return n


def load_sets():
    sets = collections.OrderedDict()
    for fn in ("sets.txt", "sets.proposed.txt"):
        for d in (HERE, MAIN_CHECKOUT + "/black-rust/tools/re"):
            p = os.path.join(d, fn)
            if os.path.exists(p):
                for l in open(p):
                    f = l.split()
                    if f and not l.startswith("#") and f[0] not in sets:
                        sets[f[0]] = [int(x) for x in f[1:] if x.isdigit()], "main" in f
                break
    return sets


def pc_dir(name):
    for base in (RE_OUT, RE_Z12):
        d = os.path.join(base, name, "pc")
        if os.path.exists(os.path.join(d, "index.jsonl")):
            return d
    return None


class SetData(object):
    """Ghidra index of one set: functions sorted by address, name<->addr, reverse call graph."""

    def __init__(self, name, mods, has_main):
        self.name, self.mods, self.has_main = name, mods, has_main
        self.dir = pc_dir(name)
        self.fn = []  # (addr, name, size, ncallers, callees)
        for l in open(os.path.join(self.dir, "index.jsonl")):
            j = json.loads(l)
            self.fn.append((int(j["addr"], 16), j["name"], j["size"], j["callers"], j["callees"]))
        self.fn.sort(key=lambda t: t[0])
        self.keys = [f[0] for f in self.fn]
        self.n2a = {}
        for f in self.fn:
            self.n2a[f[1].lower()] = f[0]
        self.rev = None

    def reverse(self):
        if self.rev is None:
            self.rev = collections.defaultdict(list)
            for a, nm, sz, nc, cs in self.fn:
                for c in cs:
                    ca = self.n2a.get(c.lower())
                    if ca is not None:
                        self.rev[ca].append(a)
        return self.rev

    def in_range(self, lo, hi):
        i = bisect.bisect_left(self.keys, lo)
        j = bisect.bisect_left(self.keys, hi)
        return self.fn[i:j]

    def owner(self, addr):
        k = bisect.bisect_right(self.keys, addr) - 1
        return self.fn[k] if k >= 0 else None

    def pc_path(self, addr, name):
        return os.path.join(self.dir, "%08x_%s.c" % (addr, name))


# ---------------------------------------------------------------- pseudo-C call argument parsing
CALL = re.compile(r"\b(?:(?:thunk_)?(?:EXT_)?(?:FUN|sub)_(?:overlay_d_\d+__)?|func_0x)([0-9A-Fa-f]{8})\(([^;{}]*?)\);")


def split_args(s):
    out, d, cur = [], 0, ""
    for c in s:
        if c == "(":
            d += 1
        elif c == ")":
            d -= 1
        if c == "," and d == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
    out.append(cur.strip())
    return out


def numarg(x):
    x = re.sub(r"^\(\w+\s*\*?\)", "", x.strip())
    try:
        return int(x, 0)
    except ValueError:
        return None


def calls_in(text):
    i = text.find("\n{")
    if i > 0:
        text = text[i:]
    for m in CALL.finditer(text):
        a = int(m.group(1), 16)
        yield a, split_args(m.group(2))


# ---------------------------------------------------------------- feature extraction
def scan_region(img, lo, hi, imgs):
    """Strings pointed to from [lo,hi) words (resolved against imgs) + strings inside + hw-reg pool words."""
    refs = collections.Counter()
    ref_sites = collections.defaultdict(list)  # string -> [pool addr]
    hw = 0
    w = img.words
    i0 = (lo - img.base + 3) // 4
    i1 = (hi - img.base) // 4
    for i in range(i0, i1):
        v = w[i]
        if v < 0x02000000:
            if 0x04000000 <= v < 0x04002000:
                hw += 1
            continue
        if v > 0x023FFFFF:
            if 0x04000000 <= v < 0x04002000:
                hw += 1
            continue
        for im in imgs:
            if im.base <= v < im.end:
                s = im.cstr(v)
                if s is not None:
                    refs[s] += 1
                    ref_sites[s].append(img.base + 4 * i)
                break
    own = []
    d = img.data
    for m in re.finditer(rb"[\x20-\x7e]{4,100}\x00", d[lo - img.base: hi - img.base]):
        s = m.group(0)[:-1].decode()
        if SRCRE.match(s):
            own.append(s)
    return refs, ref_sites, own, hw


def pretty(a):
    return "%08X" % a


def short_sub(sub):
    return sub.split(" (")[0] if sub else "?"


# ---------------------------------------------------------------- unanchored ARM9 bands (evidence in the text)
BANDS = [
    (0x02004000, 0x02006118, "boot / system init (gfl_use)", "in", "G1/G2/A9", "med",
     "main() init FUN_0200545c (heap, ArcSys_Init, overlay table); string wb_sound_data.sdat at the SDAT loader"),
    (0x020520FC, 0x020583CC, "gflib 3D math helpers", "in", "G", "low",
     "matrix/vector math over SDK MTX/VEC/FX calls; callers are gflib code and field overlays; no strings"),
    (0x020583CC, 0x02062634, "DWC / GameSpy network library", "out", "X1", "med",
     "strings sdkdev.gamespy.com, gamestats.gs.nintendowifi.net, NWCUSBAP, NDWCSHAP; 64-bit LCG constants (DWC random); "
     "callers only inside the band and net_*.c"),
    (0x02062634, 0x02072000, "NNS G3D/G2D middleware", "sdk", "X3", "low",
     "called by g3d_*.c, mcss.c, clact.c and field overlays; no strings; rendering is rewritten (D3)"),
    (0x02072000, 0x02078360, "NNS sound + SDK CARD backup", "sdk", "X3", "low",
     "string [SDK+NINTENDO:BACKUP at 0x02075DC8; callers are the SDAT sound init (0x020055F8..) and flash_access.c"),
    (0x02078360, 0x0208E010, "NitroSDK (FS/OS/MI/GX/PXI/RTC/SPI)", "sdk", "X3", "med",
     "FS_LoadOverlay 0x02079264, MI_CpuCopy8 0x02082D44, hardware-register pool words, called from everywhere"),
    (0x0208E010, 0x0209B750, "C library + MWCC runtime (printf/strtod/string/math)", "sdk", "X3", "low",
     "strings INFINITY/NAN( and the -INF/-NAN tables at 0x020923F0/0x020949F4 (printf/strtod); internal callers only; "
     "runtime helpers _ll_mul/_s32_div_f/_u32_div_f follow"),
    (0x0209B750, 0x0209CE10, "MWCC runtime library", "sdk", "X3", "high",
     "_ll_mul 0x0209C054, _s32_div_f 0x0209C0A4, _u32_div_f 0x0209C2B0 (7 in the decomp notes)"),
    (0x0209CE10, 0x02100000, "ARM9 rodata / debug-name table", "sdk", "X3", "high",
     "holds the debug source-file name table (0x020A7004..), NARC path table 0x020A6C58"),
]


def band_for(a):
    for lo, hi, sub, scope, ph, conf, ev in BANDS:
        if lo <= a < hi:
            return sub, scope, ph, conf, ev
    return None


class Anchors(object):
    """Debug source-file strings referenced from ARM9 literal pools: (pool addr, owner fn start, file)."""

    def __init__(self, ctx):
        mimg, field = ctx["mimg"], ctx["field"]
        lo, hi = 0x02004FD4, ctx["starts"][-1]
        _refs, sites, _own, _hw = scan_region(mimg, lo, hi, [mimg])
        self.items = []
        for s, pas in sites.items():
            if SRCRE.match(s):
                for pa in pas:
                    f = field.owner(pa)
                    self.items.append((pa, f[0] if f else pa, s))
        self.items.sort()
        self.pas = [t[0] for t in self.items]
        self.graph = {}
        self._refine(field)

    def _refine(self, field):
        """For functions between two different anchors: vote by already-labelled call-graph neighbours."""
        rev = field.reverse()
        label = {}
        gaps = []
        for a, nm, sz, nc, cs in field.fn:
            if a >= OVL_MIN or re.match(r"^(LAB|caseD|default|switch)", nm):
                continue
            k, sa, sb, _p, _n = self.label(a)
            if k in ("exact", "bracket"):
                label[a] = sa
            elif k == "between" and sa != sb:
                gaps.append((a, sa, sb, cs))
        for _round in range(3):
            changed = 0
            for a, sa, sb, cs in gaps:
                if a in self.graph:
                    continue
                votes = collections.Counter()
                nb = [field.n2a.get(c.lower()) for c in cs] + list(rev.get(a, []))
                for x in nb:
                    s2 = label.get(x) or self.graph.get(x)
                    if s2 in (sa, sb):
                        votes[s2] += 1
                if votes:
                    top = votes.most_common()
                    if len(top) == 1 or top[0][1] >= 2 * top[1][1]:
                        self.graph[a] = top[0][0]
                        changed += 1
            if not changed:
                break

    def label(self, a):
        """-> (kind, srcA, srcB, prev_pool, next_fn_start); kind = exact | bracket | between | before | after."""
        k = bisect.bisect_left(self.pas, a)
        best = None
        for j in range(max(0, k - 3), min(len(self.items), k + 4)):
            pa, fs, s = self.items[j]
            if fs <= a <= pa + 3 and (best is None or pa - fs < best[0]):
                best = (pa - fs, s)
        if best:
            return ("exact", best[1], best[1], 0, 0)
        pj = k - 1 if k > 0 else None
        nj = None
        for j in range(k, len(self.items)):
            if self.items[j][1] > a:
                nj = j
                break
        prev = self.items[pj][2] if pj is not None else None
        nxt = self.items[nj][2] if nj is not None else None
        ppa = self.items[pj][0] if pj is not None else 0
        nfs = self.items[nj][1] if nj is not None else 0
        if nxt is None:
            return ("after", prev, None, ppa, 0)
        if prev is None:
            return ("before", None, nxt, 0, nfs)
        return ("bracket" if prev == nxt else "between", prev, nxt, ppa, nfs)


def classify_main(anchors, a):
    """Subsystem of one ARM9 address -> dict(sub, scope, phase, src, conf, how)."""
    k, sa, sb, ppa, nfs = anchors.label(a)
    if k in ("exact", "bracket"):
        v = sub_for_src(sa) or ("unmapped source file " + sa, "in", "?")
        return dict(sub=v[0], scope=v[1], phase=v[2], src=sa, conf="high" if k == "exact" else "med", how=k)
    if k == "between":
        va, vb = sub_for_src(sa), sub_for_src(sb)
        src = "%s|%s" % (sa, sb)
        if va and vb and va[0] == vb[0]:
            return dict(sub=va[0], scope=va[1], phase=va[2], src=src, conf="med", how="between-same")
        g = anchors.graph.get(a)
        if g in (sa, sb):
            v = (va if g == sa else vb) or ("unmapped source file " + g, "in", "?")
            return dict(sub=v[0], scope=v[1], phase=v[2], src=g + "?", conf="med", how="between-graph")
        pick = (vb, sb) if (nfs - a) <= 0x200 else (va, sa)
        v = pick[0] or ("unmapped source file " + pick[1], "in", "?")
        return dict(sub=v[0], scope=v[1], phase=v[2], src=src, conf="low", how="between-nearest")
    b = band_for(a)
    if b and (k == "after" or a < 0x02006118):
        return dict(sub=b[0], scope=b[1], phase=b[2], src="", conf=b[3], how="band")
    if k == "before":
        v = sub_for_src(sb) or ("unmapped source file " + sb, "in", "?")
        return dict(sub=v[0], scope=v[1], phase=v[2], src="?|" + sb, conf="low", how="before")
    if k == "after" and sa:
        v = sub_for_src(sa) or ("unmapped source file " + sa, "in", "?")
        return dict(sub=v[0], scope=v[1], phase=v[2], src=sa + "|?", conf="low", how="after-last-anchor")
    return dict(sub="unknown", scope="?", phase="?", src="", conf="low", how="none")


def apportion(total, weights):
    s = float(sum(weights.values())) or 1.0
    raw = dict((k, total * v / s) for k, v in weights.items())
    out = dict((k, int(v)) for k, v in raw.items())
    rest = total - sum(out.values())
    for k in sorted(raw, key=lambda k: raw[k] - out[k], reverse=True)[:rest]:
        out[k] += 1
    return out


def load_overrides():
    ov = {}
    p = os.path.join(HERE, "T1-overrides.tsv")
    if os.path.exists(p):
        for l in open(p):
            if l.startswith("#") or not l.strip():
                continue
            f = l.rstrip("\n").split("\t")
            while len(f) < 8:
                f.append("")
            ov[f[0]] = f
    return ov


def load_unid():
    rows = []
    for l in open(os.path.join(HERE, "T1-unidentified.tsv")):
        if l.startswith("#") or not l.strip():
            continue
        f = l.rstrip("\n").split("\t")
        rows.append((f[0], f[1], int(f[2])))
    return rows


def ovl_files():
    out = {}
    for fn in os.listdir(os.path.join(ROM, "asm")):
        m = re.match(r"overlay_(\d{3})_([0-9A-F]{8})\.s$", fn)
        if m:
            out[int(m.group(1))] = (fn, int(m.group(2), 16))
    return out


def bl_targets(img):
    """Call targets of Thumb BL/BLX pairs and ARM BL/BLX imm in an image (linear sweep; noisy on data)."""
    out = collections.Counter()
    d = img.data
    n = len(d) // 2
    h = array.array("H")
    h.frombytes(d[: n * 2])
    for i in range(n - 1):
        a = h[i]
        if a & 0xF800 == 0xF000:
            b = h[i + 1]
            if b & 0xF800 in (0xF800, 0xE800):
                off = ((a & 0x7FF) << 12) | ((b & 0x7FF) << 1)
                if off & 0x400000:
                    off -= 0x800000
                t = img.base + 2 * i + 4 + off
                if b & 0xF800 == 0xE800:
                    t &= ~3
                out[t & ~1] += 1
    w = img.words
    for i, v in enumerate(w):
        if v >> 24 == 0xEB:  # ARM BL (cond AL)
            off = (v & 0xFFFFFF) << 2
            if off & 0x2000000:
                off -= 0x4000000
            out[img.base + 4 * i + 8 + off] += 1
    return out


def asm_starts(path):
    """Function start addresses from the `LABEL: ; 0xADDR` comments in an asm file."""
    out = set()
    rx = re.compile(r"^\w+:\s*;\s*0x([0-9A-Fa-f]{8})")
    for l in open(path, errors="replace"):
        m = rx.match(l)
        if m:
            out.add(int(m.group(1), 16))
    return out


def build_ctx():
    mimg = Img("main", MAIN_BASE, read_main())
    lsf = parse_lsf()
    starts = [a for a, _, _ in lsf]
    ovf = ovl_files()
    oimgs = {}
    for n, (fn, addr) in ovf.items():
        oimgs[n] = Img("ov%03d" % n, addr, open(os.path.join(BUILD, "OVY_%d.sbin" % n), "rb").read())
    sets = load_sets()
    cache = {}

    def getset(name):
        if name not in cache:
            cache[name] = SetData(name, sets[name][0], sets[name][1]) if pc_dir(name) else None
        return cache[name]

    def pick_set(n):
        cands = [k for k, (mods, _) in sets.items() if n in mods]
        cands.sort(key=lambda k: (k.startswith("pack-"), list(sets).index(k)))
        for k in cands:
            if pc_dir(k):
                return k
        return None

    ctx = dict(mimg=mimg, oimgs=oimgs, sets=sets, getset=getset, pick_set=pick_set, field=getset("field"),
               starts=starts, lsf=lsf, ovf=ovf)
    ctx["anchors"] = Anchors(ctx)
    return ctx


def arm9_key(ctx, a):
    k = bisect.bisect_right(ctx["starts"], a) - 1
    return "u%08X" % ctx["starts"][k] if k >= 0 else "crt0"


def key_in_set(ctx, sd, a):
    if a < OVL_MIN:
        return arm9_key(ctx, a)
    for n in sd.mods if sd else []:
        im = ctx["oimgs"].get(n)
        if im and im.base <= a < im.end:
            return "ov%03d" % n
    return "ov?"


def grep_files(d, pattern):
    import subprocess
    r = subprocess.run(["grep", "-rliE", pattern, d, "--include=*.c"], stdout=subprocess.PIPE)
    return [x for x in r.stdout.decode().split("\n") if x]


def global_scan(ctx):
    """overlay_load sites and ARC / message-bank users over every decompiled set."""
    pat = "(FUN_|sub_|FUN_overlay_d_[0-9]+__|func_0x)(%s)\\(" % "|".join(
        "%08X" % a for a in sorted(list(ARC_LOADERS) + [OVL_LOAD, MSG_CREATE]))
    load_sites = collections.defaultdict(set)
    arc_users = collections.defaultdict(collections.Counter)
    arc_fns = collections.defaultdict(list)
    msg_users = collections.defaultdict(collections.Counter)
    msg_fns = collections.defaultdict(list)
    seen = set()
    for sname in ctx["sets"]:
        sd = ctx["getset"](sname)
        if sd is None:
            continue
        for path in grep_files(sd.dir, pat):
            a = int(os.path.basename(path)[:8], 16)
            if a < OVL_MIN and sname != "field":
                continue
            if (sname if a >= OVL_MIN else "main", a) in seen:
                continue
            seen.add((sname if a >= OVL_MIN else "main", a))
            key = key_in_set(ctx, sd, a)
            try:
                txt = open(path, errors="replace").read()
            except IOError:
                continue
            for ca, ar in calls_in(txt):
                if ca == OVL_LOAD and ar and numarg(ar[0]) is not None:
                    load_sites[numarg(ar[0])].add((key, a, sname))
                elif ca in ARC_LOADERS:
                    i = ARC_LOADERS[ca]
                    v = numarg(ar[i]) if len(ar) > i else None
                    if v is not None and 0 <= v < 256:
                        arc_users[v][key] += 1
                        arc_fns[v].append((a, sname, key))
                elif ca == MSG_CREATE and len(ar) > 2:
                    t, b = numarg(ar[1]), numarg(ar[2])
                    if t in (2, 3) and b is not None:
                        msg_users[(t, b)][key] += 1
                        msg_fns[(t, b)].append((a, sname, key))
    return dict(load_sites=load_sites, arc_users=arc_users, arc_fns=arc_fns, msg_users=msg_users, msg_fns=msg_fns)


def proc_structs(ctx):
    """3 consecutive Thumb pointers into one overlay's function starts (GFL_PROC init/main/exit data)."""
    ovstarts = {}
    for n, (fn, addr) in ctx["ovf"].items():
        im = ctx["oimgs"][n]
        if len(im.data) > 64:
            ovstarts[n] = (im.base, im.end, asm_starts(os.path.join(ROM, "asm", fn)))
    cands = []  # (src image name, struct addr, target overlay id)
    for im in [ctx["mimg"]] + [ctx["oimgs"][n] for n in sorted(ctx["oimgs"])]:
        w = im.words
        for i in range(len(w) - 2):
            v = w[i]
            if not (v & 1 and OVL_MIN <= v < 0x02300000):
                continue
            v1, v2 = w[i + 1], w[i + 2]
            if not (v1 & 1 and v2 & 1):
                continue
            for n, (lo, hi, st) in ovstarts.items():
                if lo <= v < hi and lo <= v1 < hi and lo <= v2 < hi and (v & ~1) in st and (v1 & ~1) in st and (v2 & ~1) in st:
                    if im.name == "ov%03d" % n:
                        continue
                    cands.append((im.name, im.base + 4 * i, n))
    by_addr = collections.defaultdict(list)
    for src, a, n in cands:
        by_addr[a].append((src, n))
    launch = collections.defaultdict(list)  # target ovl -> ["src@addr", ...] referencing the struct
    for im in [ctx["mimg"]] + [ctx["oimgs"][n] for n in sorted(ctx["oimgs"])]:
        w = im.words
        for i, v in enumerate(w):
            if v in by_addr:
                for src, n in by_addr[v]:
                    launch[n].append("%s@%08X" % (im.name, im.base + 4 * i))
    defs = collections.defaultdict(list)
    for src, a, n in cands:
        defs[n].append("%s@%08X" % (src, a))
    return defs, launch


def file_stats(ctx, sd, lo, hi, kind, ovn, glob_):
    """Per-file call-graph features from the pseudo-C of every function in [lo, hi)."""
    A = ctx["anchors"]
    r = dict(ghidra_fns=0, named_fns=0, arc={}, msg={}, alloc={}, callees=[], callers=[], callee_files=[], caller_files=[],
             callee_subs=[], comp={}, srcs=[], runs=[])
    if sd is None:
        return r
    fns = [f for f in sd.in_range(lo, hi) if not re.match(r"^(LAB|caseD|default|switch)", f[1])]
    r["ghidra_fns"] = len(fns)
    r["named_fns"] = sum(1 for f in fns if not GENERIC.match(f[1]))
    arc, msg, alloc = collections.Counter(), collections.Counter(), collections.Counter()
    cal, subs = collections.Counter(), collections.Counter()
    for a, nm, sz, nc, cs in fns:
        try:
            txt = open(sd.pc_path(a, nm), errors="replace").read()
        except IOError:
            continue
        for ca, ar in calls_in(txt):
            if ca in ARC_LOADERS:
                i = ARC_LOADERS[ca]
                v = numarg(ar[i]) if len(ar) > i else None
                if v is not None and 0 <= v < 256:
                    arc[v] += 1
            elif ca == MSG_CREATE and len(ar) > 2:
                t, b = numarg(ar[1]), numarg(ar[2])
                if t in (2, 3) and b is not None:
                    msg["%d/%d" % (t, b)] += 1
            elif ca == ALLOC and len(ar) > 1:
                v = numarg(ar[1])
                if v is not None:
                    alloc[v] += 1
            if not (lo <= ca < hi):
                cal[ca] += 1
                if ca < OVL_MIN and ca not in HUBS:
                    c = classify_main(A, ca)
                    subs[short_sub(c["sub"])] += 1
    r["arc"], r["msg"], r["alloc"] = dict(arc), dict(msg), dict(alloc)

    def nm_of(a):
        if a in HUBS:
            return HUBS[a]
        k = key_in_set(ctx, sd, a)
        if a < OVL_MIN:
            c = classify_main(A, a)
            return "%08X[%s]" % (a, c["src"].split("|")[0] if c["src"] else short_sub(c["sub"])[:24])
        return "%08X[%s]" % (a, k)

    r["callees"] = [[nm_of(a), n] for a, n in cal.most_common(10)]
    r["callee_subs"] = [[k, n] for k, n in subs.most_common(6)]
    clf = collections.Counter()
    for a, n in cal.items():
        clf[key_in_set(ctx, sd, a)] += n
    me = "u%08X" % lo if kind == "arm9" else "ov%03d" % ovn
    r["callee_files"] = [[k, n] for k, n in clf.most_common(8) if k != me]
    rev = sd.reverse()
    ext, extf = collections.Counter(), collections.Counter()
    for a, nm, sz, nc, cs in fns:
        for c in rev.get(a, []):
            if not (lo <= c < hi):
                ext[c] += 1
                extf[key_in_set(ctx, sd, c)] += 1
    r["callers"] = [[nm_of(a), n] for a, n in ext.most_common(10)]
    r["caller_files"] = [[k, n] for k, n in extf.most_common(8)]
    if kind == "arm9":
        comp = collections.Counter()
        srcs = []
        for a, nm, sz, nc, cs in fns:
            c = classify_main(A, a)
            comp[(c["sub"], c["scope"], c["phase"], c["conf"])] += 1
            if c["how"] in ("exact", "bracket") and c["src"] not in srcs:
                srcs.append(c["src"])
        r["comp"] = [[list(k), n] for k, n in comp.most_common()]
        r["srcs"] = srcs
        runs = []
        for a, nm, sz, nc, cs in fns:
            c = classify_main(A, a)
            tag = (c["src"] or short_sub(c["sub"]), c["how"])
            if runs and runs[-1][2] == tag[0] and runs[-1][3] == tag[1]:
                runs[-1][1] = a
                runs[-1][4] += 1
            else:
                runs.append([a, a, tag[0], tag[1], 1])
        r["runs"] = runs
    return r


def extract(ctx):
    A = ctx["anchors"]
    unid = load_unid()
    mimg, oimgs, sets, ovf = ctx["mimg"], ctx["oimgs"], ctx["sets"], ctx["ovf"]
    starts, lsf = ctx["starts"], ctx["lsf"]
    glob_ = global_scan(ctx)
    defs, launch = proc_structs(ctx)
    feats = []
    for fn, oldcls, oldfns in unid:
        base = os.path.basename(fn)
        rec = {"file": base, "old_class": oldcls.strip(), "old_fns": oldfns}
        rec["asm_fns"] = asm_fn_count(os.path.join(ROM, fn))
        if base.startswith("unk_"):
            lo = int(base[4:12], 16)
            k = starts.index(lo)
            hi = starts[k + 1] if k + 1 < len(starts) else mimg.end
            rec.update(kind="arm9", ovl=None, set="main", lo=lo, hi=hi, prev=lsf[k - 1][1] if k else "crt0",
                       next=lsf[k + 1][1] if k + 1 < len(lsf) else "-")
            img, imgs, sd = mimg, [mimg], ctx["field"]
        else:
            m = re.match(r"overlay_(\d{3})_([0-9A-F]{8})\.s$", base)
            n = int(m.group(1))
            im = oimgs[n]
            sname = ctx["pick_set"](n)
            rec.update(kind="ovl", ovl=n, set=sname or "-", lo=im.base, hi=im.end,
                       prev=ovf[n - 1][0] if n - 1 in ovf else "-", next=ovf[n + 1][0] if n + 1 in ovf else "-")
            sd = ctx["getset"](sname) if sname else None
            imgs = [im, mimg] + [oimgs[x] for x in (sets[sname][0] if sname else []) if x in oimgs and x != n]
            img = im
        rec["size"] = rec["hi"] - rec["lo"]
        refs, sites, own, hw = scan_region(img, rec["lo"], rec["hi"], imgs)
        rec["src_refs"] = dict((s, c) for s, c in refs.items() if SRCRE.match(s))
        rec["src_sites"] = dict((s, sorted(v)) for s, v in sites.items() if SRCRE.match(s))
        rec["other_strs"] = [s for s, c in refs.most_common() if not SRCRE.match(s)
                             and sum(ch.isalpha() or ch == " " for ch in s) >= 0.7 * len(s) and len(s) >= 6][:6]
        rec["own_src"] = sorted(set(own))
        rec["hwregs"] = hw
        rec.update(file_stats(ctx, sd, rec["lo"], rec["hi"], rec["kind"], rec.get("ovl"), glob_))
        if rec["kind"] == "ovl":
            n = rec["ovl"]
            rec["load_sites"] = sorted("%s@%08X" % (kk, aa) for kk, aa, ss in glob_["load_sites"].get(n, []))[:6]
            rec["proc_defs"] = defs.get(n, [])[:4]
            rec["launched_by"] = launch.get(n, [])[:6]
            if sd is None and rec["asm_fns"] > 0:
                bt = bl_targets(im)
                subs, hubs = collections.Counter(), collections.Counter()
                for t, c in bt.items():
                    if 0x02004FD4 <= t < starts[-1]:
                        if t in HUBS:
                            hubs[HUBS[t]] += c
                        else:
                            subs[short_sub(classify_main(A, t)["sub"])] += c
                rec["callee_subs"] = [[k, n2] for k, n2 in subs.most_common(6)]
                rec["callees"] = [[k, n2] for k, n2 in hubs.most_common(6)]
        feats.append(rec)
    return feats, glob_


# ---------------------------------------------------------------- labelling
TRANSL = MAIN_CHECKOUT + "/black-translations/0000"


def bank_preview(t, b):
    """First text lines of a system message bank (ARC 002) from black-translations, or ''.

    Off unless TRIAGE_PREVIEW=1: the outputs are committed to a public repo, so game text stays
    out of them (D5)."""
    if t != 2 or os.environ.get("TRIAGE_PREVIEW") != "1":
        return ""
    p = os.path.join(TRANSL, "%04d.txt" % b)
    if not os.path.exists(p):
        return ""
    out = []
    for l in open(p, encoding="utf-8-sig", errors="replace"):
        l = re.sub(r"\\x[0-9a-fA-F]{4}|[^\x20-\x7e]", "", l.strip().replace("\\xfffe", " "))
        if len(l) >= 3 and not set(l) <= set("-_ "):
            out.append(l[:20])
        if len(out) >= 2:
            break
    return " / ".join(out)


def label_arm9(rec):
    comp = rec["comp"]
    tot = float(sum(n for _, n in comp)) or 1.0
    bysub = collections.Counter()
    conf = collections.Counter()
    for (sub, scope, phase, cf), n in comp:
        bysub[(sub, scope, phase)] += n
        conf[cf] += n
    ranked = bysub.most_common()
    (sub, scope, phase), n0 = ranked[0]
    share = n0 / tot
    label = sub
    if len(ranked) > 1 and ranked[1][1] / tot >= 0.2:
        label += " + " + ranked[1][0][0]
    scopes = collections.Counter()
    phases = []
    for (s, sc, ph), n in ranked:
        scopes[sc] += n
        if n / tot >= 0.15 and ph not in phases:
            phases.append(ph)
    sc = scopes.most_common(1)[0][0] if len(scopes) == 1 else "mixed(" + "/".join("%s:%d" % kv for kv in scopes.most_common()) + ")"
    good = (conf["high"] + conf["med"]) / tot
    if share >= 0.7 and good >= 0.6 and conf["high"] / tot >= 0.15:
        cf = "high"
    elif share >= 0.5 and good >= 0.5:
        cf = "med"
    else:
        cf = "low"
    if all(c[0][3] == "high" or c[0][3] == "med" for c in comp) and rec["srcs"] == [] and share >= 0.9 and good == 1.0:
        cf = "med"
    segs = "; ".join("%s:%d" % (s[:30], n) for (s, _, _), n in ranked[:4])
    if rec["srcs"]:
        srcs = ", ".join(rec["srcs"][:8])
    elif rec["lo"] >= 0x020520FC or rec["lo"] < 0x02006118:
        srcs = "(no source-file strings: address band)"
    else:
        srcs = "(between anchors)"
    return dict(subsystem=label, scope=sc, phase="/".join(phases) or phase, conf=cf, sources=srcs, segments=segs, share=share)


OVL_BASE_HINT = {
    0x021542C0: "base of the network/event overlays 0-8",
    0x021B6100: "app base (after 10+11)",
    0x021B95A0: "app base (after ui-common 67)",
    0x021EE740: "field child base (after 21)",
    0x021F36C0: "field gimmick base",
}


def label_ovl(rec):
    n = rec["ovl"]
    if rec["size"] <= 64 and rec["asm_fns"] == 0:
        return dict(subsystem="stub (32 bytes, no code)", scope="n/a", phase="-", conf="high", sources="",
                    segments="", why="size 32, 0 functions")
    srcs = sorted(set(rec["own_src"]) | set(rec["src_refs"]))
    for s in srcs:
        v = sub_for_src(s)
        if v:
            return dict(subsystem=v[0], scope=v[1], phase=v[2], conf="high", sources=", ".join(srcs), segments="",
                        why="debug source-file string " + s)
    subs = rec["callee_subs"]
    totc = float(sum(c for _, c in subs)) or 1.0
    netc = sum(c for s, c in subs if s.startswith("networking"))
    if netc / totc >= 0.25 and netc >= 4:
        return dict(subsystem="networking / C-Gear / minigames (D9)", scope="out", phase="X1/X2", conf="low", sources="",
                    segments="", why="%d%% of calls into main hit net_*.c / DWC code" % (100 * netc / totc))
    if rec["set"].startswith("pack-fld") and any(p.startswith("ov021@") for p in rec["proc_defs"]):
        return dict(subsystem="field gimmick / event overlay", scope="in", phase="F10", conf="med", sources="",
                    segments="", why="3-pointer proc table (init/main/exit) in overlay 21 targets its functions; field-child base")
    if rec["msg"] or rec["arc"] or subs:
        return dict(subsystem="UI app overlay (unnamed)", scope="in", phase="U", conf="low", sources="", segments="",
                    why="calls UI toolkit / text / graphics helpers; no debug strings")
    return dict(subsystem="unknown", scope="?", phase="?", conf="low", sources="", segments="", why="no strings, no callees")


def evidence_ovl(rec):
    bits = []
    if rec["set"] != "-":
        bits.append("set " + rec["set"])
    if rec["arc"]:
        bits.append("ARC " + ",".join(str(k) for k in sorted(rec["arc"], key=int)[:8]))
    if rec["msg"]:
        pv = []
        for k in sorted(rec["msg"]):
            t, b = [int(x) for x in k.split("/")]
            p = bank_preview(t, b)
            pv.append("%s%s" % (k, " '%s'" % p if p else ""))
        bits.append("msg " + "; ".join(pv[:4]))
    if rec["callee_subs"]:
        bits.append("calls " + ",".join("%s:%d" % (s[:16], c) for s, c in rec["callee_subs"][:4]))
    if rec.get("launched_by"):
        bits.append("launched by " + ",".join(rec["launched_by"][:2]))
    if rec.get("proc_defs"):
        bits.append("proc table in " + ",".join(rec["proc_defs"][:1]))
    if rec.get("load_sites"):
        bits.append("overlay_load from " + ",".join(rec["load_sites"][:2]))
    if rec["caller_files"]:
        bits.append("called by " + ",".join("%s:%d" % tuple(x) for x in rec["caller_files"][:2]))
    return "; ".join(bits)


def evidence_arm9(rec, lab):
    bits = []
    if rec["src_refs"]:
        bits.append("strings " + ",".join(rec["src_refs"]))
    bits.append("segments " + lab["segments"])
    if rec["arc"]:
        bits.append("ARC " + ",".join(str(k) for k in sorted(rec["arc"], key=int)[:6]))
    if rec["msg"]:
        bits.append("msg " + ",".join(sorted(rec["msg"])[:6]))
    if rec["alloc"]:
        bits.append("alloc sizes " + ",".join("%d" % int(k) for k in sorted(rec["alloc"], key=int)[:5]))
    if rec["other_strs"]:
        bits.append("str '%s'" % rec["other_strs"][0][:24])
    if rec["hwregs"] >= 3:
        bits.append("hw regs %d" % rec["hwregs"])
    return "; ".join(bits)


def compose(feats, overrides):
    rows = []
    for rec in feats:
        if rec["kind"] == "arm9":
            lab = label_arm9(rec)
            ev = evidence_arm9(rec, lab)
        else:
            lab = label_ovl(rec)
            ev = lab.pop("why") + ("; " + evidence_ovl(rec) if evidence_ovl(rec) else "")
        by = "script"
        key = rec["file"]
        if key in overrides:
            o = overrides[key]
            changed = False
            for fld, val in zip(("subsystem", "sources", "conf", "scope", "phase"), o[1:6]):
                if val:
                    lab[fld] = val
                    changed = True
            if o[6]:
                ev = o[6] + (" | script: " + ev if ev else "")
            by = "override" if changed else "script+note"
        if lab["conf"] == "unknown" or lab["subsystem"].startswith("unknown") or lab["scope"] == "?" or lab["subsystem"].startswith("unidentified"):
            by = "unknown"
        row = dict(rec)
        row.update(lab)
        row["evidence"] = ev
        row["label_by"] = by
        rows.append(row)
    return rows




LOCATED = [
    ("PokeParam / BoxPokemon (poke_tool.c)",
     "unk_02016BA0..unk_02019A50 (about 0x02016BA0-0x0201A26C); anchors FUN_020173F0, FUN_02017430, FUN_02018328",
     "pools 0x0201742C/0x02017478/0x02018428 name poke_tool.c; Heap_AllocDebug at FUN_020173F0 (line 0xA0) and FUN_02017430 (line 0xBA) with size FUN_020173EC() = 0xDC (220 B POKEMON_PARAM), FUN_02018328 size 0x68. "
     "Verified in pseudo-C: crypt FUN_02019A50 (XOR u16 stream) over LCG32 FUN_02019A7C (x*0x41C64E6D+0x6073, >>16), checksum FUN_02019A9C (u16 sum), "
     "decrypt-on-access FUN_02017DBC (flag bit 1 at +4, seed = checksum at +6, 0x80-byte block at +8) and re-encrypt FUN_02017DE4, "
     "getter FUN_02017E88 `PokeParaGet(pp,id,buf)`, setter FUN_02017E40 `PokeParaPut(pp,id,value)`, inner FUN_02018E34 / FUN_02019308"),
    ("Party (pokeparty.c)", "unk_0201A858 (0x0201A858-0x0201AF6C): FUN_0201A920, FUN_0201AB8C; accessors FUN_0201AA14..FUN_0201AB48",
     "pools 0x0201A944 (alloc line 0x41) and 0x0201AC28; FUN_0201AB8C allocates 0xDC = 220 bytes (party Pokemon struct); FUN_0201AA18 returns the member count (+4)"),
    ("Personal data access (poke_personal.c)", "unk_0201A858 tail + unk_0201AF6C head: FUN_0201AD88, FUN_0201ADA4, FUN_0201ADE4, **FUN_0201AE1C**, FUN_0201AFD4",
     "pool 0x0201AE10 (FUN_0201ADE4 allocs 0x3C = 60 B record, line 0x76). FUN_0201AE1C is the field getter: switch over ids 0..0x2B on the 0x3C-byte record (verified in pseudo-C). "
     "FUN_0201AD88 = ArcLoadMalloc(16, species) (ARC 016); FUN_0201ADA4 reads the 0x3C record from an open ARC 016 handle (`*DAT_0201adcc`); "
     "FUN_0201AFD4 maps (species, form) to the personal index using record +0x1C (first form) and +0x20 (form count)"),
    ("Level-up learnsets", "FUN_0201ADD0 (unk_0201A858)", "`ArcLoadData(dst, 18, ...)`: ARC 018, 668 records of (u16 move, u16 level)"),
    ("EXP curves / level (ARC 017)", "not located",
     "ARC 017 (8 x 101 u32) has no direct user among the verified loaders in main, field, battle or the Z12 sets; probably read through an open archive handle (like personal data). "
     "P3 should start from `PokeParaGet` ids (growth rate in the personal record, FUN_0201AE1C) and the battle EXP code. UNVERIFIED"),
    ("Evolution checks (shinka_check.c)", "unk_0201AF6C (0x0201AF6C-0x0201BAE4): FUN_0201B3B8, FUN_0201B406",
     "pool 0x0201B5D4 (FUN_0201B406); FUN_0201B406 steps entry+1 < 7: 7 evolution slots per species (ARC 019, 668 x 42 B); FUN_0201AF6C tests `0xFE/0xFF` form values. ARC 019 itself has no direct loader call (handle based)"),
    ("Move data (waza_tool.c)", "unk_0201BAE4 (0x0201BAE4-0x0201C41C): FUN_0201BC40, FUN_0201BD28, FUN_0201BF58, FUN_0201C020, anchor FUN_0201C1E8",
     "pool 0x0201C3BC; four functions call `ArcLoadData(dst, 21, move)` (ARC 021 move data, 36 B); FUN_0201BAE4/FUN_0201BABC call ArcMemberSize(7, id+1)"),
    ("PC box data (box_savedata.c)", "unk_02007AA4 (anchor FUN_02007AA4, alloc 4 B); logic probably unk_020071A8..unk_02007AA4",
     "pool 0x02007AC4; the box accessors sit in the 'between save_control.c and box_savedata.c' gap (call-graph attribution, low confidence). UI is overlay_170 (U phase)"),
    ("Bag data (myitem_savedata.c, bag_param.c, item data)", "unk_02007F7C (FUN_02008254, alloc 0x2C); unk_0201FC1C: bag_param.c FUN_0202E474, item data FUN_02020F34",
     "pools 0x0200826C and 0x0202E4E8; FUN_02020F34 calls FUN_02048F60(0x18, id) / (0x19, ...) = ARC 024 item data (36 B, 627 items) and ARC 025; pocket/sort code is in the gap before the anchors"),
    ("Save system", "save_control.c: unk_02006D3C (FUN_02006F20, pool 0x02007074); savedata.c / backup_system.c / flash_access.c: unk_0201FC1C (0x020316C8-0x02033BA4); CARD backup SDK: unk_02075DC8, unk_02076A70",
     "pools: savedata.c 0x020339B0/0x02033A80/0x02033B5C/0x02033BA4, backup_system.c 0x02031744..0x02032680, flash_access.c 0x020335A0..0x020337A4; string `[SDK+NINTENDO:BACKUP`. "
     "Per-block files: box_savedata, myitem_savedata, config, mystatus, playtime, mail, regulation, battle_rec, symbol_save, bsubway_savedata, battle_examination (unk_02007AA4-unk_0200E4CC); D9 blocks: wifilist, mystery_data, musical_dist_save, gds_profile, net_save"),
    ("RNG (MT and LCG)", "gf_standard.c in unk_0201FC1C (0x0203EF5C-0x0203F4B0): MT seed FUN_0203F034, MT next/temper FUN_0203F0A8, LCG64 FUN_0203F258 / FUN_0203F2FC, LCG32 FUN_0203F388 via FUN_0203F328",
     "constant scan below (pool words trail their function; the table column is only the nearest preceding Ghidra start): MT19937 (matrix 0x9908B0DF at 0x0203EFC4, init multiplier in FUN_0203F034 at 0x0203F0A4, tempering 0x9D2C5680 at 0x0203F1F4 in FUN_0203F0A8; FUN_0203F034 is called from boot 0x020056D0 and FUN_0203AE20, FUN_0203F0A8 has 7 callers incl. poke code 0x02017670/0x0201B170); LCG32 0x41C64E6D at 0x0203F3B0 (gf_standard) and 0x02019A94 (poke_tool crypt); "
     "LCG64 0x5D588B656C078965 + 0x269EC3 at 0x0203F2F0 (gf_standard), 0x0202A7C0/0x0202A984 (tr_tool.c / tpoke_data region), 0x0203AEE0 (net_state.c), DWC (0x02058D7C, 0x02058FA8, 0x02059638)"),
    ("String buffer / message system", "unk_02043900 (0x02043900-0x02048CEC): strbuf.c FUN_020457B0, msgdata.c FUN_02045B38 / FUN_02045CAC / FUN_02045EC0 / FUN_02045F5C; wordset.c unk_0201EC64; printsys.c unk_0201C41C",
     "pools strbuf.c 0x020457FC (StrBuf_Create(size, heap), 33 callers), msgdata.c 0x02045C00 (FUN_02045B38 = MsgData_Create(0, arc 2|3, bank, heap), alloc 0x1C, 35 callers), wordset.c 0x0201ED00 (FUN_0201EC74), printsys.c 0x0201C4B8 (FUN_0201C450 print queue alloc), gf_font.c 0x0201D838, systemfont.c 0x020462F8"),
    ("TM/HM list", "not located as a table",
     "no u16 run of TM01.. (item 328..) or HM01.. (420..) in ARM9 or any overlay (scan of main.sbin and all OVY_*.sbin); the TM move comes from the item record (ARC 024, loaded by FUN_02020F34). UNVERIFIED: fields and TM numbering -> §8"),
    ("Egg moves / baby forms", "ARC 123 (650 members, median 2 B, max 34 B) read in overlay 21; ARC 020 baby forms read by FUN_021C4504 (overlay 21)",
     "ARC 123 matches an egg-move list per species (A0 did not map it); only ARC users are field code (daycare/egg). Rust side: A-phase reader + P7"),
    ("Move tutor tables", "not located", "no ARC or ARM9/overlay table identified; tutors are probably script/overlay data -> follow-up under P4/F8"),
    ("Zone headers / game data / script VM (extra anchors)", "unk_02013980: zonedata.c FUN_020139B0 (alloc 0x4C); unk_02012224: game_event.c, game_data.c FUN_020124F4 (alloc 0x6B0); unk_020111F8: vm.c FUN_0201121C",
     "pools 0x02013A40, 0x02012300/0x02012358, 0x020126F8, 0x02011260; FUN_02013090 btl_setup.c allocs 0xF0 (battle setup); field_status.c FUN_02014428 (0x14)"),
    ("Trainer data (extra anchors)", "unk_0201FC1C: tpoke_data.c FUN_0202A128, tr_tool.c FUN_0202A440 / FUN_0202A630", "pools 0x0202A16C, 0x0202A4B0, 0x0202A7B4; LCG64 constants at 0x0202A7C0 inside FUN_0202A630"),
]


def locate_constants(ctx):
    """RNG constants in ARM9 literal pools -> (what, pool address, owner function, file key, source anchor)."""
    mimg, field, A = ctx["mimg"], ctx["field"], ctx["anchors"]
    w = mimg.words
    out = []
    for i, v in enumerate(w):
        a = mimg.base + 4 * i
        if a >= ctx["starts"][-1] or a < 0x02004FD4:
            continue
        nxt = w[i + 1] if i + 1 < len(w) else 0
        what = None
        if v == 0x6C078965:
            what = "LCG64 multiplier 0x5D588B656C078965 (pool pair)" if nxt == 0x5D588B65 else "MT19937 init multiplier 1812433253"
        elif v == 0x9908B0DF:
            what = "MT19937 matrix constant 0x9908B0DF"
        elif v == 0x9D2C5680:
            what = "MT19937 tempering 0x9D2C5680"
        elif v == 0x41C64E6D:
            what = "LCG32 multiplier 0x41C64E6D (1103515245)"
        elif v == 0x00269EC3 and w[i - 1] in (0x5D588B65, 0x6C078965):
            what = "LCG64 increment 0x269EC3"
        if what:
            f = field.owner(a)
            kind, sa, sb, _, _ = A.label(a)
            out.append((what, a, f[0] if f else 0, arm9_key(ctx, a), sa if sa == sb else "%s|%s" % (sa, sb)))
    return out


# ---------------------------------------------------------------- output
TSV_COLS = ["file", "kind", "ovl", "set", "lo", "hi", "size", "old_fns", "asm_fns", "ghidra_fns", "named_fns", "subsystem",
            "sources", "conf", "scope", "phase", "label_by", "evidence", "src_strings", "other_strings", "hw_regs", "arc_ids",
            "msg_banks", "alloc_sizes", "callees_top", "callers_top", "callee_files", "caller_files", "load_sites",
            "proc_defs", "launched_by", "prev", "next"]


def fmt_pairs(p, n=10):
    return "; ".join("%s x%d" % (a, b) for a, b in p[:n])


def tsv_row(r):
    msg = []
    for k in sorted(r["msg"]):
        t, b = [int(x) for x in k.split("/")]
        pv = bank_preview(t, b)
        msg.append("%s%s" % (k, "(%s)" % pv if pv else ""))
    vals = dict(r)
    vals.update(
        lo="%08X" % r["lo"], hi="%08X" % r["hi"], ovl="" if r["ovl"] is None else r["ovl"],
        src_strings=",".join("%s%s" % (s, "" if c == 1 else "x%d" % c) for s, c in sorted(r["src_refs"].items()))
        + ("" if not r["own_src"] else " own:" + ",".join(r["own_src"])),
        other_strings=" | ".join(r["other_strs"][:4]), hw_regs=r["hwregs"],
        arc_ids=",".join("%s%s" % (k, "" if v == 1 else "x%d" % v) for k, v in sorted(r["arc"].items(), key=lambda kv: int(kv[0]))),
        msg_banks="; ".join(msg), alloc_sizes=",".join("%d" % int(k) for k in sorted(r["alloc"], key=int)[:8]),
        callees_top=fmt_pairs(r["callees"]), callers_top=fmt_pairs(r["callers"]),
        callee_files=fmt_pairs(r["callee_files"], 6), caller_files=fmt_pairs(r["caller_files"], 6),
        load_sites=",".join(r.get("load_sites", [])), proc_defs=",".join(r.get("proc_defs", [])),
        launched_by=",".join(r.get("launched_by", [])))
    return "\t".join(str(vals.get(c, "")).replace("\t", " ").replace("\n", " ") for c in TSV_COLS)


def summarize(rows):
    """subsystem table: (subsystem, scope) -> files (primary), asm fns (apportioned), ghidra fns."""
    agg = collections.OrderedDict()

    def add(sub, scope, phase, files, asm, gh):
        k = (sub, scope)
        e = agg.setdefault(k, dict(phases=set(), files=0, asm=0, gh=0))
        e["phases"].update(p for p in phase.split("/") if p)
        e["files"] += files
        e["asm"] += asm
        e["gh"] += gh

    for r in rows:
        if r["kind"] == "ovl" or r["label_by"] == "override":
            add(r["subsystem"], r["scope"], r["phase"], 1, r["asm_fns"], r["ghidra_fns"])
        else:
            comp = r["comp"]
            w = dict((i, n) for i, (_, n) in enumerate(comp))
            ap = apportion(r["asm_fns"], w)
            for i, ((sub, scope, phase, cf), n) in enumerate(comp):
                add(sub, scope, phase, 1 if i == 0 else 0, ap[i], n)
    return agg


def scope_totals(agg, rows):
    t = collections.OrderedDict()
    for (sub, scope), e in agg.items():
        k = scope if scope in ("in", "out", "sdk", "n/a") else ("in" if scope.startswith("mixed") else scope)
        x = t.setdefault(k, dict(asm=0, gh=0))
        x["asm"] += e["asm"]
        x["gh"] += e["gh"]
    return t


def write_md(path, rows, agg, ctx, located, constants, glob_):
    A = []
    w = A.append
    arm = [r for r in rows if r["kind"] == "arm9"]
    ov = [r for r in rows if r["kind"] == "ovl"]
    w("# T1: triage of the unidentified code")
    w("")
    w("Generated by `tools/re/triage.py` (script first; hand labels live in `tools/re/T1-overrides.tsv`, 'override' in the")
    w("`label_by` column). Per-file raw data: `tools/re/T1-TRIAGE.tsv`. Regenerate: `python3 -I tools/re/triage.py` (needs the")
    w("Ghidra caches `/tmp/blk/re/{field,battle}` and `/tmp/blk/re-z12/*`, the ROM build, `black-translations`).")
    w("")
    w("Scope: **%d ARM9 files** (%d asm fns, %d Ghidra fns) and **%d overlay files** (%d asm fns, %d Ghidra fns) from the old status table"
      % (len(arm), sum(r["asm_fns"] for r in arm), sum(r["ghidra_fns"] for r in arm), len(ov),
         sum(r["asm_fns"] for r in ov), sum(r["ghidra_fns"] for r in ov)))
    w("(`ARM9 (unidentified)` 150 / 3,921 and `unidentified overlay` 131 / 2,187; asm fns = `*_func_start` count, which reproduces the")
    w("old numbers). Ghidra fns differ because `.byte` blocks hide code from the asm and the synthesized ELF seeds Thumb pointers.")
    w("")
    w("## Method and evidence (what the script uses)")
    w("")
    w("- **Debug source-file strings**: every `*.c` name in the ARM9 string table (0x020A7004..) is pointed to from a literal pool in the")
    w("  file that asserts/allocates with it (`Heap_AllocDebug(heap, size, align, \"file.c\", line)`). pool references = anchors (count in the TSV header of this run: see below).")
    w("  A function between two anchors is labelled `exact` (inside an anchor's function..pool span), `bracket` (same file on both sides),")
    w("  or `between` (different files: nearest rule, previous file unless the next anchor's function starts within 0x200 bytes, confidence low).")
    w("- **ARM9 beyond the last anchor (0x02051C54..)** has no strings: bands (`BANDS` in the script) from callers and library strings")
    w("  (gamespy/NWC* = DWC, `[SDK+NINTENDO:BACKUP`, hardware-register pool words, call graph from gflib graphics code).")
    w("- **ARC IDs** passed to the verified loaders (`ArcLoadMalloc` 0x020490F4, `ArcLoadData` 0x02048F4C, `ArcLoadMallocEx` 0x0204A1BC,")
    w("  0x02048F60, `ArcLoadDataOfs` 0x02048F74, `ArcMemberSize` 0x020490A8) and **message banks** passed to `MsgData_Create` 0x02045B38")
    w("  `(type 0, arc 2|3, bank, heap)`; bank previews come from `black-translations/0000/<bank>.txt` (ARC 002).")
    w("- **Overlays**: debug strings in the overlay, call targets into main mapped through the same anchors (pseudo-C where a set exists,")
    w("  BL decoding otherwise), proc tables (3 consecutive Thumb pointers to function starts), `overlay_load` sites, neighbours.")
    w("- Confidence: `high` = debug string inside the span, `med` = bracketed or same-subsystem between anchors / band with corroborating")
    w("  strings, `low` = nearest-anchor guess or call-graph only. `unknown` is left unknown.")
    w("")
    w("## Summary by subsystem")
    w("")
    w("asm fns of ARM9 files are apportioned to subsystems by the Ghidra function attribution (files can span several subsystems); the")
    w("`files` column counts a file under its dominant subsystem only.")
    w("")
    w("| subsystem | scope | phase | files | asm fns | Ghidra fns |")
    w("|---|---|---|---|---|---|")
    for (sub, scope), e in sorted(agg.items(), key=lambda kv: (-kv[1]["asm"], kv[0])):
        w("| %s | %s | %s | %d | %d | %d |" % (sub, scope, "/".join(sorted(e["phases"])) or "-", e["files"], e["asm"], e["gh"]))
    t = scope_totals(agg, rows)
    cf = collections.Counter()
    for r in arm:
        for (sub, scope, phase, c), n in r["comp"]:
            cf[c] += n
    w("")
    w("ARM9 Ghidra functions by attribution confidence: " + ", ".join("%s %d" % (k, cf[k]) for k in ("high", "med", "low"))
      + ". `low` = between two different source files (nearest/graph guess) or an address band without strings.")
    w("")
    w("Totals by scope: " + "; ".join("**%s** %d asm fns (%d Ghidra)" % (k, v["asm"], v["gh"]) for k, v in t.items()) + ".")
    w("")
    w("## Located items (step 2)")
    w("")
    w("Addresses are RAM addresses; `FUN_x` = Ghidra/decomp name. 'verified' = read in pseudo-C or confirmed by a constant/loader scan.")
    w("")
    w("| item | where | evidence |")
    w("|---|---|---|")
    for it, where, ev in located:
        w("| %s | %s | %s |" % (it, where.replace("|", "/"), ev.replace("|", "/")))
    w("")
    w("RNG constants in ARM9 literal pools (scan of `main.sbin`):")
    w("")
    w("| constant | pool addr | nearest preceding fn | file | source anchor |")
    w("|---|---|---|---|---|")
    for what, a, fs, key, src in constants:
        w("| %s | %08X | FUN_%08X | %s | %s |" % (what, a, fs, key, src.replace("|", " / ")))
    w("")
    w("ARC users found by the loader scan (ids with a code user in main/field/battle/other decompiled sets; key = asm file / overlay):")
    w("")
    for arc in (16, 17, 18, 19, 20, 21, 24, 25, 123):
        us = glob_["arc_users"].get(arc)
        fns = sorted(set("%08X" % a for a, s, k in glob_["arc_fns"].get(arc, [])))[:8]
        w("- ARC %03d: %s" % (arc, ("%s; fns %s" % (", ".join("%s x%d" % kv for kv in us.most_common(5)), ",".join(fns))) if us else "no direct user via the verified loaders"))
    w("")
    w(PROPOSALS)
    w("")
    w("## Segment maps of the large ARM9 files")
    w("")
    w("Source-file segments (by Ghidra function; `?` = decided by call-graph votes, `A|B` = between two anchors, nearest rule). Only files with 5+ distinct sources.")
    for r in arm:
        prim = collections.OrderedDict()
        for lo, hi, src, how, n in r["runs"]:
            key = src.split("|")[0].rstrip("?")
            e = prim.setdefault(key, [lo, hi, 0])
            e[1] = max(e[1], hi)
            e[2] += n
        if len(prim) < 5:
            continue
        w("")
        w("**%s** (%08X-%08X)" % (r["file"], r["lo"], r["hi"]))
        w("")
        w("| source (primary) | first fn | last fn | fns |")
        w("|---|---|---|---|")
        for key, (lo, hi, n) in prim.items():
            w("| %s | %08X | %08X | %d |" % (key, lo, hi, n))
    w("")
    w("## Per-file table: ARM9 (150 files, address order)")
    w("")
    w("| file | range | asm/Ghidra fns | subsystem | sources | conf | scope | phase | by |")
    w("|---|---|---|---|---|---|---|---|---|")
    for r in arm:
        w("| %s | %08X-%08X | %d/%d | %s | %s | %s | %s | %s | %s |" % (
            r["file"][:-2], r["lo"], r["hi"], r["asm_fns"], r["ghidra_fns"], r["subsystem"], r["sources"][:70], r["conf"],
            r["scope"], r["phase"], r["label_by"]))
    w("")
    w("## Per-file table: overlays (131 files)")
    w("")
    w("| file | base | size | asm/Ghidra fns | set | subsystem | conf | scope | phase | by | evidence |")
    w("|---|---|---|---|---|---|---|---|---|---|---|")
    for r in ov:
        w("| ov%03d | %08X | %d | %d/%d | %s | %s | %s | %s | %s | %s | %s |" % (
            r["ovl"], r["lo"], r["size"], r["asm_fns"], r["ghidra_fns"], r["set"], r["subsystem"], r["conf"], r["scope"], r["phase"],
            r["label_by"], r["evidence"][:150].replace("|", "/")))
    unk = [r for r in rows if r["label_by"] == "unknown"]
    w("")
    w("## Unknown (%d files, %d asm fns)" % (len(unk), sum(r["asm_fns"] for r in unk)))
    w("")
    for r in unk:
        w("- %s (%d fns): %s" % (r["file"], r["asm_fns"], r["evidence"][:140]))
    open(path, "w").write("\n".join(A) + "\n")


PROPOSALS = ""  # filled from T1-proposals.md next to the script (hand-written, kept out of the code)


def main():
    global PROPOSALS
    args = sys.argv[1:]
    out_tsv = os.path.join(HERE, "T1-TRIAGE.tsv")
    out_md = os.path.join(HERE, "T1-TRIAGE.md")
    feat_json = None
    use_json = None
    while args:
        a = args.pop(0)
        if a == "--out":
            out_tsv = args.pop(0)
        elif a == "--md":
            out_md = args.pop(0)
        elif a == "--features":
            feat_json = args.pop(0)
        elif a == "--from-features":
            use_json = args.pop(0)
    pp = os.path.join(HERE, "T1-proposals.md")
    if os.path.exists(pp):
        PROPOSALS = open(pp).read().rstrip("\n")
    ctx = build_ctx()
    if use_json:
        data = json.load(open(use_json))
        feats, glob_ = data["feats"], data["glob"]
        for g in ("arc_users", "msg_users"):
            glob_[g] = dict((int(k) if g == "arc_users" else k, collections.Counter(v)) for k, v in glob_[g].items())
        glob_["arc_fns"] = dict((int(k), [tuple(x) for x in v]) for k, v in glob_["arc_fns"].items())
    else:
        feats, glob_ = extract(ctx)
        ser = dict(arc_users=dict((str(k), dict(v)) for k, v in glob_["arc_users"].items()),
                   msg_users=dict((str(k), dict(v)) for k, v in glob_["msg_users"].items()),
                   arc_fns=dict((str(k), v) for k, v in glob_["arc_fns"].items()))
        if feat_json:
            json.dump(dict(feats=feats, glob=ser), open(feat_json, "w"))
            return
        glob_ = dict(glob_)
    rows = compose(feats, load_overrides())
    agg = summarize(rows)
    open(out_tsv, "w").write("\t".join(TSV_COLS) + "\n" + "\n".join(tsv_row(r) for r in rows) + "\n")
    write_md(out_md, rows, agg, ctx, LOCATED, locate_constants(ctx), glob_)
    print("wrote", out_tsv, out_md, "(%d rows)" % len(rows))


if __name__ == "__main__":
    main()
