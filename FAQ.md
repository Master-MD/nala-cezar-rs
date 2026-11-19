# ❓ Frequently Asked Questions (`FAQ.md`)

## 1. Is `NALA-Cezar-RS` compatible with existing `.ai/cezar/` repositories?
**Yes.** `NALA-Cezar-RS` reads and writes plain `.ai/cezar/runs/<id>/run.json`, append-only `events.ndjson` (Normalized Agent-Event Protocol v2), `.ai/skills/*.md`, and `.ai/cezar/workflows/*.yaml` chains, while adding an optional `64-byte` aligned `.cezar.idx` binary accelerator for `80.6 ns` $O(1)$ lookups.

## 2. Does `NALA-Cezar-RS` phone home or send telemetry?
**Never (`0%` outbound telemetry).** All upstream `stats.openmercato.cloud` endpoints, daily inventory pings, and cloud relay modules have been completely removed. `NALA-Cezar-RS` runs 100% offline / airgapped (`env -i` or `docker run --network none`).

## 3. Why are there `0` external crates in `Cargo.toml`?
Zero external dependencies (`#![forbid(unsafe_code)]`, 100% Rust `std`) eliminate supply-chain attacks (`build.rs` hooks, typosquatted crates, npm lockfile drift) and allow deterministic sub-4-second compilation across macOS (`arm64`/`x86_64`), Linux (`musl` static `amd64`/`arm64`), and Windows (`x86_64`).

## 4. Which AI coding agents are supported?
All **13 backends**: `claude`, `codex`, `opencode`, `cursor`, `pi`, `junie`, `copilot`, `omp`, `antigravity` (`agy`), `gemini`, `kimi`, `zcode`, and `local-openai` (`Ollama`, `llama-server`, `vLLM`).

## 5. How do I register `NALA-Cezar-RS` as an MCP Server in Claude Code, Codex, Cursor, or Antigravity?
Add the static 12-tool stdio server (`listChanged: false` — never invalidates your LLM prompt cache!):
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

## 6. How does `NALA-Cezar-RS` prevent the 17 GB worktree buildup (Upstream `#842`)?
1. Uses Copy-on-Write (`cp -c` APFS `clonefile` on macOS, `cp --reflink=auto` on Linux Btrfs/XFS/ZFS) and skips duplicating `target/` and `node_modules/`.
2. `nala-cezar-rs worktrees reclaim` enforces **both** `keep_count` (`10`) **and** `max_bytes` (`4 GiB` default), while never deleting dirty or actively leased worktrees.
