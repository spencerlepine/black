//! `pokeparty.c` (S1): the party container: capacity, count, six 0xDC-byte Pokemon records and a
//! per-slot flag byte. Main ARM9 code, verified per function with
//! `tools/check-fn.sh black-storage party` (spec `tools/rom-diff/specs/black-storage/party.spec`).
//!
//! This file holds the smoke ports that prove the spec; S1 ports the rest of the area.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_A858,
    0x0201_A98C,
    0x0201_AA14,
    0x0201_AA18,
    0x0201_AA1C,
    0x0201_AA50,
    0x0201_AA88,
    0x0201_AAD4,
    0x0201_AB08,
    0x0201_AB18,
    0x0201_AB28,
    0x0201_AB34,
    0x0201_AB48,
    0x0201_AB8C,
    0x0201_AC2C,
    0x0201_AC44,
    0x0201_AC80,
    0x0201_ACC4,
    0x0201_ACDC,
    0x0201_AD18,
    0x0201_AD40,
    0x0201_AD58,
    0x0201_A920,
    0x0201_A948,
    0x0201_A954,
    0x0201_A9C8,
];

/// Size of the party container in bytes (`FUN_0201A954` fills this many).
pub const PARTY_SIZE: usize = 0x534;
/// Size of one Pokemon record.
pub const MON_SIZE: usize = 0xDC;
/// Offset of the per-slot flag byte.
pub const FLAGS_OFS: usize = 0x530;
/// C: the `"pokeparty.c"` allocation name, as the ROM passes its address.
pub const NAME_POKEPARTY: u32 = 0x020A_7304;

/// Undecompiled routines the party code calls.
pub trait PartyHooks {
    /// C: `FUN_02017E1C(mon, id, 0)`: read a field of the Pokemon record at `mon_addr`.
    fn mon_param(&mut self, mon_addr: u32, id: u32) -> u32;
    /// C: `FUN_02017528(mon)`: clear one 0xDC-byte Pokemon record (`mon` is that record).
    fn clear_mon(&mut self, mon: &mut [u8]);
    /// C: `FUN_02030734(a0, a1, a2, a3, a4)`: heap allocation, the five C arguments in call order
    /// (r0..r3, then the stack word). Returns the block address.
    fn alloc(&mut self, args: [u32; 5]) -> u32;
    /// C: `FUN_020307B0(ptr)`: heap free.
    fn free(&mut self, ptr: u32);
}

fn rd32(p: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(p[ofs..ofs + 4].try_into().unwrap())
}

fn wr32(p: &mut [u8], ofs: usize, v: u32) {
    p[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}

/// C: `FUN_0201AA14`: the party capacity (word at +0).
pub fn capacity(party: &[u8]) -> u32 {
    rd32(party, 0)
}

/// C: `FUN_0201AA18`: the number of Pokemon in the party (word at +4).
pub fn count(party: &[u8]) -> u32 {
    rd32(party, 4)
}

/// C: `FUN_0201AB28`: the offset of slot `idx` (`+ 8 + idx * 0xDC`, wrapping like the pointer).
pub fn mon_offset(idx: u32) -> u32 {
    8u32.wrapping_add(idx.wrapping_mul(MON_SIZE as u32))
}

/// `1 << (idx & 0xFF)` as the ARM `lsls r, r` computes it: 0 once the shift reaches 32.
fn flag_bit(idx: u32) -> u32 {
    let n = idx & 0xFF;
    if n >= 32 {
        0
    } else {
        1 << n
    }
}

/// C: `FUN_0201ACC4`: is the flag of slot `idx` set (byte at +0x530).
pub fn flag_get(party: &[u8], idx: u32) -> bool {
    party[FLAGS_OFS] as u32 & flag_bit(idx) != 0
}

/// C: `FUN_0201ACDC`: set (`on == 1`) or clear the flag of slot `idx`.
pub fn flag_set(party: &mut [u8], idx: u32, on: u32) {
    let bit = flag_bit(idx);
    let b = party[FLAGS_OFS] as u32;
    party[FLAGS_OFS] = if on == 1 {
        (b | bit) as u8
    } else {
        (0xFFu32.wrapping_sub(bit) & b) as u8
    };
}

/// C: `FUN_0201A98C`: append a copy of `mon` (0xDC bytes) if `count < capacity` (signed), set its
/// flag, bump the count; returns whether it was added.
pub fn add(party: &mut [u8], mon: &[u8]) -> bool {
    let n = count(party);
    if n as i32 >= capacity(party) as i32 {
        return false;
    }
    let o = 8 + n as usize * MON_SIZE;
    party[o..o + MON_SIZE].copy_from_slice(&mon[..MON_SIZE]);
    flag_set(party, n, 1);
    let n = count(party);
    wr32(party, 4, n.wrapping_add(1));
    true
}

/// C: `FUN_0201AA1C`: the number of slots whose field `0x4C` (is-egg) is not 1. `base` is the
/// address of the party in the machine the hooks model.
pub fn count_non_egg<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    let n = count(party) as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        if h.mon_param(base.wrapping_add(mon_offset(i)), 0x4C) != 1 {
            r += 1;
        }
    }
    r
}

/// C: `FUN_0201AD40`: field `0x4C` of the Pokemon at `mon_addr` is not 1.
pub fn not_egg<H: PartyHooks>(mon_addr: u32, h: &mut H) -> bool {
    h.mon_param(mon_addr, 0x4C) != 1
}

/// C: `FUN_0201A858(p)`: advance the 6-digit odometer `p[0..6]` to the next tuple of pairwise
/// distinct digits in 1..6 (`p[5]` is pre-incremented first). Returns false once `p[0]` runs past 6.
/// Byte arithmetic follows the original (wrapping `u8`).
pub fn next_digit_tuple(p: &mut [u8]) -> bool {
    fn distinct(p: &[u8]) -> bool {
        (0..6).all(|i| (i + 1..6).all(|j| p[i] != p[j]))
    }
    p[5] = p[5].wrapping_add(1);
    loop {
        if p[0] > 6 {
            return false;
        }
        while p[1] < 7 {
            while p[2] < 7 {
                while p[3] < 7 {
                    while p[4] < 7 {
                        while p[5] < 7 {
                            if distinct(p) {
                                return true;
                            }
                            p[5] = p[5].wrapping_add(1);
                        }
                        p[5] = 1;
                        p[4] = p[4].wrapping_add(1);
                    }
                    p[4] = 1;
                    p[3] = p[3].wrapping_add(1);
                }
                p[3] = 1;
                p[2] = p[2].wrapping_add(1);
            }
            p[2] = 1;
            p[1] = p[1].wrapping_add(1);
        }
        p[1] = 1;
        p[0] = p[0].wrapping_add(1);
    }
}

/// C: `FUN_0201A954(party, capacity)`: zero the container, set the capacity, count 0, flags 0,
/// and clear the six records through the hook.
pub fn init<H: PartyHooks>(party: &mut [u8], capacity: u32, h: &mut H) {
    party[..PARTY_SIZE].fill(0);
    wr32(party, 0, capacity);
    wr32(party, 4, 0);
    party[FLAGS_OFS] = 0;
    for i in 0..6 {
        let o = 8 + i * MON_SIZE;
        h.clear_mon(&mut party[o..o + MON_SIZE]);
    }
}

/// C: `FUN_0201A948(party)`: `FUN_0201A954(party, 6)` (tail call).
pub fn init_default<H: PartyHooks>(party: &mut [u8], h: &mut H) {
    init(party, 6, h);
}

/// C: `FUN_0201A920(heap)`: allocate a party block, initialise it with capacity 6, and return
/// its address. `party` is the block (its address is `base`); the result is the offset from `base`.
pub fn create<H: PartyHooks>(heap: u32, party: &mut [u8], base: u32, h: &mut H) -> u32 {
    let p = h.alloc([heap, PARTY_SIZE as u32, 0, NAME_POKEPARTY, 0x41]);
    init_default(party, h);
    p.wrapping_sub(base)
}

/// C: `FUN_0201A9C8(party, idx)`: remove slot `idx`: the records after it move down one slot up to
/// the last one, which is cleared; count--. The flags are left as they are.
pub fn remove<H: PartyHooks>(party: &mut [u8], idx: u32, h: &mut H) -> bool {
    let count = count(party);
    let last = count.wrapping_sub(1) as i32;
    let mut j = idx;
    if (idx as i32) < last {
        loop {
            let (src, dst) = (mon_offset(j + 1) as usize, mon_offset(j) as usize);
            party.copy_within(src..src + MON_SIZE, dst);
            j += 1;
            if (j as i32) >= last {
                break;
            }
        }
    }
    let o = mon_offset(j) as usize;
    h.clear_mon(&mut party[o..o + MON_SIZE]);
    wr32(party, 4, count.wrapping_sub(1));
    true
}

/// C: `FUN_0201AA50(party)`: number of slots for which `FUN_0201AD18` (battle-able) holds.
pub fn count_battle_able<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    let n = count(party) as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        if battle_able(base.wrapping_add(mon_offset(i)), h) {
            r += 1;
        }
    }
    r
}

/// C: `FUN_0201AA88(party)`: number of slots with field `0x4C` == 1 and field `3` == 0.
pub fn count_egg_and_field3_clear<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    let n = count(party) as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        let m = base.wrapping_add(mon_offset(i));
        let egg = h.mon_param(m, 0x4C);
        let f3 = h.mon_param(m, 3);
        if egg == 1 && f3 == 0 {
            r += 1;
        }
    }
    r
}

/// C: `FUN_0201AAD4(party)`: number of slots with field `3` == 1.
pub fn count_field3_set<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    let n = count(party) as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        if h.mon_param(base.wrapping_add(mon_offset(i)), 3) == 1 {
            r += 1;
        }
    }
    r
}

/// C: `FUN_0201AB08(party)`: index of the first battle-able slot (count if none).
pub fn find_battle_able<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    find_first(party, base, battle_able::<H>, h)
}

/// C: `FUN_0201AB18(party)`: index of the first slot with `FUN_0201AD40` (not an egg) true (count if none).
pub fn find_not_egg<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    find_first(party, base, not_egg::<H>, h)
}

/// C: `FUN_0201AD58(party, pred)`: index of the first slot where `pred(&mon)` holds (count if none).
pub fn find_first<H: PartyHooks>(
    party: &[u8],
    base: u32,
    pred: fn(u32, &mut H) -> bool,
    h: &mut H,
) -> u32 {
    let n = count(party) as i32;
    for i in 0..n.max(0) as u32 {
        if pred(base.wrapping_add(mon_offset(i)), h) {
            return i;
        }
    }
    n as u32
}

/// C: `FUN_0201AB34(party, idx)`: `FUN_0201AD18(&mon[idx])`.
pub fn battle_able_at<H: PartyHooks>(idx: u32, base: u32, h: &mut H) -> bool {
    battle_able(base.wrapping_add(mon_offset(idx)), h)
}

/// C: `FUN_0201AB48(party, idx, mon)`: `count += get(&mon[idx], 0xA9) - get(src, 0xA9)`, then
/// the record `mon[idx]` is replaced by `src`. `src_addr` is the address of `src` in the machine.
pub fn replace_from<H: PartyHooks>(
    party: &mut [u8],
    base: u32,
    idx: u32,
    src: &[u8],
    src_addr: u32,
    h: &mut H,
) {
    let o = mon_offset(idx) as usize;
    let a = h.mon_param(base.wrapping_add(mon_offset(idx)), 0xA9);
    let b = h.mon_param(src_addr, 0xA9);
    let d = a.wrapping_sub(b);
    party[o..o + MON_SIZE].copy_from_slice(&src[..MON_SIZE]);
    wr32(party, 4, count(party).wrapping_add(d));
}

/// C: `FUN_0201AB8C(party, a, b, heap)`: swap records `a` and `b` through a temporary block
/// (`tmp` is that block, allocated by the hook), free it, and swap the two slot flags. Returns 0.
pub fn swap_slots<H: PartyHooks>(
    party: &mut [u8],
    a: u32,
    b: u32,
    heap: u32,
    tmp: &mut [u8],
    h: &mut H,
) -> u32 {
    let t = h.alloc([heap, MON_SIZE as u32, 0, NAME_POKEPARTY, 382]);
    let (oa, ob) = (mon_offset(a) as usize, mon_offset(b) as usize);
    tmp[..MON_SIZE].copy_from_slice(&party[oa..oa + MON_SIZE]);
    party.copy_within(ob..ob + MON_SIZE, oa);
    party[ob..ob + MON_SIZE].copy_from_slice(&tmp[..MON_SIZE]);
    h.free(t);
    let (fa, fb) = (flag_get(party, a), flag_get(party, b));
    flag_set(party, a, fb as u32);
    flag_set(party, b, fa as u32);
    0
}

/// C: `FUN_0201AC2C(src, dst)`: copy the 0x534-byte party.
pub fn copy(src: &[u8], dst: &mut [u8]) {
    dst[..PARTY_SIZE].copy_from_slice(&src[..PARTY_SIZE]);
}

/// C: `FUN_0201AC44(party, v)`: whether some slot has field `5` == `v`.
pub fn any_field5<H: PartyHooks>(party: &[u8], base: u32, v: u32, h: &mut H) -> bool {
    let n = count(party) as i32;
    for i in 0..n.max(0) as u32 {
        if h.mon_param(base.wrapping_add(mon_offset(i)), 5) == v {
            return true;
        }
    }
    false
}

/// C: `FUN_0201AC80(party)`: number of slots with field `0xA0` != 0 and field `0x4C` == 0.
pub fn count_pending<H: PartyHooks>(party: &[u8], base: u32, h: &mut H) -> u32 {
    let n = count(party) as i32;
    let mut r = 0;
    for i in 0..n.max(0) as u32 {
        let m = base.wrapping_add(mon_offset(i));
        if h.mon_param(m, 0xA0) != 0 && h.mon_param(m, 0x4C) == 0 {
            r += 1;
        }
    }
    r
}

/// C: `FUN_0201AD18(mon)` (`sub_0201ad18`): field `0x4C` != 1 and field `0xA0` != 0 (both read).
pub fn battle_able<H: PartyHooks>(mon_addr: u32, h: &mut H) -> bool {
    let egg = h.mon_param(mon_addr, 0x4C);
    let a0 = h.mon_param(mon_addr, 0xA0);
    egg != 1 && a0 != 0
}

#[cfg(test)]
mod tests;
