#!/usr/bin/env python3
r"""F12: generate `crates/black-field/src/integration/script_hooks.rs`: the field integration's `Env` implements
every hook trait of black-script (`CoreHooks`, `TalkHooks`, ..., `ScriptSysHooks`) and the named-callee traits
of `black_field::hooks` (`BtlSetupHooks`, `GameDataHooks`, `ZoneHooks`). Each required method named
`fun_xxxxxxxx` calls address 0xXXXXXXXX through the integration dispatcher (`Env::call_args`: a port, a host
model or the original code in the interpreter), with buffer parameters passed as stack scratch (in: copied
there, out: copied back). The few methods with other names are listed in `SPECIAL` and forward to hand-written
`Env` methods in `script.rs`.

  python3 -I tools/field-integ/gen_script_hooks.py            # rewrite script_hooks.rs
  python3 -I tools/field-integ/gen_script_hooks.py --check    # exit 1 when it is stale
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SOURCES = [
    (os.path.join(ROOT, "crates/black-script/src"), "black_script"),
    (os.path.join(ROOT, "crates/black-field/src/hooks.rs"), "crate"),
]
OUT = os.path.join(ROOT, "crates/black-field/src/integration/script_hooks.rs")
# traits implemented by hand (script.rs / mod.rs)
MANUAL = {"MemEnv", "HeapHooks", "CommandDispatch", "VmHooks", "ScriptHost", "FieldEnv"}
SPECIAL = {
    "call_end_callback", "call_init_proc", "fun_02045924_str", "call_wordset_fn", "fun_02045994_at",
    "call_proc_callback", "fun_020124ac_at",
}


def traits():
    out = []
    for src, path in SOURCES:
        files = sorted(glob.glob(os.path.join(src, "*.rs"))) if os.path.isdir(src) else [src]
        for f in files:
            mod = os.path.basename(f)[:-3]
            s = open(f).read()
            for m in re.finditer(r"pub trait (\w+)[^{]*\{", s):
                i = m.end()
                d = 1
                while d:
                    d += {"{": 1, "}": -1}.get(s[i], 0)
                    i += 1
                body = s[m.end():i - 1]
                meths = []
                for mm in re.finditer(r"fn (\w+)\s*(<[^>]*>)?\(([^)]*)\)([^;{]*);", body):
                    meths.append((mm.group(1), mm.group(3), re.sub(r"where.*", "", mm.group(4), flags=re.S).strip()))
                out.append((path, mod, m.group(1), meths))
    return out


def method(name, params, ret):
    ps = [p.strip() for p in params.split(",") if p.strip()]
    assert ps[0] in ("&mut self",), (name, ps)
    rty = ret.replace("->", "").strip()
    lines = ["    fn %s(%s)%s {" % (name, ", ".join(ps), (" -> " + rty) if rty else "")]
    if name in SPECIAL:
        names = [p.split(":")[0].strip() for p in ps[1:]]
        lines.append("        self.%s_impl(%s)" % (name, ", ".join(names)))
        lines.append("    }")
        return lines
    am = re.fullmatch(r"fun_([0-9a-f]{8})", name)
    assert am, name
    addr = int(am.group(1), 16)
    lines.append("        let mut ab = Args::new();")
    post = []
    for p in ps[1:]:
        n, ty = [x.strip() for x in p.split(":", 1)]
        if ty == "u32":
            lines.append("        ab.v(%s);" % n)
        elif ty in ("u16", "u8", "bool"):
            lines.append("        ab.v(%s as u32);" % n)
        elif re.fullmatch(r"&\[u8(; [0-9a-fA-Fx]+)?\]", ty):
            lines.append("        ab.inb(&%s[..]);" % n)
        elif re.fullmatch(r"&\[u32; [0-9a-fA-Fx]+\]", ty):
            lines.append("        ab.inw(&%s[..]);" % n)
        elif re.fullmatch(r"&mut \[u8; [0-9a-fA-Fx]+\]", ty):
            lines.append("        let k_%s = ab.outb(%s.len());" % (n, n))
            post.append("        %s.copy_from_slice(&ab.out_bytes(k_%s));" % (n, n))
        elif re.fullmatch(r"&mut \[u32; [0-9a-fA-Fx]+\]", ty):
            lines.append("        let k_%s = ab.outb(4 * %s.len());" % (n, n))
            post.append("        ab.out_words(k_%s, &mut %s[..]);" % (n, n))
        elif ty == "&mut u32":
            lines.append("        let k_%s = ab.inoutb(&%s.to_le_bytes());" % (n, n))
            post.append("        *%s = u32::from_le_bytes(ab.out_bytes(k_%s)[..4].try_into().unwrap());" % (n, n))
        elif ty == "&u32":
            lines.append("        ab.inb(&%s.to_le_bytes());" % n)
        elif ty == "&mut u8":
            lines.append("        let k_%s = ab.inoutb(&[*%s]);" % (n, n))
            post.append("        *%s = ab.out_bytes(k_%s)[0];" % (n, n))
        elif ty == "Option<&[u8; 4]>":
            lines.append("        match %s {" % n)
            lines.append("            Some(b) => ab.inb(&b[..]),")
            lines.append("            None => ab.v(0),")
            lines.append("        }")
        elif ty == "Buf":
            lines.append("        match %s {" % n)
            lines.append("            Buf::Addr(x) => ab.v(x),")
            lines.append("            Buf::Local(s) => ab.inb(&s.iter().flat_map(|h| h.to_le_bytes()).collect::<Vec<u8>>()),")
            lines.append("        }")
        else:
            raise SystemExit("unhandled parameter type %r in %s" % (ty, name))
    call = "self.call_args(0x%04X_%04X, &mut ab)" % (addr >> 16, addr & 0xFFFF)
    if rty == "":
        lines.append("        %s;" % call)
        lines.extend(post)
    else:
        lines.append("        let r = %s;" % call)
        lines.extend(post)
        if rty == "u32":
            lines.append("        r as u32")
        elif rty == "u64":
            lines.append("        r")
        elif rty == "bool":
            lines.append("        r as u32 != 0")
        else:
            raise SystemExit("unhandled return type %r in %s" % (rty, name))
    lines.append("    }")
    return lines


def render():
    o = ["//! Generated by `tools/field-integ/gen_script_hooks.py` (do not edit): the black-script hook traits and the",
         "//! named-callee traits of `black_field::hooks` on the F12 integration `Env`. Every `fun_xxxxxxxx` goes to",
         "//! `Env::call_args(0xXXXXXXXX, ..)` (port, host model or the original code in the interpreter); buffers are",
         "//! stack scratch (`Args`).",
         "",
         "#![allow(clippy::too_many_arguments, unused_mut, clippy::needless_borrow, clippy::redundant_slicing, clippy::let_and_return)]",
         "",
         "use super::args::Args;",
         "use super::Env;",
         "use black_script::scrcmd_bsubway::Buf;",
         ""]
    for path, mod, tr, meths in traits():
        if tr in MANUAL:
            continue
        o.append("impl %s::%s::%s for Env {" % (path, mod, tr))
        for name, params, ret in meths:
            o.extend(method(name, params, ret))
        o.append("}")
        o.append("")
    return "\n".join(o)


def rustfmt(text):
    """The text as `cargo fmt` leaves it (edition 2021)."""
    import shutil
    import subprocess
    exe = shutil.which("rustfmt") or os.path.expanduser("~/.cargo/bin/rustfmt")
    return subprocess.run([exe, "--edition", "2021", "--emit", "stdout"], input=text, capture_output=True,
                          text=True, check=True).stdout


def main():
    text = render()
    text = rustfmt(text)
    if "--check" in sys.argv:
        cur = open(OUT).read() if os.path.exists(OUT) else ""
        if cur != text:
            print("script_hooks.rs is stale: run tools/field-integ/gen_script_hooks.py")
            sys.exit(1)
        return
    open(OUT, "w").write(text)
    print("written", OUT)


if __name__ == "__main__":
    main()
