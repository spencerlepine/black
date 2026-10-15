//! F12: field integration test (test-only, ROM-gated: skips without the ROM build unless `REQUIRE_ROM` is
//! set). A real map of the ROM runs headless through the ported field code, frame by frame, with
//! scripted key input: see `session.rs` for the session and the trace.
//!
//! # Machine
//!
//! [`Env`] is one DS-like address space: ARM9 main (`main.sbin` at 0x02004000 plus its autoload blocks),
//! the field overlays at their link addresses (loaded and unloaded as the game asks), main RAM
//! 0x02000000..0x02E00000 (the host heap at 0x02500000), the interpreter stack below 0x02F00000, the
//! `Mem::push_local` windows at 0x02EC0000, ITCM / DTCM and the I/O page. The ported code (black-field
//! `FieldEnv` ports, black-script hook ports) reads and writes it through its `Mem` window; the
//! original code runs in the ARMv5TE interpreter [`cpu`] over the same memory.
//!
//! # Dispatch
//!
//! Every call goes by address ([`Env::call_words`]): a host model (`host.rs`: the system boundary — heap,
//! file system, overlay loader, keys, clock, sound, the D3 render side) or a Rust port (`ports.rs`,
//! generated from the verified `PORTED` lists of black-field / black-script) when there is one for that
//! address in a loaded module, else the original code in the interpreter. The interpreter enters a
//! hooked address (host model or port) the same way, so original code calling a ported function runs the
//! port. Nothing is guessed: an address with neither a port nor loaded original code panics.
//!
//! # Memory views
//!
//! black-field and black-script each have their own `Mem` type (same layout); [`Env`] keeps the bytes
//! in whichever view the running code asked for last (`fmem()` / `smem()` move the windows over, O(1)).

#![allow(dead_code)]

pub mod args;
pub mod cpu;
pub mod host;
pub mod ports;
pub mod rom;
pub mod script;
pub mod script_hooks;
mod session;
pub mod view;

use crate::hooks::{CArg, FieldEnv, Mem as FMem};
use args::Args;
use black_script::hooks::Mem as SMem;
use cpu::{Bus, Cpu};
use rom::Rom;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Return-address sentinel of a top-level interpreted call (as in `tools/rom-diff/emu.py`).
pub const RET: u32 = 0x02FF_F000;
/// Main RAM as the integration maps it (the DS has 4 MB; the host heap lives above).
pub const MAIN_LO: u32 = 0x0200_0000;
pub const MAIN_SIZE: u32 = 0x00E0_0000;
/// The host heap (`host.rs` `HostHeap`): one arena per gfl heap, carved out of this root; zeroed blocks,
/// 16-byte gaps, first fit over the freed spans.
pub const HEAP_BASE: u32 = 0x0250_0000;
pub const HEAP_END: u32 = 0x02E0_0000;
/// The interpreter stack (also the stack of the buffers a port passes by pointer).
pub const STACK_TOP: u32 = 0x02F0_0000;
pub const STACK_LO: u32 = 0x02ED_0000;
pub const ITCM: u32 = 0x01FF_8000;
pub const DTCM: u32 = 0x02FE_0000;
/// The interpreter steps one top-level call may take.
const STEP_LIMIT: u64 = 50_000_000;
/// Hooked addresses are code of ITCM / main / the overlays: the bitmap covers this range.
const HOOK_LO: u32 = 0x01FF_8000;
const HOOK_HI: u32 = 0x0220_0000;

/// The overlays that make up the field set (`tools/re/sets.txt`: main 10 11 18 20 21).
pub const FIELD_OVERLAYS: [u16; 5] = [10, 11, 18, 20, 21];

/// Hex addresses from the environment variable `k` (comma separated; diagnostics).
fn env_list(k: &str) -> Vec<u32> {
    std::env::var(k)
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok())
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hook {
    Host,
    Port,
    /// Original code entered through `call_words` so that `F12_TRACE` sees it (diagnostics).
    Trace,
}

/// Counters for the report.
#[derive(Default, Debug)]
pub struct Stats {
    /// Calls of each port.
    pub ports: BTreeMap<u32, u64>,
    /// Calls of each host model.
    pub hosts: BTreeMap<u32, u64>,
    /// Top-level entries into original code, by address.
    pub interp: BTreeMap<u32, u64>,
    pub steps: u64,
}

/// The integration environment (one machine).
pub struct Env {
    pub rom: Rom,
    fmem: FMem,
    smem: SMem,
    script_owns: bool,
    pub loaded: BTreeSet<u16>,
    hook_bits: Vec<u64>,
    hooks: HashMap<u32, Hook>,
    /// Interpreter stack pointer for the next nested call.
    pub sp: u32,
    pub host: host::HostState,
    pub stats: Stats,
    /// Entry addresses of the running calls (diagnostics).
    pub calls: Vec<u32>,
    /// The last interpreted instruction and its lr (diagnostics).
    pub pc: u32,
    pub lr: u32,
    pub isp: u32,
    /// The 4 KB page of the last instruction checked to be loaded code.
    code_page: u32,
    /// Words whose writes are logged (`F12_WATCH=addr,..`; diagnostics).
    watch: Vec<u32>,
    /// Calls whose arguments / result are logged (`F12_TRACE=addr,..`; only calls through `call_words`: ports,
    /// host models, interpreted entries from Rust or into hooks).
    trace: Vec<u32>,
    /// The entry address of the interpreted call starting now (its own hook is not taken).
    skip_hook: Option<u32>,
}

impl Env {
    /// The machine with ARM9 main and the field overlays 10 / 11 loaded (the others load on demand); `None`
    /// without the ROM build.
    pub fn new() -> Option<Env> {
        let rom = Rom::load()?;
        let mut m = FMem::new(MAIN_LO, vec![0; MAIN_SIZE as usize]);
        m.add_window(ITCM, vec![0; 0x8000]);
        m.add_window(DTCM, vec![0; 0x4000]);
        m.add_window(STACK_LO, vec![0; (STACK_TOP - STACK_LO) as usize]);
        // the shared work area at the top of main RAM (boot / ARM7 system data: zeros here)
        m.add_window(0x02FF_F000, vec![0; 0x1000]);
        // video memory as plain memory (D3: nothing reads it back for the logic): palettes, VRAM (the BG / OBJ
        // engine ranges and LCDC), OAM
        m.add_window(0x0500_0000, vec![0; 0x800]);
        m.add_window(0x0600_0000, vec![0; 0x8_0000]);
        m.add_window(0x0620_0000, vec![0; 0x2_0000]);
        m.add_window(0x0640_0000, vec![0; 0x4_0000]);
        m.add_window(0x0660_0000, vec![0; 0x2_0000]);
        m.add_window(0x0680_0000, vec![0; 0xA_4000]);
        m.add_window(0x0700_0000, vec![0; 0x800]);
        // the I/O page (host::io_read / io_write give it its behaviour) and the IPC FIFO receive register
        m.add_window(0x0400_0000, vec![0; 0x2000]);
        m.add_window(0x0410_0000, vec![0; 0x20]);
        // the DSi register page (reads 0 on a DS)
        m.add_window(0x0400_4000, vec![0; 0x1000]);
        m.write(rom::MAIN_BASE, &rom.main);
        for (dst, bytes, _bss) in &rom.autoload {
            m.write(*dst, bytes);
        }
        let mut env = Env {
            rom,
            fmem: m,
            smem: SMem::default(),
            script_owns: false,
            loaded: BTreeSet::new(),
            hook_bits: vec![0; ((HOOK_HI - HOOK_LO) / 2 / 64) as usize],
            hooks: HashMap::new(),
            sp: STACK_TOP - 0x100,
            host: host::HostState::default(),
            stats: Stats::default(),
            calls: Vec::new(),
            pc: 0,
            lr: 0,
            isp: 0,
            code_page: u32::MAX,
            watch: env_list("F12_WATCH"),
            trace: env_list("F12_TRACE"),
            skip_hook: None,
        };
        env.rebuild_hooks();
        Some(env)
    }

    // ------------------------------------------------------------------ memory views

    /// The memory as black-field's `Mem`.
    pub fn fmem(&mut self) -> &mut FMem {
        if self.script_owns {
            self.fmem.base = self.smem.base;
            self.fmem.bytes = std::mem::take(&mut self.smem.bytes);
            self.fmem.extra = std::mem::take(&mut self.smem.extra);
            self.script_owns = false;
        }
        &mut self.fmem
    }

    /// The memory as black-script's `Mem`.
    pub fn smem(&mut self) -> &mut SMem {
        if !self.script_owns {
            self.smem.base = self.fmem.base;
            self.smem.bytes = std::mem::take(&mut self.fmem.bytes);
            self.smem.extra = std::mem::take(&mut self.fmem.extra);
            self.script_owns = true;
        }
        &mut self.smem
    }

    pub fn r8(&mut self, a: u32) -> u32 {
        Bus::rd(self, a, 1)
    }
    pub fn r16(&mut self, a: u32) -> u32 {
        Bus::rd(self, a, 2)
    }
    pub fn r32(&mut self, a: u32) -> u32 {
        Bus::rd(self, a, 4)
    }
    pub fn w8(&mut self, a: u32, v: u32) {
        Bus::wr(self, a, v, 1)
    }
    pub fn w16(&mut self, a: u32, v: u32) {
        Bus::wr(self, a, v, 2)
    }
    pub fn w32(&mut self, a: u32, v: u32) {
        Bus::wr(self, a, v, 4)
    }

    pub fn read_bytes(&mut self, a: u32, n: usize) -> Vec<u8> {
        (0..n as u32).map(|i| self.r8(a + i) as u8).collect()
    }

    pub fn write_bytes(&mut self, a: u32, b: &[u8]) {
        for (i, &x) in b.iter().enumerate() {
            self.w8(a + i as u32, x as u32);
        }
    }

    fn mapped(&mut self, a: u32, n: u32) -> bool {
        let m = self.fmem();
        let fits =
            |base: u32, len: usize| a.wrapping_sub(base) as usize + n as usize <= len && a >= base;
        fits(m.base, m.bytes.len()) || m.extra.iter().any(|(b, v)| fits(*b, v.len()))
    }

    // ------------------------------------------------------------------ overlays and hooks

    /// Load overlay `n` (image at its link address, BSS zeroed); hooks follow.
    pub fn load_overlay(&mut self, n: u16) {
        let o = self.rom.overlay(n);
        let base = o.base;
        // an overlay whose range overlaps a loaded one replaces it
        let end = o.bss + o.bss_size;
        let clash: Vec<u16> = self
            .loaded
            .iter()
            .copied()
            .filter(|&k| {
                let p = self.rom.overlay(k);
                p.base < end && base < p.bss + p.bss_size
            })
            .collect();
        for k in clash {
            self.loaded.remove(&k);
        }
        let m = self.fmem();
        m.write(base, &o.image);
        m.fill(o.bss, o.bss_size, 0);
        self.loaded.insert(n);
        self.rebuild_hooks();
        // the loader then runs the overlay's static initialisers
        for f in o.sinit.iter().copied().filter(|&f| f != 0) {
            self.call_words(f, &[]);
        }
    }

    pub fn unload_overlay(&mut self, n: u16) {
        self.loaded.remove(&n);
        self.rebuild_hooks();
    }

    /// The loaded overlay whose code contains `a` (`None`: main / autoloads).
    pub fn overlay_at(&mut self, a: u32) -> Option<u16> {
        let l: Vec<u16> = self.loaded.iter().copied().collect();
        l.into_iter().find(|&k| {
            let o = self.rom.overlay(k);
            (o.base..o.base + o.image.len() as u32).contains(&a)
        })
    }

    fn code_loaded(&mut self, a: u32) -> bool {
        (rom::MAIN_BASE..rom::MAIN_BASE + rom::MAIN_CODE_SIZE as u32).contains(&a)
            || (ITCM..ITCM + 0x8000).contains(&a)
            || (0x0689_8000..0x0689_8020).contains(&a)
            || self.overlay_at(a).is_some()
    }

    fn rebuild_hooks(&mut self) {
        self.code_page = u32::MAX;
        self.hooks.clear();
        self.hook_bits.iter_mut().for_each(|w| *w = 0);
        let add = |env: &mut Env, a: u32, ov: u16, h: Hook| {
            if ov != 0 && !env.loaded.contains(&ov) {
                return;
            }
            if env.hooks.contains_key(&a) {
                return; // host models first
            }
            env.hooks.insert(a, h);
            if (HOOK_LO..HOOK_HI).contains(&a) {
                let i = ((a - HOOK_LO) / 2) as usize;
                env.hook_bits[i / 64] |= 1 << (i % 64);
            }
        };
        for &(a, ov, _) in host::HOSTS {
            add(self, a, ov, Hook::Host);
        }
        for &(a, _) in view::WINDOWS {
            add(self, a, 21, Hook::Host);
        }
        for &(a, ov, _) in ports::PORTS {
            add(self, a, ov, Hook::Port);
        }
        for a in self.trace.clone() {
            add(self, a, 0, Hook::Trace);
        }
    }

    // ------------------------------------------------------------------ calls

    /// Calls the routine at `addr` (Thumb bit ignored) with the argument words `w` (r0-r3, then stack
    /// words): its host model, its port, or its original code in the interpreter. Returns r0 | r1 << 32.
    pub fn call_words(&mut self, addr: u32, w: &[u32]) -> u64 {
        let a = addr & !1;
        self.calls.push(a);
        let before: Vec<u32> = if self.watch.is_empty() {
            Vec::new()
        } else {
            let w = self.watch.clone();
            w.iter().map(|&x| self.fmem().r32(x)).collect()
        };
        let traced = self.trace.contains(&a);
        if traced {
            eprintln!(
                "F12 trace: -> {a:08x}({:x?}) (frame {}; {})",
                &w[..w.len().min(6)],
                self.host.frame,
                self.call_stack()
            );
        }
        let r = self.dispatch(a, w);
        if traced {
            eprintln!("F12 trace: <- {a:08x} = {r:#x}");
        }
        if !before.is_empty() {
            for (i, x) in self.watch.clone().into_iter().enumerate() {
                let now = self.fmem().r32(x);
                if now != before[i] {
                    eprintln!(
                        "F12 watch: [{x:#010x}] {:#x} -> {now:#x} during {a:08x} ({})",
                        before[i],
                        self.call_stack()
                    );
                }
            }
        }
        self.calls.pop();
        r
    }

    fn dispatch(&mut self, a: u32, w: &[u32]) -> u64 {
        if !self.hooks.contains_key(&a) {
            if let Some(r) = host::RANGES.iter().find(|r| (r.0..r.1).contains(&a)) {
                *self.stats.hosts.entry(r.0).or_default() += 1;
                return r.2;
            }
        }
        match self.hooks.get(&a).copied() {
            Some(Hook::Host) => {
                *self.stats.hosts.entry(a).or_default() += 1;
                host::call(self, a, w)
            }
            Some(Hook::Port) => {
                *self.stats.ports.entry(a).or_default() += 1;
                let mut v = w.to_vec();
                v.resize(12, 0);
                ports::call(self, a, &v)
            }
            None | Some(Hook::Trace) => self.interp(a, w),
        }
    }

    /// Runs the original code at `a` in the interpreter.
    fn interp(&mut self, a: u32, w: &[u32]) -> u64 {
        assert!(
            self.code_loaded(a),
            "F12 gap: call of {a:#010x}: no port, no host model and no loaded code there (calls {})",
            self.call_stack()
        );
        *self.stats.interp.entry(a).or_default() += 1;
        let arm = self.entry_is_arm(a);
        let mut cpu = Cpu {
            thumb: !arm,
            ..Default::default()
        };
        for (i, &x) in w.iter().take(4).enumerate() {
            cpu.r[i] = x;
        }
        let saved = self.sp;
        let mut sp = self.sp & !7;
        let extra = w.len().saturating_sub(4) as u32;
        sp -= (4 * extra + 7) & !7;
        for (k, &x) in w.iter().skip(4).enumerate() {
            self.w32(sp + 4 * k as u32, x);
        }
        cpu.r[13] = sp;
        cpu.r[14] = RET;
        cpu.r[15] = a;
        let skip = self.skip_hook.replace(a);
        let steps = cpu::run(self, &mut cpu, RET, STEP_LIMIT);
        self.skip_hook = skip;
        self.stats.steps += steps;
        self.sp = saved;
        cpu.r[0] as u64 | (cpu.r[1] as u64) << 32
    }

    /// Whether the function entered at `a` from Rust is ARM code. The decomp's mapping symbols mark many ARM
    /// functions `$t` / `$d`, so the first instruction decides: a Thumb `push` or a Thumb `bl` / `blx` pair is
    /// Thumb, any other word with condition AL (0xE) is ARM; the `$a` symbols confirm.
    fn entry_is_arm(&mut self, a: u32) -> bool {
        if a & 3 != 0 {
            return false;
        }
        let ov = self.overlay_at(a);
        if self.rom.is_arm(a, ov) {
            return true;
        }
        let w = self.r32(a);
        let (lo, hi) = (w & 0xFFFF, w >> 16);
        if lo >> 9 == 0x5A || (lo >> 11 == 0x1E && matches!(hi >> 11, 0x1D | 0x1F)) {
            return false;
        }
        w >> 28 == 0xE
    }

    /// The running calls, innermost last (diagnostics).
    pub fn call_stack(&self) -> String {
        let c = self
            .calls
            .iter()
            .map(|a| format!("{a:08x}"))
            .collect::<Vec<_>>()
            .join(" > ");
        // words on the interpreter stack that look like return addresses
        let mut ret = Vec::new();
        let m = if self.script_owns {
            None
        } else {
            Some(&self.fmem)
        };
        if let Some(m) = m {
            let mut a = self.isp & !3;
            while a + 4 <= STACK_TOP && a >= STACK_LO && ret.len() < 24 && a < self.isp + 0x600 {
                let v = m.r32(a);
                if (rom::MAIN_BASE..0x0220_0000).contains(&v) && (v & 1 == 1 || v & 3 == 0) {
                    ret.push(format!("{v:08x}"));
                }
                a += 4;
            }
        }
        format!(
            "{c}; last pc {:08x} lr {:08x}; overlays {:?}; stack {}",
            self.pc,
            self.lr,
            self.loaded,
            ret.join(" ")
        )
    }

    pub fn cv(&mut self, addr: u32, w: &[u32]) -> u32 {
        self.call_words(addr, w) as u32
    }
}

impl Bus for Env {
    fn rd(&mut self, a: u32, n: u32) -> u32 {
        if (0x0400_0000..0x0480_0000).contains(&a) {
            return host::io_read(self, a, n);
        }
        if !self.mapped(a, n) {
            panic!(
                "F12: read of unmapped {a:#010x}+{n} (calls {})",
                self.call_stack()
            );
        }
        let m = self.fmem();
        match n {
            1 => m.r8(a) as u32,
            2 => m.r16(a) as u32,
            _ => m.r32(a),
        }
    }

    fn wr(&mut self, a: u32, v: u32, n: u32) {
        if (0x0400_0000..0x0480_0000).contains(&a) {
            return host::io_write(self, a, v, n);
        }
        if !self.mapped(a, n) {
            panic!(
                "F12: write of unmapped {a:#010x}+{n} = {v:#x} (calls {})",
                self.call_stack()
            );
        }
        if self.watch.contains(&(a & !3)) {
            eprintln!("F12 watch: [{a:#010x}] = {v:#x} ({})", self.call_stack());
        }
        let m = self.fmem();
        match n {
            1 => m.w8(a, v as u8),
            2 => m.w16(a, v as u16),
            _ => m.w32(a, v),
        }
    }

    fn hooked(&self, pc: u32) -> bool {
        if !(HOOK_LO..HOOK_HI).contains(&pc) || self.skip_hook == Some(pc) {
            return false;
        }
        if host::RANGES.iter().any(|r| (r.0..r.1).contains(&pc)) {
            return true;
        }
        let i = ((pc - HOOK_LO) / 2) as usize;
        self.hook_bits[i / 64] & (1 << (i % 64)) != 0
    }

    fn hook(&mut self, cpu: &mut Cpu) {
        let a = cpu.r[15];
        let sp = cpu.r[13];
        let mut w = vec![cpu.r[0], cpu.r[1], cpu.r[2], cpu.r[3]];
        for k in 0..8 {
            w.push(self.r32(sp + 4 * k));
        }
        let saved = self.sp;
        self.sp = sp & !7;
        let r = self.call_words(a, &w);
        self.sp = saved;
        cpu.r[0] = r as u32;
        cpu.r[1] = (r >> 32) as u32;
        let lr = cpu.r[14];
        cpu.bx(lr);
    }

    fn swi(&mut self, cpu: &mut Cpu, n: u32) {
        host::swi(self, cpu, n);
    }

    fn fail(&mut self, msg: &str) -> ! {
        panic!("{msg} (calls {})", self.call_stack());
    }

    fn at(&mut self, pc: u32, lr: u32, sp: u32) {
        self.skip_hook = None;
        self.pc = pc;
        self.lr = lr;
        self.isp = sp;
        if pc >> 12 != self.code_page {
            assert!(
                self.code_loaded(pc),
                "F12: executing {pc:#010x}, which is not loaded code (lr {lr:#010x}; calls {})",
                self.call_stack()
            );
            self.code_page = pc >> 12;
        }
    }
}

impl crate::hooks::MemEnv for Env {
    fn mem(&mut self) -> &mut FMem {
        self.fmem()
    }
}

impl FieldEnv for Env {
    fn call(&mut self, addr: u32, args: &mut [CArg<'_>]) -> u32 {
        let mut a = Args::new();
        let mut outs = Vec::new();
        for (i, x) in args.iter().enumerate() {
            match x {
                CArg::V(v) | CArg::P(v) | CArg::X(v) | CArg::In(v, _) | CArg::Out(v, _) => a.v(*v),
                CArg::InBuf(b) => a.inb(b),
                CArg::OutBuf(b) => outs.push((i, a.inoutb(b))),
            }
        }
        let r = self.call_args(addr, &mut a);
        for (i, k) in outs {
            if let CArg::OutBuf(b) = &mut args[i] {
                b.copy_from_slice(&a.out_bytes(k));
            }
        }
        r as u32
    }
}

impl crate::mapchange_proc::MapChangeEnv for Env {
    fn call64(&mut self, addr: u32) -> u64 {
        self.call_words(addr, &[])
    }
}

impl crate::field_tasks::FieldEnv64 for Env {
    fn call64(&mut self, addr: u32, args: &[u32]) -> u64 {
        self.call_words(addr, args)
    }
}
