//! Address -> port / host model.

use super::Env;
use crate::server_flow::env::{Arg, Lent};

/// Calls `addr` (bit 0 cleared); `None` when nothing answers it.
pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let addr = addr & !1;
    // unported overlay-93 glue that runs as original Thumb code (see `OV93_THUMB`)
    if OV93_THUMB.contains(&addr) {
        // the turn-end walks get the flow work as a pass-through `P` (from the disassembly)
        let p_work = matches!(addr, 0x021C_4B8C | 0x021C_4C24);
        let wa = env.waddr;
        let v: Vec<u32> = args
            .iter()
            .enumerate()
            .map(|(i, a)| match a {
                Arg::V(v) | Arg::PAt(v) => *v,
                Arg::P if p_work && i == 0 => wa,
                _ => panic!("B16 gap: overlay-93 {addr:#x} (interpreted) gets a pointer the caller does not know"),
            })
            .collect();
        return Some(env.run_thumb(addr, &v));
    }
    // `bx pc` veneers of overlay 93 into overlay 95
    if (0x021B_6100..0x021F_3B20).contains(&addr) {
        let t = env.resolve_veneer(addr);
        if t != addr {
            return Some(super::ov95::call(env, addr, t, args));
        }
    }
    if (0x0689_8020..0x0689_E7E0).contains(&addr) {
        return Some(super::ov95::call(env, addr, addr, args));
    }
    match addr {
        0x0200_0000..=0x020F_FFFF => super::arm9::call(env, lent, addr, args),
        0x0218_7F00..=0x0218_A573 => super::ai::call(env, lent, addr, args),
        // overlay 231 `021809D8()`: the cartridge's infrared-chip probe (anti-piracy; command
        // 0x42 must answer 0xAA): genuine hardware -> 1
        0x0218_09D8 => Some(1),
        0x021E_F71C..=0x021E_FCFF => super::field::call(env, lent, addr, args),
        0x021E_EA30..=0x021E_EC77 => super::adapt::call(env, lent, addr, args),
        0x021E_E8B4..=0x021E_EA2F => super::recb::call(env, lent, addr, args),
        0x021B_6100..=0x021B_AA93 => super::server::call(env, lent, addr, args),
        0x021B_AA94..=0x021B_B967 => super::side::call(env, lent, addr, args),
        0x021B_B968..=0x021C_D6EB => {
            if addr == 0x021B_B968 {
                return Some(flow_create(env, lent, args));
            }
            super::flowcase::call(env, lent, addr, args)
        }
        0x021D_74C8..=0x021D_85AF => super::calc::call(env, lent, addr, args),
        0x021C_D6EC..=0x021D_3F03 => super::client::call(env, lent, addr, args),
        0x021D_3F04..=0x021D_4D5B => super::net::call(env, lent, addr, args),
        0x021D_4D5C..=0x021D_6EAB => super::bpp::call(env, lent, addr, args),
        0x021D_6EAC..=0x021D_74C7 => super::efactor::call(env, lent, addr, args),
        0x021E_8918..=0x021E_8C3F => super::sick::call(env, lent, addr, args),
        0x021E_8CE0..=0x021E_8DBF => super::sick::type_call(env, lent, addr, args),
        0x021E_8000..=0x021E_EA2F | 0x021E_F71C..=0x021E_FFFF | 0x021F_7000..=0x021F_9FFF => {
            super::view::call(env, lent, addr, args)
        }
        _ => None,
    }
}

/// Whether `addr` is in a ported area's `PORTED` list.
pub fn is_ported(addr: u32) -> bool {
    use crate::server_flow as sf;
    [
        sf::cmd_queue::PORTED,
        sf::core1::PORTED,
        sf::core2::PORTED,
        sf::core3::PORTED,
        sf::core4::PORTED,
        sf::core5::PORTED,
        sf::core6::PORTED,
        sf::api::PORTED,
        sf::hem::PORTED,
        sf::handex::PORTED,
        sf::events::PORTED,
        sf::server_side::PORTED,
        sf::adapter::PORTED,
    ]
    .iter()
    .any(|p| p.contains(&addr))
}

/// `021bb968(S, M, pokecon, queue, mode, heap)`: the flow work is created at the address the
/// differential tests use for it ([`super::flowcase::WORK_ADDR`]), so their `call` tables can turn
/// work pointers into offsets; the port runs on it.
fn flow_create(env: &mut Env, _lent: &mut Lent<'_>, args: &mut [Arg<'_>]) -> u32 {
    let v: Vec<u32> = (0..6).map(|i| super::val(args, i)).collect();
    let wa = super::flowcase::WORK_ADDR;
    env.objs.borrow_mut().map(
        wa,
        vec![0; crate::server_flow::SvFlowWork::SIZE],
        "flow work",
    );
    env.waddr = wa;
    env.pending_alloc = Some((wa, crate::server_flow::SvFlowWork::SIZE as u32));
    let r = env.with_work(&mut Lent::default(), |e, w| {
        crate::server_flow::core1::create(e, w, wa, v[0], v[1], v[2], v[3], v[4], v[5] as u16)
    });
    assert!(
        env.pending_alloc.is_none(),
        "B16: the flow work was not allocated by 021bb968"
    );
    r
}

/// Unported overlay-93 routines run as original Thumb code in the interpreter (test host): the
/// held-item factor registration of `battle_calc.s` (0x021DCEC4-0x021DCFB4: registry 0x021F1EA4)
/// and the move-handler one (0x021E02DC-0x021E03D7: registry 0x021F3030).
pub const OV93_THUMB: &[u32] = &[
    0x021D_CEC4,
    0x021D_CF24,
    0x021D_CF5C,
    0x021D_CF78,
    0x021D_CF9C,
    0x021D_CFA4,
    0x021D_CFB4,
    // move-handler factor registration (registry 0x021F3030): add / already-there check / remove
    0x021E_02DC,
    0x021E_0354,
    0x021E_0394,
    0x021E_0434,
    // turn-end side-effect walks (core4 `021c446c` stage 4): `021c4b8c` / `021c4c24` hand a
    // callback (`021c4b9c` / `021c4c40`) to the overlay-95 / field iterators
    0x021C_4B8C,
    0x021C_4B9C,
    0x021C_4C24,
    0x021C_4C40,
];
