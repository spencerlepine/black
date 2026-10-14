//! Callees a server-flow function reaches that are not ported yet (B10h harness).
//!
//! A port never invents the behaviour of such a callee: it forwards the call through
//! [`FlowEnv::call`] with the callee's address and arguments, usually from a one-line wrapper per
//! callee in its own module:
//!
//! ```ignore
//! fn fun_021ccc0c<E: FlowEnv>(env: &mut E, q: u32, cmd: u32, out: u32) {
//!     env.call(0x021C_CC0C, &mut [Arg::PAt(q), Arg::V(cmd), Arg::V(0), Arg::PAt(out)]);
//! }
//! ```
//!
//! When the callee is ported later only the wrapper body changes. The differential test supplies
//! `testkit::TestEnv`, whose behaviour (log, return value, output bytes) comes from the stub
//! lines of `tools/rom-diff/svflow_specs/<area>.spec`, the same lines the Unicorn driver
//! (`tools/rom-diff/svflow.py`) uses.
//!
//! # Pass-through pointers (B18)
//!
//! A pointer the callee's own spec declares as a buffer or BattleMon (`arg buf` / `arg bpp`) is a
//! `ptr` stub argument: the stub logs its value (an address in the stack window as `PTR_LOCAL`),
//! so the port must pass the pointer the original passes: [`Arg::PAt`] (an address), [`Arg::PHost`]
//! / [`Arg::PBuf`] (a byte of an image the port holds; the test environment maps it back to the
//! emulator address, any other host address counts as a local buffer). A bare [`Arg::P`] there
//! panics. `P` stays for pointers the callee's spec does not declare as a buffer.
//!
//! # Lending (B16)
//!
//! The battle integration test (`crate::integration`) answers `env.call` with the Rust port of the
//! callee. A port that holds an object image (the flow work `w`, a client object `c`, ...) while it
//! calls out must hand that image to the callee, or the callee would work on a stale copy: the
//! `call_w` / `call_wr` / `call_held` / [`FlowEnv::call_lent`] forms and the [`Lend`] wrapper do
//! that. The differential tests' environments keep the provided bodies, which ignore the lent
//! objects and forward to [`FlowEnv::call`], so the stub behaviour (and every recorded hash) is
//! unchanged.

use super::SvFlowWork;
use crate::battle_mon::BattleMon;
use crate::calc::BattleCalc;
use crate::event_var::EventVarStack;

/// One argument of a stubbed callee, in the callee's own register/stack order.
pub enum Arg<'a> {
    /// Scalar (or a pointer whose value matters): logged and mixed into the result.
    V(u32),
    /// Pointer the callee only passes through: ignored by the stub.
    P,
    /// Pointer to bytes the callee reads: the stub logs them (as little-endian words).
    In(&'a [u8]),
    /// Pointer to bytes the callee writes: the stub fills them.
    Out(&'a mut [u8]),
    /// A pass-through pointer to bytes the caller holds (B16): the stubs treat it exactly like
    /// [`Arg::P`] (ignored); the battle integration hands the bytes to the callee's port.
    PBuf(&'a mut [u8]),
    /// A pass-through pointer into an object the caller holds and lends (B16), given as the host
    /// address of the byte it points at (`slice[ofs..].as_ptr() as usize`; never dereferenced):
    /// the stubs treat it exactly like [`Arg::P`]; the battle integration maps it back to the
    /// object's address.
    PHost(usize),
    /// A pass-through pointer whose value the caller knows (B16): the stubs treat it exactly like
    /// [`Arg::P`]; the battle integration uses the address.
    PAt(u32),
    /// The flow work (`SvFlowWork`) plus a byte offset (B18b), for a port that passes the work
    /// pointer through without holding it: at a `ptr` stub argument the testkit logs the work
    /// image's address + offset (`arg work` callees); the battle integration passes its flow
    /// work's address + offset.
    PWork(u32),
}

/// A pointer a port only passes through (B18), kept as a value so it can be handed down: the
/// original's address ([`Arg::PAt`]; 0 = null) or the host address of a byte in an image the caller
/// holds ([`Arg::PHost`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassPtr {
    At(u32),
    Host(usize),
}

impl PassPtr {
    /// The argument that passes this pointer.
    pub fn arg(self) -> Arg<'static> {
        match self {
            PassPtr::At(v) => Arg::PAt(v),
            PassPtr::Host(h) => Arg::PHost(h),
        }
    }

    /// The original's null pointer.
    pub fn is_null(self) -> bool {
        self == PassPtr::At(0)
    }
}

/// Objects the caller holds while it calls out (B16): the callee must see (and may change) these
/// images instead of the copies its environment would otherwise use.
#[derive(Default)]
pub struct Lent<'a> {
    /// The flow work, writable.
    pub work: Option<&'a mut SvFlowWork>,
    /// The flow work, held read-only by the caller (a callee that writes it is a wiring gap).
    pub work_ref: Option<&'a SvFlowWork>,
    /// Other object images (whole objects the environment handed to the caller: a client object,
    /// the server object, ...), identified by the environment from the slice address.
    pub bytes: Vec<&'a mut [u8]>,
    /// Object images the caller holds read-only (a callee that writes them is a wiring gap).
    pub ro: Vec<&'a [u8]>,
}

impl<'a> Lent<'a> {
    /// Lends the writable flow work.
    pub fn work(w: &'a mut SvFlowWork) -> Self {
        Lent {
            work: Some(w),
            ..Lent::default()
        }
    }

    /// Lends the read-only flow work.
    pub fn work_ref(w: &'a SvFlowWork) -> Self {
        Lent {
            work_ref: Some(w),
            ..Lent::default()
        }
    }

    /// Lends the given work (writable or read-only) and object images.
    pub fn of(
        work: Option<&'a mut SvFlowWork>,
        work_ref: Option<&'a SvFlowWork>,
        bytes: Vec<&'a mut [u8]>,
    ) -> Self {
        Lent {
            work,
            work_ref,
            bytes,
            ro: Vec::new(),
        }
    }

    /// [`Self::of`] plus a command context.
    pub fn of_cx(
        work: Option<&'a mut SvFlowWork>,
        work_ref: Option<&'a SvFlowWork>,
        mut bytes: Vec<&'a mut [u8]>,
        cx: &'a mut dyn CmdCtx,
    ) -> Self {
        bytes.extend(cx.images());
        Lent {
            work,
            work_ref,
            bytes,
            ro: Vec::new(),
        }
    }

    /// Lends one object image.
    pub fn held(b: &'a mut [u8]) -> Self {
        Lent {
            bytes: vec![b],
            ..Lent::default()
        }
    }

    /// Lends one object image the caller holds read-only.
    pub fn held_ref(b: &'a [u8]) -> Self {
        Lent {
            ro: vec![b],
            ..Lent::default()
        }
    }
}

/// The environment of a server-flow function: every callee outside the function and not ported.
pub trait FlowEnv {
    /// Calls the unported function at `addr`; returns its r0 (0 for a callee returning nothing).
    /// A callee marked `noreturn` in the spec (a fatal/diagnostic routine) does not return in the
    /// test environment either: it unwinds to the testkit, which hashes the state reached so far.
    /// Ports still `return` right after such a call (a production environment may return).
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32;

    /// [`Self::call`] with the objects the caller holds lent to the callee (B16). The stub
    /// environments ignore `lent`.
    fn call_lent(&mut self, lent: Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        let _ = lent;
        self.call(addr, args)
    }

    /// [`Self::call`] lending the writable flow work.
    fn call_w(&mut self, w: &mut SvFlowWork, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.call_lent(Lent::work(w), addr, args)
    }

    /// [`Self::call`] lending the flow work the caller holds read-only.
    fn call_wr(&mut self, w: &SvFlowWork, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.call_lent(Lent::work_ref(w), addr, args)
    }

    /// [`Self::call_w`] with the work last, so the arguments may read the work: `env.callw(addr,
    /// &mut [Arg::V(w.u32_at(8))], w)`.
    fn callw(&mut self, addr: u32, args: &mut [Arg<'_>], w: &mut SvFlowWork) -> u32 {
        self.call_w(w, addr, args)
    }

    /// [`Self::call_wr`] with the work last.
    fn callwr(&mut self, addr: u32, args: &mut [Arg<'_>], w: &SvFlowWork) -> u32 {
        self.call_wr(w, addr, args)
    }

    /// [`Self::call_lent`] with the lent objects last (so the arguments may read them).
    fn call_l(&mut self, addr: u32, args: &mut [Arg<'_>], lent: Lent<'_>) -> u32 {
        self.call_lent(lent, addr, args)
    }

    /// [`Self::call`] lending an image the caller holds read-only: `env.callr(addr, args, c)`.
    fn callr(&mut self, addr: u32, args: &mut [Arg<'_>], held: &[u8]) -> u32 {
        self.call_lent(Lent::held_ref(held), addr, args)
    }

    /// [`Self::call_held`] with the held image last: `env.callh(addr, &mut [Arg::V(rd32(c, 8))], c)`.
    fn callh(&mut self, addr: u32, args: &mut [Arg<'_>], held: &mut [u8]) -> u32 {
        self.call_held(held, addr, args)
    }

    /// [`Self::call`] lending the command context (the queue and the scratch words).
    fn callc(&mut self, addr: u32, args: &mut [Arg<'_>], cx: &mut dyn CmdCtx) -> u32 {
        let lent = Lent {
            bytes: cx.images().into(),
            ..Lent::default()
        };
        self.call_lent(lent, addr, args)
    }

    /// [`Self::callw`] also lending the command context.
    fn callwc(
        &mut self,
        addr: u32,
        args: &mut [Arg<'_>],
        w: &mut SvFlowWork,
        cx: &mut dyn CmdCtx,
    ) -> u32 {
        let lent = Lent {
            work: Some(w),
            bytes: cx.images().into(),
            ..Lent::default()
        };
        self.call_lent(lent, addr, args)
    }

    /// [`Self::callwr`] also lending the command context.
    fn callwrc(
        &mut self,
        addr: u32,
        args: &mut [Arg<'_>],
        w: &SvFlowWork,
        cx: &mut dyn CmdCtx,
    ) -> u32 {
        let lent = Lent {
            work_ref: Some(w),
            bytes: cx.images().into(),
            ..Lent::default()
        };
        self.call_lent(lent, addr, args)
    }

    /// [`Self::call`] lending one object image the caller holds (e.g. the client object).
    fn call_held(&mut self, held: &mut [u8], addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.call_lent(Lent::held(held), addr, args)
    }

    /// B16: reads memory outside the images a port holds (a pointer a callee returned that
    /// points elsewhere than the port's model assumed). `false` (the default, the stubs): the port
    /// keeps its own model.
    fn peek(&mut self, addr: u32, out: &mut [u8]) -> bool {
        let _ = (addr, out);
        false
    }

    /// B16: the event-variable stack (global 0x021F3ECC) when the environment owns it (the
    /// handlers then use it instead of [`crate::flow::Battle::vars`]). `None` in the stubs.
    fn ext_vars(&self) -> Option<&EventVarStack> {
        None
    }

    /// Writable [`Self::ext_vars`].
    fn ext_vars_mut(&mut self) -> Option<&mut EventVarStack> {
        None
    }

    /// B16: the battle RNG / calc state when the environment owns it.
    fn ext_calc_mut(&mut self) -> Option<&mut BattleCalc> {
        None
    }

    /// B16: the battle Pokemon at address `addr` when the environment owns the battle Pokemon.
    fn ext_mon_at(&mut self, addr: u32) -> Option<&mut BattleMon> {
        let _ = addr;
        None
    }
}

/// A command context the flow ports hold ([`super::core2::Cx`], [`super::core4::Cx`]: the queue
/// and the globals `put_cmd` writes), lent with the work (B16).
pub trait CmdCtx {
    /// The two byte images (queue, globals).
    fn images(&mut self) -> [&mut [u8]; 2];
}

impl CmdCtx for super::core2::Cx<'_> {
    fn images(&mut self) -> [&mut [u8]; 2] {
        [&mut *self.q, &mut *self.scratch]
    }
}

impl CmdCtx for super::core4::Cx<'_> {
    fn images(&mut self) -> [&mut [u8]; 2] {
        [&mut *self.q, &mut *self.g]
    }
}

/// A [`FlowEnv`] that lends the objects it holds on every call (B16): the writable or read-only
/// flow work and / or the command context ([`Cx`]: the queue and the `put_cmd` scratch words). A
/// port holding them passes `&mut Lend::new(env, w)` (or [`Lend::wc`] / [`Lend::c`] /
/// [`LendRef`]) to an in-area helper that calls out but does not take them itself (e.g. the
/// event-message helpers around `021d70f0`), so the callee still sees the caller's images.
pub struct Lend<'a, E: FlowEnv + ?Sized> {
    pub env: &'a mut E,
    pub w: Option<&'a mut SvFlowWork>,
    pub w_ref: Option<&'a SvFlowWork>,
    pub cx: Option<&'a mut dyn CmdCtx>,
    /// Object images the caller holds (e.g. the client object), lent on every call.
    pub held: Vec<&'a mut [u8]>,
    /// Object images the caller holds read-only, lent on every call.
    pub held_ro: Vec<&'a [u8]>,
}

/// The read-only-work form of [`Lend`].
pub type LendRef<'a, E> = Lend<'a, E>;

impl<'a, E: FlowEnv + ?Sized> Lend<'a, E> {
    /// Lends the writable work.
    pub fn new(env: &'a mut E, w: &'a mut SvFlowWork) -> Self {
        Lend {
            env,
            w: Some(w),
            w_ref: None,
            cx: None,
            held: Vec::new(),
            held_ro: Vec::new(),
        }
    }

    /// Lends the writable work and the command context.
    pub fn wc(env: &'a mut E, w: &'a mut SvFlowWork, cx: &'a mut dyn CmdCtx) -> Self {
        Lend {
            env,
            w: Some(w),
            w_ref: None,
            cx: Some(cx),
            held: Vec::new(),
            held_ro: Vec::new(),
        }
    }

    /// Lends the command context.
    pub fn c(env: &'a mut E, cx: &'a mut dyn CmdCtx) -> Self {
        Lend {
            env,
            w: None,
            w_ref: None,
            cx: Some(cx),
            held: Vec::new(),
            held_ro: Vec::new(),
        }
    }

    /// Lends the read-only work ([`LendRef`]).
    pub fn new_ref(env: &'a mut E, w: &'a SvFlowWork) -> Self {
        Lend {
            env,
            w: None,
            w_ref: Some(w),
            cx: None,
            held: Vec::new(),
            held_ro: Vec::new(),
        }
    }

    /// Lends an object image the caller holds (a client object, ...).
    pub fn h(env: &'a mut E, held: &'a mut [u8]) -> Self {
        Lend {
            env,
            w: None,
            w_ref: None,
            cx: None,
            held: vec![held],
            held_ro: Vec::new(),
        }
    }

    /// Lends an object image the caller holds read-only.
    pub fn r(env: &'a mut E, held: &'a [u8]) -> Self {
        Lend {
            env,
            w: None,
            w_ref: None,
            cx: None,
            held: Vec::new(),
            held_ro: vec![held],
        }
    }

    /// [`Self::wh`] with the work held read-only.
    pub fn whr(env: &'a mut E, w: &'a SvFlowWork, held: Vec<&'a mut [u8]>) -> Self {
        Lend {
            env,
            w: None,
            w_ref: Some(w),
            cx: None,
            held,
            held_ro: Vec::new(),
        }
    }

    /// Lends object images the caller holds (the command queue and its scratch words, ...).
    pub fn hv(env: &'a mut E, held: Vec<&'a mut [u8]>) -> Self {
        Lend {
            env,
            w: None,
            w_ref: None,
            cx: None,
            held,
            held_ro: Vec::new(),
        }
    }

    /// Lends the writable work and object images (the server object, the queue, ...).
    pub fn wh(env: &'a mut E, w: &'a mut SvFlowWork, held: Vec<&'a mut [u8]>) -> Self {
        Lend {
            env,
            w: Some(w),
            w_ref: None,
            cx: None,
            held,
            held_ro: Vec::new(),
        }
    }

    /// Lends the read-only work and the command context.
    pub fn wrc(env: &'a mut E, w: &'a SvFlowWork, cx: &'a mut dyn CmdCtx) -> Self {
        Lend {
            env,
            w: None,
            w_ref: Some(w),
            cx: Some(cx),
            held: Vec::new(),
            held_ro: Vec::new(),
        }
    }
}

impl<E: FlowEnv + ?Sized> FlowEnv for Lend<'_, E> {
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.call_lent(Lent::default(), addr, args)
    }

    fn call_lent(&mut self, lent: Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        let Lent {
            mut work,
            mut work_ref,
            mut bytes,
            mut ro,
        } = lent;
        for h in self.held_ro.iter() {
            if !ro.iter().any(|b| b.as_ptr() == h.as_ptr()) {
                ro.push(h);
            }
        }
        if work.is_none() && work_ref.is_none() {
            if let Some(w) = self.w.as_deref_mut() {
                work = Some(w);
            } else if let Some(w) = self.w_ref {
                work_ref = Some(w);
            }
        }
        for h in self.held.iter_mut() {
            if !bytes.iter().any(|b| b.as_ptr() == h.as_ptr()) {
                bytes.push(&mut **h);
            }
        }
        if let Some(cx) = self.cx.as_deref_mut() {
            let [q, g] = cx.images();
            if !bytes.iter().any(|b| b.as_ptr() == q.as_ptr()) {
                bytes.push(q);
                bytes.push(g);
            }
        }
        self.env.call_lent(
            Lent {
                work,
                work_ref,
                bytes,
                ro,
            },
            addr,
            args,
        )
    }

    fn peek(&mut self, addr: u32, out: &mut [u8]) -> bool {
        self.env.peek(addr, out)
    }

    fn ext_vars(&self) -> Option<&EventVarStack> {
        self.env.ext_vars()
    }

    fn ext_vars_mut(&mut self) -> Option<&mut EventVarStack> {
        self.env.ext_vars_mut()
    }

    fn ext_calc_mut(&mut self) -> Option<&mut BattleCalc> {
        self.env.ext_calc_mut()
    }

    fn ext_mon_at(&mut self, addr: u32) -> Option<&mut BattleMon> {
        self.env.ext_mon_at(addr)
    }
}

impl<E: FlowEnv + crate::battle_mon::BppEnv + ?Sized> crate::battle_mon::BppEnv for Lend<'_, E> {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.env.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.env.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.env.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.env.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.env.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.env.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.env.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.env.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.env.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.env.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.env.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.env.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.env.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.env.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.env.sick_is_passable(sick)
    }
}

/// A borrowed environment is an environment (B16: the integration hands `&mut Env` to the
/// differential tests' `call` tables).
impl<T: FlowEnv + ?Sized> FlowEnv for &mut T {
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        (**self).call(addr, args)
    }

    fn call_lent(&mut self, lent: Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        (**self).call_lent(lent, addr, args)
    }

    fn peek(&mut self, addr: u32, out: &mut [u8]) -> bool {
        (**self).peek(addr, out)
    }

    fn ext_vars(&self) -> Option<&EventVarStack> {
        (**self).ext_vars()
    }

    fn ext_vars_mut(&mut self) -> Option<&mut EventVarStack> {
        (**self).ext_vars_mut()
    }

    fn ext_calc_mut(&mut self) -> Option<&mut BattleCalc> {
        (**self).ext_calc_mut()
    }

    fn ext_mon_at(&mut self, addr: u32) -> Option<&mut BattleMon> {
        (**self).ext_mon_at(addr)
    }
}
