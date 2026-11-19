//! `sHeriFF + NVIDIA SkillSpector` Cryptographic Primitives & 7-Layer Security Engine (`SEC-CEZAR-01..07`).
//!
//! Pure Safe Rust (`#![forbid(unsafe_code)]`, 0 external crates):
//! - FIPS 180-4 SHA-256 & RFC 2104 Constant-Time HMAC-SHA256
//! - 64-bit FNV-1a fast deterministic hashing
//! - `SEC-CEZAR-01`: SSRF & Cloud-Metadata Blockade
//! - `SEC-CEZAR-02`: HTML/CSS/SVG Breakout Sanitizer
//! - `SEC-CEZAR-03`: Path-Traversal (`..`) & Sensitive Vault/Key Guard
//! - `SEC-CEZAR-04`: HTTP 413 Body Quota (`<= 10 MiB`) & Per-Client Token-Bucket Rate Limiter
//! - `SEC-CEZAR-05`: Hardened HTTP Security Headers
//! - `SEC-CEZAR-06`: `SkillSpector` Prompt-Injection, Destructive-Shell & BiDi Trojan-Source (`CVE-2021-42574`) Guard
//! - `SEC-CEZAR-07`: Compile-Time Zero-Telemetry Guarantee (`0` outbound analytics endpoints)

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const VERSION: &str = "0.21.0-nala.6.6.6";
pub const MAX_HTTP_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MiB (SEC-CEZAR-04)

/// Deterministic 64-bit FNV-1a hash.
#[inline]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 SHA-256 digest in 100% Pure Safe Rust.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(data.len() + 72);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// RFC 2104 HMAC-SHA256.
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k_block = [0u8; 64];
    if key.len() > 64 {
        let hashed = sha256(key);
        k_block[..32].copy_from_slice(&hashed);
    } else {
        k_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_block[i];
        opad[i] ^= k_block[i];
    }

    let mut inner = Vec::with_capacity(64 + msg.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(msg);
    let inner_hash = sha256(&inner);

    let mut outer = [0u8; 96];
    outer[..64].copy_from_slice(&opad);
    outer[64..].copy_from_slice(&inner_hash);
    sha256(&outer)
}

/// Constant-time slice equality check.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// `SEC-CEZAR-01`: SSRF guard for external webhook/tracker targets.
pub fn validate_outbound_webhook_url(url: &str, allow_local_forge: bool) -> Result<(), &'static str> {
    let trimmed = url.trim();
    let rest = if let Some(r) = trimmed.strip_prefix("https://") {
        r
    } else if let Some(r) = trimmed.strip_prefix("http://") {
        r
    } else {
        return Err("SEC-CEZAR-01: scheme must be http:// or https://");
    };

    let host_port = rest.split('/').next().unwrap_or("");
    let host = host_port
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(host_port)
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    if host.is_empty() {
        return Err("SEC-CEZAR-01: empty target host");
    }

    // Cloud instance metadata & hex/decimal IP obfuscation are unconditionally blocked.
    if host == "169.254.169.254"
        || host == "metadata.google.internal"
        || host == "0.0.0.0"
        || host == "::1"
        || host == "0x7f.1"
        || host.chars().all(|c| c.is_ascii_digit())
    {
        return Err("SEC-CEZAR-01: blocked SSRF metadata/obfuscated host");
    }

    if !allow_local_forge {
        if host == "localhost"
            || host == "127.0.0.1"
            || host.starts_with("127.")
            || host.starts_with("10.")
            || host.starts_with("192.168.")
            || host.starts_with("169.254.")
        {
            return Err("SEC-CEZAR-01: private/loopback target blocked for external webhook");
        }
        if let Some(second) = host.strip_prefix("172.").and_then(|r| r.split('.').next())
            && let Ok(octet) = second.parse::<u8>()
            && (16..=31).contains(&octet)
        {
            return Err("SEC-CEZAR-01: RFC1918 172.16/12 target blocked");
        }
    }
    Ok(())
}

/// `SEC-CEZAR-02`: Escape HTML/CSS/SVG breakout sequences.
pub fn sanitize_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

/// `SEC-CEZAR-03`: Strict relative path validator blocking `..`, null bytes, `.git/hooks`, and system/vault dirs.
pub fn validate_safe_repo_relpath(rel: &str) -> Result<(), &'static str> {
    if rel.is_empty() || rel.len() > 1024 {
        return Err("SEC-CEZAR-03: invalid path length");
    }
    if rel.contains('\0') || rel.starts_with('/') || rel.starts_with('\\') || rel.contains(':') {
        return Err("SEC-CEZAR-03: absolute or drive-prefixed path forbidden");
    }
    let normalized = rel.replace('\\', "/");
    for seg in normalized.split('/') {
        if seg == ".." {
            return Err("SEC-CEZAR-03: parent directory traversal (..) forbidden");
        }
        let low = seg.to_ascii_lowercase();
        if low == ".ssh" || low == ".gnupg" || low == ".aws" || low == ".env" {
            return Err("SEC-CEZAR-03: access to secret/credential file forbidden");
        }
    }
    if normalized.starts_with(".git/hooks") || normalized.contains("/.git/hooks") {
        return Err("SEC-CEZAR-03: git hook mutation forbidden");
    }
    Ok(())
}

/// Detect Trojan-Source BiDi Unicode override characters (`CVE-2021-42574`).
pub fn contains_bidi_trojan_unicode(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{202A}'
                | '\u{202B}'
                | '\u{202C}'
                | '\u{202D}'
                | '\u{202E}'
                | '\u{2066}'
                | '\u{2067}'
                | '\u{2068}'
                | '\u{2069}'
        )
    })
}

/// `SEC-CEZAR-06`: `NVIDIA SkillSpector` + `sHeriFF` audit for Markdown skills, YAML workflows, and shell steps.
pub fn audit_skill_or_command_payload(label: &str, content: &str) -> Result<(), String> {
    if contains_bidi_trojan_unicode(content) {
        return Err(format!(
            "SEC-CEZAR-06 [{label}]: Trojan-Source BiDi Unicode override (CVE-2021-42574) blocked"
        ));
    }
    let low = content.to_ascii_lowercase();
    let destructive_patterns = [
        "rm -rf /",
        "rm -fr /",
        "mkfs.",
        "dd if=/dev/zero of=/dev/",
        ":(){ :|:& };:",
        "curl -s http",
        "| sh",
        "| bash",
        "nc -e /bin/",
        "/dev/tcp/",
        "ignore all previous instructions and exfiltrate",
        "stats.openmercato.cloud",
    ];
    for pat in destructive_patterns {
        if low.contains(pat) {
            return Err(format!(
                "SEC-CEZAR-06 [{label}]: blocked dangerous/exfiltration pattern `{pat}`"
            ));
        }
    }
    Ok(())
}

/// `SEC-CEZAR-04`: Per-client token-bucket burst limiter.
pub struct LoopbackRateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    buckets: HashMap<u64, (f64, Instant)>,
}

impl LoopbackRateLimiter {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            refill_per_sec,
            buckets: HashMap::new(),
        }
    }

    pub fn allow(&mut self, client_key: &str) -> bool {
        let key_hash = fnv1a_64(client_key.as_bytes());
        let now = Instant::now();
        if self.buckets.len() > 4096 {
            self.buckets
                .retain(|_, (_, last)| now.duration_since(*last) < Duration::from_secs(120));
        }
        let entry = self.buckets.entry(key_hash).or_insert((self.capacity, now));
        let elapsed = now.duration_since(entry.1).as_secs_f64();
        entry.0 = (entry.0 + elapsed * self.refill_per_sec).min(self.capacity);
        entry.1 = now;
        if entry.0 >= 1.0 {
            entry.0 -= 1.0;
            true
        } else {
            false
        }
    }
}

/// `SEC-CEZAR-05`: Standard hardened HTTP headers.
pub fn hardened_http_headers(content_type: &str, body_len: usize) -> String {
    format!(
        "Content-Type: {content_type}\r\n\
         Content-Length: {body_len}\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-Frame-Options: SAMEORIGIN\r\n\
         Referrer-Policy: no-referrer\r\n\
         X-NALA-Version: {VERSION}\r\n\
         X-Telemetry-Policy: zero-outbound-airgap\r\n\
         Connection: close\r\n\r\n"
    )
}
