//! Battle main module and server-object create (`asm/battle_server.s`, overlay 93, 0x021B6100-0x021BAA94; B11).
//!
//! # What the range is (B11-spec)
//!
//! The decomp calls the file `battle_server.s`; the code up to 0x021BA9C8 reads like `btl_main.c` (the battle `GameProc`
//! and the *main module* M every other battle file reaches through getters), and `btl_server.c` proper starts at
//! 0x021BA9C8 with the create of the server object (`Heap_AllocDebug(heap, 0xCC4, 1, "btl_server.c", 0xAD)`); the rest of
//! `btl_server.c` is [`crate::server_flow::server_side`] (0x021BAA94+). File naming UNVERIFIED.
//!
//! 212 function entries (Thumb): 207 real functions + 5 thunks (`ldr r3,[pc]; bx r3`, not specced, callers stub the
//! target): 021b8ddc -> 021b8d08, 021b8de4 -> 021b8cfc, 021b917c -> 021ba54c, 021b98a4 -> 021b98ac, 021b99b8 -> 021b6994.
//! No `bx pc` veneers. The asm / Ghidra label another 28 entries that are pieces of bigger functions (switch cases,
//! `.byte` blocks: 6444, 6536..66a4, 6c1a, 7392, 73e2..744e, 74e6, 7576, 75c6, 7778, 798e, 7bd4, 7ce6, 7e3c / 7e3e, 80a2,
//! 8210, 837e, 845e, 8502). The asm has no label for 37 real entries that are reached only through pointers or are `.byte`
//! blocks (the sub-procs 6aac..71d4, 734c..7518, 75a8..835c, the loops 83f4..84d8, and 6434 8c5c 91e4 9234 994c 9c10 9c98
//! ba2d4 ba5c4 ba5d0 plus the thunks); Ghidra has no function at 021b6500 (proc exit). Entry list: walk of the Thumb control
//! flow from every candidate (asm labels, Ghidra, rizin), jump tables decoded; nothing in the range is left unreached but
//! alignment padding.
//!
//! # Split (svflow areas, `nowork`, specs in `tools/rom-diff/svflow_specs/serverN.spec`)
//!
//! Cut points from a DP over the call graph that minimises crossing call edges: 150 of the 349 in-range call edges cross
//! (the optimum for 3 areas is 149 at 0x021B7518; 0x021B75A8 is the start of the sub-proc tables). The graph is hub-heavy
//! (the getters 021b8588 / 8b9c / 8c50 / 9a40 ... are called from every part), so more areas only add crossings.
//!
//! | area | range | fns | instructions | what |
//! |---|---|---|---|---|
//! | `server1` | 021B6100-021B75A8 | 24 | 2302 | BattleProc init 6100 / main 6434 / exit 6500 (proc table words 0x021B6101 / 6435 / 6501 at the end of `battle_server_rodata`), sub-proc selection 66d4, the local setup sub-procs 6aac / 6c5c / 6dd0 / 7050 / 71d4 and the comm ones 734c / 73c4 / 7488 / 7518, own-client / party / view / record-reader helpers |
//! | `server2` | 021B75A8-021B902C | 67 | 2896 | sub-proc steps `step(M, &seq)` of the tables at 0x021EFD00.. (comm handshake 75a8 7758 7968 7bb0; server + client create 7cb4 7e18 8070 81dc; view + main loop 835c), per-frame loops 83f4 8434 84d8, the main-module getters 8548..8750, target expansion 8778 / 87e8 / 8834 / 88ec, position <-> client-id maps 8b78..8e28, side / partner helpers 8e70..9010 |
//! | `server3` | 021B902C-021BAA94 | 116 | 2932 | main-module helpers part 2 (money-like counters, Illusion handling), pokecon (`021b93e0` init, client setup 9460, write-back 969c / 97a0), party container (`BattleParty_*` 9b40..9ce8), player records 9db8..ba064, party copies ba094..ba178, battle-end results ba2e4 / ba54c / ba878, teardown ba73c, and the `btl_server.c` create 021ba9c8 |
//!
//! All 207 functions are specced; ROM-side instruction coverage 8130 / 8130 (100%, every function), `svflow.py serverN
//! --all --strict-callees` clean. **Closure stubs:** every block lists the stubs of its callees and of the callees of all its
//! in-area descendants (checked by running every function with the whole area treated as ported: no fault, no stray write, no
//! unstubbed callee), so porting in any order needs no spec edit. Callees in another B11 area or another svflow area
//! (server_side, core1-6, api / hem / handex, events, cmd_queue, client, adapter) are `force` stubs (the Rust port keeps reaching
//! them through `env.call`); ARM9 main, battle_net (021d4xxx), BattleMon (021d5xxx), view (021e8xxx-021e9xxx) and record
//! reader callees are plain stubs. Function-pointer fields are fixed to fake callee addresses in the unused page
//! 0x02EE0000-0x02EEFFFF (stubbed there, e.g. `stub 02ee0010`); a port calls `env.call(ptr & !1, ..)`.
//!
//! **Traps for the port job:** the other areas' specs stub 81 functions of this range (652 stub lines, none `force`); each
//! needs `force` in those specs when it is ported here (list and arity/kind mismatches: B11-spec report).
//!
//! # Objects (from the disassembly; spec arenas in brackets)
//!
//! **M, main module** (0x474 bytes; allocated by 021b6100 with `0203159C(heap, 0x474, 0x12)`, cleared by `02082A7C`)
//! [`arg buf 0x1000`, M at 0, BATTLE_SETUP at arena 0x480]:
//!
//! | ofs | field |
//! |---|---|
//! | `+0x000` | `BATTLE_SETUP *` (btl_setup.c, 0xF0 bytes) |
//! | `+0x004` | battle view (021e8f20 result, freed by 021e9034) |
//! | `+0x008` / `+0x00C` | server object S (server console / client console; 021ba9c8) |
//! | `+0x010..+0x01C` | 4 client objects (021cd6f0) |
//! | `+0x020 + 0x28 k` | 4 player records (+0 ptr, NULL = empty; +4 name StrBuf; +8 / +0xA u16; +0xC u32; +0x10 u16[4]; +0x18 / +0x20 8 bytes) |
//! | `+0x0C0` | word = setup+0x34 |
//! | `+0x0C4..+0x0C7` | u8 per-client done flags |
//! | `+0x0C8` / `+0x1B0` | pokecon 1 / 0 (0xE8 bytes each, inline) |
//! | `+0x298` / `+0x2A8` / `+0x2B8` / `+0x2BC` | party copies (4 + 4 + 1 + 1, 0201a920) |
//! | `+0x2C0` | Pokemon for the Illusion disguise (020173f0) |
//! | `+0x2C4` / `+0x2C8` | comm data handle / BattleRecordReader |
//! | `+0x3E0 + 4 i` | 4 received blocks / view-side objects |
//! | `+0x3F0` | 0x18-byte block (0203F258 or a copy of setup+0x9C) handed to the server / client creates |
//! | `+0x408` | 0x20-byte comm record |
//! | `+0x428..+0x42D` | u8[6] battle position -> client id |
//! | `+0x430` / `+0x434` / `+0x438` | words (021d7f88(setup); money-like counter capped at 99999; 9244 result); main writes setup+0x88 from them |
//! | `+0x43C` | 02008374(setup+0x5C) |
//! | `+0x440` / `+0x442` | u16 = setup+0x74 / +0x76 |
//! | `+0x444` | result code (7 = none yet) |
//! | `+0x448` | 021d8230 object / 2-word result pair |
//! | `+0x450` | sub-proc slot {+0 fn, +4 fn 2, +8 arg, +0xC state}; run as `fn(M + 0x45C, M[0x458])` |
//! | `+0x460` | sub-state / brightness |
//! | `+0x464` | main-loop function `fn(M)` (8434 / 84d8 / 83f4) |
//! | `+0x468` | u16 heap id (0x12) |
//! | `+0x46A..+0x46F` | u8: client count, own client id (= setup+0x21), side, "is server", 020083C8 != 0 |
//! | `+0x470..+0x472` | u8 loop counters |
//! | `+0x473` | flag bits (0 done / abort, 1, 2 = 020083AC == 0, 3 / 4 / 5 / 6 one-shot and doubling flags of 919c / 9244 / 9210, 7) |
//!
//! **BATTLE_SETUP** fields used by this range: `+0x00` battle type (0..4), `+0x04` rule (0..3), `+0x08`, `+0x1A` u16,
//! `+0x1C`, `+0x20` u8 (0 local / comm), `+0x21` u8 own client id, `+0x22` u8 (variant 0..4), `+0x23` u8 comm flag, `+0x24` /
//! `+0x34` / `+0x48` 4 pointers each, `+0x44` u8[4], `+0x58`, `+0x5C`, `+0x60` / `+0x64`, `+0x68` / `+0x6C` / `+0x70`, `+0x74` /
//! `+0x76` / `+0x84` / `+0x86` u16, `+0x7F` / `+0x80` / `+0x90` / `+0x91` u8, `+0x88` / `+0x8C` result words, `+0x94` / `+0x98`
//! record buffer + size, `+0x9C..+0xB3`, `+0xB4..+0xC0` statistics, `+0xC1` flag bits, `+0xC2` u8[6], `+0xC8 + 6 k` u8[4][6],
//! `+0xE0 + 4 k` u32[4]. Meanings are UNVERIFIED.
//!
//! **Pokecon** (0xE8 bytes; init 021b93e0(pc, M, flag)): `+0x00` M, `+0x04 + 0x1C i` 4 party containers, `+0x74 + 4 i` the
//! Pokemon party of client i, `+0x84 + 4 j` 24 BattleMon pointers by mon id (first slot per client {0, 12, 6, 18} from the table
//! at 0x021EFCF0), `+0xE4` flag.
//!
//! **Party container** (0x1C bytes): `+0x00..+0x14` 6 BattleMon pointers, `+0x18` u8 count.
//!
//! **S, server object** (`btl_server.c`, 0xCC4 bytes; 021ba9c8(M, src18, r2, r3, heap u16 on the stack)): `+0x00` state
//! function (021babec(S, 0x021BAC05)), `+0x0C` M, `+0x10` r2, `+0x14 + 12 i` 4 client slots (021bb828), `+0x44` flow work
//! (021bb968(S, M, S[0x10], S + 0xF0, S[0x4C], heap)), `+0x4C` r3, `+0x50` 0x18 bytes copied from r1, `+0xAC` StrBuf
//! (020457b0(0x20, heap)), `+0xB0` 021bc550(), `+0xED` u8, `+0xF0` inline area (`+0xCB0` points at it), `+0xCBA` / `+0xCBF` u8,
//! `+0xCC0` u16 heap id. Rest: [`crate::server_flow::server_side`].

pub mod server1;
pub mod server2;
pub mod server3;

use std::rc::Rc;

/// B16: memory outside a [`Mem`]'s own windows / blocks: the battle integration's object table
/// (`crate::integration`), shared by every port so a pointer read by one area sees what another
/// area wrote. The differential tests never install one.
pub trait MemExt {
    /// Reads `out.len()` bytes at `addr` (panics outside every object).
    fn read(&self, addr: u32, out: &mut [u8]);
    /// Writes `data` at `addr` (panics outside every object).
    fn write(&self, addr: u32, data: &[u8]);
}

/// A view of the original address space for the `battle_server.s` ports (B11a): a list of windows `(base address, bytes)`.
///
/// The main module M and the objects it points at (BATTLE_SETUP, party containers, sub-proc slots, received blocks) are
/// reached through raw pointers stored in each other, so the ports keep pointers as `u32` addresses and read or write
/// through a `Mem`, which keeps every pointer value the original hands to its callees identical. An access outside every
/// window goes to the [`MemExt`] backend when one is installed (B16) and panics otherwise (the original would touch
/// whatever memory is there; UNVERIFIED for such addresses).
#[derive(Default)]
pub struct Mem<'a> {
    wins: Vec<(u32, &'a mut [u8])>,
    ext: Option<Rc<dyn MemExt>>,
}

impl<'a> Mem<'a> {
    /// An empty view.
    pub fn new() -> Self {
        Mem {
            wins: Vec::new(),
            ext: None,
        }
    }

    /// An empty view over the shared memory `ext` (B16).
    pub fn with_ext(ext: Rc<dyn MemExt>) -> Self {
        Mem {
            wins: Vec::new(),
            ext: Some(ext),
        }
    }

    /// Adds the window `bytes` at address `base`.
    pub fn add(&mut self, base: u32, bytes: &'a mut [u8]) {
        self.wins.push((base, bytes));
    }

    /// Window index and offset of `addr..addr + len`.
    fn find(&self, addr: u32, len: usize) -> Option<(usize, usize)> {
        for (i, (b, v)) in self.wins.iter().enumerate() {
            let o = addr.wrapping_sub(*b) as usize;
            if o.checked_add(len).is_some_and(|e| e <= v.len()) {
                return Some((i, o));
            }
        }
        None
    }

    /// Window index and offset of `addr..addr + len` (panics outside every window).
    fn at(&self, addr: u32, len: usize) -> (usize, usize) {
        match self.find(addr, len) {
            Some(r) => r,
            None => panic!("address {addr:#010x}+{len} is outside every memory window"),
        }
    }

    /// The `len` bytes at `addr` (windows only).
    pub fn bytes(&self, addr: u32, len: usize) -> &[u8] {
        let (w, o) = self.at(addr, len);
        &self.wins[w].1[o..o + len]
    }

    /// The `len` bytes at `addr`, writable (windows only).
    pub fn bytes_mut(&mut self, addr: u32, len: usize) -> &mut [u8] {
        let (w, o) = self.at(addr, len);
        &mut self.wins[w].1[o..o + len]
    }

    /// A copy of the `len` bytes at `addr` (a window, or the [`MemExt`] backend).
    pub fn read_vec(&self, addr: u32, len: usize) -> Vec<u8> {
        let mut v = vec![0; len];
        self.read_into(addr, &mut v);
        v
    }

    /// Reads `out.len()` bytes at `addr` (a window, or the [`MemExt`] backend).
    pub fn read_into(&self, addr: u32, out: &mut [u8]) {
        match (self.find(addr, out.len()), &self.ext) {
            (Some((w, o)), _) => out.copy_from_slice(&self.wins[w].1[o..o + out.len()]),
            (None, Some(e)) => e.read(addr, out),
            (None, None) => {
                self.at(addr, out.len());
            }
        }
    }

    /// Writes `data` at `addr` (a window, or the [`MemExt`] backend).
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        match self.find(addr, data.len()) {
            Some((w, o)) => self.wins[w].1[o..o + data.len()].copy_from_slice(data),
            None => match &self.ext {
                Some(e) => e.write(addr, data),
                None => {
                    self.at(addr, data.len());
                }
            },
        }
    }

    pub fn r8(&self, addr: u32) -> u8 {
        let mut b = [0; 1];
        self.read_into(addr, &mut b);
        b[0]
    }

    pub fn r16(&self, addr: u32) -> u16 {
        let mut b = [0; 2];
        self.read_into(addr, &mut b);
        u16::from_le_bytes(b)
    }

    pub fn r32(&self, addr: u32) -> u32 {
        let mut b = [0; 4];
        self.read_into(addr, &mut b);
        u32::from_le_bytes(b)
    }

    pub fn w8(&mut self, addr: u32, v: u8) {
        self.write_bytes(addr, &[v]);
    }

    pub fn w16(&mut self, addr: u32, v: u16) {
        self.write_bytes(addr, &v.to_le_bytes());
    }

    pub fn w32(&mut self, addr: u32, v: u32) {
        self.write_bytes(addr, &v.to_le_bytes());
    }
}
