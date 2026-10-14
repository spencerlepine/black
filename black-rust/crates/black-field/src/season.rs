//! F11: calendar, weather and the time-of-day light. The ports live in one area per original file:
//! [`crate::calender`] (`calender.c` 0x02164778), [`crate::weather`] (`weather.c` 0x021A19E8),
//! [`crate::weather_task`] (`weather_task.c` 0x021A1DB4) and [`crate::field_light`] (`field_light.c` 0x021A03D0).
//! The season and the time zones themselves are ARM9 main code (`FUN_020113D8`, the hour table `FUN_02011400`) that
//! these areas reach through [`crate::hooks::FieldEnv`].
