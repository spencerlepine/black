"""Per-handler instruction coverage of the differential harness (Z18).

Usage: python -I coverage.py abilities|items|moves|effects [--all] [--jobs N] [--threshold 70]
                             [--only ADDR[,ADDR...]] [--json] [--json-out FILE]

For every handler: the denominator is the set of Thumb instructions statically reachable from the
entry (a control-flow walk over `b`, conditional `b`, fall-through and `bl`/`blx` returns; calls
are not followed, literal pools and data are never reached), the numerator is how many of those
the harness's 200 inputs executed (Unicorn code hook, `handlers.run(cov=...)`; the hook does not
change any hash). Code of callees is not counted for the caller.

Default handler set = the module's PORTED list; `--all` = every handler of the registry (items,
moves, effects; effects = the overlay-95 handlers, whose code is read from `OVY_95`) so unported/UNVERIFIED handlers are included. Prints a text table of the handlers below
the threshold (default 70%) plus per-area totals; `--json` prints the full result as JSON,
`--json-out FILE` writes it while printing the table. `missed` lists the unexecuted instruction
ranges, which is where harness hints (event variables, stub salts) are needed.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import handlers as H  # noqa: E402
from emu import Emu  # noqa: E402

WINDOW = 0x1000  # a handler's code never reaches further than this from its entry


def isize(hw):
    """Thumb instruction length in bytes from its first halfword (ARMv5: only BL/BLX are 32-bit)."""
    return 4 if hw >> 11 in (0x1D, 0x1E, 0x1F) else 2


def sext(v, bits):
    return v - (1 << bits) if v >> (bits - 1) else v


def reachable(ovy, entry, stops, base=None):
    """Addresses of the instructions reachable from `entry` (no Thumb bit) inside the function.
    Branch targets outside [entry, entry + WINDOW) or onto another handler (`stops`) are tail
    calls and not followed. Returns (set of addresses, number of indirect jumps met)."""
    seen, work, indirect = set(), [entry], 0
    lo, hi = entry, entry + WINDOW

    base = H.OVY_BASE if base is None else base

    def hw(a):
        o = a - base
        return ovy[o] | ovy[o + 1] << 8

    while work:
        a = work.pop()
        while lo <= a < hi and a not in seen:
            seen.add(a)
            w = hw(a)
            n = isize(w)
            nxt = a + n
            if w >> 11 == 0x1C:  # b
                t = a + 4 + 2 * sext(w & 0x7FF, 11)
                if t not in stops:
                    work.append(t)
                break
            if w >> 12 == 0xD and (w >> 8) & 0xF < 0xE:  # b<cond>
                t = a + 4 + 2 * sext(w & 0xFF, 8)
                if t not in stops:
                    work.append(t)
            elif w & 0xFF87 == 0x4700:  # bx rN (return or indirect tail call)
                break
            elif w & 0xFF00 == 0xBD00:  # pop {..., pc}
                break
            elif w & 0xFF87 in (0x4687, 0x4487):  # mov pc, rN / add pc, rN
                indirect += 1
                break
            a = nxt
    return seen, indirect


def ranges(addrs, ovy, entry, base=None):
    """Collapse sorted instruction addresses into [start, end) hex ranges."""
    base = H.OVY_BASE if base is None else base
    out, cur = [], None
    for a in sorted(addrs):
        o = a - base
        end = a + isize(ovy[o] | ovy[o + 1] << 8)
        if cur and cur[1] == a:
            cur[1] = end
        else:
            cur = [a, end]
            out.append(cur)
    return ["%08x-%08x" % (s, e) for s, e in out]


def handler_list(mod, every):
    if mod == "abilities" or not every:
        return H.ported(mod)
    if mod == "effects":
        return sorted(H.fx_registry(H.load_ovy95())[0])
    reg = H.item_registry if mod == "items" else H.move_registry
    return sorted(reg(Emu().ovy)[1])


def _work(arg):
    mod, addrs, stops = arg
    cov = {}
    H.run(addrs, items=mod == "items", moves=mod == "moves", cov=cov, effects=mod == "effects")
    ovy, base = (H.load_ovy95(), H.OVY95_BASE) if mod == "effects" else (Emu().ovy, None)
    res = {}
    for a in addrs:
        reach, indirect = reachable(ovy, a, stops, base)
        hit = cov[a] & reach
        res["%08x" % a] = {
            "total": len(reach), "executed": len(hit),
            "pct": round(100.0 * len(hit) / len(reach), 1) if reach else 100.0,
            "indirect_jumps": indirect,
            "missed": ranges(reach - hit, ovy, a, base),
        }
    return res


def measure(mod, addrs, jobs):
    stops = set(addrs) | set(handler_list(mod, True)) if mod != "abilities" else set(addrs)
    n = max(1, min(len(addrs), jobs * 4))
    parts = [(mod, addrs[i::n], stops) for i in range(n)]
    if jobs > 1:
        from multiprocessing import Pool
        with Pool(jobs) as p:
            results = p.map(_work, parts)
    else:
        results = [_work(x) for x in parts]
    funcs = {}
    for r in results:
        funcs.update(r)
    return {"%08x" % a: funcs["%08x" % a] for a in addrs}


# Overlay-95 helpers reached only by a tail call from handlers: measured from the union of the
# callers' executed instructions. helper -> its calling handlers.
EFFECT_HELPERS = {0x06898E54: (0x06898E28, 0x06898E44)}


def helper_coverage():
    cov = {}
    callers = sorted({c for cs in EFFECT_HELPERS.values() for c in cs})
    H.run(callers, cov=cov, effects=True)
    ovy = H.load_ovy95()
    res = {}
    for h, cs in EFFECT_HELPERS.items():
        reach, indirect = reachable(ovy, h, set(callers), H.OVY95_BASE)
        hit = set().union(*(cov[c] for c in cs)) & reach
        res["%08x" % h] = {
            "total": len(reach), "executed": len(hit),
            "pct": round(100.0 * len(hit) / len(reach), 1) if reach else 100.0,
            "indirect_jumps": indirect, "missed": ranges(reach - hit, ovy, h, H.OVY95_BASE)}
    return res


def summary(funcs, threshold):
    tot = sum(f["total"] for f in funcs.values())
    hit = sum(f["executed"] for f in funcs.values())
    low = sorted((a for a, f in funcs.items() if f["pct"] < threshold), key=lambda a: funcs[a]["pct"])
    return {"handlers": len(funcs), "instructions": tot, "executed": hit,
            "pct": round(100.0 * hit / tot, 1) if tot else 100.0,
            "below_threshold": low}


def table(mod, funcs, s, threshold):
    lines = ["%s: %d handlers, %d/%d instructions executed (%.1f%%), %d below %g%%"
             % (mod, s["handlers"], s["executed"], s["instructions"], s["pct"],
                len(s["below_threshold"]), threshold),
             "handler   exec/total   pct   missed (first ranges)"]
    for a in s["below_threshold"]:
        f = funcs[a]
        m = " ".join(f["missed"][:4]) + (" ..." if len(f["missed"]) > 4 else "")
        lines.append("%s  %4d/%-4d  %5.1f  %s" % (a, f["executed"], f["total"], f["pct"], m))
    return "\n".join(lines)


def main(argv):
    args = argv[1:]

    def opt(name, default=None):
        if name in args:
            return args[args.index(name) + 1]
        return default

    jobs = int(opt("--jobs", os.cpu_count() or 1))
    threshold = float(opt("--threshold", 70))
    json_out = opt("--json-out")
    only = opt("--only")
    mods = [a for a in args if a in ("abilities", "items", "moves", "effects")] or ["abilities"]
    out = {}
    for mod in mods:
        addrs = [int(x, 16) & ~1 for x in only.split(",")] if only else handler_list(mod, "--all" in args)
        funcs = measure(mod, addrs, jobs)
        if mod == "effects" and not only and "--all" in args:
            funcs.update(helper_coverage())
        s = summary(funcs, threshold)
        out[mod] = {"summary": s, "funcs": funcs}
        if "--json" not in args:
            print(table(mod, funcs, s, threshold))
    if "--json" in args:
        print(json.dumps(out))
    if json_out:
        with open(json_out, "w") as f:
            json.dump(out, f)


if __name__ == "__main__":
    main(sys.argv)
