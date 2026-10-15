#!/usr/bin/env python3
r"""F12: generate `crates/black-field/src/integration/ports.rs`, the address -> Rust port table of the field
integration test.

Only verified ports (the module's `PORTED` list) are dispatched; everything else runs as original code in the
interpreter. The argument mapping of a port comes from its module's differential test (`tests.rs`: the
`match addr { 0x.. => port(e, v[0], v[2]) }` arms, which give the register / stack word of every parameter --
ports drop the arguments the original never reads, so the Rust parameter order is not always the register
order). Arms that build their arguments from test-only data (case buffers, globals) are left out; a ported
function without a usable arm falls back to its signature (env, memory window, scalars in register order).

  python3 -I tools/field-integ/gen_ports.py            # rewrite ports.rs
  python3 -I tools/field-integ/gen_ports.py --check    # exit 1 when ports.rs is stale
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "crates/black-field/src/integration/ports.rs")
CRATES = [("black-field", "crate", "env.fmem()"), ("black-script", "black_script", "env.smem()")]

# Overlay load ranges (code start, BSS start) of the field set (the overlay table); 0 = ARM9 main.
OVERLAYS = [(10, 0x02155160, 0x02172BA0), (11, 0x02172BC0, 0x02177020), (18, 0x0217AE20, 0x02183B60),
            (20, 0x02183B60, 0x02187F00), (21, 0x02187F00, 0x021DE700)]

FN = re.compile(r"((?:^///[^\n]*\n)+)(?:#\[[^\n]*\n)*pub fn (\w+)(<[^>]*>)?\(([^)]*)\)\s*(?:->\s*([^{]+?))?\s*(?:where[^{]*)?\{", re.M)
ADDR = re.compile(r"C: `FUN_([0-9A-Fa-f]{8})|unlabeled routine at `0x([0-9A-Fa-f_]{8,9})`")
PORTED = re.compile(r"pub const PORTED: &\[u32\] = &\[([^\]]*)\]", re.S)
# ports whose spec (so the tests.rs arm) gives fewer inputs than the call passes on: a tail call that forwards
# r1 unchanged; the arm as the field calls it
CALL_AS = {0x02012FBC: "gd_call_020085d4(h, v[0], v[1])"}
SCALAR = {"u32": "{}", "i32": "{} as i32", "u16": "{} as u16", "u8": "{} as u8", "bool": "{} != 0"}
RET = {"u32": "r as u64", "bool": "r as u64", "u16": "r as u64", "u8": "r as u64", "i32": "r as u32 as u64", "()": "0"}
# test-only data an arm may not use
BAD = re.compile(r"\bc\.|bytes::|\bbuf\(|\bWORK\b|stubs|testenv|\blet\b|\bglobals\b|\bbufs\b|Arg::|\bpeek\b")


def overlay_of(a):
    for n, lo, hi in OVERLAYS:
        if lo <= a < hi:
            return n
    if a < 0x02155160:
        return 0
    return None


def balanced(s, i, open_, close):
    """End index (exclusive) of the balanced group starting at s[i] == open_."""
    d = 0
    j = i
    while j < len(s):
        if s[j] == open_:
            d += 1
        elif s[j] == close:
            d -= 1
            if d == 0:
                return j + 1
        j += 1
    raise ValueError("unbalanced")


def arms(test_src):
    """`{addr: expression}` of the `match addr` arms of a tests.rs."""
    out = {}
    for m in re.finditer(r"match addr \{", test_src):
        body = test_src[m.end():balanced(test_src, m.end() - 1, "{", "}") - 1]
        i = 0
        while True:
            am = re.compile(r"(0x[0-9A-Fa-f_]+)\s*=>\s*").search(body, i)
            if not am:
                break
            j = am.end()
            if body[j] == "{":
                k = balanced(body, j, "{", "}")
                expr = body[j:k]
            else:
                # up to the ',' at depth 0
                d = 0
                k = j
                while k < len(body):
                    ch = body[k]
                    if ch in "([{":
                        d += 1
                    elif ch in ")]}":
                        d -= 1
                    elif ch == "," and d == 0:
                        break
                    k += 1
                expr = body[j:k]
            out.setdefault(int(am.group(1).replace("_", ""), 16), expr.strip())
            i = k
    return out


def unwrap(expr, name):
    """`name(X, ...)` -> `X` wherever it occurs (the test's result encoders)."""
    while True:
        m = re.search(r"\b%s\(" % name, expr)
        if not m:
            return expr
        s = m.end() - 1
        e = balanced(expr, s, "(", ")")
        inner = expr[s + 1:e - 1]
        # first argument at depth 0
        d = 0
        cut = len(inner)
        for k, ch in enumerate(inner):
            if ch in "([{":
                d += 1
            elif ch in ")]}":
                d -= 1
            elif ch == "," and d == 0:
                cut = k
                break
        expr = expr[:m.start()] + inner[:cut] + expr[e:]


def convert(expr, path, mod, fns, memexpr):
    """The arm as code over `env` / `a`, or None."""
    if BAD.search(expr):
        return None
    for w in ("ofs_work", "ofs_arg", "ofs"):
        expr = unwrap(expr, w)
    expr = re.sub(r"&mut e\.mem\b|&e\.mem\b|&mut h\.mem\(\)|&h\.mem\(\)|h\.mem\(\)|e\.mem\(\)", memexpr, expr)
    expr = re.sub(r"\be\.mem\b", memexpr, expr)
    if re.search(r"\b(e|h)\.\w", expr):
        return None
    expr = re.sub(r"\b(e|h)\b", "env", expr)
    expr = re.sub(r"\bv\[", "a[", expr)

    def qual(m):
        n = m.group(1)
        if n in fns:
            return "%s::%s::%s(" % (path, mod, n)
        return m.group(0)
    expr = re.sub(r"(?<![:.\w])([a-z_][a-z0-9_]*)\(", qual, expr)
    if "super::" in expr or "self::" in expr:
        return None
    expr = re.sub(r"(?<![:\w])([A-Z]\w*)::", r"%s::%s::\1::" % (path, mod), expr)
    # every other free identifier call must be a std / known thing
    for n in re.findall(r"(?<![:.\w])([a-z_][a-z0-9_]*)\(", expr):
        if n not in ("u32", "u64"):
            return None
    # no free variables but `env` and `a`
    ok = {"env", "a", "as", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "bool", "true", "false", "mut"}
    for m2 in re.finditer(r"(?<![:.\w])([A-Za-z_][A-Za-z0-9_]*)\b(?!\s*(\(|::|!))", expr):
        if m2.group(1) not in ok:
            return None
    if expr.startswith("{"):
        return "(%s) as u64" % expr
    return "(%s) as u64" % expr


def generate():
    table = {}
    skipped = []
    sources = []
    for crate, path, memexpr in CRATES:
        for f in sorted(glob.glob(os.path.join(ROOT, "crates", crate, "src", "*.rs"))):
            mod = os.path.basename(f)[:-3]
            if mod in ("lib", "hooks", "testenv", "host"):
                continue
            s = open(f).read()
            pm = PORTED.search(s)
            if not pm:
                continue
            ported = {int(x.replace("_", ""), 16) for x in re.findall(r"0x[0-9A-Fa-f_]+", pm.group(1))}
            sources.append("%s::%s::PORTED" % (path, mod))
            sigs = {}
            for m in FN.finditer(s):
                am = ADDR.search(m.group(1).split("\n")[0])
                if am:
                    sigs[int((am.group(1) or am.group(2)).replace("_", ""), 16)] = m
            fns = set(re.findall(r"^pub fn (\w+)", s, re.M))
            tf = os.path.join(os.path.dirname(f), mod, "tests.rs")
            tarms = arms(open(tf).read()) if os.path.exists(tf) else {}
            for addr in sorted(ported):
                if addr in table:
                    continue
                code = None
                if addr in CALL_AS:
                    tarms[addr] = CALL_AS[addr]
                if addr in tarms:
                    code = convert(tarms[addr], path, mod, fns, memexpr)
                if code is None and addr in sigs:
                    code = from_signature(sigs[addr], path, mod, memexpr)
                if code is None:
                    skipped.append((addr, "%s::%s" % (path, mod)))
                    continue
                table[addr] = (code, mod)
    return table, skipped, sources


def from_signature(m, path, mod, memexpr):
    params = [p.strip() for p in m.group(4).split(",") if p.strip()]
    ret = (m.group(5) or "()").strip()
    if ret not in RET:
        return None
    args = []
    k = 0
    for p in params:
        ty = p.split(":", 1)[1].strip()
        if re.fullmatch(r"&mut [EHM]", ty):
            args.append("env")
        elif ty in ("&mut Mem", "&Mem"):
            args.append(memexpr)
        elif ty in SCALAR:
            args.append(SCALAR[ty].format("a[%d]" % k))
            k += 1
        else:
            return None
    call = "%s::%s::%s(%s)" % (path, mod, m.group(2), ", ".join(args))
    if ret == "()":
        return "{ %s; 0u64 }" % call
    return "(%s) as u64" % call


def render(table, skipped, sources):
    o = ["//! Generated by `tools/field-integ/gen_ports.py` (do not edit): the address -> Rust port table of the F12",
         "//! integration (verified ports only; the argument words follow each port's differential-test arm).",
         "//!",
         "//! Verified ports without a usable mapping (they run as original code):"]
    for a, n in sorted(skipped):
        o.append("//! - `%08X` `%s`" % (a, n))
    o += ["", "#![allow(clippy::unnecessary_cast, clippy::identity_op, unused_braces, clippy::unit_arg)]", "",
          "use super::Env;", "",
          "/// The `PORTED` lists the table was generated from (`ports_fresh` checks it against them).",
          "pub const SOURCES: &[&[u32]] = &["] + ["    %s," % x for x in sources] + ["];", "",
          "/// Verified ports without a usable mapping (they run as original code).",
          "pub const UNMAPPED: &[u32] = &["] + ["    0x%04X_%04X," % (a >> 16, a & 0xFFFF) for a, _ in sorted(skipped)] + ["];", "",
          "/// `(address, overlay (0 = ARM9 main), module)` of every dispatched port, by address.",
          "pub const PORTS: &[(u32, u16, &str)] = &["]
    for a in sorted(table):
        o.append("    (0x%04X_%04X, %d, \"%s\")," % (a >> 16, a & 0xFFFF, overlay_of(a), table[a][1]))
    o += ["];", "",
          "/// Runs the port of `addr` with the argument words `a` (r0-r3, then the stack words); its r0 (r1 above).",
          "#[allow(unreachable_code)]",
          "pub fn call(env: &mut Env, addr: u32, a: &[u32]) -> u64 {",
          "    match addr {"]
    for a in sorted(table):
        o.append("        0x%04X_%04X => %s," % (a >> 16, a & 0xFFFF, table[a][0]))
    o += ["        _ => unreachable!(\"F12: no port at {addr:#010x}\"),", "    }", "}"]
    return "\n".join(o) + "\n"


def rustfmt(text):
    """The text as `cargo fmt` leaves it (edition 2021)."""
    import shutil
    import subprocess
    exe = shutil.which("rustfmt") or os.path.expanduser("~/.cargo/bin/rustfmt")
    return subprocess.run([exe, "--edition", "2021", "--emit", "stdout"], input=text, capture_output=True,
                          text=True, check=True).stdout


def main():
    table, skipped, sources = generate()
    for a in table:
        assert overlay_of(a) is not None, hex(a)
    text = render(table, skipped, sources)
    text = rustfmt(text)
    if "--check" in sys.argv:
        cur = open(OUT).read() if os.path.exists(OUT) else ""
        if cur != text:
            print("ports.rs is stale: run tools/field-integ/gen_ports.py")
            sys.exit(1)
        return
    open(OUT, "w").write(text)
    print("%d ports, %d without a mapping" % (len(table), len(skipped)))


if __name__ == "__main__":
    main()
