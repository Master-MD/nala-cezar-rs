//! Embedded Anti-AI-Slop 5-Language (`DE/FR/IT/RM/EN`) Live Orchestrator Cockpit UI.
//! Zero external CDN calls (100% airgap-ready), Precision Carbon/Emerald Telemetry Aesthetic,
//! 2-Click Safety Execution Gate, Live Best-of-3 Diff Tournament, and Byte-Aware Worktree Monitor.

pub const COCKPIT_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>NALA-Cezar-RS — Parallel Coding Agent Cockpit</title>
<style>
  :root {
    --bg: #0b0d0e;
    --panel: #111517;
    --panel-alt: #161b1e;
    --border: #232b30;
    --text: #e6ecee;
    --muted: #8a99a0;
    --accent: #10b981;
    --accent-dim: rgba(16, 185, 129, 0.14);
    --amber: #f59e0b;
    --danger: #ef4444;
    --mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    --sans: -apple-system, BlinkMacSystemFont, "Inter", "Segoe UI", sans-serif;
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: var(--bg);
    color: var(--text);
    font-family: var(--sans);
    font-size: 14px;
    line-height: 1.5;
    min-height: 100vh;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .brand { display: flex; align-items: center; gap: 12px; }
  .brand-mark {
    width: 30px; height: 30px; border-radius: 6px;
    background: var(--accent-dim); border: 1px solid var(--accent);
    color: var(--accent); font-family: var(--mono); font-weight: 700;
    display: grid; place-items: center; font-size: 13px;
  }
  .brand h1 { font-size: 15px; font-weight: 600; letter-spacing: -0.01em; }
  .brand span { font-size: 12px; color: var(--muted); font-family: var(--mono); }
  .badges { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .badge {
    font-family: var(--mono); font-size: 11px; padding: 4px 8px;
    border-radius: 4px; border: 1px solid var(--border); background: var(--panel-alt);
    color: var(--muted);
  }
  .badge.ok { border-color: rgba(16,185,129,0.45); color: var(--accent); background: var(--accent-dim); }
  .lang-bar button {
    background: transparent; color: var(--muted); border: 1px solid var(--border);
    padding: 3px 7px; font-family: var(--mono); font-size: 11px; border-radius: 4px; cursor: pointer;
  }
  .lang-bar button.active { color: var(--text); border-color: var(--accent); background: var(--accent-dim); }
  main {
    max-width: 1380px; margin: 0 auto; padding: 20px 24px;
    display: grid; grid-template-columns: 360px 1fr; gap: 18px;
  }
  @media (max-width: 980px) { main { grid-template-columns: 1fr; } }
  .card {
    background: var(--panel); border: 1px solid var(--border);
    border-radius: 8px; padding: 16px; margin-bottom: 16px;
  }
  .card h2 {
    font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--muted); margin-bottom: 12px; font-family: var(--mono);
  }
  label { display: block; font-size: 12px; color: var(--muted); margin: 8px 0 4px; }
  input, select, textarea {
    width: 100%; background: var(--bg); color: var(--text);
    border: 1px solid var(--border); border-radius: 5px;
    padding: 8px 10px; font-family: inherit; font-size: 13px;
  }
  textarea { font-family: var(--mono); min-height: 76px; resize: vertical; }
  .btn-row { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
  button.primary {
    background: var(--accent); color: #04130d; border: none; font-weight: 600;
    padding: 8px 14px; border-radius: 5px; cursor: pointer; font-size: 13px;
  }
  button.secondary {
    background: var(--panel-alt); color: var(--text); border: 1px solid var(--border);
    padding: 8px 12px; border-radius: 5px; cursor: pointer; font-size: 12px;
  }
  button.confirm { background: var(--amber); color: #181002; border: none; font-weight: 700; padding: 8px 14px; border-radius: 5px; cursor: pointer; }
  .gate-box {
    display: none; margin-top: 10px; padding: 10px; border-radius: 6px;
    border: 1px solid var(--amber); background: rgba(245, 158, 11, 0.09);
  }
  .gate-box.visible { display: block; }
  table { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  th, td { text-align: left; padding: 8px 10px; border-bottom: 1px solid var(--border); }
  th { color: var(--muted); font-family: var(--mono); font-size: 11px; font-weight: 500; }
  .mono { font-family: var(--mono); }
  .pill { padding: 2px 6px; border-radius: 3px; font-family: var(--mono); font-size: 11px; }
  .pill.on { background: var(--accent-dim); color: var(--accent); }
  .pill.off { background: #1d2226; color: var(--muted); }
  pre {
    background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
    padding: 12px; font-family: var(--mono); font-size: 12px; overflow-x: auto; color: #c7d5da;
  }
</style>
</head>
<body>
<header>
  <div class="brand">
    <div class="brand-mark">CZ</div>
    <div>
      <h1>NALA-Cezar-RS <span id="ver">v0.21.0-nala.6.6.6</span></h1>
      <span id="subtitle">100% Native Safe-Rust Parallel Coding Agent Orchestrator · Zero-Node.js · Zero-Python</span>
    </div>
  </div>
  <div class="badges">
    <span class="badge ok" id="sheriff-badge">sHeriFF: ZERO-TELEMETRY &amp; PASS_ZERO_PII</span>
    <span class="badge" id="forge-badge">Forge: probing…</span>
    <div class="lang-bar">
      <button onclick="setLang('de')" id="l-de">DE</button>
      <button onclick="setLang('fr')" id="l-fr">FR</button>
      <button onclick="setLang('it')" id="l-it">IT</button>
      <button onclick="setLang('rm')" id="l-rm">RM</button>
      <button onclick="setLang('en')" id="l-en" class="active">EN</button>
    </div>
  </div>
</header>
<main>
  <section>
    <div class="card">
      <h2 id="h-dispatch">Dispatch Task / Best-of-3 Arena</h2>
      <label id="lbl-title">Task Title / Prompt</label>
      <textarea id="task-title" placeholder="Refactor module with unit tests and zero allocations…"></textarea>
      <label>Runner Backend</label>
      <select id="task-runner">
        <option value="codex">codex (GPT-5.3-Codex)</option>
        <option value="claude">claude (Claude Code CLI)</option>
        <option value="antigravity">antigravity (agy CLI)</option>
        <option value="opencode">opencode</option>
        <option value="zcode">zcode</option>
        <option value="gemini">gemini</option>
        <option value="local-openai">local-openai (Ollama / SwitchYard)</option>
      </select>
      <label>Mode</label>
      <select id="task-mode">
        <option value="single">Single Isolated CoW Worktree</option>
        <option value="arena3">x3 Best-of-3 Variant Tournament (Codex + Claude + Local)</option>
      </select>
      <div class="btn-row">
        <button class="primary" onclick="armTwoClickGate()" id="btn-arm">1. Prepare Dispatch</button>
        <button class="secondary" onclick="reclaimWorktrees()">Reclaim Worktrees (#842)</button>
      </div>
      <div class="gate-box" id="gate-box">
        <div style="font-size:12px;margin-bottom:8px;" id="gate-msg">2-Click Safety Gate: Confirm isolated CoW worktree creation?</div>
        <div class="btn-row" style="margin-top:0;">
          <button class="confirm" onclick="executeDispatch()">2. EXECUTE NOW</button>
          <button class="secondary" onclick="cancelGate()">Cancel</button>
        </div>
      </div>
    </div>

    <div class="card">
      <h2>13 Coding Agent Runners</h2>
      <table>
        <thead><tr><th>Runner</th><th>CLI</th><th>Status</th></tr></thead>
        <tbody id="backends-tbody"></tbody>
      </table>
    </div>
  </section>

  <section>
    <div class="card">
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;">
        <h2 style="margin:0;" id="h-runs">Active &amp; Recorded Worktree Runs (.ai/cezar/runs)</h2>
        <button class="secondary" onclick="refreshAll()">Refresh</button>
      </div>
      <table>
        <thead><tr><th>Run ID</th><th>Group</th><th>Runner / Model</th><th>Status</th><th>Tokens</th></tr></thead>
        <tbody id="runs-tbody"><tr><td colspan="5" class="mono">Loading…</td></tr></tbody>
      </table>
    </div>

    <div class="card">
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;">
        <h2 style="margin:0;">Live Hardware Benchmark vs. Node.js Upstream</h2>
        <button class="secondary" onclick="runLiveBench()">Run 10-Subsystem Benchmark</button>
      </div>
      <pre id="bench-out">Click "Run 10-Subsystem Benchmark" to measure live nanosecond latencies on this host.</pre>
    </div>
  </section>
</main>
<script>
const I18N = {
  en: { sub: "100% Native Safe-Rust Parallel Coding Agent Orchestrator · Zero-Node.js · Zero-Python", disp: "Dispatch Task / Best-of-3 Arena", runs: "Active & Recorded Worktree Runs (.ai/cezar/runs)", gate: "2-Click Safety Gate: Confirm isolated CoW worktree execution?" },
  de: { sub: "100% Nativer Safe-Rust Parallel-Coding-Agent-Orchestrator · 0% Node.js · 0% Python", disp: "Aufgabe / Best-of-3 Arena starten", runs: "Aktive & gespeicherte Worktree-Runs (.ai/cezar/runs)", gate: "2-Klick-Sicherheits-Gate: Wollen Sie den isolierten CoW-Worktree-Run wirklich AUSFÜHREN?" },
  fr: { sub: "Orchestrateur d'agents IA parallèle 100% Rust natif sûr · Zéro Node.js · Zéro Python", disp: "Lancer une tâche / Arène Best-of-3", runs: "Exécutions Worktree actives (.ai/cezar/runs)", gate: "Validation en 2 clics : Confirmer l'exécution dans un worktree CoW isolé ?" },
  it: { sub: "Orchestratore parallelo di agenti IA 100% Rust nativo sicuro · Zero Node.js · Zero Python", disp: "Avvia attività / Arena Best-of-3", runs: "Esecuzioni Worktree attive (.ai/cezar/runs)", gate: "Gate di sicurezza a 2 clic: Confermi l'esecuzione nel worktree CoW isolato?" },
  rm: { sub: "Orchestratur parallel d'agents d'IA 100% Rust nativ segir · Zero Node.js · Zero Python", disp: "Lantschar incumbensa / Arena Best-of-3", runs: "Execuziuns Worktree activas (.ai/cezar/runs)", gate: "Porta da segirezza en 2 clics: Confermar l'execuziun en il worktree CoW isolà?" }
};
function setLang(l) {
  document.querySelectorAll('.lang-bar button').forEach(b => b.classList.remove('active'));
  document.getElementById('l-' + l).classList.add('active');
  const t = I18N[l] || I18N.en;
  document.getElementById('subtitle').textContent = t.sub;
  document.getElementById('h-dispatch').textContent = t.disp;
  document.getElementById('h-runs').textContent = t.runs;
  document.getElementById('gate-msg').textContent = t.gate;
}
function armTwoClickGate() { document.getElementById('gate-box').classList.add('visible'); }
function cancelGate() { document.getElementById('gate-box').classList.remove('visible'); }
async function executeDispatch() {
  cancelGate();
  const title = document.getElementById('task-title').value.trim() || 'Automated verification task';
  const runner = document.getElementById('task-runner').value;
  const mode = document.getElementById('task-mode').value;
  await fetch('/api/v1/runs', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({ title, runner, mode })
  });
  await refreshAll();
}
async function reclaimWorktrees() {
  const r = await fetch('/api/v1/worktrees/reclaim', { method: 'POST' });
  const j = await r.json();
  document.getElementById('bench-out').textContent = 'Byte-Aware Worktree Reaper (#842):\n' + JSON.stringify(j, null, 2);
}
async function runLiveBench() {
  document.getElementById('bench-out').textContent = 'Running 10 hardware benchmarks…';
  const r = await fetch('/api/v1/bench');
  const j = await r.json();
  let lines = ['ID   Subsystem                                         Rust Latency     Node Upstream    Speedup', '────────────────────────────────────────────────────────────────────────────────────────────────'];
  for (const b of j.benchmarks) {
    const rStr = b.rust_ns >= 1000 ? (b.rust_ns/1000).toFixed(2)+' us' : b.rust_ns.toFixed(1)+' ns';
    const nStr = b.upstream_node_ns >= 1e6 ? (b.upstream_node_ns/1e6).toFixed(2)+' ms' : (b.upstream_node_ns/1000).toFixed(1)+' us';
    lines.push(b.id.padEnd(5) + b.subsystem.padEnd(50) + rStr.padStart(12) + nStr.padStart(17) + (b.speedup_x.toFixed(1)+'x').padStart(11));
  }
  document.getElementById('bench-out').textContent = lines.join('\n');
}
async function refreshAll() {
  try {
    const h = await (await fetch('/api/v1/health')).json();
    document.getElementById('ver').textContent = 'v' + h.version;
    document.getElementById('forge-badge').textContent = h.forge ? ('Forge: ' + h.forge.kind + ' (' + h.forge.host + ')') : ('Branch: ' + h.branch);
    const bk = await (await fetch('/api/v1/backends')).json();
    document.getElementById('backends-tbody').innerHTML = bk.backends.map(b =>
      `<tr><td class="mono">${b.name}</td><td class="mono">${b.binary}</td><td><span class="pill ${b.available?'on':'off'}">${b.available?'READY':'CLI ABSENT'}</span></td></tr>`
    ).join('');
    const rn = await (await fetch('/api/v1/runs')).json();
    if (!rn.runs.length) {
      document.getElementById('runs-tbody').innerHTML = '<tr><td colspan="5" class="mono" style="color:var(--muted)">No runs recorded yet — use the 2-Click Dispatch panel on the left.</td></tr>';
    } else {
      document.getElementById('runs-tbody').innerHTML = rn.runs.map(r =>
        `<tr><td class="mono">${r.id}</td><td class="mono">${r.groupId||'-'}</td><td class="mono">${r.runner} · ${r.model}</td><td><span class="pill on">${r.status}</span></td><td class="mono">${r.tokensTotal}</td></tr>`
      ).join('');
    }
  } catch (e) { console.error(e); }
}
refreshAll();
</script>
</body>
</html>
"##;
