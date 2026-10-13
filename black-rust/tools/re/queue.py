#!/usr/bin/env python3
"""Work queue: next leaf-first batch of unported functions (MIGRATION-AGENT.md section 3b, Z8).

usage: python3 -I tools/re/queue.py [SET] [--range START END] [-n N] [--min-size BYTES] [--json] [--stats] [--selftest]

Reads <RE_OUT>/<set>/pc/index.jsonl (RE_OUT defaults to /tmp/blk/re) and every `/// C:` doc comment
in crates/*/src to decide what is already ported. Output order: functions whose callees are all
ported / trait-stubbed / runtime helpers first, then by fewest unported callees (then size).
Also the shared library for tools/re/packet.py (which loads this file by path).

Status of an address, from the Rust sources:
  ported - a `fn` with a body carries `/// C: ...` naming it, or a free `fn fun_XXXXXXXX` exists
           (unless its doc says "Env wrapper" / "UNVERIFIED port": those count as stubs)
  stub   - a trait method declaration carries `/// C:` naming it, or any method called
           `fun_XXXXXXXX` / `sub_XXXXXXXX` (a FlowHost-style wrapper around an unported callee)
  ref    - the `/// C:` comment is on a struct/const/...: documented, but not a ported function
Pure Python 3 stdlib (written for 3.7).
"""
import collections
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))  # black-rust/
RE_OUT = os.environ.get("RE_OUT", "/tmp/blk/re")
PACKETS_OUT = os.environ.get("PACKETS_OUT", "/tmp/blk/packets")

GENERIC = re.compile(r"^(FUN|sub|LAB|thunk|caseD|PTR|DAT)_|^thunk_")
RUNTIME_PREFIX = re.compile(r"^(SVC_|MI_|OS_|FX_|MATH_|CP_|CARD_|PXI_|SND_|_s32|_u32|_ll|_f|__|_start)")
ADDR_IN_NAME = re.compile(r"(?<![0-9a-fA-F])([0-9a-fA-F]{8})$")


# ---------------------------------------------------------------- Rust side

def _addr_tokens(head, name_to_addr):
    """Addresses named in the head of a `/// C:` comment (text before the first sentence break)."""
    # cut at the first ':' , '. ', '; ' or ' - ' outside backticks
    cut = len(head)
    in_bt = False
    for i, ch in enumerate(head):
        if ch == "`":
            in_bt = not in_bt
        if in_bt:
            continue
        rest = head[i:i + 2]
        if ch == ":" or rest == ". " or rest == "; " or head[i:i + 3] == " - " or (ch == "." and i + 1 == len(head)):
            cut = i
            break
    head = head[:cut]
    out = []
    for m in re.finditer(r"`([^`]*)`|(?<![0-9A-Za-z_])(0x[0-9a-fA-F]{6,8})\b", head):
        tok = m.group(1) if m.group(1) is not None else m.group(2)
        for a in re.finditer(r"(?:FUN|sub)_[A-Za-z0-9_]*?([0-9a-fA-F]{8})(?![0-9a-fA-F])", tok):
            out.append(a.group(1).lower())
        for a in re.finditer(r"(?<![0-9A-Za-z_])0x([0-9a-fA-F]{6,8})\b", tok if m.group(1) is not None else m.group(2)):
            out.append(a.group(1).lower().zfill(8))
        ident = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*(\(.*\))?$", tok.strip())
        if ident and not GENERIC.match(ident.group(1)) and ident.group(1) in name_to_addr:
            out.append(name_to_addr[ident.group(1)])
    res = []
    for a in out:
        a = "%08x" % (int(a, 16) & ~1)  # drop the Thumb bit
        if a not in res:
            res.append(a)
    return res


def _fn_has_body(lines, i):
    """True if the `fn` starting at line i has a body (`{`), False for a declaration (`;`)."""
    depth = 0
    for j in range(i, min(i + 40, len(lines))):
        for ch in lines[j]:
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif depth <= 0 and ch == "{":
                return True
            elif depth <= 0 and ch == ";":
                return False
        if lines[j].lstrip().startswith("//"):
            continue
    return True


def scan_rust(crates_dir, name_to_addr):
    """Return {addr: {"status","item","file","line","kind"}} from all crates/*/src/**/*.rs."""
    rank = {"ported": 3, "stub": 2, "ref": 1}
    found = {}

    def put(addr, status, item, path, line, kind):
        cur = found.get(addr)
        if cur is None or rank[status] > rank[cur["status"]]:
            found[addr] = {"status": status, "item": item, "file": path, "line": line, "kind": kind}

    item_re = re.compile(
        r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:(?:const|unsafe|async|extern\s+\"C\")\s+)*"
        r"(fn|struct|enum|const|static|type|trait|impl|mod|union)\b\s*(?:<[^>]*>\s*)?([A-Za-z_][A-Za-z0-9_]*)?")
    for crate in sorted(os.listdir(crates_dir)):
        src = os.path.join(crates_dir, crate, "src")
        for dp, dn, fns in os.walk(src):
            dn.sort()  # deterministic tie-break in put() on every host
            for fn in sorted(fns):
                if not fn.endswith(".rs"):
                    continue
                full = os.path.join(dp, fn)
                rel = os.path.relpath(full, ROOT)
                with open(full, encoding="utf-8") as fh:
                    lines = fh.read().split("\n")
                scope = ""
                for i, ln in enumerate(lines):
                    ind = len(ln) - len(ln.lstrip())
                    if ind == 0:
                        sm = re.match(r"^(?:pub\s+)?(?:unsafe\s+)?(impl|trait)\b(.*?)\s*(?:\{|where|$)", ln)
                        if sm:
                            scope = (sm.group(1) + sm.group(2)).strip()
                    # name-based stubs / free functions: fn fun_XXXXXXXX, fn sub_XXXXXXXX
                    nm = re.match(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const\s+)?fn\s+(?:fun|sub)_([0-9a-fA-F]{8})\b", ln)
                    if nm:
                        a = nm.group(1).lower()
                        doc = " ".join(lines[max(0, i - 8):i])
                        wrapper = "Env wrapper" in doc or "UNVERIFIED port" in doc
                        if ind == 0 and _fn_has_body(lines, i) and not wrapper:
                            put(a, "ported", ln.strip().split("(")[0].split("fn ")[-1], rel, i + 1, "fn")
                        else:
                            put(a, "stub", _qual(scope, ln), rel, i + 1, "fn")
                    cm = re.match(r"^\s*///\s*C:\s*(.*)$", ln)
                    if not cm:
                        continue
                    addrs = _addr_tokens(cm.group(1), name_to_addr)
                    if not addrs:
                        continue
                    j = i + 1
                    while j < len(lines) and (not lines[j].strip() or lines[j].lstrip().startswith(("//", "#["))):
                        j += 1
                    if j >= len(lines):
                        continue
                    im = item_re.match(lines[j])
                    if not im:
                        continue
                    kind, iname = im.group(1), im.group(2) or ""
                    if kind == "fn":
                        status = "ported" if _fn_has_body(lines, j) else "stub"
                        item = _qual(scope, lines[j]) if lines[j].startswith(" ") else iname
                    else:
                        status, item = "ref", kind + " " + iname
                    for a in addrs:
                        put(a, status, item, rel, j + 1, kind)
    return found


def _qual(scope, ln):
    m = re.search(r"fn\s+([A-Za-z_][A-Za-z0-9_]*)", ln)
    name = m.group(1) if m else ln.strip()
    if scope:
        sc = re.sub(r"<[^>]*>", "", scope)
        sc = re.sub(r"^(trait\s+\w+)\s*:.*", r"\1", sc)
        sc = re.sub(r"^impl\s+(\S+)\s+for\s+(\S+).*", r"\2 (\1)", sc)
        sc = re.sub(r"^(impl|trait)\s+", "", sc).strip()
        return "%s::%s" % (sc, name)
    return name


# ---------------------------------------------------------------- index side

class Index(object):
    def __init__(self, setname):
        self.setname = setname
        self.dir = os.path.join(RE_OUT, setname)
        self.pc = os.path.join(self.dir, "pc")
        self.rows = {}
        order = []
        with open(os.path.join(self.pc, "index.jsonl")) as fh:
            for line in fh:
                r = json.loads(line)
                self.rows[r["addr"]] = r
                order.append(r)
        self.order = order
        # name -> addr: embedded address wins, else the lowest main-RAM (0x02......) one
        self.name_to_addr = {}
        for r in order:
            n = r["name"]
            m = ADDR_IN_NAME.search(n)
            if m and not n.startswith(("thunk_",)):
                if int(m.group(1), 16) & ~1 == int(r["addr"], 16):
                    self.name_to_addr[n] = r["addr"]
                    continue
            cur = self.name_to_addr.get(n)
            if cur is None or (r["addr"].startswith("02") and not cur.startswith("02")):
                if not (cur and ADDR_IN_NAME.search(n)):
                    self.name_to_addr[n] = r["addr"]
        self.callers = collections.defaultdict(list)
        for r in order:
            for c in r["callees"]:
                a = self.resolve(c)
                if a:
                    self.callers[a].append(r["addr"])

    def resolve(self, name):
        a = self.name_to_addr.get(name)
        if a:
            return a
        m = ADDR_IN_NAME.search(name)
        if m:
            a = "%08x" % (int(m.group(1), 16) & ~1)
            if a in self.rows:
                return a
        return None

    def callee_addrs(self, r):
        out = []
        for c in r["callees"]:
            a = self.resolve(c)
            out.append((c, a))
        return out

    def is_thunk(self, r):
        return "thunk_" in r["name"] or r["name"].startswith("LAB_") or "SDK_OVERLAY" in r["name"]

    def clusters(self):
        """(cluster_of {addr: id}, members {id: [addr...]}); same algorithm and order as clusters.py."""
        cache = os.path.join(PACKETS_OUT, self.setname, ".clusters.json")
        idx = os.path.join(self.pc, "index.jsonl")
        key = [os.path.getmtime(idx), len(self.order)]
        try:
            with open(cache) as fh:
                d = json.load(fh)
            if d["key"] == key:
                return d["of"], {int(k): v for k, v in d["members"].items()}
        except (IOError, OSError, ValueError, KeyError):
            pass
        cl = collections.OrderedDict()
        for f in sorted(os.listdir(self.pc)):
            if not f.endswith(".c"):
                continue
            with open(os.path.join(self.pc, f), errors="replace") as fh:
                s = fh.read()
            s = s.replace(f[9:-2], "SELF")
            s = re.sub(r"\b(FUN|sub|LAB|DAT|PTR|thunk)_\w+", "ID", s)
            s = re.sub(r"\b0x[0-9a-fA-F]+\b|\b\d+\b", "K", s)
            cl.setdefault(s, []).append(f[:8])
        groups = sorted(cl.values(), key=len, reverse=True)
        multi = [g for g in groups if len(g) > 1]
        of, members = {}, {}
        for n, g in enumerate(multi, 1):
            members[n] = g
            for a in g:
                of[a] = n
        try:
            os.makedirs(os.path.dirname(cache), exist_ok=True)
            with open(cache, "w") as fh:
                json.dump({"key": key, "of": of, "members": members}, fh)
        except (IOError, OSError):
            pass
        return of, members


def runtime_names(sig_path):
    """Names/addresses in the '# runtime helpers' section of signatures.tsv."""
    out = set()
    sec = False
    try:
        with open(sig_path) as fh:
            for ln in fh:
                if ln.startswith("#"):
                    sec = "runtime helpers" in ln
                    continue
                if sec and ln.strip():
                    out.add(ln.split("\t")[0].strip().lower())
    except IOError:
        pass
    return out


class Status(object):
    """Combined view: index + Rust ports."""

    def __init__(self, setname):
        self.ix = Index(setname)
        self.rust = scan_rust(os.path.join(ROOT, "crates"), self.ix.name_to_addr)
        self.rt = runtime_names(os.path.join(HERE, "signatures.tsv"))

    def state(self, addr):
        r = self.rust.get(addr)
        return r["status"] if r else None

    def callee_class(self, name, addr, lo, hi):
        """ported | stub | runtime | in (in-range unported) | ext (out-of-range unported)"""
        if addr is None:
            return "runtime"  # unresolved (indirect / data): cannot be ported separately
        st = self.state(addr)
        if st in ("ported", "stub"):
            return st
        row = self.ix.rows.get(addr)
        if row and self.ix.is_thunk(row):
            return "runtime"
        inr = lo <= int(addr, 16) < hi
        if not inr and (RUNTIME_PREFIX.match(name) or name.lower() in self.rt or addr in self.rt):
            return "runtime"
        return "in" if inr else "ext"

    def candidates(self, lo, hi):
        out = []
        for r in self.ix.order:
            a = int(r["addr"], 16)
            if not (lo <= a < hi) or r["size"] <= 0 or self.ix.is_thunk(r):
                continue
            out.append(r)
        return out

    def queue(self, lo, hi, min_size=0):
        of, _members = self.ix.clusters()
        items, ported, stubbed = [], 0, 0
        total = 0
        for r in self.candidates(lo, hi):
            total += 1
            st = self.state(r["addr"])
            if st == "ported":
                ported += 1
                continue
            if st == "stub":
                stubbed += 1
                continue
            if r["size"] < min_size:
                continue
            cls = collections.Counter()
            for name, a in self.ix.callee_addrs(r):
                cls[self.callee_class(name, a, lo, hi)] += 1
            items.append({
                "addr": r["addr"], "name": r["name"], "size": r["size"],
                "unported_in_range": cls["in"], "unported_external": cls["ext"],
                "unported": cls["in"] + cls["ext"], "callees": len(r["callees"]),
                "cluster": of.get(r["addr"]),
            })
        items.sort(key=lambda x: (x["unported"], x["unported_in_range"], x["size"], x["addr"]))
        return items, {"total": total, "ported": ported, "stubbed": stubbed, "todo": len(items)}


def parse_range(args, ix):
    if args:
        return int(args[0], 16), int(args[1], 16)
    return 0, 0xFFFFFFFF


# ---------------------------------------------------------------- CLI

def selftest():
    # comment forms (no /tmp needed)
    n2a = {"BattleMon_Get": "021d59b4", "ApplyTypeEffectiveness": "021d7a6c"}
    cases = [
        ("`FUN_overlay_d_93__021d8484`: registers", ["021d8484"]),
        ("unlabeled routine at `0x021D7B08`: signed maximum.", ["021d7b08"]),
        ("`_u32_div_f` (`FUN_0209C2B0`) quotient.", ["0209c2b0"]),
        ("`ApplyTypeEffectiveness` (0x021D7A6C): scales", ["021d7a6c"]),
        ("`FUN_02008574 @ 0x02008574` - stores", ["02008574"]),
        ("`BattleMon_Get`", ["021d59b4"]),
        ("`FUN_02017988(pp, form)`: change; see `FUN_0201BD28`", ["02017988"]),
        ("`BattleMon_ClearIllusionMon` / `FUN_overlay_d_93__021d6c2c`", ["021d6c2c"]),
        ("unnamed Thumb routine at `0x021D778D`:", ["021d778c"]),
        ("`FALSE`", []),
    ]
    for text, want in cases:
        got = _addr_tokens(text, n2a)
        assert got == want, (text, got, want)
    body_lines = ["    fn a(&mut self, out: &mut [u8; 8]) -> u32;", "x"]
    assert _fn_has_body(body_lines, 0) is False
    assert _fn_has_body(["pub fn a(x: [u8; 2]) -> u32 {"], 0) is True
    st = Status("battle")
    for addr, want in (("021d8484", "ported"), ("021d80bc", "ported"), ("021d7a6c", "ported"),
                       ("021c8234", "stub"), ("021d74f8", "ported"), ("0209c2b0", "ported")):
        assert st.state(addr) == want, (addr, st.state(addr), want)
    field = [a for a, r in st.rust.items() if r["status"] == "ported" and r["file"].endswith("field.rs")]
    assert field, "no battle_field functions detected as ported"
    items, stats = st.queue(0x021B6100, 0x021F3B20)
    assert stats["ported"] > 50 and stats["total"] > 1000, stats
    assert all(i["addr"] != "021d8484" for i in items)
    print("selftest ok (ported in overlay 93: %d/%d; field.rs ports: %d)" % (stats["ported"], stats["total"], len(field)))


def main(argv):
    a = argv[1:]
    if "--selftest" in a:
        selftest()
        return 0
    setname, rng, n, as_json, stats, min_size = "battle", None, 20, False, False, 0
    i = 0
    while i < len(a):
        x = a[i]
        if x == "--range":
            rng = (a[i + 1], a[i + 2])
            i += 3
        elif x == "-n":
            n = int(a[i + 1])
            i += 2
        elif x == "--min-size":
            min_size = int(a[i + 1])
            i += 2
        elif x == "--json":
            as_json = True
            i += 1
        elif x == "--stats":
            stats = True
            i += 1
        elif x in ("-h", "--help"):
            print(__doc__)
            return 0
        else:
            setname = x
            i += 1
    st = Status(setname)
    lo, hi = parse_range(rng, st.ix)
    items, s = st.queue(lo, hi, min_size)
    if as_json:
        print(json.dumps({"stats": s, "queue": items[:n] if not stats else []}, indent=1))
        return 0
    print("# %s %08x-%08x: %d/%d ported, %d trait-stubbed, %d todo" % (
        setname, lo, hi, s["ported"], s["total"], s["stubbed"], s["todo"]))
    if stats:
        return 0
    print("%-8s %-34s %5s %4s %4s %s" % ("addr", "name", "size", "unpI", "unpX", "cluster"))
    for it in items[:n]:
        print("%-8s %-34s %5d %4d %4d %s" % (
            it["addr"], it["name"][:34], it["size"], it["unported_in_range"], it["unported_external"],
            it["cluster"] if it["cluster"] else "-"))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
