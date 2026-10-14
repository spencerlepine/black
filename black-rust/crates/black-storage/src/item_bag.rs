//! Item helpers of the bag overlay 70 (S4b; `0x021B619C`, `0x021BA1D0`). Logic only (D3): the bag
//! screen's menus and messages are not here.

use crate::bag::is_tm;
use crate::item::{item_param, tm_move, ItemHooks};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_619C, 0x021B_A1D0];

/// The moves of the field-usable HMs: Cut (15), Fly (19), Surf (57), Strength (70), Waterfall (127),
/// Dive (291).
pub const FIELD_HM_MOVES: [u32; 6] = [15, 19, 57, 70, 127, 291];

/// C: `FUN_overlay_d_70__021b619c(item, count, heap)`: the sell price of `count` of an item: `count *
/// (price / 2)`, the price being item parameter 0 (`u16 * 10`), the halving signed.
pub fn sell_price<H: ItemHooks>(h: &mut H, item: u32, count: i32, heap: u16) -> u32 {
    let price = item_param(h, item & 0xFFFF, 0, heap) as i32;
    count.wrapping_mul(price / 2) as u32
}

/// C: `FUN_overlay_d_70__021ba1d0(item)`: whether the item is a TM/HM item whose move is one of the
/// field HM moves ([`FIELD_HM_MOVES`]); `tm_moves` is the TM/HM move table (0x0209EAA0).
pub fn is_field_hm_item(item: u32, tm_moves: &[u16]) -> bool {
    if !is_tm(item) {
        return false;
    }
    FIELD_HM_MOVES.contains(&tm_move(item, tm_moves))
}

#[cfg(test)]
mod tests;
