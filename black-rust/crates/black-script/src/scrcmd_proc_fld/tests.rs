use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B4BEC, "2e04b61ac872d2c4"),
    (0x021B4C14, "129dd6c7680e838c"),
    (0x021B4C34, "56b4976f50297ba1"),
    (0x021B4C54, "02a6f32562e6e6fc"),
    (0x021B4C98, "8b74a878dab85a2f"),
    (0x021B4CDC, "9fc87b288fed4f38"),
    (0x021B4D14, "98492bc4ca85ce27"),
    (0x021B4D60, "3696508005379b5c"),
    (0x021B4E58, "5aa2f09886ab1fd8"),
    (0x021B4E9C, "a0e48204d70d98aa"),
    (0x021B4EE0, "119c08c6ce2a51bf"),
    (0x021B4F1C, "f30db220b600c350"),
    (0x021B4F68, "019b2bdeadfb17ec"),
    (0x021B4FB0, "4500f0e7724b7d16"),
    (0x021B5008, "512a864798025cc5"),
    (0x021B503C, "c5ce23a1c83cbfe2"),
    (0x021B5094, "c5bf5e6158c45769"),
    (0x021B50D0, "9afe6ce6bdd0968d"),
    (0x021B50F8, "6de1fece90ad5261"),
    (0x021B5128, "776194a8c4fdd1f4"),
    (0x021B5154, "ab1312e245b3e87c"),
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
impl ProcFldHooks for Rom<'_> {
    fn fun_0215898c(&mut self, sw: u32, ev: u32) {
        self.stubs.call(0x0215898c, &mut [Arg::V(sw), Arg::V(ev)]);
    }
    fn fun_02158f54(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f54, &mut [Arg::V(sw)])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f58, &mut [Arg::V(sw)])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_02012138(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x02012138, &mut [Arg::V(gamesys)])
    }
    fn fun_021896f4(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021896f4, &mut [Arg::V(x)])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)])
    }
    fn fun_0201283c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201283c, &mut [Arg::V(gamedata)])
    }
    fn fun_02009ce4(&mut self, x: u32) -> u32 {
        self.stubs.call(0x02009ce4, &mut [Arg::V(x)])
    }
    fn fun_02009a34(&mut self, x: u32) -> u32 {
        self.stubs.call(0x02009a34, &mut [Arg::V(x)])
    }
    fn fun_0201293c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201293c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012934, &mut [Arg::V(gamedata)])
    }
    fn fun_0201292c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201292c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012944, &mut [Arg::V(gamedata)])
    }
    fn fun_0200873c(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200873c, &mut [Arg::V(save)])
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0200ca1c, &mut [Arg::V(gamedata)])
    }
    fn fun_0200ca38(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200ca38, &mut [Arg::V(x)])
    }
    fn fun_0200bac8(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200bac8, &mut [Arg::V(save)])
    }
    fn fun_0200baf4(&mut self, block: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x0200baf4, &mut [Arg::V(block), Arg::V(heap)])
    }
    fn fun_0200bbfc(&mut self, block: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x0200bbfc, &mut [Arg::V(block), Arg::V(heap)])
    }
    fn fun_0200bb58(&mut self, block: u32, data: u32) {
        self.stubs
            .call(0x0200bb58, &mut [Arg::V(block), Arg::V(data)]);
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32 {
        self.stubs.call(
            0x02030734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(from_end),
                Arg::V(file),
                Arg::V(line),
            ],
        )
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.stubs.call(0x020307b0, &mut [Arg::V(block)]);
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
        self.stubs.call(
            0x02014840,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(ovl),
                Arg::V(proc_table),
                Arg::V(param),
                Arg::V(cb),
                Arg::V(cbwork),
            ],
        )
    }
    fn fun_020146f8(
        &mut self,
        gamesys: u32,
        fm: u32,
        ovl: u32,
        proc_table: u32,
        param: u32,
    ) -> u32 {
        self.stubs.call(
            0x020146f8,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(ovl),
                Arg::V(proc_table),
                Arg::V(param),
            ],
        )
    }
    fn fun_021cd2a4(&mut self, gamesys: u32, fm: u32, a: u32, mode: u32, var: u32) -> u32 {
        self.stubs.call(
            0x021cd2a4,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(a),
                Arg::V(mode),
                Arg::V(var),
            ],
        )
    }
    fn fun_0215f8a8(&mut self, gamesys: u32, v: u32) -> u32 {
        self.stubs
            .call(0x0215f8a8, &mut [Arg::V(gamesys), Arg::V(v)])
    }
    fn fun_02009210(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02009210, &mut [Arg::V(save)])
    }
    fn fun_020090d8(&mut self, x: u32, on: u32) {
        self.stubs.call(0x020090d8, &mut [Arg::V(x), Arg::V(on)]);
    }
    fn fun_02168bbc(&mut self, gamesys: u32, v: u32) -> u32 {
        self.stubs
            .call(0x02168bbc, &mut [Arg::V(gamesys), Arg::V(v)])
    }
    fn fun_021d30b8(&mut self, gamesys: u32, var: u32) -> u32 {
        self.stubs
            .call(0x021d30b8, &mut [Arg::V(gamesys), Arg::V(var)])
    }
    fn fun_02188ce8(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188ce8, &mut [Arg::V(fm)])
    }
    fn fun_021aa0a0(&mut self, x: u32, a: u32, b: u32, c: u32) -> u32 {
        self.stubs.call(
            0x021aa0a0,
            &mut [Arg::V(x), Arg::V(a), Arg::V(b), Arg::V(c)],
        )
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
        self.stubs.call(
            0x021be92c,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
            ],
        )
    }
    fn fun_02012a44(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012a44, &mut [Arg::V(gamedata)])
    }
    fn fun_0216efcc(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0216efcc, &mut [Arg::V(x)])
    }
    fn fun_0216e95c(&mut self, gamesys: u32, fm: u32, n: u32) -> u32 {
        self.stubs
            .call(0x0216e95c, &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(n)])
    }
    fn fun_021aab48(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021aab48, &mut [Arg::V(x)])
    }
    fn fun_0200d418(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200d418, &mut [Arg::V(save)])
    }
    fn fun_0200d488(&mut self, x: u32, i: u32) -> u32 {
        self.stubs.call(0x0200d488, &mut [Arg::V(x), Arg::V(i)])
    }
    fn fun_0215e440(&mut self, gamedata: u32, i: u32) {
        self.stubs
            .call(0x0215e440, &mut [Arg::V(gamedata), Arg::V(i)]);
    }
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8; 8]) -> u32 {
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
    fn fun_020124ac_at(&mut self, gamesys: u32, kind: u32, proc_table: u32, param: u32) -> u32 {
        self.s(0x0201_24AC, &[gamesys, kind, proc_table, param])
    }
    // the spec stub of 021c5bd0 takes r0 only (`args=v`)
    fn fun_021c5bd0(&mut self, gamesys: u32, _flag: u32, _var: u32) -> u32 {
        self.s(0x021C_5BD0, &[gamesys])
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
        0x021B_4D14 => {
            cb_335(h, v[0]);
            0
        }
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The procs, apps, callback and file string are the ROM's literals.
#[test]
fn literals_match_rom() {
    let Some(ovy) = overlay21() else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0218_7F00) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    assert_eq!((word(0x021B_4C90), word(0x021B_4C94)), (KIND_353, PROC_353));
    assert_eq!((word(0x021B_4CD4), word(0x021B_4CD8)), (KIND_354, PROC_354));
    assert_eq!((word(0x021B_4D0C), word(0x021B_4D10)), (KIND_358, PROC_358));
    assert_eq!(word(0x021B_4E48), FILE);
    assert_eq!(word(0x021B_4E4C), CB_335);
    assert_eq!((word(0x021B_4E50), word(0x021B_4E54)), APP_335);
    assert_eq!((word(0x021B_4FA8), word(0x021B_4FAC)), APP_338);
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_proc_fld`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_proc_fld",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_proc_fld.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// Overlay 10 of the ROM build (`$BLACK_ROM_BUILD/OVY_10.sbin`, loaded at 0x02155160), or `None` (skip) when it is
/// missing and `REQUIRE_ROM` is not set.
fn overlay10() -> Option<Vec<u8>> {
    rom_file("OVY_10.sbin")
}

/// Overlay 21 of the ROM build (`$BLACK_ROM_BUILD/OVY_21.sbin`, loaded at 0x02187F00), or `None` (skip).
#[allow(dead_code)]
fn overlay21() -> Option<Vec<u8>> {
    rom_file("OVY_21.sbin")
}

fn rom_file(name: &str) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/{name}")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/{name} is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), the ids are unique and
/// every handler is ported and dispatched.
#[test]
fn commands_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0, "ids not sorted / unique at {}", w[1].0);
    }
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    let Some(ovy) = overlay10() else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}
