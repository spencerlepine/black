"""Instruction coverage of one spec function under the generic fndiff harness (P-spec2).

Usage: python -I fncov.py AREA --spec SPEC [--ported-src FILE]... --fn HEXADDR [--size N]
                          [--label NAME=LO-HI]... [--case NAME=ADDR]... [--win ADDR]... [--reset ADDR]...
                          [--inputs N] [--missed]

Runs the spec's inputs for the function on the original code (like `svflow.py AREA --fn`), records which
instructions of [HEXADDR, HEXADDR + size) execute (Unicorn code hook; callee code is not counted, the hook
does not change any hash), counts inputs that fault or exhaust the instruction budget, and prints
  * the covered/total of the function's disassembled range (every halfword address; data and literal
    pools in the range count as "not instructions": use --size to stop before them), and
  * for each `--label NAME=LO-HI` (hex address range, e.g. one case of a switch) whether any instruction in
    the range ran and how many distinct ones (the per-case report of a jump table).
`--case NAME=ADDR` marks a case entry (a switch target): the report counts how often each case was entered
and how often a `--win ADDR` instruction (the shared "this case matched" block) ran while that case was the
most recent entry (`--reset ADDR` clears the current case, e.g. the loop head).
`--missed` lists the unexecuted halfword addresses of the range. The hit set is the same for every
`--ported-src` choice, since only the area's own function runs on the emulator side.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import svflow as S  # noqa: E402


def main(argv):
    area = argv[1]
    opt = argv[2:]

    def vals(flag):
        out, i = [], 0
        while i < len(opt):
            if opt[i] == flag:
                out.append(opt[i + 1])
                i += 2
            else:
                i += 1
        return out

    for p in vals("--spec"):
        S.SPEC_PATH[area] = p
    for p in vals("--ported-src"):
        S.PORTED_SRC.append(p)
    addr = int(vals("--fn")[0], 16)
    size = int(vals("--size")[0], 0) if vals("--size") else 0x400
    labels = []
    for lab in vals("--label"):
        name, rng = lab.split("=")
        lo, hi = rng.split("-")
        labels.append((name, int(lo, 16), int(hi, 16)))
    specs = S.load_spec(area)
    S.SET[0] = specs.set
    fs = {f.addr: f for f in specs}[addr]
    if vals("--inputs"):
        fs.inputs = int(vals("--inputs")[0])
    allported = set(S.ported())
    S.ALLPORTED.update(allported)
    rn = S.Runner(fs, S.allstubs(fs, allported))
    from unicorn import UC_HOOK_CODE

    cases = {int(c.split("=")[1], 16): c.split("=")[0] for c in vals("--case")}
    wins = {int(x, 16) for x in vals("--win")}
    resets = {int(x, 16) for x in vals("--reset")}
    entered, won, cur = {}, {}, [None]
    hit = set()

    def hook(uc, a, sz, _):
        a &= ~1
        hit.add(a)
        if a in resets:
            cur[0] = None
        if a in cases:
            cur[0] = cases[a]
            entered[cur[0]] = entered.get(cur[0], 0) + 1
        if a in wins and cur[0]:
            won[cur[0]] = won.get(cur[0], 0) + 1

    rn.e.uc.hook_add(UC_HOOK_CODE, hook, None, addr, addr + size)
    faults = 0
    for seed in range(fs.inputs):
        _, st = rn.one(seed, True)
        if any(n == "fault" for n, _ in st):
            faults += 1
    total = size // 2
    print("fn %08x: %d inputs, %d faulted; %d distinct instruction addresses executed in [%08x, %08x)"
          % (addr, fs.inputs, faults, len(hit), addr, addr + size))
    for name, lo, hi in labels:
        n = len([a for a in hit if lo <= a < hi])
        print("  %-22s %08x-%08x  %s  (%d instructions ran)" % (name, lo, hi, "HIT " if n else "MISS", n))
    for name in cases.values():
        print("  case %-14s entered %6d  matched %6d" % (name, entered.get(name, 0), won.get(name, 0)))
    if "--missed" in opt:
        miss = [a for a in range(addr, addr + size, 2) if a not in hit]
        print("  unexecuted halfword addresses:", " ".join("%08x" % a for a in miss))
    del total
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
