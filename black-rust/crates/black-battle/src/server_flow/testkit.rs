//! Differential driver for the server-flow areas (B10): a thin wrapper over `black-fndiff`.
//!
//! The spec parser, input generator, stub model and hash live in `crates/black-fndiff` and mirror
//! tools/rom-diff/svflow.py (grammar: tools/rom-diff/svflow_specs/README.md). This file adds what
//! is specific to the server flow: the `SvFlowWork` image as `Case::work`, the [`FlowEnv`] view of
//! the stubs, and the BattleMon `bpp` argument / `env bpp` API stubs. Porting workers wire their
//! function into the area's `call` match; they do not touch this file, the specs or the recorded
//! hashes.

use super::env::{Arg, FlowEnv};
use super::SvFlowWork;
use crate::battle_mon::tests::{random_bpp, MockEnv};
use black_fndiff as fnd;
use black_fndiff::Rng;

/// The unported callees of one function under test; behaviour comes from its spec stub lines.
pub struct TestEnv {
    stub: fnd::StubEnv,
    /// BattleMon-API stubs (`env bpp` in the spec); same mock the BattleMon tests use.
    pub bpp: MockEnv,
    /// B18: `(host address, length, emulator address)` of every image of the input (work, `buf` /
    /// `bpp` arguments, regions, globals), so a pass-through pointer into one of them
    /// ([`Arg::PHost`] / [`Arg::PBuf`]) reaches a `ptr` stub as the address the original passes.
    layout: Vec<(usize, usize, u32)>,
}

/// B18: the emulator address of host address `h` inside one of the input's images; any other
/// address is a buffer local to the port (a stack address: `canon_ptr` logs it as `PTR_LOCAL`,
/// as it does for the original's locals). One past the end of an image maps to its end.
fn emu_addr(layout: &[(usize, usize, u32)], h: usize) -> u32 {
    if let Some(&(s, _, e)) = layout.iter().find(|&&(s, n, _)| h >= s && h < s + n) {
        return e + (h - s) as u32;
    }
    if let Some(&(_, n, e)) = layout.iter().find(|&&(s, n, _)| h == s + n) {
        return e + n as u32;
    }
    fnd::STACK_ZERO.start
}

impl FlowEnv for TestEnv {
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        let layout = &self.layout;
        let mut conv: Vec<fnd::Arg<'_>> = args
            .iter_mut()
            .map(|a| match a {
                Arg::V(v) => fnd::Arg::V(*v),
                Arg::P => fnd::Arg::P,
                Arg::PAt(v) => fnd::Arg::Ptr(*v),
                Arg::PWork(o) => fnd::Arg::Ptr(fnd::WORK_ADDR.wrapping_add(*o)),
                Arg::PHost(h) => fnd::Arg::Ptr(emu_addr(layout, *h)),
                Arg::PBuf(b) => fnd::Arg::Ptr(emu_addr(layout, b.as_ptr() as usize)),
                Arg::In(b) => fnd::Arg::In(b),
                Arg::Out(b) => fnd::Arg::Out(b),
            })
            .collect();
        self.stub.call(addr, &mut conv)
    }
}

impl TestEnv {
    /// A `ret=u64` stub (B14-spec, e.g. OS_GetTick 0x020870D0): forwards to [`fnd::StubEnv::call64`], `r1 << 32 | r0`
    /// (Z21: the AI ports reach it through `ai::AiEnv::call64`).
    pub fn call64(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u64 {
        let layout = &self.layout;
        let mut conv: Vec<fnd::Arg<'_>> = args
            .iter_mut()
            .map(|a| match a {
                Arg::V(v) => fnd::Arg::V(*v),
                Arg::P => fnd::Arg::P,
                Arg::PAt(v) => fnd::Arg::Ptr(*v),
                Arg::PWork(o) => fnd::Arg::Ptr(fnd::WORK_ADDR.wrapping_add(*o)),
                Arg::PHost(h) => fnd::Arg::Ptr(emu_addr(layout, *h)),
                Arg::PBuf(b) => fnd::Arg::Ptr(emu_addr(layout, b.as_ptr() as usize)),
                Arg::In(b) => fnd::Arg::In(b),
                Arg::Out(b) => fnd::Arg::Out(b),
            })
            .collect();
        self.stub.call64(addr, &mut conv)
    }
}

/// One generated input. `vals[i]` is argument `i` as passed to the original (a scalar, or the
/// fixed emulator address of a pointer argument); `bufs[i]` holds the bytes behind a `buf`/`bpp`
/// argument (empty for scalars). The port runs on these and the testkit hashes them afterwards.
///
/// B16: generic over the environment so the battle integration (`crate::integration`) can run the
/// same `call` tables on its own environment; the differential tests use `Case<TestEnv>`.
pub struct Case<E = TestEnv> {
    pub work: SvFlowWork,
    pub vals: Vec<u32>,
    pub bufs: Vec<Vec<u8>>,
    pub globals: Vec<Vec<u8>>,
    /// `region NAME SIZE` blocks of the spec, in declaration order (B10f-spec); region k is at
    /// `0x02F1_0000 + k * 0x10000` in `vals`.
    pub regions: Vec<Vec<u8>>,
    pub env: E,
}

impl<E> Case<E> {
    /// Argument word `i` as the original sees it at entry: `vals[i]` for a declared argument, 0
    /// for r0-r3 or a stack word past the declared ones (Z19: the emulator zeroes r0-r12 and the
    /// stack window before every call, see [`fnd::UNSET_WORD`]). E.g. a caller's saved r3 that
    /// the original reads back through a stack buffer (021b6848) is `c.word(3)`.
    pub fn word(&self, i: usize) -> u32 {
        self.vals.get(i).copied().unwrap_or(fnd::UNSET_WORD)
    }
}

/// The crate-specific half of the spec: `arg bpp` and `env bpp`.
#[derive(Default)]
struct Bpp {
    env: MockEnv,
}

impl fnd::Ext for Bpp {
    fn begin(&mut self, envs: &[String], r: &mut Rng) {
        if envs.iter().any(|e| e == "bpp") {
            self.env.field = r.next() & 1 != 0;
        }
    }

    fn custom_arg(&mut self, kind: &str, r: &mut Rng) -> Vec<u8> {
        assert_eq!(kind, "bpp", "svflow spec: unknown arg kind {kind:?}");
        random_bpp(r).raw.to_vec()
    }

    fn env_words(&self, _envs: &[String]) -> Vec<u32> {
        self.env.log.iter().flatten().copied().collect()
    }
}

/// Runs every function in `ported` over its spec inputs through `call` (which maps the address
/// to the port and returns its result), and checks the per-function hashes against `recorded`.
///
/// Harness env vars (used by tools/check-svflow.sh): `SVFLOW_JSON`, `SVFLOW_ONLY`,
/// `SVFLOW_DETAIL` + `SVFLOW_DETAIL_OUT`, as documented on [`fnd::run_suite`].
pub fn run_area_suite(
    area: &str,
    spec_text: &str,
    ported: &[u32],
    recorded: &[(u32, &str)],
    call: fn(u32, &mut Case) -> u64,
) {
    let suite = fnd::Suite {
        env_prefix: "SVFLOW",
        tool: "tools/check-svflow.sh",
        ..fnd::Suite::new(area, spec_text, ported, recorded)
    };
    fnd::run_suite(&suite, &|addr, fc: &mut fnd::Case<Bpp>| {
        let nowork = fc.work.is_empty();
        let work = if nowork {
            SvFlowWork::new()
        } else {
            SvFlowWork {
                mem: std::mem::take(&mut fc.work)
                    .into_boxed_slice()
                    .try_into()
                    .unwrap(),
            }
        };
        let mut c = Case {
            work,
            vals: std::mem::take(&mut fc.vals),
            bufs: std::mem::take(&mut fc.bufs),
            globals: std::mem::take(&mut fc.globals),
            regions: std::mem::take(&mut fc.regions),
            env: TestEnv {
                stub: std::mem::take(&mut fc.env),
                bpp: std::mem::take(&mut fc.ext.env),
                layout: Vec::new(),
            },
        };
        let mut layout = Vec::new();
        if !nowork {
            layout.push((
                c.work.mem.as_ptr() as usize,
                c.work.mem.len(),
                fnd::WORK_ADDR,
            ));
        }
        let images = c
            .bufs
            .iter()
            .enumerate()
            .map(|(i, b)| (b, fnd::buf_addr(i)));
        let images = images.chain(
            c.regions
                .iter()
                .enumerate()
                .map(|(k, b)| (b, fnd::region_addr(k))),
        );
        let images = images.chain(
            c.globals
                .iter()
                .enumerate()
                .map(|(k, b)| (b, fc.global_addr(k))),
        );
        for (b, e) in images {
            if !b.is_empty() {
                layout.push((b.as_ptr() as usize, b.len(), e));
            }
        }
        c.env.layout = layout;
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(addr, &mut c)));
        // a `noreturn` stub unwinds out of the port: hand the state back before resuming
        let Case {
            work,
            vals,
            bufs,
            globals,
            regions,
            env,
        } = c;
        fc.work = if nowork {
            Vec::new()
        } else {
            (work.mem as Box<[u8]>).into_vec()
        };
        fc.vals = vals;
        fc.bufs = bufs;
        fc.globals = globals;
        fc.regions = regions;
        fc.env = env.stub;
        fc.ext.env = env.bpp;
        match res {
            Ok(r) => r,
            Err(e) => std::panic::resume_unwind(e),
        }
    });
}

/// Harness self-test: three trivial ROM functions re-implemented inline, checked against the
/// hashes `tools/rom-diff/svflow.py selftest --all --json` prints. Guards the Python/Rust parity
/// of input generation, the argument kinds `work`/`bpp`, return masking and hashing.
#[cfg(test)]
mod tests {
    use super::*;

    const RECORDED: &[(u32, &str)] = &[
        (0x021C7CD4, "2a551cbea4b3ffa5"),
        (0x021C8138, "285dae81a96611bc"),
        (0x021D5448, "d4bc1625b622289c"),
    ];

    fn call(addr: u32, c: &mut Case) -> u64 {
        match addr {
            0x021C7CD4 => c.work.u16_at(0x1F80) as u64,
            0x021C8138 => (c.work.u32_at(0x10) & 0xFFFF) as u64,
            0x021D5448 => c.bufs[0][0x140] as u64,
            _ => unreachable!(),
        }
    }

    /// Z19: undeclared argument words (r1-r3, stack) are the emulator's reset value 0.
    #[test]
    fn word_past_declared_args_is_zero() {
        let c = Case {
            work: SvFlowWork::new(),
            vals: vec![0x02E0_0000, 5],
            bufs: Vec::new(),
            globals: Vec::new(),
            regions: Vec::new(),
            env: TestEnv {
                stub: fnd::StubEnv::default(),
                bpp: MockEnv::default(),
                layout: Vec::new(),
            },
        };
        assert_eq!(c.word(0), 0x02E0_0000);
        assert_eq!(c.word(1), 5);
        assert!((2..10).all(|i| c.word(i) == 0));
    }

    /// B18: every battle svflow spec (tools/rom-diff/svflow_specs/*.spec).
    const BATTLE_SPECS: &[(&str, &str)] = &[
        (
            "adapter",
            include_str!("../../../../tools/rom-diff/svflow_specs/adapter.spec"),
        ),
        (
            "ai1",
            include_str!("../../../../tools/rom-diff/svflow_specs/ai1.spec"),
        ),
        (
            "api",
            include_str!("../../../../tools/rom-diff/svflow_specs/api.spec"),
        ),
        (
            "client1",
            include_str!("../../../../tools/rom-diff/svflow_specs/client1.spec"),
        ),
        (
            "client2",
            include_str!("../../../../tools/rom-diff/svflow_specs/client2.spec"),
        ),
        (
            "client3",
            include_str!("../../../../tools/rom-diff/svflow_specs/client3.spec"),
        ),
        (
            "client4",
            include_str!("../../../../tools/rom-diff/svflow_specs/client4.spec"),
        ),
        (
            "client5",
            include_str!("../../../../tools/rom-diff/svflow_specs/client5.spec"),
        ),
        (
            "cmd_queue",
            include_str!("../../../../tools/rom-diff/svflow_specs/cmd_queue.spec"),
        ),
        (
            "core1",
            include_str!("../../../../tools/rom-diff/svflow_specs/core1.spec"),
        ),
        (
            "core2",
            include_str!("../../../../tools/rom-diff/svflow_specs/core2.spec"),
        ),
        (
            "core3",
            include_str!("../../../../tools/rom-diff/svflow_specs/core3.spec"),
        ),
        (
            "core4",
            include_str!("../../../../tools/rom-diff/svflow_specs/core4.spec"),
        ),
        (
            "core5",
            include_str!("../../../../tools/rom-diff/svflow_specs/core5.spec"),
        ),
        (
            "core6",
            include_str!("../../../../tools/rom-diff/svflow_specs/core6.spec"),
        ),
        (
            "events",
            include_str!("../../../../tools/rom-diff/svflow_specs/events.spec"),
        ),
        (
            "handex",
            include_str!("../../../../tools/rom-diff/svflow_specs/handex.spec"),
        ),
        (
            "hem",
            include_str!("../../../../tools/rom-diff/svflow_specs/hem.spec"),
        ),
        (
            "selftest",
            include_str!("../../../../tools/rom-diff/svflow_specs/selftest.spec"),
        ),
        (
            "server1",
            include_str!("../../../../tools/rom-diff/svflow_specs/server1.spec"),
        ),
        (
            "server2",
            include_str!("../../../../tools/rom-diff/svflow_specs/server2.spec"),
        ),
        (
            "server3",
            include_str!("../../../../tools/rom-diff/svflow_specs/server3.spec"),
        ),
        (
            "server_side",
            include_str!("../../../../tools/rom-diff/svflow_specs/server_side.spec"),
        ),
    ];

    /// B18: no stub of a battle svflow spec passes a `buf` / `bpp` argument of its callee as a
    /// bare `p` (the same lint `svflow.py` runs when it loads a battle area and in `--selftest`).
    #[test]
    fn battle_specs_pass_buffers_as_ptr() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/rom-diff/svflow_specs"
        );
        let mut files: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| {
                let n = e.unwrap().file_name().into_string().unwrap();
                n.strip_suffix(".spec").map(str::to_string)
            })
            .collect();
        files.sort();
        let listed: Vec<&str> = BATTLE_SPECS.iter().map(|s| s.0).collect();
        assert_eq!(files, listed, "BATTLE_SPECS must list every svflow spec");
        let bad = fnd::bare_p_lint(BATTLE_SPECS);
        assert!(
            bad.is_empty(),
            "bare `p` for buf/bpp arguments:\n{}",
            bad.join("\n")
        );
    }

    #[test]
    fn selftest_matches_python_driver() {
        let addrs: Vec<u32> = RECORDED.iter().map(|r| r.0).collect();
        run_area_suite(
            "selftest",
            include_str!("../../../../tools/rom-diff/svflow_specs/selftest.spec"),
            &addrs,
            RECORDED,
            call,
        );
    }
}
