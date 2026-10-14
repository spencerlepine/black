use super::*;
use crate::le::{r32, w16, w16s, w32};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200EA14, "25ca311b27a3fe19"),
    (0x0200EA2C, "fc4a5ce386faadc1"),
    (0x0200EA3C, "22fcd5fa070606ca"),
    (0x0200EC14, "8c59a798414b731e"),
    (0x0200EC2C, "9f85b071c7561973"),
    (0x0200EC98, "0e07f20606f73ede"),
];
// fndiff-recorded-end

/// The callees routed to the spec's stub lines (string copies write nothing).
/// B18b: `.1` = (host address, emulator address) of the name arrays of the port's record copy, so
/// `FUN_02045A38`'s `dst` (a `ptr` stub argument) is the address in the block (`arg0`) the original
/// passes; any other array is a local of the port.
struct Rom<'a>(&'a mut StubEnv, Vec<(usize, u32)>);

/// B18b: the names of entry `k` at block + `MON_SIZE * k` + 0x0C (nickname) / + 0x22 (OT name).
fn name_addrs(r: &HofRecord) -> Vec<(usize, u32)> {
    let base = black_fndiff::buf_addr(0);
    let mut v = Vec::new();
    for (k, m) in r.mons.iter().enumerate() {
        let e = base + (MON_SIZE * k) as u32;
        v.push((m.nickname.as_ptr() as usize, e + 0x0C));
        v.push((m.ot_name.as_ptr() as usize, e + 0x22));
    }
    v
}

impl HofHooks for Rom<'_> {
    fn party_count(&mut self, party: u32) -> u32 {
        self.0.call(0x0201_AA18, &mut [Arg::V(party)])
    }
    fn party_mon(&mut self, party: u32, i: u32) -> u32 {
        self.0.call(0x0201_AB28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn lock(&mut self, mon: u32) -> u32 {
        self.0.call(0x0201_7D30, &mut [Arg::V(mon)])
    }
    fn unlock(&mut self, mon: u32, state: u32) {
        self.0.call(0x0201_7D70, &mut [Arg::V(mon), Arg::V(state)]);
    }
    fn param(&mut self, mon: u32, id: u32, arg: u32) -> u32 {
        self.0
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::V(arg)])
    }
    fn str_alloc(&mut self, heap: u32) -> u32 {
        self.0.call(0x0204_57B0, &mut [Arg::V(11), Arg::V(heap)])
    }
    fn str_read(&mut self, s: u32, dst: &mut [u16]) {
        self.0.call(
            0x0204_5A38,
            &mut [
                Arg::V(s),
                Arg::Ptr(
                    self.1
                        .iter()
                        .find(|&&(h, _)| h == dst.as_ptr() as usize)
                        .map_or(black_fndiff::STACK_ZERO.start, |&(_, e)| e),
                ),
                Arg::V(dst.len() as u32),
            ],
        );
    }
    fn str_write(&mut self, s: u32, src: &[u16]) {
        let mut b = [0u8; 16];
        w16s(&mut b, 0, &src[..8]);
        self.0.call(
            0x0204_5994,
            &mut [Arg::V(s), Arg::In(&b), Arg::V(src.len() as u32)],
        );
    }
    fn str_free(&mut self, s: u32) {
        self.0.call(0x0204_5808, &mut [Arg::V(s)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_EA14 {
        return HofRecord::size() as u64;
    }
    let mut r = HofRecord::from_bytes(&c.bufs[0]);
    match addr {
        0x0200_EA2C => r.init(),
        0x0200_EA3C => {
            let d = &c.bufs[2];
            let date = [r32(d, 0), r32(d, 4), r32(d, 8)];
            let names = name_addrs(&r);
            r.fill(v[1], date, v[3], &mut Rom(&mut c.env, names));
        }
        0x0200_EC14 => return r.count() as u64,
        0x0200_EC2C => {
            let o = c.bufs[2].clone();
            let mut view = HofMonView::default();
            r.mon_view(
                v[1] as usize,
                &mut view,
                r32(&o, 0),
                r32(&o, 4),
                &mut Rom(&mut c.env, Vec::new()),
            );
            let o = &mut c.bufs[2];
            w32(o, 8, view.pid);
            w32(o, 12, view.id);
            w16(o, 0x10, view.species);
            o[0x12] = view.form;
            o[0x13] = view.b3;
            w16s(o, 0x14, &view.moves);
        }
        0x0200_EC98 => {
            for (k, w) in r.date().iter().enumerate() {
                w32(&mut c.bufs[1], 4 * k, *w);
            }
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
    c.bufs[0] = r.to_bytes();
    0
}

/// Every ported function against the original (`tools/check-fn.sh black-storage dendou_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "dendou_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/dendou_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn count_and_round_trip() {
    let mut r = HofRecord::default();
    r.mons[0].species = 495;
    r.mons[1].species = 498;
    r.year = 2026;
    assert_eq!(r.count(), 2);
    assert_eq!(r.date(), [2026, 0, 0, 0]);
    assert_eq!(HofRecord::from_bytes(&r.to_bytes()), r);
}
