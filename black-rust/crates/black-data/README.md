# black-data

Data types and loaders for the extracted assets (`assets/data`, `assets/maps`, `assets/text`).

One module per table. Each owns its serde types plus, behind the `rom` feature (native only,
pulls in `nds-rom`), a converter from the ROM's records. Converters are used by
`black extract-assets`; the game loads only the RON output (D4). Never enable `rom` for wasm.

| module | source | item |
|---|---|---|
| `personal` | ARC 016 | A7a |
| `learnset` | ARC 018 | A7b |
| `evolution` | ARC 019, 020 | A7c |
| `moves` | ARC 021 | A7d |
| `items` | ARC 024 | A7e |
| `trainers` | ARC 092, 093 | A7f |
| `encounters` | ARC 126 | A7g |
| `growth` | ARC 017 | A7h |
| `types` | type chart (ROM) | A7i |
