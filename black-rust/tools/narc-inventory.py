"""A0: inventory of the ROM NARCs (files/a/X/Y/Z, ARC ID = XYZ decimal; path table at main 0x020A6C58).

Prints per NARC: total size, member count, member size range, member magic / LZ counts.
Usage: python3 -I tools/narc-inventory.py [black-rom/files/a]
"""
import struct,os,collections
import sys
R=sys.argv[1] if len(sys.argv)>1 else os.path.join(os.path.dirname(os.path.abspath(__file__)),'../../black-rom/files/a')
def narc(b):
    assert b[:4]==b'NARC'
    o=struct.unpack_from('<H',b,12)[0]
    # BTAF
    assert b[o:o+4]==b'BTAF'; sz=struct.unpack_from('<I',b,o+4)[0]; n=struct.unpack_from('<I',b,o+8)[0]
    fat=[struct.unpack_from('<II',b,o+12+8*i) for i in range(n)]
    o2=o+sz; assert b[o2:o2+4]==b'BTNF'; sz2=struct.unpack_from('<I',b,o2+4)[0]
    o3=o2+sz2; assert b[o3:o3+4]==b'GMIF'
    d=o3+8
    return [b[d+s:d+e] for s,e in fat], sz2>0x10
out=[]
for i in range(235):
    p='%s/%d/%d/%d'%(R,i//100,i//10%10,i%10)
    b=open(p,'rb').read()
    try: fs,named=narc(b)
    except Exception as e: out.append('%03d %8d NOT-NARC %r'%(i,len(b),b[:4])); continue
    mag=collections.Counter()
    for f in fs:
        m=f[:4]
        if f[:1] in (b'\x10',b'\x11') and len(f)>4: k='LZ%x'%f[0]
        elif all(32<=c<127 for c in m): k=m.decode()
        else: k='bin'
        mag[k]+=1
    sizes=sorted(len(f) for f in fs)
    out.append('%03d %8d n=%-5d %s sz[min %d med %d max %d] %s'%(i,len(b),len(fs),'named' if named else '',sizes[0] if sizes else 0,sizes[len(sizes)//2] if sizes else 0,sizes[-1] if sizes else 0,' '.join('%s:%d'%kv for kv in mag.most_common(4))))

print('\n'.join(out))
