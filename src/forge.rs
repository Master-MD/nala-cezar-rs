//! Pluggable Multi-Forge Adapter (GitHub, GitLab, Forgejo, Gitea — Solves Upstream `#847`)
//! & Safe Loopback Task Preview Port Registry (Solves Upstream `#1877`).

use crate::security::validate_outbound_webhook_url;
use crate::sync_gate::json_escape;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeKind {
    GitHub,
    GitLab,
    Forgejo,
    Gitea,
    GenericGit,
}

impl ForgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ForgeKind::GitHub => "github",
            ForgeKind::GitLab => "gitlab",
            ForgeKind::Forgejo => "forgejo",
            ForgeKind::Gitea => "gitea",
            ForgeKind::GenericGit => "generic-git",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForgeTarget {
    pub kind: ForgeKind,
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub api_base: String,
    pub cli_tool: &'static str,
}

impl ForgeTarget {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"kind\":\"{}\",\"host\":\"{}\",\"owner\":\"{}\",\"repo\":\"{}\",\"api_base\":\"{}\",\"cli_tool\":\"{}\"}}",
            self.kind.as_str(),
            json_escape(&self.host),
            json_escape(&self.owner),
            json_escape(&self.repo),
            json_escape(&self.api_base),
            self.cli_tool
        )
    }

    /// Build the Draft-PR creation CLI command or REST endpoint payload for this forge.
    pub fn build_draft_pr_command(&self, head_branch: &str, base_branch: &str, title: &str) -> Vec<String> {
        match self.kind {
            ForgeKind::GitHub => vec![
                "gh".to_string(),
                "pr".to_string(),
                "create".to_string(),
                "--draft".to_string(),
                "--head".to_string(),
                head_branch.to_string(),
                "--base".to_string(),
                base_branch.to_string(),
                "--title".to_string(),
                title.to_string(),
            ],
            ForgeKind::GitLab => vec![
                "glab".to_string(),
                "mr".to_string(),
                "create".to_string(),
                "--draft".to_string(),
                "--source-branch".to_string(),
                head_branch.to_string(),
                "--target-branch".to_string(),
                base_branch.to_string(),
                "--title".to_string(),
                title.to_string(),
            ],
            ForgeKind::Forgejo | ForgeKind::Gitea => vec![
                "tea".to_string(),
                "pr".to_string(),
                "create".to_string(),
                "--head".to_string(),
                head_branch.to_string(),
                "--base".to_string(),
                base_branch.to_string(),
                "--title".to_string(),
                format!("WIP: {title}"),
            ],
            ForgeKind::GenericGit => vec![
                "git".to_string(),
                "push".to_string(),
                "-u".to_string(),
                "origin".to_string(),
                head_branch.to_string(),
            ],
        }
    }
}

/// Parse any `https://host/owner/repo.git` or `git@host:owner/repo.git` remote into a `ForgeTarget`.
pub fn parse_forge_remote(remote_url: &str, hint_kind: Option<&str>) -> Option<ForgeTarget> {
    let trimmed = remote_url.trim();
    let (scheme, host, path_part) = if let Some(rest) = trimmed.strip_prefix("https://") {
        let (h, p) = rest.split_once('/')?;
        ("https", h, p)
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        let (h, p) = rest.split_once('/')?;
        ("http", h, p)
    } else if let Some(rest) = trimmed.strip_prefix("ssh://") {
        let after_at = rest.rsplit_once('@').map(|(_, r)| r).unwrap_or(rest);
        let (h, p) = after_at.split_once('/')?;
        ("https", h, p)
    } else if let Some((_, after_at)) = trimmed.split_once('@') {
        let (h, p) = after_at.split_once(':')?;
        ("https", h, p)
    } else {
        return None;
    };

    let clean_path = path_part.trim_matches('/').trim_end_matches(".git");
    let mut segs = clean_path.split('/');
    let owner = segs.next()?.to_string();
    let repo_parts: Vec<&str> = segs.collect();
    if owner.is_empty() || repo_parts.is_empty() {
        return None;
    }
    let repo = repo_parts.join("/");
    let host_low = host.to_ascii_lowercase();

    let kind = match hint_kind.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("forgejo") => ForgeKind::Forgejo,
        Some("gitea") => ForgeKind::Gitea,
        Some("gitlab") => ForgeKind::GitLab,
        Some("github") => ForgeKind::GitHub,
        _ if host_low == "github.com" || host_low.ends_with(".ghe.com") => ForgeKind::GitHub,
        _ if host_low.contains("gitlab") => ForgeKind::GitLab,
        _ if host_low.contains("forgejo") || host_low.contains("codeberg.org") || host_low.starts_with("git.") => {
            ForgeKind::Forgejo
        }
        _ if host_low.contains("gitea") => ForgeKind::Gitea,
        _ => ForgeKind::GenericGit,
    };

    let (api_base, cli_tool) = match kind {
        ForgeKind::GitHub if host_low == "github.com" => ("https://api.github.com".to_string(), "gh"),
        ForgeKind::GitHub => (format!("{scheme}://{host}/api/v3"), "gh"),
        ForgeKind::GitLab => (format!("{scheme}://{host}/api/v4"), "glab"),
        ForgeKind::Forgejo | ForgeKind::Gitea => (format!("{scheme}://{host}/api/v1"), "tea"),
        ForgeKind::GenericGit => (format!("{scheme}://{host}"), "git"),
    };

    let _ = validate_outbound_webhook_url(&api_base, true);

    Some(ForgeTarget {
        kind,
        host: host.to_string(),
        owner,
        repo,
        api_base,
        cli_tool,
    })
}

pub fn detect_repo_forge(repo_root: &Path) -> Option<ForgeTarget> {
    let hint = std::env::var("CEZ_FORGE_KIND").ok();
    let out = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&out.stdout);
    parse_forge_remote(&url, hint.as_deref())
}

/// Safe Loopback Preview Port Registry (Solves Upstream `#1877`).
pub struct PreviewPortRegistry {
    routes: HashMap<String, u16>,
}

impl PreviewPortRegistry {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn register(&mut self, run_id: &str, port: u16) -> Result<String, &'static str> {
        if port < 1024 {
            return Err("preview port must be an unprivileged user port (>= 1024)");
        }
        self.routes.insert(run_id.to_string(), port);
        Ok(format!("/preview/{run_id}/ -> http://127.0.0.1:{port}/"))
    }

    pub fn resolve(&self, run_id: &str) -> Option<u16> {
        self.routes.get(run_id).copied()
    }
}
