"""Compare per-function handler results (ROM side vs Rust side). Used by tools/check-handlers.sh.

  diffmaps.py maps ROM.json RUST.json   print "ADDR SEED" (first failing input) per mismatching
                                        function; exit 1 if any function differs or is missing
  diffmaps.py stream ROM.json RUST.json first divergence of two `--detail` word streams, with the
                                        section name, word index (byte offset), and context
"""
import json
import sys


def maps(rom, rust):
    a = json.load(open(rom))["funcs"]
    b = json.load(open(rust))["funcs"]
    bad = 0 if a and b else 1
    for addr in sorted(set(a) | set(b)):
        x, y = a.get(addr), b.get(addr)
        if x is None or y is None:
            print("%s MISSING %s" % (addr, "in ROM map" if x is None else "in Rust map"))
            bad += 1
        elif x["hash"] != y["hash"] or x["n_inputs"] != y["n_inputs"]:
            seed = next((i for i, (p, q) in enumerate(zip(x["seeds"], y["seeds"])) if p != q), 0)
            print("%s %d" % (addr, seed))
            bad += 1
    print("%d/%d functions match" % (len(a) - bad, len(a)), file=sys.stderr)
    return 1 if bad else 0


def stream(rom, rust):
    a = json.load(open(rom))
    b = json.load(open(rust))
    n = min(len(a), len(b))
    i = next((k for k in range(n) if a[k] != b[k]), None)
    if i is None:
        if len(a) == len(b):
            print("  streams identical (hash differs elsewhere?)")
            return 0
        i = n
    sec = (a[i] if i < len(a) else b[i])[0]
    first = next(k for k in range(len(a)) if a[k][0] == sec) if i < len(a) else i
    print("  first divergence: section %s word %d (byte offset 0x%x) of %d vs %d words"
          % (sec, i - first, 4 * (i - first), len(a), len(b)))
    # hostlog/envlog words are stub-call records (0x1xx = call id, then its arguments)
    for name, s in (("ROM ", a), ("Rust", b)):
        ctx = s[max(0, i - 3):i + 4]
        print("  %s: %s" % (name, " ".join("%s=0x%x%s" % (x[0], x[1], "<<" if k == min(i, 3) else "")
                                         for k, x in enumerate(ctx))))
    return 1


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "maps":
        sys.exit(maps(sys.argv[2], sys.argv[3]))
    if len(sys.argv) == 4 and sys.argv[1] == "stream":
        sys.exit(stream(sys.argv[2], sys.argv[3]))
    print(__doc__)
    sys.exit(2)
