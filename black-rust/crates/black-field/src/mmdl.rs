//! F4 (planned): map-model objects (NPCs, the player object): core, movement types, animation commands, trainer sight.
//!
//! Original areas (see `tools/re/F1-SPLIT.md`): `fldmmdl.c` 0x0216CC58-0x0216E954 (overlay 10, 246 fns), `fldmmdl_acmd.c`
//! 0x02197D3C-0x0219C552 (overlay 21, 452 fns, movement-type tables at 0x021D4160..), `event_trainer_eye.c` 0x021AE454.
//! Billboard/3D drawing (`fldmmdl_blact.c`, `fldmmdl_g3dobj.c`) is U2. Empty until F4a starts.
