use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216A420, "21e6ccfe4b7cd07f"),
    (0x0216A838, "09ff5a60f11888a6"),
    (0x0216AAE4, "dc043948ac8ac24c"),
    (0x0216B538, "67e732fc1407b9a5"),
    (0x0216B5AC, "370f953244d6d584"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer of the case in one memory window set (at the emulator's addresses).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl Rom<'_> {
    fn s(&mut self, addr: u32, args: &[u32]) -> u32 {
        let mut a: Vec<Arg> = args.iter().map(|&v| Arg::V(v)).collect();
        self.stubs.call(addr, &mut a)
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl Misc10Hooks for Rom<'_> {
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5E4, &[ctx])
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5F4, &[ctx])
    }
    fn fun_020120f4(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F4, &[gamesys])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2EBC, &[gamedata])
    }
    fn fun_02012f50(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F50, &[gamedata])
    }
    fn fun_021d0a48(&mut self, gamesys: u32, rec: u32) {
        self.s(0x021D_0A48, &[gamesys, rec]);
    }
    fn fun_0200c0f0(&mut self, save: u32) -> u32 {
        self.s(0x0200_C0F0, &[save])
    }
    fn fun_0200c224(&mut self, block: u32) -> u32 {
        self.s(0x0200_C224, &[block])
    }
    fn fun_0200c22c(&mut self, block: u32, n: u32) {
        self.s(0x0200_C22C, &[block, n]);
    }
    fn fun_0200c33c(&mut self, block: u32, i: u32) -> u32 {
        self.s(0x0200_C33C, &[block, i])
    }
    fn fun_02027138(&mut self, n: u32) {
        self.s(0x0202_7138, &[n]);
    }
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]) {
        self.stubs
            .call(0x0216_D89C, &mut [Arg::V(mmdl), Arg::In(pos)]);
    }
    fn fun_0200ca1c(&mut self, x: u32) -> u32 {
        self.s(0x0200_CA1C, &[x])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA18, &[party])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.s(0x0201_AB28, &[party, i])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.s(0x0201_7E1C, &[mon, id, x])
    }
    fn fun_02017988(&mut self, mon: u32, form: u32) {
        self.s(0x0201_7988, &[mon, form]);
    }
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32) {
        self.s(0x0200_CFA0, &[dex, mon]);
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address and the function's globals at `globals`; the
/// buffers are put back afterwards.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(Some(i));
        }
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push(None);
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    let mut k = 0;
    for (slot, (_, bytes)) in keep.into_iter().zip(wins) {
        match slot {
            Some(i) => c.bufs[i] = bytes,
            None => {
                c.globals[k] = bytes;
                k += 1;
            }
        }
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(c, &[], |h, v| match addr {
        0x0216_A420 => cmd_506(h, v[0], v[1]) as u64,
        0x0216_A838 => event_549(h, v[0], v[1], v[2]) as u64,
        0x0216_AAE4 => {
            update_season_forms(h, v[0], v[1], v[2]);
            0
        }
        0x0216_B538 => cmd_543(h, v[0], v[1]) as u64,
        0x0216_B5AC => cmd_531(h, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
