# 🪨 Caveman Guide — `NALA-Cezar-RS` in 60 Seconds

Old tool (`open-mercato/cezar`):
- Needs heavy Node.js 20+ (`~100 MB` runtime + `~250 MB` RAM).
- `10` git worktrees eat `17 GB` disk (`#842`).
- Lockfile gets stuck `"run is still active"` (`#798`).
- Only speaks GitHub (`#847`).

New tool (**`NALA-Cezar-RS`**):
- **One tiny `~540 KB` Rust file.** Uses **`2.0 MB` RAM**. Zero Node.js. Zero Python. Zero Telemetry.
- **Copy-on-Write worktrees + Byte Reaper**: Disk stays clean.
- **Speaks GitHub, GitLab, Forgejo, and Gitea.**
- **Runs `x3` agents at once** (`Codex` + `Claude` + `Local LLM`) and ranks the safest, cleanest diff first!

---

## 1. Install (1 Command)

**macOS & Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.sh | bash
```

**Ubuntu / Debian (`.deb`):**
```bash
sudo dpkg -i nala-cezar-rs_0.21.0_amd64.deb
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Master-MD/nala-cezar-rs/main/install.ps1 | iex
```

*(Or mount `NALA-Cezar-RS-0.21.0-macos-arm64.dmg` on macOS / run `nala-cezar-rs-windows-x86_64.exe` on Windows!)*

---

## 2. Use It (4 Commands)

```bash
# 1. Start the Live Browser Cockpit on http://127.0.0.1:18740
nala-cezar-rs serve

# 2. Check which of the 13 AI coding agents are installed (< 140 microseconds)
nala-cezar-rs backends

# 3. Run a task x3 in parallel isolated Copy-on-Write worktrees + run tests
nala-cezar-rs run "Refactor parser with unit tests" --runner codex --variants 3 --check "cargo test"

# 4. Run live hardware benchmark vs Node.js upstream
nala-cezar-rs bench
```
