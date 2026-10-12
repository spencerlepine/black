//! Callees a server-flow function reaches that are not ported yet (B10h harness).
//!
//! A port never invents the behaviour of such a callee: it forwards the call through
//! [`FlowEnv::call`] with the callee's address and arguments, usually from a one-line wrapper per
//! callee in its own module:
//!
//! ```ignore
//! fn fun_021ccc0c<E: FlowEnv>(env: &mut E, q: u32, cmd: u32) {
//!     env.call(0x021C_CC0C, &mut [Arg::P, Arg::V(cmd), Arg::V(0), Arg::P]);
//! }
//! ```
//!
//! When the callee is ported later only the wrapper body changes. The differential test supplies
//! `testkit::TestEnv`, whose behaviour (log, return value, output bytes) comes from the stub
//! lines of `tools/rom-diff/svflow_specs/<area>.spec`, the same lines the Unicorn driver
//! (`tools/rom-diff/svflow.py`) uses.

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
}

/// The environment of a server-flow function: every callee outside the function and not ported.
pub trait FlowEnv {
    /// Calls the unported function at `addr`; returns its r0 (0 for a callee returning nothing).
    /// A callee marked `noreturn` in the spec (a fatal/diagnostic routine) does not return in the
    /// test environment either: it unwinds to the testkit, which hashes the state reached so far.
    /// Ports still `return` right after such a call (a production environment may return).
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32;
}
