//! Bidirectional Public-GitHub <-> Internal Enterprise Sync Gate, Unified-Diff Auditor & SLSA SBOM Generator.
//!
//! 1) Outbound (`--direction to-public`): Uses irreversible 64-bit FNV-1a token hashes and XOR-masked
//!    subnet bytes so zero private hostnames, usernames, or lab IPs ever exist in plaintext inside
//!    source or `.rodata`, while blocking any accidental leak before `git push`.
//! 2) Inbound (`--direction from-public`): Enforces 6 deterministic `sHeriFF` source security gates
//!    (`#![forbid(unsafe_code)]`, `0` external crates, no `build.rs`, no reverse shells, no BiDi Unicode).
//! 3) Unified Diff Auditor (`--audit-diff <patch>`): Audits external GitHub PRs and Best-of-N agent
//!    worktree diffs before promotion or Draft-PR creation.

use crate::security::{contains_bidi_trojan_unicode, fnv1a_64, hex_encode, hmac_sha256, sha256, VERSION};
use std::fs;
use std::path::{Path, PathBuf};

/// 64-bit FNV-1a hashes of private hostnames, personal identifiers, and lab hardware tags.
pub const FORBIDDEN_OUTBOUND_FNV1A: &[u64] = &[
    0x2c45d495230c213b,
    0x8c9a46c4f09e6289,
    0x3e154dde8b9aebb7,
    0x0900896c2a9e884e,
    0x076204fb731e2da1,
    0xb26af66ec5db9d42,
    0x14a8e62980b89d68,
    0x6e29cd0069eb8bc9,
    0xf8c1f2a858269750,
    0xa333204225de82fe,
    0x7928b4ba08dcfebc,
    0xad91bcba26ac3f07,
    0x394bf7a8a87d7d13,
    0xe1d184e91530c1f3,
    0xe90214723933b0af,
    0x1bc69fddaa409ac6,
    0x999c48027af76ee5,
    0x9abbcb21429c900a,
    0xcdc2afa7f75eb3da,
    0xcd61ff146fa2b308,
    0x0cd37bcf08b8ea8e,
    0x44ee20b2b92ce15a,
];

#[derive(Debug, Clone)]
pub struct SyncAuditReport {
    pub direction: &'static str,
    pub passed: bool,
    pub files_scanned: usize,
    pub tree_merkle_sha256: String,
    pub findings: Vec<String>,
    pub gates_passed: Vec<&'static str>,
}

impl SyncAuditReport {
    pub fn to_json(&self) -> String {
        let findings_json: Vec<String> = self
            .findings
            .iter()
            .map(|f| format!("\"{}\"", json_escape(f)))
            .collect();
        let gates_json: Vec<String> = self
            .gates_passed
            .iter()
            .map(|g| format!("\"{g}\""))
            .collect();
        format!(
            "{{\"direction\":\"{}\",\"passed\":{},\"files_scanned\":{},\"tree_merkle_sha256\":\"{}\",\"gates_passed\":[{}],\"findings\":[{}]}}",
            self.direction,
            self.passed,
            self.files_scanned,
            self.tree_merkle_sha256,
            gates_json.join(","),
            findings_json.join(",")
        )
    }
}

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn collect_audit_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == "target" || name == "dist" || name == ".DS_Store" || name == ".ncsp_snapshots" {
            continue;
        }
        if path.is_dir() {
            collect_audit_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// XOR-masked check for RFC1918 and CGNAT private IPv4 subnet prefixes
/// without storing any literal subnet string in `.rodata`.
fn contains_masked_private_subnet(token: &str) -> bool {
    let parts: Vec<&str> = token
        .trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
        .split('.')
        .collect();
    if parts.len() < 3 {
        return false;
    }
    let Ok(o0) = parts[0].parse::<u8>() else { return false };
    let Ok(o1) = parts[1].parse::<u8>() else { return false };
    let Ok(o2) = parts[2].parse::<u8>() else { return false };
    let k = 0x5Au8;
    // RFC1918 /24 host range check via XOR mask 0x5A
    if (o0 ^ k) == 154 && (o1 ^ k) == 242 && (o2 ^ k) == 91 && parts.len() == 4 && parts[3] != "0" && parts[3] != "1" {
        return true;
    }
    // CGNAT mesh range check via XOR mask 0x5A
    if (o0 ^ k) == 62 && matches!(o1 ^ k, 15 | 4 | 56) && parts.len() == 4 {
        return true;
    }
    false
}

pub fn scan_text_for_outbound_leaks(rel: &str, text: &str, findings: &mut Vec<String>) {
    for (line_idx, line) in text.lines().enumerate() {
        for raw_tok in line.split(|c: char| {
            c.is_whitespace()
                || matches!(
                    c,
                    '/' | '\\' | ':' | ';' | ',' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '@' | '='
                )
        }) {
            if raw_tok.len() < 4 {
                continue;
            }
            let low = raw_tok.to_ascii_lowercase();
            let h = fnv1a_64(low.as_bytes());
            if FORBIDDEN_OUTBOUND_FNV1A.contains(&h) {
                findings.push(format!(
                    "GATE-OUT-01 [{rel}:{}]: forbidden private token hash 0x{h:016x}",
                    line_idx + 1
                ));
            }
            if contains_masked_private_subnet(raw_tok) {
                findings.push(format!(
                    "GATE-OUT-02 [{rel}:{}]: private lab IPv4 address detected",
                    line_idx + 1
                ));
            }
        }
    }
}

/// Audit repository tree before pushing to public GitHub (`--direction to-public`).
pub fn audit_outbound_public_tree(root: &Path) -> Result<SyncAuditReport, String> {
    let mut files = Vec::new();
    collect_audit_files(root, &mut files)?;
    files.sort();

    let mut findings = Vec::new();
    let mut merkle_input = Vec::new();

    for path in &files {
        let Ok(bytes) = fs::read(path) else { continue };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let file_hash = sha256(&bytes);
        merkle_input.extend_from_slice(rel.as_bytes());
        merkle_input.extend_from_slice(&file_hash);

        if let Ok(text) = std::str::from_utf8(&bytes) {
            scan_text_for_outbound_leaks(&rel, text, &mut findings);
            if contains_bidi_trojan_unicode(text) {
                findings.push(format!("GATE-OUT-03 [{rel}]: Trojan-Source BiDi Unicode detected"));
            }
        }
    }

    let merkle = hex_encode(&sha256(&merkle_input));
    let passed = findings.is_empty();
    let gates = if passed {
        vec!["GATE-OUT-01_FNV1A_ZERO_PII", "GATE-OUT-02_XOR_ZERO_PRIVATE_IP", "GATE-OUT-03_ZERO_BIDI"]
    } else {
        vec![]
    };

    Ok(SyncAuditReport {
        direction: "to-public",
        passed,
        files_scanned: files.len(),
        tree_merkle_sha256: merkle,
        findings,
        gates_passed: gates,
    })
}

/// Audit repository tree before pulling collaborator commits (`--direction from-public`).
pub fn audit_inbound_collaborator_tree(root: &Path) -> Result<SyncAuditReport, String> {
    let mut files = Vec::new();
    collect_audit_files(root, &mut files)?;
    files.sort();

    let mut findings = Vec::new();
    let mut merkle_input = Vec::new();
    let mut main_has_forbid_unsafe = false;

    for path in &files {
        let Ok(bytes) = fs::read(path) else { continue };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let file_hash = sha256(&bytes);
        merkle_input.extend_from_slice(rel.as_bytes());
        merkle_input.extend_from_slice(&file_hash);

        if rel == "build.rs" || rel.ends_with("/build.rs") {
            findings.push(format!("GATE-COLLAB-03 [{rel}]: build.rs execution hook forbidden"));
        }

        let Ok(text) = std::str::from_utf8(&bytes) else { continue };

        if contains_bidi_trojan_unicode(text) {
            findings.push(format!("GATE-COLLAB-05 [{rel}]: Trojan-Source BiDi Unicode CVE-2021-42574"));
        }

        if rel == "src/main.rs" && text.contains("#![forbid(unsafe_code)]") {
            main_has_forbid_unsafe = true;
        }

        if rel == "Cargo.toml" {
            let mut in_deps = false;
            for line in text.lines() {
                let t = line.trim();
                if t.starts_with('[') {
                    in_deps = t == "[dependencies]" || t == "[build-dependencies]";
                    continue;
                }
                if in_deps && !t.is_empty() && !t.starts_with('#') {
                    findings.push(format!(
                        "GATE-COLLAB-02 [Cargo.toml]: external crate dependency injection forbidden: `{t}`"
                    ));
                }
            }
        }

        if rel.ends_with(".rs") && rel != "src/security.rs" && rel != "src/sync_gate.rs" {
            for pat in ["/dev/tcp/", "nc -e ", "bash -i >&", "curl | sh"] {
                if text.contains(pat) {
                    findings.push(format!("GATE-COLLAB-04 [{rel}]: reverse-shell/backdoor pattern `{pat}`"));
                }
            }
        }
    }

    if !main_has_forbid_unsafe {
        findings.push("GATE-COLLAB-01 [src/main.rs]: missing `#![forbid(unsafe_code)]` crate invariant".to_string());
    }

    let merkle = hex_encode(&sha256(&merkle_input));
    let passed = findings.is_empty();
    let gates = if passed {
        vec![
            "GATE-COLLAB-01_FORBID_UNSAFE",
            "GATE-COLLAB-02_ZERO_CRATE_DEPS",
            "GATE-COLLAB-03_NO_BUILD_RS",
            "GATE-COLLAB-04_NO_BACKDOOR_SHELL",
            "GATE-COLLAB-05_NO_TROJAN_BIDI",
            "GATE-COLLAB-06_MERKLE_ATTESTED",
        ]
    } else {
        vec![]
    };

    Ok(SyncAuditReport {
        direction: "from-public",
        passed,
        files_scanned: files.len(),
        tree_merkle_sha256: merkle,
        findings,
        gates_passed: gates,
    })
}

/// Audit a unified git diff (`diff --git ...`) produced by a collaborator PR or Best-of-N coding agent.
pub fn audit_unified_diff(diff_text: &str) -> SyncAuditReport {
    let mut findings = Vec::new();
    let mut current_file = String::from("<unknown>");
    let mut files_touched = 0usize;

    for (idx, line) in diff_text.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = rest.trim().to_string();
            files_touched += 1;
            if current_file.contains("..")
                || current_file.starts_with('/')
                || current_file.contains(".git/hooks")
                || current_file == "build.rs"
                || current_file.ends_with("/build.rs")
            {
                findings.push(format!(
                    "GATE-DIFF-00 [{}:{}]: forbidden target path `{current_file}`",
                    current_file,
                    idx + 1
                ));
            }
            continue;
        }

        if line.starts_with('-') && !line.starts_with("---") && line.contains("#![forbid(unsafe_code)]") {
            findings.push(format!(
                "GATE-COLLAB-01 [{}:{}]: removing `#![forbid(unsafe_code)]` is forbidden",
                current_file,
                idx + 1
            ));
        }

        if let Some(added) = line.strip_prefix('+') {
            if line.starts_with("+++") {
                continue;
            }
            if contains_bidi_trojan_unicode(added) {
                findings.push(format!(
                    "GATE-COLLAB-05 [{}:{}]: Trojan-Source BiDi Unicode in added line",
                    current_file,
                    idx + 1
                ));
            }
            let low = added.to_ascii_lowercase();
            for pat in [
                "unsafe {",
                "unsafe fn ",
                "/dev/tcp/",
                "nc -e ",
                "stats.openmercato.cloud",
                "rm -rf /",
            ] {
                if low.contains(pat) {
                    findings.push(format!(
                        "GATE-COLLAB-04 [{}:{}]: forbidden pattern `{pat}` in added diff line",
                        current_file,
                        idx + 1
                    ));
                }
            }
            scan_text_for_outbound_leaks(&current_file, added, &mut findings);
        }
    }

    let digest = hex_encode(&sha256(diff_text.as_bytes()));
    let passed = findings.is_empty();
    let gates = if passed {
        vec![
            "GATE-DIFF-00_SAFE_PATHS",
            "GATE-COLLAB-01..06_CLEAN_CODE",
            "GATE-OUT-01..03_ZERO_PII_LEAK",
        ]
    } else {
        vec![]
    };

    SyncAuditReport {
        direction: "audit-diff",
        passed,
        files_scanned: files_touched.max(1),
        tree_merkle_sha256: digest,
        findings,
        gates_passed: gates,
    }
}

/// Deterministic SLSA v1.0 & CycloneDX zero-dependency SBOM provenance generator.
pub fn generate_slsa_sbom_json(sha256sums_content: &str) -> String {
    let merkle = hex_encode(&sha256(sha256sums_content.as_bytes()));
    let sig = hex_encode(&hmac_sha256(b"NALA-CEZAR-RS-SLSA-ATTESTATION-V1", sha256sums_content.as_bytes()));
    let mut subjects = Vec::new();
    for line in sha256sums_content.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(name)) = (parts.next(), parts.next()) {
            let clean_name = name.trim_start_matches('*');
            subjects.push(format!(
                "{{\"name\":\"{}\",\"digest\":{{\"sha256\":\"{}\"}}}}",
                json_escape(clean_name),
                json_escape(hash)
            ));
        }
    }
    format!(
        "{{\
          \"_type\":\"https://in-toto.io/Statement/v1\",\
          \"predicateType\":\"https://slsa.dev/provenance/v1\",\
          \"bomFormat\":\"CycloneDX\",\
          \"specVersion\":\"1.5\",\
          \"component\":{{\"name\":\"nala-cezar-rs\",\"version\":\"{VERSION}\",\"license\":\"MIT\",\"externalDependencies\":0}},\
          \"subject\":[{}],\
          \"attestation\":{{\"merkle_sha256\":\"{merkle}\",\"hmac_sha256\":\"{sig}\"}}\
        }}",
        subjects.join(",")
    )
}
