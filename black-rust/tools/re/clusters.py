#!/usr/bin/env python3
"""Group decompiled functions by shape (names and numeric constants masked).

usage: clusters.py <pc dir>   (output of DecompileAll.java)
A cluster is one porting job: port one member by hand, generate the rest from its constants.
"""
import collections
import os
import re
import sys

d = sys.argv[1]
cl = collections.defaultdict(list)
for f in sorted(os.listdir(d)):
    if not f.endswith(".c"):
        continue
    s = open(os.path.join(d, f)).read()
    s = s.replace(f[9:-2], "SELF")
    s = re.sub(r"\b(FUN|sub|LAB|DAT|PTR|thunk)_\w+", "ID", s)
    s = re.sub(r"\b0x[0-9a-fA-F]+\b|\b\d+\b", "K", s)
    cl[s].append(f[:-2])
groups = sorted(cl.values(), key=len, reverse=True)
multi = [g for g in groups if len(g) > 1]
n = sum(len(g) for g in groups)
print("%d functions, %d shapes, %d clusters of 2+ covering %d functions" % (n, len(groups), len(multi), sum(map(len, multi))))
for g in multi:
    print("%4d  %s" % (len(g), " ".join(g[:6]) + (" ..." if len(g) > 6 else "")))
