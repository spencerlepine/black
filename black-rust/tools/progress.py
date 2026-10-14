#!/usr/bin/env python3
"""Compute the migration's completion percentage instead of estimating it.

    python3 -I tools/progress.py            # table on stdout; rewrites docs/migration/PROGRESS.json and the table block
                                            # between <!-- progress-table:start/end --> in docs/migration/STATE.md
    python3 -I tools/progress.py --percent  # the single integer for the commit subject ("(NN% complete)")
    python3 -I tools/progress.py --check    # exit 1 if anything cannot be computed (gate step); writes nothing

Run from black-rust/ (any cwd works: paths are resolved from this file). Stdlib only.

Logic bucket = ported / in-scope:
  * ported   = union of (memory set, address) over every `pub const PORTED: &[u32]` in crates/*/src (the same arrays
               tools/rom-diff/svflow.py and tools/check-force.py read). black-battle files are battle-set code; any
               other crate's src/AREA.rs takes the `set` line of tools/rom-diff/specs/<crate>/AREA.spec (default
               battle); ARM9 main addresses (0x02004000 + main.sbin size, or 0x0209E000 without the ROM build) are
               the same code in every set and are keyed "main" (Z20).
  * in-scope = sum over docs/migration/SCOPE.tsv rows with scope in {logic, ui} of max(labelled, ghidra_fns) × share,
               where labelled = `(thumb|arm)_func_start` lines of black-rom/asm/<file> (recounted here; a row whose
               file does not exist uses its `fns` column) and ghidra_fns is the recorded Ghidra function count of the
               file's address range (battle + field sets; `.byte`-encoded handlers have no labels but Ghidra finds
               them). ui rows contribute only their logic share (e.g. battle_view_core's helpers).
  The numerator counts Ghidra-level functions and is not mapped to files; SCOPE.tsv is the place to re-baseline.

Phase buckets A, G, U, H, R = ticked items / items in docs/migration/checklist/<P>.md (top-level `- [.]` lines only;
`[x]` = 1, `[~]` = 0.5, `[!]`/`[-]` excluded) plus the `[x]` stubs of that phase in checklist/DONE.md (items archived
by tools/archive-checklist.py count as done). A `[x]` Phase U item counts as 1 only if every ui-ref/manifest.json
case for that screen has a passing report under ui-ref/report/ (else 0.5 = pending sign-off).

Overall = 0.56·logic + 0.075·A + 0.05·G + 0.19·U + 0.125·(H+R)/2   (weights from the 2026-10-11 effort estimate,
docs/migration/NOTES.md "Effort estimate"; H and R share the integration bucket).
"""
import glob
import json
import os
import re
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.abspath(os.path.join(HERE, ".."))
ROOT = os.path.abspath(os.path.join(RUST, ".."))
ROM = os.environ.get("BLACK_ROM_DIR", os.path.join(ROOT, "black-rom"))
ROM_BUILD = os.environ.get("BLACK_ROM_BUILD", os.path.join(ROM, "build", "black.us"))
SCOPE = os.path.join(ROOT, "docs", "migration", "SCOPE.tsv")
CHECKLIST = os.path.join(ROOT, "docs", "migration", "checklist")
OUT = os.path.join(ROOT, "docs", "migration", "PROGRESS.json")
MANIFEST = os.path.join(RUST, "ui-ref", "manifest.json")
REPORTS = os.path.join(RUST, "ui-ref", "report")

WEIGHTS = {"logic": 0.56, "A": 0.075, "G": 0.05, "U": 0.19, "HR": 0.125}

PORTED_RE = re.compile(r"pub const PORTED: &\[u32\] = &\[(.*?)\];", re.S)
HEX_RE = re.compile(r"0x([0-9A-Fa-f_]+)")
FUNC_RE = re.compile(r"\s*(thumb|arm)_func_start")


def main_range():
    try:
        return 0x02004000, 0x02004000 + os.path.getsize(os.path.join(ROM_BUILD, "main.sbin"))
    except OSError:
        return 0x02004000, 0x0209E000  # ARM9 main is ~0x9A000 bytes; close enough for keying when the build is absent


def spec_set(crate, area):
    spec = os.path.join(HERE, "rom-diff", "specs", crate, area + ".spec")
    if not os.path.exists(spec):
        return None
    for line in open(spec):
        m = re.match(r"\s*set\s+(\S+)", line)
        if m:
            return m.group(1)
    return "battle"


def ported():
    """{(set, addr)}, plus per-crate unique counts and the per-file list."""
    lo, hi = main_range()
    keyed = set()
    by_crate = {}
    files = {}
    for crate in sorted(os.listdir(os.path.join(RUST, "crates"))):
        src = os.path.join(RUST, "crates", crate, "src")
        if not os.path.isdir(src):
            continue
        crate_addrs = set()
        for dp, dns, fns in os.walk(src):
            dns.sort()
            for fn in sorted(fns):
                if not fn.endswith(".rs"):
                    continue
                path = os.path.join(dp, fn)
                text = open(path, encoding="utf-8").read()
                addrs = []
                for m in PORTED_RE.finditer(text):
                    body = re.sub(r"//[^\n]*", "", m.group(1))
                    addrs += [int(h.replace("_", ""), 16) for h in HEX_RE.findall(body)]
                if not addrs:
                    continue
                if crate == "black-battle":
                    s = "battle"
                else:
                    s = spec_set(crate, fn[:-3]) or "battle"
                if s.startswith("battle+"):
                    s = "battle"
                for a in addrs:
                    key = ("main", a) if lo <= a < hi else (s, a)
                    keyed.add(key)
                    crate_addrs.add(key)
                files[os.path.relpath(path, RUST)] = {"set": s, "fns": len(set(addrs))}
        if crate_addrs:
            by_crate[crate] = len(crate_addrs)
    return keyed, by_crate, files


def count_labels(path):
    n = 0
    with open(path, errors="replace") as f:
        for line in f:
            if FUNC_RE.match(line):
                n += 1
    return n


def in_scope():
    """(weighted in-scope function count, rows) from SCOPE.tsv."""
    rows = []
    total = 0.0
    raw = {"logic": 0, "ui": 0, "sdk": 0, "out": 0}
    missing = []
    with open(SCOPE) as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            cells = line.rstrip("\n").split("\t")
            if cells[0] == "file":
                continue
            file, scope, share, fns = cells[0], cells[1], float(cells[2]), int(cells[3])
            ghidra = int(cells[4]) if len(cells) > 4 and cells[4].isdigit() else 0
            path = os.path.join(ROM, "asm", file)
            if os.path.exists(path):
                fns = count_labels(path)
            elif not file.startswith("("):
                missing.append(file)
            raw[scope] = raw.get(scope, 0) + fns
            w = max(fns, ghidra) * share if scope in ("logic", "ui") else 0.0
            total += w
            if w:
                rows.append({"file": file, "scope": scope, "share": share, "fns": fns, "ghidra_fns": ghidra, "logic_fns": round(w, 1)})
    return total, raw, rows, missing


ITEM_RE = re.compile(r"^- \[(.)\] (\S+)")


def ui_pass_by_screen():
    """{screen: (cases, passing)} from ui-ref/manifest.json + ui-ref/report/<case>.json."""
    out = {}
    if not os.path.exists(MANIFEST):
        return out
    try:
        cases = json.load(open(MANIFEST)).get("cases", [])
    except (OSError, ValueError):
        return out
    for c in cases:
        screen = c.get("screen", "")
        rep = os.path.join(REPORTS, c["id"] + ".json")
        ok = False
        if os.path.exists(rep):
            try:
                ok = bool(json.load(open(rep)).get("pass"))
            except (OSError, ValueError):
                ok = False
        n, p = out.get(screen, (0, 0))
        out[screen] = (n + 1, p + (1 if ok else 0))
    return out


def archived(letter):
    """Finished items moved out of <P>.md by tools/archive-checklist.py: stubs `- [x] ID — P — ...` in DONE.md."""
    path = os.path.join(CHECKLIST, "DONE.md")
    if not os.path.exists(path):
        return []
    out = []
    for line in open(path, encoding="utf-8"):
        m = re.match(r"^- \[(.)\] (\S+) — (\w) — ", line)
        if m and m.group(3) == letter:
            out.append((m.group(1), m.group(2)))
    return out


def phase(letter, ui=None):
    path = os.path.join(CHECKLIST, letter + ".md")
    done = half = total = 0
    items = []
    if not os.path.exists(path):
        return {"error": "missing " + path}
    for mark, iid in archived(letter):
        if mark == "x":
            total += 1
            done += 1
            items.append({"id": iid, "mark": "x", "score": 1.0, "archived": True})
    for line in open(path, encoding="utf-8"):
        m = ITEM_RE.match(line)
        if not m:
            continue
        mark, iid = m.group(1), m.group(2)
        if mark in ("!", "-"):
            continue
        total += 1
        score = {"x": 1.0, "~": 0.5}.get(mark, 0.0)
        note = ""
        if letter == "U" and mark == "x" and ui is not None:
            n, p = ui.get(iid, (0, 0))
            if n == 0 or p < n:
                score = 0.5
                note = "pending sign-off (%d/%d ui cases pass)" % (p, n)
        done += score == 1.0
        half += score == 0.5
        items.append({"id": iid, "mark": mark, "score": score, "note": note} if note else {"id": iid, "mark": mark, "score": score})
    return {"done": done, "half": half, "total": total, "score": (done + 0.5 * half) / total if total else 0.0, "items": items}


def ui_scores():
    """Scores recorded by tools/ui-diff/compare.py, for PROGRESS.json (the validator's V6 and the gate compare against them)."""
    out = {}
    for rep in sorted(glob.glob(os.path.join(REPORTS, "*.json"))):
        try:
            d = json.load(open(rep))
        except (OSError, ValueError):
            continue
        out[os.path.basename(rep)[:-5]] = {"score": d.get("score"), "pass": d.get("pass"), "min_screen_score": d.get("min_screen_score")}
    return out


def compute():
    keyed, by_crate, files = ported()
    denom, raw, rows, missing = in_scope()
    logic_ratio = min(1.0, len(keyed) / denom) if denom else 0.0
    ui = ui_pass_by_screen()
    phases = {p: phase(p, ui if p == "U" else None) for p in "AGUHR"}
    hr = (phases["H"]["score"] + phases["R"]["score"]) / 2
    overall = (WEIGHTS["logic"] * logic_ratio + WEIGHTS["A"] * phases["A"]["score"] + WEIGHTS["G"] * phases["G"]["score"]
               + WEIGHTS["U"] * phases["U"]["score"] + WEIGHTS["HR"] * hr)
    try:
        commit = subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"], capture_output=True, text=True, check=False).stdout.strip()
    except OSError:
        commit = ""
    return {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "commit": commit,
        "weights": WEIGHTS,
        "overall": round(overall, 4),
        "overall_percent": int(overall * 100 + 0.5),
        "logic": {
            "ported": len(keyed),
            "in_scope": round(denom, 1),
            "ratio": round(logic_ratio, 4),
            "ported_by_crate": by_crate,
            "labelled_fns_by_scope": raw,
            "scope_rows_missing_file": missing,
            "in_scope_files": rows,
            "ported_files": files,
        },
        "phases": {p: {k: v for k, v in phases[p].items() if k != "items"} for p in phases},
        "phase_items": {p: phases[p].get("items", []) for p in phases},
        "ui_cases": ui_scores(),
    }


def table(d):
    lines = ["| bucket | done | of | ratio | weight | contributes |", "| --- | ---: | ---: | ---: | ---: | ---: |"]
    lg = d["logic"]
    lines.append("| logic (ported fns / in-scope fns per SCOPE.tsv) | %d | %s | %.1f%% | %.3f | %.1f%% |" % (lg["ported"], lg["in_scope"], 100 * lg["ratio"], WEIGHTS["logic"], 100 * WEIGHTS["logic"] * lg["ratio"]))
    for p, w in (("A", WEIGHTS["A"]), ("G", WEIGHTS["G"]), ("U", WEIGHTS["U"])):
        ph = d["phases"][p]
        lines.append("| phase %s (items) | %g | %d | %.1f%% | %.3f | %.1f%% |" % (p, ph["done"] + 0.5 * ph["half"], ph["total"], 100 * ph["score"], w, 100 * w * ph["score"]))
    h, r = d["phases"]["H"], d["phases"]["R"]
    hr = (h["score"] + r["score"]) / 2
    lines.append("| phases H+R (items) | %g | %d | %.1f%% | %.3f | %.1f%% |" % (h["done"] + r["done"] + 0.5 * (h["half"] + r["half"]), h["total"] + r["total"], 100 * hr, WEIGHTS["HR"], 100 * WEIGHTS["HR"] * hr))
    lines.append("| **overall** | | | | | **%d%%** |" % d["overall_percent"])
    lines.append("")
    lines.append("Ported by crate: " + ", ".join("%s %d" % kv for kv in sorted(lg["ported_by_crate"].items(), key=lambda kv: -kv[1])) + ".")
    if d["ui_cases"]:
        lines.append("UI cases: " + ", ".join("%s %s" % (k, "%.2f" % v["score"] if v["score"] is not None else "n/a") for k, v in sorted(d["ui_cases"].items())) + ".")
    return "\n".join(lines)


def main(argv):
    check = "--check" in argv
    d = compute()
    problems = []
    if d["logic"]["in_scope"] <= 0:
        problems.append("SCOPE.tsv yields no in-scope functions (is black-rom/asm present? BLACK_ROM_DIR=%s)" % ROM)
    if d["logic"]["ported"] <= 0:
        problems.append("no PORTED arrays found under crates/")
    for p, ph in d["phases"].items():
        if "error" in ph:
            problems.append(ph["error"])
    if problems:
        print("\n".join("progress: " + p for p in problems), file=sys.stderr)
        return 1
    if check:
        print("progress: ok (%d%%)" % d["overall_percent"])
        return 0
    if "--percent" in argv:
        print(d["overall_percent"])
        return 0
    with open(OUT, "w") as f:
        json.dump(d, f, indent=1, sort_keys=False)
        f.write("\n")
    tbl = table(d)
    print(tbl)
    print("\nwrote", os.path.relpath(OUT, ROOT))
    if update_state(tbl, d):
        print("updated", os.path.relpath(STATE, ROOT), "(progress table)")
    return 0


STATE = os.path.join(ROOT, "docs", "migration", "STATE.md")
MARK_A, MARK_B = "<!-- progress-table:start -->", "<!-- progress-table:end -->"


def update_state(tbl, d):
    """Replace the block between the progress-table markers in STATE.md (if present) with the fresh table."""
    try:
        text = open(STATE, encoding="utf-8").read()
    except OSError:
        return False
    a, b = text.find(MARK_A), text.find(MARK_B)
    if a == -1 or b == -1 or b < a:
        return False
    block = "%s\n%s\n\n_%s, commit %s, `python3 -I tools/progress.py`._\n%s" % (MARK_A, tbl, d["generated"], d["commit"] or "?", MARK_B)
    new = text[:a] + block + text[b + len(MARK_B):]
    if new != text:
        open(STATE, "w", encoding="utf-8").write(new)
    return True


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
