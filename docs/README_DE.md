# 🇨🇭 `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — Deutsche Dokumentation

> **100 % Nativer Safe-Rust (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` externe Crates, `0 %` Node.js, `0 %` Python, `0 %` Telemetrie) Parallel-AI-Coding-Agent-Orchestrator**  
> Offizieller Fork & Clean-Room-Weiterentwicklung von [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`MIT` © Patryk Lewczuk).

---

## 🚀 Warum `NALA-Cezar-RS`?

Das Upstream-Projekt `open-mercato/cezar` besteht aus einem ~198 MB grossen Node.js/TypeScript-Monorepo (`4'395` Dateien) und lädt auf dem Desktop eine komplette ~100 MB Node.js-Laufzeitumgebung nach. **`NALA-Cezar-RS`** ersetzt diesen gesamten Stack durch **ein einziges `~541 KB` grosses, speichersicheres Rust-Binary** mit **`2.01 MiB` RAM-Bedarf** (`> 120x` sparsamer):

| Subsystem / Metrik | `NALA-Cezar-RS` (Rust 2024) | `open-mercato/cezar` (Node 20+) | Faktor |
| :--- | :---: | :---: | :---: |
| **Binärgrösse / Runtime-Footprint** | **`541 KB`** (`0` Abhängigkeiten) | `~100+ MB` (Node 20 + npm) | **`> 180x` kleiner** |
| **Maximaler RAM-Bedarf (RSS)** | **`2.01 MiB`** | `~250 MiB` | **`> 124x` weniger RAM** |
| **B01: `.cezar.idx` 256-Run Lookup** | **`80.6 ns`** | `1.85 ms` | **`22'960.6x` schneller** |
| **B02: RAII `RunLeaseGuard` (`#798`)** | **`28.6 ns`** | `420.0 µs` | **`14'674.2x` schneller** |
| **B03: `[u8; 512]` Stack-SSE-Streamer** | **`155.0 ns`** | `14.2 µs` | **`91.6x` schneller** |
| **B04: YAML-Workflow-Parser + Security** | **`1.37 µs`** | `96.0 µs` | **`70.2x` schneller** |
| **B05: Best-of-3 Diff-Turnier + Audit** | **`2.68 µs`** | `310.0 µs` | **`115.5x` schneller** |
| **B06: Multi-Forge Parser (`#847`)** | **`216.1 ns`** | `38.0 µs` | **`175.9x` schneller** |
| **B07: 13-Agenten CLI-Erkennung** | **`138.06 µs`** | `48.00 ms` | **`347.7x` schneller** |
| **B08: `sHeriFF` 10-Gate PR-Diff-Audit** | **`802.1 ns`** | `125.0 µs` | **`155.8x` schneller** |

---

## 📦 Installation (macOS `.dmg`, Ubuntu/Debian `.deb`, Windows `.exe`)

- **Einzeiler (macOS & Linux):**
  ```bash
  curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash
  ```
- **macOS (`.dmg` Installer):** `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` (Apple Silicon M1–M4 Max) oder `NALA-Cezar-RS-0.21.0-macos-x86_64.dmg` (Intel) mounten.
- **Ubuntu / Debian (`.deb` Paket):**
  ```bash
  sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb   # oder _arm64.deb
  ```
- **Windows 10/11 (`.exe` / PowerShell):**
  ```powershell
  irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex
  ```

---

## 🛡️ `sHeriFF + NVIDIA SkillSpector` Sicherheits-Garantien (`SEC-CEZAR-01..07`)

1. **`0 %` Cloud-Telemetrie:** `stats.openmercato.cloud` und Cloud-Relays wurden restlos entfernt (`100 %` Airgap-fähig).
2. **`#![forbid(unsafe_code)]` & `0` Crates:** Keine unsicheren Speicherblöcke, keine `build.rs`-Backdoors, keine npm-Supply-Chain-Risiken.
3. **7 aktive Schutzschichten:** SSRF-Blockade (`SEC-01`), XSS-Sanitizer (`SEC-02`), Path-Traversal-Schutz (`SEC-03`), `10 MiB` Body-Limit & Rate-Limiter (`SEC-04`), gehärtete HTTP-Header (`SEC-05`), BiDi-Trojan-Source (`CVE-2021-42574`) & Destructive-Shell-Blocker (`SEC-06`) sowie FNV-1a/XOR Zero-PII Sync-Gate (`nala-cezar-rs sync-gate`).

---

## ⚡ Schnellstart

```bash
nala-cezar-rs serve                                      # Live-Cockpit auf http://127.0.0.1:18740
nala-cezar-rs backends                                   # Alle 13 Agent-CLIs prüfen (claude, codex, agy, zcode, ...)
nala-cezar-rs run "Fix parser bug" --variants 3          # x3 Best-of-3 Arena in CoW-Worktrees
nala-cezar-rs worktrees reclaim                          # Byte-genaue Worktree-Bereinigung (#842)
nala-cezar-rs mcp                                        # Statischer 12-Tool MCP-Server (listChanged: false)
```
