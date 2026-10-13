"""Unit tests of svflow.py's stub model and PORTED keying (Z20). Run with the Unicorn venv:

  /tmp/blk/venv/bin/python -I tools/rom-diff/test_svflow.py

The stub vectors are the same as in crates/black-fndiff/src/stub.rs (`outk_*` tests): both sides must agree.
"""
import os
import struct
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import svflow as S  # noqa: E402


class FakeEm:
    """Just enough of emu.Emu for make_stub: a sparse byte memory."""

    def __init__(self):
        self.mem = {}

    def rbytes(self, a, n):
        return bytes(self.mem.get(a + i, 0) for i in range(n))

    def wbytes(self, a, b):
        for i, x in enumerate(b):
            self.mem[a + i] = x

    def r32(self, a):
        return struct.unpack("<I", self.rbytes(a, 4))[0]


def call(args, raw, em, seq=True, salt=0x1234):
    ctx = S.Ctx()
    ctx.salt = salt
    f = S.make_stub(S.Stub(0x021BB798, "u32", args, False, seq), ctx)
    raw = list(raw) + [0] * (4 - len(raw))
    return f(em, raw[0], raw[1], raw[2], raw[3], 0x02EF0000), ctx


class OutV(unittest.TestCase):
    # F3a-spec `outv:N:V1/V2/..`: the same vectors as crates/black-fndiff/src/stub.rs (`outv_writes_one_of_the_listed_values`)
    def test_outv_picks_a_listed_value(self):
        em = FakeEm()
        r, _ = call(["v", "outv:2:0x3FFF/0/1"], [0xABCD, 0x1000], em, seq=False)
        self.assertEqual(list(em.rbytes(0x1000, 4)), [0xFF, 0x3F, 0, 0])  # only N bytes are written
        self.assertEqual(r, 0x731C9F54)
        em = FakeEm()
        r, _ = call(["outv:4:0x11223344/5/6/7/8"], [0x1000], em, seq=False)
        self.assertEqual((r, em.r32(0x1000)), (0x18685E8C, 7))
        em = FakeEm()
        r, _ = call(["v", "outv:4:0x11223344/5/6/7/8"], [9, 0x1000], em, seq=False)
        self.assertEqual((r, em.r32(0x1000)), (0x35129D94, 6))


class OutK(unittest.TestCase):
    def test_outm_vector_still_holds(self):
        # the B10-specs vector of crates/black-fndiff/src/stub.rs (`outm_masks_the_bytes_and_ptr_returns_an_address`)
        em = FakeEm()
        call(["v", "outm:5:0xF"], [0xABCD, 0x1000], em)
        self.assertEqual(list(em.rbytes(0x1000, 5)), [1, 7, 15, 3, 15])

    def test_outk_keeps_the_bits_outside_the_mask(self):
        em = FakeEm()
        em.wbytes(0x1000, bytes([0x11, 0x22, 0x33, 0x44, 0x55]))
        r, ctx = call(["v", "outk:5:0xF/0"], [0xABCD, 0x1000], em)
        # bits under 0xF = the `outm:5:0xF` bytes [1, 7, 15, 3, 15] masked per byte with 0xF / 0 / 0xF / 0 / 0xF
        self.assertEqual(list(em.rbytes(0x1000, 5)), [0x11, 0x22, 0x3F, 0x44, 0x5F])
        self.assertEqual(ctx.log, [0x021BB798, 0xABCD])  # the out argument is not logged
        # same result word as an `outm` call (outs never feed the result)
        em2 = FakeEm()
        r2, _ = call(["v", "outm:5:0xF"], [0xABCD, 0x1000], em2)
        self.assertEqual(r, r2)

    def test_outk_full_mask_is_outm(self):
        em, em2 = FakeEm(), FakeEm()
        em.wbytes(0x1000, bytes([0xAA] * 6))
        call(["v", "outk:6:0xFF"], [0xABCD, 0x1000], em)
        call(["v", "outm:6:0xFF"], [0xABCD, 0x1000], em2)
        self.assertEqual(em.rbytes(0x1000, 6), em2.rbytes(0x1000, 6))
        self.assertEqual(list(em.rbytes(0x1000, 6)), [113, 135, 79, 211, 223, 219])

    def test_outki_writes_the_record_behind_the_pointer(self):
        # arg 0 = a struct whose u32 at +0x4A8 is the record address: same bytes as `outk` on the record itself
        em, em2 = FakeEm(), FakeEm()
        em.wbytes(0x2000 + 0x4A8, struct.pack("<I", 0x3000))
        for e in (em, em2):
            e.wbytes(0x3000, bytes([0x10, 0x20, 0x30, 0x40, 0x50, 0x67]))
        call(["outki:0x4A8:6:0/0/0/0/0/3", "v"], [0x2000, 7], em)
        call(["outk:6:0/0/0/0/0/3", "v"], [0x3000, 7], em2)
        self.assertEqual(em.rbytes(0x3000, 6), em2.rbytes(0x3000, 6))
        got = list(em.rbytes(0x3000, 6))
        self.assertEqual(got[:5], [0x10, 0x20, 0x30, 0x40, 0x50])
        self.assertEqual(got[5] & ~3 & 0xFF, 0x64)
        self.assertEqual(got, [0x10, 0x20, 0x30, 0x40, 0x50, 0x65])
        self.assertEqual(em.r32(0x2000 + 0x4A8), 0x3000)  # the struct itself is untouched


class OutW(unittest.TestCase):
    """F8-spec4: `outw:N:W0/W1/..` fills N bytes with 32-bit words picked from the list (vectors shared with
    crates/black-fndiff/src/stub.rs `outw_picks_words_from_the_list`)."""

    def test_outw_picks_listed_words(self):
        em = FakeEm()
        r, ctx = call(["v", "outw:12:0/0x5A000/0xB4000/0x10E000"], [0xABCD, 0x1000], em)
        self.assertEqual(list(struct.unpack("<3I", em.rbytes(0x1000, 12))), [0xB4000, 0x5A000, 0xB4000])
        self.assertEqual(ctx.log, [0x021BB798, 0xABCD])  # the out argument is not logged
        em2 = FakeEm()
        r2, _ = call(["v", "outm:12:0xFF"], [0xABCD, 0x1000], em2)
        self.assertEqual(r, r2)  # outs never feed the result

    def test_outw_argument_position_and_no_seq(self):
        em = FakeEm()
        call(["outw:8:1/2/3", "v"], [0x1000, 7], em, seq=False)
        self.assertEqual(list(struct.unpack("<2I", em.rbytes(0x1000, 8))), [2, 2])

class InFeedsTheAnswer(unittest.TestCase):
    """Z22: the words an `in:N` stub reads are mixed into its answer (after the scalars), so two queries at different
    positions answer differently. Same vectors as `in_words_feed_the_answer` in crates/black-fndiff/src/stub.rs."""

    def test_in_words_change_the_result(self):
        em = FakeEm()
        em.wbytes(0x1000, bytes([1, 2, 3, 4, 5, 6, 7, 8]))
        em.wbytes(0x2000, bytes([1, 2, 3, 4, 5, 6, 7, 9]))
        r1, c1 = call(["v", "in:8"], [0xABCD, 0x1000], em, seq=False)
        r2, _ = call(["v", "in:8"], [0xABCD, 0x2000], em, seq=False)
        self.assertEqual((r1, r2), (0xF747FDDA, 0xCE42B9D0))
        self.assertEqual(c1.log, [0x021BB798, 0xABCD, 0x04030201, 0x08070605])  # logging is unchanged
        # no `in:` argument: the answer is the old one (the `u64_stub_returns_two_words` low word)
        r3, _ = call(["v"], [0xABCD], em, seq=False)
        self.assertEqual(r3, 0x731C9F54)

    def test_in_words_feed_the_outs_too(self):
        em = FakeEm()
        em.wbytes(0x3000, bytes([0xAA] * 3))
        r, ctx = call(["in:3", "v", "out:4"], [0x3000, 7, 0x4000], em, seq=False)
        self.assertEqual(r, 0x2DC13597)
        self.assertEqual(list(em.rbytes(0x4000, 4)), [36, 203, 171, 8])
        self.assertEqual(ctx.log, [0x021BB798, 7, 0xAAAAAA])


class PortedPerSet(unittest.TestCase):
    """Z20: the black-battle modules' PORTED lists deactivate stubs only in battle-set areas."""

    def setUp(self):
        self.saved = (S.SET[0], list(S.PORTED_SRC), set(S.CRATE_PORTED))

    def tearDown(self):
        S.SET[0], S.PORTED_SRC[:], = self.saved[0], self.saved[1]
        S.CRATE_PORTED.clear()
        S.CRATE_PORTED.update(self.saved[2])

    def test_check_fn_area_sees_battle_modules_only_on_the_battle_set(self):
        own = os.path.join(S.ROOT, "crates", "black-field", "src", "fieldmap.rs")
        S.PORTED_SRC[:] = [own]
        flow = set(S.ported_flow())
        self.assertTrue(flow)
        S.SET[0] = "field"
        self.assertEqual(set(S.ported()), set(S.ported_in(own)))
        self.assertEqual(set(S.ported("fieldmap")), set(S.ported_in(own)))
        S.SET[0] = "battle"
        self.assertTrue(flow <= set(S.ported()))
        S.SET[0] = None  # no `set` line = battle
        self.assertTrue(flow <= set(S.ported()))
        S.SET[0] = "battle+96"  # the battle image with more overlays
        self.assertTrue(flow <= set(S.ported()))
        self.assertEqual(set(S.ported("x")), set(S.ported_in(own)))

    def test_battle_area_unchanged(self):
        S.PORTED_SRC[:] = []
        S.SET[0] = None
        self.assertEqual(S.ported(), S.ported_flow())
        self.assertEqual(S.ported("core1"), S.ported_flow("core1"))

    def test_src_set(self):
        self.assertEqual(S.src_set(os.path.join(S.FLOW_SRC, "core1.rs")), "battle")
        self.assertEqual(S.src_set(os.path.join(S.ROOT, "crates", "black-battle", "src", "handlers", "items.rs")), "battle")
        self.assertEqual(S.src_set(os.path.join(S.ROOT, "crates", "black-field", "src", "fieldmap.rs")), "field")
        self.assertEqual(S.src_set(os.path.join(S.ROOT, "crates", "black-core", "src", "softfloat.rs")), "battle")
        self.assertIsNone(S.src_set(os.path.join(S.ROOT, "crates", "black-field", "src", "lib.rs")))

    def test_crate_ported_drops_other_sets_overlay_addresses(self):
        lo, hi = S.main_range()
        flow_ovl = {a for a in S.ported_flow() if not lo <= a < hi}
        # every field-set crate's PORTED (black-field and black-script; field overlay addresses may equal battle ones,
        # e.g. 0x021B6AAC is battle server1 6aac and ov21 scrcmd_pokemon_fld)
        field = set()
        for root, _, files in os.walk(os.path.join(S.ROOT, "crates")):
            for fn in files:
                path = os.path.join(root, fn)
                if fn.endswith(".rs") and S.src_set(path) == "field":
                    field.update(S.ported_in(path))
        S.SET[0] = "field"
        S.CRATE_PORTED.clear()
        cp = set(S.crate_ported())
        self.assertTrue(field <= cp)
        self.assertFalse((flow_ovl - field) & cp)
        S.SET[0] = None
        S.CRATE_PORTED.clear()
        cp = set(S.crate_ported())
        self.assertTrue(flow_ovl <= cp)
        self.assertFalse({a for a in field if not lo <= a < hi} - flow_ovl & cp)


if __name__ == "__main__":
    unittest.main()
