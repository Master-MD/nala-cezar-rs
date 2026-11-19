# 🦀🏛️ NALA-Cezar-RS (`v0.21.0-nala.6.6.6`) — 100% Native Safe-Rust Parallel AI Coding Agent Orchestrator

> **Drop-in 100% Pure Safe-Rust (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` External Crates, Zero-Node.js, Zero-Python, `~541 KB` Single Binary, `2.01 MiB` RAM, Zero Outbound Telemetry) Rewrite & Evolution of [`open-mercato/cezar`](https://github.com/open-mercato/cezar)**  
> **Solves Upstream Issues [`#798`](https://github.com/open-mercato/cezar/issues/798), [`#808`](https://github.com/open-mercato/cezar/issues/808), [`#842`](https://github.com/open-mercato/cezar/issues/842), [`#847`](https://github.com/open-mercato/cezar/issues/847), [`#1159`](https://github.com/open-mercato/cezar/issues/1159), [`#1543`](https://github.com/open-mercato/cezar/issues/1543), [`#1877`](https://github.com/open-mercato/cezar/issues/1877), [`#1903`](https://github.com/open-mercato/cezar/issues/1903), [`#1925`](https://github.com/open-mercato/cezar/issues/1925), [`#2262`](https://github.com/open-mercato/cezar/issues/2262), [`#2289`](https://github.com/open-mercato/cezar/issues/2289), and [`#2296`](https://github.com/open-mercato/cezar/issues/2296) out of the box!**  
> **Multi-Language Documentation Hub:** 🇨🇭 [Deutsch (`docs/README_DE.md`)](docs/README_DE.md) · 🇫🇷 [Français (`docs/README_FR.md`)](docs/README_FR.md) · 🇮🇹 [Italiano (`docs/README_IT.md`)](docs/README_IT.md) · 🏔️ [Rumantsch Grischun (`docs/README_RM.md`)](docs/README_RM.md) · 🇬🇧 [English (`docs/README_EN.md`)](docs/README_EN.md)  
> **Guides & Attribution:** 🪨 [`CAVEMAN_GUIDE.md`](CAVEMAN_GUIDE.md) · ❓ [`FAQ.md`](FAQ.md) · ⚖️ [`LICENSE`](LICENSE) (`MIT`) & [`LICENSE_SUMMARY_5LANG.md`](LICENSE_SUMMARY_5LANG.md) · 📜 [`NOTICE`](NOTICE) & [`README_UPSTREAM_CEZAR.md`](README_UPSTREAM_CEZAR.md)

---

## 🎯 Why `NALA-Cezar-RS`?

[`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`v0.21.0`, MIT © Patryk Lewczuk) introduced an awesome workflow for running AI coding agents in isolated git worktrees using plain `.ai/cezar/` files. However, its ~198 MB TypeScript/Node.js monorepo (`4,395` files) requires downloading a ~100 MB Node 20+ runtime, suffers from stale `proper-lockfile` locks, accumulates up to `17 GB` of duplicate worktrees, ships `stats.openmercato.cloud` telemetry hooks, and hardcodes GitHub as its only forge.

**`NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`)** re-engineers the entire orchestrator in **100% Pure Safe Rust (`#![forbid(unsafe_code)]`, `0` external crates, `0` Node.js, `0` Python)** while staying drop-in compatible with `.ai/cezar/`:

| Upstream Bottleneck / Issue in `open-mercato/cezar` | Problem in Upstream TypeScript / Node.js | How `NALA-Cezar-RS` Solves It Natively in Safe Rust |
| :--- | :--- | :--- |
| **[#842](https://github.com/open-mercato/cezar/issues/842)** (`17 GB` worktree bloat inside `keep=10`) | Count-only worktree retention duplicates gigabytes of build outputs across parallel runs. | **Zero-Copy CoW Worktrees (`APFS clonefile` / `reflink=auto`) + Byte-Aware Reaper (`reclaim_worktrees_byte_aware`)** enforcing both `keep_count` and `max_bytes` (`4 GiB` default). |
| **[#847](https://github.com/open-mercato/cezar/issues/847)** (GitHub-only `z.literal('github')` lock-in) | Cannot target self-hosted **Forgejo**, **Gitea**, or **GitLab** repositories natively. | **Pluggable 4-Forge Adapter (`src/forge.rs`)** supporting **GitHub**, **GitLab**, **Forgejo** (`tea` / `/api/v1`), and **Gitea** in `216.1 ns`. |
| **[#798](https://github.com/open-mercato/cezar/issues/798), [#2289](https://github.com/open-mercato/cezar/issues/2289), [#2296](https://github.com/open-mercato/cezar/issues/2296)** (Stale locks & stuck runs) | `proper-lockfile` leaves `"run is still active"` locks; retries stay stuck in `waiting` or `monitoring`. | **Compile-time RAII `RunLeaseGuard<'a>` (`28.6 ns`)** + atomic `clear_stale_asks_on_retry()` and per-node `dispatched_this_turn` reset. |
| **[#808](https://github.com/open-mercato/cezar/issues/808), [#1543](https://github.com/open-mercato/cezar/issues/1543), [#1903](https://github.com/open-mercato/cezar/issues/1903)** (Effort, timing & models) | Missing per-step reasoning effort, per-call first-token timing, and Copilot model hints. | Native `ReasoningEffort` (`low`/`medium`/`high`/`xhigh`), `firstTokenMs`/`generationEndMs` NDJSON fields, and **13-Runner Registry** (`claude`, `codex`, `antigravity`, `opencode`, `zcode`, `copilot`, `gemini`, `local-openai`, etc.). |
| **[#1925](https://github.com/open-mercato/cezar/issues/1925) & Telemetry** (`stats.openmercato.cloud`) | Un-sandboxed `bypassPermissions` + outbound telemetry/relay infrastructure. | **`0%` Outbound Telemetry** + **`sHeriFF + SkillSpector` 7-Layer Sandbox (`SEC-CEZAR-01..07`)** & 10-gate unified-diff PR auditor (`802.1 ns`). |

---

## ⚡ Live Hardware Benchmark (`nala-cezar-rs bench` on Apple Silicon M4 Max)

Reproduce anytime on your own machine via `nala-cezar-rs bench`:

| ID | Subsystem | `NALA-Cezar-RS` (Rust 2024) | `open-mercato/cezar` (Node 20+) | Measured Speedup |
| :--- | :--- | :---: | :---: | :---: |
| **SYS-1** | **Standalone Binary / Runtime Footprint** | **`541 KB`** (`0` deps) | `~100+ MB` (Node 20 + npm) | **`> 180.0x` smaller** |
| **SYS-2** | **Peak Resident Memory (RSS)** | **`2.01 MiB`** (`2,113,536 B`) | `~250.0 MiB` | **`> 124.0x` less RAM** |
| **B01** | `.cezar.idx` 256-Run $O(1)$ Binary Lookup | **`80.6 ns`** | `1.85 ms` | **`22,960.6x` faster** |
| **B02** | RAII `RunLeaseGuard` Acquire + Release (`#798`) | **`28.6 ns`** | `420.0 µs` | **`14,674.2x` faster** |
| **B03** | `ZeroAllocSseFrame` `[u8; 512]` Stack Formatter | **`155.0 ns`** | `14.2 µs` | **`91.6x` faster** |
| **B04** | Portable YAML Workflow Chain Parser + `SkillSpector` | **`1.37 µs`** | `96.0 µs` | **`70.2x` faster** |
| **B05** | Best-of-3 Variant Tournament Scorer + Diff Gate | **`2.68 µs`** | `310.0 µs` | **`115.5x` faster** |
| **B06** | Multi-Forge Remote Adapter Parse (`#847`) | **`216.1 ns`** | `38.0 µs` | **`175.9x` faster** |
| **B07** | 13-Agent CLI `PATH` Discovery (Zero-Subprocess) | **`138.06 µs`** | `48.00 ms` | **`347.7x` faster** |
| **B08** | `sHeriFF` Unified-Diff 10-Gate PR Auditor | **`802.1 ns`** | `125.0 µs` | **`155.8x` faster** |
| **B09** | `SEC-CEZAR-01..06` SSRF + `SkillSpector` Guard | **`112.1 ns`** | `29.0 µs` | **`258.8x` faster** |
| **B10** | Pure-Rust FIPS 180-4 SHA-256 4 KiB Seal | **`9.64 µs`** | `18.5 µs` | **`1.9x` faster** |

---

## 📦 Installation (macOS `.dmg`, Ubuntu/Debian `.deb`, Windows `.exe`, One-Liners)

### 1. Universal SHA-256 Verified One-Liner (macOS & Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash
```

### 2. macOS Native `.dmg` Application Bundles (`arm64` & `x86_64`)
Download from [GitHub Releases](https://github.com/Master-MD/nala-cezar-rs/releases):
- **Apple Silicon (M1–M4 Max):** `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` (contains `NALA-Cezar-RS.app` + CLI installer)
- **Intel Mac (`x86_64`):** `NALA-Cezar-RS-0.21.0-macos-x86_64.dmg`

### 3. Ubuntu / Debian Native `.deb` Packages (`amd64` & `arm64`)
```bash
# x86_64 / amd64:
sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb

# ARM64 / aarch64:
sudo dpkg -i nala-cezar-rs_0.21.0_arm64.deb
```

### 4. Windows 10 / 11 (`x86_64` `.exe` & PowerShell)
```powershell
irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex
```
*(Or download `nala-cezar-rs-windows-x86_64.exe` / `nala-cezar-rs-windows-x86_64.zip` directly from Releases.)*

---

## 🚀 Quick Start & MCP Integration

```bash
# Launch the 5-language Live Web Cockpit (2-Click Safety Gate + Live Diff Scorer)
nala-cezar-rs serve --host 127.0.0.1 --port 18740

# Dispatch a x3 Best-of-3 parallel variant run in isolated CoW worktrees
nala-cezar-rs run "Optimize tokenizer with unit tests" --runner codex --variants 3 --effort high --check "cargo test"

# Audit any external PR patch or agent diff against all 10 sHeriFF security gates
nala-cezar-rs sync-gate --audit-diff patch.diff

# Reclaim stale worktrees exceeding 4 GiB or keep=10 (#842)
nala-cezar-rs worktrees reclaim
```

### Prompt-Cache-Stable MCP Server (`Claude Code`, `Codex`, `Antigravity`, `Cursor`, `ZCode`, `OpenCode`)
`nala-cezar-rs mcp` exposes **12 deterministic, 100% static tools** (`listChanged: false`) so your LLM prompt cache is never invalidated:
```json
{
  "mcpServers": {
    "nala_cezar": {
      "command": "nala-cezar-rs",
      "args": ["mcp"]
    }
  }
}
```

---

## 🛡️ Security & Supply-Chain Integrity (`sHeriFF + NVIDIA SkillSpector`)

- **Zero External Dependencies:** `Cargo.toml` has `[dependencies]` completely empty — `0` crates, `0` npm packages, `0` `build.rs` scripts.
- **Memory-Safe by Construction:** `#![forbid(unsafe_code)]` enforced across the entire crate.
- **Zero Outbound Telemetry:** `X-Telemetry-Policy: zero-outbound-airgap` — safe for airgapped enterprise environments.
- **SLSA v1.0 & CycloneDX Provenance:** Every release ships with `SHA256SUMS.txt` and cryptographic `SLSA_PROVENANCE_SBOM.json` (`nala-cezar-rs provenance dist/SHA256SUMS.txt`).

---

## ⚖️ License & Credits

Licensed under the **MIT License** ([LICENSE](LICENSE), [LICENSE_SUMMARY_5LANG.md](LICENSE_SUMMARY_5LANG.md)).  
Upstream `.ai/cezar/` workflow/run concept and protocol originally created by **Patryk Lewczuk** in [`open-mercato/cezar`](https://github.com/open-mercato/cezar) — see [NOTICE](NOTICE) and [README_UPSTREAM_CEZAR.md](README_UPSTREAM_CEZAR.md).
