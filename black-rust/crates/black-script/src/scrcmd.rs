//! F8 (planned): script command handlers, split into jobs by area.
//!
//! The command table is at 0x021705F0 (overlay 10 data, 624 slots, 614 non-null). Handler areas and counts: see the F8 list in
//! `tools/re/F1-SPLIT.md` (`scrcmd.c` bodies 0x02159204-0x0215E2AA, `scrcmd_*.c` 0x021B28E0-0x021B73F6, unanchored clusters at
//! 0x021AE948, 0x021B0D14, 0x0216A420, 0x021CF7DC, 0x021D1670). Empty until F8 starts.
