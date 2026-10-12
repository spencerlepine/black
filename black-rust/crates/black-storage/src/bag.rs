//! `myitem_savedata.c` (S3): the bag's pocket arrays of `{item, count}` slots, plus the TM/HM item
//! tests it uses, and the `bag_param.c` constructor. Main ARM9 code, verified per function with
//! `tools/check-fn.sh black-storage bag` (spec `tools/rom-diff/specs/black-storage/bag.spec`).
//!
//! Pointers into the bag are byte offsets from the bag start (the test wrappers add the fixed
//! emulator address `0x02E1_0000` where the C writes a pointer to memory).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_7ECC,
    0x0200_7F7C,
    0x0200_7FD0,
    0x0200_8038,
    0x0200_8216,
    0x0200_823C,
    0x0202_1288,
    0x0202_130C,
    0x0200_7D00,
    0x0200_7D6C,
    0x0200_7D94,
    0x0200_7DF8,
    0x0200_7E3C,
    0x0200_7E50,
    0x0200_7EF8,
    0x0200_7F1C,
    0x0200_7F68,
    0x0200_7FA0,
    0x0200_7FB8,
    0x0200_7FF4,
    0x0200_8058,
    0x0200_80A0,
    0x0200_80FC,
    0x0200_816C,
    0x0200_81DC,
    0x0202_E474,
];

/// Bytes per slot: `u16 item` then `u16 count`.
pub const SLOT_SIZE: usize = 4;

/// C: the pocket table of `FUN_02007D00` / `FUN_020081DC`: (offset of the first slot, slot count)
/// for pocket ids 0..=4.
pub const POCKETS: [(usize, u32); 5] = [
    (0x000, 310),
    (0x7D8, 48),
    (0x624, 109),
    (0x898, 64),
    (0x4D8, 83),
];

/// C: the 16-bit heap id of the item lookups (`ldrh` of a stack word).
pub type Heap = u16;

/// Undecompiled routines the bag code calls.
pub trait BagHooks {
    /// C: `FUN_02020FE4(item, 5, heap)`: the pocket id of an item (from its record in the item NARC).
    fn pocket_of(&mut self, item: u32, heap: Heap) -> u32;
}

/// Undecompiled routines the `bag_param.c` constructor calls (the game-data accessors).
pub trait BagParamHooks {
    /// C: `FUN_02030734(heap, size, align, file, line)`: allocate; returns the block's address.
    fn alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020071CC(x, sel)`, reached as the thunk `FUN_0200873C(x)` with `sel = 0x1B`.
    fn field_71cc(&mut self, x: u32, sel: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`.
    fn gamedata_2ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`.
    fn gamedata_2944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012924(gamedata)`.
    fn gamedata_2924(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)`.
    fn gamedata_292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)`.
    fn gamedata_27a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012B28(x)`: `x` is the result of `FUN_020127A4`.
    fn gamedata_2b28(&mut self, x: u32) -> u32;
}

fn rd16(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes([b[ofs], b[ofs + 1]])
}

fn wr16(b: &mut [u8], ofs: usize, v: u16) {
    b[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
}

fn wr32(b: &mut [u8], ofs: usize, v: u32) {
    b[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}

/// C: `FUN_02007ECC`: the offset of the first of `n` slots holding `item`, if it has at least
/// `min` of them; `None` if the item is absent or short (unsigned compare).
pub fn find(slots: &[u8], n: u32, item: u32, min: u32) -> Option<usize> {
    for i in 0..n as usize {
        if item == rd16(slots, i * SLOT_SIZE) as u32 {
            return (rd16(slots, i * SLOT_SIZE + 2) as u32 >= min).then_some(i * SLOT_SIZE);
        }
    }
    None
}

/// C: `FUN_02007F7C`: the number of the first `n` slots with a nonzero item id.
pub fn count_used(slots: &[u8], n: u32) -> u32 {
    (0..n as usize)
        .filter(|&i| rd16(slots, i * SLOT_SIZE) != 0)
        .count() as u32
}

/// C: `FUN_02007FD0`: swap two slots.
pub fn swap(a: &mut [u8], b: &mut [u8]) {
    for i in 0..SLOT_SIZE {
        std::mem::swap(&mut a[i], &mut b[i]);
    }
}

/// Swap slots `i` and `j` (`i < j`) of one slot array (`FUN_02007FD0` on two of its slots).
fn swap_slots(slots: &mut [u8], i: usize, j: usize) {
    let (a, b) = slots.split_at_mut(j * SLOT_SIZE);
    swap(
        &mut a[i * SLOT_SIZE..i * SLOT_SIZE + SLOT_SIZE],
        &mut b[..SLOT_SIZE],
    );
}

/// C: `FUN_02008038`: zero the count of every one of `n` slots whose item id is 0.
pub fn clear_empty_counts(slots: &mut [u8], n: u32) {
    for i in 0..n as usize {
        if rd16(slots, i * SLOT_SIZE) == 0 {
            slots[i * SLOT_SIZE + 2..i * SLOT_SIZE + 4].fill(0);
        }
    }
}

/// C: `FUN_02008216`: `ofs + idx * 4` (an offset from the array base) if `idx < n`, else `None`.
pub fn slot_ofs(ofs: u32, idx: u32, n: u32) -> Option<u32> {
    (idx < n).then(|| ofs.wrapping_add(idx.wrapping_mul(4)))
}

/// C: `FUN_02021288`: is the item a TM/HM (`0x148..=0x1A9` or `0x26A..=0x26C`).
pub fn is_tm(item: u32) -> bool {
    (0x148..=0x1A9).contains(&item) || (0x26A..=0x26C).contains(&item)
}

/// C: `FUN_0202130C`: the TM/HM index byte of an item (0xFF if it is not one).
pub fn tm_index(item: u32) -> u32 {
    if !is_tm(item) {
        return 0xFF;
    }
    if item >= 0x26A {
        (item - 0x20E) & 0xFF
    } else if item >= 0x1A4 {
        item.wrapping_sub(0x145) & 0xFF
    } else {
        item.wrapping_sub(0x148) & 0xFF
    }
}

/// C: `FUN_0200823C`: the stack limit of an item: 1 for a TM/HM, else 999.
pub fn max_stack(item: u32) -> u32 {
    if is_tm(item) {
        1
    } else {
        999
    }
}

/// C: `FUN_02007D00`: the (offset of the first slot, slot count) of a pocket; `None` for an id > 4
/// (the C sets the base to 0 and leaves the count alone).
pub fn pocket(id: u32) -> Option<(usize, u32)> {
    POCKETS.get(id as usize).copied()
}

/// C: `FUN_02007D6C`: the pocket id of an item, and that pocket's (offset, count).
pub fn locate<H: BagHooks>(h: &mut H, item: u32, heap: Heap) -> (u32, Option<(usize, u32)>) {
    let p = h.pocket_of(item, heap);
    (p, pocket(p))
}

/// C: `FUN_02007D94`: the slot an item of `add` more can go into: the slot holding the item if
/// its count + `add` fits `max`, else (no such slot) the first empty slot. `None` if neither.
pub fn slot_for_add(slots: &[u8], n: u32, item: u32, add: u32, max: u16) -> Option<usize> {
    let mut empty = None;
    for i in 0..n as usize {
        let it = rd16(slots, i * SLOT_SIZE) as u32;
        let count = rd16(slots, i * SLOT_SIZE + 2) as u32;
        if item == it {
            let sum = add.wrapping_add(count) as i32;
            return ((max as i32) >= sum).then_some(i * SLOT_SIZE);
        }
        if empty.is_none() && it == 0 && count == 0 {
            empty = Some(i * SLOT_SIZE);
        }
    }
    empty
}

/// C: `FUN_02007DF8`: the bag offset of the slot an item of `add` more goes into (see
/// `slot_for_add`); pocket 2 (TMs) holds one of each, the others 999.
pub fn add_slot<H: BagHooks>(
    h: &mut H,
    bag: &[u8],
    item: u32,
    add: u32,
    heap: Heap,
) -> Option<usize> {
    // An id > 4 never comes from the item table (the C would search at address 0).
    let (p, loc) = locate(h, item, heap);
    let (base, n) = loc?;
    let max = if p == 2 { 1 } else { 999 };
    slot_for_add(&bag[base..], n, item, add, max).map(|o| base + o)
}

/// C: `FUN_02007E3C`: can `add` of the item be put in the bag.
pub fn can_add<H: BagHooks>(h: &mut H, bag: &[u8], item: u32, add: u32, heap: Heap) -> bool {
    add_slot(h, bag, item, add, heap).is_some()
}

/// C: `FUN_02007EF8`: the bag offset of the slot holding at least `min` of the item.
pub fn held_slot<H: BagHooks>(
    h: &mut H,
    bag: &[u8],
    item: u32,
    min: u32,
    heap: Heap,
) -> Option<usize> {
    let (_, loc) = locate(h, item, heap);
    let (base, n) = loc?;
    find(&bag[base..], n, item, min).map(|o| base + o)
}

/// C: `FUN_02007F68`: does the bag hold at least `min` of the item.
pub fn has_held<H: BagHooks>(h: &mut H, bag: &[u8], item: u32, min: u32, heap: Heap) -> bool {
    held_slot(h, bag, item, min, heap).is_some()
}

/// C: `FUN_02007FB8`: how many of the item the bag holds (0 if none).
pub fn item_count<H: BagHooks>(h: &mut H, bag: &[u8], item: u32, heap: Heap) -> u16 {
    held_slot(h, bag, item, 1, heap).map_or(0, |s| rd16(bag, s + 2))
}

/// C: `FUN_02007FA0`: the number of used slots of a pocket (0 for an invalid pocket id).
pub fn pocket_used(bag: &[u8], id: u32) -> u32 {
    pocket(id).map_or(0, |(base, n)| count_used(&bag[base..], n))
}

/// C: `FUN_02007FF4`: for `n >= 2`, shift the slots up by one from the last used one down to slot
/// 0, carrying the empty slot's content: `temp = slot[n-1]`, then for `i = n-2 ..= 0`: if `temp`
/// holds an item, `slot[i+1] = slot[i]`, else `temp = slot[i]`; finally `slot[0] = temp`.
pub fn shift_front(slots: &mut [u8], n: u32) {
    if n < 2 {
        return;
    }
    let mut temp = [0u8; SLOT_SIZE];
    for i in (0..n as usize).rev() {
        if rd16(&temp, 0) == 0 {
            temp.copy_from_slice(&slots[i * SLOT_SIZE..i * SLOT_SIZE + SLOT_SIZE]);
        } else {
            slots.copy_within(
                i * SLOT_SIZE..i * SLOT_SIZE + SLOT_SIZE,
                (i + 1) * SLOT_SIZE,
            );
        }
    }
    slots[..SLOT_SIZE].copy_from_slice(&temp);
}

/// C: `FUN_02008058`: clear the counts of empty slots, then move the empty-count slots behind
/// the used ones (selection with `swap_slots`). `n >= 1`.
pub fn compact(slots: &mut [u8], n: u32) {
    clear_empty_counts(slots, n);
    let n = n as usize;
    for i in 0..n.saturating_sub(1) {
        for j in i + 1..n {
            if rd16(slots, i * SLOT_SIZE + 2) == 0 {
                swap_slots(slots, i, j);
            }
        }
    }
}

/// C: `FUN_020080A0`: clear empty counts, then sort used slots by ascending item id (empty ones
/// last). `n >= 1`.
pub fn sort_by_id(slots: &mut [u8], n: u32) {
    clear_empty_counts(slots, n);
    let n = n as usize;
    for i in 0..n.saturating_sub(1) {
        for j in i + 1..n {
            let ci = rd16(slots, i * SLOT_SIZE + 2);
            let cj = rd16(slots, j * SLOT_SIZE + 2);
            if ci == 0 || (cj != 0 && rd16(slots, j * SLOT_SIZE) < rd16(slots, i * SLOT_SIZE)) {
                swap_slots(slots, i, j);
            }
        }
    }
}

/// C: `FUN_020080FC`: like `sort_by_id`, ordered by the TM/HM index (`tm_index`). `n >= 1`.
pub fn sort_by_tm(slots: &mut [u8], n: u32) {
    clear_empty_counts(slots, n);
    let n = n as usize;
    for i in 0..n.saturating_sub(1) {
        for j in i + 1..n {
            // An empty slot i always gives way; else a used slot j with a lower TM index does.
            let i_empty = rd16(slots, i * SLOT_SIZE + 2) == 0;
            let j_used = rd16(slots, j * SLOT_SIZE + 2) != 0;
            let lower_tm = tm_index(rd16(slots, j * SLOT_SIZE) as u32)
                < tm_index(rd16(slots, i * SLOT_SIZE) as u32);
            if i_empty || (j_used && lower_tm) {
                swap_slots(slots, i, j);
            }
        }
    }
}

/// C: `FUN_02007E50`: add `add` of an item: its slot (see `add_slot`) gets the item and the count
/// grows by `add` (16-bit). Then the pocket is re-sorted (3: by id, 2: by TM index), or, if the
/// slot was empty and the pocket is 0/1/4, its used slots move to the front. `false` if no slot.
pub fn add_item<H: BagHooks>(h: &mut H, bag: &mut [u8], item: u32, add: u32, heap: Heap) -> bool {
    let Some(slot) = add_slot(h, bag, item, add, heap) else {
        return false;
    };
    let old = rd16(bag, slot + 2);
    wr16(bag, slot, item as u16);
    wr16(bag, slot + 2, old.wrapping_add(add as u16));
    let (p, loc) = locate(h, item, heap);
    if let Some((base, n)) = loc {
        match p {
            3 => sort_by_id(&mut bag[base..], n),
            2 => sort_by_tm(&mut bag[base..], n),
            _ if old == 0 => {
                let used = count_used(&bag[base..], n);
                shift_front(&mut bag[base..], used);
            }
            _ => {}
        }
    }
    true
}

/// C: `FUN_02007F1C`: remove `sub` of an item: its slot, which must hold at least `sub` (the C
/// passes `sub` on as the `min` of `held_slot`), loses `sub` (16-bit), the item id is cleared at 0,
/// and the pocket is compacted. `false` if not held.
pub fn remove_item<H: BagHooks>(
    h: &mut H,
    bag: &mut [u8],
    item: u32,
    sub: u32,
    heap: Heap,
) -> bool {
    let Some(slot) = held_slot(h, bag, item, sub, heap) else {
        return false;
    };
    let count = rd16(bag, slot + 2).wrapping_sub(sub as u16);
    wr16(bag, slot + 2, count);
    if count == 0 {
        wr16(bag, slot, 0);
    }
    if let (_, Some((base, n))) = locate(h, item, heap) {
        compact(&mut bag[base..], n);
    }
    true
}

/// C: `FUN_0200816C`: `compact` every pocket, in the order 4, 0, 3, 1, 2.
pub fn compact_all(bag: &mut [u8]) {
    for id in [4, 0, 3, 1, 2] {
        if let Some((base, n)) = pocket(id) {
            compact(&mut bag[base..], n);
        }
    }
}

/// C: `FUN_020081DC`: the bag offset of slot `idx` of a pocket, `None` if `idx` is past its slot
/// count. An invalid pocket id uses the whole bag with `size` as its slot count.
pub fn pocket_slot(id: u32, idx: u32, size: u32) -> Option<usize> {
    let (base, n) = pocket(id).unwrap_or((0, size));
    (idx < n).then(|| base + idx as usize * SLOT_SIZE)
}

/// C: `FUN_0202E474`: the `bag_param` constructor. Allocates the 0x3C-byte block (`out`, at
/// `out_base`), stores `gamedata`, the accessor results, copies `src` (0x18 bytes) at +0x14 if
/// given, stores `v` at +0x2C and whether `FUN_02012B28` says 1 at +0x30. Returns the block's offset.
pub fn new_param<H: BagParamHooks>(
    h: &mut H,
    out: &mut [u8],
    out_base: u32,
    gamedata: u32,
    src: Option<&[u8]>,
    v: u32,
    heap: u32,
) -> u32 {
    // The original calls FUN_02012EBC before the allocation (its result is used after it).
    let x = h.gamedata_2ebc(gamedata);
    // 0x020A_77C4 is the file name literal "bag_param.c" of the original.
    let p = h.alloc(heap, 0x3C, 1, 0x020A_77C4, 0x37);
    let ofs = p.wrapping_sub(out_base) as usize;
    let block = &mut out[ofs..ofs + 0x3C];
    wr32(block, 0x00, gamedata);
    let a = h.field_71cc(x, 0x1B);
    wr32(block, 0x04, a);
    let b = h.gamedata_2944(gamedata);
    wr32(block, 0x08, b);
    let c = h.gamedata_2924(gamedata);
    wr32(block, 0x0C, c);
    let d = h.gamedata_292c(gamedata);
    wr32(block, 0x10, d);
    if let Some(src) = src {
        block[0x14..0x2C].copy_from_slice(&src[..0x18]);
    }
    wr32(block, 0x2C, v);
    let t = h.gamedata_27a4(gamedata);
    let ok = h.gamedata_2b28(t) == 1;
    wr32(block, 0x30, ok as u32);
    p.wrapping_sub(out_base)
}

#[cfg(test)]
mod tests;
