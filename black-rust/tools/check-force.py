#!/usr/bin/env python3
"""Find spec stub lines for functions that are already PORTED in another area but lack `force`.

When a function is ported (listed in some area's `pub const PORTED`), every *other* area's spec
that stubs it must mark that stub `force`, else the stub is deactivated and the ROM side runs the
real code while the Rust side calls the stub (or the other way round). Porters add `force` for the
specs they know about; specs written in parallel (another worktree) can miss it. Run this after
every merge batch (coordinator, MIGRATION-AGENT.md §5 rule 6).

  python3 tools/check-force.py          # list unforced lines, exit 1 if any
  python3 tools/check-force.py --fix    # add `force` to them (force-only edit)

A stub of a function in the spec's own area is left alone (in-area stubs are deactivated by design).
Only specs on the battle memory set are scanned (Z20, see SPECS below): the battle modules' PORTED
addresses never deactivate a stub of a field-set (or other set) area, whose overlay addresses mean
different code.
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# svflow.py deactivates a stub when the callee is in any PORTED list under these black-battle dirs
# (`ported_flow`), and only for areas on the battle memory set (Z20: PORTED is keyed per set). So the
# scanned specs are every svflow spec plus the check-fn specs (tools/rom-diff/specs/<crate>/) whose
# `set` is battle (no `set` line = battle); a check-fn spec on another set (field, ...) sees only its own
# area and FNDIFF_PORTED_EXTRA, so the battle modules' ports can't deactivate its stubs and it is skipped.
SPECS = glob.glob(os.path.join(ROOT, "tools/rom-diff/svflow_specs/*.spec"))
FN_SPECS = glob.glob(os.path.join(ROOT, "tools/rom-diff/specs/*/*.spec"))
# Unforced on purpose (the caller's Rust port calls the ported callee directly, so both sides run real code):
# (spec, callee). Each entry is documented in docs/migration/checklist/B.md (B13-spec2 / B10g) and docs/migration/OPEN.md (B13a merge note).
EXPECTED = {("client5", 0x021CD9F4), ("core2", 0x021BCB14), ("core2", 0x021BD148), ("client4", 0x021CDF1C)}
# The dirs whose PORTED lists svflow.py's `ported_flow` scans, read from svflow.py itself so the two can't drift
# (B14: the ai dir was missing here).
FLOW_DIRS = [
    "crates/black-battle/src/" + d
    for d in re.findall(
        r'os\.path\.join\(ROOT, "crates", "black-battle", "src", "(\w+)"\)',
        open(os.path.join(ROOT, "tools/rom-diff/svflow.py")).read(),
    )
]


def ported_by_area():
    """{area name: set of addresses} from every `pub const PORTED` in FLOW_DIRS/<area>.rs (as svflow.py reads them)."""
    out = {}
    for fn in [f for d in FLOW_DIRS for f in glob.glob(os.path.join(ROOT, d, "*.rs"))]:
        src = open(fn).read()
        m = re.search(r"pub const PORTED[^=]*=\s*&\[(.*?)\];", src, re.S)
        if not m:
            continue
        area = os.path.splitext(os.path.basename(fn))[0]
        if area == "mod":
            area = os.path.basename(os.path.dirname(fn))
        out.setdefault(area, set()).update(
            int(x.replace("_", ""), 16) for x in re.findall(r"0x([0-9A-Fa-f_]+)", m.group(1))
        )
    return out


def spec_set(path):
    """The `set NAME` line of a spec file (None = battle)."""
    for line in open(path):
        t = line.split("#")[0].split()
        if len(t) == 2 and t[0] == "set":
            return t[1]
    return None


def scanned():
    """(spec path, area key) of every spec whose stubs the battle modules' PORTED lists can deactivate."""
    out = [(f, os.path.splitext(os.path.basename(f))[0]) for f in sorted(SPECS)]
    # a check-fn area is never a black-battle module: every battle module's PORTED is foreign to it
    out += [(f, None) for f in sorted(FN_SPECS) if spec_set(f) in (None, "battle") or spec_set(f).startswith("battle+")]
    return out


def main():
    fix = "--fix" in sys.argv
    by_area = ported_by_area()
    bad = 0
    for f, area in scanned():
        foreign = set().union(*(v for k, v in by_area.items() if k != area))
        lines = open(f).read().split("\n")
        changed = False
        for i, l in enumerate(lines):
            m = re.match(r"(\s*stub ([0-9a-fA-F]{8}))(\s.*|)$", l)
            if (
                m
                and int(m.group(2), 16) in foreign
                and (area, int(m.group(2), 16)) not in EXPECTED
                and not re.search(r"\sforce\b", m.group(3))
            ):
                bad += 1
                print(f"{os.path.relpath(f, ROOT)}:{i + 1}: {m.group(2)} unforced")
                if fix:
                    lines[i] = m.group(1) + " force" + m.group(3)
                    changed = True
        if changed:
            open(f, "w").write("\n".join(lines))
    print(f"{bad} unforced stub line(s){' fixed' if fix and bad else ''}")
    sys.exit(1 if bad and not fix else 0)


if __name__ == "__main__":
    main()
