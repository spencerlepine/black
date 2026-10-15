//! black-script on the F12 machine: the hand-written part of the hook traits (`script_hooks.rs` has the
//! generated `fun_xxxxxxxx` part). Function pointers the script ports call through (command handlers,
//! wait callbacks, the dispatch hook, init procs, end callbacks) go by address like every call: the port
//! when there is one, else the original code.

use super::args::Args;
use super::Env;
use black_script::hooks::Mem as SMem;

impl black_script::hooks::MemEnv for Env {
    fn mem(&mut self) -> &mut SMem {
        self.smem()
    }
}

impl black_script::hooks::HeapHooks for Env {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.cv(crate::hooks::HEAP_ALLOC, &[heap, size, flag, file, line])
    }
    fn heap_free(&mut self, block: u32) {
        self.cv(crate::hooks::HEAP_FREE, &[block]);
    }
}

impl crate::hooks::HeapHooks for Env {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.cv(crate::hooks::HEAP_ALLOC, &[heap, size, flag, file, line])
    }
    fn heap_free(&mut self, block: u32) {
        self.cv(crate::hooks::HEAP_FREE, &[block]);
    }
}

impl black_script::hooks::CommandDispatch for Env {
    fn command(&mut self, handler: u32, vm: u32, ctx: u32) -> u32 {
        self.cv(handler, &[vm, ctx])
    }
}

impl black_script::hooks::VmHooks for Env {
    fn call_wait(&mut self, f: u32, vm: u32, ctx: u32) -> u32 {
        self.cv(f, &[vm, ctx])
    }
    fn call_hook(&mut self, f: u32, vm: u32, ctx: u32, arg: u32, cmd: u32) -> u32 {
        self.cv(f, &[vm, ctx, arg, cmd])
    }
}

impl Env {
    pub(super) fn call_end_callback_impl(&mut self, f: u32, x: u32, arg: u32) -> u32 {
        self.cv(f, &[x, arg])
    }
    pub(super) fn call_init_proc_impl(&mut self, f: u32, task: u32, seq: u32) -> u32 {
        self.cv(f, &[task, seq])
    }
    pub(super) fn fun_02045924_str_impl(&mut self, buf: u32, s: &[u8; 2]) {
        let mut a = Args::new();
        a.v(buf);
        // the stack string: the character and its terminator slot
        a.inb(&[s[0], s[1], 0xFF, 0xFF]);
        self.call_args(0x0204_5924, &mut a);
    }
    pub(super) fn call_wordset_fn_impl(&mut self, f: u32, ws: u32, idx: u32, v: u32) {
        self.cv(f, &[ws, idx, v]);
    }
    pub(super) fn fun_02045994_at_impl(&mut self, buf: u32, src: u32, n: u32) {
        self.cv(0x0204_5994, &[buf, src, n]);
    }
    pub(super) fn call_proc_callback_impl(&mut self, cb: u32, w: u32) {
        self.cv(cb, &[w]);
    }
    pub(super) fn fun_020124ac_at_impl(
        &mut self,
        gs: u32,
        kind: u32,
        proc_table: u32,
        param: u32,
    ) -> u32 {
        self.cv(0x0201_24AC, &[gs, kind, proc_table, param])
    }
}
