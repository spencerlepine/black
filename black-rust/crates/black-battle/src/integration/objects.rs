//! The integration address space: byte objects at original-style addresses.
//!
//! Every object the ports reach through a pointer (the main module M, the BATTLE_SETUP, the server
//! object S, the flow work, client objects, command queues, adapter channels, parties, ...) is a
//! [`Block`] of bytes at a fixed address. A port that works on a slice or an `SvFlowWork` *takes*
//! the block for the duration of the call (the block is then "held"); a nested call that needs a
//! held block must get it lent by the caller (`FlowEnv::call_lent`), otherwise the access panics
//! with the block's name, which is how missing lends (wiring gaps) show up.

use crate::server::MemExt;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// First address of the integration heap (above every overlay and the ARM9 BSS).
pub const HEAP_BASE: u32 = 0x0230_0000;
/// End of the integration heap.
pub const HEAP_END: u32 = 0x0280_0000;

/// One object.
pub struct Block {
    pub name: String,
    pub data: Vec<u8>,
    /// Size while held (`data` is empty then).
    pub len: usize,
    /// Address of the held buffer (to recognise a lent slice), 0 when not held.
    pub held_ptr: usize,
    pub held: bool,
    /// Copies of parts of this block a port is working on (`Env::aliases`): the block is held
    /// while non-zero.
    pub alias_holds: u32,
}

/// The object table.
#[derive(Default)]
pub struct Objects {
    pub blocks: BTreeMap<u32, Block>,
    next: u32,
}

impl Objects {
    pub fn new() -> Self {
        Objects {
            blocks: BTreeMap::new(),
            next: HEAP_BASE,
        }
    }

    /// Maps `data` at `base` (a global or a ROM table).
    pub fn map(&mut self, base: u32, data: Vec<u8>, name: &str) {
        let len = data.len();
        if let Some((b, blk)) = self.find_block(base, 1) {
            panic!(
                "B16: mapping {name} at {base:#010x} overlaps {} at {b:#010x}",
                blk.name
            );
        }
        self.blocks.insert(
            base,
            Block {
                name: name.to_string(),
                data,
                len,
                held_ptr: 0,
                held: false,
                alias_holds: 0,
            },
        );
    }

    /// Allocates a zeroed block (4-byte aligned, a 16-byte gap between blocks so an overrun is an
    /// unmapped access).
    pub fn alloc(&mut self, size: u32, name: &str) -> u32 {
        let base = self.next;
        let size4 = (size.max(1) + 3) & !3;
        self.next = base + size4 + 0x10;
        assert!(self.next < HEAP_END, "B16: integration heap exhausted");
        self.map(base, vec![0; size as usize], name);
        base
    }

    /// Frees a heap block (unknown / non-heap addresses panic).
    pub fn free(&mut self, base: u32) {
        let b = self
            .blocks
            .remove(&base)
            .unwrap_or_else(|| panic!("B16: free of unknown block {base:#010x}"));
        assert!(
            !b.held && b.alias_holds == 0,
            "B16: free of held block {}",
            b.name
        );
    }

    /// The block containing `addr..addr + len`.
    pub fn find_block(&self, addr: u32, len: usize) -> Option<(u32, &Block)> {
        let (&b, blk) = self.blocks.range(..=addr).next_back()?;
        let o = (addr - b) as usize;
        if o + len <= blk.len.max(1) && (o < blk.len || len == 0) {
            Some((b, blk))
        } else {
            None
        }
    }

    /// Base address and name of the block containing `addr`.
    pub fn base_of(&self, addr: u32) -> Option<u32> {
        self.find_block(addr, 1).map(|(b, _)| b)
    }

    pub fn name_of(&self, addr: u32) -> String {
        match self.find_block(addr, 1) {
            Some((b, blk)) => format!("{}+{:#x}", blk.name, addr - b),
            None => format!("{addr:#010x}"),
        }
    }

    fn block_mut(&mut self, addr: u32, len: usize) -> (&mut Block, usize) {
        let base = match self.find_block(addr, len) {
            Some((b, _)) => b,
            None => panic!("B16: access to unmapped memory {addr:#010x}+{len}"),
        };
        let blk = self.blocks.get_mut(&base).unwrap();
        if blk.held || blk.alias_holds > 0 {
            panic!(
                "B16: {} ({base:#010x}) is held by a port frame and was not lent (access {addr:#010x}+{len})",
                blk.name
            );
        }
        (blk, (addr - base) as usize)
    }

    pub fn read(&mut self, addr: u32, out: &mut [u8]) {
        let (b, o) = self.block_mut(addr, out.len());
        out.copy_from_slice(&b.data[o..o + out.len()]);
    }

    pub fn write(&mut self, addr: u32, data: &[u8]) {
        let (b, o) = self.block_mut(addr, data.len());
        b.data[o..o + data.len()].copy_from_slice(data);
    }

    pub fn r8(&mut self, addr: u32) -> u32 {
        let mut b = [0; 1];
        self.read(addr, &mut b);
        b[0] as u32
    }

    pub fn r16(&mut self, addr: u32) -> u32 {
        let mut b = [0; 2];
        self.read(addr, &mut b);
        u16::from_le_bytes(b) as u32
    }

    pub fn r32(&mut self, addr: u32) -> u32 {
        let mut b = [0; 4];
        self.read(addr, &mut b);
        u32::from_le_bytes(b)
    }

    pub fn w8(&mut self, addr: u32, v: u32) {
        self.write(addr, &[v as u8]);
    }

    pub fn w16(&mut self, addr: u32, v: u32) {
        self.write(addr, &(v as u16).to_le_bytes());
    }

    pub fn w32(&mut self, addr: u32, v: u32) {
        self.write(addr, &v.to_le_bytes());
    }

    /// Takes the whole block at `base` out of the table (it is held until [`Self::put`]).
    pub fn take(&mut self, base: u32) -> Vec<u8> {
        let blk = self
            .blocks
            .get_mut(&base)
            .unwrap_or_else(|| panic!("B16: take of unknown block {base:#010x}"));
        if blk.held || blk.alias_holds > 0 {
            panic!(
                "B16: {} ({base:#010x}) is already held by an outer port frame and was not lent",
                blk.name
            );
        }
        // the block keeps a (stale) copy while held: a holder that lends a part of it gets that
        // part flushed into it for the duration of a call (`Env::dispatch`)
        let v = blk.data.clone();
        blk.held = true;
        blk.held_ptr = v.as_ptr() as usize;
        v
    }

    /// Returns a block taken with [`Self::take`].
    pub fn put(&mut self, base: u32, data: Vec<u8>) {
        let blk = self.blocks.get_mut(&base).expect("put of unknown block");
        assert!(blk.held, "put of a block that is not held");
        assert_eq!(data.len(), blk.len, "put of a resized block {}", blk.name);
        blk.data = data;
        blk.held = false;
        blk.held_ptr = 0;
    }

    /// The held block whose holder's buffer contains `ptr..ptr + len`: `(base, offset)`.
    /// Live holder buffers never overlap, so two matches mean a stale `held_ptr` (a holder's buffer
    /// that moved or was freed while its block stayed held): that panics instead of picking one.
    pub fn held_block(&self, ptr: usize, len: usize) -> Option<(u32, usize)> {
        let mut m = self
            .blocks
            .iter()
            .filter(|(_, blk)| {
                blk.held && ptr >= blk.held_ptr && ptr + len <= blk.held_ptr + blk.len
            })
            .map(|(&b, blk)| (b, ptr - blk.held_ptr));
        let r = m.next();
        if let Some((b2, _)) = m.next() {
            panic!(
                "B16: a lent slice matches two held blocks ({:#010x}, {b2:#010x}): stale holder",
                r.unwrap().0
            );
        }
        r
    }

    /// Address of a lent slice: the held block whose buffer contains `ptr`.
    pub fn held_addr(&self, ptr: usize, len: usize) -> Option<u32> {
        self.held_block(ptr, len).map(|(b, o)| b + o as u32)
    }
}

/// The table as the `server` / `ai` ports' [`MemExt`] backend.
pub struct Shared(pub Rc<RefCell<Objects>>);

impl MemExt for Shared {
    fn read(&self, addr: u32, out: &mut [u8]) {
        self.0.borrow_mut().read(addr, out)
    }

    fn write(&self, addr: u32, data: &[u8]) {
        self.0.borrow_mut().write(addr, data)
    }
}
