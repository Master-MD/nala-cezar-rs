# 🇫🇷 `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — Documentation en Français

> **Orchestrateur parallèle d'agents IA de codage 100 % Rust natif sûr (`Edition 2024`, `#![forbid(unsafe_code)]`, `0` crate externe, zéro Node.js, zéro Python, zéro télémétrie)**  
> Fork officiel et réécriture clean-room de [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`MIT` © Patryk Lewczuk).

---

## 🚀 Performances mesurées (Apple M4 Max)

`NALA-Cezar-RS` remplace le monorepo Node.js/TypeScript (~198 Mo) par **un seul binaire Rust de `541 Ko`** ne consommant que **`2.01 Mio` de RAM** :

- **Recherche binaire `.cezar.idx` (`B01`)** : **`80.6 ns`** contre `1.85 ms` (**`22 960x` plus rapide**)
- **Verrou RAII `RunLeaseGuard` (`B02`, résout `#798`)** : **`28.6 ns`** contre `420 µs` (**`14 674x` plus rapide**)
- **Flux SSE sur pile `[u8; 512]` (`B03`, zéro allocation)** : **`155.0 ns`** (**`91.6x` plus rapide**)
- **Détection des 13 CLIs d'agents (`B07`)** : **`138.06 µs`** contre `48.00 ms` (**`347.7x` plus rapide**)
- **Adaptateur Multi-Forge natif (`#847`)** : **GitHub, GitLab, Forgejo et Gitea** pris en charge nativement.
- **Worktrees Copy-on-Write + Quota en octets (`#842`)** : Fini les `17 Go` perdus sur disque.

---

## 📦 Installation rapide

- **macOS & Linux** : `curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash`
- **macOS `.dmg`** : Monter `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` ou `NALA-Cezar-RS-0.21.0-macos-x86_64.dmg`
- **Ubuntu / Debian `.deb`** : `sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb`
- **Windows 10/11 `.exe`** : `irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex`

---

## 🛡️ Sécurité `sHeriFF + SkillSpector`

Zéro télémétrie sortante (`stats.openmercato.cloud` supprimé), `#![forbid(unsafe_code)]`, protection SSRF, anti-traversée de chemin (`..`), blocage Unicode BiDi Trojan-Source (`CVE-2021-42574`), audit de diff PR (`nala-cezar-rs sync-gate --audit-diff`) et serveur MCP à 12 outils statiques (`nala-cezar-rs mcp`).
