use super::*;
use crate::hooks::{CommandDispatch, HeapHooks, Mem, MemEnv, ScriptSysHooks, VmHooks};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02158804, "13ae38f70b1bc551"),
    (0x0215882C, "193c0096a4478823"),
    (0x0215886C, "8ad7a9f160e98841"),
    (0x02158894, "5d602ab894c36c61"),
    (0x021588A4, "f66a5bc196af144b"),
    (0x021588B0, "1d9fb6062996bd76"),
    (0x021588C0, "de7eb73650ba21ac"),
    (0x021588D0, "228b54fefbd35f19"),
    (0x021588DC, "c6307786ad66b83f"),
    (0x02158920, "26e65fdb5fcbf303"),
    (0x0215897C, "a2f328cafbd35461"),
    (0x0215898C, "c85aa099c67675d3"),
    (0x021589A0, "0a03cbafdb4d19e6"),
    (0x021589D0, "831c2f4aebbb650e"),
    (0x021589DC, "47e1b1651bbbfc62"),
    (0x021589F4, "6749ccc0d32fde7f"),
    (0x02158A10, "f0546b7252f63364"),
    (0x02158A20, "7c1a304f93ae9c31"),
    (0x02158A34, "c8869635856310b1"),
    (0x02158A90, "a65ef52007182edd"),
    (0x02158AC8, "203989b6077fdda2"),
    (0x02158AD4, "a594c087b4f5d6d5"),
    (0x02158AE0, "c844815d4220aa18"),
    (0x02158B00, "e1afb71832852b77"),
    (0x02158B3C, "77fe191fdd7cbcf6"),
    (0x02158B6C, "28d10b68f1651aae"),
    (0x02158BFC, "89aa814e57d84e18"),
    (0x02158CB4, "cd41177c81212bea"),
    (0x02158CD0, "26ba28bf5b5e5b63"),
    (0x02158D78, "cb315757c3c5a148"),
    (0x02158D90, "02e009169e4be232"),
    (0x02158DB0, "6e4b95376563e136"),
    (0x02158DBC, "8b9dcd19797f4912"),
    (0x02158DE8, "b5c5d3a200fb75cf"),
    (0x02158E38, "9ed7dd3a77824ccf"),
    (0x02158E54, "b62c651a6951827a"),
    (0x02158E7C, "41f4c88bb5246e04"),
    (0x02158F18, "c6a54ffc5fabef18"),
    (0x02158F50, "1adffe77cd7a5780"),
    (0x02158F54, "7d4d12e69604047f"),
    (0x02158F58, "5bb189e2b1dec54b"),
    (0x02158F6C, "a0eb4d7578dfd94f"),
    (0x02158F74, "a215ed7520e4ecbe"),
    (0x02158F78, "7ae61a290306c1c5"),
    (0x02158F7C, "06e32c44ed131308"),
    (0x02158F80, "ebfed7729293cae2"),
    (0x02158F84, "b42ecf4233ea3e0c"),
    (0x02158F88, "1ffac243396b6cf0"),
    (0x02158F8C, "77a236a78b836fdc"),
    (0x02158F90, "b01c4b4297e40d79"),
    (0x02158F94, "b8271f65fec2f7f5"),
    (0x02158FB4, "c38c55bdd0d14eb4"),
    (0x02158FB8, "fb0c0fd33e096514"),
    (0x02158FBC, "d43be10d2df1e6ae"),
    (0x02158FD0, "56ea18b799bbab48"),
    (0x02158FE0, "772cd73ba11435e5"),
    (0x0215901C, "57e6dda369a318e5"),
    (0x02159030, "4f7b7885488494e6"),
    (0x02159058, "181a5d3c3eba22bb"),
    (0x0215909C, "c445c274dae1788b"),
    (0x021590D4, "2229cbc30ff8201b"),
    (0x021590F4, "7ac783f9ac7da983"),
    (0x021590FC, "1c1e5d78dc5eee02"),
    (0x02159104, "3174191918f47daa"),
    (0x0215910C, "c316f16c9a8f7c94"),
    (0x02159114, "a9383e3781068d4d"),
    (0x0215911C, "a90db842da26a784"),
    (0x02159124, "a56051b033a73e59"),
    (0x02159130, "ffa673b434806e05"),
    (0x02159144, "83b18ba06d9a20d8"),
    (0x02159168, "25e265ac4f16aa4e"),
    (0x02159198, "4bc8cd0339fb523d"),
    (0x021591B4, "e09a2fb1d6363f0f"),
    (0x02159960, "132013adeae7638c"),
    (0x02159974, "0d546fb524e56251"),
    (0x02159984, "ded36b515443b40d"),
    (0x02159998, "43591bf4a4430767"),
    (0x021599A8, "efa4ef6bbfff9882"),
    (0x02159A64, "7e022877925b1318"),
    (0x02159AE8, "00b7df75567caeed"),
    (0x02159B10, "f9cecdc908d8133c"),
    (0x02159B38, "e9913fe91d9d66f9"),
    (0x0215A5CC, "d0f84553748d4ebd"),
    (0x0215A5D0, "e614734b8df927cc"),
    (0x0215A5D4, "b2d1a06ae5d74d12"),
    (0x0215A5D8, "acdc7ec4e1190a2c"),
    (0x0215A5DC, "dc3e97bd87e6acb4"),
    (0x0215A5E4, "48637254c191b563"),
    (0x0215A5EC, "01e61f0401a1f126"),
    (0x0215A5F4, "3a411a5ea0e7c6dc"),
];
// fndiff-recorded-end

/// The spec's stubs of the unported callees, with every buffer and global of the case in one memory window set (at the
/// addresses the emulator uses).
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

impl HeapHooks for Rom<'_> {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn heap_free(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
}

// The VM's own callbacks are not reached from this area (vm.c is stubbed here).
impl CommandDispatch for Rom<'_> {
    fn command(&mut self, _handler: u32, _vm: u32, _ctx: u32) -> u32 {
        unreachable!("vm.c is stubbed in the script_sys spec")
    }
}

impl VmHooks for Rom<'_> {
    fn call_wait(&mut self, _f: u32, _vm: u32, _ctx: u32) -> u32 {
        unreachable!("vm.c is stubbed in the script_sys spec")
    }
    fn call_hook(&mut self, _f: u32, _vm: u32, _ctx: u32, _arg: u32, _cmd: u32) -> u32 {
        unreachable!("vm.c is stubbed in the script_sys spec")
    }
}

impl ScriptSysHooks for Rom<'_> {
    fn fun_0201121c(&mut self, heap: u32, init: &[u8; 12]) -> u32 {
        self.stubs
            .call(0x0201_121C, &mut [Arg::V(heap), Arg::In(init)])
    }
    fn fun_0201126c(&mut self, vm: u32, ctx: u32) {
        self.s(0x0201_126C, &[vm, ctx]);
    }
    fn fun_0201127c(&mut self, vm: u32, pc: u32) {
        self.s(0x0201_127C, &[vm, pc]);
    }
    fn fun_02011298(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1298, &[vm])
    }
    fn fun_02011324(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1324, &[vm])
    }
    fn fun_02011328(&mut self, vm: u32, hook: u32, arg: u32) {
        self.s(0x0201_1328, &[vm, hook, arg]);
    }
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1330, &[vm])
    }
    fn fun_0201134c(&mut self, vm: u32) -> u32 {
        self.s(0x0201_134C, &[vm])
    }
    fn fun_020113d0(&mut self, vm: u32, f: u32) {
        self.s(0x0201_13D0, &[vm, f]);
    }
    fn call_end_callback(&mut self, f: u32, x: u32, arg: u32) -> u32 {
        self.s(f & !1, &[x, arg])
    }
    fn call_init_proc(&mut self, f: u32, task: u32, seq: u32) -> u32 {
        self.s(f & !1, &[task, seq])
    }
    fn fun_020120f4(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F4, &[gamesys])
    }
    fn fun_020120f8(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F8, &[gamesys])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32 {
        self.s(0x0201_22C0, &[gamesys, parent, f, size])
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        self.s(0x0201_2374, &[event, child]);
    }
    fn fun_020124e8(&mut self, event: u32) -> u32 {
        self.s(0x0201_24E8, &[event])
    }
    fn fun_020127a4(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_27A4, &[gamedata])
    }
    fn fun_02012844(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2844, &[gamedata])
    }
    fn fun_0201296c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_296C, &[gamedata])
    }
    fn fun_02012ac8(&mut self, x: u32) -> u32 {
        self.s(0x0201_2AC8, &[x])
    }
    fn fun_02012dac(&mut self, gamedata: u32) {
        self.s(0x0201_2DAC, &[gamedata]);
    }
    fn fun_02012dd0(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2DD0, &[gamedata])
    }
    fn fun_02012e40(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2E40, &[gamedata])
    }
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F2C, &[gamedata])
    }
    fn fun_02008df0(&mut self, x: u32, v: u32) {
        self.s(0x0200_8DF0, &[x, v]);
    }
    fn fun_02013b5c(&mut self, zone: u32) -> u32 {
        self.s(0x0201_3B5C, &[zone])
    }
    fn fun_02013b68(&mut self, zone: u32) -> u32 {
        self.s(0x0201_3B68, &[zone])
    }
    fn fun_02014368(&mut self, ew: u32, first: u32, last: u32) {
        self.s(0x0201_4368, &[ew, first, last]);
    }
    fn fun_020143cc(&mut self, ew: u32, var: u32) -> u32 {
        self.s(0x0201_43CC, &[ew, var])
    }
    fn fun_020143d8(&mut self, ew: u32, first: u32, end: u32) {
        self.s(0x0201_43D8, &[ew, first, end]);
    }
    fn fun_0201ec74(&mut self, n: u32, len: u32, heap: u32) -> u32 {
        self.s(0x0201_EC74, &[n, len, heap])
    }
    fn fun_0201ed04(&mut self, wordset: u32) {
        self.s(0x0201_ED04, &[wordset]);
    }
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32 {
        self.s(0x0204_57B0, &[len, heap])
    }
    fn fun_02045808(&mut self, strbuf: u32) {
        self.s(0x0204_5808, &[strbuf]);
    }
    fn fun_0204a1bc(&mut self, arc: u32, file: u32, flag: u32, heap: u32) -> u32 {
        self.s(0x0204_A1BC, &[arc, file, flag, heap])
    }
    fn fun_02034714(&mut self, a: u32, b: u32, c: u32) {
        self.s(0x0203_4714, &[a, b, c]);
    }
    fn fun_020362dc(&mut self) -> u32 {
        self.s(0x0203_62DC, &[])
    }
    fn fun_0203cd98(&mut self) -> u32 {
        self.s(0x0203_CD98, &[])
    }
    fn fun_0203cf04(&mut self) {
        self.s(0x0203_CF04, &[]);
    }
    fn fun_0203ceac(&mut self, v: u32) {
        self.s(0x0203_CEAC, &[v]);
    }
    fn fun_0203d0d0(&mut self) {
        self.s(0x0203_D0D0, &[]);
    }
    fn fun_0203cf5c(&mut self) {
        self.s(0x0203_CF5C, &[]);
    }
    fn fun_0203cdbc(&mut self) -> u32 {
        self.s(0x0203_CDBC, &[])
    }
    fn fun_020053ec(&mut self) {
        self.s(0x0200_53EC, &[]);
    }
    fn fun_02158768(&mut self, gamesys: u32, id: u32, a: u32, x: u32) -> u32 {
        self.s(0x0215_8768, &[gamesys, id, a, x])
    }
    fn fun_0215a514(&mut self, sw: u32, heap: u32) -> u32 {
        self.s(0x0215_A514, &[sw, heap])
    }
    fn fun_0215a55c(&mut self, init: &[u8; 16], heap: u32) -> u32 {
        self.stubs
            .call(0x0215_A55C, &mut [Arg::In(init), Arg::V(heap)])
    }
    fn fun_0215a5a4(&mut self, ctx: u32) {
        self.s(0x0215_A5A4, &[ctx]);
    }
    fn fun_0215a760(&mut self, ctx: u32, ty: u32, msg: u32) {
        self.s(0x0215_A760, &[ctx, ty, msg]);
    }
    fn fun_02162450(&mut self, x: u32) -> u32 {
        self.s(0x0216_2450, &[x])
    }
    fn fun_0216d588(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D588, &[mmdl])
    }
    fn fun_02188c54(&mut self, fieldmap: u32) -> u32 {
        self.s(0x0218_8C54, &[fieldmap])
    }
    fn fun_02188c84(&mut self, fieldmap: u32) -> u32 {
        self.s(0x0218_8C84, &[fieldmap])
    }
    fn fun_02188ccc(&mut self, fieldmap: u32) -> u32 {
        self.s(0x0218_8CCC, &[fieldmap])
    }
    fn fun_021a0f90(&mut self, x: u32) -> u32 {
        self.s(0x021A_0F90, &[x])
    }
    fn fun_021a1524(&mut self, x: u32) {
        self.s(0x021A_1524, &[x]);
    }
    fn fun_021a1530(&mut self, x: u32) {
        self.s(0x021A_1530, &[x]);
    }
    fn fun_021a153c(&mut self, x: u32) -> u32 {
        self.s(0x021A_153C, &[x])
    }
    fn fun_021a1548(&mut self, x: u32) {
        self.s(0x021A_1548, &[x]);
    }
    fn fun_021d1164(&mut self, a: u32, fieldmap: u32) -> u32 {
        self.s(0x021D_1164, &[a, fieldmap])
    }
    fn fun_021d1194(&mut self, x: u32) {
        self.s(0x021D_1194, &[x]);
    }
    fn fun_021d11b0(&mut self, x: u32) {
        self.s(0x021D_11B0, &[x]);
    }
    fn fun_021d11d8(&mut self, x: u32) {
        self.s(0x021D_11D8, &[x]);
    }
}

/// Run `f` on the case's memory: every non-empty buffer at its argument address, the globals at `globals`.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push((0, i));
        }
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push((1, k));
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    // a `noreturn` stub unwinds: put the memory back before passing the unwind on
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut rom, &vals)));
    let wins = std::mem::take(&mut rom.mem.extra);
    for ((kind, i), (_, bytes)) in keep.into_iter().zip(wins) {
        match kind {
            0 => c.bufs[i] = bytes,
            _ => c.globals[i] = bytes,
        }
    }
    match r {
        Ok(v) => v,
        Err(e) => std::panic::resume_unwind(e),
    }
}

/// `ret=ofs:argK`: 0 stays 0, else the offset from the buffer's address.
fn ofs(addr: u32, base: u32) -> u64 {
    if addr == 0 {
        0
    } else {
        addr.wrapping_sub(base) as u64
    }
}

/// The function's `global` lines (only the init-proc mask in this area).
fn globals_of(addr: u32) -> &'static [u32] {
    match addr {
        0x0215_8920 | 0x0215_8A34 | 0x0215_8A10 | 0x0215_8A20 | 0x0215_8A90 | 0x0215_8AC8
        | 0x0215_8AD4 | 0x0215_8AE0 | 0x0215_8E7C | 0x0215_9130 | 0x0215_9144 | 0x0215_9168
        | 0x0215_9198 | 0x0215_91B4 => &[PROC_MASK_ADDR],
        _ => &[],
    }
}

fn run(addr: u32, c: &mut Case) -> u64 {
    let g = globals_of(addr);
    if addr == 0x0215_8CD0 {
        // the three out pointers are 4-byte buffers (u16 at offset 0)
        return with_rom(c, g, |h, v| {
            let (mut f, mut t, mut m) = (0u16, 0u16, 0u16);
            let r = script_lookup(h, v[0], v[1], &mut f, &mut t, &mut m);
            h.w16(v[2], f);
            h.w16(v[3], t);
            h.w16(v[4], m);
            r as u64
        });
    }
    with_rom(c, g, |h, v| match addr {
        0x0215_8804 => ofs(sys_new(h, v[0]), v[1]),
        0x0215_882C => sys_step(h, v[0]) as u64,
        0x0215_886C => sys_add_vm(h, v[0], v[1]) as u64,
        0x0215_8894 => sys_vm(h, v[0], v[1]) as u64,
        0x0215_88A4 => {
            sys_set_end_callback(h, v[0], v[1]);
            0
        }
        0x0215_88B0 => sys_run_end_callback(h, v[0], v[1]) as u64,
        0x0215_88C0 => sys_has_end_callback(h, v[0]) as u64,
        0x0215_88D0 => zone_of(h, v[0]) as u64,
        0x0215_88DC => event_fn(h, v[0], v[1], v[2]) as u64,
        0x0215_8920 => start_event(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0215_897C => {
            sw_call_child(h, v[0], v[1]);
            0
        }
        0x0215_898C => {
            sw_set_end_callback(h, v[0], v[1]);
            0
        }
        0x0215_89A0 => sw_start_script(h, v[0], v[1], v[2]) as u64,
        0x0215_89D0 => event_sw(h, v[0]) as u64,
        0x0215_89DC => sw_sys(h, v[0]) as u64,
        0x0215_89F4 => {
            clear_local_flags(h, v[0]);
            0
        }
        0x0215_8A10 => {
            run_sync_9600(h, v[0]);
            0
        }
        0x0215_8A20 => {
            run_sync_9601(h, v[0]);
            0
        }
        0x0215_8A34 => {
            run_sync(h, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_8A90 => run_special(h, v[0], v[1], v[2], v[3]) as u64,
        0x0215_8AC8 => {
            run_special_4(h, v[0], v[1]);
            0
        }
        0x0215_8AD4 => {
            run_special_3(h, v[0], v[1]);
            0
        }
        0x0215_8AE0 => {
            run_special_enter(h, v[0], v[1]);
            0
        }
        0x0215_8B00 => run_conditional(h, v[0], v[1]) as u64,
        0x0215_8B3C => find_special(h, v[0], v[1]) as u64,
        0x0215_8B6C => find_conditional(h, v[0], v[1], v[2]) as u64,
        0x0215_8BFC => ofs(vm_new(h, v[0], v[1], v[2], v[3], v[4]), v[1]),
        0x0215_8CB4 => {
            vm_free(h, v[0]);
            0
        }
        0x0215_8D78 => load_script_file(h, v[0], v[1]) as u64,
        0x0215_8D90 => is_low_script_id(v[0]) as u64,
        0x0215_8DB0 => nonzero(v[0]) as u64,
        0x0215_8DBC => fatal(h),
        0x0215_8DE8 => dispatch_hook(h, v[0], v[1], v[2], v[3]) as u64,
        0x0215_8E38 => {
            install_hook(h, v[0]);
            0
        }
        0x0215_8E54 => {
            field_block_refresh(h, v[0], v[1]);
            0
        }
        0x0215_8E7C => ofs(sw_new(h, v[0], v[1], v[2], v[3], v[4], v[5]), v[6]),
        0x0215_8F18 => {
            sw_free(h, v[0]);
            0
        }
        0x0215_8F50 => sw_event(h, v[0]) as u64,
        0x0215_8F54 => sw_gamesys(h, v[0]) as u64,
        0x0215_8F58 => ofs(sw_field_block(h, v[0]), v[0]),
        0x0215_8F6C => sw_scrcmd_work(h, v[0]) as u64,
        0x0215_8F74 => sw_wordset(h, v[0]) as u64,
        0x0215_8F78 => sw_strbuf1(h, v[0]) as u64,
        0x0215_8F7C => sw_strbuf2(h, v[0]) as u64,
        0x0215_8F80 => {
            sw_set_38(h, v[0], v[1]);
            0
        }
        0x0215_8F84 => sw_get_38(h, v[0]) as u64,
        0x0215_8F88 => ofs(sw_addr_40(v[0]), v[0]),
        0x0215_8F8C => sw_script_id(h, v[0]) as u64,
        0x0215_8F90 => sw_mmdl(h, v[0]) as u64,
        0x0215_8F94 => {
            sw_set_mmdl(h, v[0], v[1]);
            0
        }
        0x0215_8FB4 => ofs(sw_addr_3c(v[0]), v[0]),
        0x0215_8FB8 => sw_get_3c(h, v[0]) as u64,
        0x0215_8FBC => {
            sw_free_3c(h, v[0]);
            0
        }
        0x0215_8FD0 => sw_var_addr(v[0], v[1]) as u64,
        0x0215_8FE0 => ofs(var_addr(h, v[0], v[1], v[2]), v[0]),
        0x0215_901C => var_value(h, v[0], v[1], v[2]) as u64,
        0x0215_9030 => set_var(h, v[0], v[1], v[2]) as u64,
        0x0215_9058 => {
            set_vars_8000(h, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x0215_909C => ofs(save_vars(h, v[0]), v[0]),
        0x0215_90D4 => {
            restore_vars(h, v[0], v[1]);
            0
        }
        0x0215_90F4 => {
            sw_set_13c(h, v[0], v[1]);
            0
        }
        0x0215_90FC => sw_get_13c(h, v[0]) as u64,
        0x0215_9104 => {
            sw_set_140(h, v[0], v[1]);
            0
        }
        0x0215_910C => sw_get_140(h, v[0]) as u64,
        0x0215_9114 => {
            sw_set_144(h, v[0], v[1]);
            0
        }
        0x0215_911C => sw_get_144(h, v[0]) as u64,
        0x0215_9124 => sw_record_addr(v[0], v[1]) as u64,
        0x0215_9130 => {
            proc_mask_reset(h);
            0
        }
        0x0215_9144 => {
            proc_mask_set(h, v[0]);
            0
        }
        0x0215_9168 => {
            proc_mask_clear(h, v[0]);
            0
        }
        0x0215_9198 => proc_mask_test(h, v[0]) as u64,
        0x0215_91B4 => run_init_procs(h, v[0], v[1], v[2]) as u64,
        0x0215_9960 => wait_pad_a(h, v[0], v[1]) as u64,
        0x0215_9974 => cmd_wait_pad_a(h, v[0], v[1]) as u64,
        0x0215_9984 => wait_pad_b(h, v[0], v[1]) as u64,
        0x0215_9998 => cmd_wait_pad_b(h, v[0], v[1]) as u64,
        0x0215_99A8 => wait_311(h, v[0], v[1]) as u64,
        0x0215_9A64 => cmd_311(h, v[0], v[1]) as u64,
        0x0215_9AE8 => ofs(cmd_var_addr(h, v[0], v[1]), v[1]),
        0x0215_9B10 => cmd_var_value(h, v[0], v[1]) as u64,
        0x0215_9B38 => ctx_zone_id(h, v[0]) as u64,
        0x0215_A5CC => ctx_heap(h, v[0]) as u64,
        0x0215_A5D0 => ctx_zone(h, v[0]) as u64,
        0x0215_A5D4 => ctx_mode_nonzero(h, v[0]) as u64,
        0x0215_A5D8 => ctx_mode(h, v[0]) as u64,
        0x0215_A5DC => ctx_gamesys(h, v[0]) as u64,
        0x0215_A5E4 => ctx_gamedata(h, v[0]) as u64,
        0x0215_A5EC => ctx_work_3(h, v[0]) as u64,
        0x0215_A5F4 => ctx_script_work(h, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script script_sys`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "script_sys",
            include_str!("../../../../tools/rom-diff/specs/black-script/script_sys.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

#[test]
fn script_var_addresses() {
    assert_eq!(sw_var_addr(0x1000, 0x8000), 0x107C);
    assert_eq!(sw_var_addr(0x1000, 0x8011), 0x109E);
    assert_eq!(sw_var_addr(0x1000, 0x805F), 0x113A);
}

/// Plain memory holding the init-proc mask.
struct Plain(Mem);

impl MemEnv for Plain {
    fn mem(&mut self) -> &mut Mem {
        &mut self.0
    }
}

#[test]
fn proc_mask_bits() {
    let mut m = Plain(Mem::new(PROC_MASK_ADDR, vec![0; 4]));
    let mut test = |mask: u32, bit: u32| {
        m.w32(PROC_MASK_ADDR, mask);
        proc_mask_test(&mut m, bit)
    };
    assert_eq!(test(0b100, 2), 1);
    assert_eq!(test(0b100, 1), 0);
    assert_eq!(test(u32::MAX, 14), 0);
    // a negative bit shifts by its low byte: -1 -> asrs by 255 -> the sign bit
    assert_eq!(test(0x8000_0000, u32::MAX), 1);
    assert_eq!(test(0x7FFF_FFFF, u32::MAX), 0);
    m.w32(PROC_MASK_ADDR, 0);
    proc_mask_set(&mut m, 13);
    proc_mask_set(&mut m, 14);
    assert_eq!(m.r32(PROC_MASK_ADDR), 1 << 13);
    proc_mask_clear(&mut m, 13);
    assert_eq!(m.r32(PROC_MASK_ADDR), 0);
}

#[test]
fn script_ids_map_to_files() {
    // ranges are checked top-down; every range's lo is above the next one's hi
    for w in SCRIPT_RANGES.windows(2) {
        assert!(w[0].lo > w[1].hi);
    }
    assert_eq!(SCRIPT_RANGES[45].lo, 2000);
    assert_eq!(CMD_PERMISSIONS.len() as u32, CMD_COUNT);
    assert_eq!(INIT_PROCS[14], 0);
}
