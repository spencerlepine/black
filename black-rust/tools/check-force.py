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
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# svflow.py deactivates a stub when the callee is in any PORTED list under these black-battle dirs
# (`ported_flow`); check-fn specs (tools/rom-diff/specs/<crate>/) only see their own area, so they
# can't hit this trap and are not scanned.
SPECS = glob.glob(os.path.join(ROOT, "tools/rom-diff/svflow_specs/*.spec"))
# Unforced on purpose (the caller's Rust port calls the ported callee directly, so both sides run real code):
# (spec, callee). Each entry is documented in MIGRATION-AGENT.md (§6 B13-spec2 / B10g / B13a merge note).
EXPECTED = {("client5", 0x021CD9F4), ("core2", 0x021BCB14), ("core2", 0x021BD148), ("client4", 0x021CDF1C)}
FLOW_DIRS = ["crates/black-battle/src/server_flow", "crates/black-battle/src/client", "crates/black-battle/src/server"]


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


def main():
    fix = "--fix" in sys.argv
    by_area = ported_by_area()
    bad = 0
    for f in sorted(SPECS):
        area = os.path.splitext(os.path.basename(f))[0]
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
