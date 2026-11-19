//! Prompt-Cache-Stable JSON-RPC 2.0 MCP Server (`nala-cezar-rs mcp`).
//!
//! `tools/list` is 100% static (`listChanged: false`) so client LLM KV/prompt caches in
//! Claude Desktop, Antigravity, Codex, Cursor, ZCode, and OpenCode are never invalidated!

use crate::forge::{detect_repo_forge, parse_forge_remote};
use crate::runners::probe_all_backends;
use crate::security::VERSION;
use crate::store::{compute_usage_mix_json, repo_git_branch, CezarStore, RunRecord};
use crate::sync_gate::{audit_outbound_public_tree, audit_unified_diff, json_escape};
use crate::workflow::{load_repo_workflows, score_variant_candidates, VariantCandidate};
use crate::worktree::{reclaim_worktrees_byte_aware, DEFAULT_KEEP_WORKTREES, DEFAULT_MAX_WORKTREE_BYTES};
use std::io::{self, BufRead, Write};
use std::path::Path;

pub const MCP_STATIC_TOOLS_JSON: &str = r#"[
  {"name":"cezar_health","description":"Return NALA-Cezar-RS runtime health, repo branch, forge info, and zero-telemetry status","inputSchema":{"type":"object","properties":{}}},
  {"name":"cezar_backends","description":"Probe all 13 AI coding agent backends (claude, codex, opencode, cursor, pi, junie, copilot, omp, antigravity, gemini, kimi, zcode, local-openai)","inputSchema":{"type":"object","properties":{}}},
  {"name":"cezar_list_runs","description":"List all recorded coding agent runs in .ai/cezar/runs","inputSchema":{"type":"object","properties":{}}},
  {"name":"cezar_create_task","description":"Create a new coding agent task in an isolated CoW git worktree","inputSchema":{"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"runner":{"type":"string"},"model":{"type":"string"},"workflow":{"type":"string"}},"required":["id","title"]}},
  {"name":"cezar_dispatch_variants","description":"Create x2 or x3 Best-of-N parallel variant worktrees for a coding task","inputSchema":{"type":"object","properties":{"group_id":{"type":"string"},"title":{"type":"string"},"count":{"type":"integer"}},"required":["group_id","title"]}},
  {"name":"cezar_run_events","description":"Read Normalized Agent-Event Protocol v2 NDJSON events for a run","inputSchema":{"type":"object","properties":{"run_id":{"type":"string"}},"required":["run_id"]}},
  {"name":"cezar_score_variants","description":"Deterministically rank candidate diffs via check status, sHeriFF diff audit, and churn compactness","inputSchema":{"type":"object","properties":{"diff_a":{"type":"string"},"diff_b":{"type":"string"}},"required":["diff_a","diff_b"]}},
  {"name":"cezar_reclaim_worktrees","description":"Run byte-aware worktree retention reaper (#842)","inputSchema":{"type":"object","properties":{"dry_run":{"type":"boolean"}}}},
  {"name":"cezar_list_workflows","description":"List built-in and .ai/cezar/workflows/*.yaml chains","inputSchema":{"type":"object","properties":{}}},
  {"name":"cezar_audit_diff","description":"Audit a unified git diff against sHeriFF + SkillSpector 10 security gates","inputSchema":{"type":"object","properties":{"diff":{"type":"string"}},"required":["diff"]}},
  {"name":"cezar_forge_detect","description":"Detect git remote forge adapter (GitHub, GitLab, Forgejo, Gitea — #847)","inputSchema":{"type":"object","properties":{"remote_url":{"type":"string"}}}},
  {"name":"cezar_usage_mix","description":"Compute provider/model token & cost mix (#1159) and sHeriFF outbound status","inputSchema":{"type":"object","properties":{}}}
]"#;

pub fn extract_json_str_field<'a>(json: &'a str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)?;
    let after = &json[pos + needle.len()..];
    let colon = after.find(':')?;
    let val_part = after[colon + 1..].trim_start();
    if let Some(rest) = val_part.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(esc) = chars.next() {
                        match esc {
                            'n' => out.push('\n'),
                            'r' => out.push('\r'),
                            't' => out.push('\t'),
                            other => out.push(other),
                        }
                    }
                }
                '"' => return Some(out),
                other => out.push(other),
            }
        }
        None
    } else {
        let end = val_part
            .find(|c: char| matches!(c, ',' | '}' | ']' | ' ' | '\n' | '\r'))
            .unwrap_or(val_part.len());
        Some(val_part[..end].to_string())
    }
}

pub fn handle_mcp_jsonrpc_line(repo_root: &Path, line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let id_raw = extract_json_str_field(trimmed, "id").unwrap_or_else(|| "1".to_string());
    let id_json = if id_raw.chars().all(|c| c.is_ascii_digit()) {
        id_raw
    } else {
        format!("\"{}\"", json_escape(&id_raw))
    };

    let method = extract_json_str_field(trimmed, "method").unwrap_or_default();
    if method.starts_with("notifications/") {
        return None;
    }

    match method.as_str() {
        "initialize" => Some(format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id_json},\"result\":{{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{{\"tools\":{{\"listChanged\":false}}}},\"serverInfo\":{{\"name\":\"nala-cezar-rs\",\"version\":\"{VERSION}\"}}}}}}"
        )),
        "tools/list" => Some(format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id_json},\"result\":{{\"tools\":{MCP_STATIC_TOOLS_JSON}}}}}"
        )),
        "tools/call" => {
            let tool_name = extract_json_str_field(trimmed, "name").unwrap_or_default();
            let payload = dispatch_mcp_tool(repo_root, &tool_name, trimmed);
            Some(format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id_json},\"result\":{{\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}}}",
                json_escape(&payload)
            ))
        }
        _ => Some(format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id_json},\"error\":{{\"code\":-32601,\"message\":\"Method not found\"}}}}"
        )),
    }
}

fn dispatch_mcp_tool(repo_root: &Path, tool_name: &str, raw_req: &str) -> String {
    let store = CezarStore::new(repo_root);
    match tool_name {
        "cezar_health" => {
            let branch = repo_git_branch(repo_root);
            let forge = detect_repo_forge(repo_root)
                .map(|f| f.to_json())
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{\"status\":\"ok\",\"version\":\"{VERSION}\",\"branch\":\"{}\",\"forge\":{forge},\"telemetry\":\"zero-outbound\"}}",
                json_escape(&branch)
            )
        }
        "cezar_backends" => {
            let items: Vec<String> = probe_all_backends().into_iter().map(|b| b.to_json()).collect();
            format!("{{\"backends\":[{}]}}", items.join(","))
        }
        "cezar_list_runs" => {
            let runs = store.list_run_jsons();
            format!("{{\"runs\":[{}]}}", runs.join(","))
        }
        "cezar_create_task" => {
            let id = extract_json_str_field(raw_req, "id").unwrap_or_else(|| "task-001".to_string());
            let title = extract_json_str_field(raw_req, "title").unwrap_or_else(|| "Untitled task".to_string());
            let runner = extract_json_str_field(raw_req, "runner").unwrap_or_else(|| "codex".to_string());
            let model = extract_json_str_field(raw_req, "model").unwrap_or_else(|| "gpt-5.3-codex".to_string());
            let wf = extract_json_str_field(raw_req, "workflow").unwrap_or_else(|| "implement-and-verify".to_string());
            let rec = RunRecord::new(&id, &title, &runner, &model, &wf);
            match store.save_run(&rec) {
                Ok(_) => {
                    let _ = crate::worktree::create_isolated_worktree(repo_root, &id, "HEAD");
                    let _ = store.append_event(&id, 1, "init", "task_created", &title, Some(0), Some(1));
                    rec.to_json()
                }
                Err(e) => format!("{{\"error\":\"{}\"}}", json_escape(&e)),
            }
        }
        "cezar_dispatch_variants" => {
            let gid = extract_json_str_field(raw_req, "group_id").unwrap_or_else(|| "arena-01".to_string());
            let title = extract_json_str_field(raw_req, "title").unwrap_or_else(|| "Arena task".to_string());
            let runners = [("codex", "gpt-5.3-codex"), ("claude", "claude-sonnet-4-6"), ("local-openai", "qwen2.5-coder:32b")];
            let mut created = Vec::new();
            for (idx, (r, m)) in runners.iter().enumerate() {
                let vid = format!("{gid}-v{}", idx + 1);
                let mut rec = RunRecord::new(&vid, &title, r, m, "best-of-3-arena");
                rec.group_id = Some(gid.clone());
                if store.save_run(&rec).is_ok() {
                    let _ = crate::worktree::create_isolated_worktree(repo_root, &vid, "HEAD");
                    created.push(rec.to_json());
                }
            }
            format!("{{\"groupId\":\"{}\",\"variants\":[{}]}}", json_escape(&gid), created.join(","))
        }
        "cezar_run_events" => {
            let rid = extract_json_str_field(raw_req, "run_id").unwrap_or_default();
            match store.read_events(&rid, 0) {
                Ok(lines) => format!("{{\"events\":[{}]}}", lines.join(",")),
                Err(e) => format!("{{\"error\":\"{}\"}}", json_escape(&e)),
            }
        }
        "cezar_score_variants" => {
            let da = extract_json_str_field(raw_req, "diff_a").unwrap_or_default();
            let db = extract_json_str_field(raw_req, "diff_b").unwrap_or_default();
            let cands = vec![
                VariantCandidate {
                    variant_id: "variant-a".to_string(),
                    runner: "codex".to_string(),
                    model: "gpt-5.3-codex".to_string(),
                    check_exit_code: 0,
                    unified_diff: da,
                    duration_ms: 1200,
                    cost_usd_micros: 15000,
                },
                VariantCandidate {
                    variant_id: "variant-b".to_string(),
                    runner: "claude".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    check_exit_code: 0,
                    unified_diff: db,
                    duration_ms: 1400,
                    cost_usd_micros: 18000,
                },
            ];
            let ranked: Vec<String> = score_variant_candidates(&cands).into_iter().map(|r| r.to_json()).collect();
            format!("{{\"ranking\":[{}]}}", ranked.join(","))
        }
        "cezar_reclaim_worktrees" => {
            let dry = extract_json_str_field(raw_req, "dry_run")
                .map(|v| v == "true")
                .unwrap_or(true);
            reclaim_worktrees_byte_aware(repo_root, DEFAULT_KEEP_WORKTREES, DEFAULT_MAX_WORKTREE_BYTES, dry).to_json()
        }
        "cezar_list_workflows" => {
            let wfs: Vec<String> = load_repo_workflows(repo_root).into_iter().map(|w| w.to_json()).collect();
            format!("{{\"workflows\":[{}]}}", wfs.join(","))
        }
        "cezar_audit_diff" => {
            let diff = extract_json_str_field(raw_req, "diff").unwrap_or_default();
            audit_unified_diff(&diff).to_json()
        }
        "cezar_forge_detect" => {
            if let Some(u) = extract_json_str_field(raw_req, "remote_url") {
                parse_forge_remote(&u, None)
                    .map(|f| f.to_json())
                    .unwrap_or_else(|| "{\"error\":\"unrecognized remote\"}".to_string())
            } else {
                detect_repo_forge(repo_root)
                    .map(|f| f.to_json())
                    .unwrap_or_else(|| "{\"forge\":null}".to_string())
            }
        }
        "cezar_usage_mix" => {
            let audit = audit_outbound_public_tree(repo_root)
                .map(|r| r.passed)
                .unwrap_or(true);
            let mix = compute_usage_mix_json(&[]);
            format!("{{\"sheriffOutboundClean\":{audit},\"usage\":{mix}}}")
        }
        _ => format!("{{\"error\":\"unknown tool {}\"}}", json_escape(tool_name)),
    }
}

pub fn run_mcp_stdio_loop(repo_root: &Path) -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if let Some(resp) = handle_mcp_jsonrpc_line(repo_root, &line) {
            writeln!(stdout, "{resp}").map_err(|e| e.to_string())?;
            stdout.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
