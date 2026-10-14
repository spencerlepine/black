//! B16: end-to-end battle integration test (test-only).
//!
//! One battle runs headless through the ported battle code: `btl_main` / `btl_server` (server1-3),
//! `btl_server_flow` (cmd_queue, server_side, events, core1-6, hem, handex, api), `btl_client`
//! (client1-5; the AI client through ai1), the adapter (D7 command stream) and the event handlers
//! through the production [`crate::server_flow::host::FlowWorkHost`].
//!
//! # Dispatch
//!
//! [`Env`] is the [`FlowEnv`] every port gets. `env.call(addr, args)` goes to [`Env::dispatch`],
//! which maps `addr` to the Rust port of that address (a table over the ported areas, see
//! `dispatch.rs`) or to an explicit host model (ARM9 main: heap, Pokemon data, strings; the battle
//! view / sound / view objects; overlay 95; battle_net), each in its own module. An address with
//! neither panics with the address and the call stack, so every gap is visible.
//!
//! # Memory and coherence
//!
//! The ports keep the original's pointers, so the integration keeps an address space
//! ([`objects::Objects`]): every object is a byte block at an address. The server / AI ports read
//! it through [`crate::server::MemExt`] (always coherent); the flow ports work on an `SvFlowWork`
//! and the client / server-side ports on byte slices, which the dispatcher *takes* from the table
//! while the port runs. A callee that needs an object a caller up the stack holds gets it through
//! the `Lent` of the call (`call_w` / `call_wr` / `call_held` / `Lend`); without it the access
//! panics ("held by a port frame and was not lent"), which marks a call site to fix.
//!
//! The battle state the handlers share with the flow (event variables, RNG, battle Pokemon) lives
//! in [`Env`] itself and reaches the handlers through the `ext_*` hooks of `FlowHost`.

#![allow(dead_code)]

pub mod adapt;
pub mod ai;
pub mod arm9;
pub mod bpp;
pub mod calc;
pub mod client;
pub mod data;
pub mod efactor;
pub mod field;
pub mod flowcase;
pub mod net;
pub mod objects;
pub mod ov95;
pub mod recb;
pub mod rom;
pub mod server;
pub mod sick;
pub mod side;
pub mod thumb;
pub mod unported;
pub mod view;

use crate::battle_mon::BattleMon;
use crate::calc::BattleCalc;
use crate::event_factor::EventSystem;
use crate::event_var::EventVarStack;
use crate::field::Field;
use crate::server_flow::env::{Arg, FlowEnv, Lent};
use crate::server_flow::SvFlowWork;
use objects::{Objects, Shared};
use rom::Rom;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// The integration environment.
pub struct Env {
    pub rom: Rc<Rom>,
    pub arc: Rc<data::Archives>,
    pub objs: Rc<RefCell<Objects>>,
    /// `btl_field.c` instances by address (the global one at [`field::FIELD_GLOBAL`]).
    pub fields: Rc<RefCell<BTreeMap<u32, Field>>>,
    pub vars: EventVarStack,
    pub calc: BattleCalc,
    /// Battle Pokemon by address.
    pub mons: BTreeMap<u32, BattleMon>,
    /// The event-factor system (taken while a dispatch runs).
    pub events: Option<EventSystem>,
    /// The snapshot a nested dispatch inside a running handler works on.
    pub events_shadow: Option<EventSystem>,
    /// Handlers running (innermost last), for diagnostics.
    pub handler_stack: Vec<u32>,
    /// The overlay-95 side-effect layer array (0x0689E7E0).
    pub side_effects: crate::handlers::effects::SideEffects,
    /// Address of the flow work, once created.
    pub waddr: u32,
    /// Address of the main module M, once created.
    pub main_module: u32,
    /// The BATTLE_SETUP.
    pub setup: u32,
    /// Temporary copies a port works on that stand for a part of an object (host pointer, length,
    /// address): the command queue inside the server object S, handed to the ports that also take
    /// S as a separate slice.
    pub aliases: Vec<(usize, usize, u32)>,
    /// Parsed svflow specs by area.
    pub specs:
        std::collections::HashMap<&'static str, std::collections::HashMap<u32, flowcase::FnSpec>>,
    /// The interpreter's stack pointer while it is calling out (nested runs continue below it).
    pub thumb_sp: Option<u32>,
    /// Open archive handles -> ARC id.
    pub arc_handles: BTreeMap<u32, u32>,
    /// The move-record cache exists (`0201C328`).
    pub waza_cache: bool,
    /// UI calls that request something, in order (the headless view's log).
    pub ui_log: Vec<u32>,
    /// The open move menu: (action record, battle Pokemon).
    pub ui_fight: Option<(u32, u32)>,
    /// The moves the scripted player chose.
    pub choices: Vec<u32>,
    /// The scripted player's move slot for each move-menu answer, in order; the last one repeats
    /// (B19; `[0]` = the B16 script: slot 0 every turn).
    pub script: Vec<u32>,
    /// The scripted player's party slot for each party-list opening (`021e9a78`), in order; the
    /// last one repeats (B19).
    pub switch_script: Vec<u32>,
    /// The party slots the scripted player chose.
    pub switches: Vec<u32>,
    /// Coherence notes (distinct messages, with counts).
    pub notes: BTreeMap<String, u32>,
    /// The OS tick (`OS_GetTick`).
    pub tick: u64,
    /// Calls of overlay-96 (AI) functions from outside the overlay, by address (B17).
    pub ai_calls: BTreeMap<u32, u32>,
    /// The AI mask `021b9fbc` answers for the AI client (B17 test knob; `None` = the setup's).
    pub ai_mask: Option<u32>,
    /// Run overlay 96 as original Thumb code in the interpreter instead of the ai1 ports, except
    /// the entry points create / start / chosen move / chosen target (B17 cross-check).
    pub ai_interp: bool,
    /// The image of each read-only holder at its last flush, keyed by the holder's frame depth and
    /// the image's place `(depth, block, offset, len)` (not by its host address: an allocator that
    /// reuses a freed image's address must not hand its snapshot to a later image); a frame's
    /// entries are dropped when it returns. A read-only holder cannot take a callee's writes back,
    /// so a later flush stores only the bytes its image changed since then (changes an outer
    /// writable holder made) and keeps the callee's.
    pub ro_snap: BTreeMap<(usize, u32, usize, usize), Vec<u8>>,
    /// B19: the flow-work image a call that lent the work writable handed back to the port frame
    /// at depth `d` (the object table holds exactly that image then). A flow case merges the
    /// port's final image against it (bytes the port changed since then are the port's, others
    /// the table's), not against the image it started with: a port that changed a byte, lent the
    /// work, and changed it back (the turn-end state `work[0x77E]` 0 -> 7 -> 0) left an echo of
    /// its own intermediate value in the table that the old merge kept.
    pub work_sync: BTreeMap<usize, Vec<u8>>,
    /// The heap the battle calc's blocks come from (`021d7854` / `021d7894`).
    pub calc_heap: black_core::heap::HostHeap,
    /// The command frames the server sent (message 8 of the adapter), in order.
    pub cmd_frames: Vec<Vec<u8>>,
    /// Call trace switch (diagnostics): > 0 prints every dispatched call.
    pub trace: u32,
    /// The system Mersenne Twister (`*0x021469F8`, `0203F0A8`), seeded from [`Self::seed`].
    pub mt: black_core::rng::Mt19937,
    /// Save blocks by (save, id) (`020071CC`).
    pub save_blocks: BTreeMap<(u32, u32), u32>,
    /// A block a port's caller pre-allocated for the next `Heap_AllocDebug` of that size.
    pub pending_alloc: Option<(u32, u32)>,
    /// Seed of the battle RNG context (0203F258).
    pub seed: u64,
    /// Client objects whose ports are running (innermost last).
    pub client_stack: Vec<u32>,
    /// Call stack (callee addresses) for diagnostics.
    pub stack: Vec<u32>,
    /// Calls per address.
    pub counts: BTreeMap<u32, u64>,
}

impl Env {
    pub fn new(rom: Rc<Rom>) -> Self {
        let mut objs = Objects::new();
        // overlay-93 globals
        objs.map(adapt::TABLE, vec![0; 0x14], "adapter slot table 0x021F6384");
        objs.map(
            0x021F_6398,
            vec![0; 0x60],
            "message printer context 0x021F6398",
        );
        objs.map(
            0x021F_3B20,
            vec![0; 0x64],
            "flow scratch globals 0x021F3B20",
        );
        objs.map(
            0x0218_A7C0,
            vec![0; 0x40],
            "overlay 96 BSS (AI table pointer)",
        );
        let mut fields = BTreeMap::new();
        fields.insert(field::FIELD_GLOBAL, Field::new());
        Env {
            arc: Rc::new(data::Archives::load(&rom)),
            rom,
            objs: Rc::new(RefCell::new(objs)),
            fields: Rc::new(RefCell::new(fields)),
            vars: EventVarStack::new(),
            calc: BattleCalc::new(),
            mons: BTreeMap::new(),
            events: Some(EventSystem::new()),
            events_shadow: None,
            handler_stack: Vec::new(),
            side_effects: Default::default(),
            waddr: 0,
            main_module: 0,
            seed: 0x0123_4567_89AB_CDEF,
            setup: 0,
            pending_alloc: None,
            save_blocks: BTreeMap::new(),
            aliases: Vec::new(),
            specs: std::collections::HashMap::new(),
            thumb_sp: None,
            tick: 0,
            ai_calls: BTreeMap::new(),
            ai_mask: None,
            ai_interp: false,
            trace: 0,
            cmd_frames: Vec::new(),
            calc_heap: Default::default(),
            ro_snap: BTreeMap::new(),
            work_sync: BTreeMap::new(),
            mt: {
                let mut mt = black_core::rng::Mt19937::new();
                mt.seed(0x89AB_CDEF);
                mt
            },
            notes: BTreeMap::new(),
            ui_log: Vec::new(),
            ui_fight: None,
            choices: Vec::new(),
            script: vec![0],
            switch_script: vec![1],
            switches: Vec::new(),
            waza_cache: false,
            arc_handles: BTreeMap::new(),
            client_stack: Vec::new(),
            stack: Vec::new(),
            counts: BTreeMap::new(),
        }
    }

    /// The shared-state handles for code that cannot borrow the environment.
    pub fn unp(&self) -> data::Unp {
        data::Unp {
            msg: vec![0; 0x80],
            objs: self.objs.clone(),
            fields: self.fields.clone(),
            arc: self.arc.clone(),
        }
    }

    /// The event-factor system; panics while a dispatch holds it (a nested dispatch).
    pub fn events_mut(&mut self) -> &mut EventSystem {
        if self.events.is_none() && self.events_shadow.is_none() {
            panic!("B16: no event system ({})", self.backtrace());
        }
        match &mut self.events {
            Some(e) => e,
            None => self.events_shadow.as_mut().unwrap(),
        }
    }

    /// The BattleMon address of `poke_id` (pokecon `M + 0xC8`, the one the server flow uses:
    /// `pc[0x84 + 4 * id]`).
    pub fn mon_addr_by_id(&self, poke_id: u32) -> u32 {
        if poke_id >= 24 {
            return 0;
        }
        let m = self.main_module;
        assert!(m != 0, "B16: main module not created yet");
        self.r32(m + 0xC8 + 0x84 + 4 * poke_id)
    }

    /// A `server` port memory view over the object table.
    pub fn mem(&self) -> crate::server::Mem<'static> {
        crate::server::Mem::with_ext(Rc::new(Shared(self.objs.clone())))
    }

    /// A `server3` / `ai` port memory view over the object table (plus the ROM tables).
    pub fn mem3(&self) -> crate::server::server3::Mem {
        crate::server::server3::Mem::with_ext(Rc::new(Shared(self.objs.clone())))
    }

    pub fn r32(&self, addr: u32) -> u32 {
        self.objs.borrow_mut().r32(addr)
    }

    pub fn w32(&self, addr: u32, v: u32) {
        self.objs.borrow_mut().w32(addr, v)
    }

    /// Records a coherence note (reported at the end of the test).
    pub fn note(&mut self, s: String) {
        *self.notes.entry(s).or_default() += 1;
    }

    /// The call stack as text.
    pub fn backtrace(&self) -> String {
        self.stack
            .iter()
            .map(|a| format!("{a:08x}"))
            .collect::<Vec<_>>()
            .join(" > ")
    }

    /// Runs `f` on the flow work (taken from the table for the duration).
    pub fn with_work<R>(
        &mut self,
        _lent: &mut Lent<'_>,
        f: impl FnOnce(&mut Env, &mut SvFlowWork) -> R,
    ) -> R {
        assert!(self.waddr != 0, "B16: flow work used before it was created");
        let base = self.waddr;
        let v = self.objs.borrow_mut().take(base);
        let mut w = SvFlowWork {
            mem: v.into_boxed_slice().try_into().expect("flow work size"),
        };
        let r = f(self, &mut w);
        let v: Box<[u8]> = w.mem;
        self.objs.borrow_mut().put(base, v.into_vec());
        r
    }

    /// Runs `f` on a snapshot of the flow work (for ports that take it read-only; the table keeps
    /// it, so callees see it).
    pub fn with_work_ref<R>(&mut self, f: impl FnOnce(&mut Env, &SvFlowWork) -> R) -> R {
        let base = self.waddr;
        let mut v = vec![0u8; SvFlowWork::SIZE];
        self.objs.borrow_mut().read(base, &mut v);
        let w = SvFlowWork {
            mem: v.into_boxed_slice().try_into().unwrap(),
        };
        f(self, &w)
    }

    /// Runs `f` on the bytes of the object at `addr` to the end of its block (taken from the
    /// table for the duration).
    pub fn with_obj<R>(
        &mut self,
        _lent: &mut Lent<'_>,
        addr: u32,
        f: impl FnOnce(&mut Env, &mut [u8]) -> R,
    ) -> R {
        let base = self.objs.borrow().base_of(addr).unwrap_or_else(|| {
            panic!(
                "B16: object at unmapped address {addr:#010x} ({})",
                self.backtrace()
            )
        });
        let mut v = self.objs.borrow_mut().take(base);
        let o = (addr - base) as usize;
        let r = f(self, &mut v[o..]);
        // a port may free its own object (`021cd8e0`, the client's destructor)
        if self.objs.borrow().blocks.contains_key(&base) {
            self.objs.borrow_mut().put(base, v);
        }
        r
    }

    /// A snapshot of the object at `addr` to the end of its block (for read-only ports).
    pub fn obj_copy(&self, addr: u32) -> Vec<u8> {
        let o = self.objs.borrow();
        let (base, blk) = o
            .find_block(addr, 1)
            .unwrap_or_else(|| panic!("B16: object at unmapped address {addr:#010x}"));
        assert!(
            !blk.held,
            "B16: {} is held by a port frame and was not lent ({})",
            blk.name,
            self.backtrace()
        );
        blk.data[(addr - base) as usize..].to_vec()
    }

    /// Entry point of every call. The objects the caller lends are flushed into the object table
    /// for the duration of the call (so the callee and everything it calls see the caller's
    /// images) and reloaded into the caller's buffers afterwards; `PHost` arguments become
    /// addresses first.
    pub fn dispatch(&mut self, lent: Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.stack.push(addr);
        // frame depth of the caller (the holder of the lent images)
        let depth = self.stack.len() - 1;
        *self.counts.entry(addr).or_default() += 1;
        for a in args.iter_mut() {
            if let Arg::PHost(h) = *a {
                let p = self.host_addr(&lent, h);
                *a = Arg::PAt(p);
            }
        }
        let Lent {
            mut work,
            work_ref,
            mut bytes,
            ro,
        } = lent;
        // (base, offset) of each lent image (byte slices first, then the work)
        let mut locs: Vec<(u32, usize)> = Vec::new();
        // the hold state of every block a lent image is in, restored after the call
        let mut saved: Vec<(u32, bool, usize, u32)> = Vec::new();
        let mut ro_locs: Vec<(u32, usize, usize)> = Vec::new();
        let mut ro_work_before: Option<Vec<u8>> = None;
        let mut ro_imgs: Vec<Vec<u8>> = Vec::new();
        {
            let mut o = self.objs.borrow_mut();
            // a lent slice that is not (part of) a held object or a registered alias (a caller's
            // local buffer) needs no flush
            let aliases = &self.aliases;
            let find = |o: &Objects, b: &[u8]| -> Option<(u32, usize)> {
                let p = b.as_ptr() as usize;
                if let Some(r) = o.held_block(p, b.len()) {
                    return Some(r);
                }
                aliases.iter().find_map(|&(ap, alen, addr)| {
                    (p >= ap && p + b.len() <= ap + alen).then(|| {
                        let a = addr + (p - ap) as u32;
                        let base = o.base_of(a).expect("alias of an unmapped address");
                        (base, (a - base) as usize)
                    })
                })
            };
            bytes.retain(|b| find(&o, b).is_some());
            for b in bytes.iter() {
                locs.push(find(&o, b).unwrap());
            }
            if work.is_some() || work_ref.is_some() {
                locs.push((self.waddr, 0));
            }
            // read-only images: flushed like the others, checked after the call
            let ro: Vec<&[u8]> = ro.into_iter().filter(|b| find(&o, b).is_some()).collect();
            for b in ro.iter() {
                let (base, off) = find(&o, b).unwrap();
                let blk = o.blocks.get_mut(&base).unwrap();
                ro_flush(
                    &mut self.ro_snap,
                    (depth, base, off, b.len()),
                    &mut blk.data[off..off + b.len()],
                    b,
                    true,
                );
                ro_locs.push((base, off, b.len()));
            }
            for (k, &(base, off)) in locs.iter().enumerate() {
                let (data, read_only): (&[u8], bool) = if k < bytes.len() {
                    (&bytes[k][..], false)
                } else if let Some(w) = work.as_deref() {
                    (&w.mem[..], false)
                } else {
                    (&work_ref.unwrap().mem[..], true)
                };
                let blk = o.blocks.get_mut(&base).unwrap();
                ro_flush(
                    &mut self.ro_snap,
                    (depth, base, off, data.len()),
                    &mut blk.data[off..off + data.len()],
                    data,
                    read_only,
                );
                if read_only {
                    ro_work_before = Some(blk.data[..SvFlowWork::SIZE].to_vec());
                }
                if !saved.iter().any(|x| x.0 == base) {
                    saved.push((base, blk.held, blk.held_ptr, blk.alias_holds));
                }
            }
            for &(base, off, n) in &ro_locs {
                let blk = o.blocks.get_mut(&base).unwrap();
                if !saved.iter().any(|x| x.0 == base) {
                    saved.push((base, blk.held, blk.held_ptr, blk.alias_holds));
                }
                ro_imgs.push(blk.data[off..off + n].to_vec());
            }
            for &(base, ..) in &saved {
                let blk = o.blocks.get_mut(&base).unwrap();
                blk.held = false;
                blk.held_ptr = 0;
                blk.alias_holds = 0;
            }
        }
        let r = match dispatch::call(self, &mut Lent::default(), addr, args) {
            Some(r) => r,
            None => panic!(
                "B16 gap: no port or host model for callee {addr:#010x} (stack {})",
                self.backtrace()
            ),
        };
        // the callee's frames have ended: their read-only snapshots go
        self.ro_snap.retain(|k, _| k.0 <= depth);
        self.work_sync.retain(|&k, _| k <= depth);
        let mut ro_note = None;
        let mut synced = None;
        {
            let mut o = self.objs.borrow_mut();
            let nb = bytes.len();
            for (k, &(base, off)) in locs.iter().enumerate() {
                // a callee that frees the object its caller holds (the caller's destructor frees
                // its own object): nothing to reload
                let Some(blk) = o.blocks.get(&base) else {
                    continue;
                };
                assert!(
                    !blk.held && blk.alias_holds == 0,
                    "B16: a lent object ({}) is still held after the call to {addr:#x}",
                    blk.name
                );
                if k < nb {
                    let n = bytes[k].len();
                    bytes[k].copy_from_slice(&blk.data[off..off + n]);
                } else if let Some(w) = work.as_deref_mut() {
                    w.mem.copy_from_slice(&blk.data[..SvFlowWork::SIZE]);
                    synced = Some(blk.data[..SvFlowWork::SIZE].to_vec());
                } else if work_ref.is_some() {
                    let before = ro_work_before.as_ref().unwrap();
                    if before[..] != blk.data[..SvFlowWork::SIZE] {
                        let at = (0..SvFlowWork::SIZE)
                            .find(|&i| before[i] != blk.data[i])
                            .unwrap();
                        let caller = self
                            .stack
                            .get(self.stack.len().wrapping_sub(2))
                            .copied()
                            .unwrap_or(0);
                        ro_note = Some(format!(
                            "callee {addr:#x} wrote the flow work (+{at:#x}) that its caller {caller:#x} holds read-only"
                        ));
                    }
                }
            }
            for (k, &(base, off, _)) in ro_locs.iter().enumerate() {
                let Some(blk) = o.blocks.get(&base) else {
                    continue;
                };
                let n = ro_imgs[k].len();
                if blk.data[off..off + n] != ro_imgs[k][..] {
                    let at = (0..n)
                        .find(|&i| blk.data[off + i] != ro_imgs[k][i])
                        .unwrap();
                    ro_note = Some(format!(
                        "callee {addr:#x} wrote {} (+{:#x}) that its caller holds read-only",
                        blk.name,
                        off + at
                    ));
                }
            }
            for &(base, held, ptr, ah) in &saved {
                let Some(blk) = o.blocks.get_mut(&base) else {
                    continue;
                };
                blk.held = held;
                blk.held_ptr = ptr;
                blk.alias_holds = ah;
            }
        }
        if let Some(n) = ro_note {
            self.note(n);
        }
        if let Some(s) = synced {
            self.work_sync.insert(depth, s);
        }
        if self.trace > 0 {
            let a: Vec<String> = args
                .iter()
                .map(|x| match x {
                    Arg::V(v) | Arg::PAt(v) => format!("{v:x}"),
                    Arg::P => "P".into(),
                    _ => "buf".into(),
                })
                .collect();
            eprintln!(
                "{:w$}{addr:08x}({}) = {r:x}",
                "",
                a.join(","),
                w = 2 * self.stack.len()
            );
        }
        self.stack.pop();
        r
    }
}

impl FlowEnv for Env {
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.dispatch(Lent::default(), addr, args)
    }

    fn call_lent(&mut self, lent: Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.dispatch(lent, addr, args)
    }

    fn peek(&mut self, addr: u32, out: &mut [u8]) -> bool {
        // memory a port frame holds is the port's own image (it keeps its model)
        let held = {
            let o = self.objs.borrow();
            o.base_of(addr)
                .is_some_and(|b| o.blocks[&b].held || o.blocks[&b].alias_holds > 0)
        };
        if held {
            return false;
        }
        self.objs.borrow_mut().read(addr, out);
        true
    }

    fn ext_vars(&self) -> Option<&EventVarStack> {
        Some(&self.vars)
    }

    fn ext_vars_mut(&mut self) -> Option<&mut EventVarStack> {
        Some(&mut self.vars)
    }

    fn ext_calc_mut(&mut self) -> Option<&mut BattleCalc> {
        Some(&mut self.calc)
    }

    fn ext_mon_at(&mut self, addr: u32) -> Option<&mut BattleMon> {
        self.mons.get_mut(&addr)
    }
}

/// Buffer arguments of a call: a caller's slice (`In` / `Out` / `PBuf`) or an address (`V`): in a
/// lent object, else in the object table (copied in and back around the port).
pub struct Bufs<'a, 'b> {
    pub args: &'a mut [Arg<'b>],
}

impl<'a, 'b> Bufs<'a, 'b> {
    pub fn new(args: &'a mut [Arg<'b>]) -> Self {
        Bufs { args }
    }

    /// The bytes of argument `i` (at most `len` for an address, bounded by its block).
    pub fn get(&mut self, env: &Env, lent: &mut Lent<'_>, i: usize, len: usize) -> Vec<u8> {
        match &self.args[i] {
            Arg::In(b) => b.to_vec(),
            Arg::Out(b) | Arg::PBuf(b) => b.to_vec(),
            Arg::V(a) | Arg::PAt(a) => env.read_vec(lent, *a, len),
            Arg::PHost(h) => {
                let a = env.host_addr(lent, *h);
                env.read_vec(lent, a, len)
            }
            Arg::P => panic!(
                "B16: buffer argument {i} passed through (P) ({})",
                env.backtrace()
            ),
        }
    }

    /// Stores `data` back into argument `i`.
    pub fn put(&mut self, env: &Env, lent: &mut Lent<'_>, i: usize, data: &[u8]) {
        match &mut self.args[i] {
            Arg::Out(b) | Arg::PBuf(b) => {
                let n = b.len().min(data.len());
                b[..n].copy_from_slice(&data[..n]);
            }
            Arg::In(_) => {}
            Arg::V(a) | Arg::PAt(a) => env.write_bytes(lent, *a, data),
            Arg::PHost(h) => {
                let a = env.host_addr(lent, *h);
                env.write_bytes(lent, a, data)
            }
            Arg::P => unreachable!(),
        }
    }

    /// Runs `f` on argument `i` as a buffer.
    pub fn with<R>(
        &mut self,
        env: &mut Env,
        lent: &mut Lent<'_>,
        i: usize,
        len: usize,
        f: impl FnOnce(&mut Env, &mut [u8]) -> R,
    ) -> R {
        let mut b = self.get(env, lent, i, len);
        let r = f(env, &mut b);
        self.put(env, lent, i, &b);
        r
    }
}

impl Env {
    /// The address of a [`Arg::PHost`] pointer: the held block (or lent work) containing it.
    pub fn host_addr(&self, lent: &Lent<'_>, h: usize) -> u32 {
        if let Some(w) = lent.work.as_deref().or(lent.work_ref) {
            let p = w.mem.as_ptr() as usize;
            if h >= p && h < p + SvFlowWork::SIZE {
                return self.waddr + (h - p) as u32;
            }
        }
        if let Some(a) = self.objs.borrow().held_addr(h, 0) {
            return a;
        }
        // a buffer a flow case built from a known address
        self.aliases
            .iter()
            .rev()
            .find_map(|&(ap, alen, addr)| {
                (h >= ap && h < ap + alen).then(|| addr + (h - ap) as u32)
            })
            .unwrap_or_else(|| {
                panic!(
                    "B16: PHost pointer outside every held object ({})",
                    self.backtrace()
                )
            })
    }

    /// The word at `addr`.
    pub fn read_u32(&self, _lent: &mut Lent<'_>, addr: u32) -> u32 {
        self.r32(addr)
    }

    /// Reads up to `len` bytes at `addr` (bounded by the object).
    pub fn read_vec(&self, _lent: &mut Lent<'_>, addr: u32, len: usize) -> Vec<u8> {
        let mut ob = self.objs.borrow_mut();
        let (base, blk) = ob
            .find_block(addr, 1)
            .unwrap_or_else(|| panic!("B16: buffer at unmapped {addr:#x} ({})", self.backtrace()));
        let n = len.min(blk.len - (addr - base) as usize);
        let mut v = vec![0; n];
        ob.read(addr, &mut v);
        v
    }

    /// Writes `data` at `addr`.
    pub fn write_bytes(&self, _lent: &mut Lent<'_>, addr: u32, data: &[u8]) {
        self.objs.borrow_mut().write(addr, data);
    }
}

/// Argument `i` as a value (a `P` argument must be resolved by the adapter).
pub fn val(args: &[Arg<'_>], i: usize) -> u32 {
    match args.get(i) {
        Some(Arg::V(v)) | Some(Arg::PAt(v)) => *v,
        Some(Arg::P) => panic!("B16: argument {i} is a pass-through pointer (resolve it)"),
        Some(_) => panic!("B16: argument {i} is a buffer"),
        None => 0,
    }
}

#[cfg(test)]
mod b19;
pub mod dispatch;
#[cfg(test)]
mod tests;

/// The battle Pokemon's `POKEMON_PARAM` side for ports that take a `BppEnv` (the events area):
/// the ROM-data hooks ([`data::Unp`]).
macro_rules! bpp_forward {
    ($t:ty) => {
        impl crate::battle_mon::BppEnv for $t {
            fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
                self.unp().pp_get(pp, id)
            }
            fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
                self.unp().pp_put(pp, id, value)
            }
            fn pp_fast_begin(&mut self, pp: u32) -> u32 {
                self.unp().pp_fast_begin(pp)
            }
            fn pp_fast_end(&mut self, pp: u32, token: u32) {
                self.unp().pp_fast_end(pp, token)
            }
            fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
                self.unp().pp_set_move(pp, mv, slot)
            }
            fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
                self.unp().pp_fun_02017f00(pp)
            }
            fn pp_status(&mut self, pp: u32) -> u32 {
                self.unp().pp_status(pp)
            }
            fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
                self.unp().move_max_pp(mv, pp_ups)
            }
            fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
                self.unp().personal(species, form, param)
            }
            fn field_effect_active(&mut self, id: u32) -> bool {
                self.unp().field_effect_active(id)
            }
            fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
                self.unp().exp_for_level(species, form, level)
            }
            fn pp_recalc_stats(&mut self, pp: u32) {
                self.unp().pp_recalc_stats(pp)
            }
            fn pp_set_status(&mut self, pp: u32, status: u32) {
                self.unp().pp_set_status(pp, status)
            }
            fn pp_set_form(&mut self, pp: u32, form: u32) {
                self.unp().pp_set_form(pp, form)
            }
            fn fun_0202142c(&mut self, value: u32) -> u32 {
                self.unp().fun_0202142c(value)
            }
            fn sick_is_passable(&mut self, sick: u32) -> bool {
                self.unp().sick_is_passable(sick)
            }
        }
    };
}
bpp_forward!(Env);
bpp_forward!(&mut Env);

/// Flushes a lent image into its block: writable images entirely; read-only ones only where they
/// changed since their last flush (see [`Env::ro_snap`]).
fn ro_flush(
    snap: &mut BTreeMap<(usize, u32, usize, usize), Vec<u8>>,
    key: (usize, u32, usize, usize),
    dst: &mut [u8],
    img: &[u8],
    read_only: bool,
) {
    if !read_only {
        dst.copy_from_slice(img);
        return;
    }
    match snap.get(&key) {
        Some(prev) => {
            for i in 0..img.len() {
                if img[i] != prev[i] {
                    dst[i] = img[i];
                }
            }
        }
        None => dst.copy_from_slice(img),
    }
    snap.insert(key, img.to_vec());
}
