//! Host models: the system boundary of the F12 machine (each one documented; the same boundary as
//! `tools/rom-diff/battle_diff.py`'s shims where both have one). Everything above it — the gfl archive
//! code, save data, the GMEVENT / proc / task systems, MI / FX helpers — runs as original code or as a
//! port.

use super::cpu::Cpu;
use super::Env;
use std::collections::{BTreeMap, HashMap};

/// The system state behind the host models.
#[derive(Default)]
pub struct HostState {
    pub heap: HostHeap,
    /// Open files (FSFile address -> (ARC id, bytes)).
    pub files: HashMap<u32, (u32, Vec<u8>)>,
    /// OS_GetTick.
    pub tick: u64,
    /// Keys held in the running frame (DS KEYINPUT bit order, set = pressed).
    pub held: u32,
    /// Reads / writes of I/O registers without a model (diagnostics).
    pub io_unknown: BTreeMap<u32, u64>,
    /// Sound effects / BGM requests (frame, id).
    pub sounds: Vec<(u32, u32)>,
    pub frame: u32,
    /// The messages printed into the headless windows (`view.rs`).
    pub printed: Vec<super::view::Printed>,
}

/// `(address, overlay (0 = ARM9 main), what)` of every host model.
pub const HOSTS: &[(u32, u16, &str)] = &[
    // ---- gfl heap (one host arena per heap id: HostHeap) ----
    (
        0x0203_064C,
        0,
        "Heap_Create(parent, id, size): the heap carved out of its parent",
    ),
    (0x0203_0678, 0, "Heap_CreateAt(addr, size, id): 1"),
    (
        0x0203_06F0,
        0,
        "Heap_Destroy(id): its block back to the parent",
    ),
    (
        0x0203_0734,
        0,
        "Heap_AllocDebug(heap, size, flag, file, line): a zeroed host block",
    ),
    (0x0203_07B0, 0, "Heap_Free(p)"),
    (
        0x0203_05C4,
        0,
        "OS heap alloc(size) (OS_AllocFromHeap of the main arena): a zeroed host block",
    ),
    (0x0203_05E4, 0, "OS heap free(p): 1"),
    (
        0x0203_07C8,
        0,
        "Heap_InitAllocator(NNSFndAllocator*, heap, align): an allocator over the host heap",
    ),
    (
        ALLOCATOR_ALLOC,
        0,
        "the host allocator's pfAlloc(allocator, size)",
    ),
    (
        ALLOCATOR_FREE,
        0,
        "the host allocator's pfFree(allocator, p)",
    ),
    (
        0x0203_07E0,
        0,
        "Heap_ResizeMemoryBlock(block, size): shrink in place, 1",
    ),
    (
        0x0203_0830,
        0,
        "Heap_GetTotalFreeSize(heap): the host arena's free bytes",
    ),
    // ---- overlays ----
    (
        0x0203_4AC4,
        0,
        "overlay load(id): copy OVY_<id>.sbin to its link address, zero its BSS",
    ),
    (0x0203_4A5C, 0, "overlay unload(id)"),
    // ---- keys / touch / clock ----
    (0x0203_5C60, 0, "touch panel: none"),
    (0x0208_A924, 0, "TP_Init (ARM7 via PXI): nothing"),
    (
        0x0208_A998,
        0,
        "TP_GetUserInfo(&calib): 0 = none (the defaults apply)",
    ),
    (0x0208_AA2C, 0, "TP_SetCalibrateParam(&calib): nothing"),
    (0x0208_AB1C, 0, "TP_RequestSamplingAsync: nothing"),
    (
        0x0208_ABB8,
        0,
        "TP_WaitRawResult(&tp): no touch (x, y, touch, validity = 0), 0",
    ),
    (
        0x0208_BFD0,
        0,
        "ARM7 system value query (k, &out) (PXI): out = 0",
    ),
    (
        0x0208_70D0,
        0,
        "OS_GetTick: TICKS_PER_FRAME a frame, one more per call within it",
    ),
    (
        0x0208_7AE4,
        0,
        "OS_WaitVBlankIntr: returns at once (no interrupts here)",
    ),
    (0x0203_F258, 0, "MathRand seed from the RTC: a fixed seed"),
    // ---- file system below the gfl archive code ----
    (0x0204_8DDC, 0, "FS open ARC by id (FSFile, arc)"),
    (
        0x0202_0DEC,
        0,
        "FS open ARC by id (FSFile, arc) (the whole-file loader's path)",
    ),
    (0x0207_8B44, 0, "FS_SeekFile(file, offset, whence)"),
    (0x0207_8B70, 0, "FS_ReadFile(file, dst, len)"),
    (0x0207_8A80, 0, "FS_CloseFile(file)"),
    (0x0207_8C28, 0, "FS_GetFileLength(file): bottom - top"),
    // ---- D9: the wireless / comm library init of the boot (overlays 9 / 13, PXI to the ARM7) ----
    (0x0203_CB34, 0, "comm library init (D9): nothing"),
    (
        0x0201_07CC,
        0,
        "boot: start an ARM7 service over PXI on the 0x02010794 work: nothing",
    ),
    // ---- overlay 228: the backup (flash) device probe of the save-data boot (02006F20) ----
    (
        0x021D_FC20,
        228,
        "backup probe(heap): a zeroed 0x400 probe object (no device type)",
    ),
    (0x021D_FDB8, 228, "free the probe object"),
    // ---- overlay 230: the cartridge integrity checks (ROM / card reads with thread waits): a genuine cartridge
    // answers `!callback` for `(invoker, callback, x)` (event_battle / event_mapchange docs) ----
    (0x0218_82A0, 230, "integrity check: !r1"),
    (0x0218_82DC, 230, "integrity check: !r1"),
    (0x0218_8318, 230, "integrity check: !r1"),
    (0x0218_8354, 230, "integrity check: !r1"),
    (0x0218_8390, 230, "integrity check: !r1"),
    (0x0218_83CC, 230, "integrity check: !r1"),
    (
        0x0200_6300,
        0,
        "sound: start the sound-data load thread (OS thread): nothing to load",
    ),
    (
        0x0200_63B4,
        0,
        "sound: BGM data load + start step (bgm, &state, reset): done at once (1; 0 for a reset)",
    ),
    // ---- D3: the per-frame / teardown render-only functions of fieldmap.c (crates/black-field README "F2a-spec":
    // draw dispatch, 3D draw, VBlank, teardown; the set-up ones run as original code: they create the BG system
    // and the camera / light objects that other subsystems use) ----
    (0x0218_8DB0, 21, "render"),
    (0x0218_8DF0, 21, "render"),
    (0x0218_8E38, 21, "render"),
    (0x0218_9084, 21, "render"),
    (0x0218_90AC, 21, "render"),
    (0x0218_90C8, 21, "render"),
    (0x0218_9108, 21, "render"),
    (0x0218_9140, 21, "render"),
    (0x0218_9154, 21, "render"),
    (0x0218_91C0, 21, "render"),
    (0x0218_97DC, 21, "render"),
    (0x0218_9810, 21, "render"),
    (0x0218_9998, 21, "render"),
    (0x0218_99C4, 21, "render"),
    // ---- D3 / G8: the entrance camera of the door events (`entrance_camera.c`: a camera animation from ARC 179) ----
    (
        0x021B_DCC8,
        21,
        "entrance camera start (camera, record): no camera animation",
    ),
    // ---- sound (nothing plays) ----
    (0x0200_5CA4, 0, "sound"),
    (0x0200_5D28, 0, "sound"),
    (0x0200_5D90, 0, "sound"),
    (0x0200_5DF8, 0, "sound"),
    (0x0200_5E30, 0, "sound"),
    (0x0200_6734, 0, "sound"),
    (0x0200_6740, 0, "sound"),
    (0x0200_5E50, 0, "sound"),
    (0x0200_61E4, 0, "play SE(se)"),
    (0x0200_57AC, 0, "sound"),
    (0x0200_57C4, 0, "sound: 1"),
];

/// Address ranges whose every entry is a host model answering a constant: `(lo, hi, answer, what)`.
pub const RANGES: &[(u32, u32, u64, &str)] = &[
    (
        0x0207_26D0,
        0x0207_5C00,
        0,
        "the NNS sound library (ARM; its globals 0x0214B000..0x0214F000): no sound hardware, every call answers 0",
    ),
    (
        0x0200_57AC,
        0x0200_67E4,
        0,
        "the gfl sound module (BGM / SE players over NNS sound; its system init is not run): every call answers 0 \
         (nothing plays, nothing is pending), apart from the entries listed in HOSTS",
    ),
];

/// Host code addresses (an unused page above overlay 21's BSS, inside the hook range): the function table of
/// the NNS allocators `Heap_InitAllocator` sets up.
pub const HOST_PAGE: u32 = 0x021F_F000;
pub const ALLOCATOR_ALLOC: u32 = HOST_PAGE;
pub const ALLOCATOR_FREE: u32 = HOST_PAGE + 0x10;
/// `NNSFndAllocatorFunc { pfAlloc, pfFree }` of the host allocator.
pub const ALLOCATOR_FUNCS: u32 = HOST_PAGE + 0x100;

/// OS_GetTick ticks a frame.
pub const TICKS_PER_FRAME: u64 = 8753;

/// Seed of the MathRand context (`0203F258`; `battle_diff.py` RNG_SEED).
pub const RNG_SEED: u64 = 0x0123_4567_89AB_CDEF;

/// A block of the root arena (the OS heap, the overlay-228 allocator).
fn alloc(env: &mut Env, size: u32) -> u32 {
    alloc_in(env, ROOT_HEAP, size, 4)
}

/// A zeroed block of `size` bytes in gfl heap `heap` (the root arena when the heap was not created).
fn alloc_in(env: &mut Env, heap: u32, size: u32, align: u32) -> u32 {
    assert!(
        size < 0x40_0000 && align.is_power_of_two(),
        "F12: allocation of {size:#x} bytes (align {align:#x}) (calls {})",
        env.call_stack()
    );
    let Some(base) = env.host.heap.alloc(heap, size, align) else {
        panic!(
            "F12: heap {heap:#x} exhausted by {size:#x} bytes (calls {})",
            env.call_stack()
        );
    };
    if std::env::var("F12_HEAPLOG").is_ok() {
        eprintln!(
            "HEAP f{} {heap:x} {size:x} {align:x} {base:08x} {}",
            env.host.frame,
            env.call_stack()
        );
    }
    env.fmem().fill(base, size, 0);
    base
}

/// The arena of the heaps that were not created through `Heap_Create` (the boot's system heaps, the OS heap).
pub const ROOT_HEAP: u32 = u32::MAX;
/// The bytes a created heap keeps at its start.
const HEAP_HEADER: u32 = 0x40;

/// One heap of the host: first fit in address order over the freed spans (coalesced), else the top; every
/// block is followed by a 16-byte gap.
pub struct Arena {
    lo: u32,
    top: u32,
    end: u32,
    /// Live blocks: address -> (size, span).
    pub used: BTreeMap<u32, (u32, u32)>,
    free: BTreeMap<u32, u32>,
    /// `Heap_Create`: (parent heap, the block of the parent that holds this heap).
    parent: Option<(u32, u32)>,
}

impl Arena {
    fn new(lo: u32, end: u32, parent: Option<(u32, u32)>) -> Self {
        Arena {
            lo,
            top: lo,
            end,
            used: BTreeMap::new(),
            free: BTreeMap::new(),
            parent,
        }
    }

    fn alloc(&mut self, size: u32, align: u32) -> Option<u32> {
        let span = ((size.max(1) + 3) & !3) + 0x10;
        let up = |a: u32| (a + align - 1) & !(align - 1);
        let hit = self
            .free
            .iter()
            .map(|(&a, &s)| (a, s))
            .find(|&(a, s)| up(a) + span <= a + s);
        let base = if let Some((a, s)) = hit {
            self.free.remove(&a);
            let base = up(a);
            if base > a {
                self.free.insert(a, base - a);
            }
            let end = a + s;
            if end > base + span {
                self.free.insert(base + span, end - base - span);
            }
            base
        } else {
            let base = up(self.top);
            if base + span > self.end {
                return None;
            }
            if base > self.top {
                self.free.insert(self.top, base - self.top);
            }
            self.top = base + span;
            base
        };
        self.used.insert(base, (size, span));
        Some(base)
    }

    /// Frees `p`; false when it is not a live block of this arena.
    fn free(&mut self, p: u32) -> bool {
        let Some((_, span)) = self.used.remove(&p) else {
            return false;
        };
        let (mut a, mut s) = (p, span);
        if let Some((&pa, &ps)) = self.free.range(..p).next_back() {
            if pa + ps == p {
                self.free.remove(&pa);
                a = pa;
                s += ps;
            }
        }
        if let Some(&ns) = self.free.get(&(a + s)) {
            self.free.remove(&(a + s));
            s += ns;
        }
        if a + s == self.top {
            self.top = a;
        } else {
            self.free.insert(a, s);
        }
        true
    }
}

/// The host heaps, one arena per gfl heap id as the game creates them (`Heap_Create` carves the heap out of
/// its parent, `Heap_Destroy` returns it), so a heap that is destroyed and created again (the field heaps
/// across a map change) hands out the same addresses as before, as the game's heaps do: the field code keeps
/// pointers across that (the map change's FIELDMAP pointer). Blocks are zeroed. Deterministic.
pub struct HostHeap {
    pub arenas: BTreeMap<u32, Arena>,
}

impl Default for HostHeap {
    fn default() -> Self {
        let root = Arena::new(super::HEAP_BASE, super::HEAP_END, None);
        HostHeap {
            arenas: BTreeMap::from([(ROOT_HEAP, root)]),
        }
    }
}

impl HostHeap {
    fn arena(&mut self, heap: u32) -> &mut Arena {
        let k = if self.arenas.contains_key(&heap) {
            heap
        } else {
            ROOT_HEAP
        };
        self.arenas.get_mut(&k).unwrap()
    }

    pub fn alloc(&mut self, heap: u32, size: u32, align: u32) -> Option<u32> {
        self.arena(heap).alloc(size, align)
    }

    /// Frees `p` (in whichever heap holds it); false when it is not a live block.
    pub fn free(&mut self, p: u32) -> bool {
        self.arenas.values_mut().any(|a| a.free(p))
    }

    /// The (size, span) of live block `p`.
    pub fn block_mut(&mut self, p: u32) -> Option<&mut (u32, u32)> {
        self.arenas.values_mut().find_map(|a| a.used.get_mut(&p))
    }

    /// `Heap_Create(parent, id, size)`: false when the parent has no room.
    pub fn create(&mut self, parent: u32, id: u32, size: u32) -> bool {
        assert!(
            !self.arenas.contains_key(&id),
            "F12: heap {id:#x} created twice"
        );
        let Some(b) = self.alloc(parent, size, 16) else {
            return false;
        };
        let parent = if self.arenas.contains_key(&parent) {
            parent
        } else {
            ROOT_HEAP
        };
        // the heap's own header first (as the NNS heap's), so that no block of the heap has the address of the
        // parent's block that holds it
        self.arenas
            .insert(id, Arena::new(b + HEAP_HEADER, b + size, Some((parent, b))));
        true
    }

    /// `Heap_CreateAt(addr, size, id)`: a heap over memory the caller owns.
    pub fn create_at(&mut self, addr: u32, size: u32, id: u32) {
        assert!(
            !self.arenas.contains_key(&id),
            "F12: heap {id:#x} created twice"
        );
        self.arenas.insert(id, Arena::new(addr, addr + size, None));
    }

    /// `Heap_Destroy(id)`: its blocks go with it.
    pub fn destroy(&mut self, id: u32) {
        if let Some(a) = self.arenas.remove(&id) {
            if let Some((parent, b)) = a.parent {
                self.arenas.get_mut(&parent).map(|p| p.free(b));
            }
        }
    }

    /// The free bytes of `heap` (as `Heap_GetTotalFreeSize`, up to the 16-byte gaps).
    pub fn free_size(&mut self, heap: u32) -> u32 {
        let a = self.arena(heap);
        let used: u32 = a.used.values().map(|v| v.1).sum();
        a.end - a.lo - used
    }
}

/// Runs the host model of `a` with the argument words `w`.
pub fn call(env: &mut Env, a: u32, w: &[u32]) -> u64 {
    if super::view::WINDOWS.iter().any(|v| v.0 == a) {
        return super::view::call(env, a, w);
    }
    let arg = |i: usize| w.get(i).copied().unwrap_or(0);
    match a {
        0x0203_064C => env.host.heap.create(arg(0), arg(1), arg(2)) as u64,
        0x0203_0678 => {
            env.host.heap.create_at(arg(0), arg(1), arg(2));
            1
        }
        0x0203_06F0 => {
            env.host.heap.destroy(arg(0));
            0
        }
        0x0203_0734 => alloc_in(env, arg(0), arg(1), 4) as u64,
        0x0203_05C4 => alloc(env, arg(0)) as u64,
        0x0203_07C8 => {
            // NNSFndAllocator { pFunc, pHeap, heapParam1 (align), heapParam2 }
            let a = arg(0);
            env.w32(ALLOCATOR_FUNCS, ALLOCATOR_ALLOC | 1);
            env.w32(ALLOCATOR_FUNCS + 4, ALLOCATOR_FREE | 1);
            env.w32(a, ALLOCATOR_FUNCS);
            env.w32(a + 4, arg(1));
            env.w32(a + 8, arg(2));
            env.w32(a + 12, 0);
            0
        }
        ALLOCATOR_ALLOC => {
            // a negative alignment allocates from the heap's tail in NNS: here the same first fit
            let align = (env.r32(arg(0) + 8) as i32).unsigned_abs().max(4);
            let heap = env.r32(arg(0) + 4);
            alloc_in(env, heap, arg(1), align) as u64
        }
        0x021B_DCC8 | 0x0208_7AE4 => 0,
        ALLOCATOR_FREE => {
            env.host.heap.free(arg(1));
            0
        }
        0x0203_07E0 => {
            let (p, n) = (arg(0), arg(1));
            let cur = env
                .host
                .heap
                .block_mut(p)
                .unwrap_or_else(|| panic!("F12: resize of unknown block {p:#x}"));
            assert!(
                n <= cur.0,
                "F12: Heap_ResizeMemoryBlock grows {p:#x} {:#x} -> {n:#x}",
                cur.0
            );
            cur.0 = n;
            1
        }
        0x0203_0830 => env.host.heap.free_size(arg(0)) as u64,
        0x0203_05E4 => {
            env.host.heap.free(arg(0));
            1
        }
        0x0203_07B0 => {
            let p = arg(0);
            if p != 0 {
                assert!(
                    env.host.heap.free(p),
                    "F12: Heap_Free of unknown block {p:#010x} (calls {})",
                    env.call_stack()
                );
            }
            0
        }
        0x0203_4AC4 => {
            let n = arg(0) as u16;
            if std::env::var_os("F12_DEBUG").is_some() {
                eprintln!("F12: overlay load {n} ({})", env.call_stack());
            }
            if !env.loaded.contains(&n) {
                env.load_overlay(n);
            }
            1
        }
        0x0203_4A5C => {
            env.unload_overlay(arg(0) as u16);
            1
        }
        0x0203_5C60 => 0,
        0x0208_A924 | 0x0208_A998 | 0x0208_AA2C | 0x0208_AB1C => 0,
        0x0208_ABB8 => {
            env.write_bytes(arg(0), &[0; 8]);
            0
        }
        0x0208_BFD0 => {
            env.w32(arg(1), 0);
            0
        }
        0x0208_70D0 => {
            // the DS's 33.513982 MHz / 64 timer at 59.8261 frames a second: 8753 ticks a frame, one more each
            // call within a frame (so the time is the frame's, whichever code asks how often)
            env.host.tick = (env.host.tick + 1).max(env.host.frame as u64 * TICKS_PER_FRAME);
            env.host.tick
        }
        0x0203_F258 => {
            let ctx = arg(0);
            let mut b = Vec::new();
            for v in [RNG_SEED, 0x5D58_8B65_6C07_8965, 0x0026_9EC3] {
                b.extend_from_slice(&v.to_le_bytes());
            }
            env.write_bytes(ctx, &b);
            0
        }
        0x0204_8DDC | 0x0202_0DEC => fs_open(env, arg(0), arg(1)),
        0x0207_8B44 => fs_seek(env, arg(0), arg(1), arg(2)),
        0x0207_8B70 => fs_read(env, arg(0), arg(1), arg(2)),
        0x0207_8C28 => (env.r32(arg(0) + 0x28) - env.r32(arg(0) + 0x24)) as u64,
        0x0207_8A80 => {
            env.host.files.remove(&arg(0));
            1
        }
        0x0200_61E4 => {
            let f = env.host.frame;
            env.host.sounds.push((f, arg(0)));
            0
        }
        0x0200_57C4 => 1,
        0x0203_CB34 | 0x0201_07CC => 0,
        0x0200_6300 => 0,
        0x0200_63B4 => (arg(2) != 1) as u64,
        0x021D_FC20 => alloc(env, 0x400) as u64,
        0x0218_82A0 | 0x0218_82DC | 0x0218_8318 | 0x0218_8354 | 0x0218_8390 | 0x0218_83CC => {
            !arg(1) as u64
        }
        0x021D_FDB8 => {
            env.host.heap.free(arg(0));
            0
        }
        _ if HOSTS
            .iter()
            .any(|h| h.0 == a && matches!(h.2, "sound" | "NNS snd: 0" | "render")) =>
        {
            0
        }
        _ => panic!("F12: host model {a:#010x} listed but not implemented"),
    }
}

// FSFile (NitroSDK, 0x48 bytes): top / bottom / pos at +0x24 / +0x28 / +0x2C; the bytes stay host-side.
fn fs_open(env: &mut Env, f: u32, arc: u32) -> u64 {
    if std::env::var_os("F12_DEBUG").is_some() {
        eprintln!("F12: open ARC {arc} ({})", env.call_stack());
    }
    let path = env.rom.arc_path(arc);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("F12: ARC {arc}: {e}"));
    env.write_bytes(f, &[0; 0x48]);
    env.w32(f + 0x28, data.len() as u32);
    env.host.files.insert(f, (arc, data));
    1
}

fn fs_seek(env: &mut Env, f: u32, off: u32, whence: u32) -> u64 {
    assert!(
        env.host.files.contains_key(&f),
        "F12: FS_SeekFile on a closed file"
    );
    let (top, bottom, pos) = (env.r32(f + 0x24), env.r32(f + 0x28), env.r32(f + 0x2C));
    let base = match whence {
        0 => top,
        1 => pos,
        2 => bottom,
        _ => panic!("F12: FS_SeekFile whence {whence}"),
    };
    let p = base as i64 + off as i32 as i64;
    if p < top as i64 || p > bottom as i64 {
        return 0;
    }
    env.w32(f + 0x2C, p as u32);
    1
}

fn fs_read(env: &mut Env, f: u32, dst: u32, n: u32) -> u64 {
    let (pos, bottom) = (env.r32(f + 0x2C), env.r32(f + 0x28));
    let n = n.min(bottom - pos);
    let data = env
        .host
        .files
        .get(&f)
        .expect("F12: FS_ReadFile on a closed file")
        .1[pos as usize..(pos + n) as usize]
        .to_vec();
    env.write_bytes(dst, &data);
    env.w32(f + 0x2C, pos + n);
    n as u64
}

// ---------------------------------------------------------------- I/O

/// The I/O page (a memory window at 0x04000000, so the ports that toggle EXMEMCNT see the same register): the
/// division / square-root coprocessor is computed and DMA runs at once on a write; IME / IE / IF read 0; every
/// other register reads back what was written (accesses counted in `io_unknown`).
pub fn io_read(env: &mut Env, a: u32, n: u32) -> u32 {
    if matches!(a, 0x0400_0208 | 0x0400_0210 | 0x0400_0214) {
        return 0;
    }
    if !((0x0400_0280..0x0400_02C0).contains(&a) || a == 0x0400_0204) {
        *env.host.io_unknown.entry(a).or_default() += 1;
    }
    io_bytes(env, a, n) as u32
}

pub fn io_write(env: &mut Env, a: u32, v: u32, n: u32) {
    io_set(env, a, v as u64, n);
    if (0x0400_0280..0x0400_02C0).contains(&a) {
        math(env);
        return;
    }
    if matches!(a, 0x0400_0204 | 0x0400_0208 | 0x0400_0210 | 0x0400_0214) {
        return;
    }
    if (0x0400_00B0..0x0400_00E0).contains(&a) {
        let ch = (a - 0x0400_00B0) / 12;
        dma(env, ch);
        return;
    }
    *env.host.io_unknown.entry(a).or_default() += 1;
}

/// A DMA channel whose control word has the enable bit runs at once (every start mode; a transfer into the
/// I/O page, e.g. the geometry FIFO, is dropped) and its enable bit clears.
fn dma(env: &mut Env, ch: u32) {
    let base = 0x0400_00B0 + 12 * ch;
    let cnt = io_bytes(env, base + 8, 4) as u32;
    if cnt >> 31 == 0 {
        return;
    }
    let (mut src, mut dst) = (
        io_bytes(env, base, 4) as u32,
        io_bytes(env, base + 4, 4) as u32,
    );
    let n = match cnt & 0x1F_FFFF {
        0 => 0x20_0000,
        k => k,
    };
    let unit = if cnt & (1 << 26) != 0 { 4 } else { 2 };
    let step = |ctl: u32| -> i64 {
        match ctl {
            1 => -(unit as i64),
            2 => 0,
            _ => unit as i64,
        }
    };
    let (ds, ss) = (step((cnt >> 21) & 3), step((cnt >> 23) & 3));
    if !(0x0400_0000..0x0500_0000).contains(&dst) {
        for _ in 0..n {
            let v = super::cpu::Bus::rd(env, src, unit);
            super::cpu::Bus::wr(env, dst, v, unit);
            src = (src as i64 + ss) as u32;
            dst = (dst as i64 + ds) as u32;
        }
    }
    io_set(env, base + 8, (cnt & 0x7FFF_FFFF) as u64, 4);
}

fn io_bytes(env: &mut Env, a: u32, n: u32) -> u64 {
    let m = env.fmem();
    (0..n).fold(0u64, |v, i| v | (m.r8(a + i) as u64) << (8 * i))
}

fn io_set(env: &mut Env, a: u32, v: u64, n: u32) {
    let m = env.fmem();
    for i in 0..n {
        m.w8(a + i, (v >> (8 * i)) as u8);
    }
}

/// DIVCNT / DIV_NUMER / DIV_DENOM -> DIV_RESULT / DIVREM_RESULT, SQRTCNT / SQRT_PARAM -> SQRT_RESULT (always
/// ready; division by 0 gives +-1 / the numerator), as `battle_diff.py`.
fn math(env: &mut Env) {
    let cnt = io_bytes(env, 0x0400_0280, 2) & 3;
    let num = io_bytes(env, 0x0400_0290, 8);
    let den = io_bytes(env, 0x0400_0298, 8);
    let (n, d): (i128, i128) = match cnt {
        0 => (num as u32 as i32 as i128, den as u32 as i32 as i128),
        1 => (num as i64 as i128, den as u32 as i32 as i128),
        _ => (num as i64 as i128, den as i64 as i128),
    };
    let (q, r) = if d == 0 {
        (if n >= 0 { -1 } else { 1 }, n)
    } else {
        (n / d, n % d)
    };
    io_set(env, 0x0400_02A0, q as u64, 8);
    io_set(env, 0x0400_02A8, r as u64, 8);
    let c = io_bytes(env, 0x0400_0280, 2) & !0x8000;
    io_set(env, 0x0400_0280, c, 2);
    let scnt = io_bytes(env, 0x0400_02B0, 2) & 1;
    let mut p = io_bytes(env, 0x0400_02B8, 8);
    if scnt == 0 {
        p &= 0xFFFF_FFFF;
    }
    io_set(env, 0x0400_02B4, isqrt(p), 4);
    let c = io_bytes(env, 0x0400_02B0, 2) & !0x8000;
    io_set(env, 0x0400_02B0, c, 2);
}

fn isqrt(v: u64) -> u64 {
    if v == 0 {
        return 0;
    }
    let mut x = (v as f64).sqrt() as u64;
    while x * x > v {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= v {
        x += 1;
    }
    x
}

// ---------------------------------------------------------------- BIOS

/// The BIOS calls the field code makes (SWI numbers of the ARM9 BIOS).
pub fn swi(env: &mut Env, c: &mut Cpu, n: u32) {
    match n {
        // WaitByLoop / Halt / IntrWait / VBlankIntrWait: nothing to wait for
        0x03..=0x06 => {}
        // Div
        0x09 => {
            let (a, b) = (c.r[0] as i32, c.r[1] as i32);
            assert!(b != 0, "F12: BIOS Div by 0");
            let q = a.wrapping_div(b);
            c.r[0] = q as u32;
            c.r[1] = a.wrapping_rem(b) as u32;
            c.r[3] = q.unsigned_abs();
        }
        // Sqrt
        0x0D => c.r[0] = isqrt(c.r[0] as u64) as u32,
        _ => panic!(
            "F12: BIOS call {n:#x} at {:#010x} not modelled (calls {})",
            c.r[15],
            env.call_stack()
        ),
    }
}
