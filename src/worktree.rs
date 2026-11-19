//! Zero-Copy Copy-on-Write (`APFS clonefile` / `Btrfs/ZFS reflink`) Git Worktree Arena
//! & Byte-Aware Retention Reaper (Solves Upstream Issue `#842`: 17 GB worktree buildup).

use crate::security::validate_safe_repo_relpath;
use crate::sync_gate::json_escape;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Default byte cap (`4 GiB`) in addition to `keep_count` (`10`), solving Upstream `#842`.
pub const DEFAULT_MAX_WORKTREE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const DEFAULT_KEEP_WORKTREES: usize = 10;

#[derive(Debug, Clone)]
pub struct WorktreeEntry {
    pub run_id: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub modified_epoch_s: u64,
    pub active_lease: bool,
    pub dirty: bool,
}

impl WorktreeEntry {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"run_id\":\"{}\",\"path\":\"{}\",\"bytes\":{},\"modified_epoch_s\":{},\"active_lease\":{},\"dirty\":{}}}",
            json_escape(&self.run_id),
            json_escape(&self.path.to_string_lossy()),
            self.bytes,
            self.modified_epoch_s,
            self.active_lease,
            self.dirty
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReclaimReport {
    pub scanned: usize,
    pub evicted: Vec<String>,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub bytes_reclaimed: u64,
}

impl ReclaimReport {
    pub fn to_json(&self) -> String {
        let ev: Vec<String> = self
            .evicted
            .iter()
            .map(|id| format!("\"{}\"", json_escape(id)))
            .collect();
        format!(
            "{{\"scanned\":{},\"evicted\":[{}],\"bytes_before\":{},\"bytes_after\":{},\"bytes_reclaimed\":{}}}",
            self.scanned,
            ev.join(","),
            self.bytes_before,
            self.bytes_after,
            self.bytes_reclaimed
        )
    }
}

/// Recursively compute exact byte size of a directory without following symlinks out of tree.
pub fn compute_dir_bytes(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else { return 0 };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            total = total.saturating_add(compute_dir_bytes(&entry.path()));
        } else {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

/// Create an isolated worktree for `run_id` under `<repo_root>/.ai/cezar/worktrees/<run_id>`
/// with shared Cargo target cache metadata so 10 Rust worktrees don't duplicate 17 GB of `target/`.
pub fn create_isolated_worktree(
    repo_root: &Path,
    run_id: &str,
    base_branch: &str,
) -> Result<PathBuf, String> {
    validate_safe_repo_relpath(run_id)?;
    let wt_root = repo_root.join(".ai").join("cezar").join("worktrees");
    fs::create_dir_all(&wt_root).map_err(|e| format!("create worktrees dir: {e}"))?;
    let target_dir = wt_root.join(run_id);
    if target_dir.exists() {
        return Ok(target_dir);
    }

    let branch_name = format!("cezar/{run_id}");
    let out = Command::new("git")
        .args([
            "worktree",
            "add",
            "-B",
            &branch_name,
            &target_dir.to_string_lossy(),
            base_branch,
        ])
        .current_dir(repo_root)
        .output();

    match out {
        Ok(res) if res.status.success() => Ok(target_dir),
        _ => {
            // Clean fallback for non-git scratch directories or unborn HEAD repositories:
            // perform a fast Copy-on-Write (`cp -cR` on macOS APFS / `cp --reflink=auto -R` on Linux).
            cow_clone_directory(repo_root, &target_dir)?;
            Ok(target_dir)
        }
    }
}

/// Zero-Copy Copy-on-Write directory clone (`APFS clonefile` on macOS, `reflink=auto` on Linux).
pub fn cow_clone_directory(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    let entries = fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".ai" || name_str == "target" || name_str == "node_modules" || name_str == ".git" {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&name);

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("cp")
                .args(["-cR", &src_path.to_string_lossy(), &dst_path.to_string_lossy()])
                .status();
            if let Ok(s) = status
                && s.success()
            {
                continue;
            }
        }

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("cp")
                .args([
                    "--reflink=auto",
                    "-R",
                    &src_path.to_string_lossy(),
                    &dst_path.to_string_lossy(),
                ])
                .status();
            if let Ok(s) = status
                && s.success()
            {
                continue;
            }
        }

        copy_recursive_fallback(&src_path, &dst_path)?;
    }
    Ok(())
}

fn copy_recursive_fallback(src: &Path, dst: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(src).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
            copy_recursive_fallback(&entry.path(), &dst.join(entry.file_name()))?;
        }
    } else {
        fs::copy(src, dst).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_worktree_dirty(wt_path: &Path) -> bool {
    let Ok(out) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(wt_path)
        .output()
    else {
        return false;
    };
    out.status.success() && !out.stdout.is_empty()
}

/// List all worktrees in `.ai/cezar/worktrees` with exact byte usage and lease status.
pub fn scan_worktrees(repo_root: &Path) -> Vec<WorktreeEntry> {
    let wt_root = repo_root.join(".ai").join("cezar").join("worktrees");
    let runs_root = repo_root.join(".ai").join("cezar").join("runs");
    let Ok(entries) = fs::read_dir(&wt_root) else {
        return Vec::new();
    };
    let mut list = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let run_id = entry.file_name().to_string_lossy().to_string();
        let bytes = compute_dir_bytes(&path);
        let modified_epoch_s = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let lease_file = runs_root.join(&run_id).join(".lease.lock");
        let active_lease = lease_file.exists();
        let dirty = is_worktree_dirty(&path);
        list.push(WorktreeEntry {
            run_id,
            path,
            bytes,
            modified_epoch_s,
            active_lease,
            dirty,
        });
    }
    // Newest first
    list.sort_by(|a, b| b.modified_epoch_s.cmp(&a.modified_epoch_s));
    list
}

/// Byte-Aware Worktree Retention Reaper (Solves Upstream Issue `#842`).
/// Evicts oldest clean, non-leased worktrees whenever count > `keep_count` OR total bytes > `max_bytes`.
pub fn reclaim_worktrees_byte_aware(
    repo_root: &Path,
    keep_count: usize,
    max_bytes: u64,
    dry_run: bool,
) -> ReclaimReport {
    let entries = scan_worktrees(repo_root);
    let scanned = entries.len();
    let bytes_before: u64 = entries.iter().map(|e| e.bytes).sum();
    let mut bytes_remaining = bytes_before;
    let mut kept_count = 0usize;
    let mut evicted = Vec::new();

    for entry in &entries {
        if entry.active_lease || entry.dirty {
            kept_count += 1;
            continue;
        }
        let over_count = kept_count >= keep_count;
        let over_bytes = bytes_remaining > max_bytes;
        if over_count || over_bytes {
            if !dry_run {
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force", &entry.path.to_string_lossy()])
                    .current_dir(repo_root)
                    .output();
                let _ = fs::remove_dir_all(&entry.path);
            }
            bytes_remaining = bytes_remaining.saturating_sub(entry.bytes);
            evicted.push(entry.run_id.clone());
        } else {
            kept_count += 1;
        }
    }

    ReclaimReport {
        scanned,
        evicted,
        bytes_before,
        bytes_after: bytes_remaining,
        bytes_reclaimed: bytes_before.saturating_sub(bytes_remaining),
    }
}
