"""Write the RECORDED table of an area's tests.rs (svflow-recorded-* or fndiff-recorded-* markers)
from a ROM-side svflow --json map.

  svrecord.py crates/black-battle/src/server_flow/AREA/tests.rs ROM.json

Called by tools/check-svflow.sh and tools/check-fn.sh only after the ROM and Rust maps matched. Refuses an empty map and
any hash that is not 16 hex digits, has zero inputs, or is the FNV offset basis (nothing hashed).
Pure Python 3 stdlib.
"""
import json
import re
import sys

EMPTY = "cbf29ce484222325"


def main(path, rom):
    funcs = json.load(open(rom))["funcs"]
    if not funcs:
        sys.exit("svrecord: empty function map, nothing to record")
    rows = []
    for addr in sorted(funcs):
        f = funcs[addr]
        h = f.get("hash", "")
        if not re.match(r"^[0-9a-f]{16}$", h) or h == EMPTY or f.get("n_inputs", 0) < 1:
            sys.exit("svrecord: refusing bad hash for %s: %r" % (addr, f))
        rows.append('    (0x%s, "%s"),' % (addr.upper(), h))
    src = open(path).read()
    tag = "fndiff" if "// fndiff-recorded-begin\n" in src else "svflow"
    a = src.index("// %s-recorded-begin\n" % tag) + len("// %s-recorded-begin\n" % tag)
    b = src.index("// %s-recorded-end" % tag)
    block = "const RECORDED: &[(u32, &str)] = &[\n" + "\n".join(rows) + "\n];\n"
    open(path, "w").write(src[:a] + block + src[b:])


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    main(sys.argv[1], sys.argv[2])
