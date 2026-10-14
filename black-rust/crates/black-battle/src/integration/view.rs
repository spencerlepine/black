//! Host model of the battle screen (D3 / D7: rewritten in Bevy, never ported): the battle view
//! (`battle_view_core`, 0x021E8xxx-0x021EAxxx), the view-side objects (0x021EF9xx), overlay 94
//! (sound / effects) and the overlay loader. Headless: graphics calls do nothing, every
//! "is it still running" query answers "done", and the player's choices come from the script in
//! [`super::Env`]. Meanings of the entry points are from the disassembly and the client ports'
//! docs (UNVERIFIED).

use super::{val, Env};
use crate::server_flow::env::{Arg, Lent};

pub fn call(env: &mut Env, _lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let _ = (&env, &args, val);
    Some(match addr {
        // 021e8dc4(heap): loads overlays 94 / 95 / 96 (`02034AC4`); 021e8df8 unloads them.
        0x021E_8DC4 | 0x021E_8DF8 => 0,
        // 021e88cc: loads overlay 140 on DSi hardware (global 0x021F6380); 021e88f8 unloads it.
        0x021E_88CC | 0x021E_88F8 => 0,
        // 021e8f20(M, client, pokecon, mode, heap): creates the battle view; a handle block here
        0x021E_8F20 => env.objs.borrow_mut().alloc(0x40, "battle view (headless)"),
        // 021e9034(view): frees it
        0x021E_9034 => {
            env.objs.borrow_mut().free(val(args, 0));
            0
        }
        // 021e908c(view): the view's per-frame update
        0x021E_908C => 0,
        // ---- requests that start a view effect / window (answer unused) ----
        0x021E_90A0 | 0x021E_979C | 0x021E_9760 | 0x021E_9808 | 0x021E_9858 | 0x021E_98AC
        | 0x021E_9920 | 0x021E_9954 | 0x021E_9964 | 0x021E_9B14 | 0x021E_9E70 | 0x021E_A058
        | 0x021E_A614 | 0x021E_A65C | 0x021E_A7FC | 0x021E_AA5C | 0x021E_AE24 | 0x021E_AE44
        | 0x021E_AEA8 | 0x021E_A824 => {
            env.ui_log.push(addr);
            0
        }
        // message building into the view's buffer (text only)
        0x021E_AF18 | 0x021E_AF50 | 0x021E_AF78 => 0,
        // ---- "is it finished" polls: always finished ----
        0x021E_90D0 | 0x021E_9974 | 0x021E_9B20 | 0x021E_9E80 | 0x021E_A710 | 0x021E_A720
        | 0x021E_A814 | 0x021E_AE34 | 0x021E_AE54 => 1,
        // ---- the player's choices (scripted) ----
        // 021e97a4: the action menu answer: 1 = Fight
        0x021E_97A4 => {
            env.ui_log.push(addr);
            1
        }
        // 021e97bc(view, mon, &record): opens the move menu for `mon`; the answer goes to the
        // action record. The script (`Env::script`, B16: slot 0) picks the move slot; the target is
        // the default one.
        0x021E_97BC => {
            let rec = val(args, 2);
            let mon = val(args, 1);
            env.ui_fight = Some((rec, mon));
            0
        }
        // 021e98b4(view): the move menu is answered (the record is written)
        0x021E_98B4 => {
            let (rec, mon) = env.ui_fight.take().expect("move menu was not opened");
            let k = env.choices.len().min(env.script.len() - 1);
            let slot = env.script[k];
            let mv = env.with_mon(mon, |_, b| b.move_id(slot)) as u32;
            let mut r = crate::calc_action::ActionRecord(0);
            r.set_fight(mv, 1);
            env.w32(rec, r.0);
            env.choices.push(mv);
            1
        }
        // 021e98e8: the target menu answer: 1 = confirmed
        0x021E_98E8 => 1,
        // 021eac54(view, &answer): the yes / no window has an answer (B19): 0, the first choice
        // ("yes", e.g. send out the next Pokemon)
        0x021E_AC54 => {
            env.ui_log.push(addr);
            super::Bufs::new(args).put(env, _lent, 1, &0u32.to_le_bytes());
            1
        }
        // 021e9a78(view, block, pos, flag, sel): opens the party list (B19). The scripted player
        // picks party slot `Env::switch_script[k]` (the last one repeats) for position `pos`, as
        // the list's "add" does (overlay 95 `0689CAEC(sel, pos, slot)`: when `sel[6] < sel[7]`,
        // `sel[sel[6]] = slot`, `sel[3 + sel[6]] = pos`, `sel[6] += 1`); 021e9b20 then answers
        // "finished" (the range default below)
        0x021E_9A78 => {
            let (pos, sel) = (val(args, 2), val(args, 4));
            let k = env.switches.len().min(env.switch_script.len() - 1);
            let slot = env.switch_script[k];
            env.switches.push(slot);
            env.ui_log.push(addr);
            let mut o = env.objs.borrow_mut();
            let (n, max) = (o.r8(sel + 6), o.r8(sel + 7));
            if n < max {
                o.w8(sel + n, slot);
                o.w8(sel + 3 + n, pos);
                o.w8(sel + 6, n + 1);
            }
            0
        }
        // sound / BGM (ov94 0x021F7xxx-0x021F8xxx): starts do nothing; "playing" queries say no
        0x021F_7998 | 0x021F_84E8 | 0x021F_8504 | 0x021F_87EC | 0x021F_88BC | 0x021F_8B10 => 0,
        0x021F_7EB4 | 0x021F_8774 | 0x021F_8678 => 0,
        0x021F_8524 => 1,
        // any other battle-view entry: an effect / window / message starts (logged), or a
        // "finished?" query (the client ports' docs: nonzero = done): headless, everything is
        // done at once
        a if (0x021E_8F20..=0x021E_AFFF).contains(&a) => {
            env.ui_log.push(addr);
            1
        }
        // any other sound / effect call (overlay 94): nothing plays; "still playing?" -> 0
        a if (0x021F_7000..=0x021F_9FFF).contains(&a) => 0,
        _ => return None,
    })
}
