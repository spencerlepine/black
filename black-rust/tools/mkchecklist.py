"""Create checklists/<file>.md: one checkbox per labelled function in an asm file.

Usage: python3 tools/mkchecklist.py asm/battle_pokeparam.s
Existing ticks are preserved when re-run.
"""
import os
import re
import sys

ROM = os.environ.get("BLACK_ROM_DIR", os.path.join(os.path.dirname(__file__), "../../black-rom"))
HERE = os.path.join(os.path.dirname(__file__), "..")

rel = sys.argv[1]
name = os.path.basename(rel)[:-2]
out = os.path.join(HERE, "checklists", name + ".md")
done = {}
if os.path.exists(out):
    for line in open(out):
        m = re.match(r"- \[(.)\] `([^`]+)`(.*)", line)
        if m:
            done[m.group(2)] = (m.group(1), m.group(3))
funcs = []
for line in open(os.path.join(ROM, rel)):
    m = re.match(r"\s*(thumb|arm)_func_start (\S+)", line)
    if m:
        funcs.append(m.group(2))
with open(out, "w") as f:
    f.write(f"# {rel}\n\n`x` ported+verified, `~` ported (unverified), `-` stubbed/skipped (see note).\n\n")
    for fn in funcs:
        mark, note = done.get(fn, (" ", ""))
        f.write(f"- [{mark}] `{fn}`{note}\n")
print(out, len(funcs), "functions", sum(1 for v in done.values() if v[0] != " "), "ticked")
