use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CC8D8, "d097d3ab3117f55d"),
    (0x021CC918, "3c8908fea37933f0"),
    (0x021CC95C, "b204d0358fadc09c"),
    (0x021CC960, "5c90a01065a72408"),
    (0x021CC96C, "7b8e2a3672f32ef7"),
    (0x021CC974, "11d709461690ebce"),
    (0x021CC99C, "5fe294e406f19d24"),
    (0x021CC9A4, "5fce965bb416ca99"),
    (0x021CC9AC, "fd70e7631c94754a"),
    (0x021CCA74, "5c2b0c1336b4b8db"),
    (0x021CCAA8, "717b641ac0bee7b1"),
    (0x021CCAD4, "adfe3c2811d60e90"),
    (0x021CCAE0, "c4915fb6b925e9ee"),
    (0x021CCB18, "29cea320cc2cd339"),
    (0x021CCB34, "1bcea44f4436a5b1"),
    (0x021CCB78, "f8d44c27ef736f26"),
    (0x021CCB98, "9461f84b745d8068"),
    (0x021CCBE8, "b0aa1f2636e7bde7"),
    (0x021CCC0C, "080d7579a66e13b9"),
    (0x021CD0F0, "05d04f801bd38593"),
    (0x021CD4B0, "3828818267e10b8f"),
    (0x021CD4FC, "235235c79a65bf80"),
    (0x021CD568, "f40cf96b7d34e260"),
    (0x021CD5E0, "e90aebcbfb5f752e"),
    (0x021CD64C, "3ed1b49993961442"),
    (0x021CD6AC, "6dad57048fa76e10"),
    (0x021CD6EC, "050bb2865e26402f"),
];
// svflow-recorded-end

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    match addr {
        0x021CC8D8 => super::advance_fields(&mut c.bufs[0]) as u64,
        0x021CC918 => super::advance_fields_set_low10(&mut c.bufs[0], c.vals[1]) as u64,
        0x021CC95C => {
            super::set_word(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021CC960 => super::word_low10(&c.bufs[0]) as u64,
        0x021CC96C => super::word_flag30(&c.bufs[0]) as u64,
        0x021CC974 => {
            super::set_mode_flag30(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021CC99C => super::word_bits28_29(&c.bufs[0]) as u64,
        0x021CCAA8 => {
            super::push_u8(&mut c.env, &mut c.bufs[0], c.vals[1] as u8);
            0
        }
        0x021CCAD4 => super::pop_u8(&mut c.bufs[0]) as u64,
        0x021CCAE0 => {
            super::push_u16(&mut c.env, &mut c.bufs[0], c.vals[1] as u16);
            0
        }
        0x021CCB18 => super::pop_u16(&mut c.bufs[0]) as u64,
        0x021CCB78 => super::pop_u24(&mut c.bufs[0]) as u64,
        0x021CD4B0 => {
            // the original reads words past r3 from the caller's stack: vals[2..] are r2, r3, stack
            super::put_cmd(
                &mut c.env,
                &mut c.bufs[0],
                c.vals[1],
                &c.vals[2..17],
                &mut c.globals[0],
            );
            0
        }
        0x021CC9A4 => super::word_flag29(&c.bufs[0]) as u64,
        0x021CC9AC => super::alloc_record(&mut c.bufs[0], c.vals[1], c.vals[2]) as u64,
        0x021CCA74 => {
            // vals[1] is the record's address; the port takes its offset from arg0
            super::free_last_record(&mut c.bufs[0], c.vals[1].wrapping_sub(c.vals[0]));
            0
        }
        0x021CCB34 => {
            super::push_u24(&mut c.env, &mut c.bufs[0], c.vals[1]);
            0
        }
        0x021CCB98 => {
            super::push_u32(&mut c.env, &mut c.bufs[0], c.vals[1]);
            0
        }
        0x021CCBE8 => super::pop_u32(&mut c.bufs[0]) as u64,
        0x021CCC0C => {
            let args = c.bufs[3].clone();
            super::write_cmd(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2], &args);
            0
        }
        0x021CD0F0 => {
            let (q, out) = c.bufs.split_at_mut(1);
            super::read_cmd_args(&mut q[0], c.vals[1], &mut out[1]);
            0
        }
        0x021CD4FC => {
            super::put_cmd_placeholder(&mut c.env, &mut c.bufs[0], c.vals[1], &mut c.globals[0])
                as u64
        }
        0x021CD568 => {
            // vals[3..] = r3 then the stack words
            super::put_cmd_at(
                &mut c.env,
                &mut c.bufs[0],
                c.vals[1],
                c.vals[2],
                &c.vals[3..],
                &mut c.globals[0],
            );
            0
        }
        0x021CD5E0 => {
            let (q, out) = c.bufs.split_at_mut(1);
            super::read_cmd(&mut q[0], &mut out[0]) as u64
        }
        0x021CD64C => {
            // vals[3..] = r3 then the stack words; the original goes on reading zeros past them
            let mut rest = c.vals[3..].to_vec();
            rest.resize(1100, 0);
            super::put_cmd_list(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2], &rest);
            0
        }
        0x021CD6AC => {
            let (q, out) = c.bufs.split_at_mut(1);
            super::read_cmd_list(&mut q[0], c.vals[1], &mut out[1]);
            0
        }
        0x021CD6EC => {
            super::nop_021cd6ec();
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py cmd_queue`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "cmd_queue",
        include_str!("../../../../../tools/rom-diff/svflow_specs/cmd_queue.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
