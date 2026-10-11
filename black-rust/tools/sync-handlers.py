"""Regenerate PORTED and handler() in crates/black-battle/src/handlers/<module>.rs from the
`pub fn h_xxxxxxxx` handlers defined there. Usage: python3 tools/sync-handlers.py abilities
"""
import os
import re
import sys

for mod in sys.argv[1:]:
    p = os.path.join(os.path.dirname(__file__), "../crates/black-battle/src/handlers", mod + ".rs")
    s = open(p).read()
    addrs = sorted(set(int(a, 16) for a in re.findall(r"pub fn h_([0-9a-f]{8})<", s)))
    body = ",\n".join("    " + f"0x{a:08X}" for a in addrs)
    ported = f"pub const PORTED: &[u32] = &[\n{body},\n];"
    s = re.sub(r"pub const PORTED: &\[u32\] = &\[.*?\];", ported, s, flags=re.S)
    arms = "\n".join(f"        0x{a:08X} => h_{a:08x}::<H>," for a in addrs)
    i = s.index("pub fn handler<")
    s = s[:i] + re.sub(r"(    Some\(match addr & !1 \{\n).*?(        _ => return None,)",
                       lambda m: m.group(1) + arms + "\n" + m.group(2), s[i:], count=1, flags=re.S)
    open(p, "w").write(s)
    print(mod, len(addrs), "handlers")
