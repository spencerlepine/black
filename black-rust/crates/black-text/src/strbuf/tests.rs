use super::*;
use crate::hooks::{MsgHooks, StrHeap};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

/// The allocator / free callees routed to the spec's stub lines. The one block handed out is
/// the spec's second buffer (`blk`, at address `blk_addr`).
struct Rom<'a> {
    env: &'a mut StubEnv,
    blk: &'a mut [u8],
    blk_addr: u32,
}

impl StrHeap for Rom<'_> {
    fn alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32 {
        let r = self.env.call(
            0x0203_0734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(align),
                Arg::V(file),
                Arg::V(line),
            ],
        );
        assert_eq!(r, self.blk_addr);
        r
    }
    fn free(&mut self, addr: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(addr)]);
    }
    fn bytes(&mut self, addr: u32) -> &mut [u8] {
        assert_eq!(addr, self.blk_addr);
        self.blk
    }
}

/// `FUN_02046180` routed to the spec's stub line.
struct Reload<'a>(&'a mut StubEnv);

impl MsgHooks for Reload<'_> {
    fn reload_language(&mut self, _msgdata: u32) {
        self.0.call(0x0204_6180, &mut [Arg::P]);
    }
}

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02045730, "05fa7904a9b8fc41"),
    (0x02045738, "d1734f8e8957fc6b"),
    (0x02045740, "64569e29755e05bf"),
    (0x02045748, "dc9fb51ac6137542"),
    (0x02045750, "186ec6cf425099c0"),
    (0x02045758, "807f1e591b0c9f36"),
    (0x02045760, "30b2ed9e557b0359"),
    (0x02045768, "f2da71322ae527cd"),
    (0x02045770, "717500d192b0fe29"),
    (0x02045778, "f17f0954b2057458"),
    (0x02045780, "0d984ca18ce6021e"),
    (0x02045788, "ce952d0b65c7f581"),
    (0x02045790, "a81258b3839882f3"),
    (0x02045798, "e4c1878f641ab0c9"),
    (0x020457A0, "2bc2101b303009f6"),
    (0x020457B0, "c3c55071751832f3"),
    (0x02045808, "3f6ab79b64960154"),
    (0x0204581C, "88370a64b1e0b0f9"),
    (0x02045838, "988de18f2cc7f8a8"),
    (0x02045874, "ffd265c81aaa63e1"),
    (0x0204589C, "18717d13b6d43a2a"),
    (0x020458F8, "7748498dd2285aa4"),
    (0x02045900, "57038791849003e6"),
    (0x02045924, "ecd6f70d220e7f1a"),
    (0x02045A00, "7346fdc1b57544e7"),
    (0x02045A38, "3d4cbdea1994a564"),
    (0x02045A90, "eb157f9bd332f13a"),
    (0x02045A98, "07a1f52f9d44df45"),
    (0x02045AA8, "7b79b8250259c156"),
    (0x02045AFC, "a59cfb73f8b650e9"),
    (0x02045CAC, "9b29e92e49b03e0f"),
    (0x02045E04, "e91b9e90b1404424"),
    (0x02045E3C, "d7515e9e14fdc4c2"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0204_5730 => rec_u8_2(&c.bufs[0]) as u64,
        0x0204_5738 => rec_u8_5(&c.bufs[0]) as u64,
        0x0204_5740 => rec_u8_6(&c.bufs[0]) as u64,
        0x0204_5748 => rec_u8_7(&c.bufs[0]) as u64,
        0x0204_5750 => rec_u8_8(&c.bufs[0]) as u64,
        0x0204_5758 => rec_u8_3(&c.bufs[0]) as u64,
        0x0204_5760 => rec_u8_4(&c.bufs[0]) as u64,
        0x0204_5768 => rec_u16_a(&c.bufs[0]) as u64,
        0x0204_5770 => rec_u32_c(&c.bufs[0]) as u64,
        0x0204_5778 => rec_u8_9(&c.bufs[0]) as u64,
        0x0204_5780 => {
            let v = c.vals[1] as u8;
            set_rec_u8_3(&mut c.bufs[0], v);
            0
        }
        0x0204_5788 => {
            let v = c.vals[1] as u8;
            set_rec_u8_4(&mut c.bufs[0], v);
            0
        }
        0x0204_5790 => {
            let v = c.vals[1] as u8;
            set_rec_u8_8(&mut c.bufs[0], v);
            0
        }
        0x0204_5798 => {
            let v = c.vals[1] as u8;
            set_rec_u8_9(&mut c.bufs[0], v);
            0
        }
        0x0204_57A0 => {
            let v = c.vals[0] as u16;
            set_eos(&mut c.globals[0], v);
            0
        }
        0x0204_57B0 => {
            let (cap, heap, base) = (c.vals[0], c.vals[1], c.vals[2]);
            let mut h = Rom {
                env: &mut c.env,
                blk: &mut c.bufs[2],
                blk_addr: base,
            };
            create(cap, heap, &mut h).wrapping_sub(base) as u64
        }
        0x0204_5808 => {
            let base = c.vals[0];
            let mut none = [];
            let mut h = Rom {
                env: &mut c.env,
                blk: &mut none,
                blk_addr: base,
            };
            free(&mut c.bufs[0], base, &mut h);
            0
        }
        0x0204_5874 => {
            let (heap, base) = (c.vals[1], c.vals[2]);
            let (a, b) = c.bufs.split_at_mut(2);
            let mut h = Rom {
                env: &mut c.env,
                blk: &mut b[0],
                blk_addr: base,
            };
            dup(&a[0], heap, &mut h).wrapping_sub(base) as u64
        }
        0x0204_5924 => {
            let (a, b) = c.bufs.split_at_mut(1);
            copy_chars(&mut a[0], &b[0]);
            0
        }
        0x0204_5A00 => {
            let n = c.vals[2];
            let (a, b) = c.bufs.split_at_mut(1);
            set_from(&mut a[0], &b[0], n);
            0
        }
        0x0204_5A38 => {
            let n = c.vals[2];
            let (a, b) = c.bufs.split_at_mut(1);
            copy_out(&a[0], &mut b[0], n);
            0
        }
        0x0204_5E04 => {
            let id = c.vals[1];
            decrypt_buf(&mut c.bufs[0], id);
            0
        }
        0x0204_5CAC => {
            let (md, id) = (c.vals[0], c.vals[1]);
            let count = rd16(&c.bufs[3], 2);
            let table = c.bufs[4].clone();
            msg_get(
                md,
                count,
                &table,
                id,
                &mut c.bufs[2],
                &mut Reload(&mut c.env),
            );
            0
        }
        0x0204_581C => {
            clear(&mut c.bufs[0]);
            0
        }
        0x0204_5838 => {
            let (a, b) = c.bufs.split_at_mut(1);
            copy(&mut a[0], &b[0]);
            0
        }
        0x0204_589C => eq(&c.bufs[0], &c.bufs[1]) as u64,
        0x0204_58F8 => len(&c.bufs[0]) as u64,
        0x0204_5900 => {
            let i = c.vals[1];
            truncate(&mut c.bufs[0], i);
            0
        }
        0x0204_5A90 => CHARS_OFS as u64,
        0x0204_5A98 => EOS as u64,
        0x0204_5AA8 => {
            let (a, b) = c.bufs.split_at_mut(1);
            append(&mut a[0], &b[0]);
            0
        }
        0x0204_5AFC => {
            let v = c.vals[1] as u16;
            add_char(&mut c.bufs[0], v);
            0
        }
        0x0204_5E3C => {
            let (n, id) = (c.vals[1], c.vals[2]);
            decrypt(&mut c.bufs[0], n, id) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-text strbuf`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "strbuf",
            include_str!("../../../../tools/rom-diff/specs/black-text/strbuf.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn add_and_truncate() {
    let mut sb = vec![0u8; 0x20];
    wr16(&mut sb, 0, 4);
    clear(&mut sb);
    for c in [1, 2, 3, 4] {
        add_char(&mut sb, c);
    }
    assert_eq!(len(&sb), 3); // capacity 4 keeps one slot for the terminator
    assert_eq!(ch(&sb, 3), EOS);
    truncate(&mut sb, 1);
    assert_eq!((len(&sb), ch(&sb, 1)), (1, EOS));
}

#[test]
fn decrypt_round_trip_key() {
    // id 0xFFFD gives key 0, so a stored EOS decrypts to EOS at once
    let mut b = [0xFFu8; 8];
    assert_eq!(decrypt(&mut b, 4, 0xFFFD), 0);
    assert_eq!(decrypt(&mut [0u8; 8], 0, 5), 0);
}
