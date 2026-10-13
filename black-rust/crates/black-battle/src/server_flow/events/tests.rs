use crate::battle_mon::{BattleMon, BppEnv};
use crate::server_flow::env::{Arg, FlowEnv};
use crate::server_flow::testkit::{run_area_suite, Case, TestEnv};
use crate::server_flow::SvFlowWork;

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CA8F8, "9d25d674b9832003"),
    (0x021CAA24, "1ea2cac46007136a"),
    (0x021CAA58, "62d4b5aafea2a69e"),
    (0x021CACF4, "cbb8f6dacf4945d8"),
    (0x021CAFD8, "5367856210339e6c"),
    (0x021CB174, "4d984a37ed944f75"),
    (0x021CB1A4, "68d8f70ad688192d"),
    (0x021CB1D4, "9f5cbc4b245821a4"),
    (0x021CB214, "fe2aae5d86964f69"),
    (0x021CB274, "f45726fc7e055657"),
    (0x021CB2D4, "e29eaac3f0aa23e7"),
    (0x021CB55C, "3c974d6c78287e9d"),
    (0x021CB5F0, "4bcdbfd61a1aea59"),
    (0x021CB730, "e36411acdb9b9b64"),
    (0x021CBA40, "8fa7fc2cf48b6a00"),
    (0x021CBB34, "5711bbfde1ef96d6"),
    (0x021CBCF4, "b45266a035375807"),
    (0x021CBD48, "dd2fa9fe48c267dc"),
    (0x021CBEA8, "c30b2d92f13614cc"),
    (0x021CBF30, "de9b72bda5797ef9"),
    (0x021CBF40, "fd7ab219b89e9eff"),
    (0x021CBF50, "3f86979647ebadf3"),
    (0x021CBF60, "b1a0602306172933"),
    (0x021CBF70, "ef536207babb9132"),
    (0x021CBF80, "ffe5d0d1cffac207"),
    (0x021CBF90, "0e63c3b8a1c62967"),
    (0x021CBFA0, "e84da423049479ab"),
    (0x021CC008, "2de68b692e3d9371"),
    (0x021CC098, "b56c5034868be420"),
    (0x021CC0A8, "453a16bb83bcc579"),
    (0x021CC0B8, "a27c9e7af64fb0e9"),
    (0x021CC0C8, "9ea0c806dbdbda72"),
    (0x021CC0D8, "fac9d361521b1eb9"),
    (0x021CC0E8, "733de9d376c48e39"),
    (0x021CC0F8, "210c5591004eae62"),
    (0x021CC178, "2dc0ce92fe24a4d4"),
    (0x021CC214, "8e2224c1c1552973"),
    (0x021CC2C0, "d56699fdc6302c90"),
    (0x021CC350, "e1e7e7b4f9073342"),
    (0x021CC398, "a80029377a000566"),
    (0x021CC438, "6b9b6892bf48aa04"),
    (0x021CC4AC, "000f34456fa42d92"),
    (0x021CC508, "704bbc817c14e1fe"),
    (0x021CC558, "373b2f044e94ffb8"),
    (0x021CC5F4, "305b1587c0860702"),
    (0x021CC668, "5854e5f4d3c9f262"),
    (0x021CC768, "507a221b0d12770a"),
    (0x021CC800, "f126c369cc927d64"),
    (0x021CC8A0, "b6fb62a26cc97655"),
    (0x021CC8B0, "5a9ac2e9c65edbfc"),
    (0x021CC8B8, "81985a42d310a6ff"),
    (0x021CC8C0, "120ec2c2d6671122"),
    (0x021CC8C8, "db26f7a74e521202"),
];
// svflow-recorded-end

/// The test environment's `BppEnv` is the `env bpp` mock.
impl BppEnv for TestEnv {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.bpp.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.bpp.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.bpp.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.bpp.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.bpp.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.bpp.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.bpp.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.bpp.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.bpp.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.bpp.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.bpp.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.bpp.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.bpp.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.bpp.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.bpp.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.bpp.sick_is_passable(sick)
    }
}

/// The item-effect handlers as the spec of `021cb730` models them: stubbed by address.
struct StubEffects;

impl<E: FlowEnv> super::ItemEffects<E> for StubEffects {
    fn run(
        &mut self,
        env: &mut E,
        _w: &mut SvFlowWork,
        waddr: u32,
        h: u32,
        _target: &mut Option<BattleMon>,
        target_addr: u32,
        item: u32,
        r3: u32,
        extra: u32,
        _io: &mut super::ItemIo<'_>,
    ) -> u32 {
        let args = &mut [
            Arg::V(waddr),
            Arg::V(target_addr),
            Arg::V(item),
            Arg::V(r3),
            Arg::V(extra),
        ];
        env.call(h, args)
    }
}

/// Resolver for BattleMon addresses a stub returned: `images` = (address, bytes) of the buffers.
fn mons_of(images: &[(u32, Vec<u8>)]) -> impl Fn(u32) -> BattleMon {
    let images = images.to_vec();
    move |a: u32| {
        let (base, img) = images
            .iter()
            .find(|(b, img)| a >= *b && ((a - *b) as usize) < img.len())
            .unwrap_or_else(|| panic!("no image at {a:#x}"));
        let o = (a - base) as usize;
        BattleMon::from_bytes(img[o..o + crate::battle_mon::BPP_SIZE].try_into().unwrap())
    }
}

/// The BattleMon behind buffer argument `i`.
fn mon<E>(c: &Case<E>, i: usize) -> BattleMon {
    BattleMon::from_bytes(c.bufs[i].as_slice().try_into().unwrap())
}

pub(crate) fn call<E: FlowEnv + BppEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    match addr {
        0x021C_B274 => {
            let mon = mon(c, 1);
            super::dispatch_value(&mut c.env, c.vals[0], &mon, &c.bufs[2], c.vals[3]) as u64
        }
        0x021C_C768 => {
            let mons = mons_of(&[(c.vals[4], c.bufs[4].clone())]);
            super::compare_sides(
                &mut c.env, &c.work, &mons, &c.bufs[1], &c.bufs[2], c.vals[3],
            ) as u64
        }
        0x021C_C8A0 => {
            super::table_fill_6(&mut c.bufs[0]);
            0
        }
        0x021C_C8B0 => {
            super::table_set(&mut c.bufs[0], c.vals[1], c.vals[2]);
            0
        }
        0x021C_C8B8 | 0x021C_C8C0 => super::table_get(&c.bufs[0], c.vals[1]) as u64,
        0x021C_C8C8 => {
            super::queue_clear(&mut c.bufs[0]);
            0
        }
        0x021C_B174 => super::mon_of_main_a(&mut c.env, c.vals[0], &c.bufs[0]) as u64,
        0x021C_B1A4 => super::mon_of_main_b(&mut c.env, c.vals[0], &c.bufs[0]) as u64,
        0x021C_B214 => {
            let mon = mon(c, 1);
            super::dispatch_check(&mut c.env, c.vals[0], &mon, &c.bufs[2]) as u64
        }
        0x021C_B1D4 => {
            let mon = mon(c, 1);
            super::dispatch_value_bracketed(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &mon,
                &c.bufs[2],
                c.vals[3],
            ) as u64
        }
        0x021C_BFA0 => {
            let mon = mon(c, 1);
            super::msg_side(&mut c.env, &mut c.work, c.vals[0], &mon) as u64
        }
        0x021C_C008 => {
            let mon = mon(c, 1);
            super::revive(&mut c.env, &mut c.work, c.vals[0], &mon, c.vals[2]) as u64
        }
        0x021C_C0F8 => {
            let mut m = mon(c, 1);
            let r = super::raise_crit(
                &mut c.env,
                &mut c.bufs[4],
                &mut c.globals[0],
                &mut m,
                c.vals[3],
            );
            c.bufs[1].copy_from_slice(&m.raw);
            r as u64
        }
        0x021C_C178 => {
            let mon = mon(c, 1);
            super::pp_restore_one(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &mon,
                c.vals[2],
                c.vals[4],
            ) as u64
        }
        0x021C_C214 => {
            let mon = mon(c, 1);
            super::pp_restore_all(&mut c.env, &mut c.work, c.vals[0], &mon, c.vals[2]) as u64
        }
        0x021C_C2C0 => {
            let mon = mon(c, 1);
            super::heal(&mut c.env, &mut c.work, c.vals[0], &mon, c.vals[2]) as u64
        }
        0x021C_C350 => {
            let mon = mon(c, 1);
            super::cure_status(&mut c.env, &mut c.work, c.vals[0], &mon, c.vals[4]) as u64
        }
        0x021C_C398 => {
            let mon = mon(c, 1);
            super::cure_all_statuses(&mut c.env, &mut c.work, c.vals[0], &mon) as u64
        }
        0x021C_C438 => {
            let mon = mon(c, 1);
            super::change_stage(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &mon,
                c.vals[3],
                c.vals[4],
            ) as u64
        }
        0x021C_C4AC => {
            let mon = mon(c, 1);
            super::held_item_check(&mut c.env, c.vals[0], &mon, c.vals[1]) as u64
        }
        0x021C_C508 => {
            let mon = mon(c, 1);
            super::ability_trigger(&mut c.env, &mut c.work, c.vals[0], &mon, c.vals[1]) as u64
        }
        0x021C_C558 => {
            let mon = mon(c, 1);
            super::show_held_item(&mut c.env, &mut c.work, c.vals[0], &mon) as u64
        }
        0x021C_C5F4 => {
            let mon = mon(c, 1);
            super::flinch_message(&mut c.env, &mut c.work, c.vals[0], &mon) as u64
        }
        0x021C_BF30 | 0x021C_BF40 | 0x021C_BF50 | 0x021C_BF60 | 0x021C_BF70 | 0x021C_BF80 => {
            let mon = mon(c, 1);
            let (e, w, a) = (&mut c.env, &mut c.work, c.vals[0]);
            (match addr {
                0x021C_BF30 => super::cure_entry_2(e, w, a, &mon),
                0x021C_BF40 => super::cure_entry_5(e, w, a, &mon),
                0x021C_BF50 => super::cure_entry_4(e, w, a, &mon),
                0x021C_BF60 => super::cure_entry_3(e, w, a, &mon),
                0x021C_BF70 => super::cure_entry_1(e, w, a, &mon),
                _ => super::cure_entry_6(e, w, a, &mon),
            }) as u64
        }
        0x021C_BF90 => {
            let mon = mon(c, 1);
            super::cure_all_entry(&mut c.env, &mut c.work, c.vals[0], &mon) as u64
        }
        0x021C_C098 | 0x021C_C0A8 | 0x021C_C0B8 | 0x021C_C0C8 | 0x021C_C0D8 | 0x021C_C0E8 => {
            let mon = mon(c, 1);
            let (e, w, a, n) = (&mut c.env, &mut c.work, c.vals[0], c.vals[3]);
            (match addr {
                0x021C_C098 => super::stage_entry_1(e, w, a, &mon, n),
                0x021C_C0A8 => super::stage_entry_2(e, w, a, &mon, n),
                0x021C_C0B8 => super::stage_entry_3(e, w, a, &mon, n),
                0x021C_C0C8 => super::stage_entry_4(e, w, a, &mon, n),
                0x021C_C0D8 => super::stage_entry_5(e, w, a, &mon, n),
                _ => super::stage_entry_6(e, w, a, &mon, n),
            }) as u64
        }
        0x021C_C668 => {
            let mons = mons_of(&[(c.vals[1], c.bufs[1].clone())]);
            super::who_leads(&mut c.env, &c.work, c.vals[0], &mons) as u64
        }
        0x021C_C800 => {
            let img = c.bufs[4].clone();
            let base = c.vals[4];
            let mons = move |a: u32| {
                let o = a.wrapping_sub(base) as usize;
                BattleMon::from_bytes(img[o..o + crate::battle_mon::BPP_SIZE].try_into().unwrap())
            };
            let (mut hp, mut pct) = (c.bufs[1].clone(), c.bufs[2].clone());
            super::side_totals(&mut c.env, &c.work, &mons, &mut hp, &mut pct, c.vals[3]);
            c.bufs[1] = hp;
            c.bufs[2] = pct;
            0
        }
        0x021C_BCF4 => super::capture_bonus(&mut c.env, &c.work) as u64,
        0x021C_BEA8 => super::capture_shake(&mut c.env, &c.work, c.vals[1] as i32) as u64,
        0x021C_BD48 => {
            let mon = mon(c, 2);
            super::ball_multiplier(
                &mut c.env, &c.work, &mon, c.vals[2], c.vals[3], c.vals[4], &c.bufs[4],
            ) as u64
        }
        0x021C_B55C => {
            let m = mon(c, 0);
            super::exp_scale(&mut c.env, &m, c.vals[1], c.vals[2], c.vals[3]) as u64
        }
        0x021C_B5F0 => {
            let (winner, loser) = (mon(c, 0), mon(c, 1));
            super::ev_gain(&mut c.env, &winner, &loser, &mut c.bufs[2]);
            0
        }
        0x021C_B2D4 => {
            let fainted = mon(c, 2);
            let img = c.bufs[4].clone();
            let base = c.vals[4];
            let mons = move |a: u32| {
                let o = a.wrapping_sub(base) as usize;
                BattleMon::from_bytes(img[o..o + crate::battle_mon::BPP_SIZE].try_into().unwrap())
            };
            super::exp_distribute(
                &mut c.env,
                &c.work,
                c.vals[1],
                &fainted,
                &mut c.bufs[3],
                &mons,
            );
            0
        }
        0x021C_BB34 => {
            let m = mon(c, 2);
            let img = c.bufs[2].clone();
            let (mut count, mut flag) = (c.bufs[4].clone(), c.bufs[5].clone());
            let r = super::capture_roll(
                &mut c.env, &c.work, &m, c.vals[2], c.vals[3], &mut count, &mut flag, c.vals[2],
                &img,
            );
            c.bufs[4] = count;
            c.bufs[5] = flag;
            r as u64
        }
        0x021C_BA40 => {
            let thrower = mon(c, 1);
            let img = c.bufs[4].clone();
            let base = c.vals[4];
            let mons = {
                let img = img.clone();
                move |a: u32| {
                    let o = a.wrapping_sub(base) as usize;
                    BattleMon::from_bytes(
                        img[o..o + crate::battle_mon::BPP_SIZE].try_into().unwrap(),
                    )
                }
            };
            super::throw_ball(
                &mut c.env,
                &mut c.work,
                &thrower,
                c.vals[2],
                &mut c.bufs[3],
                &mut c.globals[0],
                &mons,
                base,
                &img,
            ) as u64
        }
        0x021C_B730 => {
            let user = mon(c, 1);
            let mut q = std::mem::take(&mut c.bufs[5]);
            let mut img = std::mem::take(&mut c.bufs[6]);
            let r = super::use_item(
                &mut c.env,
                &mut StubEffects,
                &mut c.work,
                c.vals[0],
                &user,
                c.vals[2],
                c.vals[3],
                c.vals[4],
                &mut super::ItemIo {
                    q: &mut q,
                    scratch: &mut c.globals[0],
                },
                &mut img,
                c.vals[6],
            );
            c.bufs[5] = q;
            c.bufs[6] = img;
            r as u64
        }
        0x021C_A8F8 => {
            let m = mon(c, 1);
            super::message_args_a(
                &mut c.env, &c.work, c.vals[0], &m, c.vals[1], &c.bufs[3], c.vals[4], c.vals[5],
            ) as u64
        }
        0x021C_AA24 => super::target_override(
            &mut c.env, &c.work, &c.bufs[1], c.vals[2], c.vals[3], c.vals[4],
        ) as u64,
        0x021C_AA58 => {
            let m = mon(c, 1);
            let mons = mons_of(&[(c.vals[6], c.bufs[6].clone())]);
            super::message_args_b(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &m,
                c.vals[1],
                c.vals[2],
                &c.bufs[3],
                c.vals[4],
                c.vals[5],
                &mons,
            ) as u64
        }
        0x021C_ACF4 => {
            let m = mon(c, 1);
            let mons = mons_of(&[(c.vals[6], c.bufs[6].clone())]);
            super::message_args_c(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &m,
                c.vals[1],
                c.vals[2],
                &c.bufs[3],
                c.vals[4],
                c.vals[5],
                &c.bufs[7],
                &mons,
            ) as u64
        }
        0x021C_AFD8 => {
            let m = mon(c, 2);
            let mons = mons_of(&[
                (c.vals[6], c.bufs[6].clone()),
                (c.vals[7], c.bufs[7].clone()),
            ]);
            let mv = u16::from_le_bytes([c.bufs[3][0], c.bufs[3][1]]) as u32;
            super::pick_ally(
                &mut c.env, &c.work, c.vals[1], &m, mv, c.vals[4], c.vals[5], &mons,
            ) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py events`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "events",
        include_str!("../../../../../tools/rom-diff/svflow_specs/events.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
