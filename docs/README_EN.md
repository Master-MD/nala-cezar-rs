# 🇬🇧 `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — English Reference Guide

> **100% Native Safe-Rust (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` External Crates, Zero-Node.js, Zero-Python, Zero-Telemetry) Parallel AI Coding Agent Orchestrator**  
> Official fork and clean-room evolution of [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`MIT` © Patryk Lewczuk).

---

## 🚀 Architectural Summary

`NALA-Cezar-RS` compiles the entire parallel coding-agent cockpit, `.ai/cezar/` plain-file run store, Copy-on-Write worktree manager, Best-of-N tournament diff scorer, 4-forge adapter (`GitHub`, `GitLab`, `Forgejo`, `Gitea`), 12-tool MCP server, and 5-language web UI into **one `541 KB` standalone executable** consuming **`2.01 MiB` peak RSS RAM**.

### Key Capabilities
1. **13 Coding Agent Runners**: `claude`, `codex`, `opencode`, `cursor`, `pi`, `junie`, `copilot`, `omp`, `antigravity` (`agy`), `gemini`, `kimi`, `zcode`, and `local-openai` (`Ollama`, `llama-server`, `vLLM`).
2. **Copy-on-Write Worktrees + Byte-Aware Reaper (`#842`)**: Instant APFS `clonefile` / Linux `reflink` snapshots plus strict byte-quota pruning (`4 GiB` default cap).
3. **Compile-Time RAII `RunLeaseGuard` (`#798`, `#2289`, `#2296`)**: Eliminates stale `proper-lockfile` locks and stuck `waiting` / `monitoring` graph states.
4. **`sHeriFF + NVIDIA SkillSpector` 7-Layer Hardening (`SEC-CEZAR-01..07`)**: Zero telemetry (`stats.openmercato.cloud` removed), SSRF blockade, path-traversal protection, Trojan-Source BiDi Unicode (`CVE-2021-42574`) rejection, and unified-diff PR auditing (`nala-cezar-rs sync-gate --audit-diff`).

---

## 📦 Installation & Verification

```bash
# macOS & Linux (verifies FIPS 180-4 SHA-256 digest automatically)
curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash

# Ubuntu / Debian native package
sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb

# Windows 10/11 PowerShell
irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex
```

Run `nala-cezar-rs --help` or `nala-cezar-rs bench` to verify your installation.
