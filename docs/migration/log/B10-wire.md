# B10-wire — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase B, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] B10-wire — (6068f1e; Opus, ~1.5 h, 280k tokens) **production `server_flow::host::FlowWorkHost<E, U: Unported>`** over SvFlowWork: of 87 FlowHost methods 42 mapped (34 api, 4 hem, 3 server2/3, side_effect_layers), 6 adapted (party_ids/partner_party_ids, 021c81c8, 021c83dc, 021c846c, 021c2e4c → core4::counter_next), 38 → `Unported` (9 move-data methods ported in black-pokemon::waza, 3 item/param, 21 ov95, 4 api ov95 tail calls, msg_param); ~25 FlowHost docs rewritten from the verified ports (e.g. fun_021c83e8 tests battle TYPE not rule; fun_021c82b8 always false; positions_adjacent = reach list incl. self, not symmetric); 50 host tests (3/3 swaps caught); bool call sites checked to compare with 0. **Decision (coordinator): black-battle may depend on black-pokemon** (both logic, no cycle, D6 ok) → B10-wire2 wires the 9 move-data methods
