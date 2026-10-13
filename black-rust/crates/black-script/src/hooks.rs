//! Requests the script VM makes to the game (messages, menus, fades) and the undecompiled routines the
//! ported script code calls, modelled as traits.
//!
//! The original passes raw pointers between these routines. The ported code keeps pointers as `u32`
//! addresses and reads or writes through a [`Mem`] window, so the pointer values the original passes to its
//! callees stay identical (same model as `black_field::hooks`; UNVERIFIED for addresses outside the window:
//! the original would read or write whatever is there, the port panics). The callees themselves are trait
//! methods named after their address; their meaning is only what the doc comment says.

/// A window onto the address space: `bytes` is the memory at `base..base + bytes.len()`, plus more
/// windows in `extra` for records that live far apart.
#[derive(Debug, Default, Clone)]
pub struct Mem {
    pub base: u32,
    pub bytes: Vec<u8>,
    pub extra: Vec<(u32, Vec<u8>)>,
}

impl Mem {
    pub fn new(base: u32, bytes: Vec<u8>) -> Self {
        Mem {
            base,
            bytes,
            extra: Vec::new(),
        }
    }

    /// Add another window; returns its index in `extra`.
    pub fn add_window(&mut self, base: u32, bytes: Vec<u8>) -> usize {
        self.extra.push((base, bytes));
        self.extra.len() - 1
    }

    /// Window and offset of `addr..addr + len` (window 0 is the main one, `i + 1` is `extra[i]`).
    fn at(&self, addr: u32, len: usize) -> (usize, usize) {
        let fits = |base: u32, n: usize| {
            let o = addr.wrapping_sub(base) as usize;
            o.checked_add(len).is_some_and(|e| e <= n).then_some(o)
        };
        if let Some(o) = fits(self.base, self.bytes.len()) {
            return (0, o);
        }
        for (i, (b, v)) in self.extra.iter().enumerate() {
            if let Some(o) = fits(*b, v.len()) {
                return (i + 1, o);
            }
        }
        panic!(
            "address {addr:#010x}+{len} is outside the memory window {:#010x}+{:#x}",
            self.base,
            self.bytes.len()
        );
    }

    fn win(&self, w: usize) -> &[u8] {
        if w == 0 {
            &self.bytes
        } else {
            &self.extra[w - 1].1
        }
    }

    fn win_mut(&mut self, w: usize) -> &mut [u8] {
        if w == 0 {
            &mut self.bytes
        } else {
            &mut self.extra[w - 1].1
        }
    }

    pub fn r8(&self, addr: u32) -> u8 {
        let (w, o) = self.at(addr, 1);
        self.win(w)[o]
    }

    pub fn r16(&self, addr: u32) -> u16 {
        let (w, o) = self.at(addr, 2);
        let v = self.win(w);
        u16::from_le_bytes([v[o], v[o + 1]])
    }

    pub fn r32(&self, addr: u32) -> u32 {
        let (w, o) = self.at(addr, 4);
        u32::from_le_bytes(self.win(w)[o..o + 4].try_into().unwrap())
    }

    pub fn w8(&mut self, addr: u32, v: u8) {
        let (w, o) = self.at(addr, 1);
        self.win_mut(w)[o] = v;
    }

    pub fn w16(&mut self, addr: u32, v: u16) {
        let (w, o) = self.at(addr, 2);
        self.win_mut(w)[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }

    pub fn w32(&mut self, addr: u32, v: u32) {
        let (w, o) = self.at(addr, 4);
        self.win_mut(w)[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// Anything that owns the memory window the ported code works in.
pub trait MemEnv {
    fn mem(&mut self) -> &mut Mem;

    fn r8(&mut self, addr: u32) -> u8 {
        self.mem().r8(addr)
    }
    fn r16(&mut self, addr: u32) -> u16 {
        self.mem().r16(addr)
    }
    fn r32(&mut self, addr: u32) -> u32 {
        self.mem().r32(addr)
    }
    fn w8(&mut self, addr: u32, v: u8) {
        self.mem().w8(addr, v);
    }
    fn w16(&mut self, addr: u32, v: u16) {
        self.mem().w16(addr, v);
    }
    fn w32(&mut self, addr: u32, v: u32) {
        self.mem().w32(addr, v);
    }
}

/// Unported callees of `script_sys.c` / `script_work.c` (main ARM9 and the scrcmd ctx getters not ported
/// yet). Named by address.
pub trait ScriptSysHooks: MemEnv {
    /// C: `FUN_0201296C(gamedata)`: the EVENT WORK object of the game data (`black_field::game_data`, +0x1AC).
    fn fun_0201296c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020143CC(ew, var)`: address of event var `var` (`black_field::event_work::var_addr`).
    fn fun_020143cc(&mut self, ew: u32, var: u32) -> u32;
    /// C: `FUN_02011330(vm)` (vm.c): read the next u16 of the script at the VM's PC (PC += 2).
    fn fun_02011330(&mut self, vm: u32) -> u32;
    /// C: `FUN_0215A5E4(ctx)`: the GAMEDATA of the scrcmd ctx (word 2 of the scrcmd work at ctx + 0x20).
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32;
}
