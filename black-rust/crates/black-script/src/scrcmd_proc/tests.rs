use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215C63C, "12f00e7615ebb409"),
    (0x0215C69C, "afabfa4ad1aa91c1"),
    (0x0215C6F0, "c92956ddfc1e3ef8"),
    (0x0215C700, "619f7bf9888630ea"),
    (0x0215C72C, "3176212dae74d66b"),
    (0x0215C7C4, "d47a2e75d75d78cb"),
    (0x0215C7E0, "2419ba3c80764644"),
    (0x0215C870, "7c9f20a4e15b7551"),
    (0x0215C89C, "c5712fbe4b167697"),
    (0x0215C95C, "155d9365ece97fdd"),
    (0x0215CA64, "bef04f9b68d4b84c"),
    (0x0215CAB4, "35e23df902fbe5a5"),
    (0x0215CB08, "9074be5407aa79d8"),
    (0x0215CBF0, "e1cf93ad2d6abef8"),
    (0x0215CC38, "5624eb6e1a64c3ef"),
    (0x0215CC4C, "87ccd952da9fb056"),
    (0x0215CCE8, "a1b5d6eadf0c2fc4"),
    (0x0215CD3C, "fe993c887605baf4"),
    (0x0215CD80, "5a594d3b998b6926"),
    (0x0215CDB8, "20f1a9c57087bc1b"),
    (0x0215CDF0, "fd7f083043234471"),
    (0x0215CE70, "bf1fd41c53ce3b5d"),
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
    /// A stub `args=v..,out:N` whose out pointer is the address `out` in the memory window: the stub fills N bytes there.
    fn s_out<const N: usize>(&mut self, addr: u32, vals: &[u32], out: u32) -> u32 {
        let mut buf = [0u8; N];
        let r = {
            let mut a: Vec<Arg> = vals.iter().map(|&v| Arg::V(v)).collect();
            a.push(Arg::Out(&mut buf));
            self.stubs.call(addr, &mut a)
        };
        for (i, b) in buf.iter().enumerate() {
            self.mem.w8(out + i as u32, *b);
        }
        r
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl CoreHooks for Rom<'_> {
    fn fun_02011290(&mut self, vm: u32) {
        self.s(0x0201_1290, &[vm]);
    }
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1330, &[vm])
    }
    fn fun_0201134c(&mut self, vm: u32) -> u32 {
        self.s(0x0201_134C, &[vm])
    }
    fn fun_0201137c(&mut self, vm: u32, v: u32) {
        self.s(0x0201_137C, &[vm, v]);
    }
    fn fun_02011398(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1398, &[vm])
    }
    fn fun_020113b0(&mut self, vm: u32, pc: u32) {
        self.s(0x0201_13B0, &[vm, pc]);
    }
    fn fun_020113b4(&mut self, vm: u32, pc: u32) {
        self.s(0x0201_13B4, &[vm, pc]);
    }
    fn fun_020113c4(&mut self, vm: u32) {
        self.s(0x0201_13C4, &[vm]);
    }
    fn fun_020113d0(&mut self, vm: u32, f: u32) {
        self.s(0x0201_13D0, &[vm, f]);
    }
    fn fun_0215a5d4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5D4, &[ctx])
    }
    fn fun_0215a5d8(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5D8, &[ctx])
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
    fn fun_021587e0(&mut self, sw: u32, slot: u32) -> u32 {
        self.s(0x0215_87E0, &[sw, slot])
    }
    fn fun_0215897c(&mut self, sw: u32, child: u32) {
        self.s(0x0215_897C, &[sw, child]);
    }
    fn fun_021589a0(&mut self, sw: u32, zone: u32, id: u32) -> u32 {
        self.s(0x0215_89A0, &[sw, zone, id])
    }
    fn fun_02158a34(&mut self, gamesys: u32, sw: u32, id: u32, mode: u32) {
        self.s(0x0215_8A34, &[gamesys, sw, id, mode]);
    }
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_02159b38(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_9B38, &[ctx])
    }
    fn fun_0215a684(&mut self, ctx: u32, n: u32) {
        self.s(0x0215_A684, &[ctx, n]);
    }
    fn fun_0215a690(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A690, &[ctx])
    }
    fn fun_0215aa24(&mut self, ctx: u32) {
        self.s(0x0215_AA24, &[ctx]);
    }
    fn fun_0215aa34(&mut self, ctx: u32) {
        self.s(0x0215_AA34, &[ctx]);
    }
    fn fun_0215aa48(&mut self, ctx: u32, slot: u32) {
        self.s(0x0215_AA48, &[ctx, slot]);
    }
    fn fun_0215aa4c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_AA4C, &[ctx])
    }
    fn fun_020120f4(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F4, &[gamesys])
    }
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32 {
        self.s(0x0201_22C0, &[gamesys, parent, f, size])
    }
    fn fun_020124e8(&mut self, event: u32) -> u32 {
        self.s(0x0201_24E8, &[event])
    }
    fn fun_0201296c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_296C, &[gamedata])
    }
    fn fun_02012a30(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2A30, &[gamedata])
    }
    fn fun_02012b38(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2B38, &[gamedata])
    }
    fn fun_020142e8(&mut self, ew: u32, flag: u32) -> u32 {
        self.s(0x0201_42E8, &[ew, flag])
    }
    fn fun_02014314(&mut self, ew: u32, flag: u32) {
        self.s(0x0201_4314, &[ew, flag]);
    }
    fn fun_0201433c(&mut self, ew: u32, flag: u32) {
        self.s(0x0201_433C, &[ew, flag]);
    }
    fn fun_02014460(&mut self, x: u32) -> u32 {
        self.s(0x0201_4460, &[x])
    }
    fn fun_02014474(&mut self, x: u32, v: u32) {
        self.s(0x0201_4474, &[x, v]);
    }
    fn fun_020288c4(&mut self, x: u32) -> u32 {
        self.s(0x0202_88C4, &[x])
    }
    fn fun_0202972c(&mut self, gamesys: u32, a: u32) -> u32 {
        self.s(0x0202_972C, &[gamesys, a])
    }
    fn fun_021b1bb8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x021B_1BB8, &[vm, ctx])
    }
    fn fun_021b1be8(&mut self, ctx: u32) {
        self.s(0x021B_1BE8, &[ctx]);
    }
}

#[allow(clippy::too_many_arguments)]
impl ProcHooks for Rom<'_> {
    fn fun_02158f50(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F50, &[sw])
    }
    fn fun_02158fb4(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8FB4, &[sw])
    }
    fn fun_02158fb8(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8FB8, &[sw])
    }
    fn fun_02158fbc(&mut self, sw: u32) {
        self.s(0x0215_8FBC, &[sw]);
    }
    fn call_proc_callback(&mut self, cb: u32, w: u32) {
        self.s(cb & !1, &[w]);
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, from_end, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_0216f940(&mut self, block: u32) {
        self.s(0x0216_F940, &[block]);
    }
    fn fun_020120b4(&mut self, gamesys: u32, ovl: u32, proc_table: u32, param: u32) {
        self.s(0x0201_20B4, &[gamesys, ovl, proc_table, param]);
    }
    fn fun_020120f0(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F0, &[gamesys])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02012138(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_2138, &[gamesys])
    }
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8]) -> u32 {
        self.stubs.call(
            0x0201_24AC,
            &mut [
                Arg::V(gamesys),
                Arg::V(kind),
                Arg::V(proc_table),
                Arg::In(rec),
            ],
        )
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2934, &[gamedata])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2944, &[gamedata])
    }
    fn fun_02012b58(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2B58, &[gamedata])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2EBC, &[gamedata])
    }
    fn fun_0200be48(&mut self, save: u32) {
        self.s(0x0200_BE48, &[save]);
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.s(0x0201_AB28, &[party, i])
    }
    fn fun_0202e474(&mut self, gamedata: u32, a: u32, m: u32, heap: u32) -> u32 {
        self.s(0x0202_E474, &[gamedata, a, m, heap])
    }
    fn fun_02014808(
        &mut self,
        gamesys: u32,
        fm: u32,
        ovl: u32,
        proc_table: u32,
        param: u32,
        cb: u32,
        cbwork: u32,
    ) -> u32 {
        self.s(
            0x0201_4808,
            &[gamesys, fm, ovl, proc_table, param, cb, cbwork],
        )
    }
    fn fun_02014840(
        &mut self,
        gamesys: u32,
        fm: u32,
        ovl: u32,
        proc_table: u32,
        param: u32,
        cb: u32,
        cbwork: u32,
    ) -> u32 {
        self.s(
            0x0201_4840,
            &[gamesys, fm, ovl, proc_table, param, cb, cbwork],
        )
    }
    fn fun_02034ac4(&mut self, ovl: u32) {
        self.s(0x0203_4AC4, &[ovl]);
    }
    fn fun_02034a5c(&mut self, ovl: u32) {
        self.s(0x0203_4A5C, &[ovl]);
    }
    fn fun_02158344(&mut self, x: u32) -> u32 {
        self.s(0x0215_8344, &[x])
    }
    fn fun_0215840c(&mut self, x: u32, i: u32) -> u32 {
        self.s(0x0215_840C, &[x, i])
    }
    fn fun_0215841c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0215_841C, &[gamedata])
    }
    fn fun_02161b90(&mut self, gamesys: u32, event: u32, a: u32, b: u32, c: u32) -> u32 {
        self.s(0x0216_1B90, &[gamesys, event, a, b, c])
    }
    fn fun_02165344(&mut self) {
        self.s(0x0216_5344, &[]);
    }
    fn fun_0216f918(&mut self, heap: u32) -> u32 {
        self.s(0x0216_F918, &[heap])
    }
    fn fun_0216f948(&mut self, mon: u32, heap: u32) -> u32 {
        self.s(0x0216_F948, &[mon, heap])
    }
    fn fun_02175648(&mut self, gamedata: u32, out: u32) {
        self.s_out::<16>(0x0217_5648, &[gamedata], out);
    }
    fn fun_02175fb4(&mut self, x: u32) -> u32 {
        self.s(0x0217_5FB4, &[x])
    }
    fn fun_021f36c0(&mut self, gamesys: u32, a: u32, b: u32) {
        self.s(0x021F_36C0, &[gamesys, a, b]);
    }
    fn fun_021f36e8(&mut self, gamesys: u32, v: u32) {
        self.s(0x021F_36E8, &[gamesys, v]);
    }
    fn fun_021f3724(&mut self, gamesys: u32, v: u32) {
        self.s(0x021F_3724, &[gamesys, v]);
    }
    fn fun_021f3768(&mut self, gamesys: u32, vars: &[u8; 32]) {
        self.stubs
            .call(0x021F_3768, &mut [Arg::V(gamesys), Arg::In(vars)]);
    }
    fn fun_021f3800(&mut self, gamesys: u32) {
        self.s(0x021F_3800, &[gamesys]);
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
        0x0215_C63C => {
            proc_call_wait(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x0215_C69C => proc_wait(h, v[0], v[1]) as u64,
        0x0215_C700 => {
            cb_334(h, v[0]);
            0
        }
        0x0215_C7C4 => {
            cb_336(h, v[0]);
            0
        }
        0x0215_C870 => {
            cb_489(h, v[0]);
            0
        }
        // command 356 enters with r4 = r6 = 0 (the Z19 entry state)
        0x0215_CC4C => cmd_356(h, v[0], v[1], 0, 0) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_proc`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_proc",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_proc.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// Overlay 10 of the ROM build (`$BLACK_ROM_BUILD/OVY_10.sbin`, loaded at 0x02155160), or `None` (skip) when it is
/// missing and `REQUIRE_ROM` is not set.
fn overlay10() -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_10.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_10.sbin is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), and every handler is
/// ported; the code addresses the commands store (wait callback, result callbacks) are the ROM's literals.
#[test]
fn commands_match_rom() {
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    let Some(ovy) = overlay10() else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0215_5160) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    for &(id, addr) in COMMANDS {
        assert_eq!(
            word(crate::script_sys::CMD_TABLE + 4 * id as u32),
            addr | 1,
            "command {id}"
        );
    }
    // literal pools of 0215c63c / 0215c72c / 0215c7e0 / 0215c89c
    assert_eq!(word(0x0215_C698), PROC_WAIT);
    assert_eq!(word(0x0215_C7BC), CB_334);
    assert_eq!(word(0x0215_C864), CB_336);
    assert_eq!(word(0x0215_C950), CB_489);
    assert_eq!(word(0x0215_C694), FILE);
}
