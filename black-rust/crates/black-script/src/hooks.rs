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

/// The heap (`FUN_02030734` allocate, `FUN_020307B0` free; same routines as `black_field::hooks::HeapHooks`).
pub trait HeapHooks: MemEnv {
    /// C: `FUN_02030734(heap, size, flag, file, line)` (`Heap_AllocDebug`): allocate a block, returns its address.
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(block)` (`Heap_Free`); also the target of the veneers `0x02011264`, `0x02158824`, `0x0215A554`.
    fn heap_free(&mut self, block: u32);
}

/// Script command dispatch (the F8 seam). The VM step (`vm::step`, `0x02011298`) reads the handler address from the
/// command table (`VM + 4`; `0x0217062C` for scripts, index = command id) and calls it as `handler(vm, ctx)`; the result
/// is 1 = yield (the VM keeps running next frame) or anything else = run the next command. The implementation maps the
/// address to the ported handler (F8: `scrcmd` module); commands 49, 50 and 311 are ported in `script_sys`
/// (`cmd_wait_pad_a` / `cmd_wait_pad_b` / `cmd_311`).
pub trait CommandDispatch {
    /// C: `table[cmd](vm, ctx)` at `0x02011312`: run the command handler at address `handler` (Thumb bit set).
    fn command(&mut self, handler: u32, vm: u32, ctx: u32) -> u32;
}

/// Callees of `vm.c` reached through the function pointers stored in the VM.
pub trait VmHooks: HeapHooks + CommandDispatch {
    /// C: `(VM+0x10)(vm, ctx)` at `0x020112B8`: the wait callback a command installed with `vm::wait` (`0x020113D0`);
    /// 1 = done. Script callbacks: `0x02159961` / `0x02159985` (`script_sys::wait_pad_a` / `wait_pad_b`), `0x021599A9`
    /// (`script_sys::wait_311`); the others belong to F8 commands.
    fn call_wait(&mut self, f: u32, vm: u32, ctx: u32) -> u32;
    /// C: `(VM+0x24)(vm, ctx, VM+0x28, cmd)` at `0x020112FC`: the pre-dispatch hook; 0 stops the VM. Scripts install
    /// `0x02158DE9` (`script_sys::dispatch_hook`, the command permission check) with the GAMESYS as its argument.
    fn call_hook(&mut self, f: u32, vm: u32, ctx: u32, arg: u32, cmd: u32) -> u32;
}

/// Callees of `script_sys.c` / `script_work.c` / the scrcmd ctx getters that live in other units (game data, event
/// work, field map, message resources, archive loading, the VM in main). Named by address; their meaning is only what
/// the doc comment says. The `vm.c` routines default to the ports in [`crate::vm`] (`vm.c` is its own check area, so the
/// `script_sys` test overrides them with the spec stubs).
pub trait ScriptSysHooks: VmHooks {
    // ---- vm.c (main), ported in crate::vm
    /// C: `FUN_0201121C(heap, init)`: create a VM from the 12-byte init record `{u16 stack, u16 regs, table, count}`.
    fn fun_0201121c(&mut self, heap: u32, init: &[u8; 12]) -> u32
    where
        Self: Sized,
    {
        crate::vm::create(self, heap, init)
    }
    /// C: `FUN_0201126C(vm, ctx)`: reset the VM and set its ctx.
    fn fun_0201126c(&mut self, vm: u32, ctx: u32)
    where
        Self: Sized,
    {
        crate::vm::init(self, vm, ctx)
    }
    /// C: `FUN_0201127C(vm, pc)`: start the VM at `pc`.
    fn fun_0201127c(&mut self, vm: u32, pc: u32)
    where
        Self: Sized,
    {
        crate::vm::start(self, vm, pc)
    }
    /// C: `FUN_02011298(vm)`: run the VM for one frame (0 = stopped).
    fn fun_02011298(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::step(self, vm)
    }
    /// C: `FUN_02011324(vm)`: the VM's ctx.
    fn fun_02011324(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::ctx(self, vm)
    }
    /// C: `FUN_02011328(vm, hook, arg)`: install the pre-dispatch hook.
    fn fun_02011328(&mut self, vm: u32, hook: u32, arg: u32)
    where
        Self: Sized,
    {
        crate::vm::set_hook(self, vm, hook, arg)
    }
    /// C: `FUN_02011330(vm)`: read the next u16 of the script at the VM's PC (PC += 2).
    fn fun_02011330(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::read_u16(self, vm)
    }
    /// C: `FUN_0201134C(vm)`: read the next u32 of the script at the VM's PC (PC += 4).
    fn fun_0201134c(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::read_u32(self, vm)
    }
    /// C: `FUN_020113D0(vm, f)`: wait on the callback `f` (status 2).
    fn fun_020113d0(&mut self, vm: u32, f: u32)
    where
        Self: Sized,
    {
        crate::vm::wait(self, vm, f)
    }

    // ---- function pointers
    /// C: `(SYS+0x10)(x, SYS+0x14)` at `0x021588BA`: the script event's end callback. `script_sys::sys_set_end_callback`
    /// installs `0x0215889D`, a veneer to main `0x0201235C` (unported).
    fn call_end_callback(&mut self, f: u32, x: u32, arg: u32) -> u32;
    /// C: `proc(task, task + 0xC)` at `0x021591DC`: init proc `f` of the table `0x021705F0` (`script_sys::INIT_PROCS`;
    /// overlay 10/21 code, F8/F10). 0 = not done yet.
    fn call_init_proc(&mut self, f: u32, task: u32, seq: u32) -> u32;

    // ---- game system / game data / event work (main; most are ported in black_field::game_data / event_work)
    /// C: `FUN_020120F4(gamesys)`: the GAMEDATA of the game system.
    fn fun_020120f4(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020120F8(gamesys)`: the GAMEDATA block whose u16 zone id `FUN_02012AC8` reads.
    fn fun_020120f8(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)`: the field map (FIELDMAP) of the game system, 0 when there is none.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020122C0(gamesys, parent, fn, work size)`: create a GMEVENT.
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32;
    /// C: `FUN_02012374(event, child)`: call the child event `child` from `event`.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_020124E8(event)`: the work block of a GMEVENT.
    fn fun_020124e8(&mut self, event: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)`: address of the 0x20-byte block at GAMEDATA + 0x114.
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012844(gamedata)`: the GAMEDATA field whose `FUN_02162450` is the zone's special-script table.
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201296C(gamedata)`: the EVENT WORK of the game data (`black_field::game_data`, +0x1AC).
    fn fun_0201296c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012AC8(x)`: the u16 zone id at `x`.
    fn fun_02012ac8(&mut self, x: u32) -> u32;
    /// C: `FUN_02012DAC(gamedata)`: (command 311) starts a save-side step (meaning unverified).
    fn fun_02012dac(&mut self, gamedata: u32);
    /// C: `FUN_02012DD0(gamedata)`: (command 311) status of that step (0..3 are handled).
    fn fun_02012dd0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012E40(gamedata)`: the u8 at GAMEDATA + 0x1CE (`black_field::game_data::gd_get_02012e40`).
    fn fun_02012e40(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F2C(gamedata)`: a GAMEDATA sub object (command 311 passes it to `FUN_02008DF0`).
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02008DF0(x, v)`.
    fn fun_02008df0(&mut self, x: u32, v: u32);
    /// C: `FUN_02013B5C(zone)`: the zone's script file (`black_field::zonedata`).
    fn fun_02013b5c(&mut self, zone: u32) -> u32;
    /// C: `FUN_02013B68(zone)`: the zone's message bank (`black_field::zonedata`).
    fn fun_02013b68(&mut self, zone: u32) -> u32;
    /// C: `FUN_02014368(ew, first, last)`: clear event flags `first..=last` (`black_field::event_work`).
    fn fun_02014368(&mut self, ew: u32, first: u32, last: u32);
    /// C: `FUN_020143CC(ew, var)`: address of event var `var` (`black_field::event_work::var_addr`).
    fn fun_020143cc(&mut self, ew: u32, var: u32) -> u32;
    /// C: `FUN_020143D8(ew, first, end)`: clear event vars `first..end` (`black_field::event_work`).
    fn fun_020143d8(&mut self, ew: u32, first: u32, end: u32);

    // ---- text / archives / system (main)
    /// C: `FUN_0201EC74(n, len, heap)`: create a WORDSET (`black_text::wordset`).
    fn fun_0201ec74(&mut self, n: u32, len: u32, heap: u32) -> u32;
    /// C: `FUN_0201ED04(wordset)`: free a WORDSET.
    fn fun_0201ed04(&mut self, wordset: u32);
    /// C: `FUN_020457B0(len, heap)`: create a STRBUF (`black_text::strbuf`).
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32;
    /// C: `FUN_02045808(strbuf)`: free a STRBUF.
    fn fun_02045808(&mut self, strbuf: u32);
    /// C: `FUN_0204A1BC(arc, file, flag, heap)`: load file `file` of ARC `arc` into a new heap block.
    fn fun_0204a1bc(&mut self, arc: u32, file: u32, flag: u32, heap: u32) -> u32;
    /// C: `FUN_02034714(a, b, c)`: assertion hook (calls a registered handler if any, then returns; the differential
    /// test treats it as noreturn).
    fn fun_02034714(&mut self, a: u32, b: u32, c: u32);
    /// C: `FUN_020362DC()`: the held pad/touch bits (`ScriptHost::pad_held`).
    fn fun_020362dc(&mut self) -> u32;
    /// C: `FUN_0203CD98()` (fatal path of the permission check).
    fn fun_0203cd98(&mut self) -> u32;
    /// C: `FUN_0203CF04()`.
    fn fun_0203cf04(&mut self);
    /// C: `FUN_0203CEAC(v)`.
    fn fun_0203ceac(&mut self, v: u32);
    /// C: `FUN_0203D0D0()`.
    fn fun_0203d0d0(&mut self);
    /// C: `FUN_0203CF5C()`.
    fn fun_0203cf5c(&mut self);
    /// C: `FUN_0203CDBC()`.
    fn fun_0203cdbc(&mut self) -> u32;
    /// C: `FUN_020053EC()`: one idle step of the fatal stop loop.
    fn fun_020053ec(&mut self);

    // ---- overlay 10 outside this area (scrcmd_work.c = F8, field code)
    /// C: `FUN_02158768(gamesys, id, 0, x)`: start a conditional special script (delivery/IRC side).
    fn fun_02158768(&mut self, gamesys: u32, id: u32, a: u32, x: u32) -> u32;
    /// C: `FUN_0215A514(sw, heap)`: create the scrcmd work (0x110 bytes) of a SCRIPT_WORK.
    fn fun_0215a514(&mut self, sw: u32, heap: u32) -> u32;
    /// C: `FUN_0215A55C(init, heap)`: create a scrcmd CTX from the 16-byte init record
    /// `{u16 zone, u16 script id, mode, mode != 0, SW}`.
    fn fun_0215a55c(&mut self, init: &[u8; 16], heap: u32) -> u32;
    /// C: `FUN_0215A5A4(ctx)`: free a scrcmd CTX.
    fn fun_0215a5a4(&mut self, ctx: u32);
    /// C: `FUN_0215A760(ctx, type, msg)`: open the script's message bank (text side of `ScriptHost`).
    fn fun_0215a760(&mut self, ctx: u32, ty: u32, msg: u32);
    /// C: `FUN_02162450(x)`: the zone's special-script table (6-byte records).
    fn fun_02162450(&mut self, x: u32) -> u32;
    /// C: `FUN_0216D588(mmdl)`: the OBJ id of a field object (fldmmdl, F4a).
    fn fun_0216d588(&mut self, mmdl: u32) -> u32;

    // ---- overlay 21 (field map; command 311)
    /// C: `FUN_02188C54(fieldmap)`.
    fn fun_02188c54(&mut self, fieldmap: u32) -> u32;
    /// C: `FUN_02188C84(fieldmap)`.
    fn fun_02188c84(&mut self, fieldmap: u32) -> u32;
    /// C: `FUN_02188CCC(fieldmap)`.
    fn fun_02188ccc(&mut self, fieldmap: u32) -> u32;
    /// C: `FUN_021A0F90(x)`: a state value (command 311 special-cases 7).
    fn fun_021a0f90(&mut self, x: u32) -> u32;
    /// C: `FUN_021A1524(x)`.
    fn fun_021a1524(&mut self, x: u32);
    /// C: `FUN_021A1530(x)`.
    fn fun_021a1530(&mut self, x: u32);
    /// C: `FUN_021A153C(x)`.
    fn fun_021a153c(&mut self, x: u32) -> u32;
    /// C: `FUN_021A1548(x)`.
    fn fun_021a1548(&mut self, x: u32);
    /// C: `FUN_021D1164(a, fieldmap)`: create the object command 311 keeps at SW + 0x13C.
    fn fun_021d1164(&mut self, a: u32, fieldmap: u32) -> u32;
    /// C: `FUN_021D1194(x)`: free it.
    fn fun_021d1194(&mut self, x: u32);
    /// C: `FUN_021D11B0(x)`.
    fn fun_021d11b0(&mut self, x: u32);
    /// C: `FUN_021D11D8(x)`.
    fn fun_021d11d8(&mut self, x: u32);
}
