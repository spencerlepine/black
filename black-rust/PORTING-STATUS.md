# Porting status

Read this first in every session. Source: `../black-rom` (read-only). Checks (all must pass):
`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo build --workspace --release`.

## Summary

| crate | covers | functions | verified |
|---|---|---|---|
| `black-core` | `src/battle_record.c`, `src/unk_02008574.c` (+ asm `BattleRecordBuilder_Load`) | 23 (+2 SDK shims) | differential vs host-compiled C |
| `black-battle` | battle_calc.s 0x021D74F8-0x021D80BC, battle_view_core.s 0x021E8940-0x021E8DA8 | 68 | 64 vs original machine code, 4 unit-tested |
| `nds-rom`, `black-cli` | ROM tooling (not game logic) | n/a | real-ROM tests |

## Verification method

- **Decompiled C** (`src/*.c`): compile the C on the host with shims, then compare fuzz hashes
  (`black-core`).
- **Assembly only**: `tools/rom-diff` runs the original ARM9 + overlay 93 code from the decomp
  build under Unicorn, with Python stubs for undecompiled callees. Each `*_matches_rom_fuzz`
  test hashes the same generated inputs and must give the same FNV hash. See
  `tools/rom-diff/README.md`.
- rizin is for cross-checks only. Its ESIL emulation hung on Thumb/ARM interworking.
- The decomp build products (`main.sbin`, `OVY_93.sbin`) match the retail ROM. Overlay 93
  loads at 0x021B6100 and ARM9 at 0x02004000.

## Asm reading notes

- `thumb_func_end` markers often sit at the first branch. A function runs to the next
  `func_start`.
- Thumb `add rX, pc` (hi-register form) reads PC as instruction + 4, **unaligned**. Jump tables
  `add pc, r0` branch to instruction + 4 + offset.
- Some code is stored as `.byte` blocks (e.g. `0x021D778C`, `0x021D7D78`, and the big gaps in
  battle_calc.s after 0x021D85C0). Disassemble them with rizin on `OVY_93.sbin`, or hand-decode.
- Runtime helpers: `FUN_0209C2B0` = `_u32_div_f` (returns dividend on /0, remainder in r1),
  `FUN_0209C0A4` = `_s32_div_f`, `FUN_0209C054` = `_ll_mul`.
- Debug filename strings in each file's `.data` identify the original source file (table
  below). The ARM9 table lives in `unk_0209CE10.s`.

## Open questions / unverified

- `_s32_div_f` division by zero (unused by ported callers).
- `FUN_02034714` (called when the event-var stack overflows): presumably a diagnostic or assert.
  Modelled as `Err(StackOverflow)`.
- Inferred names, not semantics-verified: `bpp::{MAX_HP=0x0E, LEVEL=0x0F, FORM=0x13}`, weather
  IDs (1 sun, 2 rain, 3 hail, 4 sand), condition IDs (1 par .. 7 attract), personal param 9 =
  base EXP yield, prize-money table by badge count.
- Unknown meaning: `calc::{is_not_0_or_3, fun_021d7f00, fun_021d7f2c, fun_021d7f38/4c/60,
  fun_021d80a8}`, `BattleCalc::unk_00`, `calc` work buffer (0x460 bytes) contents.
- battle_calc.s not yet ported in the ported range: `021d7e10` (ARM overlay call at 0x0689C9E4),
  `021d7f88`/`021d7fbc` (ARM9 item/save calls), `021d805c`, `021d8058` (empty), and the
  unreferenced duplicates at `0x021D7D78`.
- 283 files have no subsystem yet (`?` below): 153 ARM9 `unk_*.s` files and overlays without
  debug strings.

## Next batch

1. **battle_calc.s from 0x021D80BC** (110 labelled functions plus `.byte` code blocks): the
   rest of `btl_calc.c`, i.e. the damage-flow and handler helpers that consume
   `EventVarStack`. Add their callees in `battle_pokeparam.s` to the hooks first.
2. **battle_pokeparam.s** (176 fns): `BattleMon_Get`, stat stages (`021d58a8` + `apply_stat_stage`),
   `HasType` (`021d5814`). This turns the `BattleMon` trait into a real struct.
3. **battle_field.s** (34 fns, `btl_field.c`): weather/field state.
4. Map ARM9 `unk_*.s` files to `poke_tool.c`, `poke_personal.c`, `shinka_check.c`,
   `waza_tool.c` (P2), `pokeparty.c`, `box_savedata.c` (P3), and `myitem_savedata.c` (P4) by
   following callers of known functions (e.g. `FUN_02019C90` personal getter, `FUN_0202A3FC`).
5. Then `battle_ungrouped.s` (`btl_server_flow.c`, 944 fns) for turn order and move execution.

## Files

Subsystem and priority come from embedded debug filenames where present. `fns` = number of
labelled functions in the `.s` (excludes `.byte`-encoded code).

| source file | subsystem | priority | status | Rust crate::module | notes |
|---|---|---|---|---|---|
| `asm/battle_adapter.s` | battle core | P1 | todo |  | 42 fns; btl_adapter.c: server/client messaging |
| `asm/battle_calc.s` | battle core | P1 | partial | `black-battle::{calc,event_var}` | 169 fns; btl_calc.c; 0x021D74F8-0x021D80BC ported (52 fns); 0x021D80BC onward todo |
| `asm/battle_client.s` | battle core | P1 | todo |  | 230 fns; btl_client.c: action selection |
| `asm/battle_field.s` | battle core | P1 | todo |  | 34 fns; btl_field.c: field effects |
| `asm/battle_pokeparam.s` | battle core | P1 | todo |  | 176 fns; btl_pokeparam.c: BTL_POKEPARAM (BattleMon_Get etc.); used via black-battle::hooks traits |
| `asm/battle_record.s` | battle core | P1 | partial | `black-core::battle_record` | 9 fns; asm-only BattleRecordBuilder_Load ported |
| `asm/battle_server.s` | battle core | P1 | todo |  | 209 fns; btl_server.c |
| `asm/battle_unattributed.s` | battle core | P1 | todo |  | 0 fns; data only |
| `asm/battle_ungrouped.s` | battle core | P1 | todo |  | 944 fns; btl_server_flow.c: turn flow, move execution, event dispatch |
| `asm/overlay_096_02187F00.s` | battle AI | P1 | todo |  | 75 fns; tr_ai.c |
| `src/battle_record.c` | battle core | P1 | verified | `black-core::battle_record` | decompiled C; differential fuzz vs host-compiled C |
| `asm/overlay_170_021DC860.s` | PC box storage UI | P3 | todo |  | 662 fns; box2_main.c box2_seq.c box2_ui.c |
| `asm/overlay_070_021B6100.s` | items/bag UI | P4 | todo |  | 148 fns; itemmenu.c |
| `asm/overlay_197_0220B120.s` | items/bag UI | P4 | todo |  | 83 fns; b_bag_main.c |
| `asm/overlay_010_02155160.s` | field scripts/events | P6 | todo |  | 1260 fns; script_sys.c, script_work.c, scrcmd_*.c, eventdata_system.c, fldmmdl.c, map_matrix.c, move_pokemon.c, waza_oshie.c |
| `asm/overlay_020_02183B60.s` | field / events | P6 | todo |  | 157 fns; el_scoreboard.c event_mapchange.c field_wfbc_data_local.c |
| `asm/overlay_021_02187F00.s` | field core | P6 | todo |  | 3575 fns; fieldmap.c, field_player*.c (movement), field_encount.c/effect_encount.c (P5 wild encounters), scrcmd_*.c, weather.c, sodateya.c |
| `asm/overlay_048_021F36C0.s` | field / events | P6 | todo |  | 65 fns; field_gimmick_gate.c gimmick_obj_elboard.c |
| `asm/overlay_051_021F36C0.s` | field / events | P6 | todo |  | 22 fns; field_gimmick_d06.c |
| `asm/overlay_052_021F36C0.s` | field / events | P6 | todo |  | 6 fns; field_gimmick_bsubway.c |
| `asm/overlay_056_021F36C0.s` | field / events | P6 | todo |  | 22 fns; field_fk_sound_anime.c |
| `asm/overlay_059_021F36C0.s` | field / events | P6 | todo |  | 1 fns; sp_poke_gimmick.c |
| `asm/overlay_063_021F36C0.s` | field / events | P6 | todo |  | 2 fns; rail_slipdown.c |
| `asm/overlay_082_021F5260.s` | field / events | P6 | todo |  | 19 fns; fld_faceup.c |
| `asm/overlay_092_021B95A0.s` | field / events | P6 | todo |  | 10 fns; event_battle_return.c |
| `asm/arm9/arm9_autoload0_01FF8000.s` | ARM9 autoload (ITCM/DTCM) | P7 | todo |  | 1 fns |
| `asm/arm9/arm9_autoload1_02FE0000.s` | ARM9 autoload (ITCM/DTCM) | P7 | todo |  | 0 fns |
| `asm/arm9/arm9_autoload2_02400000.s` | ARM9 autoload (ITCM/DTCM) | P7 | todo |  | 0 fns |
| `asm/arm9/arm9_autoload3_06898000.s` | ARM9 autoload (ITCM/DTCM) | P7 | todo |  | 0 fns |
| `asm/battle_net.s` | networking | P7 | todo |  | 63 fns; btl_net.c |
| `asm/battle_view_core.s` | battle UI | P7 | partial | `black-battle::{sick_cont,poke_type}` | 78 fns; btlv_core.c; 0x021E8940-0x021E8DA8 logic helpers (16 fns) ported, verified; rest is UI |
| `asm/battle_view_scu.s` | battle UI | P7 | todo |  | 313 fns; btlv_scu.c |
| `asm/crt0.s` | SDK/OS | P7 | todo |  | 13 fns |
| `asm/overlay_001_021542C0.s` | networking / C-Gear | P7 | todo |  | 1 fns; event_gtsnego.c |
| `asm/overlay_002_021542C0.s` | networking / C-Gear | P7 | todo |  | 3 fns; event_wifibattlematch.c |
| `asm/overlay_006_021542C0.s` | networking / C-Gear | P7 | todo |  | 7 fns; event_wificlub.c |
| `asm/overlay_009_02155160.s` | networking / C-Gear | P7 | todo |  | 1591 fns; dwc_rap.c dwc_rapcommon.c net_devwifi.c |
| `asm/overlay_011_02172BC0.s` | networking / C-Gear | P7 | todo |  | 108 fns; intrude_comm.c |
| `asm/overlay_012_02172BC0.s` | networking / C-Gear | P7 | todo |  | 48 fns; colosseum.c union_comm.c union_main.c |
| `asm/overlay_014_02177020.s` | networking / C-Gear | P7 | todo |  | 135 fns; net_devwireless.c net_whpipe.c wih.c |
| `asm/overlay_016_021B6100.s` | networking / C-Gear | P7 | todo |  | 996 fns; dwc_raputil.c |
| `asm/overlay_018_0217AE20.s` | networking / C-Gear | P7 | todo |  | 199 fns; event_comm_common.c intrude_field.c intrude_subdisp.c |
| `asm/overlay_019_0217AE20.s` | networking / C-Gear | P7 | todo |  | 95 fns; union_eff.c union_msg.c union_oneself.c |
| `asm/overlay_022_02183B60.s` | networking / C-Gear | P7 | todo |  | 18 fns; union_app.c |
| `asm/overlay_023_021841E0.s` | networking / C-Gear | P7 | todo |  | 80 fns; oekaki.c |
| `asm/overlay_029_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 127 fns; c_gear.c |
| `asm/overlay_030_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 9 fns; no_gear.c |
| `asm/overlay_031_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 34 fns; union_subdisp.c |
| `asm/overlay_032_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 146 fns; beacon_view.c |
| `asm/overlay_033_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 23 fns; dowsing.c |
| `asm/overlay_034_021EFEC0.s` | UI / app | P7 | todo |  | 9 fns; report.c |
| `asm/overlay_035_021EFEC0.s` | networking / C-Gear | P7 | todo |  | 25 fns; c_gear_power_onoff.c |
| `asm/overlay_067_021B6100.s` | UI / app | P7 | todo |  | 94 fns; frame_list.c msgsearch.c touchbar.c ui_scene.c yesno_menu.c |
| `asm/overlay_071_021B6100.s` | UI / app | P7 | todo |  | 194 fns; monolith_tool.c |
| `asm/overlay_072_021B95A0.s` | UI / app | P7 | todo |  | 109 fns; townmap.c townmap_grh.c |
| `asm/overlay_088_021B95A0.s` | UI / app | P7 | todo |  | 146 fns; save_control_intr.c startmenu.c |
| `asm/overlay_091_021B6240.s` | party/summary UI | P7 | todo |  | 204 fns; plist_battle.c plist_demo.c plist_menu.c plist_message.c plist_plate.c pokelist.c |
| `asm/overlay_094_021F6560.s` | battle UI | P7 | todo |  | 471 fns; btlv_b_gauge.c btlv_bg.c btlv_camera.c btlv_clact.c btlv_effect.c btlv_effvm.c btlv_field.c btlv_finger_cursor.c |
| `asm/overlay_095_06898020.s` | battle UI | P7 | todo |  | 269 fns; btlv_scd.c |
| `asm/overlay_097_021F6560.s` | battle UI | P7 | todo |  | 38 fns; vs_multi_list.c |
| `asm/overlay_098_021B6100.s` | networking / C-Gear | P7 | todo |  | 43 fns; pdc.c pdc_main.c pdc_msg.c |
| `asm/overlay_100_021B95A0.s` | networking / C-Gear | P7 | todo |  | 89 fns; ircbattlematch.c |
| `asm/overlay_101_021B6100.s` | networking / C-Gear | P7 | todo |  | 65 fns; compatible_irc_sys.c irc_appbar.c irc_compatible.c |
| `asm/overlay_106_021B7560.s` | networking / C-Gear | P7 | todo |  | 69 fns; irc_ranking.c |
| `asm/overlay_107_021EE740.s` | networking / C-Gear | P7 | todo |  | 216 fns; mb_comm_sys.c mb_parent_sys.c mb_util_msg.c mbp.c |
| `asm/overlay_110_021B95A0.s` | UI / app | P7 | todo |  | 397 fns; pms_input.c pms_input_data.c pms_input_view.c pms_select_graphic.c pmsi_search.c pmsiv_category.c pmsiv_edit.c pmsiv_menu.c |
| `asm/overlay_112_021F36C0.s` | UI / app | P7 | todo |  | 151 fns; mus_item_data.c mus_item_draw.c mus_poke_draw.c mus_shot_info.c mus_shot_photo.c musical_mcss.c musical_shot_sys.c musical_system.c |
| `asm/overlay_114_021B95A0.s` | networking / C-Gear | P7 | todo |  | 445 fns; ghttpEncryption.c nhttp_rap.c nhttp_rap_evilcheck.c |
| `asm/overlay_115_021D4240.s` | networking / C-Gear | P7 | todo |  | 108 fns; wifilogin_disp.c wifilogin_message.c |
| `asm/overlay_116_021D6660.s` | networking / C-Gear | P7 | todo |  | 90 fns; wifi_bsubway_graphic.c |
| `asm/overlay_117_021D4240.s` | networking / C-Gear | P7 | todo |  | 1 fns; pokemontrade_demo_common.c pokemontrade_ircdemo.c |
| `asm/overlay_118_021D4240.s` | networking / C-Gear | P7 | todo |  | 43 fns; pokemontrade_demo_common.c pokemontrade_demo_down.c pokemontrade_demo_middle.c pokemontrade_demo_up.c |
| `asm/overlay_119_021D8980.s` | networking / C-Gear | P7 | todo |  | 523 fns; pokemontrade_2d.c pokemontrade_3d.c pokemontrade_mcss.c pokemontrade_nego.c pokemontrade_proc.c pokemontrade_save.c |
| `asm/overlay_120_021D4240.s` | networking / C-Gear | P7 | todo |  | 180 fns; gtsnego_disp.c gtsnego_message.c |
| `asm/overlay_121_021D6660.s` | networking / C-Gear | P7 | todo |  | 269 fns; mystery.c mystery_album.c mystery_graphic.c mystery_net.c mystery_util.c |
| `asm/overlay_123_021D4240.s` | networking / C-Gear | P7 | todo |  | 277 fns; gsync_disp.c gsync_download.c gsync_menu.c gsync_message.c gsync_state.c pdwacc_disp.c pdwacc_message.c |
| `asm/overlay_124_021D4240.s` | UI / app | P7 | todo |  | 18 fns; cg_help.c |
| `asm/overlay_125_021D6660.s` | networking / C-Gear | P7 | todo |  | 53 fns; gds_rap.c |
| `asm/overlay_127_021C3EE0.s` | networking / C-Gear | P7 | todo |  | 106 fns; wf2dmap_cmdq.c wf2dmap_map.c wf2dmap_obj.c wf2dmap_objdraw.c wf2dmap_scrdraw.c wifi_2dchar.c |
| `asm/overlay_131_021D4240.s` | party/summary UI | P7 | todo |  | 163 fns; p_sta_info.c p_sta_oam.c p_sta_ribbon.c p_sta_skill.c p_sta_sub.c p_status.c |
| `asm/overlay_132_021F8480.s` | networking / C-Gear | P7 | todo |  | 159 fns; mus_comm_func.c script_command.c sta_act_audience.c sta_act_button.c sta_act_effect.c sta_act_script.c sta_acting.c |
| `asm/overlay_133_021B6100.s` | UI / app | P7 | todo |  | 88 fns; dup_fitting.c dup_fitting_item.c |
| `asm/overlay_135_021F4580.s` | networking / C-Gear | P7 | todo |  | 552 fns; worldtrade.c worldtrade_adapter.c worldtrade_box.c worldtrade_demo.c worldtrade_deposit.c worldtrade_enter.c worldtrade_input.c worldtrade_search.c |
| `asm/overlay_136_021C3CC0.s` | networking / C-Gear | P7 | todo |  | 148 fns; mailsys.c mailtool.c mailview.c mb_seq.c |
| `asm/overlay_137_021DC860.s` | networking / C-Gear | P7 | todo |  | 256 fns; guru2.c guru2_comm.c guru2_main.c |
| `asm/overlay_168_02183B60.s` | UI / app | P7 | todo |  | 47 fns; mictest.c |
| `asm/overlay_172_021B6100.s` | networking / C-Gear | P7 | todo |  | 356 fns; camera_system.c comm_tvt_sys.c ctvt_call.c ctvt_camera.c ctvt_comm.c ctvt_draw.c ctvt_mic.c ctvt_talk.c |
| `asm/overlay_174_021B6100.s` | networking / C-Gear | P7 | todo |  | 39 fns; local_tvt_chara.c local_tvt_sys.c |
| `asm/overlay_175_021D4240.s` | networking / C-Gear | P7 | todo |  | 403 fns; wifibattlematch_net.c |
| `asm/overlay_176_0217AE20.s` | networking / C-Gear | P7 | todo |  | 180 fns; battle_championship_flow.c digitalcardcheck.c livebattlematch_flow.c livebattlematch_irc.c |
| `asm/overlay_177_021E5440.s` | networking / C-Gear | P7 | todo |  | 56 fns; wifibattlematch_graphic.c wifibattlematch_util.c wifibattlematch_view.c |
| `asm/overlay_179_021B6100.s` | UI / app | P7 | todo |  | 22 fns; cdemo_main.c |
| `asm/overlay_182_021B95A0.s` | UI / app | P7 | todo |  | 18 fns; the_end_graphic.c |
| `asm/overlay_183_021DB2E0.s` | networking / C-Gear | P7 | todo |  | 221 fns; br_codein.c |
| `asm/overlay_186_021F90C0.s` | networking / C-Gear | P7 | todo |  | 282 fns; br_btn.c br_btn_data.c br_fade.c br_graphic.c br_net.c br_pokesearch.c br_proc_sys.c br_res.c |
| `asm/overlay_187_0220AF40.s` | networking / C-Gear | P7 | todo |  | 13 fns; br_main.c |
| `asm/overlay_188_021EE920.s` | UI / app | P7 | todo |  | 6 fns; battle_rec_tool.c |
| `asm/overlay_194_021F1440.s` | UI / app | P7 | todo |  | 166 fns; namein_data.c namein_graphic.c nsbtx_to_clwk.c prof_word.c |
| `asm/overlay_195_02203FA0.s` | evolution/egg UI | P7 | todo |  | 147 fns; shinka_demo.c shinka_demo_effect.c shinka_demo_graphic.c shinka_demo_view.c |
| `asm/overlay_196_0220AF40.s` | battle UI | P7 | todo |  | 6 fns; b_app_tool.c |
| `asm/overlay_198_0220B120.s` | party/summary UI | P7 | todo |  | 183 fns; b_plist_anm.c b_plist_main.c |
| `asm/overlay_199_02212560.s` | networking / C-Gear | P7 | todo |  | 35 fns; wifibattlematch_sys.c |
| `asm/overlay_203_021BDA40.s` | UI / app | P7 | todo |  | 98 fns; demo3d.c demo3d_cmdsys.c demo3d_data.c demo3d_engine.c demo3d_exp_c_cruiser.c demo3d_graphic.c |
| `asm/overlay_204_021BDA40.s` | UI / app | P7 | todo |  | 121 fns; intro_cmd.c intro_g3d.c intro_graphic.c intro_mcss.c intro_msg.c intro_particle.c |
| `asm/overlay_206_021B9DE0.s` | UI / app | P7 | todo |  | 54 fns; zukan_info.c |
| `asm/overlay_207_0220AF40.s` | UI / app | P7 | todo |  | 29 fns; zukan_nickname.c zukan_toroku_graphic.c |
| `asm/overlay_208_021BC240.s` | networking / C-Gear | P7 | todo |  | 353 fns; zknd_tbar.c zukan_detail.c zukan_detail_common.c zukan_detail_form.c zukan_detail_graphic.c zukan_detail_headbar.c zukan_detail_procsys.c zukan_detail_touchbar.c |
| `asm/overlay_210_021BC240.s` | UI / app | P7 | todo |  | 193 fns; zknsearch_main.c |
| `asm/overlay_212_021C8E80.s` | UI / app | P7 | todo |  | 29 fns; zukan.c zukan_search_engine.c |
| `asm/overlay_213_021B95A0.s` | battle UI | P7 | todo |  | 74 fns; comm_btl_demo_graphic.c |
| `asm/overlay_214_021B95A0.s` | battle UI | P7 | todo |  | 89 fns; btl_rec_sel.c btl_rec_sel_graphic.c |
| `asm/overlay_215_021F57E0.s` | evolution/egg UI | P7 | todo |  | 91 fns; egg_demo_graphic.c egg_demo_view.c |
| `asm/overlay_216_021BBD80.s` | networking / C-Gear | P7 | todo |  | 73 fns; beacon_detail_gra.c |
| `asm/overlay_217_021B95A0.s` | networking / C-Gear | P7 | todo |  | 727 fns; arrow.c bg_font.c circle_graph.c palette_anime.c percentage.c queue.c research_common.c research_graph.c |
| `asm/overlay_218_021B95A0.s` | UI / app | P7 | todo |  | 61 fns; un_select.c un_select_graphic.c |
| `asm/overlay_220_021B95A0.s` | UI / app | P7 | todo |  | 26 fns; th_award_graphic.c |
| `asm/overlay_221_021B95A0.s` | UI / app | P7 | todo |  | 30 fns; chihou_zukan_award.c chihou_zukan_award_graphic.c |
| `asm/overlay_222_021B95A0.s` | UI / app | P7 | todo |  | 30 fns; zenkoku_zukan_award.c zenkoku_zukan_award_graphic.c |
| `asm/overlay_223_021B95A0.s` | UI / app | P7 | todo |  | 132 fns; psel.c psel_graphic.c |
| `asm/overlay_224_021B95A0.s` | UI / app | P7 | todo |  | 16 fns; subway_map_graphic.c |
| `asm/overlay_225_021B95A0.s` | networking / C-Gear | P7 | todo |  | 111 fns; manual_category.c manual_common.c manual_data.c manual_explain.c manual_graphic.c manual_list.c manual_title.c manual_top.c |
| `asm/overlay_227_021B95A0.s` | networking / C-Gear | P7 | todo |  | 3 fns; event_pdc_return.c |
| `asm/overlay_228_021DFC20.s` | UI / app | P7 | todo |  | 3 fns; 0pGsave_outside.c |
| `asm/unk_0209B750.s` | runtime library | P7 | todo |  | 9 fns; _ll_mul (0x0209C054), _s32_div_f (0x0209C0A4); modelled as Rust integer ops |
| `asm/unk_0209C2B0.s` | runtime library | P7 | partial | `black-battle::calc::u32_div` | 8 fns; _u32_div_f and other MWCC runtime helpers |
| `asm/nonmatchings/battle_record_restorestart.s` | unidentified | ? | todo |  | 0 fns |
| `asm/overlay_000_021542C0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_003_021542C0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_004_021542C0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_005_021542C0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_007_021542C0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_008_02154CA0.s` | unidentified overlay | ? | todo |  | 4 fns |
| `asm/overlay_013_021AEB20.s` | unidentified overlay | ? | todo |  | 93 fns |
| `asm/overlay_015_021797C0.s` | unidentified overlay | ? | todo |  | 52 fns |
| `asm/overlay_017_021B6100.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_024_021EE740.s` | unidentified overlay | ? | todo |  | 7 fns |
| `asm/overlay_025_021EE740.s` | unidentified overlay | ? | todo |  | 19 fns |
| `asm/overlay_026_021EE740.s` | unidentified overlay | ? | todo |  | 38 fns |
| `asm/overlay_027_021EE740.s` | unidentified overlay | ? | todo |  | 7 fns |
| `asm/overlay_028_021EE740.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_036_021F36C0.s` | unidentified overlay | ? | todo |  | 7 fns |
| `asm/overlay_037_021F36C0.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_038_021F36C0.s` | unidentified overlay | ? | todo |  | 6 fns |
| `asm/overlay_039_021F36C0.s` | unidentified overlay | ? | todo |  | 15 fns |
| `asm/overlay_040_021F36C0.s` | unidentified overlay | ? | todo |  | 4 fns |
| `asm/overlay_041_021F36C0.s` | unidentified overlay | ? | todo |  | 2 fns |
| `asm/overlay_042_021F36C0.s` | unidentified overlay | ? | todo |  | 2 fns |
| `asm/overlay_043_021F36C0.s` | unidentified overlay | ? | todo |  | 13 fns |
| `asm/overlay_044_021F36C0.s` | unidentified overlay | ? | todo |  | 2 fns |
| `asm/overlay_045_021F36C0.s` | unidentified overlay | ? | todo |  | 11 fns |
| `asm/overlay_046_021F36C0.s` | unidentified overlay | ? | todo |  | 16 fns |
| `asm/overlay_047_021F36C0.s` | unidentified overlay | ? | todo |  | 4 fns |
| `asm/overlay_049_021F36C0.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_050_021F36C0.s` | unidentified overlay | ? | todo |  | 9 fns |
| `asm/overlay_053_021F36C0.s` | unidentified overlay | ? | todo |  | 9 fns |
| `asm/overlay_054_021F36C0.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_055_021F39C0.s` | unidentified overlay | ? | todo |  | 33 fns |
| `asm/overlay_057_021F36C0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_058_021F36C0.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_060_021F36C0.s` | unidentified overlay | ? | todo |  | 12 fns |
| `asm/overlay_061_021F36C0.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_062_021F36C0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_064_021F36C0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_065_021F36C0.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_066_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_068_021B95A0.s` | unidentified overlay | ? | todo |  | 89 fns |
| `asm/overlay_069_021B95A0.s` | unidentified overlay | ? | todo |  | 35 fns |
| `asm/overlay_073_021F5260.s` | unidentified overlay | ? | todo |  | 2 fns |
| `asm/overlay_074_021F54E0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_075_021F5260.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_076_021F5260.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_077_021F5620.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_078_021F5620.s` | unidentified overlay | ? | todo |  | 4 fns |
| `asm/overlay_079_021F5A80.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_080_021F5A80.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_081_021F5A80.s` | unidentified overlay | ? | todo |  | 10 fns |
| `asm/overlay_083_021F5260.s` | unidentified overlay | ? | todo |  | 7 fns |
| `asm/overlay_084_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_085_021B6100.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_086_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_087_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_089_021BDA40.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_090_021B6100.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_099_021B95A0.s` | unidentified overlay | ? | todo |  | 433 fns |
| `asm/overlay_102_021B7560.s` | unidentified overlay | ? | todo |  | 64 fns |
| `asm/overlay_103_021B7560.s` | unidentified overlay | ? | todo |  | 74 fns |
| `asm/overlay_104_021B7560.s` | unidentified overlay | ? | todo |  | 73 fns |
| `asm/overlay_105_021B7560.s` | unidentified overlay | ? | todo |  | 50 fns |
| `asm/overlay_108_021BB8C0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_109_021BB8E0.s` | unidentified overlay | ? | todo |  | 1 fns |
| `asm/overlay_111_021C3CC0.s` | unidentified overlay | ? | todo |  | 114 fns |
| `asm/overlay_113_021B6100.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_122_021D4240.s` | unidentified overlay | ? | todo |  | 32 fns |
| `asm/overlay_126_021EE920.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_128_021C6180.s` | unidentified overlay | ? | todo |  | 167 fns |
| `asm/overlay_129_021D85C0.s` | unidentified overlay | ? | todo |  | 55 fns |
| `asm/overlay_130_021B6100.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_134_021DC860.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_138_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_139_02400020.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_140_02400040.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_141_02400040.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_142_02400040.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_143_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_144_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_145_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_146_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_147_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_148_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_149_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_150_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_151_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_152_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_153_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_154_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_155_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_156_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_157_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_158_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_159_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_160_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_161_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_162_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_163_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_164_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_165_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_166_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_167_021F5260.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_169_021B95A0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_171_021DC860.s` | unidentified overlay | ? | todo |  | 74 fns |
| `asm/overlay_173_021B6100.s` | unidentified overlay | ? | todo |  | 74 fns |
| `asm/overlay_178_021E5440.s` | unidentified overlay | ? | todo |  | 16 fns |
| `asm/overlay_180_021B6100.s` | unidentified overlay | ? | todo |  | 98 fns |
| `asm/overlay_181_021B95A0.s` | unidentified overlay | ? | todo |  | 53 fns |
| `asm/overlay_184_021F8480.s` | unidentified overlay | ? | todo |  | 16 fns |
| `asm/overlay_185_021F8480.s` | unidentified overlay | ? | todo |  | 37 fns |
| `asm/overlay_189_0217AE20.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_190_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_191_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_192_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_193_021EE740.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_200_021F57E0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_201_021B96A0.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_202_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_205_021B95A0.s` | unidentified overlay | ? | todo |  | 17 fns |
| `asm/overlay_209_021BC240.s` | unidentified overlay | ? | todo |  | 94 fns |
| `asm/overlay_211_021BC240.s` | unidentified overlay | ? | todo |  | 19 fns |
| `asm/overlay_219_02183B60.s` | unidentified overlay | ? | todo |  | 48 fns |
| `asm/overlay_226_0220BC60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_229_02183B60.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_230_02187F00.s` | unidentified overlay | ? | todo |  | 3 fns |
| `asm/overlay_231_02180820.s` | unidentified overlay | ? | todo |  | 5 fns |
| `asm/overlay_232_02212560.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_233_02212560.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_234_02212560.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_235_02212560.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/overlay_236_02212560.s` | unidentified overlay | ? | todo |  | 0 fns |
| `asm/unk_02004FD4.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_020055F8.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_02005BC0.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_02006118.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_0200674C.s` | ARM9 (unidentified) | ? | todo |  | 13 fns |
| `asm/unk_02006D3C.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_020071A8.s` | ARM9 (unidentified) | ? | todo |  | 19 fns |
| `asm/unk_02007AA4.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_02007F7C.s` | ARM9 (unidentified) | ? | todo |  | 21 fns |
| `asm/unk_02008468.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_0200857C.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_02008850.s` | ARM9 (unidentified) | ? | todo |  | 25 fns |
| `asm/unk_02008EF8.s` | ARM9 (unidentified) | ? | todo |  | 29 fns |
| `asm/unk_02009690.s` | ARM9 (unidentified) | ? | todo |  | 26 fns |
| `asm/unk_02009C44.s` | ARM9 (unidentified) | ? | todo |  | 22 fns |
| `asm/unk_0200A480.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_0200AE2C.s` | ARM9 (unidentified) | ? | todo |  | 25 fns |
| `asm/unk_0200B628.s` | ARM9 (unidentified) | ? | todo |  | 22 fns |
| `asm/unk_0200BDE4.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_0200C7C0.s` | ARM9 (unidentified) | ? | todo |  | 12 fns |
| `asm/unk_0200D3A4.s` | ARM9 (unidentified) | ? | todo |  | 21 fns |
| `asm/unk_0200E024.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_0200E4CC.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_0200EC84.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_0200F150.s` | ARM9 (unidentified) | ? | todo |  | 21 fns |
| `asm/unk_0200F7B8.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_0200FD38.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_020101D4.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_02010794.s` | ARM9 (unidentified) | ? | todo |  | 22 fns |
| `asm/unk_020111F8.s` | ARM9 (unidentified) | ? | todo |  | 24 fns |
| `asm/unk_02011D40.s` | ARM9 (unidentified) | ? | todo |  | 31 fns |
| `asm/unk_02012224.s` | ARM9 (unidentified) | ? | todo |  | 31 fns |
| `asm/unk_02012704.s` | ARM9 (unidentified) | ? | todo |  | 40 fns |
| `asm/unk_02012CE0.s` | ARM9 (unidentified) | ? | todo |  | 24 fns |
| `asm/unk_02013270.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_02013980.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_02014128.s` | ARM9 (unidentified) | ? | todo |  | 33 fns |
| `asm/unk_020147DC.s` | ARM9 (unidentified) | ? | todo |  | 31 fns |
| `asm/unk_02015580.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02015BA4.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02016630.s` | ARM9 (unidentified) | ? | todo |  | 7 fns |
| `asm/unk_02016BA0.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02016FC8.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02017550.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_02017BC8.s` | ARM9 (unidentified) | ? | todo |  | 21 fns |
| `asm/unk_02017FB0.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_020185B4.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02018D10.s` | ARM9 (unidentified) | ? | todo |  | 13 fns |
| `asm/unk_02019388.s` | ARM9 (unidentified) | ? | todo |  | 2 fns |
| `asm/unk_02019A50.s` | ARM9 (unidentified) | ? | todo |  | 25 fns |
| `asm/unk_0201A26C.s` | ARM9 (unidentified) | ? | todo |  | 12 fns |
| `asm/unk_0201A858.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_0201AF6C.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_0201BAE4.s` | ARM9 (unidentified) | ? | todo |  | 6 fns |
| `asm/unk_0201C41C.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_0201CA5C.s` | ARM9 (unidentified) | ? | todo |  | 7 fns |
| `asm/unk_0201D26C.s` | ARM9 (unidentified) | ? | todo |  | 19 fns |
| `asm/unk_0201D6C8.s` | ARM9 (unidentified) | ? | todo |  | 18 fns |
| `asm/unk_0201DB20.s` | ARM9 (unidentified) | ? | todo |  | 29 fns |
| `asm/unk_0201E4D4.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_0201EC64.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_0201F41C.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_0201FC1C.s` | ARM9 (unidentified) | ? | todo |  | 1214 fns |
| `asm/unk_02040618.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02040F84.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_02041B6C.s` | ARM9 (unidentified) | ? | todo |  | 9 fns |
| `asm/unk_020425B4.s` | ARM9 (unidentified) | ? | todo |  | 5 fns |
| `asm/unk_02042D2C.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02043900.s` | ARM9 (unidentified) | ? | todo |  | 138 fns |
| `asm/unk_02048CEC.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_0204944C.s` | ARM9 (unidentified) | ? | todo |  | 54 fns |
| `asm/unk_0204B0C8.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_0204BD3C.s` | ARM9 (unidentified) | ? | todo |  | 9 fns |
| `asm/unk_0204C98C.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_0204D9B4.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0204E6F4.s` | ARM9 (unidentified) | ? | todo |  | 5 fns |
| `asm/unk_02050A70.s` | ARM9 (unidentified) | ? | todo |  | 5 fns |
| `asm/unk_020520FC.s` | ARM9 (unidentified) | ? | todo |  | 2 fns |
| `asm/unk_02055964.s` | ARM9 (unidentified) | ? | todo |  | 40 fns |
| `asm/unk_020583CC.s` | ARM9 (unidentified) | ? | todo |  | 66 fns |
| `asm/unk_0205B1E4.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_0205B98C.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0205C0EC.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_0205C81C.s` | ARM9 (unidentified) | ? | todo |  | 8 fns |
| `asm/unk_0205D01C.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_0205D848.s` | ARM9 (unidentified) | ? | todo |  | 8 fns |
| `asm/unk_0205DFA8.s` | ARM9 (unidentified) | ? | todo |  | 8 fns |
| `asm/unk_0205E790.s` | ARM9 (unidentified) | ? | todo |  | 13 fns |
| `asm/unk_0205EE28.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0205F504.s` | ARM9 (unidentified) | ? | todo |  | 12 fns |
| `asm/unk_0205FFC4.s` | ARM9 (unidentified) | ? | todo |  | 33 fns |
| `asm/unk_02060C14.s` | ARM9 (unidentified) | ? | todo |  | 51 fns |
| `asm/unk_02062634.s` | ARM9 (unidentified) | ? | todo |  | 27 fns |
| `asm/unk_02062D40.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_020637F4.s` | ARM9 (unidentified) | ? | todo |  | 6 fns |
| `asm/unk_02063E54.s` | ARM9 (unidentified) | ? | todo |  | 102 fns |
| `asm/unk_02066878.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02067340.s` | ARM9 (unidentified) | ? | todo |  | 5 fns |
| `asm/unk_02067AAC.s` | ARM9 (unidentified) | ? | todo |  | 28 fns |
| `asm/unk_020726D0.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_02072DE8.s` | ARM9 (unidentified) | ? | todo |  | 23 fns |
| `asm/unk_02073484.s` | ARM9 (unidentified) | ? | todo |  | 18 fns |
| `asm/unk_02073FB8.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_02074770.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02074E10.s` | ARM9 (unidentified) | ? | todo |  | 12 fns |
| `asm/unk_02075698.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_02075DC8.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_02076A70.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_020774C0.s` | ARM9 (unidentified) | ? | todo |  | 8 fns |
| `asm/unk_02077C78.s` | ARM9 (unidentified) | ? | todo |  | 7 fns |
| `asm/unk_02078360.s` | ARM9 (unidentified) | ? | todo |  | 18 fns |
| `asm/unk_02078C18.s` | ARM9 (unidentified) | ? | todo |  | 13 fns |
| `asm/unk_020792B8.s` | ARM9 (unidentified) | ? | todo |  | 7 fns |
| `asm/unk_0207A7A8.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_0207B51C.s` | ARM9 (unidentified) | ? | todo |  | 4 fns |
| `asm/unk_0207C554.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_0207CDB8.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_0207D7C4.s` | ARM9 (unidentified) | ? | todo |  | 19 fns |
| `asm/unk_0207E2A0.s` | ARM9 (unidentified) | ? | todo |  | 13 fns |
| `asm/unk_0207E944.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_0207F024.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_0207F6A8.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_02080100.s` | ARM9 (unidentified) | ? | todo |  | 9 fns |
| `asm/unk_0208091C.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02081120.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_02081940.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02082594.s` | ARM9 (unidentified) | ? | todo |  | 12 fns |
| `asm/unk_02083484.s` | ARM9 (unidentified) | ? | todo |  | 10 fns |
| `asm/unk_020840F8.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_02084B90.s` | ARM9 (unidentified) | ? | todo |  | 20 fns |
| `asm/unk_020853C0.s` | ARM9 (unidentified) | ? | todo |  | 18 fns |
| `asm/unk_02085C08.s` | ARM9 (unidentified) | ? | todo |  | 19 fns |
| `asm/unk_020862B8.s` | ARM9 (unidentified) | ? | todo |  | 19 fns |
| `asm/unk_02086A50.s` | ARM9 (unidentified) | ? | todo |  | 18 fns |
| `asm/unk_0208738C.s` | ARM9 (unidentified) | ? | todo |  | 22 fns |
| `asm/unk_02087C6C.s` | ARM9 (unidentified) | ? | todo |  | 22 fns |
| `asm/unk_02088398.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_02088ED4.s` | ARM9 (unidentified) | ? | todo |  | 31 fns |
| `asm/unk_02089804.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_02089E74.s` | ARM9 (unidentified) | ? | todo |  | 15 fns |
| `asm/unk_0208A638.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0208AF18.s` | ARM9 (unidentified) | ? | todo |  | 9 fns |
| `asm/unk_0208B798.s` | ARM9 (unidentified) | ? | todo |  | 16 fns |
| `asm/unk_0208BEF0.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0208C64C.s` | ARM9 (unidentified) | ? | todo |  | 17 fns |
| `asm/unk_0208E010.s` | ARM9 (unidentified) | ? | todo |  | 11 fns |
| `asm/unk_0208EAA0.s` | ARM9 (unidentified) | ? | todo |  | 8 fns |
| `asm/unk_02090ED8.s` | ARM9 (unidentified) | ? | todo |  | 14 fns |
| `asm/unk_020923F0.s` | ARM9 (unidentified) | ? | todo |  | 1 fns |
| `asm/unk_020949F4.s` | ARM9 (unidentified) | ? | todo |  | 1 fns |
| `asm/unk_0209CE10.s` | ARM9 data | n/a | todo |  | 3 fns; holds the ARM9 debug filename table (poke_tool.c, pokeparty.c, box_savedata.c, myitem_savedata.c, shinka_check.c, waza_tool.c, vm.c, ...) |
| `src/unk_02008574.c` | unidentified | ? | verified | `black-core::unk_02008574` | decompiled C; differential fuzz vs host-compiled C |
