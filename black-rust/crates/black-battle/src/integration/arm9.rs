//! Host models of the ARM9 main routines the battle code calls (D3: system, heap, sound driver,
//! Pokemon data). Each model names its original and says what is modelled; meanings come from the
//! disassembly (main.sbin at 0x02004000) and are UNVERIFIED unless the routine is ported elsewhere.

use super::{val, Bufs, Env};
use crate::battle_mon::BppEnv;
use crate::server_flow::env::{Arg, Lent};

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    Some(match addr {
        // Heap_Create(parent, id, size): the integration has one heap.
        0x0203_064C => 1,
        // Heap_AllocDebug(heap, size, align, file, line): a zeroed block (the pre-allocated one when
        // a caller set `Env::pending_alloc` for this size).
        0x0203_0734 => {
            let size = val(args, 1);
            match env.pending_alloc {
                Some((p, n)) if n == size => {
                    env.pending_alloc = None;
                    p
                }
                _ => {
                    let line = val(args, 4);
                    let name = format!("heap block (line {line:#x}, {})", env.backtrace());
                    env.objs.borrow_mut().alloc(size, &name)
                }
            }
        }
        // Heap_Free(p).
        0x0203_07B0 => {
            let p = val(args, 0);
            if p != 0 {
                env.objs.borrow_mut().free(p);
            }
            0
        }
        // Heap_Destroy(id).
        0x0203_06F0 => 0,
        // GameProc_FreeWork(proc): frees the block at proc + 0x14.
        0x0203_15D4 => {
            let w = env.r32(val(args, 0) + 0x14);
            if w != 0 && env.objs.borrow().blocks.contains_key(&w) {
                env.objs.borrow_mut().free(w);
            }
            env.w32(val(args, 0) + 0x14, 0);
            0
        }
        // GameProc_AllocWork(proc, size, heap): heap block stored at proc + 0x14.
        0x0203_159C => {
            let (proc, size) = (val(args, 0), val(args, 1));
            let w = env.objs.borrow_mut().alloc(size, "proc work (M)");
            env.w32(proc + 0x14, w);
            env.main_module = w;
            w
        }
        // ---- NitroSDK MI memory routines ----
        // MIi_CpuClear32(data, dest, size) / MIi_CpuClearFast: word fill. proc_init clears the
        // block it just allocated (always zeroed here) with a pass-through destination: no-op.
        0x0208_2A7C | 0x0208_2AC0 => {
            if let Some(Arg::V(dst) | Arg::PAt(dst)) = args.get(1) {
                let (d, n) = (*dst, val(args, 2));
                let w = val(args, 0).to_le_bytes();
                let fill: Vec<u8> = (0..n as usize).map(|i| w[i & 3]).collect();
                env.objs.borrow_mut().write(d, &fill);
            }
            0
        }
        // MIi_CpuCopy32(src, dest, size) / MIi_CpuCopyFast / MI_CpuCopy8(src, dest, size)
        0x0208_2A90 | 0x0208_2B0C | 0x0208_2D44 => {
            let (src, dst, n) = (val(args, 0), val(args, 1), val(args, 2));
            let mut b = vec![0u8; n as usize];
            env.objs.borrow_mut().read(src, &mut b);
            env.objs.borrow_mut().write(dst, &b);
            0
        }
        // 0207C7B4(x) (FX_Sqrt): the hardware 64-bit square root of `x << 32`, `(r + 0x200) >> 10`;
        // 0 for x <= 0
        0x0207_C7B4 => {
            let x = val(args, 0);
            if (x as i32) <= 0 {
                0
            } else {
                let r = ((x as u64) << 32).isqrt() as u32;
                r.wrapping_add(0x200) >> 10
            }
        }
        // 02011168 / 0201119C / 020111BC(x): the Entralink Pass Power multipliers (EXP / prize
        // money / ...; state 0x02146208, '0' = inactive): no Pass Power is active -> x
        0x0201_1168 | 0x0201_119C | 0x0201_11BC => val(args, 0),
        // 020362DC(): the keys pressed this frame; 02035C60(): touch. The scripted player
        // presses A whenever the battle waits for a key.
        0x0203_62DC => 1,
        0x0203_5C60 => 0,
        // 02034AC4(id) / 02034A5C(id): overlay load / unload (the host has every routine)
        0x0203_4AC4 | 0x0203_4A5C => 1,
        // 0203F0A8(): the system Mersenne Twister's next output (seeded with `Env::seed` when
        // the battle setup is built; the game seeds it at boot)
        0x0203_F0A8 => env.mt.next_u32(),
        // 0203F258(ctx): seeds a MathRandContext32 from the RTC (SHA-1 of the time); the
        // integration uses a fixed seed (`Env::seed`). The only caller passes M + 0x3F0.
        0x0203_F258 => {
            let p = match args[0] {
                Arg::P => env.main_module + 0x3F0,
                _ => val(args, 0),
            };
            let ctx = black_core::rng::MathRandContext32 {
                x: env.seed,
                mul: 0x5D58_8B65_6C07_8965,
                add: 0x0026_9EC3,
            };
            env.objs.borrow_mut().write(p, &ctx.to_bytes());
            0
        }
        // MI_CpuFill8(dest, data, size)
        0x0208_2BCC => {
            let (dst, v, n) = (val(args, 0), val(args, 1), val(args, 2));
            env.objs.borrow_mut().write(dst, &vec![v as u8; n as usize]);
            0
        }
        // main-module sound layer: fades / BGM; 02005E50 "fade running" -> no
        0x0200_5CA4 | 0x0200_5D28 | 0x0200_5D90 | 0x0200_5DF8 | 0x0200_5E30 | 0x0200_6734
        | 0x0200_6740 => 0,
        0x0200_5E50 => 0,
        // 020057C4: is the VBlank callback the sound one (0x02005795) -> 1; the battle then skips
        // 020057AC (installing it).
        0x0200_57C4 => 1,
        0x0200_57AC => 0,
        // Game config (u16 bitfield behind setup + 0x5C): 02008374 = bits 0-3 (message speed),
        // 020083AC = bit 7, 020083C8 = bit 6.
        0x0200_8374 => env.objs.borrow_mut().r16(val(args, 0)) & 0xF,
        0x0200_83AC => (env.objs.borrow_mut().r16(val(args, 0)) >> 7) & 1,
        0x0200_83C8 => (env.objs.borrow_mut().r16(val(args, 0)) >> 6) & 1,
        // ---- poke_tool.c (POKEMON_PARAM accessors, the verified black-pokemon ports) ----
        0x0201_7E1C => env.unp().pp_get(val(args, 0), val(args, 1)),
        0x0201_7E40 => {
            env.unp().pp_put(val(args, 0), val(args, 1), val(args, 2));
            0
        }
        // 02018480(pp, mystatus): is `pp`'s original trainer the player: OT id (param 7) ==
        // MyStatus +0x10, OT gender (0x9A) == +0x1D and OT name (0x8E) == the MyStatus name
        // (`0201F448`). Strings are UI-only here: the name compare is not modelled (equal).
        0x0201_8480 => {
            let (pp, ms) = (val(args, 0), val(args, 1));
            let id = env.unp().pp_get(pp, 7);
            let g = env.unp().pp_get(pp, 0x9A);
            let mut o = env.objs.borrow_mut();
            (id == o.r32(ms + 0x10) && g == o.r8(ms + 0x1D)) as u32
        }
        // ---- strbuf.c (StrBuf: u16 max, u16 len, u32 magic 0xB6F8D2EC, u16 text[max + 1]); the
        // text is UI-only, so the copy / format routines below keep the layout but not the text ----
        0x0204_57B0 => {
            let (n, _heap) = (val(args, 0), val(args, 1));
            let p = env.objs.borrow_mut().alloc(2 * n + 0xA, "StrBuf");
            let mut o = env.objs.borrow_mut();
            o.w16(p, n);
            o.w16(p + 2, 0);
            o.w32(p + 4, 0xB6F8_D2EC);
            o.w16(p + 8, 0xFFFF);
            p
        }
        0x0204_5808 => {
            env.objs.borrow_mut().free(val(args, 0));
            0
        }
        0x0204_581C => {
            let p = val(args, 0);
            let mut o = env.objs.borrow_mut();
            o.w16(p + 2, 0);
            o.w16(p + 8, 0xFFFF);
            0
        }
        // ---- archives ----
        // 0201B70C(heap): opens ARC 0x13 (020490F4(0x13, heap)) for reading; a handle the
        // integration recognises (reads through it are answered from the ROM file system)
        0x0201_B70C => arc_open(env, 0x13),
        // 020490F4(arc, heap): opens archive `arc`; 02049250(handle, member, heap): loads a member
        // into a new heap block; 02049238(handle): closes
        0x0204_90F4 => arc_open(env, val(args, 0)),
        0x0204_9250 => {
            let (h, k) = (val(args, 0), val(args, 1));
            let arc = *env.arc_handles.get(&h).expect("unknown archive handle");
            let data = arc_member(env, arc, k);
            let p = env
                .objs
                .borrow_mut()
                .alloc(data.len() as u32, &format!("ARC {arc} member {k}"));
            env.objs.borrow_mut().write(p, &data);
            p
        }
        0x0204_9238 => 0,
        // 02045924(sb, text): set from a 0xFFFF-terminated u16 string (at most max - 1 chars)
        0x0204_5924 => {
            let (sb, src) = (val(args, 0), val(args, 1));
            let mut t = Vec::new();
            loop {
                let c = env.objs.borrow_mut().r16(src + 2 * t.len() as u32);
                if c == 0xFFFF {
                    break;
                }
                t.push(c as u16);
            }
            strbuf_set(env, sb, &t);
            0
        }
        // 02045A00(sb, text, n): set from n u16 (n <= max); the text up to its terminator
        0x0204_5A00 => {
            let (sb, src, n) = (val(args, 0), val(args, 1), val(args, 2));
            let mut t = Vec::new();
            for i in 0..n {
                let c = env.objs.borrow_mut().r16(src + 2 * i);
                if c == 0xFFFF {
                    break;
                }
                t.push(c as u16);
            }
            strbuf_set(env, sb, &t);
            0
        }
        // 02045838(dst, src): copy; 02045874(src, heap): duplicate
        0x0204_5838 => {
            let t = strbuf_get(env, val(args, 1));
            strbuf_set(env, val(args, 0), &t);
            0
        }
        0x0204_5874 => {
            let src = val(args, 0);
            let t = strbuf_get(env, src);
            let max = (t.len() + 1) as u32;
            let p = env.objs.borrow_mut().alloc(2 * max + 0xA, "StrBuf");
            {
                let mut o = env.objs.borrow_mut();
                o.w16(p, max);
                o.w32(p + 4, 0xB6F8_D2EC);
            }
            strbuf_set(env, p, &t);
            p
        }
        // 02045A38(sb, dst, n): text out into a u16 array (terminated)
        0x0204_5A38 => {
            let (sb, dst, n) = (val(args, 0), val(args, 1), val(args, 2));
            let t = strbuf_get(env, sb);
            let mut o = env.objs.borrow_mut();
            for i in 0..n {
                let c = t.get(i as usize).copied().unwrap_or(0xFFFF);
                o.w16(dst + 2 * i, c as u32);
                if c == 0xFFFF {
                    break;
                }
            }
            0
        }
        // ---- waza_tool.c move data (records read from ARC 21; the cache is only a flag here) ----
        0x0201_C408 => env.waza_cache as u32,
        0x0201_C328 => {
            env.waza_cache = true;
            0
        }
        0x0201_C3C0 => {
            env.waza_cache = false;
            0
        }
        0x0201_BD28 => env.unp().move_param(val(args, 0), val(args, 1)),
        // 0201C1F4(move): the move's byte +4 is 0x65; 0201C284(move): byte +0xE is 6 (raw
        // waza-record tests through the record cache)
        0x0201_C1F4 => (env.arc.move_record(val(args, 0))[4] == 0x65) as u32,
        0x0201_C284 => (env.arc.move_record(val(args, 0))[0xE] == 6) as u32,
        // 0201C020(move): number of stat changes; 0201C0BC(move, i, &amount): stat i, amount
        0x0201_C020 => {
            use crate::server_flow::host::Unported;
            env.unp().move_fun_0201c020(val(args, 0))
        }
        0x0201_C0BC => {
            use crate::server_flow::host::Unported;
            let (mv, i) = (val(args, 0), val(args, 1));
            let amount = env.unp().move_fun_0201c0bc(mv, i);
            Bufs::new(args).put(env, lent, 2, &amount.to_le_bytes());
            env.arc.move_record(mv)[0x15 + i as usize] as u32
        }
        0x0201_BEB8 => env.unp().move_flag(val(args, 0), val(args, 1)) as u32,
        0x0201_C1BC => env.unp().move_param(val(args, 0), 1),
        0x0201_C1C8 => env.unp().move_param(val(args, 0), 0) & 0xFF,
        0x0201_C1D8 => env.unp().move_param(val(args, 0), 3) & 0xFFFF,
        0x0201_C1E8 => env.unp().move_param(val(args, 0), 2),
        0x0201_C314 => (env.unp().move_param(val(args, 0), 3) != 0) as u32,
        0x0201_C158 => env.arc.max_pp(val(args, 0), val(args, 1)),
        // personal data FUN_02019C90(species, form, param)
        0x0201_9C90 => env
            .arc
            .personal_param(val(args, 0), val(args, 1), val(args, 2)),
        // record.c game-record counters (setup + 0x6C; `02008DAC(rec, id, n)` add, `02008DF0(rec,
        // id)` +1): statistics of the save file, outside the battle's logic; not modelled
        0x0200_8DAC | 0x0200_8DF0 => 0,
        // 0200CFA0(dex, mon): registers `mon` as seen in the Pokedex (save data, outside the
        // battle's logic); not modelled
        0x0200_CFA0 => 0,
        // ---- game data / save blocks ----
        // 02012EBC(game data): its first word, the save data
        0x0201_2EBC => env.r32(val(args, 0)),
        // 02007B44(save) = save block 0x24 (the Chatot voice, `perapvoice.c`, 0x3EC bytes)
        0x0200_7B44 => save_block(env, val(args, 0), 0x24, 0x3EC),
        // 02007B20(heap): a new cleared voice block
        0x0200_7B20 => env.objs.borrow_mut().alloc(0x3EC, "perap voice"),
        // 02007C20(dst, src): copies a voice block
        0x0200_7C20 => {
            let mut b = vec![0u8; 0x3EC];
            env.objs.borrow_mut().read(val(args, 1), &mut b);
            env.objs.borrow_mut().write(val(args, 0), &b);
            0
        }
        // 02007B50(voice): recorded flag (word 0); 02007C70: 0 unless recorded
        0x0200_7B50 => env.r32(val(args, 0)),
        0x0200_7C70 => {
            if env.r32(val(args, 0)) == 1 {
                panic!("B16 gap: 02007C70 on a recorded Chatot voice");
            }
            0
        }
        // ---- pokeparty.c (PokeParty: u32 max, u32 count, 6 x POKEMON_PARAM at +8, u8 flags at
        // +0x530; 0x534 bytes) ----
        0x0201_A918 => PARTY_SIZE,
        // 0201AC2C(src, dst): copies a whole PokeParty
        0x0201_AC2C => {
            let mut b = vec![0u8; PARTY_SIZE as usize];
            env.objs.borrow_mut().read(val(args, 0), &mut b);
            env.objs.borrow_mut().write(val(args, 1), &b);
            0
        }
        0x0201_A920 => {
            let p = env.objs.borrow_mut().alloc(PARTY_SIZE, "PokeParty");
            party_init(env, p, 6);
            p
        }
        0x0201_A948 => {
            party_init(env, val(args, 0), 6);
            0
        }
        0x0201_A954 => {
            party_init(env, val(args, 0), val(args, 1));
            0
        }
        0x0201_A98C => party_add(env, val(args, 0), val(args, 1)),
        0x0201_AA14 => env.r32(val(args, 0)),
        0x0201_AA18 => env.r32(val(args, 0) + 4),
        // number of members that are not eggs (get 0x4C != 1)
        0x0201_AA1C => {
            let p = val(args, 0);
            let n = env.r32(p + 4);
            let mut unp = env.unp();
            (0..n)
                .filter(|&i| unp.pp_get(party_member(p, i), 0x4C) != 1)
                .count() as u32
        }
        // number of members alive (0201AD18)
        0x0201_AA50 => {
            let p = val(args, 0);
            let n = env.r32(p + 4);
            let mut unp = env.unp();
            (0..n)
                .filter(|&i| mon_alive(&mut unp, party_member(p, i)))
                .count() as u32
        }
        0x0201_AB28 => party_member(val(args, 0), val(args, 1)),
        0x0201_AB34 => {
            let m = party_member(val(args, 0), val(args, 1));
            mon_alive(&mut env.unp(), m) as u32
        }
        0x0201_AD18 => mon_alive(&mut env.unp(), val(args, 0)) as u32,
        // not an egg
        0x0201_AD40 => u32::from(env.unp().pp_get(val(args, 0), 0x4C) != 1),
        0x0201_ACDC => {
            let (p, i, on) = (val(args, 0), val(args, 1), val(args, 2));
            let mut o = env.objs.borrow_mut();
            let f = o.r8(p + 0x530);
            o.w8(
                p + 0x530,
                if on == 1 {
                    f | (1 << i)
                } else {
                    f & !(1 << i) & 0xFF
                },
            );
            0
        }
        a if THUMB_ARM9.contains(&a) || THUMB_ARM9_RANGES.iter().any(|r| r.contains(&a)) => {
            let vmc = (0x0201_11EC..0x0201_13D8).contains(&a);
            let cur = env.ai_vm.last().copied();
            let v: Vec<u32> = args
                .iter()
                .enumerate()
                .map(|(i, x)| match x {
                    Arg::V(v) | Arg::PAt(v) => *v,
                    // the AI ports' pass-through VM handle (see `ai::call`)
                    Arg::P if vmc && i == 0 && cur.is_some() => cur.unwrap(),
                    _ => panic!(
                        "B16 gap: ARM9 {a:#x} gets a pointer the caller does not know ({})",
                        env.backtrace()
                    ),
                })
                .collect();
            let r = env.run_thumb(a, &v);
            if a == 0x0201_121C {
                env.vms.insert(r);
            }
            r
        }
        _ => return None,
    })
}

/// ARM9 Thumb routines that run as original code in the interpreter ([`super::thumb`]):
/// small accessors (MyStatus, the 8-byte name records of `0202428C`) with no port.
pub const THUMB_ARM9: &[u32] = &[
    0x0200_846C, // MyStatus alloc + init (mystatus.c)
    0x0200_84A0, // MyStatus init
    0x0200_8494, // MyStatus copy (-> MI_CpuCopy8 0x20)
    0x0200_8500, // MyStatus name -> StrBuf
    0x0200_8510, // MyStatus name StrBuf create
    0x0200_8550, // MyStatus byte +0x1D
    0x0200_8560, // MyStatus byte +0x1B set
    0x0202_0F14, // item data archive open (-> 020490F4(0x18, heap))
    0x0202_0F20, // item data member load
    0x0202_428C, // 8-byte record clear
    0x0202_4570, // 8-byte record copy
];

/// The text of a StrBuf.
pub fn strbuf_get(env: &Env, sb: u32) -> Vec<u16> {
    let mut o = env.objs.borrow_mut();
    let n = o.r16(sb + 2);
    (0..n).map(|i| o.r16(sb + 8 + 2 * i) as u16).collect()
}

/// Sets the text of a StrBuf (truncated to max - 1, terminated).
pub fn strbuf_set(env: &Env, sb: u32, t: &[u16]) {
    let mut o = env.objs.borrow_mut();
    let max = o.r16(sb) as usize;
    let n = t.len().min(max.saturating_sub(1));
    for (i, &c) in t[..n].iter().enumerate() {
        o.w16(sb + 8 + 2 * i as u32, c as u32);
    }
    o.w16(sb + 8 + 2 * n as u32, 0xFFFF);
    o.w16(sb + 2, n as u32);
}

/// Whole Thumb files of ARM9 main run as original code: `btl_setup.c` (BATTLE_SETUP accessors,
/// 0x02012CE0-0x02013980; B11 notes it as unported).
pub const THUMB_ARM9_RANGES: &[std::ops::Range<u32>] = &[
    0x0201_2CE0..0x0201_3980,
    // vm.c (ported in black-script, whose `Mem` the integration does not provide yet)
    0x0201_11EC..0x0201_13D8,
];

/// `020071CC(save, id)`: save block `id` (allocated on first use).
pub fn save_block(env: &mut Env, save: u32, id: u32, size: u32) -> u32 {
    let key = (save, id);
    if let Some(&b) = env.save_blocks.get(&key) {
        return b;
    }
    let b = env
        .objs
        .borrow_mut()
        .alloc(size, &format!("save block {id:#x}"));
    env.save_blocks.insert(key, b);
    b
}

/// Opens archive `arc` (a handle block remembering its id).
pub fn arc_open(env: &mut Env, arc: u32) -> u32 {
    let h = env
        .objs
        .borrow_mut()
        .alloc(0x50, &format!("ARC {arc} handle"));
    env.arc_handles.insert(h, arc);
    h
}

/// Member `k` of ROM archive `arc` (`a/x/y/z` for the decimal digits of `arc`).
pub fn arc_member(env: &Env, arc: u32, k: u32) -> Vec<u8> {
    let path = env
        .rom
        .files
        .join(format!("a/{}/{}/{}", arc / 100, arc / 10 % 10, arc % 10));
    let b = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    nds_rom::narc::Narc::parse(&b)
        .unwrap()
        .file(k as usize)
        .to_vec()
}

/// Size of a PokeParty.
pub const PARTY_SIZE: u32 = 0x534;

/// Address of member `i` (0201AB28).
pub fn party_member(p: u32, i: u32) -> u32 {
    p + 8 + 0xDC * i
}

/// 0201A954(party, max): zeroed, max, count 0, flags 0, every member an encrypted empty record
/// (02017528).
pub fn party_init(env: &mut Env, p: u32, max: u32) {
    let mut b = vec![0u8; PARTY_SIZE as usize];
    b[..4].copy_from_slice(&max.to_le_bytes());
    for i in 0..6 {
        let o = 8 + 0xDC * i;
        black_pokemon::create::clear_pokemon(&mut b[o..o + 0xDC]);
    }
    env.objs.borrow_mut().write(p, &b);
}

/// 0201A98C(party, mon): appends a copy of `mon` (0 when full), sets flag bit `count`.
pub fn party_add(env: &mut Env, p: u32, mon: u32) -> u32 {
    let mut o = env.objs.borrow_mut();
    let (max, n) = (o.r32(p), o.r32(p + 4));
    if n as i32 >= max as i32 {
        return 0;
    }
    let mut m = vec![0u8; 0xDC];
    o.read(mon, &mut m);
    o.write(party_member(p, n), &m);
    let f = o.r8(p + 0x530);
    o.w8(p + 0x530, f | (1 << n));
    o.w32(p + 4, n + 1);
    1
}

/// 0201AD18(mon): not an egg (get 0x4C) and HP (get 0xA0) nonzero.
pub fn mon_alive(unp: &mut super::data::Unp, mon: u32) -> bool {
    unp.pp_get(mon, 0x4C) != 1 && unp.pp_get(mon, 0xA0) != 0
}
