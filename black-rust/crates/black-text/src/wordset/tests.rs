use super::*;
use black_fndiff::{region_addr, run_suite, Arg, Case, Layout, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201ED50, "e16e4c0ef8aa5ef2"),
    (0x0201ED64, "34a7735b5b3487b2"),
    (0x0201ED84, "4eb71b3463491128"),
    (0x0201EDD0, "6e4a340bc5ce9f94"),
    (0x0201EE0C, "167ce35d533747ed"),
    (0x0201EE60, "65515b114ad0a99d"),
    (0x0201EFA4, "48b4140c6d61b95b"),
    (0x0201F02C, "9344c162196335bb"),
    (0x0201F048, "e4bd7b0df4df07ad"),
    (0x0201F184, "9288dffa9ca2226a"),
    (0x0201F198, "7cde83bb8eb21489"),
    (0x0201F1B0, "fcec8166847561c3"),
    (0x0201F1C8, "5d4dd311ea57578b"),
    (0x0201F1E0, "0a74891875a7ce2b"),
    (0x0201F230, "7dfd8d3571413682"),
    (0x0201F250, "a19a8c445e647e19"),
    (0x0201F31C, "71617b43ba5e3f8f"),
    (0x0201F35C, "0139da324487d3c1"),
    (0x0201F41C, "6d7d017a580265e1"),
    (0x0201F448, "4ed44bd40057e539"),
    (0x0201F47C, "142d9a0d266af799"),
    (0x0201F4A0, "94862a6a7e039611"),
];
// fndiff-recorded-end

/// Fixed addresses of the harness: pointer argument i, region k.
const ARG_BASE: u32 = 0x02E1_0000;
const REGION_BASE: u32 = 0x02F1_0000;
const ENTRIES: usize = 4;

fn rd32(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

/// Region index of the scratch (`regs[ENTRIES]`) and of each entry's string (`regs[i]`).
type Regs = [usize; ENTRIES + 1];

/// Reads the word set the spec builds in `arg0` (header at 0, entries at `*(+8)`, scratch at
/// `*(+0xC)`, entry strings in regions); returns it with the region indices of its buffers.
fn load_ws(c: &Case) -> (WordSet, Regs) {
    let a = &c.bufs[0];
    let base = (rd32(a, 8) - ARG_BASE) as usize;
    let mut regs = [0; ENTRIES + 1];
    regs[ENTRIES] = ((rd32(a, 0xC) - REGION_BASE) / 0x1_0000) as usize;
    let entries = (0..ENTRIES)
        .map(|i| {
            let o = base + 12 * i;
            regs[i] = ((rd32(a, o + 8) - REGION_BASE) / 0x1_0000) as usize;
            Entry {
                info: a[o..o + 5].try_into().unwrap(),
                buf: c.regions[regs[i]].clone(),
            }
        })
        .collect();
    (
        WordSet {
            count: rd32(a, 0),
            heap: rd32(a, 4),
            entries,
            scratch: c.regions[regs[ENTRIES]].clone(),
        },
        regs,
    )
}

/// B18b: the emulator addresses of the word set's string buffers (its copies of the regions), so
/// `FUN_02045CAC`'s StrBuf argument (`ptr`) is the region the original passes.
fn ws_layout(ws: &WordSet, regs: Regs) -> Layout {
    let mut l = Layout::default();
    l.push(&ws.scratch, region_addr(regs[ENTRIES]));
    for (e, &k) in ws.entries.iter().zip(&regs[..ENTRIES]) {
        l.push(&e.buf, region_addr(k));
    }
    l
}

fn store_ws(c: &mut Case, ws: &WordSet, regs: Regs) {
    let base = (rd32(&c.bufs[0], 8) - ARG_BASE) as usize;
    for (i, e) in ws.entries.iter().enumerate() {
        let o = base + 12 * i;
        c.bufs[0][o..o + 5].copy_from_slice(&e.info);
        c.regions[regs[i]].copy_from_slice(&e.buf);
    }
    c.regions[regs[ENTRIES]].copy_from_slice(&ws.scratch);
}

/// The `MsgData` callees and `FUN_02008500`, routed to the spec's stub lines. The string buffers
/// they fill are not written: the stub lines say the callees leave them alone.
struct Stubs<'a>(&'a mut StubEnv, Layout);

impl WordSetHooks for Stubs<'_> {
    fn msg_create(&mut self, typ: u32, arc: u32, bank: u32, heap: u32) -> u32 {
        self.0.call(
            0x0204_5B38,
            &mut [Arg::V(typ), Arg::V(arc), Arg::V(bank), Arg::V(heap)],
        )
    }
    fn msg_fetch(&mut self, md: u32, msgid: u32, sb: &mut [u8]) {
        self.0.call(
            0x0204_5CAC,
            &mut [Arg::V(md), Arg::V(msgid), self.1.ptr(sb)],
        );
    }
    fn msg_delete(&mut self, md: u32) {
        self.0.call(0x0204_5C04, &mut [Arg::V(md)]);
    }
    fn msg_count(&mut self, md: u32) -> u32 {
        self.0.call(0x0204_5F50, &mut [Arg::V(md)])
    }
    fn fill_02008500(&mut self, v: u32, _sb: &mut [u8]) {
        self.0.call(0x0200_8500, &mut [Arg::V(v), Arg::P]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_ED50 => {
            let mut info: [u8; 5] = c.bufs[0][..5].try_into().unwrap();
            entry_reset(&mut info);
            c.bufs[0][..5].copy_from_slice(&info);
            0
        }
        0x0201_ED64 => {
            let (mut ws, regs) = load_ws(c);
            clear_entry(&mut ws, c.vals[1] as usize);
            store_ws(c, &ws, regs);
            0
        }
        0x0201_ED84 => {
            let (mut ws, regs) = load_ws(c);
            let info: Option<[u8; 5]> =
                (c.vals[3] != 0).then(|| c.bufs[4][..5].try_into().unwrap());
            set_entry(&mut ws, c.vals[1] as usize, &c.bufs[2], info.as_ref());
            store_ws(c, &ws, regs);
            0
        }
        0x0201_EDD0 => {
            let (mut ws, regs) = load_ws(c);
            let lay = ws_layout(&ws, regs);
            let (bank, msgid) = (c.vals[2], c.vals[3]);
            set_msg(
                &mut ws,
                c.vals[1] as usize,
                bank,
                msgid,
                &mut Stubs(&mut c.env, lay),
            );
            store_ws(c, &ws, regs);
            0
        }
        0x0201_EE0C => {
            let (mut ws, regs) = load_ws(c);
            let src = c.bufs[2].clone();
            set_entry_count(&mut ws, c.vals[1] as usize, &src, c.vals[3], c.vals[4]);
            store_ws(c, &ws, regs);
            0
        }
        0x0201_EE60 => {
            let (mut ws, regs) = load_ws(c);
            let lay = ws_layout(&ws, regs);
            let md = rd32(&c.globals[0], 0);
            set_msg_global(
                &mut ws,
                c.vals[1] as usize,
                md,
                c.vals[2],
                &mut Stubs(&mut c.env, lay),
            );
            store_ws(c, &ws, regs);
            0
        }
        0x0201_EFA4 => {
            let (mut ws, regs) = load_ws(c);
            let v = &c.vals;
            set_number(&mut ws, v[1] as usize, v[2] as i32, v[3], v[4], v[5]);
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F02C => {
            let (mut ws, regs) = load_ws(c);
            let lay = ws_layout(&ws, regs);
            set_via_02008500(
                &mut ws,
                c.vals[1] as usize,
                c.vals[2],
                &mut Stubs(&mut c.env, lay),
            );
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F048 => {
            let (mut ws, regs) = load_ws(c);
            let lay = ws_layout(&ws, regs);
            set_msg_bank_e7(
                &mut ws,
                c.vals[1] as usize,
                c.vals[2],
                &mut Stubs(&mut c.env, lay),
            );
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F184 | 0x0201_F198 | 0x0201_F1B0 | 0x0201_F1C8 | 0x0201_F1E0 | 0x0201_F230 => {
            let (mut ws, regs) = load_ws(c);
            let lay = ws_layout(&ws, regs);
            let (i, v) = (c.vals[1] as usize, c.vals[2]);
            let h = &mut Stubs(&mut c.env, lay);
            match addr {
                0x0201_F184 => set_msg_bank_a0(&mut ws, i, v, h),
                0x0201_F198 => set_msg_bank_2b(&mut ws, i, v, h),
                0x0201_F1B0 => set_msg_bank_0(&mut ws, i, v, h),
                0x0201_F1C8 => set_msg_bank_38(&mut ws, i, v, h),
                0x0201_F1E0 => set_msg_bank_461(&mut ws, i, v, h),
                _ => set_msg_bank_278(&mut ws, i, v, h),
            }
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F250 => {
            let (mut ws, regs) = load_ws(c);
            let src = c.bufs[2].clone();
            expand(&mut ws, &mut c.bufs[1], &src);
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F31C => {
            let (mut ws, regs) = load_ws(c);
            clear_all(&mut ws);
            store_ws(c, &ws, regs);
            0
        }
        0x0201_F35C => {
            let (v, d, m, a) = (c.vals[1], c.vals[2], c.vals[3], c.vals[4]);
            format_number(&mut c.bufs[0], v as i32, d, m, a != 0);
            0
        }
        0x0201_F41C => {
            let (a, b) = c.bufs.split_at_mut(1);
            copy_units(&a[0], &mut b[0], c.vals[2]);
            0
        }
        0x0201_F448 => units_eq(&c.bufs[0], &c.bufs[1]) as u64,
        0x0201_F47C => is_compressed(&c.bufs[0]) as u64,
        0x0201_F4A0 => {
            let (a, b) = c.bufs.split_at_mut(1);
            append_word(&mut a[0], &b[0]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-text wordset`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "wordset",
            include_str!("../../../../tools/rom-diff/specs/black-text/wordset.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

fn sb(cap: u16, text: &[u16]) -> Vec<u8> {
    let mut b = vec![0u8; 0x60];
    strbuf::wr16(&mut b, 0, cap);
    clear(&mut b);
    for &c in text {
        add_char(&mut b, c);
    }
    b
}

fn text(b: &[u8]) -> Vec<u16> {
    (0..len(b) as usize).map(|i| ch(b, i)).collect()
}

#[test]
fn number_formats() {
    let mut b = sb(20, &[]);
    format_number(&mut b, 305, 5, 0, true);
    assert_eq!(text(&b), [0x33, 0x30, 0x35]);
    format_number(&mut b, 305, 5, 1, true);
    assert_eq!(text(&b), [0x20, 0x20, 0x33, 0x30, 0x35]);
    format_number(&mut b, -7, 3, 2, false);
    assert_eq!(text(&b), [0x30FC, 0xFF10, 0xFF10, 0xFF17]);
    format_number(&mut b, 0, 3, 0, true);
    assert_eq!(text(&b), [0x30]);
}

#[test]
fn expand_substitutes_a_word_and_copies_other_tags() {
    let word = sb(20, &[0x41, 0x42]);
    let mut ws = WordSet {
        count: 2,
        heap: 0,
        scratch: sb(20, &[]),
        entries: vec![
            Entry {
                info: [0; 5],
                buf: sb(20, &[]),
            },
            Entry {
                info: [0; 5],
                buf: word,
            },
        ],
    };
    // "x" {F000 0100 1 [1]} {F000 0300 1 [9]} "y"
    let src = sb(40, &[0x78, TAG, 0x0100, 1, 1, TAG, 0x0300, 1, 9, 0x79]);
    let mut dst = sb(30, &[]);
    expand(&mut ws, &mut dst, &src);
    assert_eq!(text(&dst), [0x78, 0x41, 0x42, TAG, 0x0300, 1, 9, 0x79]);
}

#[test]
fn compressed_word_decodes_9_bit_codes() {
    // codes 0x41, 0x100, 0x1FF packed LSB first after the F100 header
    let bits: u32 = 0x41 | 0x100 << 9 | 0x1FF << 18;
    let src = sb(20, &[COMPRESSED, bits as u16, (bits >> 16) as u16, 0xFFFF]);
    assert!(is_compressed(&src));
    let mut dst = sb(20, &[]);
    append_word(&mut dst, &src);
    assert_eq!(text(&dst), [0x41, 0x152]);
}
