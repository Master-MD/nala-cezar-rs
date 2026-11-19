# 🇮🇹 `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — Documentazione in Italiano

> **Orchestratore parallelo di agenti IA di programmazione 100% Rust nativo sicuro (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` crate esterni, zero Node.js, zero Python, zero telemetria)**  
> Fork ufficiale e reimplementazione clean-room di [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`MIT` © Patryk Lewczuk).

---

## 🚀 Prestazioni Misurate

`NALA-Cezar-RS` sostituisce l'intero stack Node.js 20+ (~100 MB runtime) con **un singolo eseguibile Rust da `541 KB`** e soli **`2.01 MiB` di RAM RSS**:

- **Indice binario `.cezar.idx` (`B01`)**: **`80.6 ns`** vs `1.85 ms` (**`22.960x` più veloce**)
- **RAII `RunLeaseGuard` (`B02`, risolve `#798`)**: **`28.6 ns`** vs `420 µs` (**`14.674x` più veloce**)
- **Streaming SSE su stack `[u8; 512]` (`B03`)**: **`155.0 ns`** (**`91.6x` più veloce**)
- **Torneo Best-of-3 Diff + Audit (`B05`)**: **`2.68 µs`** (**`115.5x` più veloce**)
- **Multi-Forge nativo (`#847`)**: Supporto integrato per **GitHub, GitLab, Forgejo e Gitea**.
- **Pulizia Worktree per byte (`#842`)**: Copy-on-Write (`APFS` / `reflink`) + limite automatico in byte.

---

## 📦 Installazione

- **macOS & Linux**: `curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash`
- **macOS `.dmg`**: `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` / `macos-x86_64.dmg`
- **Ubuntu / Debian `.deb`**: `sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb`
- **Windows 10/11 `.exe`**: `irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex`

---

## 🛡️ Sicurezza `sHeriFF + SkillSpector`

Zero telemetria cloud, zero dipendenze esterne, `#![forbid(unsafe_code)]`, 7 livelli di difesa (`SEC-CEZAR-01..07`) contro SSRF, XSS, Path Traversal, comandi distruttivi e Trojan-Source BiDi (`CVE-2021-42574`).
