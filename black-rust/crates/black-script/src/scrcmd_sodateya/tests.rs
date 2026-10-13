use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215CE98, "aedb3c6d0c351d93"),
    (0x0215CFFC, "8033c18dd85e49f5"),
    (0x0215D14C, "5861e3016c1de71b"),
    (0x0215D198, "7f3652f705cf42cf"),
    (0x0215D1E4, "e220737c4e5b1863"),
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

impl SodateyaHooks for Rom<'_> {
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02188d50(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D50, &[fm])
    }
    fn fun_021c3400(&mut self, daycare: u32) -> u32 {
        self.s(0x021C_3400, &[daycare])
    }
    fn fun_021c34f8(&mut self, daycare: u32, slot: u32) -> u32 {
        self.s(0x021C_34F8, &[daycare, slot])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.s(0x0201_7E1C, &[mon, id, x])
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        self.s(0x0201_2374, &[event, child]);
    }
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32 {
        self.s(0x021B_E8D4, &[gamesys, fm, a, b])
    }
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32 {
        self.s(0x0201_44DC, &[gamesys, fm])
    }
    fn fun_02014910(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32 {
        self.s(0x0201_4910, &[gamesys, fm, a, b])
    }
    fn fun_020145cc(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_45CC, &[gamesys])
    }
    fn fun_021be92c(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32 {
        self.s(0x021B_E92C, &[gamesys, fm, a, b, c, d, e])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
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
        0x0215_CE98 => cmd_235(h, v[0], v[1]) as u64,
        0x0215_CFFC => cmd_242(h, v[0], v[1]) as u64,
        0x0215_D14C => cmd_593(h, v[0], v[1]) as u64,
        0x0215_D198 => gender_mark(h, v[0]) as u64,
        0x0215_D1E4 => event_247(h, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_sodateya`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_sodateya",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_sodateya.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
