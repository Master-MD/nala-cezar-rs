# 🏔️ `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — Documentaziun en Rumantsch Grischun

> **Orchestratur parallel d'agents d'IA da programmaziun 100% Rust nativ segir (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` crates externs, nagin Node.js, nagin Python, nagina telemetria)**  
> Fork uffizial e svilup clean-room da [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`MIT` © Patryk Lewczuk).

---

## 🚀 Prestaziun mesirada

`NALA-Cezar-RS` remplazza il pachet Node.js/TypeScript (~198 MB) tras **ina suletta datoteca binara Rust da `541 KB`** cun mo **`2.01 MiB` RAM**:

- **Index binar `.cezar.idx` (`B01`)**: **`80.6 ns`** enstagl da `1.85 ms` (**`22'960x` pli svelt**)
- **RAII `RunLeaseGuard` (`B02`, schlia `#798`)**: **`28.6 ns`** (**`14'674x` pli svelt**)
- **Stack SSE `[u8; 512]` (`B03`)**: **`155.0 ns`** (**`91.6x` pli svelt**)
- **13 Agents & 4 Forges (`#847`)**: Sustegna `claude`, `codex`, `antigravity`, `opencode`, `zcode`, `gemini`, `local-openai` sco er **GitHub, GitLab, Forgejo e Gitea**.
- **Worktrees Copy-on-Write (`#842`)**: Controlla automatica da la plazza sin il disc.

---

## 📦 Installaziun

- **macOS & Linux**: `curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash`
- **macOS `.dmg`**: `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` / `macos-x86_64.dmg`
- **Ubuntu / Debian `.deb`**: `sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb`
- **Windows 10/11 `.exe`**: `irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex`

---

## 🛡️ Segirezza `sHeriFF`

Nagina telemetria externa (`0%`), `#![forbid(unsafe_code)]`, 7 portas da segirezza (`SEC-CEZAR-01..07`) ed MCP-Server cun 12 utensils statics (`nala-cezar-rs mcp`).
