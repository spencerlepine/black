//! `fldeff_namipoke.c` (overlay 21, `0x021ACA38..0x021AD758`): the surf Pokemon field effect: the effect task the
//! player core adds when surfing starts (021ACB38), its state (mode +9: 0 off, 1 bobbing, 2 ridden; +0xA hide), the
//! position logic of its update procs (bobbing offset applied to the player MMDL, the follower that trails the player),
//! and the splash effect of the surf start / end jumps (021AD564 / 021AD598).
//!
//! F3a-spec specced the 18 logic / mixed functions (`tools/rom-diff/specs/black-field/namipoke.spec`, header: task
//! work layouts); the 3D-model resource loads / frees and the draw procs (021ACA70, 021ACB0C, 021ACCB0, 021ACFD8,
//! 021AD274, 021AD410, 021AD530, 021AD6A4, 021AD738) are render-only (D3: U2 rewrites them) and are reached only as
//! addresses. F3a ported all 18 (every callee outside the area, the render-only functions included, through
//! [`FieldEnv::call`] by address; the original's stack vectors are [`Mem::push_local`] windows).

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_CA38,
    0x021A_CA5C,
    0x021A_CB38,
    0x021A_CBB8,
    0x021A_CBC8,
    0x021A_CBE4,
    0x021A_CC04,
    0x021A_CC3C,
    0x021A_CCAC,
    0x021A_CDC4,
    0x021A_CFF8,
    0x021A_D3D4,
    0x021A_D3FC,
    0x021A_D564,
    0x021A_D598,
    0x021A_D5CC,
    0x021A_D5E0,
    0x021A_D6B8,
];

/// `FUN_021AC1E0`: the work of an effect task (field effect system, U2).
const TASK_WORK: u32 = 0x021A_C1E0;
const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// Address of the string `"fldeff_namipoke.c"`.
const FILE_NAMIPOKE: u32 = 0x021D_E080;
// render-only functions of this file (D3, U2), reached by address
const SURF_MODEL_LOAD: u32 = 0x021A_CA70;
const SURF_MODEL_FREE: u32 = 0x021A_CB0C;
const SURF_DRAW: u32 = 0x021A_CCB0;
const SPLASH_MODEL_LOAD: u32 = 0x021A_D410;
const SPLASH_MODEL_FREE: u32 = 0x021A_D530;
// field effect system (overlay 21)
const FX_RES_SYS: u32 = 0x021A_BE18; // 3D resource system of a control
const FX_WORK_OF: u32 = 0x021A_BEA0; // work of an effect id (ctrl, id)
const FX_ADD_TASK: u32 = 0x021A_BFA8; // add task (ctrl, table, &pos / x, prio, &init, flag)
const TASK_DELETE: u32 = 0x021A_C154;
const TASK_UPDATE: u32 = 0x021A_C178;
const TASK_PARAM: u32 = 0x021A_C1A0;
const TASK_INIT: u32 = 0x021A_C1AC; // task init data
const TASK_POS_GET: u32 = 0x021A_C1B8; // task position -> out
const TASK_POS_SET: u32 = 0x021A_C1CC; // task position = *v
                                       // fld3d (render)
const ANIM_CREATE: u32 = 0x021C_6590;
const ANIM_STEP_DONE: u32 = 0x021C_6694;
const ANIM_LOOP: u32 = 0x021C_66EC;
// fldmmdl (F4a; force stubs here)
const MMDL_STATUS_TEST: u32 = 0x0216_D55C;
const MMDL_OBJ_ID: u32 = 0x0216_D588;
const MMDL_ZONE: u32 = 0x0216_D590;
const MMDL_OBJ_CODE: u32 = 0x0216_D598;
const MMDL_DISP_DIR: u32 = 0x0216_D5F8;
const MMDL_POS: u32 = 0x0216_D88C;
const MMDL_DRAW_OFS_GET: u32 = 0x0216_D8D0;
const MMDL_DRAW_OFS_SET: u32 = 0x0216_D8E0;
const MMDL_DRAW_POS: u32 = 0x0219_7CCC; // fldmmdl_acmd
const RAIL_DIR_VEC: u32 = 0x0219_E784; // rail: direction vector (m, dir, &VecFx16)
const VEC16_NORMALIZE: u32 = 0x0207_CDB8;
const VEC_DISTANCE: u32 = 0x0207_CF3C;
/// ROM `0x021D915C` / `0x021D9184` / `0x021D9170`: the effect task tables (rail surf, grid surf, splash).
const TABLE_RAIL: u32 = 0x021D_915C;
const TABLE_GRID: u32 = 0x021D_9184;
const TABLE_SPLASH: u32 = 0x021D_9170;
/// ROM `0x021D91F8`: the splash offset VecFx32 per kind 0..4 (only kind 2 is nonzero: y -0x4000).
const SPLASH_OFFSETS: [[i32; 3]; 5] = [[0; 3], [0; 3], [0, -0x4000, 0], [0; 3], [0; 3]];

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn vec_get<E: FieldEnv>(env: &mut E, p: u32) -> [u32; 3] {
    [
        env.r32(p),
        env.r32(p.wrapping_add(4)),
        env.r32(p.wrapping_add(8)),
    ]
}

fn vec_set<E: FieldEnv>(env: &mut E, p: u32, v: [u32; 3]) {
    for (i, x) in v.iter().enumerate() {
        env.w32(p.wrapping_add(4 * i as u32), *x);
    }
}

/// `(v * 2^k + 0x800) >> 12` in 64 bits, low word (the `lsls / adds / adcs / lsrs | lsls` sequences of the rail maths).
fn fx64(v: i64) -> u32 {
    (v.wrapping_add(0x800) >> 12) as u32
}

/// C: `FUN_021ACA38` (unlabeled routine at `0x021ACA38`): allocate the surf RESOURCE block (0xC,
/// `Heap_AllocDebug(heap, 0xC, 1, "fldeff_namipoke.c", 0xAC)`), `+0` = the field effect control, load its models
/// (`FUN_021ACA70`, render); returns the block.
pub fn res_create<E: FieldEnv>(env: &mut E, ctrl: u32, heap: u32) -> u32 {
    let r = env.callv(HEAP_ALLOC, &[heap, 0xC, 1, FILE_NAMIPOKE, 0xAC]);
    env.w32(r, ctrl);
    c1(env, SURF_MODEL_LOAD, r);
    r
}

/// C: `FUN_021ACA5C` (unlabeled routine at `0x021ACA5C`): free the surf models (`FUN_021ACB0C`, render) and the
/// RESOURCE block.
pub fn res_delete<E: FieldEnv>(env: &mut E, _ctrl: u32, res: u32) {
    c1(env, SURF_MODEL_FREE, res);
    c1(env, HEAP_FREE, res);
}

/// C: `FUN_021ACB38` (unlabeled routine at `0x021ACB38`): add the surf Pokemon effect task for MMDL `m`: init data
/// `{FUN_021ABEA0(ctrl, 3), m, u16 OBJ id, u16 zone id, u16 dir}` and a copy of `*pos` on the stack; the rail table
/// (ROM `0x021D915C`) when the MMDL status has 0x2000, else the grid table (`0x021D9184`):
/// `FUN_021ABFA8(ctrl, table, &pos, flag, &init, 0)`; returns the task.
pub fn add_surf_task<E: FieldEnv>(
    env: &mut E,
    ctrl: u32,
    dir: u32,
    pos: u32,
    m: u32,
    flag: u32,
) -> u32 {
    let res = env.callv(FX_WORK_OF, &[ctrl, 3]);
    let mut init = [0u8; 14];
    init[0..4].copy_from_slice(&res.to_le_bytes());
    init[4..8].copy_from_slice(&m.to_le_bytes());
    let id = c1(env, MMDL_OBJ_ID, m);
    init[8..10].copy_from_slice(&(id as u16).to_le_bytes());
    let zone = c1(env, MMDL_ZONE, m);
    init[10..12].copy_from_slice(&(zone as u16).to_le_bytes());
    init[12..14].copy_from_slice(&(dir as u16).to_le_bytes());
    let mut p = [0u8; 12];
    for (i, b) in p.iter_mut().enumerate() {
        *b = env.r8(pos.wrapping_add(i as u32));
    }
    let table = if env.callv(MMDL_STATUS_TEST, &[m, 0x2000]) != 0 {
        TABLE_RAIL
    } else {
        TABLE_GRID
    };
    env.call(
        FX_ADD_TASK,
        &mut [
            CArg::V(ctrl),
            CArg::V(table),
            CArg::InBuf(&p),
            CArg::V(flag),
            CArg::InBuf(&init),
            CArg::V(0),
        ],
    )
}

/// C: `FUN_021ACBB8` (unlabeled routine at `0x021ACBB8`): set the surf effect mode (`work +9`, u8) of a task;
/// nothing for a null task.
pub fn set_mode<E: FieldEnv>(env: &mut E, task: u32, mode: u32) {
    if task != 0 {
        let w = env.callv(TASK_WORK, &[task]);
        env.mem().w8(w.wrapping_add(9), mode as u8);
    }
}

/// C: `FUN_021ACBC8` (unlabeled routine at `0x021ACBC8`): show / hide the surf Pokemon: `work +0xA` = 0 when `on`
/// is 1, else 1; nothing for a null task.
pub fn set_visible<E: FieldEnv>(env: &mut E, task: u32, on: u32) {
    if task != 0 {
        let w = env.callv(TASK_WORK, &[task]);
        env.w8(w.wrapping_add(0xA), if on == 1 { 0 } else { 1 });
    }
}

/// C: `FUN_021ACBE4` (unlabeled routine at `0x021ACBE4`): the animation speed of a follower distance:
/// `min(d, 0x40000) / 0xA3D * 0x28` (signed, the division truncates toward zero).
pub fn follow_speed(d: i32) -> i32 {
    d.min(0x4_0000).wrapping_div(0xA3D).wrapping_mul(0x28)
}

/// C: `FUN_021ACC04` (unlabeled routine at `0x021ACC04`): how far VecFx32 `b` is ahead of `a` along direction
/// `dir` (0 up: `b.z - a.z`, 1 down: `a.z - b.z`, 2 left: `b.x - a.x`, 3 right: `a.x - b.x`), 0 for other dirs.
pub fn ahead_along(mem: &Mem, a: u32, b: u32, dir: u32) -> u32 {
    let r = |p: u32| mem.r32(p);
    match dir {
        0 => r(b.wrapping_add(8)).wrapping_sub(r(a.wrapping_add(8))),
        1 => r(a.wrapping_add(8)).wrapping_sub(r(b.wrapping_add(8))),
        2 => r(b).wrapping_sub(r(a)),
        3 => r(a).wrapping_sub(r(b)),
        _ => 0,
    }
}

/// C: `FUN_021ACC3C` (unlabeled routine at `0x021ACC3C`): init proc of the surf task: copy the init data
/// (`FUN_021AC1AC`, 0x1C bytes) to `w +0x1C`; task position = init `+0x10` (`FUN_021AC1CC`); `w +0xC` = init dir;
/// `FUN_021ABE18(RESOURCE +0)` (result unused); `w +0x18` = RESOURCE `+8`; `w +9` = `FUN_021AC1A0(task)`; bob height
/// `+0x10` = 0x1000, step `+0x14` = 0x400; follower: `+0x54` = RESOURCE `+0xA`, `+0x38` = 1, `+0x3A` = 9, scale
/// `+0x58..+0x60` = 0x1000; `FUN_021AC178(task)`.
pub fn surf_init<E: FieldEnv>(env: &mut E, task: u32, w: u32) {
    let i = c1(env, TASK_INIT, task);
    env.mem().copy(w.wrapping_add(0x1C), i, 0x1C);
    env.call(
        TASK_POS_SET,
        &mut [CArg::V(task), CArg::In(i.wrapping_add(0x10), 12)],
    );
    let d = env.r16(i.wrapping_add(0xC));
    env.w16(w.wrapping_add(0xC), d);
    let res = env.r32(w.wrapping_add(0x1C));
    let r0 = env.r32(res);
    c1(env, FX_RES_SYS, r0);
    let res = env.r32(w.wrapping_add(0x1C));
    let h = env.r16(res.wrapping_add(8));
    env.w16(w.wrapping_add(0x18), h);
    let p = c1(env, TASK_PARAM, task);
    env.w8(w.wrapping_add(9), p as u8);
    env.w32(w.wrapping_add(0x14), 0x400);
    env.w32(w.wrapping_add(0x10), 0x1000);
    let res = env.r32(w.wrapping_add(0x1C));
    let h = env.r16(res.wrapping_add(0xA));
    let f = w.wrapping_add(0x38);
    env.w16(f.wrapping_add(0x1C), h);
    env.w16(f, 1);
    env.w16(f.wrapping_add(2), 9);
    for o in [0x20, 0x24, 0x28] {
        env.w32(f.wrapping_add(o), 0x1000);
    }
    c1(env, TASK_UPDATE, task);
}

/// C: `FUN_021ACCAC` (unlabeled routine at `0x021ACCAC`): the empty draw proc of the grid surf effect.
pub fn draw_none() {}

/// The bob step of both update procs: height `+0x10` += step `+0x14`; at or above 0x4000 it is set to 0x4000, at or
/// below 0x1000 to 0x1000, and the step is negated there.
fn bob<E: FieldEnv>(env: &mut E, w: u32) {
    let h = env.r32(w.wrapping_add(0x10));
    let step = env.r32(w.wrapping_add(0x14));
    let n = h.wrapping_add(step);
    env.w32(w.wrapping_add(0x10), n);
    if n as i32 >= 0x4000 {
        env.w32(w.wrapping_add(0x10), 0x4000);
    } else if n as i32 > 0x1000 {
        return;
    } else {
        env.w32(w.wrapping_add(0x10), 0x1000);
    }
    env.w32(w.wrapping_add(0x14), step.wrapping_neg());
}

/// Task position = MMDL draw position (`FUN_02197CCC`) - `off`, with `y += bob - 0x1000` when `bob_y`.
fn place_task<E: FieldEnv>(env: &mut E, task: u32, w: u32, m: u32, off: [u32; 3], bob_y: bool) {
    let mut p = [0u8; 12];
    env.call(MMDL_DRAW_POS, &mut [CArg::V(m), CArg::OutBuf(&mut p)]);
    let rd = |i: usize| u32::from_le_bytes(p[4 * i..4 * i + 4].try_into().unwrap());
    let mut pos = [
        rd(0).wrapping_sub(off[0]),
        rd(1).wrapping_sub(off[1]),
        rd(2).wrapping_sub(off[2]),
    ];
    if bob_y {
        let h = env.r32(w.wrapping_add(0x10));
        pos[1] = pos[1].wrapping_add(h.wrapping_sub(0x1000));
    }
    let mut b = [0u8; 12];
    for i in 0..3 {
        b[4 * i..4 * i + 4].copy_from_slice(&pos[i].to_le_bytes());
    }
    env.call(TASK_POS_SET, &mut [CArg::V(task), CArg::InBuf(&b)]);
}

/// The ridden / hidden case of both update procs: follower dir `+0x3A` = 9, and `+0x38` = 1 when hidden. True
/// when it applies (mode 2 or `+0xA` == 1).
fn follower_off<E: FieldEnv>(env: &mut E, w: u32) -> bool {
    if env.r8(w.wrapping_add(9)) == 2 || env.r8(w.wrapping_add(0xA)) == 1 {
        let f = w.wrapping_add(0x38);
        env.w16(f.wrapping_add(2), 9);
        if env.r8(w.wrapping_add(0xA)) == 1 {
            env.w16(f, 1);
        }
        true
    } else {
        false
    }
}

/// The animation loop step of the follower (`FUN_021C66EC(FUN_021ABE18(RESOURCE +0), +0x54, 0x1000)`) and the MMDL
/// position (`FUN_0216D88C`) in a new local window, returned.
fn follower_head<E: FieldEnv>(env: &mut E, w: u32, m: u32) -> u32 {
    let res = env.r32(w.wrapping_add(0x1C));
    let r0 = env.r32(res);
    let sys = c1(env, FX_RES_SYS, r0);
    let h = env.r16(w.wrapping_add(0x54)) as u32;
    env.callv(ANIM_LOOP, &[sys, h, 0x1000]);
    let cur = env.mem().push_local(12);
    env.call(MMDL_POS, &mut [CArg::V(m), CArg::Out(cur, 12)]);
    cur
}

/// C: `FUN_021ACDC4` (unlabeled routine at `0x021ACDC4`): update proc of the grid surf task. Mode `+9` 0: only the
/// draw (`FUN_021ACCB0`, render). Else `+0xC` = the MMDL display dir; mode 1: bob ([`bob`]) and set the MMDL draw
/// offset (`FUN_0216D8E0`) to `(0, bob + 0x1000, 0x5000)` (`y = bob + 0x8000` for OBJ codes 0xA0 / 0xA1); mode 2:
/// the offset is the MMDL's (`FUN_0216D8D0`); task position = draw position - offset (`y += bob - 0x1000` unless
/// mode 2). Ridden / hidden: no follower. Else the follower `+0x38..` tracks the MMDL position: a new dir copies it;
/// the same position approaches by 0x4000 along the dir (clamped at the MMDL); [`ahead_along`] >= 0x40000 snaps it
/// 0x40000 behind; speed [`follow_speed`] < 40: still (`+0x38` = 1), else moving with scale = speed; the MMDL position
/// is remembered (`+0x48`). Then the draw.
pub fn surf_update_grid<E: FieldEnv>(env: &mut E, task: u32, w: u32) {
    let mut off = [0u32; 3];
    if env.r8(w.wrapping_add(9)) == 0 {
        env.callv(SURF_DRAW, &[task, w]);
        return;
    }
    let m = env.r32(w.wrapping_add(0x20));
    let d = c1(env, MMDL_DISP_DIR, m);
    env.w16(w.wrapping_add(0xC), d as u16);
    let mode = env.r8(w.wrapping_add(9));
    if mode == 1 {
        bob(env, w);
        let h = env.r32(w.wrapping_add(0x10));
        off = [0, h.wrapping_add(0x1000), 0x5000];
        let code = c1(env, MMDL_OBJ_CODE, m);
        if (code.wrapping_add(0xFF60) as u16) <= 1 {
            off[1] = env.r32(w.wrapping_add(0x10)).wrapping_add(0x8000);
        }
        let mut b = [0u8; 12];
        for i in 0..3 {
            b[4 * i..4 * i + 4].copy_from_slice(&off[i].to_le_bytes());
        }
        env.call(MMDL_DRAW_OFS_SET, &mut [CArg::V(m), CArg::InBuf(&b)]);
    } else if mode == 2 {
        let mut b = [0u8; 12];
        env.call(MMDL_DRAW_OFS_GET, &mut [CArg::V(m), CArg::OutBuf(&mut b)]);
        for (i, o) in off.iter_mut().enumerate() {
            *o = u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap());
        }
    }
    let bob_y = env.r8(w.wrapping_add(9)) != 2;
    place_task(env, task, w, m, off, bob_y);
    if !follower_off(env, w) {
        let f = w.wrapping_add(0x38);
        let cur = follower_head(env, w, m);
        let c = vec_get(env, cur);
        let fd = env.r16(f.wrapping_add(2)) as u32;
        let dd = env.r16(w.wrapping_add(0xC)) as u32;
        if dd != fd {
            env.w16(f.wrapping_add(2), dd as u16);
            vec_set(env, f.wrapping_add(4), c);
        } else if c == vec_get(env, f.wrapping_add(0x10)) && fd <= 3 {
            let (o, ci) = if fd < 2 { (0xC, 2) } else { (4, 0) };
            let v = env.r32(f.wrapping_add(o));
            let n = if fd & 1 == 0 {
                v.wrapping_sub(0x4000)
            } else {
                v.wrapping_add(0x4000)
            };
            env.w32(f.wrapping_add(o), n);
            let past = if fd & 1 == 0 {
                (n as i32) < c[ci] as i32
            } else {
                (n as i32) > c[ci] as i32
            };
            if past {
                env.w32(f.wrapping_add(o), c[ci]);
            }
        }
        let fd = env.r16(f.wrapping_add(2)) as u32;
        let dist = ahead_along(env.mem(), cur, f.wrapping_add(4), fd);
        if dist as i32 >= 0x4_0000 && fd <= 3 {
            let fd = env.r16(f.wrapping_add(2)) as u32;
            match fd {
                0 => env.w32(f.wrapping_add(0xC), c[2].wrapping_add(0x4_0000)),
                1 => env.w32(f.wrapping_add(0xC), c[2].wrapping_sub(0x4_0000)),
                2 => env.w32(f.wrapping_add(4), c[0].wrapping_add(0x4_0000)),
                _ => env.w32(f.wrapping_add(4), c[0].wrapping_sub(0x4_0000)),
            }
        }
        let s = follow_speed(dist as i32);
        if s < 0x28 {
            env.w16(f, 1);
        } else {
            env.w16(f, 0);
            for o in [0x20, 0x24, 0x28] {
                env.w32(f.wrapping_add(o), s as u32);
            }
        }
        vec_set(env, f.wrapping_add(0x10), c);
        env.mem().pop_local(cur);
    }
    env.callv(SURF_DRAW, &[task, w]);
}

/// C: `FUN_021ACFF8` (unlabeled routine at `0x021ACFF8`): update proc of the rail surf task (no draw call; mode 0:
/// nothing). `+0xC` = the MMDL display dir; the rail direction vectors of dir 0 (`a`) and of the display dir (`b`)
/// (`FUN_0219E784`, VecFx16), each with y = 0 normalised in place (`FUN_0207CDB8`); mode 1: bob and draw offset
/// `(-a.x * 0x7000, bob + 0x5000, -a.z * 0x7000)` (64-bit products, `(p + 0x800) >> 12`); task position = draw
/// position - offset, `y += bob - 0x1000` (also in mode 2, whose offset stays 0). Ridden / hidden: no follower. Else
/// the follower tracks the MMDL position: a new dir copies it; the same position steps `b * 0x4000` (x and z, y =
/// the MMDL y), clamped at the MMDL on the side `b` points to; distance (`FUN_0207CF3C`) >= 0x40000 puts it
/// `b * 0x40000` behind; speed [`follow_speed`] <= 40: still, else moving with scale = speed; the MMDL position is
/// remembered.
pub fn surf_update_rail<E: FieldEnv>(env: &mut E, task: u32, w: u32) {
    if env.r8(w.wrapping_add(9)) == 0 {
        return;
    }
    let m = env.r32(w.wrapping_add(0x20));
    let d = c1(env, MMDL_DISP_DIR, m);
    env.w16(w.wrapping_add(0xC), d as u16);
    // the original's sp+0 (b) and sp+6 (a), VecFx16 each
    let vb = env.mem().push_local(0xC);
    let va = vb + 6;
    env.call(
        RAIL_DIR_VEC,
        &mut [CArg::V(m), CArg::V(0), CArg::Out(va, 6)],
    );
    let dd = env.r16(w.wrapping_add(0xC)) as u32;
    env.call(
        RAIL_DIR_VEC,
        &mut [CArg::V(m), CArg::V(dd), CArg::Out(vb, 6)],
    );
    env.w16(va + 2, 0);
    env.call(VEC16_NORMALIZE, &mut [CArg::In(va, 6), CArg::Out(va, 6)]);
    env.w16(vb + 2, 0);
    env.call(VEC16_NORMALIZE, &mut [CArg::In(vb, 6), CArg::Out(vb, 6)]);
    let mut off = [0u32; 3];
    if env.r8(w.wrapping_add(9)) == 1 {
        bob(env, w);
        let ax = env.r16(va) as i16 as i32;
        off[0] = fx64((ax.wrapping_neg() as i64).wrapping_mul(0x7000));
        off[1] = env.r32(w.wrapping_add(0x10)).wrapping_add(0x5000);
        let az = env.r16(va + 4) as i16 as i32;
        off[2] = fx64((az.wrapping_neg() as i64).wrapping_mul(0x7000));
        let mut b = [0u8; 12];
        for i in 0..3 {
            b[4 * i..4 * i + 4].copy_from_slice(&off[i].to_le_bytes());
        }
        env.call(MMDL_DRAW_OFS_SET, &mut [CArg::V(m), CArg::InBuf(&b)]);
    }
    place_task(env, task, w, m, off, true);
    if !follower_off(env, w) {
        let f = w.wrapping_add(0x38);
        let cur = follower_head(env, w, m);
        let c = vec_get(env, cur);
        let dd = env.r16(w.wrapping_add(0xC)) as u32;
        let bx = env.r16(vb) as i16 as i64;
        let bz = env.r16(vb + 4) as i16 as i64;
        if dd != env.r16(f.wrapping_add(2)) as u32 {
            env.w16(f.wrapping_add(2), dd as u16);
            vec_set(env, f.wrapping_add(4), c);
        } else if c == vec_get(env, f.wrapping_add(0x10)) {
            let x = env.r32(f.wrapping_add(4)).wrapping_add(fx64(bx << 14));
            env.w32(f.wrapping_add(4), x);
            let z = env.r32(f.wrapping_add(0xC)).wrapping_add(fx64(bz << 14));
            env.w32(f.wrapping_add(0xC), z);
            env.w32(f.wrapping_add(8), c[1]);
            if (bx > 0 && x as i32 > c[0] as i32) || (bx < 0 && (x as i32) < c[0] as i32) {
                env.w32(f.wrapping_add(4), c[0]);
            }
            let z = env.r32(f.wrapping_add(0xC));
            if (bz > 0 && z as i32 > c[2] as i32) || (bz < 0 && (z as i32) < c[2] as i32) {
                env.w32(f.wrapping_add(0xC), c[2]);
            }
        }
        let dist = env.call(
            VEC_DISTANCE,
            &mut [CArg::In(cur, 12), CArg::In(f.wrapping_add(4), 12)],
        );
        if dist as i32 >= 0x4_0000 {
            env.w32(f.wrapping_add(4), c[0].wrapping_sub(fx64(bx << 18)));
            env.w32(f.wrapping_add(0xC), c[2].wrapping_sub(fx64(bz << 18)));
        }
        let s = follow_speed(dist as i32);
        if s <= 0x28 {
            env.w16(f, 1);
        } else {
            env.w16(f, 0);
            for o in [0x20, 0x24, 0x28] {
                env.w32(f.wrapping_add(o), s as u32);
            }
        }
        vec_set(env, f.wrapping_add(0x10), c);
        env.mem().pop_local(cur);
    }
    env.mem().pop_local(vb);
}

/// C: `FUN_021AD3D4` (unlabeled routine at `0x021AD3D4`): allocate the splash RESOURCE block (0x10,
/// `Heap_AllocDebug(heap, 0x10, 1, "fldeff_namipoke.c", 0x3CF)`), `+0` = the control, load its models
/// (`FUN_021AD410`, render); returns the block.
pub fn splash_res_create<E: FieldEnv>(env: &mut E, ctrl: u32, heap: u32) -> u32 {
    let r = env.callv(HEAP_ALLOC, &[heap, 0x10, 1, FILE_NAMIPOKE, 0x3CF]);
    env.w32(r, ctrl);
    c1(env, SPLASH_MODEL_LOAD, r);
    r
}

/// C: `FUN_021AD3FC` (unlabeled routine at `0x021AD3FC`): free the splash models (`FUN_021AD530`, render) and the
/// RESOURCE block.
pub fn splash_res_delete<E: FieldEnv>(env: &mut E, _ctrl: u32, res: u32) {
    c1(env, SPLASH_MODEL_FREE, res);
    c1(env, HEAP_FREE, res);
}

fn add_splash<E: FieldEnv>(env: &mut E, ctrl: u32, init: [u32; 4], x: u32) -> u32 {
    let mut b = [0u8; 16];
    for i in 0..4 {
        b[4 * i..4 * i + 4].copy_from_slice(&init[i].to_le_bytes());
    }
    env.call(
        FX_ADD_TASK,
        &mut [
            CArg::V(ctrl),
            CArg::V(TABLE_SPLASH),
            CArg::V(x),
            CArg::V(0),
            CArg::InBuf(&b),
            CArg::V(1),
        ],
    )
}

/// C: `FUN_021AD564` (unlabeled routine at `0x021AD564`): add a splash task following `parent`: init
/// `{0, FUN_021ABEA0(ctrl, 9), kind, parent}`, `FUN_021ABFA8(ctrl, ROM 0x021D9170, 0, 0, &init, 1)`; returns the task.
pub fn add_splash_follow<E: FieldEnv>(env: &mut E, ctrl: u32, kind: u32, parent: u32) -> u32 {
    let res = env.callv(FX_WORK_OF, &[ctrl, 9]);
    add_splash(env, ctrl, [0, res, kind, parent], 0)
}

/// C: `FUN_021AD598` (unlabeled routine at `0x021AD598`): add a one-shot splash task: init
/// `{1, FUN_021ABEA0(ctrl, 9), kind, 0}`, `FUN_021ABFA8(ctrl, ROM 0x021D9170, x, 0, &init, 1)`; returns the task.
pub fn add_splash_once<E: FieldEnv>(env: &mut E, ctrl: u32, kind: u32, x: u32) -> u32 {
    let res = env.callv(FX_WORK_OF, &[ctrl, 9]);
    add_splash(env, ctrl, [1, res, kind, 0], x)
}

/// C: `FUN_021AD5CC` (unlabeled routine at `0x021AD5CC`): the done word (`+0xC`) of a splash task's work; 1 for a
/// null task.
pub fn splash_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    if task == 0 {
        return 1;
    }
    let w = c1(env, TASK_WORK, task);
    env.r32(w.wrapping_add(0xC))
}

/// C: `FUN_021AD5E0` (unlabeled routine at `0x021AD5E0`): init proc of the splash task: `w +0x14..` = the init data
/// (0x10 bytes); `FUN_021ABE18(RESOURCE +0)`; kind `w +0x1C` 0..4 picks the model handle RESOURCE `+4` / `+6` / `+0xA`
/// / `+8` / `+0xC` -> `w +0x10` = `FUN_021C6590(sys, handle, 0, 0)`; with a parent `w +0x20`: offset `w +0..` =
/// [`SPLASH_OFFSETS`]`[kind]`, task position = parent position (`FUN_021AC1B8`) + offset. UNVERIFIED: a kind > 4 with
/// a parent panics (the original reads its stack past the copied table).
pub fn splash_init<E: FieldEnv>(env: &mut E, task: u32, w: u32) {
    let i = c1(env, TASK_INIT, task);
    env.mem().copy(w.wrapping_add(0x14), i, 0x10);
    let res = env.r32(w.wrapping_add(0x18));
    let r0 = env.r32(res);
    let sys = c1(env, FX_RES_SYS, r0);
    let kind = env.r32(w.wrapping_add(0x1C));
    if kind <= 4 {
        let o = [4, 6, 0xA, 8, 0xC][kind as usize];
        let res = env.r32(w.wrapping_add(0x18));
        let h = env.r16(res.wrapping_add(o)) as u32;
        let a = env.callv(ANIM_CREATE, &[sys, h, 0, 0]);
        env.w16(w.wrapping_add(0x10), a as u16);
    }
    let parent = env.r32(w.wrapping_add(0x20));
    if parent != 0 {
        let kind = env.r32(w.wrapping_add(0x1C));
        let off = SPLASH_OFFSETS[kind as usize].map(|v| v as u32);
        vec_set(env, w, off);
        splash_follow(env, task, w, parent);
    }
}

/// Task position = parent position (`FUN_021AC1B8`) + the offset at `w +0..`.
fn splash_follow<E: FieldEnv>(env: &mut E, task: u32, w: u32, parent: u32) {
    let mut p = [0u8; 12];
    env.call(TASK_POS_GET, &mut [CArg::V(parent), CArg::OutBuf(&mut p)]);
    for i in 0..3 {
        let v = u32::from_le_bytes(p[4 * i..4 * i + 4].try_into().unwrap())
            .wrapping_add(env.r32(w.wrapping_add(4 * i as u32)));
        p[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
    }
    env.call(TASK_POS_SET, &mut [CArg::V(task), CArg::InBuf(&p)]);
}

/// C: `FUN_021AD6B8` (unlabeled routine at `0x021AD6B8`): update proc of the splash task: done (`+0xC` == 1) and
/// delete-when-done (`+0x14` == 1): delete the task (`FUN_021AC154`). Else `FUN_021ABE18(RESOURCE +0)`; kind 0: not
/// done and the animation step `FUN_021C6694(sys, +0x10, 0x1000)` answers 0: done = 1; other kinds: loop step
/// `FUN_021C66EC`; with a parent: follow its position + the offset.
pub fn splash_update<E: FieldEnv>(env: &mut E, task: u32, w: u32) {
    if env.r32(w.wrapping_add(0xC)) == 1 && env.r32(w.wrapping_add(0x14)) == 1 {
        c1(env, TASK_DELETE, task);
        return;
    }
    let res = env.r32(w.wrapping_add(0x18));
    let r0 = env.r32(res);
    let sys = c1(env, FX_RES_SYS, r0);
    if env.r32(w.wrapping_add(0x1C)) == 0 {
        if env.r32(w.wrapping_add(0xC)) == 0 {
            let h = env.r16(w.wrapping_add(0x10)) as u32;
            if env.callv(ANIM_STEP_DONE, &[sys, h, 0x1000]) == 0 {
                env.w32(w.wrapping_add(0xC), 1);
            }
        }
    } else {
        let h = env.r16(w.wrapping_add(0x10)) as u32;
        env.callv(ANIM_LOOP, &[sys, h, 0x1000]);
    }
    let parent = env.r32(w.wrapping_add(0x20));
    if parent != 0 {
        splash_follow(env, task, w, parent);
    }
}

#[cfg(test)]
mod tests;
