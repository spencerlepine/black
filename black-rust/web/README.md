# black-battle playground

A browser page that runs the ported battle-calc functions as WebAssembly. It has no
dependencies: `crates/black-battle-wasm` exports plain `extern "C"` integer functions and
`app.js` loads them with `WebAssembly.instantiateStreaming`.

```sh
rustup target add wasm32-unknown-unknown   # once
./web/build.sh                             # builds web/black_battle_wasm.wasm (gitignored)
python3 -m http.server 8765 --bind 127.0.0.1 --directory web
open http://127.0.0.1:8765/
```

It must be served over HTTP; browsers won't load WASM from `file://`. Each panel shows the
output of one ported function. The full damage pipeline isn't ported yet, so the page shows no
final damage number.
