//! Undecompiled routines called by the ported text code, modelled as traits.

/// The heap the string buffers live in (`FUN_02030734` alloc, `FUN_020307B0` free).
/// Addresses are the game's; `bytes` gives access to the allocation behind one.
pub trait StrHeap {
    /// C: `FUN_02030734(heap, size, align, file, line)`: returns the address of the block.
    fn alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(addr)`.
    fn free(&mut self, addr: u32);
    /// The bytes of the block at `addr`.
    fn bytes(&mut self, addr: u32) -> &mut [u8];
}

/// The message-data hooks of `msgdata.c`.
pub trait MsgHooks {
    /// C: `FUN_02046180(msgdata)`: reload the bank if the language changed.
    fn reload_language(&mut self, msgdata: u32);
}
