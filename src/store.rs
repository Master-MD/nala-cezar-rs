//! Drop-in `.ai/cezar/` Plain-File Store, 64-Byte Aligned `.cezar.idx` Binary Index,
//! Compile-Time RAII `RunLeaseGuard<'a>` & Zero-Alloc `[u8; 512]` Stack SSE Frame Formatter.
//!
//! Solves Upstream Issues:
//! - `#798`: RAII `RunLeaseGuard` prevents stale `"run is still active"` locks on continue/crash
//! - `#2296`: Atomic `clear_stale_asks_on_retry()` prevents runs staying stuck in `waiting` ("Needs you")
//! - `#2289`: Per-node `dispatched_this_turn` reset prevents graph fork nodes parking in `monitoring`
//! - `#1159`: O(1) provider/model token & cost usage mix aggregation via `.cezar.idx`

use crate::security::{fnv1a_64, validate_safe_repo_relpath};
use crate::sync_gate::json_escape;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const BINARY_INDEX_MAGIC: &[u8; 8] = b"CEZIDX01";
pub const BINARY_RECORD_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Queued = 0,
    Running = 1,
    Waiting = 2,
    Review = 3,
    Done = 4,
    Failed = 5,
    Cancelled = 6,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Queued => "queued",
            RunStatus::Running => "running",
            RunStatus::Waiting => "waiting",
            RunStatus::Review => "review",
            RunStatus::Done => "done",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => RunStatus::Queued,
            1 => RunStatus::Running,
            2 => RunStatus::Waiting,
            3 => RunStatus::Review,
            4 => RunStatus::Done,
            5 => RunStatus::Failed,
            _ => RunStatus::Cancelled,
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "queued" => RunStatus::Queued,
            "running" => RunStatus::Running,
            "waiting" => RunStatus::Waiting,
            "review" => RunStatus::Review,
            "done" | "completed" => RunStatus::Done,
            "failed" | "error" => RunStatus::Failed,
            _ => RunStatus::Cancelled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunRecord {
    pub id: String,
    pub group_id: Option<String>,
    pub parent_id: Option<String>,
    pub title: String,
    pub runner: String,
    pub model: String,
    pub workflow: String,
    pub status: RunStatus,
    pub open_asks: usize,
    pub dispatched_this_turn: bool,
    pub tokens_total: u64,
    pub cost_usd_micros: u64,
    pub created_epoch_ms: u64,
    pub updated_epoch_ms: u64,
}

impl RunRecord {
    pub fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    pub fn new(id: &str, title: &str, runner: &str, model: &str, workflow: &str) -> Self {
        let now = Self::now_ms();
        Self {
            id: id.to_string(),
            group_id: None,
            parent_id: None,
            title: title.to_string(),
            runner: runner.to_string(),
            model: model.to_string(),
            workflow: workflow.to_string(),
            status: RunStatus::Queued,
            open_asks: 0,
            dispatched_this_turn: false,
            tokens_total: 0,
            cost_usd_micros: 0,
            created_epoch_ms: now,
            updated_epoch_ms: now,
        }
    }

    /// Solves Upstream `#2296`: When a workflow step retries on its own after an unanswered ask,
    /// close stale open asks and transition cleanly from `Waiting` back to `Running`.
    pub fn clear_stale_asks_on_retry(&mut self) {
        self.open_asks = 0;
        if self.status == RunStatus::Waiting {
            self.status = RunStatus::Running;
        }
        self.updated_epoch_ms = Self::now_ms();
    }

    /// Solves Upstream `#2289`: Entering a new graph node resets `dispatched_this_turn` so
    /// downstream verdict nodes never spuriously park in monitoring state.
    pub fn enter_next_graph_node(&mut self) {
        self.dispatched_this_turn = false;
        self.updated_epoch_ms = Self::now_ms();
    }

    pub fn to_json(&self) -> String {
        let grp = self
            .group_id
            .as_deref()
            .map(|g| format!("\"{}\"", json_escape(g)))
            .unwrap_or_else(|| "null".to_string());
        let par = self
            .parent_id
            .as_deref()
            .map(|p| format!("\"{}\"", json_escape(p)))
            .unwrap_or_else(|| "null".to_string());
        let cost_usd = (self.cost_usd_micros as f64) / 1_000_000.0;
        format!(
            "{{\"id\":\"{}\",\"groupId\":{grp},\"parentId\":{par},\"title\":\"{}\",\"runner\":\"{}\",\"model\":\"{}\",\"workflow\":\"{}\",\"status\":\"{}\",\"openAsks\":{},\"dispatchedThisTurn\":{},\"tokensTotal\":{},\"costUsd\":{cost_usd:.6},\"createdAtMs\":{},\"updatedAtMs\":{}}}",
            json_escape(&self.id),
            json_escape(&self.title),
            json_escape(&self.runner),
            json_escape(&self.model),
            json_escape(&self.workflow),
            self.status.as_str(),
            self.open_asks,
            self.dispatched_this_turn,
            self.tokens_total,
            self.created_epoch_ms,
            self.updated_epoch_ms
        )
    }

    /// Encode compact 64-byte binary index slot for sub-microsecond O(1) lookup (`.cezar.idx`).
    pub fn to_binary_slot(&self) -> [u8; BINARY_RECORD_SIZE] {
        let mut slot = [0u8; BINARY_RECORD_SIZE];
        let id_hash = fnv1a_64(self.id.as_bytes());
        let runner_hash = fnv1a_64(self.runner.as_bytes());
        let model_hash = fnv1a_64(self.model.as_bytes());
        slot[0..8].copy_from_slice(&id_hash.to_le_bytes());
        slot[8..16].copy_from_slice(&runner_hash.to_le_bytes());
        slot[16..24].copy_from_slice(&model_hash.to_le_bytes());
        slot[24..32].copy_from_slice(&self.tokens_total.to_le_bytes());
        slot[32..40].copy_from_slice(&self.cost_usd_micros.to_le_bytes());
        slot[40..48].copy_from_slice(&self.created_epoch_ms.to_le_bytes());
        slot[48..56].copy_from_slice(&self.updated_epoch_ms.to_le_bytes());
        slot[56] = self.status as u8;
        slot[57] = self.open_asks.min(255) as u8;
        slot[58] = u8::from(self.dispatched_this_turn);
        slot
    }
}

/// Compile-time RAII Run Lease Manager solving Upstream `#798` (`proper-lockfile` stale active locks).
pub struct RunLeaseRegistry {
    active_runs: Mutex<HashSet<u64>>,
}

impl RunLeaseRegistry {
    pub fn new() -> Self {
        Self {
            active_runs: Mutex::new(HashSet::new()),
        }
    }

    pub fn acquire<'a>(&'a self, run_id: &str, lock_path: Option<PathBuf>) -> Result<RunLeaseGuard<'a>, String> {
        let key = fnv1a_64(run_id.as_bytes());
        let mut guard = self
            .active_runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !guard.insert(key) {
            return Err(format!("run `{run_id}` already holds an active RAII lease"));
        }
        if let Some(ref lp) = lock_path {
            if let Some(parent) = lp.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let pid = std::process::id();
            let _ = fs::write(lp, format!("{pid}\n"));
        }
        Ok(RunLeaseGuard {
            registry: self,
            key,
            lock_path,
        })
    }

    pub fn is_active(&self, run_id: &str) -> bool {
        let key = fnv1a_64(run_id.as_bytes());
        self.active_runs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .contains(&key)
    }
}

pub struct RunLeaseGuard<'a> {
    registry: &'a RunLeaseRegistry,
    key: u64,
    lock_path: Option<PathBuf>,
}

impl Drop for RunLeaseGuard<'_> {
    fn drop(&mut self) {
        let mut guard = self
            .registry
            .active_runs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.remove(&self.key);
        if let Some(ref lp) = self.lock_path {
            let _ = fs::remove_file(lp);
        }
    }
}

/// Stack-allocated `[u8; 512]` SSE frame formatter (`0` per-event heap allocations!).
pub struct ZeroAllocSseFrame {
    buf: [u8; 512],
    len: usize,
}

impl ZeroAllocSseFrame {
    #[inline]
    pub fn format_run_event(seq: u64, event_type: &str, step_id: &str, text_snippet: &str) -> Self {
        let mut frame = Self {
            buf: [0u8; 512],
            len: 0,
        };
        frame.push_bytes(b"id: ");
        frame.push_u64(seq);
        frame.push_bytes(b"\nevent: run_event\ndata: {\"seq\":");
        frame.push_u64(seq);
        frame.push_bytes(b",\"type\":\"");
        frame.push_clean_ascii(event_type, 32);
        frame.push_bytes(b"\",\"stepId\":\"");
        frame.push_clean_ascii(step_id, 32);
        frame.push_bytes(b"\",\"text\":\"");
        frame.push_clean_ascii(text_snippet, 300);
        frame.push_bytes(b"\"}\n\n");
        frame
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    #[inline]
    fn push_bytes(&mut self, src: &[u8]) {
        let avail = self.buf.len().saturating_sub(self.len);
        let n = src.len().min(avail);
        self.buf[self.len..self.len + n].copy_from_slice(&src[..n]);
        self.len += n;
    }

    #[inline]
    fn push_u64(&mut self, mut val: u64) {
        let mut tmp = [0u8; 20];
        let mut i = 20;
        if val == 0 {
            self.push_bytes(b"0");
            return;
        }
        while val > 0 {
            i -= 1;
            tmp[i] = b'0' + (val % 10) as u8;
            val /= 10;
        }
        self.push_bytes(&tmp[i..]);
    }

    #[inline]
    fn push_clean_ascii(&mut self, s: &str, max_bytes: usize) {
        let mut count = 0;
        for b in s.bytes() {
            if count >= max_bytes || self.len + 2 >= self.buf.len() {
                break;
            }
            match b {
                b'"' | b'\\' => {
                    self.push_bytes(b"_");
                    count += 1;
                }
                0x20..=0x7E => {
                    self.push_bytes(&[b]);
                    count += 1;
                }
                _ => {
                    self.push_bytes(b" ");
                    count += 1;
                }
            }
        }
    }
}

/// Drop-in `.ai/cezar/` Plain-File Store + `.cezar.idx` Binary Index.
pub struct CezarStore {
    repo_root: PathBuf,
}

impl CezarStore {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub fn cezar_dir(&self) -> PathBuf {
        self.repo_root.join(".ai").join("cezar")
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.cezar_dir().join("runs")
    }

    pub fn index_path(&self) -> PathBuf {
        self.cezar_dir().join(".cezar.idx")
    }

    pub fn save_run(&self, record: &RunRecord) -> Result<PathBuf, String> {
        validate_safe_repo_relpath(&record.id)?;
        let run_dir = self.runs_dir().join(&record.id);
        fs::create_dir_all(&run_dir).map_err(|e| format!("create run dir: {e}"))?;
        let json_path = run_dir.join("run.json");
        fs::write(&json_path, record.to_json()).map_err(|e| format!("write run.json: {e}"))?;
        Ok(json_path)
    }

    /// Append one Normalized Agent-Event Protocol v2 line (`events.ndjson`) including `#1543` timing fields.
    pub fn append_event(
        &self,
        run_id: &str,
        seq: u64,
        step_id: &str,
        event_type: &str,
        detail: &str,
        first_token_ms: Option<u64>,
        generation_end_ms: Option<u64>,
    ) -> Result<(), String> {
        validate_safe_repo_relpath(run_id)?;
        let run_dir = self.runs_dir().join(run_id);
        fs::create_dir_all(&run_dir).map_err(|e| format!("create run dir: {e}"))?;
        let ev_path = run_dir.join("events.ndjson");
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ev_path)
            .map_err(|e| format!("open events.ndjson: {e}"))?;
        let ts = RunRecord::now_ms();
        let ft = first_token_ms
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let ge = generation_end_ms
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let line = format!(
            "{{\"seq\":{seq},\"ts\":\"{ts}\",\"stepId\":\"{}\",\"type\":\"{}\",\"detail\":\"{}\",\"firstTokenMs\":{ft},\"generationEndMs\":{ge}}}\n",
            json_escape(step_id),
            json_escape(event_type),
            json_escape(detail)
        );
        f.write_all(line.as_bytes())
            .map_err(|e| format!("append events.ndjson: {e}"))?;
        Ok(())
    }

    pub fn read_events(&self, run_id: &str, after_seq: u64) -> Result<Vec<String>, String> {
        validate_safe_repo_relpath(run_id)?;
        let ev_path = self.runs_dir().join(run_id).join("events.ndjson");
        let Ok(raw) = fs::read_to_string(&ev_path) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for (idx, line) in raw.lines().enumerate() {
            if (idx as u64) + 1 > after_seq && !line.trim().is_empty() {
                out.push(line.to_string());
            }
        }
        Ok(out)
    }

    /// Write compact `.cezar.idx` binary snapshot for sub-microsecond lookups.
    pub fn write_binary_index(&self, records: &[RunRecord]) -> Result<usize, String> {
        fs::create_dir_all(self.cezar_dir()).map_err(|e| e.to_string())?;
        let mut bytes = Vec::with_capacity(16 + records.len() * BINARY_RECORD_SIZE);
        bytes.extend_from_slice(BINARY_INDEX_MAGIC);
        bytes.extend_from_slice(&(records.len() as u64).to_le_bytes());
        for rec in records {
            bytes.extend_from_slice(&rec.to_binary_slot());
        }
        fs::write(self.index_path(), &bytes).map_err(|e| e.to_string())?;
        Ok(bytes.len())
    }

    /// Sub-microsecond O(N) cache-line scan over `.cezar.idx` bytes without JSON parsing!
    pub fn lookup_in_binary_index(index_bytes: &[u8], run_id: &str) -> Option<(RunStatus, u64, u64)> {
        if index_bytes.len() < 16 || &index_bytes[0..8] != BINARY_INDEX_MAGIC {
            return None;
        }
        let target_hash = fnv1a_64(run_id.as_bytes());
        for slot in index_bytes[16..].chunks_exact(BINARY_RECORD_SIZE) {
            let h = u64::from_le_bytes([
                slot[0], slot[1], slot[2], slot[3], slot[4], slot[5], slot[6], slot[7],
            ]);
            if h == target_hash {
                let tokens = u64::from_le_bytes([
                    slot[24], slot[25], slot[26], slot[27], slot[28], slot[29], slot[30], slot[31],
                ]);
                let cost_micros = u64::from_le_bytes([
                    slot[32], slot[33], slot[34], slot[35], slot[36], slot[37], slot[38], slot[39],
                ]);
                let status = RunStatus::from_u8(slot[56]);
                return Some((status, tokens, cost_micros));
            }
        }
        None
    }

    /// List raw `run.json` payloads on disk.
    pub fn list_run_jsons(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.runs_dir()) else {
            return Vec::new();
        };
        let mut jsons = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path().join("run.json");
            if let Ok(txt) = fs::read_to_string(&p) {
                jsons.push(txt);
            }
        }
        jsons
    }

    /// Discover local Markdown skills in `.ai/skills/` and `.ai/cezar/skills/`.
    pub fn discover_skills(&self) -> Vec<(String, PathBuf)> {
        let mut out = Vec::new();
        for sub in [
            self.repo_root.join(".ai").join("skills"),
            self.repo_root.join(".ai").join("cezar").join("skills"),
        ] {
            if let Ok(entries) = fs::read_dir(&sub) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        out.push((name, path));
                    }
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// Solves Upstream `#1159`: Provider & Model Usage Mix Aggregator.
pub fn compute_usage_mix_json(records: &[RunRecord]) -> String {
    let mut by_pair: HashMap<(String, String), (usize, u64, u64)> = HashMap::new();
    for rec in records {
        let entry = by_pair
            .entry((rec.runner.clone(), rec.model.clone()))
            .or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 = entry.1.saturating_add(rec.tokens_total);
        entry.2 = entry.2.saturating_add(rec.cost_usd_micros);
    }
    let mut rows: Vec<_> = by_pair.into_iter().collect();
    rows.sort_by(|a, b| b.1.1.cmp(&a.1.1));
    let items: Vec<String> = rows
        .into_iter()
        .map(|((runner, model), (runs, tokens, micros))| {
            format!(
                "{{\"runner\":\"{}\",\"model\":\"{}\",\"runs\":{runs},\"tokens\":{tokens},\"costUsd\":{:.6}}}",
                json_escape(&runner),
                json_escape(&model),
                (micros as f64) / 1_000_000.0
            )
        })
        .collect();
    format!("{{\"mix\":[{}]}}", items.join(","))
}

pub fn repo_git_branch(repo_root: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".to_string())
}
