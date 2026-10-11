"use strict";

const TYPES = ["Normal", "Fighting", "Flying", "Poison", "Ground", "Rock", "Bug", "Ghost", "Steel",
  "Fire", "Water", "Grass", "Electric", "Psychic", "Ice", "Dragon", "Dark"];
const NONE = 17;
const EFF = [["immune", "×0"], ["quarter", "×¼"], ["half", "×½"], ["neutral", "×1"],
  ["double", "×2"], ["quadruple", "×4"]];

const $ = (id) => document.getElementById(id);
const u32 = (v) => v >>> 0;
const num = (id) => Number($(id).value) | 0;

async function load() {
  const res = await fetch("black_battle_wasm.wasm");
  if (!res.ok) throw new Error("black_battle_wasm.wasm not found. Run web/build.sh first.");
  const { instance } = await WebAssembly.instantiateStreaming(res, {});
  return instance.exports;
}

function fillTypes(sel, withNone) {
  TYPES.forEach((t, i) => sel.add(new Option(t, i)));
  if (withNone) sel.add(new Option("(none)", NONE));
}

function main(w) {
  fillTypes($("tAtk"), false);
  fillTypes($("tD1"), false);
  fillTypes($("tD2"), true);
  $("tAtk").value = 12; $("tD1").value = 10; $("tD2").value = 2; // Electric vs Water/Flying

  // Type chart, rendered from the ported table via single-type lookups.
  const chart = $("chart");
  const head = chart.insertRow();
  head.appendChild(document.createElement("th"));
  TYPES.forEach((t) => { const th = document.createElement("th"); th.textContent = t.slice(0, 2); th.title = t; head.appendChild(th); });
  TYPES.forEach((a, ai) => {
    const row = chart.insertRow();
    const th = document.createElement("th"); th.textContent = a.slice(0, 3); row.appendChild(th);
    TYPES.forEach((d, di) => {
      const e = w.type_effectiveness(ai, di, di);
      const td = row.insertCell();
      td.className = { 0: "e0", 2: "e2", 4: "e8" }[e] || "";
      td.textContent = { 0: "0", 2: "½", 4: "2" }[e] || "";
      td.title = `${a} → ${d}: ${EFF[e][1]}`;
      td.style.cursor = "pointer";
      td.onclick = () => { $("tAtk").value = ai; $("tD1").value = di; $("tD2").value = NONE; update(); };
    });
  });

  function update() {
    const eff = w.type_effectiveness(num("tAtk"), num("tD1"), num("tD2"));
    $("effOut").textContent = `${EFF[eff][1]} (${EFF[eff][0]})`;

    const atk = w.apply_stat_stage(num("atk"), num("atkS"));
    const def = w.apply_stat_stage(num("def"), num("defS"));
    $("staged").textContent = `${atk} / ${def}`;
    const base = u32(w.base_damage(num("pow"), atk, num("lvl"), def));
    $("base").textContent = base;
    $("typed").textContent = `${u32(w.apply_type_effectiveness(base, eff))} (${EFF[eff][1]})`;
    const mod = u32(w.weather_power_modifier(num("wx"), num("tAtk")));
    $("wxMod").textContent = `0x${mod.toString(16)} (×${mod / 4096})`;
  }

  function roll() {
    const hex = $("seed").value.replace(/^0x/i, "").padStart(16, "0").slice(-16);
    if (!/^[0-9a-f]{16}$/i.test(hex)) { $("rolls").textContent = "Seed must be hex."; return; }
    w.seed_rng(parseInt(hex.slice(0, 8), 16), parseInt(hex.slice(8), 16));
    const n = Math.max(1, num("trials"));

    let crits = 0;
    for (let i = 0; i < n; i++) crits += w.roll_critical(num("critS"));
    const hist = [0, 0, 0, 0, 0, 0];
    for (let i = 0; i < n; i++) hist[w.roll_hit_count()]++;
    const acc = w.apply_accuracy_stage(num("acc"), num("accS"));
    let hits = 0;
    for (let i = 0; i < n; i++) hits += w.roll_percent(acc);

    const pct = (k) => (100 * k / n).toFixed(2) + "%";
    let html = `<div class="row"><span>Crits at stage ${num("critS")}</span><b>${crits} / ${n} (${pct(crits)})</b></div>`;
    html += `<div class="row"><span>Accuracy after stage</span><b>${acc}% → ${pct(hits)} hit</b></div>`;
    html += `<div class="row" style="margin-top:8px"><span>2–5 hit move</span><span></span></div>`;
    for (let k = 2; k <= 5; k++) {
      html += `<div class="row"><span>${k} hits</span><b>${pct(hist[k])}</b></div>` +
        `<div class="bar" style="width:${(100 * hist[k] / n).toFixed(1)}%"></div>`;
    }
    $("rolls").innerHTML = html;
  }

  document.querySelectorAll("input, select").forEach((el) => el.addEventListener("input", update));
  $("roll").onclick = roll;
  update();
  roll();
}

load().then(main).catch((e) => { $("err").textContent = e.message; });
