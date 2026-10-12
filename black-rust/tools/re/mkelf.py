#!/usr/bin/env python3
"""Synthesize a loadable ARM ELF (real bytes + decomp symbols) for one overlay set.

The decomp's main.elf carries every symbol (incl. $a/$t/$d mapping symbols) but its
sections are zero-length; the bytes only exist in main.sbin / OVY_<n>.sbin. This joins
them so Ghidra, rizin and rea can analyse the code with the decomp's names.

usage: mkelf.py <build/black.us> <out.elf> <module>...   (module = main | <overlay number>)
e.g.   mkelf.py $BLACK_ROM_BUILD /tmp/blk/re/battle.elf main 93 94 95 96
Stdlib only. Refuses overlay sets whose address ranges overlap.
"""
import os
import struct
import sys

SHT_PROGBITS, SHT_SYMTAB, SHT_STRTAB = 1, 2, 3
SHN_ABS = 0xFFF1


def read_elf(path):
    d = open(path, "rb").read()
    (e_shoff,) = struct.unpack_from("<I", d, 0x20)
    (e_flags,) = struct.unpack_from("<I", d, 0x24)
    e_shentsize, e_shnum, e_shstrndx = struct.unpack_from("<HHH", d, 0x2E)
    secs = []
    for i in range(e_shnum):
        f = struct.unpack_from("<10I", d, e_shoff + i * e_shentsize)
        secs.append(dict(name=f[0], type=f[1], flags=f[2], addr=f[3], off=f[4], size=f[5], link=f[6], entsize=f[9]))
    shstr = secs[e_shstrndx]
    for s in secs:
        o = shstr["off"] + s["name"]
        s["name"] = d[o:d.index(b"\0", o)].decode()
    return d, e_flags, secs


def main():
    build, out, mods = sys.argv[1], sys.argv[2], sys.argv[3:]
    d, e_flags, secs = read_elf(os.path.join(build, "main.elf"))
    names = ["main" if m == "main" else "OVY_%d" % int(m) for m in mods]

    # first PROGBITS section of each name holds code/data; later ones (bss) are kept as ABS symbols
    loads, keep = [], {}
    for name in names:
        idx = [i for i, s in enumerate(secs) if s["name"] == name and s["type"] == SHT_PROGBITS]
        if not idx:
            sys.exit("no section %s" % name)
        blob = open(os.path.join(build, name + ".sbin"), "rb").read()
        loads.append((name, secs[idx[0]]["addr"], blob))
        keep[idx[0]] = len(loads)  # new section index (1-based after null)
        for j in idx[1:]:
            keep[j] = SHN_ABS
    loads_sorted = sorted(loads, key=lambda t: t[1])
    for a, b in zip(loads_sorted, loads_sorted[1:]):
        if a[1] + len(a[2]) > b[1]:
            sys.exit("overlap: %s and %s" % (a[0], b[0]))

    symtab = next(s for s in secs if s["type"] == SHT_SYMTAB)
    strtab = secs[symtab["link"]]
    raw = []
    for i in range(1, symtab["size"] // 16):
        st_name, value, size, info, other, shndx = struct.unpack_from("<IIIBBH", d, symtab["off"] + i * 16)
        if shndx not in keep:
            continue
        o = strtab["off"] + st_name
        nm = d[o:d.index(b"\0", o)]
        # STT_SECTION, mwld's per-object _@DummyFn symbols (they span whole files), and the
        # $b/$m marker symbols (typed FUNC; tools would split functions at every one)
        if nm == b"" or (info & 0xF) == 3 or nm.startswith(b"_@DummyFn") or nm in (b"$b", b"$m"):
            continue
        raw.append([nm, value, size, info, other, keep[shndx]])

    # labels with a $a/$t mapping symbol at the same address are function starts:
    # type them FUNC and set the Thumb bit for $t
    mode = {}
    for s in raw:
        if s[0][:2] in (b"$a", b"$t") and s[5] != SHN_ABS:
            mode[s[1]] = s[0][:2]
    for s in raw:
        if s[0][:1] != b"$" and (s[3] & 0xF) in (0, 2) and s[1] in mode:
            s[3] = (s[3] & 0xF0) | 2
            if mode[s[1]] == b"$t":
                s[1] |= 1

    # .byte-encoded code is marked $d, so tools treat it as data. Seed a $t + FUNC symbol at
    # every odd word that points into a loaded module (handler tables, callbacks); this is
    # where the event handlers live. False positives become small bogus functions.
    import bisect
    ranges = [(addr, addr + len(blob), i + 1) for i, (_, addr, blob) in enumerate(loads)]
    maps = sorted((s[1] & ~1, s[0][:2]) for s in raw if s[0][:2] in (b"$a", b"$t", b"$d") and s[5] != SHN_ABS)
    mkeys = [m[0] for m in maps]
    funcs = {s[1] & ~1 for s in raw if (s[3] & 0xF) == 2}
    seeded = 0
    for _, addr, blob in loads:
        for o in range(0, len(blob) - 3, 4):
            (w,) = struct.unpack_from("<I", blob, o)
            t = w & ~1
            if not w & 1 or t in funcs:
                continue
            sec = next((n for lo, hi, n in ranges if lo <= t < hi), None)
            if sec is None:
                continue
            k = bisect.bisect_right(mkeys, t) - 1
            if k >= 0 and maps[k][1] == b"$a":
                continue
            if k < 0 or maps[k][1] == b"$d":
                raw.append([b"$t", t, 0, 0x02, 0, sec])
            raw.append([b"sub_%08x" % t, w, 0, 0x12, 0, sec])
            funcs.add(t)
            seeded += 1
    print("seeded %d functions from Thumb pointers" % seeded)

    strt = bytearray(b"\0")
    symt = bytearray(b"\0" * 16)
    locals_ = [s for s in raw if (s[3] >> 4) == 0]
    globals_ = [s for s in raw if (s[3] >> 4) != 0]
    for s in locals_ + globals_:
        symt += struct.pack("<IIIBBH", len(strt), s[1], s[2], s[3], s[4], s[5])
        strt += s[0] + b"\0"

    shstr = bytearray(b"\0")

    def sname(n):
        o = len(shstr)
        shstr.extend(n.encode() + b"\0")
        return o

    nload = len(loads)
    ehsize, phsize = 52, 32
    off = ehsize + phsize * nload
    body = bytearray()
    shdrs = [struct.pack("<10I", *([0] * 10))]
    phdrs = []
    for name, addr, blob in loads:
        while (off + len(body)) % 4:
            body += b"\0"
        o = off + len(body)
        body += blob
        shdrs.append(struct.pack("<10I", sname(name), SHT_PROGBITS, 0x7, addr, o, len(blob), 0, 0, 4, 0))
        phdrs.append(struct.pack("<8I", 1, o, addr, addr, len(blob), len(blob), 7, 4))
    for blob, typ, nm, link, info, ent in (
        (symt, SHT_SYMTAB, ".symtab", nload + 2, len(locals_) + 1, 16),
        (strt, SHT_STRTAB, ".strtab", 0, 0, 0),
    ):
        while (off + len(body)) % 4:
            body += b"\0"
        shdrs.append(struct.pack("<10I", sname(nm), typ, 0, 0, off + len(body), len(blob), link, info, 4, ent))
        body += blob
    shstr_name = sname(".shstrtab")
    shdrs.append(struct.pack("<10I", shstr_name, SHT_STRTAB, 0, 0, off + len(body), len(shstr), 0, 0, 1, 0))
    body += shstr
    while (off + len(body)) % 4:
        body += b"\0"
    shoff = off + len(body)
    entry = loads[0][1]
    ehdr = b"\x7fELF\x01\x01\x01" + b"\0" * 9 + struct.pack(
        "<HHIIIIIHHHHHH", 2, 40, 1, entry, ehsize, shoff, e_flags, ehsize, phsize, nload, 40, len(shdrs), len(shdrs) - 1
    )
    with open(out, "wb") as f:
        f.write(ehdr + b"".join(phdrs) + body + b"".join(shdrs))
    print("%s: %d modules, %d symbols" % (out, nload, len(raw)))


if __name__ == "__main__":
    main()
