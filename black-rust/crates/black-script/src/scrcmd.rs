//! F8 (planned): script command handlers, split into jobs by area.
//!
//! The command table is at 0x0217062C (overlay 10 data, 609 entries = the count word at 0x021705D8, index = command id; F7-spec).
//! `tools/re/F1-SPLIT.md` counted a 624-slot run from 0x021705F0, whose first 15 words are the separate init-proc table of
//! `0x021591B4`: its "slot N" is command N - 15. Handlers are `fn(vm, ctx) -> 0 (next) / 1 (yield)`; see the `script_sys` and
//! `vm` spec headers. Handler areas and counts: see the F8 list in `tools/re/F1-SPLIT.md` (`scrcmd.c` bodies
//! 0x02159204-0x0215E2AA, `scrcmd_*.c` 0x021B28E0-0x021B73F6, unanchored clusters at 0x021AE948, 0x021B0D14, 0x0216A420,
//! 0x021CF7DC, 0x021D1670). The scrcmd ctx getters 0x0215A5CC-0x0215A5FA and commands 49, 50, 311 are ported in `script_sys`.
//! The VM reaches the handlers through `hooks::CommandDispatch` (handler address -> port); UI requests go through
//! `host::ScriptHost`. Empty until F8 starts.
