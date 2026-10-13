//! Field logic: map matrix, collision, player and NPC movement (fldmmdl), encounters, warps/events.
//!
//! Logic crate (D6): no Bevy dependency.

pub mod btl_setup;
pub mod encounter;
pub mod event_battle_head;
pub mod event_work;
pub mod eventdata;
pub mod eventdata_tail;
pub mod events;
pub mod fieldmap;
pub mod fieldmap_event;
pub mod fieldmap_func;
pub mod fldmmdl;
pub mod game_data;
pub mod hooks;
pub mod land_data_patch;
pub mod map_matrix;
pub mod mmdl_move;
pub mod player;
pub mod season;
#[cfg(test)]
mod testenv;
pub mod wfbc_data;
pub mod zonedata;
