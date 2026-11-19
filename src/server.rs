//! Hardened Pure-Rust HTTP/1.1 & Zero-Alloc SSE Cockpit Server (`nala-cezar-rs serve`).
//!
//! Enforces all `SEC-CEZAR-01..07` security invariants:
//! - Loopback-default binding (`127.0.0.1:18740`)
//! - `HTTP 413 Payload Too Large` on bodies `> 10 MiB` (`SEC-CEZAR-04`)
//! - Per-client `LoopbackRateLimiter` (`HTTP 429`)
//! - Hardened response headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, `X-Telemetry-Policy: zero-outbound-airgap`)

use crate::bench::{format_bench_json, run_all_benchmarks};
use crate::forge::detect_repo_forge;
use crate::mcp::extract_json_str_field;
use crate::runners::probe_all_backends;
use crate::security::{
    hardened_http_headers, validate_safe_repo_relpath, LoopbackRateLimiter, MAX_HTTP_BODY_BYTES,
    VERSION,
};
use crate::store::{repo_git_branch, CezarStore, RunRecord, RunStatus, ZeroAllocSseFrame};
use crate::sync_gate::{audit_outbound_public_tree, audit_unified_diff, json_escape};
use crate::ui_html::COCKPIT_HTML;
use crate::workflow::load_repo_workflows;
use crate::worktree::{
    create_isolated_worktree, reclaim_worktrees_byte_aware, DEFAULT_KEEP_WORKTREES,
    DEFAULT_MAX_WORKTREE_BYTES,
};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

pub fn start_cockpit_server(repo_root: &Path, host: &str, port: u16) -> Result<(), String> {
    let addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&addr).map_err(|e| format!("bind {addr} failed: {e}"))?;
    println!("🦀 NALA-Cezar-RS v{VERSION} Live Cockpit listening on http://{addr}");
    println!("🛡️  sHeriFF Security: SEC-CEZAR-01..07 ACTIVE | Outbound Telemetry: 0% (Airgap-Safe)");

    let mut limiter = LoopbackRateLimiter::new(120.0, 40.0);

    for incoming in listener.incoming() {
        let Ok(mut stream) = incoming else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        let peer_key = stream
            .peer_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_else(|_| "local".to_string());
        if !limiter.allow(&peer_key) {
            let _ = write_json_response(&mut stream, "429 Too Many Requests", "{\"error\":\"rate limit exceeded\"}");
            continue;
        }

        let _ = handle_http_connection(&mut stream, repo_root);
    }
    Ok(())
}

fn handle_http_connection(stream: &mut TcpStream, repo_root: &Path) -> Result<(), String> {
    let mut buf = [0u8; 65536];
    let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }
    let raw = String::from_utf8_lossy(&buf[..n]);
    let (head, body_initial) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let mut lines = head.lines();
    let req_line = lines.next().unwrap_or("");
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let raw_path = parts.next().unwrap_or("/");
    let path = raw_path.split('?').next().unwrap_or("/");

    // SEC-CEZAR-04: Enforce 10 MiB body limit via Content-Length check
    let mut content_len = 0usize;
    for hdr in lines {
        if let Some((k, v)) = hdr.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length")
        {
            content_len = v.trim().parse::<usize>().unwrap_or(0);
            if content_len > MAX_HTTP_BODY_BYTES {
                return write_json_response(
                    stream,
                    "413 Payload Too Large",
                    "{\"error\":\"SEC-CEZAR-04: request body exceeds 10 MiB limit\"}",
                );
            }
        }
    }

    let mut body = body_initial.to_string();
    while body.len() < content_len && body.len() <= MAX_HTTP_BODY_BYTES {
        let mut chunk = [0u8; 8192];
        let Ok(r) = stream.read(&mut chunk) else { break };
        if r == 0 {
            break;
        }
        body.push_str(&String::from_utf8_lossy(&chunk[..r]));
    }

    let store = CezarStore::new(repo_root);

    match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            let hdrs = hardened_http_headers("text/html; charset=utf-8", COCKPIT_HTML.len());
            stream
                .write_all(format!("HTTP/1.1 200 OK\r\n{hdrs}").as_bytes())
                .map_err(|e| e.to_string())?;
            stream
                .write_all(COCKPIT_HTML.as_bytes())
                .map_err(|e| e.to_string())
        }
        ("GET", "/health") | ("GET", "/api/v1/health") => {
            let branch = repo_git_branch(repo_root);
            let forge = detect_repo_forge(repo_root)
                .map(|f| f.to_json())
                .unwrap_or_else(|| "null".to_string());
            let json = format!(
                "{{\"status\":\"ok\",\"version\":\"{VERSION}\",\"branch\":\"{}\",\"forge\":{forge},\"telemetry\":\"zero-outbound\"}}",
                json_escape(&branch)
            );
            write_json_response(stream, "200 OK", &json)
        }
        ("GET", "/api/v1/backends") => {
            let items: Vec<String> = probe_all_backends().into_iter().map(|b| b.to_json()).collect();
            write_json_response(stream, "200 OK", &format!("{{\"backends\":[{}]}}", items.join(",")))
        }
        ("GET", "/api/v1/workflows") => {
            let items: Vec<String> = load_repo_workflows(repo_root).into_iter().map(|w| w.to_json()).collect();
            write_json_response(stream, "200 OK", &format!("{{\"workflows\":[{}]}}", items.join(",")))
        }
        ("GET", "/api/v1/runs") => {
            let runs = store.list_run_jsons();
            write_json_response(stream, "200 OK", &format!("{{\"runs\":[{}]}}", runs.join(",")))
        }
        ("POST", "/api/v1/runs") => {
            let title = extract_json_str_field(&body, "title").unwrap_or_else(|| "Task".to_string());
            let runner = extract_json_str_field(&body, "runner").unwrap_or_else(|| "codex".to_string());
            let mode = extract_json_str_field(&body, "mode").unwrap_or_else(|| "single".to_string());
            let ts_id = RunRecord::now_ms() % 1_000_000;

            if mode == "arena3" {
                let gid = format!("arena-{ts_id}");
                let trio = [
                    ("codex", "gpt-5.3-codex"),
                    ("claude", "claude-sonnet-4-6"),
                    ("local-openai", "qwen2.5-coder:32b"),
                ];
                let mut created = Vec::new();
                for (i, (r, m)) in trio.iter().enumerate() {
                    let vid = format!("{gid}-v{}", i + 1);
                    let mut rec = RunRecord::new(&vid, &title, r, m, "best-of-3-arena");
                    rec.group_id = Some(gid.clone());
                    rec.status = RunStatus::Review;
                    let _ = store.save_run(&rec);
                    let _ = create_isolated_worktree(repo_root, &vid, "HEAD");
                    let _ = store.append_event(&vid, 1, "arena", "worktree_ready", &title, Some(1), Some(2));
                    created.push(rec.to_json());
                }
                write_json_response(
                    stream,
                    "201 Created",
                    &format!("{{\"groupId\":\"{gid}\",\"runs\":[{}]}}", created.join(",")),
                )
            } else {
                let id = format!("run-{ts_id}");
                let mut rec = RunRecord::new(&id, &title, &runner, "default", "implement-and-verify");
                rec.status = RunStatus::Review;
                let _ = store.save_run(&rec);
                let _ = create_isolated_worktree(repo_root, &id, "HEAD");
                let _ = store.append_event(&id, 1, "init", "worktree_ready", &title, Some(1), Some(2));
                write_json_response(stream, "201 Created", &rec.to_json())
            }
        }
        ("POST", "/api/v1/worktrees/reclaim") => {
            let rep = reclaim_worktrees_byte_aware(
                repo_root,
                DEFAULT_KEEP_WORKTREES,
                DEFAULT_MAX_WORKTREE_BYTES,
                false,
            );
            write_json_response(stream, "200 OK", &rep.to_json())
        }
        ("GET", "/api/v1/bench") => {
            let rows = run_all_benchmarks();
            write_json_response(stream, "200 OK", &format_bench_json(&rows))
        }
        ("POST", "/api/v1/sync/audit-diff") => {
            let rep = audit_unified_diff(&body);
            write_json_response(stream, "200 OK", &rep.to_json())
        }
        ("GET", "/api/v1/sync/audit") => {
            let rep = audit_outbound_public_tree(repo_root)?;
            write_json_response(stream, "200 OK", &rep.to_json())
        }
        ("GET", p) if p.starts_with("/api/v1/runs/") && p.ends_with("/events") => {
            let mid = p
                .trim_start_matches("/api/v1/runs/")
                .trim_end_matches("/events")
                .trim_matches('/');
            if validate_safe_repo_relpath(mid).is_err() {
                return write_json_response(stream, "400 Bad Request", "{\"error\":\"SEC-CEZAR-03: invalid run id\"}");
            }
            let sse_hdr = format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: text/event-stream\r\n\
                 Cache-Control: no-cache, no-transform\r\n\
                 X-Accel-Buffering: no\r\n\
                 X-Content-Type-Options: nosniff\r\n\
                 X-NALA-Version: {VERSION}\r\n\
                 Connection: close\r\n\r\n"
            );
            stream.write_all(sse_hdr.as_bytes()).map_err(|e| e.to_string())?;
            let frame = ZeroAllocSseFrame::format_run_event(1, "connected", "sse", mid);
            stream.write_all(frame.as_bytes()).map_err(|e| e.to_string())
        }
        ("GET", p) if p.starts_with("/api/v1/git/files/") => {
            let rel = p.trim_start_matches("/api/v1/git/files/");
            if let Err(msg) = validate_safe_repo_relpath(rel) {
                return write_json_response(
                    stream,
                    "403 Forbidden",
                    &format!("{{\"error\":\"{}\"}}", json_escape(msg)),
                );
            }
            let full = repo_root.join(rel);
            match std::fs::read_to_string(&full) {
                Ok(content) => write_json_response(
                    stream,
                    "200 OK",
                    &format!("{{\"path\":\"{}\",\"content\":\"{}\"}}", json_escape(rel), json_escape(&content)),
                ),
                Err(_) => write_json_response(stream, "404 Not Found", "{\"error\":\"file not found\"}"),
            }
        }
        _ => write_json_response(stream, "404 Not Found", "{\"error\":\"route not found\"}"),
    }
}

fn write_json_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), String> {
    let hdrs = hardened_http_headers("application/json; charset=utf-8", body.len());
    stream
        .write_all(format!("HTTP/1.1 {status}\r\n{hdrs}").as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body.as_bytes()).map_err(|e| e.to_string())
}
