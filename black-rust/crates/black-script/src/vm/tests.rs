use super::*;
use crate::hooks::Mem;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02011330, "13ed18a3e817c655"),
    (0x0201137C, "cdf05398993ea5c1"),
];
// fndiff-recorded-end

/// Argument buffer 0 (the VM arena) as the memory window.
struct Arena(Mem);

impl MemEnv for Arena {
    fn mem(&mut self) -> &mut Mem {
        &mut self.0
    }
}

fn with_vm<R>(c: &mut Case, f: impl FnOnce(&mut Arena, u32) -> R) -> R {
    let vm = c.vals[0];
    let mut a = Arena(Mem::new(vm, std::mem::take(&mut c.bufs[0])));
    let r = f(&mut a, vm);
    c.bufs[0] = a.0.bytes;
    r
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v1 = c.vals.get(1).copied().unwrap_or(0);
    match addr {
        0x0201_1330 => with_vm(c, read_u16) as u64,
        0x0201_137C => {
            with_vm(c, |m, vm| push(m, vm, v1));
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-script vm`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "vm",
            include_str!("../../../../tools/rom-diff/specs/black-script/vm.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn push_then_read() {
    let mut a = Arena(Mem::new(0x1000, vec![0; 0x200]));
    a.w16(0x1000, 4); // 4 stack words
    a.w32(0x1000 + VM_STACK, 0x1100);
    for v in 1..=5 {
        push(&mut a, 0x1000, v);
    }
    // depth + 1 < 4: three pushes land, the rest are dropped
    assert_eq!(a.r8(0x1000 + VM_DEPTH), 3);
    assert_eq!(a.r32(0x1108), 3);
    a.w32(0x1000 + VM_PC, 0x1180);
    a.w16(0x1180, 0x1234);
    assert_eq!(read_u16(&mut a, 0x1000), 0x1234);
    assert_eq!(a.r32(0x1000 + VM_PC), 0x1182);
}
