#![forbid(unsafe_code)]
//! `NALA-Cezar-RS` (`v0.21.0-nala.6.6.6`) — 100% Native Safe-Rust Parallel AI Coding Agent
//! Orchestrator, Copy-on-Write Git-Worktree Arena, Best-of-N Diff Tournament Scorer,
//! Pluggable 4-Forge Adapter (GitHub/GitLab/Forgejo/Gitea), Prompt-Cache-Stable MCP Server & Live Cockpit.
//!
//! Clean-room MIT rewrite & architectural evolution of `open-mercato/cezar` (`v0.21.0`,
//! Copyright (c) 2026 Patryk Lewczuk). Zero Node.js, Zero Python, Zero External Crates, Zero Outbound Telemetry.

pub mod bench;
pub mod forge;
pub mod mcp;
pub mod runners;
pub mod security;
pub mod server;
pub mod store;
pub mod sync_gate;
pub mod ui_html;
pub mod workflow;
pub mod worktree;

use crate::bench::{format_bench_json, format_bench_table, run_all_benchmarks};
use crate::forge::{detect_repo_forge, parse_forge_remote};
use crate::mcp::run_mcp_stdio_loop;
use crate::runners::{execute_check_command, probe_all_backends, ReasoningEffort, RunnerId};
use crate::security::VERSION;
use crate::server::start_cockpit_server;
use crate::store::{CezarStore, RunLeaseRegistry, RunRecord, RunStatus};
use crate::sync_gate::{
    audit_inbound_collaborator_tree, audit_outbound_public_tree, audit_unified_diff,
    generate_slsa_sbom_json,
};
use crate::workflow::load_repo_workflows;
use crate::worktree::{
    create_isolated_worktree, reclaim_worktrees_byte_aware, scan_worktrees, DEFAULT_KEEP_WORKTREES,
    DEFAULT_MAX_WORKTREE_BYTES,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const HELP_TEXT: &str = "\
NALA-Cezar-RS (cez-rs) v0.21.0-nala.6.6.6 — 100% Native Safe-Rust Parallel AI Coding Agent Orchestrator
Drop-in Zero-Node.js / Zero-Python / Zero-Telemetry Evolution of open-mercato/cezar (MIT)

USAGE:
  nala-cezar-rs serve [--host 127.0.0.1] [--port 18740]   Start Live Web Cockpit + HTTP/SSE API
  nala-cezar-rs run \"<task>\" [options]                    Dispatch task in isolated CoW git worktree
      --runner <claude|codex|antigravity|opencode|zcode|gemini|local-openai|...>
      --model <model_id>
      --effort <low|medium|high|xhigh>                    (#808 per-step reasoning effort)
      --variants <1|2|3>                                  Run x2/x3 Best-of-N parallel worktrees
      --check \"<command>\"                                 Verification command inside worktree
  nala-cezar-rs task <list|events> [<run_id>]             Inspect .ai/cezar/runs records & NDJSON events
  nala-cezar-rs backends                                  Probe all 13 coding agent CLIs (< 50 us)
  nala-cezar-rs workflows                                 List built-in & .ai/cezar/workflows/*.yaml chains
  nala-cezar-rs worktrees <list|reclaim> [--dry-run]      Byte-aware worktree retention reaper (#842)
  nala-cezar-rs forge [<remote_url>]                      Detect GitHub/GitLab/Forgejo/Gitea adapter (#847)
  nala-cezar-rs sync-gate [--direction to-public|from-public] [--audit-diff <patch>]
  nala-cezar-rs provenance <SHA256SUMS.txt>               Generate SLSA v1.0 + CycloneDX SBOM attestation
  nala-cezar-rs bench [--json]                            Run 10-subsystem hardware benchmark suite
  nala-cezar-rs mcp                                       Start 12-tool static Prompt-Cache-Stable MCP server
  nala-cezar-rs --version | --help
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if args.is_empty() {
        println!("{HELP_TEXT}");
        return ExitCode::SUCCESS;
    }

    match args[0].as_str() {
        "-v" | "--version" | "version" => {
            println!("nala-cezar-rs {VERSION} (100% Safe Rust Edition 2024, 0 crates, 0 telemetry)");
            ExitCode::SUCCESS
        }
        "-h" | "--help" | "help" => {
            println!("{HELP_TEXT}");
            ExitCode::SUCCESS
        }
        "serve" | "cockpit" => {
            let mut host = String::from("127.0.0.1");
            let mut port: u16 = 18740;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--host" if i + 1 < args.len() => {
                        host = args[i + 1].clone();
                        i += 2;
                    }
                    "--port" | "-p" if i + 1 < args.len() => {
                        port = args[i + 1].parse().unwrap_or(18740);
                        i += 2;
                    }
                    _ => i += 1,
                }
            }
            match start_cockpit_server(&repo_root, &host, port) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("[ERROR] {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "backends" | "doctor" => {
            for b in probe_all_backends() {
                println!("{}", b.to_json());
            }
            ExitCode::SUCCESS
        }
        "workflows" => {
            for wf in load_repo_workflows(&repo_root) {
                println!("{}", wf.to_json());
            }
            ExitCode::SUCCESS
        }
        "forge" => {
            let res = if args.len() > 1 {
                parse_forge_remote(&args[1], None)
            } else {
                detect_repo_forge(&repo_root)
            };
            match res {
                Some(f) => {
                    println!("{}", f.to_json());
                    ExitCode::SUCCESS
                }
                None => {
                    println!("{{\"forge\":null}}");
                    ExitCode::SUCCESS
                }
            }
        }
        "worktrees" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            if sub == "reclaim" {
                let dry = args.iter().any(|a| a == "--dry-run");
                let rep = reclaim_worktrees_byte_aware(
                    &repo_root,
                    DEFAULT_KEEP_WORKTREES,
                    DEFAULT_MAX_WORKTREE_BYTES,
                    dry,
                );
                println!("{}", rep.to_json());
            } else {
                for wt in scan_worktrees(&repo_root) {
                    println!("{}", wt.to_json());
                }
            }
            ExitCode::SUCCESS
        }
        "task" | "runs" => {
            let store = CezarStore::new(&repo_root);
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
            if sub == "events" && let Some(rid) = args.get(2) {
                match store.read_events(rid, 0) {
                    Ok(ev) => {
                        for l in ev {
                            println!("{l}");
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("[ERROR] {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                for j in store.list_run_jsons() {
                    println!("{j}");
                }
                ExitCode::SUCCESS
            }
        }
        "run" => {
            if args.len() < 2 {
                eprintln!("Usage: nala-cezar-rs run \"<task>\" [--runner codex] [--variants 1|2|3] [--check \"<cmd>\"]");
                return ExitCode::FAILURE;
            }
            let prompt = args[1].clone();
            let mut runner = RunnerId::Codex;
            let mut model = String::from("gpt-5.3-codex");
            let mut effort = ReasoningEffort::High;
            let mut variants: usize = 1;
            let mut check_cmd: Option<String> = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--runner" if i + 1 < args.len() => {
                        if let Some(r) = RunnerId::from_str_loose(&args[i + 1]) {
                            runner = r;
                            model = r.default_models().first().copied().unwrap_or("default").to_string();
                        }
                        i += 2;
                    }
                    "--model" if i + 1 < args.len() => {
                        model = args[i + 1].clone();
                        i += 2;
                    }
                    "--effort" if i + 1 < args.len() => {
                        effort = ReasoningEffort::from_str_loose(&args[i + 1]);
                        i += 2;
                    }
                    "--variants" if i + 1 < args.len() => {
                        variants = args[i + 1].parse::<usize>().unwrap_or(1).clamp(1, 3);
                        i += 2;
                    }
                    "--check" if i + 1 < args.len() => {
                        check_cmd = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            let store = CezarStore::new(&repo_root);
            let leases = RunLeaseRegistry::new();
            let base_id = format!("run-{}", RunRecord::now_ms() % 1_000_000);

            for v in 1..=variants {
                let run_id = if variants == 1 {
                    base_id.clone()
                } else {
                    format!("{base_id}-v{v}")
                };
                let lock_path = store.runs_dir().join(&run_id).join(".lease.lock");
                let Ok(_lease) = leases.acquire(&run_id, Some(lock_path)) else {
                    eprintln!("[ERROR] run {run_id} already leased");
                    return ExitCode::FAILURE;
                };
                let mut rec = RunRecord::new(&run_id, &prompt, runner.as_str(), &model, "implement-and-verify");
                if variants > 1 {
                    rec.group_id = Some(base_id.clone());
                }
                rec.status = RunStatus::Running;
                if let Err(e) = store.save_run(&rec) {
                    eprintln!("[ERROR] {e}");
                    return ExitCode::FAILURE;
                }
                let wt = match create_isolated_worktree(&repo_root, &run_id, "HEAD") {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("[ERROR] worktree {run_id}: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let _ = store.append_event(
                    &run_id,
                    1,
                    "worktree",
                    "worktree_created",
                    &format!("effort={} dir={}", effort.as_str(), wt.display()),
                    Some(0),
                    Some(1),
                );
                if let Some(ref cmd) = check_cmd {
                    match execute_check_command(&wt, cmd) {
                        Ok(res) => {
                            rec.tokens_total += res.estimated_tokens;
                            rec.status = if res.exit_code == 0 {
                                RunStatus::Review
                            } else {
                                RunStatus::Failed
                            };
                            let _ = store.append_event(
                                &run_id,
                                2,
                                "check",
                                if res.exit_code == 0 { "check_passed" } else { "check_failed" },
                                cmd,
                                Some(res.first_token_ms),
                                Some(res.generation_end_ms),
                            );
                        }
                        Err(e) => {
                            eprintln!("[SECURITY BLOCKED] {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                } else {
                    rec.status = RunStatus::Review;
                }
                let _ = store.save_run(&rec);
                println!("{}", rec.to_json());
            }
            ExitCode::SUCCESS
        }
        "sync-gate" => {
            let mut direction = "to-public";
            let mut diff_path: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--direction" if i + 1 < args.len() => {
                        direction = args[i + 1].as_str();
                        i += 2;
                    }
                    "--audit-diff" if i + 1 < args.len() => {
                        diff_path = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => i += 1,
                }
            }
            if let Some(p) = diff_path {
                match fs::read_to_string(&p) {
                    Ok(txt) => {
                        let rep = audit_unified_diff(&txt);
                        println!("{}", rep.to_json());
                        return if rep.passed { ExitCode::SUCCESS } else { ExitCode::FAILURE };
                    }
                    Err(e) => {
                        eprintln!("[ERROR] read diff {p}: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            let res = if direction == "from-public" {
                audit_inbound_collaborator_tree(&repo_root)
            } else {
                audit_outbound_public_tree(&repo_root)
            };
            match res {
                Ok(rep) => {
                    println!("{}", rep.to_json());
                    if rep.passed {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "provenance" => {
            let Some(sums_file) = args.get(1) else {
                eprintln!("Usage: nala-cezar-rs provenance <SHA256SUMS.txt>");
                return ExitCode::FAILURE;
            };
            match fs::read_to_string(sums_file) {
                Ok(txt) => {
                    println!("{}", generate_slsa_sbom_json(&txt));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[ERROR] {e}");
                    ExitCode::FAILURE
                }
            }
        }
        "bench" => {
            let rows = run_all_benchmarks();
            if args.iter().any(|a| a == "--json") {
                println!("{}", format_bench_json(&rows));
            } else {
                print!("{}", format_bench_table(&rows));
            }
            ExitCode::SUCCESS
        }
        "mcp" => match run_mcp_stdio_loop(&repo_root) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[ERROR] MCP loop: {e}");
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("[ERROR] Unknown command `{other}`. Run `nala-cezar-rs --help`.");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::ForgeKind;
    use crate::mcp::handle_mcp_jsonrpc_line;
    use crate::security::{
        audit_skill_or_command_payload, constant_time_eq, hex_encode, hmac_sha256, sanitize_html,
        sha256, validate_outbound_webhook_url, validate_safe_repo_relpath, LoopbackRateLimiter,
    };
    use crate::store::{compute_usage_mix_json, ZeroAllocSseFrame, BINARY_INDEX_MAGIC, BINARY_RECORD_SIZE};
    use crate::workflow::{
        parse_workflow_yaml, score_variant_candidates, validate_child_dispatch, VariantCandidate,
    };
    use std::path::Path;

    #[test]
    fn t01_fips_sha256_and_hmac_vectors() {
        let digest = hex_encode(&sha256(b"abc"));
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let mac = hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            hex_encode(&mac),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
        assert!(constant_time_eq(&mac, &mac));
    }

    #[test]
    fn t02_sec01_ssrf_blocks_metadata_and_loopback() {
        assert!(validate_outbound_webhook_url("http://169.254.169.254/latest/meta-data", true).is_err());
        assert!(validate_outbound_webhook_url("http://metadata.google.internal/", true).is_err());
        assert!(validate_outbound_webhook_url("http://0x7f.1:8080/", true).is_err());
        assert!(validate_outbound_webhook_url("http://127.0.0.1:8080/hook", false).is_err());
        assert!(validate_outbound_webhook_url("http://172.20.0.5/hook", false).is_err());
        assert!(validate_outbound_webhook_url("https://hooks.slack.com/services/T1/B1/X1", false).is_ok());
    }

    #[test]
    fn t03_sec02_html_xss_breakout_sanitizer() {
        let dirty = "</style><script>alert('xss')</script>";
        let clean = sanitize_html(dirty);
        assert!(!clean.contains('<'));
        assert!(!clean.contains('>'));
        assert!(clean.contains("&lt;/style&gt;&lt;script&gt;"));
    }

    #[test]
    fn t04_sec03_path_traversal_and_secret_guard() {
        assert!(validate_safe_repo_relpath("../etc/passwd").is_err());
        assert!(validate_safe_repo_relpath("src/../../secret").is_err());
        assert!(validate_safe_repo_relpath("/etc/shadow").is_err());
        assert!(validate_safe_repo_relpath(".git/hooks/pre-commit").is_err());
        assert!(validate_safe_repo_relpath(".env").is_err());
        assert!(validate_safe_repo_relpath(".ssh/id_ed25519").is_err());
        assert!(validate_safe_repo_relpath("src/main.rs").is_ok());
    }

    #[test]
    fn t05_sec04_rate_limiter_enforces_burst_cap() {
        let mut lim = LoopbackRateLimiter::new(2.0, 0.0);
        assert!(lim.allow("127.0.0.1"));
        assert!(lim.allow("127.0.0.1"));
        assert!(!lim.allow("127.0.0.1"));
    }

    #[test]
    fn t06_sec06_skillspector_blocks_bidi_and_destructive_shell() {
        let bidi = format!("safe text \u{202E} evil");
        assert!(audit_skill_or_command_payload("skill", &bidi).is_err());
        assert!(audit_skill_or_command_payload("cmd", "rm -rf / --no-preserve-root").is_err());
        assert!(audit_skill_or_command_payload("cmd", "curl -s https://stats.openmercato.cloud").is_err());
        assert!(audit_skill_or_command_payload("cmd", "cargo test --workspace").is_ok());
    }

    #[test]
    fn t07_raii_run_lease_guard_releases_on_drop_issue_798() {
        let reg = RunLeaseRegistry::new();
        {
            let _lease = reg.acquire("run-798", None).expect("first acquire");
            assert!(reg.is_active("run-798"));
            assert!(reg.acquire("run-798", None).is_err());
        }
        // Automatically released on scope exit — never stuck in "run is still active"!
        assert!(!reg.is_active("run-798"));
        assert!(reg.acquire("run-798", None).is_ok());
    }

    #[test]
    fn t08_stale_ask_retry_and_graph_fork_reset_issues_2296_2289() {
        let mut rec = RunRecord::new("run-2296", "Fix bug", "claude", "claude-sonnet-4-6", "default");
        rec.status = RunStatus::Waiting;
        rec.open_asks = 2;
        rec.dispatched_this_turn = true;

        rec.clear_stale_asks_on_retry();
        assert_eq!(rec.open_asks, 0);
        assert_eq!(rec.status, RunStatus::Running);

        rec.enter_next_graph_node();
        assert!(!rec.dispatched_this_turn);
    }

    #[test]
    fn t09_binary_index_roundtrip_and_zero_alloc_sse() {
        let mut rec = RunRecord::new("run-fast-01", "Fast index", "codex", "gpt-5.3-codex", "default");
        rec.tokens_total = 42_500;
        rec.cost_usd_micros = 125_000;
        rec.status = RunStatus::Done;

        let mut buf = Vec::with_capacity(16 + BINARY_RECORD_SIZE);
        buf.extend_from_slice(BINARY_INDEX_MAGIC);
        buf.extend_from_slice(&1u64.to_le_bytes());
        buf.extend_from_slice(&rec.to_binary_slot());

        let found = CezarStore::lookup_in_binary_index(&buf, "run-fast-01").expect("lookup");
        assert_eq!(found, (RunStatus::Done, 42_500, 125_000));

        let sse = ZeroAllocSseFrame::format_run_event(7, "tool_call", "s1", "ok");
        let s = std::str::from_utf8(sse.as_bytes()).unwrap();
        assert!(s.starts_with("id: 7\nevent: run_event\ndata: {\"seq\":7,"));
        assert!(s.ends_with("}\n\n"));
    }

    #[test]
    fn t10_multi_forge_adapter_github_gitlab_forgejo_gitea_issue_847() {
        let gh = parse_forge_remote("https://github.com/Master-MD/nala-cezar-rs.git", None).unwrap();
        assert_eq!(gh.kind, ForgeKind::GitHub);
        assert_eq!(gh.owner, "Master-MD");
        assert_eq!(gh.repo, "nala-cezar-rs");

        let gl = parse_forge_remote("git@gitlab.com:group/sub/project.git", None).unwrap();
        assert_eq!(gl.kind, ForgeKind::GitLab);
        assert_eq!(gl.repo, "sub/project");

        let fj = parse_forge_remote("https://git.example.org/devops/core.git", None).unwrap();
        assert_eq!(fj.kind, ForgeKind::Forgejo);
        assert_eq!(fj.cli_tool, "tea");

        let gt = parse_forge_remote("https://gitea.example.org/team/service.git", None).unwrap();
        assert_eq!(gt.kind, ForgeKind::Gitea);
    }

    #[test]
    fn t11_portable_yaml_workflow_parser_and_child_dispatch_cap() {
        let yaml = "\
name: review-chain
description: Skill stack + test gate
steps:
  - id: impl
    runner: antigravity
    reasoningEffort: xhigh
    prompt: \"Build module\"
  - id: test
    command: \"cargo test\"
    retry: impl
";
        let wf = parse_workflow_yaml("review-chain", yaml).expect("parse yaml");
        assert_eq!(wf.steps.len(), 2);
        assert_eq!(wf.steps[0].runner, Some(RunnerId::Antigravity));
        assert_eq!(wf.steps[0].reasoning_effort, ReasoningEffort::XHigh);
        assert!(wf.steps[1].on_fail.is_some());

        assert!(validate_child_dispatch(3, 1_000_000, 500_000).is_ok());
        assert!(validate_child_dispatch(4, 1_000_000, 500_000).is_err());
        assert!(validate_child_dispatch(1, 500_000, 500_000).is_err());
    }

    #[test]
    fn t12_best_of_3_tournament_ranks_safe_tested_diff_first() {
        let cands = vec![
            VariantCandidate {
                variant_id: "v-unsafe".to_string(),
                runner: "local-openai".to_string(),
                model: "qwen".to_string(),
                check_exit_code: 0,
                unified_diff: "+++ b/src/lib.rs\n+unsafe { }\n".to_string(),
                duration_ms: 500,
                cost_usd_micros: 0,
            },
            VariantCandidate {
                variant_id: "v-winner".to_string(),
                runner: "codex".to_string(),
                model: "gpt-5.3-codex".to_string(),
                check_exit_code: 0,
                unified_diff: "+++ b/src/lib.rs\n+pub fn ok() -> bool { true }\n+#[test]\n+fn test_ok() { assert!(ok()); }\n".to_string(),
                duration_ms: 1000,
                cost_usd_micros: 1000,
            },
        ];
        let ranked = score_variant_candidates(&cands);
        assert_eq!(ranked[0].variant_id, "v-winner");
        assert!(ranked[0].sheriff_clean);
        assert!(!ranked[1].sheriff_clean);
    }

    #[test]
    fn t13_sync_gate_diff_auditor_blocks_forbid_unsafe_removal() {
        let evil_diff = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,2 +1,2 @@
-#![forbid(unsafe_code)]
+pub fn sneak() {}
";
        let rep = audit_unified_diff(evil_diff);
        assert!(!rep.passed);

        let clean_diff = "\
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,1 +1,2 @@
+pub const READY: bool = true;
";
        let ok_rep = audit_unified_diff(clean_diff);
        assert!(ok_rep.passed);
    }

    #[test]
    fn t14_mcp_prompt_cache_stable_tools_list_and_usage_mix() {
        let init = handle_mcp_jsonrpc_line(
            Path::new("."),
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
        )
        .unwrap();
        assert!(init.contains("\"listChanged\":false"));

        let list1 = handle_mcp_jsonrpc_line(
            Path::new("."),
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        )
        .unwrap();
        let list2 = handle_mcp_jsonrpc_line(
            Path::new("."),
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        )
        .unwrap();
        assert_eq!(list1, list2);
        assert!(list1.contains("cezar_dispatch_variants"));

        let mut r = RunRecord::new("r1", "t", "codex", "gpt-5.3-codex", "w");
        r.tokens_total = 1000;
        let mix = compute_usage_mix_json(&[r]);
        assert!(mix.contains("\"tokens\":1000"));
    }

    #[test]
    fn t15_outbound_and_inbound_self_audit_pass_on_own_repo() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let out_rep = audit_outbound_public_tree(root).expect("outbound scan");
        assert!(out_rep.passed, "Outbound findings: {:?}", out_rep.findings);
        let in_rep = audit_inbound_collaborator_tree(root).expect("inbound scan");
        assert!(in_rep.passed, "Inbound findings: {:?}", in_rep.findings);
    }
}
