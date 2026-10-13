//! F8 (fourth batch): overlay-21 `scrcmd_shop.c`, Thumb code 0x021B2FD8-0x021B4BEC (F1-SPLIT row 21.46 minus its tail, which is
//! `scrcmd_proc_fld.c`): the shop command 329, its wait callback and camera step, the item lists, the purchase check and the
//! payment. The shop UI (windows, list menu, sprites, the main loop 0x021B35A4) stays behind hooks (D3).
//!
//! Every logic function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_shop.spec` (header: command-id
//! table, the shop work layout, the render / UI functions left out). The functions listed in [`PORTED`] are ported here and
//! checked with `tools/check-fn.sh black-script scrcmd_shop`; the rest are the F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_3350, 0x021B_3568, 0x021B_49EC];

/// Offsets in the shop UI work W2 (= the shop work + 8).
pub const W2_MONEY: u32 = 0xC;
pub const W2_RECORDS: u32 = 0x14;
pub const W2_BP: u32 = 0x18;
/// `u8` flag: the shop sells for BP instead of money.
pub const W2_BP_SHOP: u32 = 0x163;
/// Record counters bumped by a purchase (`FUN_02008DAC` / `FUN_02008DF0`).
pub const REC_PURCHASES: u32 = 0x15;
pub const REC_MONEY_SPENT: u32 = 0x16;
pub const REC_BP_SPENT: u32 = 0x22;

/// Callees of the scrcmd_shop functions that this area does not port, named by address (meaning per the spec).
pub trait ShopHooks: MemEnv {
    /// C: `FUN_0200C274(money)` (main): the money (UNVERIFIED name).
    fn fun_0200c274(&mut self, money: u32) -> u32;
    /// C: `FUN_0200DABC(bp)` (main): the battle points (UNVERIFIED name).
    fn fun_0200dabc(&mut self, bp: u32) -> u32;
    /// C: `FUN_0200C29C(money, n)` (main): subtract money.
    fn fun_0200c29c(&mut self, money: u32, n: u32);
    /// C: `FUN_0200DA98(bp, n)` (main): subtract battle points (`n` u16).
    fn fun_0200da98(&mut self, bp: u32, n: u32);
    /// C: `FUN_02008DAC(records, id, n)` (main): add `n` to a record counter.
    fn fun_02008dac(&mut self, records: u32, id: u32, n: u32);
    /// C: `FUN_02008DF0(records, id)` (main): increment a record counter.
    fn fun_02008df0(&mut self, records: u32, id: u32);
}

/// C: `FUN_overlay_d_21__021b3350`: the currency of the shop: BP (`FUN_0200DABC(W2+0x18)`) when W2+0x163 is set, else the
/// money (`FUN_0200C274(W2+0xC)`).
pub fn currency<H: ShopHooks>(h: &mut H, w2: u32) -> u32 {
    if h.r8(w2 + W2_BP_SHOP) == 0 {
        let m = h.r32(w2 + W2_MONEY);
        h.fun_0200c274(m)
    } else {
        let b = h.r32(w2 + W2_BP);
        h.fun_0200dabc(b)
    }
}

/// C: `FUN_overlay_d_21__021b3568`: pay `price`: money (record 0x16 += price) or BP (`(u16)price`, record 0x22 += price), then
/// record 0x15 (purchases) += 1.
pub fn pay<H: ShopHooks>(h: &mut H, w2: u32, price: u32) {
    if h.r8(w2 + W2_BP_SHOP) == 0 {
        let m = h.r32(w2 + W2_MONEY);
        h.fun_0200c29c(m, price);
        let r = h.r32(w2 + W2_RECORDS);
        h.fun_02008dac(r, REC_MONEY_SPENT, price);
    } else {
        let b = h.r32(w2 + W2_BP);
        h.fun_0200da98(b, price & 0xFFFF);
        let r = h.r32(w2 + W2_RECORDS);
        h.fun_02008dac(r, REC_BP_SPENT, price);
    }
    let r = h.r32(w2 + W2_RECORDS);
    h.fun_02008df0(r, REC_PURCHASES);
}

/// C: `FUN_overlay_d_21__021b49ec`: step the purchase count `*n` (s16) by the key `key`: -1 (0 wraps to `max`), +1 (past
/// `max` wraps to 1), -10 (floor 1), +10 (ceiling `max`). Returns 2 (went down) / 1 (went up) when `*n` changed, 0 when it
/// did not or for any other key.
pub fn count_step<H: MemEnv>(h: &mut H, n: u32, max: u32, key: u32) -> u32 {
    let old = h.r16(n) as i16 as i32;
    let max = max as i32;
    let key = key as i32;
    let (new, up) = match key {
        -1 => {
            let v = (old - 1) as i16 as i32;
            (if v <= 0 { max } else { v }, false)
        }
        -10 => {
            let v = (old - 10) as i16 as i32;
            (if v <= 0 { 1 } else { v }, false)
        }
        1 => {
            let v = (old + 1) as i16 as i32;
            (if v > max { 1 } else { v }, true)
        }
        10 => {
            let v = (old + 10) as i16 as i32;
            (if v > max { max } else { v }, true)
        }
        _ => return 0,
    };
    h.w16(n, new as u16);
    let now = h.r16(n) as i16 as i32;
    if now == old {
        0
    } else if up {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests;
