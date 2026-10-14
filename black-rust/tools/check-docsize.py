#!/usr/bin/env python3
"""Gate check: the plan files stay small enough to be read whole, and section references don't dangle.

The plan was split on 2026-10-13 (MIGRATION-AGENT.md had grown to 262 KB and lost data to edit-anchor
failures twice). This keeps it from regrowing:

  MIGRATION-AGENT.md                       <= 25 KB
  docs/migration/STATE.md, HUMAN.md        <=  8 KB
  every other docs/migration/**/*.md       <= 40 KB, except LOG.md, log/*.md, checklist/DONE.md and
                                           checklist/archive/*.md (append-only, uncapped)

Section references: `§N` is only meaningful inside MIGRATION-AGENT.md (sections 0, 1, 4, 5, 5b). In any
other plan file a bare `§N` must be a markdown link (`[§5](../MIGRATION-AGENT.md)`) or sit on a line that names
`MIGRATION-AGENT.md` ("MIGRATION-AGENT.md §5"); the old sections
2/3/3b/6/7/8/9/10 no longer exist and must be named by file. A dangling reference is reported as an error.

Run: python3 -I tools/check-docsize.py   (exit 1 and a list on stderr when something is over or dangling)
"""
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
DOCS = os.path.join(ROOT, "docs", "migration")
KB = 1000
ENTRY_SECTIONS = {"0", "1", "4", "5", "5b"}


def cap_for(rel):
    if rel == "MIGRATION-AGENT.md":
        return 25 * KB
    if rel in ("docs/migration/STATE.md", "docs/migration/HUMAN.md"):
        return 8 * KB
    if rel == "docs/migration/LOG.md" or rel.startswith("docs/migration/log/"):
        return None
    if rel == "docs/migration/checklist/DONE.md" or rel.startswith("docs/migration/checklist/archive/"):
        return None  # append-only archives of finished items (tools/archive-checklist.py)
    return 40 * KB


def plan_files():
    yield "MIGRATION-AGENT.md"
    for dp, dns, fns in os.walk(DOCS):
        dns.sort()
        for fn in sorted(fns):
            if fn.endswith(".md"):
                yield os.path.relpath(os.path.join(dp, fn), ROOT)


def line_of(text, pos):
    a = text.rfind("\n", 0, pos) + 1
    b = text.find("\n", pos)
    return text[a: b if b != -1 else len(text)]


def dangling_refs(rel, text):
    """Bare §N references that are not links (and, in the entry file, not one of its own sections)."""
    out = []
    for m in re.finditer(r"(\[?)§(\d+b?)", text):
        n = m.group(2)
        # a link looks like [§5](...) or [§5 "Usage cap"](...)
        tail = text[m.end():m.end() + 40]
        if m.group(1) == "[" and "](" in tail:
            continue
        if n in ENTRY_SECTIONS and (rel == "MIGRATION-AGENT.md" or "MIGRATION-AGENT.md" in line_of(text, m.start())):
            continue
        line = text.count("\n", 0, m.start()) + 1
        out.append("%s:%d: dangling §%s" % (rel, line, n))
    return out


def main():
    errors = []
    for rel in plan_files():
        path = os.path.join(ROOT, rel)
        if not os.path.exists(path):
            errors.append("%s: missing" % rel)
            continue
        size = os.path.getsize(path)
        cap = cap_for(rel)
        if cap is not None and size > cap:
            errors.append("%s: %d bytes > %d (move content to docs/migration/log/ or NOTES.md)" % (rel, size, cap))
        errors += dangling_refs(rel, open(path, encoding="utf-8").read())
    if errors:
        print("\n".join(errors), file=sys.stderr)
        print("check-docsize: %d problem(s)" % len(errors), file=sys.stderr)
        return 1
    print("check-docsize: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
