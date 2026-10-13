use super::*;
use crate::hooks::{CommandDispatch, Mem};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020111EC, "337fbdd1f98e2635"),
    (0x0201121C, "54ec5c37c778dc1d"),
    (0x0201126C, "aad6a2732d8e7a1b"),
    (0x0201127C, "fd87c16eb312efad"),
    (0x02011290, "9fd051ffbc336080"),
    (0x02011298, "68f978af82b1c9f9"),
    (0x02011324, "b931c8d3de24e4aa"),
    (0x02011328, "7972501353557412"),
    (0x02011330, "13ed18a3e817c655"),
    (0x0201134C, "ebfe7822bbbdbb78"),
    (0x0201137C, "cdf05398993ea5c1"),
    (0x02011398, "240396d692427727"),
    (0x020113B0, "20fd7cb07a39fc44"),
    (0x020113B4, "61d1de780f849fd2"),
    (0x020113C4, "5c32888cd27411e9"),
    (0x020113D0, "67878ac471fec1c5"),
];
// fndiff-recorded-end

/// One argument buffer (the VM arena) as the memory window, with the spec's stubs for the heap and the function pointers
/// (fake handlers at 0x02EE0011 / 21 / 31 land on the stubs 02ee0010 / 20 / 30).
struct Arena<'a> {
    mem: Mem,
    stubs: &'a mut StubEnv,
}

impl MemEnv for Arena<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl HeapHooks for Arena<'_> {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.stubs
            .call(0x0203_0734, &mut [heap, size, flag, file, line].map(Arg::V))
    }
    fn heap_free(&mut self, block: u32) {
        self.stubs.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
}

impl CommandDispatch for Arena<'_> {
    fn command(&mut self, handler: u32, vm: u32, ctx: u32) -> u32 {
        self.stubs
            .call(handler & !1, &mut [Arg::V(vm), Arg::V(ctx)])
    }
}

impl VmHooks for Arena<'_> {
    fn call_wait(&mut self, f: u32, vm: u32, ctx: u32) -> u32 {
        self.stubs.call(f & !1, &mut [Arg::V(vm), Arg::V(ctx)])
    }
    fn call_hook(&mut self, f: u32, vm: u32, ctx: u32, arg: u32, cmd: u32) -> u32 {
        self.stubs
            .call(f & !1, &mut [vm, ctx, arg, cmd].map(Arg::V))
    }
}

/// Run `f` with argument buffer `k` as the memory window; the buffer is put back afterwards.
fn with_buf<R>(c: &mut Case, k: usize, f: impl FnOnce(&mut Arena, &[u32]) -> R) -> R {
    let vals = c.vals.clone();
    let mut a = Arena {
        mem: Mem::new(vals[k], std::mem::take(&mut c.bufs[k])),
        stubs: &mut c.env,
    };
    let r = f(&mut a, &vals);
    c.bufs[k] = a.mem.bytes;
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_11EC => with_buf(c, 0, |m, v| {
            reset(m, v[0]);
            0
        }),
        0x0201_121C => {
            let init: [u8; 12] = c.bufs[1][..12].try_into().unwrap();
            with_buf(c, 2, |m, v| {
                create(m, v[0], &init).wrapping_sub(v[2]) as u64
            })
        }
        0x0201_126C => with_buf(c, 0, |m, v| {
            init(m, v[0], v[1]);
            0
        }),
        0x0201_127C => with_buf(c, 0, |m, v| {
            start(m, v[0], v[1]);
            0
        }),
        0x0201_1290 => with_buf(c, 0, |m, v| {
            stop(m, v[0]);
            0
        }),
        0x0201_1298 => with_buf(c, 0, |m, v| step(m, v[0]) as u64),
        0x0201_1324 => with_buf(c, 0, |m, v| ctx(m, v[0]) as u64),
        0x0201_1328 => with_buf(c, 0, |m, v| {
            set_hook(m, v[0], v[1], v[2]);
            0
        }),
        0x0201_1330 => with_buf(c, 0, |m, v| read_u16(m, v[0]) as u64),
        0x0201_134C => with_buf(c, 0, |m, v| read_u32(m, v[0]) as u64),
        0x0201_137C => with_buf(c, 0, |m, v| {
            push(m, v[0], v[1]);
            0
        }),
        0x0201_1398 => with_buf(c, 0, |m, v| pop(m, v[0]) as u64),
        0x0201_13B0 => with_buf(c, 0, |m, v| {
            jump(m, v[0], v[1]);
            0
        }),
        0x0201_13B4 => with_buf(c, 0, |m, v| {
            call(m, v[0], v[1]);
            0
        }),
        0x0201_13C4 => with_buf(c, 0, |m, v| {
            ret(m, v[0]);
            0
        }),
        0x0201_13D0 => with_buf(c, 0, |m, v| {
            wait(m, v[0], v[1]);
            0
        }),
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
        &run,
    );
}

/// Plain memory, a two-command table and a hook that counts calls.
#[derive(Default)]
struct Toy {
    mem: Mem,
    log: Vec<(u32, u32)>,
}

impl MemEnv for Toy {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl HeapHooks for Toy {
    fn heap_alloc(&mut self, _heap: u32, size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        assert!(size <= 0x600);
        0x1000
    }
    fn heap_free(&mut self, _block: u32) {}
}

impl CommandDispatch for Toy {
    fn command(&mut self, handler: u32, vm: u32, _ctx: u32) -> u32 {
        let arg = read_u16(self, vm);
        self.log.push((handler, arg));
        (handler == 0x2001) as u32
    }
}

impl VmHooks for Toy {
    fn call_wait(&mut self, _f: u32, _vm: u32, _ctx: u32) -> u32 {
        1
    }
    fn call_hook(&mut self, _f: u32, _vm: u32, _ctx: u32, _arg: u32, _cmd: u32) -> u32 {
        1
    }
}

#[test]
fn push_then_read() {
    let mut a = Toy {
        mem: Mem::new(0x1000, vec![0; 0x200]),
        ..Default::default()
    };
    a.w16(0x1000, 4); // 4 stack words
    a.w32(0x1000 + VM_STACK, 0x1100);
    for v in 1..=5 {
        push(&mut a, 0x1000, v);
    }
    // depth + 1 < 4: three pushes land, the rest are dropped
    assert_eq!(a.r8(0x1000 + VM_DEPTH), 3);
    assert_eq!(a.r32(0x1108), 3);
    assert_eq!(pop(&mut a, 0x1000), 3);
    a.w32(0x1000 + VM_PC, 0x1180);
    a.w16(0x1180, 0x1234);
    assert_eq!(read_u16(&mut a, 0x1000), 0x1234);
    assert_eq!(a.r32(0x1000 + VM_PC), 0x1182);
}

#[test]
fn step_runs_commands_until_one_yields() {
    let mut t = Toy {
        mem: Mem::new(0x1000, vec![0; 0x800]),
        ..Default::default()
    };
    let mut init = [0u8; 12];
    init[0..2].copy_from_slice(&0x10u16.to_le_bytes());
    init[2..4].copy_from_slice(&4u16.to_le_bytes());
    init[4..8].copy_from_slice(&0x1700u32.to_le_bytes());
    init[8..12].copy_from_slice(&2u32.to_le_bytes());
    let vm = create(&mut t, 0, &init);
    assert_eq!(t.r32(vm + VM_REGS), 0x1030);
    assert_eq!(t.r32(vm + VM_STACK), 0x1040);
    t.w32(0x1700, 0x2000); // command 0: next
    t.w32(0x1704, 0x2001); // command 1: yield
                           // script: cmd 0 (arg 7), cmd 1 (arg 9), cmd 5 (out of range: stops)
    for (i, w) in [0u16, 7, 1, 9, 5].iter().enumerate() {
        t.w16(0x1600 + 2 * i as u32, *w);
    }
    start(&mut t, vm, 0x1600);
    assert_eq!(step(&mut t, vm), 1);
    assert_eq!(t.log, [(0x2000, 7), (0x2001, 9)]);
    wait(&mut t, vm, 0x3001);
    assert_eq!(step(&mut t, vm), 1); // the callback reports done: running again, no command this frame
    assert_eq!(t.r8(vm + VM_STATUS), STATUS_RUNNING);
    assert_eq!(step(&mut t, vm), 0);
    assert_eq!(t.r8(vm + VM_STATUS), STATUS_STOPPED);
}
