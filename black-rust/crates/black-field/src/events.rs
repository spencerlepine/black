//! F6 (planned): zone event data (NPC/background/warp/coordinate events) and map-change events.
//!
//! Original areas (see `tools/re/F1-SPLIT.md`): F1 row 10.23 `eventdata_system.c` 0x02161D98-0x02164768 (overlay 10,
//! specced by F6a-spec as the areas [`crate::wfbc_data`], [`crate::eventdata`], [`crate::mmdl_move`] and
//! [`crate::eventdata_tail`]; ported by F6a), `event_mapchange.c` 0x02183B60-0x02186278 (overlay 20, 92 fns; F6b).
