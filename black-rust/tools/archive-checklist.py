#!/usr/bin/env python3
"""Move finished checklist items out of docs/migration/checklist/<P>.md (coordinator, checkpoint step).

    python3 -I tools/archive-checklist.py            # archive every finished item, print counts
    python3 -I tools/archive-checklist.py --dry-run  # only print what would move

A top-level item moves when its status is `[x]` or `[-]`, every indented sub-line under it is `[x]`/`[-]` too,
and none of those lines says "pending sign-off", "merge pending" or "verifying" (those still need an eye).
`[ ]`, `[~]`, `[!]` items stay. For each moved item:
  * the full lines are appended to docs/migration/checklist/archive/<P>.md (append-only, no size cap; grep it
    or `git log -S` for old hashes and notes);
  * one stub line `- [x] ID — P — <description> — <sha>` is appended to docs/migration/checklist/DONE.md, the
    index the coordinator checks when a dependency is not in the active file (`MIGRATION-AGENT.md` §5 rule 3)
    and that tools/progress.py counts as done.
Git history keeps every earlier state of the active files.
"""
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
CHECKLIST = os.path.join(ROOT, "docs", "migration", "checklist")
ARCHIVE = os.path.join(CHECKLIST, "archive")
DONE = os.path.join(CHECKLIST, "DONE.md")
PHASES = "ZTABPSFGUHRX"
TOP_RE = re.compile(r"^- \[(.)\] (\S+)(.*)$")
SUB_RE = re.compile(r"^\s+- \[(.)\] ")
KEEP = ("pending sign-off", "merge pending", "verifying")
SHA_RE = re.compile(r"\(([0-9a-f]{7,10})\b")


def items(lines):
    """Yield (start, end, mark, id) for every top-level item; `end` is exclusive and includes indented sub-lines."""
    i = 0
    while i < len(lines):
        m = TOP_RE.match(lines[i])
        if not m:
            i += 1
            continue
        j = i + 1
        while j < len(lines) and (lines[j].startswith((" ", "\t")) or lines[j] == "" and j + 1 < len(lines) and lines[j + 1].startswith((" ", "\t"))):
            j += 1
        yield i, j, m.group(1), m.group(2)
        i = j


def finished(block):
    marks = [TOP_RE.match(block[0]).group(1)] + [SUB_RE.match(l).group(1) for l in block[1:] if SUB_RE.match(l)]
    if any(mk not in "x-" for mk in marks):
        return False
    return not any(k in l for l in block for k in KEEP)


def stub(block, phase):
    m = TOP_RE.match(block[0])
    mark, iid, rest = m.group(1), m.group(2), m.group(3)
    desc = re.sub(r"\s+", " ", rest).strip(" —")
    desc = re.split(r" — deps:", desc)[0]
    desc = re.sub(r"^\(.*?\)\s*—\s*", "", desc)  # leading "(merged ...)" annotations
    sha = SHA_RE.search(" ".join(block))
    short = desc[:140] + ("…" if len(desc) > 140 else "")
    return "- [%s] %s — %s — %s%s" % (mark, iid, phase, short, " — (%s)" % sha.group(1) if sha else "")


def main(argv):
    dry = "--dry-run" in argv
    os.makedirs(ARCHIVE, exist_ok=True)
    if not os.path.exists(DONE) and not dry:
        with open(DONE, "w") as f:
            f.write("# Done items (index; one line each, appended by tools/archive-checklist.py)\n\n"
                    "Full lines: `archive/<P>.md`; long done-notes: `../log/<ID>.md`; history: git. A dependency named by an open item and absent from `<P>.md` is looked up here.\n\n")
    total = 0
    for p in PHASES:
        path = os.path.join(CHECKLIST, p + ".md")
        if not os.path.exists(path):
            continue
        lines = open(path, encoding="utf-8").read().split("\n")
        moved = []
        keep_ranges = []
        for s, e, mark, iid in list(items(lines)):
            block = lines[s:e]
            if finished(block):
                moved.append((s, e, block))
        if not moved:
            continue
        total += len(moved)
        print("%s: %d item(s): %s" % (p, len(moved), " ".join(TOP_RE.match(b[0]).group(2) for _, _, b in moved)))
        if dry:
            continue
        with open(os.path.join(ARCHIVE, p + ".md"), "a", encoding="utf-8") as fa, open(DONE, "a", encoding="utf-8") as fd:
            if os.path.getsize(os.path.join(ARCHIVE, p + ".md")) == 0:
                fa.write("# Phase %s — archived items (append-only; moved here by tools/archive-checklist.py)\n\n" % p)
            for _, _, block in moved:
                fa.write("\n".join(l for l in block if l != "") + "\n")
                fd.write(stub(block, p) + "\n")
        drop = set()
        for s, e, _ in moved:
            drop.update(range(s, e))
        out = [l for i, l in enumerate(lines) if i not in drop]
        text = re.sub(r"\n{3,}", "\n\n", "\n".join(out))
        open(path, "w", encoding="utf-8").write(text if text.endswith("\n") else text + "\n")
    print("archive-checklist: %d item(s) %s" % (total, "would move" if dry else "moved"))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
