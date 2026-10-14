//! Field logic: map matrix, collision, player and NPC movement (fldmmdl), encounters, warps/events.
//!
//! Logic crate (D6): no Bevy dependency.

pub mod acmd1;
pub mod acmd2a;
pub mod acmd2b;
pub mod acmd2c;
pub mod acmd2d;
pub mod acmd2e;
pub mod btl_setup;
pub mod effect_encount;
pub mod enc_pokeset;
pub mod encount;
pub mod encount_st;
pub mod encounter;
pub mod event_battle;
pub mod event_battle_head;
pub mod event_work;
pub mod eventdata;
pub mod eventdata_tail;
pub mod events;
pub mod field_skill;
pub mod fieldmap;
pub mod fieldmap_event;
pub mod fieldmap_func;
pub mod fldmmdl;
pub mod game_data;
pub mod height_ex;
pub mod hooks;
pub mod land_data_patch;
pub mod map_attr;
pub mod map_matrix;
pub mod mmdl_draw;
pub mod mmdl_move;
pub mod move_pokemon;
pub mod mvproc3a;
pub mod mvproc3b;
pub mod mvproc3c;
pub mod mvproc3d;
pub mod namipoke;
pub mod ov63_event;
pub mod player;
pub mod player_core;
pub mod player_event;
pub mod player_grid;
pub mod season;
pub mod symbol_map;
#[cfg(test)]
mod testenv;
pub mod trainer_eye;
pub mod wfbc_data;
pub mod zonedata;
