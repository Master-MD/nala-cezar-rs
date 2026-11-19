# Upstream Attribution — `open-mercato/cezar`

`NALA-Cezar-RS` (`https://github.com/Master-MD/nala-cezar-rs`) is an official GitHub fork and clean-room **100% Native Safe-Rust (`Edition 2024`, `#![forbid(unsafe_code)]`)** re-engineering of:

- **Upstream Project**: [`open-mercato/cezar`](https://github.com/open-mercato/cezar) (`v0.21.0`)
- **Original Author**: Patryk Lewczuk (`@open-mercato`)
- **Upstream License**: MIT License (`Copyright (c) 2026 Patryk Lewczuk`)

## Architectural Credit & Divergences

We gratefully credit `open-mercato/cezar` for pioneering:
1. The plain-file `.ai/cezar/` workspace layout (`runs/<id>/run.json`, append-only `events.ndjson`).
2. Portable Markdown skills (`.ai/skills/*.md`) and short YAML agent chains (`.ai/cezar/workflows/*.yaml`).
3. Parallel git-worktree isolation and Best-of-N (`x2` / `x3`) variant diff comparison without auto-merging.

`NALA-Cezar-RS` replaces the ~198 MB TypeScript/Node.js monorepo (`4,395` files, Node 20+ sidecar runtime downloader, Hono/Zod/proper-lockfile stack, and `stats.openmercato.cloud` telemetry) with a single **`~540 KB` Pure Safe-Rust binary (`2.01 MiB` peak RSS RAM, `0` external crates, `0%` outbound telemetry)** and native multi-forge support (`GitHub`, `GitLab`, `Forgejo`, `Gitea`).
