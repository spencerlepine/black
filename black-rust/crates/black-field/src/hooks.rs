//! Undecompiled routines called by the ported field code, modelled as traits.
//!
//! The original passes raw pointers between these routines. The ported code keeps pointers as `u32`
//! addresses and reads or writes through a [`Mem`] window (one contiguous block of the address space
//! that holds the record being worked on), so the pointer values the original passes to its callees stay
//! identical. The callees themselves are trait methods named after their address; their meaning is
//! only what the doc comment says.

/// A window onto the address space: `bytes` is the memory at `base..base + bytes.len()`.
///
/// Accesses outside the window panic: the ported code only follows pointers that point into the
/// record it was given (UNVERIFIED for anything else, the original would read or write whatever is
/// there).
#[derive(Debug, Default, Clone)]
pub struct Mem {
    pub base: u32,
    pub bytes: Vec<u8>,
    /// More windows (`(base, bytes)`), for records that live far apart (a buffer, a region and a ROM global).
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

    fn win(&self, w: usize) -> &Vec<u8> {
        if w == 0 {
            &self.bytes
        } else {
            &self.extra[w - 1].1
        }
    }

    fn win_mut(&mut self, w: usize) -> &mut Vec<u8> {
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

    /// Fill `len` bytes at `addr` with `v` (`MI_CpuFill8`).
    pub fn fill(&mut self, addr: u32, len: u32, v: u8) {
        let (w, o) = self.at(addr, len as usize);
        self.win_mut(w)[o..o + len as usize].fill(v);
    }

    /// Copy `src` to `addr` (`MI_CpuCopy8`).
    pub fn write(&mut self, addr: u32, src: &[u8]) {
        let (w, o) = self.at(addr, src.len());
        self.win_mut(w)[o..o + src.len()].copy_from_slice(src);
    }

    /// Add a zeroed window of `size` bytes for a stack local of the original (a struct the caller builds
    /// on its stack and hands to its callees by address); returns its address. The windows sit at
    /// [`LOCAL_BASE`] and up, below the emulator's stack (0x02EF0000..), so they never meet a record of the
    /// differential tests. Release it with [`Mem::pop_local`] (last in, first out).
    pub fn push_local(&mut self, size: u32) -> u32 {
        let top = self
            .extra
            .iter()
            .filter(|(b, _)| (LOCAL_BASE..LOCAL_BASE + 0x1_0000).contains(b))
            .map(|(b, v)| b + v.len() as u32)
            .max()
            .unwrap_or(LOCAL_BASE);
        let at = (top + 3) & !3;
        self.extra.push((at, vec![0; size as usize]));
        at
    }

    /// Drop the window [`Mem::push_local`] returned `addr` for (it must be the last one added).
    pub fn pop_local(&mut self, addr: u32) {
        let last = self.extra.pop().expect("pop_local without push_local");
        assert_eq!(last.0, addr, "pop_local out of order");
    }

    /// Copy `len` bytes from `src` to `dst` (`MI_CpuCopy8`, forward byte order).
    pub fn copy(&mut self, dst: u32, src: u32, len: u32) {
        for i in 0..len {
            let b = self.r8(src.wrapping_add(i));
            self.w8(dst.wrapping_add(i), b);
        }
    }
}

/// Where [`Mem::push_local`] puts the stack locals of the original (an unmapped page of the differential
/// tests' address space, below their stack at 0x02EF0000).
pub const LOCAL_BASE: u32 = 0x02EC_0000;

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

/// The heap routines (`FUN_02030734` allocate, `FUN_020307B0` free).
pub trait HeapHooks: MemEnv {
    /// C: `FUN_02030734(heap, size, align_flag, file, line)`: allocate a block, returns its address.
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(block)`: free a block.
    fn heap_free(&mut self, block: u32);
}

/// Address of the string `"field_status.c"` (the file argument of the allocation in `FUN_02014428`).
pub const FILE_FIELD_STATUS: u32 = 0x020A_72BC;
/// Address of the string `"btl_setup.c"` (the file argument of the allocations in `btl_setup.c`).
pub const FILE_BTL_SETUP: u32 = 0x020A_729C;

/// Callees of `btl_setup.c` that live in other units (party, game data, trainer data, message
/// bank, ...). All unported, named by address.
pub trait BtlSetupHooks: HeapHooks {
    /// C: `FUN_02045808(v)`: release the object `v` (a `StrBuf` created by `FUN_020457B0`).
    fn fun_02045808(&mut self, v: u32);
    /// C: `FUN_0201AC2C(a, b)`: (arguments in call order).
    fn fun_0201ac2c(&mut self, a: u32, b: u32);
    /// C: `FUN_0200AE54(x, n)`: read field `n` of the record `x`.
    fn fun_0200ae54(&mut self, x: u32, n: u32) -> u32;
    /// C: `FUN_0200B040(x, out)`: fill the 7 bytes at `out` (`p + 0x78`) from `x`.
    fn fun_0200b040(&mut self, x: u32, out: u32);
    /// C: `FUN_0201A30C(x, party, y)`.
    fn fun_0201a30c(&mut self, x: u32, party: u32, y: u32);
    /// C: `FUN_0201A2A8(x, party)`.
    fn fun_0201a2a8(&mut self, x: u32, party: u32);
    /// C: `FUN_020457B0(n, heap)`: create a `StrBuf` of `n` characters.
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_0202428C(q)`: initialise the sub record at `q`.
    fn fun_0202428c(&mut self, q: u32);
    /// C: `FUN_0201A920(heap)`: allocate a PARTY object.
    fn fun_0201a920(&mut self, heap: u32) -> u32;
    /// C: `FUN_0202A440(y, rec, heap)`.
    fn fun_0202a440(&mut self, y: u32, rec: u32, heap: u32);
    /// C: `FUN_0202A4B4(y, party, heap)`.
    fn fun_0202a4b4(&mut self, y: u32, party: u32, heap: u32);
    /// C: `FUN_02012984(x)`: returns a byte.
    fn fun_02012984(&mut self, x: u32) -> u32;
    /// C: `FUN_0216AAE4(x, party, v)` (overlay code).
    fn fun_0216aae4(&mut self, x: u32, party: u32, v: u32);
    /// C: `FUN_0202A3FC(v)`: reads a value out of a trainer data sub record (meaning unverified).
    fn fun_0202a3fc(&mut self, v: u32) -> u32;
    /// C: `FUN_020127A4(x)`: result unused.
    fn fun_020127a4(&mut self, x: u32) -> u32;
    /// C: `FUN_02012934(x)`.
    fn fun_02012934(&mut self, x: u32) -> u32;
    /// C: `FUN_02012944(x)`.
    fn fun_02012944(&mut self, x: u32) -> u32;
    /// C: `FUN_0201292C(x)`.
    fn fun_0201292c(&mut self, x: u32) -> u32;
    /// C: `FUN_02012924(x)`.
    fn fun_02012924(&mut self, x: u32) -> u32;
    /// C: `FUN_0200CA1C(x)`.
    fn fun_0200ca1c(&mut self, x: u32) -> u32;
    /// C: `FUN_02012F20(x)`.
    fn fun_02012f20(&mut self, x: u32) -> u32;
    /// C: `FUN_02012F2C(x)`.
    fn fun_02012f2c(&mut self, x: u32) -> u32;
    /// C: `FUN_02012EBC(x)`.
    fn fun_02012ebc(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C0E8()`.
    fn fun_0200c0e8(&mut self) -> u32;
    /// C: `FUN_0200873C(v)`.
    fn fun_0200873c(&mut self, v: u32) -> u32;
    /// C: `FUN_0200C258(v)`: returns a byte.
    fn fun_0200c258(&mut self, v: u32) -> u32;
    /// C: `FUN_0201296C(x)`: the event work object of the game data `x` (`game_data::event_work_ptr`).
    fn fun_0201296c(&mut self, x: u32) -> u32;
    /// C: `FUN_020142E8(ew, id)`: event flag test (`event_work::check_flag` on the object at `ew`).
    fn fun_020142e8(&mut self, ew: u32, id: u32) -> u32;
    /// C: `FUN_0201AA18(party)`.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AA14(party)`.
    fn fun_0201aa14(&mut self, party: u32) -> u32;
    /// C: `FUN_0201293C(x)`.
    fn fun_0201293c(&mut self, x: u32) -> u32;
    /// C: `FUN_020076D0(v, a, b)`: `a` and `b` are out parameters (initialised to 0 by the caller).
    fn fun_020076d0(&mut self, v: u32, a: &mut u32, b: &mut u32) -> u32;
    /// C: `FUN_0200846C(heap)`: allocate a 4-byte object.
    fn fun_0200846c(&mut self, heap: u32) -> u32;
    /// C: `FUN_0203D1A4(v)`: returns a byte.
    fn fun_0203d1a4(&mut self, v: u32) -> u32;
    /// C: `FUN_020082EC(heap)`.
    fn fun_020082ec(&mut self, heap: u32) -> u32;
    /// C: `FUN_02008314(a, b)`.
    fn fun_02008314(&mut self, a: u32, b: u32);
}

/// Callees of `game_data.c` / `game_event.c` that live in other units (save data, the event task
/// scheduler, the player/camera glue, overlay code, ...). All unported, named by address.
pub trait GameDataHooks: HeapHooks {
    /// C: `FUN_02031324(heap)`: the scheduler object of the game system.
    fn fun_02031324(&mut self, heap: u32) -> u32;
    /// C: `FUN_020124F4(heap)`: allocate and fill the GAMEDATA record (`game_data` constructor).
    fn fun_020124f4(&mut self, heap: u32) -> u32;
    /// C: `FUN_020261E4(heap)`: create the object stored at `sys + 0x24`.
    fn fun_020261e4(&mut self, heap: u32) -> u32;
    /// C: `FUN_0202B4BC(gamedata, heap)`: create the object stored at `sys + 0x2C`.
    fn fun_0202b4bc(&mut self, gd: u32, heap: u32) -> u32;
    /// C: `FUN_02035138(proc, arg)`: register a callback (the thunk target of `FUN_02012248`).
    fn fun_02035138(&mut self, proc_: u32, arg: u32);
    /// C: `FUN_02035064(proc, arg)`: register a callback (the thunk target of `FUN_020122A4`).
    fn fun_02035064(&mut self, proc_: u32, arg: u32);
    /// C: `FUN_020139B0(heap)`: create the zone cache (`zonedata::cache_new`, other area).
    fn fun_020139b0(&mut self, heap: u32);
    /// C: `FUN_02031348(sched, a, b, c)`: add a task to the scheduler.
    fn fun_02031348(&mut self, sched: u32, a: u32, b: u32, c: u32) -> u32;
    /// C: `FUN_02008448(x)`: read the flag word of the object `x`.
    fn fun_02008448(&mut self, x: u32) -> u32;
    /// C: `FUN_02008450(x, flag)`: write the flag of the object `x`.
    fn fun_02008450(&mut self, x: u32, flag: u32);
    /// C: `FUN_02013D34(id)`: zone id predicate (`zonedata`, other area).
    fn fun_02013d34(&mut self, id: u32) -> u32;
    /// C: `FUN_0200FB24()`.
    fn fun_0200fb24(&mut self) -> u32;
    /// C: `FUN_02158318(sys)` (overlay code).
    fn fun_02158318(&mut self, sys: u32) -> u32;
    /// C: `FUN_02035390()`.
    fn fun_02035390(&mut self) -> u32;
    /// C: `FUN_020353B0()`.
    fn fun_020353b0(&mut self) -> u32;
    /// C: `FUN_0200921C(gamedata)`: result unused.
    fn fun_0200921c(&mut self, gd: u32);
    /// C: `FUN_020090E0()`.
    fn fun_020090e0(&mut self) -> u32;
    /// C: `FUN_0203CD98()`.
    fn fun_0203cd98(&mut self) -> u32;
    /// C: `FUN_02026454(task_obj)`.
    fn fun_02026454(&mut self, x: u32) -> u32;
    /// C: `FUN_020263D8(task_obj, 1, sys)`.
    fn fun_020263d8(&mut self, x: u32, a: u32, sys: u32);
    /// C: `FUN_02188BE8(x)` (overlay code).
    fn fun_02188be8(&mut self, x: u32) -> u32;
    /// C: `FUN_0203DA4C()`.
    fn fun_0203da4c(&mut self) -> u32;
    /// C: `FUN_0200A864(save)`.
    fn fun_0200a864(&mut self, save: u32) -> u32;
    /// C: `FUN_0200A890(x)`.
    fn fun_0200a890(&mut self, x: u32) -> u32;
    /// C: `FUN_0200A898(x)`.
    fn fun_0200a898(&mut self, x: u32) -> u32;
    /// C: `FUN_02161D64(record)` (overlay code): post-process the 0x1E8-byte record at `record`.
    fn fun_02161d64(&mut self, record: u32);
    /// C: `FUN_02008850(x)`: returns the address of a halfword inside the object `x`.
    fn fun_02008850(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C40C(save)`.
    fn fun_0200c40c(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C418(x, a, b)`.
    fn fun_0200c418(&mut self, x: u32, a: u32, b: u32);
    /// C: `FUN_0200C404(save)`.
    fn fun_0200c404(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C44C(x, a)`.
    fn fun_0200c44c(&mut self, x: u32, a: u32);
    /// C: `FUN_020071CC(save, block)`: the save block `block` of the save data.
    fn fun_020071cc(&mut self, save: u32, block: u32) -> u32;
    /// C: `FUN_0200C0F0(save)`.
    fn fun_0200c0f0(&mut self, save: u32) -> u32;
    /// C: `FUN_02009EC8(save)`.
    fn fun_02009ec8(&mut self, save: u32) -> u32;
    /// C: `FUN_020089C0(save)`.
    fn fun_020089c0(&mut self, save: u32) -> u32;
    /// C: `FUN_02009448(save)`.
    fn fun_02009448(&mut self, save: u32) -> u32;
    /// C: `FUN_02007B44(save)`.
    fn fun_02007b44(&mut self, save: u32) -> u32;
    /// C: `FUN_020085D4(x)`.
    fn fun_020085d4(&mut self, x: u32) -> u32;
    /// C: `FUN_0200873C(handle)`.
    fn fun_0200873c(&mut self, handle: u32) -> u32;
    /// C: `FUN_02008374(x)`.
    fn fun_02008374(&mut self, x: u32) -> u32;
}

/// Address of the string `"zonedata.c"` (the file argument of the allocation in `FUN_020139B0`).
pub const FILE_ZONEDATA: u32 = 0x020A_72B0;

/// Callees of `zonedata.c` that live in other units (archive access, the event-work direction table,
/// the sound/timer glue). All unported, named by address.
pub trait ZoneHooks: HeapHooks {
    /// C: `FUN_020490F4(n, heap)`: open the zone header archive, returns its handle.
    fn fun_020490f4(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_0204A298(a, b, c, heap, &size)`: load a table, returns its address and stores its size in bytes.
    fn fun_0204a298(&mut self, a: u32, b: u32, c: u32, heap: u32, size: &mut u32) -> u32;
    /// C: `FUN_02049238(handle)`: close the archive.
    fn fun_02049238(&mut self, handle: u32);
    /// C: `FUN_0204944C(handle, 0, offset, size, dst)`: read `size` bytes of the archive at `offset` into `dst`.
    fn fun_0204944c(&mut self, handle: u32, a: u32, offset: u32, size: u32, dst: u32);
    /// C: `FUN_0201441C(v)`: the byte table lookup of `event_work::table_0209e050`.
    fn fun_0201441c(&mut self, v: u32) -> u32;
    /// C: `FUN_02020E08(0xD, a, b)`.
    fn fun_02020e08(&mut self, a: u32, b: u32, c: u32);
}

/// One argument of an unported callee reached through [`FieldEnv::call`], in the callee's own
/// register/stack order (F2a). Pointers stay addresses; a pointer to a local of the caller (a stack
/// struct of the original) is passed as the bytes themselves.
pub enum CArg<'a> {
    /// A scalar, or a pointer whose value the callee's answer depends on.
    V(u32),
    /// A pointer (or a register) the callee does not read in the modelled behaviour; the value is
    /// kept for a real implementation.
    P(u32),
    /// Pointer to `len` bytes of the memory window that the callee reads.
    In(u32, u32),
    /// Pointer to `len` bytes of the memory window that the callee writes.
    Out(u32, u32),
    /// Pointer to a local of the caller that the callee reads.
    InBuf(&'a [u8]),
    /// Pointer to a local of the caller that the callee writes.
    OutBuf(&'a mut [u8]),
    /// An argument the original passes that the differential spec's stub of the callee does not list
    /// (a trailing stack word); a real implementation gets it, the test stubs drop it.
    X(u32),
}

/// The callees of the F2a areas (`map_matrix`, `fieldmap`, `fieldmap_event`, `fieldmap_func`,
/// `land_data_patch`) that live in other units or areas: heap, archives, task systems, overlay
/// loading, the 3D / 2D drawing side (D3: rewritten in G8, never ported here), the field subsystems
/// of other F items, and the other black-field areas. They are reached by their address; what each
/// one does is only what the comment at the call site says (taken from the spec), nothing more.
/// Callees inside the same area that are ported are called as Rust functions instead.
pub trait FieldEnv: MemEnv {
    /// Calls the routine at `addr` (Thumb bit clear) with `args`; returns its r0 (0 when it
    /// returns nothing). A fatal routine (assert) may not return.
    fn call(&mut self, addr: u32, args: &mut [CArg<'_>]) -> u32;

    /// [`FieldEnv::call`] with scalar arguments only.
    fn callv(&mut self, addr: u32, args: &[u32]) -> u32 {
        let mut a: Vec<CArg<'_>> = args.iter().map(|&v| CArg::V(v)).collect();
        self.call(addr, &mut a)
    }
}

/// Address of `Heap_AllocDebug(heap, size, flag, file, line)`.
pub const HEAP_ALLOC: u32 = 0x0203_0734;
/// Address of `Heap_Free(block)`.
pub const HEAP_FREE: u32 = 0x0203_07B0;
