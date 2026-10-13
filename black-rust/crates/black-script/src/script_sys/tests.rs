use super::*;
use crate::hooks::{Mem, MemEnv, ScriptSysHooks};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02158FD0, "56ea18b799bbab48"),
    (0x02158FE0, "772cd73ba11435e5"),
    (0x0215901C, "57e6dda369a318e5"),
    (0x02159198, "4bc8cd0339fb523d"),
    (0x02159AE8, "20d2b590969c0ea4"),
    (0x0215A5F4, "3a411a5ea0e7c6dc"),
];
// fndiff-recorded-end

/// The spec's stubs of the unported callees, with one argument buffer as the memory window.
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl ScriptSysHooks for Rom<'_> {
    fn fun_0201296c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201_296C, &mut [Arg::V(gamedata)])
    }
    fn fun_020143cc(&mut self, ew: u32, var: u32) -> u32 {
        self.stubs.call(0x0201_43CC, &mut [Arg::V(ew), Arg::V(var)])
    }
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        self.stubs.call(0x0201_1330, &mut [Arg::V(vm)])
    }
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215_A5E4, &mut [Arg::V(ctx)])
    }
}

/// Run `f` with argument buffer `k` as the memory window; the buffer is put back afterwards.
fn with_buf<R>(c: &mut Case, k: usize, f: impl FnOnce(&mut Rom) -> R) -> R {
    let mut rom = Rom {
        stubs: &mut c.env,
        mem: Mem::new(c.vals[k], std::mem::take(&mut c.bufs[k])),
    };
    let r = f(&mut rom);
    c.bufs[k] = rom.mem.bytes;
    r
}

/// `ret=ofs:argK`: 0 stays 0, else the offset from the buffer's address.
fn ofs(addr: u32, base: u32) -> u64 {
    if addr == 0 {
        0
    } else {
        addr.wrapping_sub(base) as u64
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = |i: usize| c.vals.get(i).copied().unwrap_or(0);
    let (v0, v1, v2) = (v(0), v(1), v(2));
    match addr {
        0x0215_8FD0 => sw_var_addr(v0, v1) as u64,
        0x0215_8FE0 => {
            let p = with_buf(c, 0, |h| var_addr(h, v0, v1, v2));
            ofs(p, v0)
        }
        0x0215_901C => with_buf(c, 0, |h| var_value(h, v0, v1, v2)) as u64,
        0x0215_9198 => {
            let mask = u32::from_le_bytes(c.globals[0][..4].try_into().unwrap());
            proc_mask_test(mask, v0) as u64
        }
        0x0215_9AE8 => {
            let p = with_buf(c, 1, |h| cmd_var_addr(h, v0, v1));
            ofs(p, v1)
        }
        0x0215_A5F4 => with_buf(c, 0, |h| ctx_script_work(h, v0)) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
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
        &call,
    );
}

#[test]
fn script_var_addresses() {
    assert_eq!(sw_var_addr(0x1000, 0x8000), 0x107C);
    assert_eq!(sw_var_addr(0x1000, 0x8011), 0x109E);
    assert_eq!(sw_var_addr(0x1000, 0x805F), 0x113A);
}

#[test]
fn proc_mask_bits() {
    assert_eq!(proc_mask_test(0b100, 2), 1);
    assert_eq!(proc_mask_test(0b100, 1), 0);
    assert_eq!(proc_mask_test(u32::MAX, 14), 0);
    // a negative bit shifts by its low byte: -1 -> asrs by 255 -> the sign bit
    assert_eq!(proc_mask_test(0x8000_0000, u32::MAX), 1);
    assert_eq!(proc_mask_test(0x7FFF_FFFF, u32::MAX), 0);
}
