//! `sick_cont.c` (0x021E8918-0x021E8C3F): the 32-bit condition words ([`SickCont`], the
//! verified ports). Value functions take and return the word; the setters (`021e8ac8`,
//! `021e8b28`, `021e8bd0`) take a pointer to it.

use super::{val, Bufs, Env};
use crate::server_flow::env::{Arg, Lent};
use crate::sick_cont::SickCont;

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v = |i: usize| val(args, i);
    let c = SickCont(v(0));
    Some(match addr {
        0x021E_8918 => SickCont::none().0,
        0x021E_8928 => c.is_none() as u32,
        0x021E_8940 => SickCont::turn(v(0)).0,
        0x021E_896C => SickCont::turn_param(v(0), v(1)).0,
        0x021E_899C => SickCont::poke(v(0)).0,
        0x021E_89C0 => SickCont::permanent().0,
        0x021E_89D4 => SickCont::permanent_param(v(0)).0,
        0x021E_89F8 => SickCont::permanent_count(v(0), v(1)).0,
        0x021E_8A28 => SickCont::poke_turn(v(0), v(1)).0,
        0x021E_8A58 => SickCont::badly_poisoned().0,
        0x021E_8A78 => c.is_badly_poisoned() as u32,
        0x021E_8A98 => c.linked_poke_id(),
        0x021E_8AFC => c.turns(),
        0x021E_8B80 => c.param_any() as u32,
        0x021E_8C24 => c.flag(),
        // setters through a pointer
        0x021E_8AC8 | 0x021E_8B28 | 0x021E_8BD0 => {
            let x = v(1);
            let b = Bufs::new(args).get(env, lent, 0, 4);
            let mut c = SickCont(u32::from_le_bytes(b[..4].try_into().unwrap()));
            match addr {
                0x021E_8AC8 => c.set_linked_poke_id(x),
                0x021E_8B28 => c.set_param_any(x),
                _ => c.set_flag(x),
            }
            Bufs::new(args).put(env, lent, 0, &c.0.to_le_bytes());
            0
        }
        _ => return None,
    })
}

/// `poke_type.c` (0x021E8CE0-0x021E8DBF): packed type pairs ([`crate::poke_type`]).
pub fn type_call(
    env: &mut Env,
    lent: &mut Lent<'_>,
    addr: u32,
    args: &mut [Arg<'_>],
) -> Option<u32> {
    use crate::poke_type as pt;
    let v = |i: usize| val(args, i);
    Some(match addr {
        0x021E_8CE0 => pt::make_pair(v(0) as u8, v(1) as u8) as u32,
        0x021E_8CF0 => pt::make_mono_pair(v(0) as u8) as u32,
        0x021E_8CFC => pt::pair_type1(v(0) as u16) as u32,
        0x021E_8D04 => pt::pair_type2(v(0) as u16) as u32,
        0x021E_8D0C => {
            let (a, b) = pt::split_pair(v(0) as u16);
            Bufs::new(args).put(env, lent, 1, &[a]);
            Bufs::new(args).put(env, lent, 2, &[b]);
            0
        }
        0x021E_8D24 => pt::pair_has_type(v(0) as u16, v(1) as u8) as u32,
        0x021E_8DA8 => pt::pair_is_mono(v(0) as u16) as u32,
        _ => return None,
    })
}
