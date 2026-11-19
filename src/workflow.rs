//! Portable `.ai/cezar/workflows/*.yaml` Chain Parser, Parent->Child Dispatch Governor
//! & Best-of-N (`x2` / `x3` Variants) Tournament Diff Scorer.

use crate::runners::{ReasoningEffort, RunnerId};
use crate::security::audit_skill_or_command_payload;
use crate::sync_gate::{audit_unified_diff, json_escape};
use std::fs;
use std::path::Path;

pub const MAX_CHILDREN_IN_FLIGHT: usize = 4;

#[derive(Debug, Clone)]
pub struct OnFailRule {
    pub retry_step_id: String,
    pub max_retries: u32,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub id: String,
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub skill: Option<String>,
    pub runner: Option<RunnerId>,
    pub model: Option<String>,
    pub reasoning_effort: ReasoningEffort,
    pub command: Option<String>,
    pub on_fail: Option<OnFailRule>,
}

#[derive(Debug, Clone)]
pub struct WorkflowDef {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

impl WorkflowDef {
    pub fn to_json(&self) -> String {
        let steps_json: Vec<String> = self
            .steps
            .iter()
            .map(|s| {
                let r = s.runner.map(|x| format!("\"{}\"", x.as_str())).unwrap_or_else(|| "null".to_string());
                let sk = s.skill.as_deref().map(|x| format!("\"{}\"", json_escape(x))).unwrap_or_else(|| "null".to_string());
                let cmd = s.command.as_deref().map(|x| format!("\"{}\"", json_escape(x))).unwrap_or_else(|| "null".to_string());
                format!(
                    "{{\"id\":\"{}\",\"runner\":{r},\"skill\":{sk},\"effort\":\"{}\",\"command\":{cmd}}}",
                    json_escape(&s.id),
                    s.reasoning_effort.as_str()
                )
            })
            .collect();
        format!(
            "{{\"name\":\"{}\",\"description\":\"{}\",\"steps\":[{}]}}",
            json_escape(&self.name),
            json_escape(&self.description),
            steps_json.join(",")
        )
    }
}

/// Built-in workflows mirroring upstream `open-mercato/cezar` + NALA security gates.
pub fn builtin_workflows() -> Vec<WorkflowDef> {
    vec![
        WorkflowDef {
            name: "implement-and-verify".to_string(),
            description: "Implement task in isolated CoW worktree, run test suite, and audit diff via sHeriFF".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "plan-implement".to_string(),
                    name: Some("Implement".to_string()),
                    prompt: Some("Implement the requested change cleanly with unit tests.".to_string()),
                    skill: None,
                    runner: Some(RunnerId::Codex),
                    model: None,
                    reasoning_effort: ReasoningEffort::High,
                    command: None,
                    on_fail: None,
                },
                WorkflowStep {
                    id: "cargo-verify".to_string(),
                    name: Some("Verify Tests".to_string()),
                    prompt: None,
                    skill: None,
                    runner: None,
                    model: None,
                    reasoning_effort: ReasoningEffort::Medium,
                    command: Some("cargo test --quiet".to_string()),
                    on_fail: Some(OnFailRule {
                        retry_step_id: "plan-implement".to_string(),
                        max_retries: 2,
                    }),
                },
            ],
        },
        WorkflowDef {
            name: "best-of-3-arena".to_string(),
            description: "Run x3 parallel coding agents (Codex, Claude, Local-OpenAI) and rank diffs deterministically".to_string(),
            steps: vec![WorkflowStep {
                id: "arena-implement".to_string(),
                name: Some("Parallel Arena".to_string()),
                prompt: Some("Solve the issue cleanly with minimal diff churn and full test coverage.".to_string()),
                skill: None,
                runner: Some(RunnerId::Claude),
                model: None,
                reasoning_effort: ReasoningEffort::High,
                command: None,
                on_fail: None,
            }],
        },
    ]
}

/// Zero-dependency parser for `.ai/cezar/workflows/*.yaml` (supports both `skills:` shorthand and `steps:` lists).
pub fn parse_workflow_yaml(default_name: &str, yaml_text: &str) -> Result<WorkflowDef, String> {
    audit_skill_or_command_payload(default_name, yaml_text)?;
    let mut name = default_name.to_string();
    let mut description = String::new();
    let mut steps: Vec<WorkflowStep> = Vec::new();
    let mut in_skills_list = false;
    let mut in_steps_list = false;
    let mut current_step: Option<WorkflowStep> = None;

    for raw_line in yaml_text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw_line.len() - raw_line.trim_start().len();

        if indent == 0 {
            if let Some(st) = current_step.take() {
                steps.push(st);
            }
            in_skills_list = false;
            in_steps_list = false;
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim();
                let val = unquote_yaml(v.trim());
                match key {
                    "name" if !val.is_empty() => name = val,
                    "description" => description = val,
                    "skills" => in_skills_list = true,
                    "steps" => in_steps_list = true,
                    _ => {}
                }
            }
            continue;
        }

        if in_skills_list && let Some(item) = trimmed.strip_prefix('-') {
            let skill_name = unquote_yaml(item.trim());
            if !skill_name.is_empty() {
                let idx = steps.len() + 1;
                steps.push(WorkflowStep {
                    id: format!("skill-{idx}-{skill_name}"),
                    name: Some(skill_name.clone()),
                    prompt: None,
                    skill: Some(skill_name),
                    runner: None,
                    model: None,
                    reasoning_effort: ReasoningEffort::Medium,
                    command: None,
                    on_fail: None,
                });
            }
            continue;
        }

        if in_steps_list {
            if let Some(rest) = trimmed.strip_prefix('-') {
                if let Some(prev) = current_step.take() {
                    steps.push(prev);
                }
                let next_idx = steps.len() + 1;
                let mut st = WorkflowStep {
                    id: format!("step-{next_idx}"),
                    name: None,
                    prompt: None,
                    skill: None,
                    runner: None,
                    model: None,
                    reasoning_effort: ReasoningEffort::Medium,
                    command: None,
                    on_fail: None,
                };
                if let Some((k, v)) = rest.trim().split_once(':') {
                    apply_step_kv(&mut st, k.trim(), &unquote_yaml(v.trim()))?;
                }
                current_step = Some(st);
            } else if let Some(ref mut st) = current_step
                && let Some((k, v)) = trimmed.split_once(':')
            {
                apply_step_kv(st, k.trim(), &unquote_yaml(v.trim()))?;
            }
        }
    }

    if let Some(st) = current_step.take() {
        steps.push(st);
    }

    Ok(WorkflowDef {
        name,
        description,
        steps,
    })
}

fn apply_step_kv(step: &mut WorkflowStep, key: &str, val: &str) -> Result<(), String> {
    if val.is_empty() {
        return Ok(());
    }
    match key {
        "id" => step.id = val.to_string(),
        "name" => step.name = Some(val.to_string()),
        "prompt" => step.prompt = Some(val.to_string()),
        "skill" => step.skill = Some(val.to_string()),
        "runner" => step.runner = RunnerId::from_str_loose(val),
        "model" => step.model = Some(val.to_string()),
        "reasoningEffort" | "effort" => step.reasoning_effort = ReasoningEffort::from_str_loose(val),
        "command" => {
            audit_skill_or_command_payload(&step.id, val)?;
            step.command = Some(val.to_string());
        }
        "retry" => {
            step.on_fail = Some(OnFailRule {
                retry_step_id: val.to_string(),
                max_retries: 2,
            });
        }
        _ => {}
    }
    Ok(())
}

fn unquote_yaml(s: &str) -> String {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
    {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

pub fn load_repo_workflows(repo_root: &Path) -> Vec<WorkflowDef> {
    let mut all = builtin_workflows();
    let wf_dir = repo_root.join(".ai").join("cezar").join("workflows");
    if let Ok(entries) = fs::read_dir(&wf_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if (ext == "yaml" || ext == "yml")
                && let Ok(txt) = fs::read_to_string(&p)
            {
                let stem = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "custom".to_string());
                if let Ok(wf) = parse_workflow_yaml(&stem, &txt) {
                    all.push(wf);
                }
            }
        }
    }
    all
}

/// Parent -> Child Task Dispatch Governor (`<= 4` children in flight, shared budget, zero auto-merge).
pub fn validate_child_dispatch(
    active_children: usize,
    parent_budget_usd_micros: u64,
    spent_usd_micros: u64,
) -> Result<(), &'static str> {
    if active_children >= MAX_CHILDREN_IN_FLIGHT {
        return Err("dispatch limit reached: cezar permits at most 4 children in flight per parent");
    }
    if parent_budget_usd_micros > 0 && spent_usd_micros >= parent_budget_usd_micros {
        return Err("dispatch blocked: parent shared token/cost budget exhausted");
    }
    Ok(())
}

/// Candidate input for Best-of-N (`x2` / `x3` Variants) Tournament Diff Scoring.
#[derive(Debug, Clone)]
pub struct VariantCandidate {
    pub variant_id: String,
    pub runner: String,
    pub model: String,
    pub check_exit_code: i32,
    pub unified_diff: String,
    pub duration_ms: u64,
    pub cost_usd_micros: u64,
}

#[derive(Debug, Clone)]
pub struct RankedVariant {
    pub rank: usize,
    pub variant_id: String,
    pub runner: String,
    pub model: String,
    pub score: f64,
    pub sheriff_clean: bool,
    pub checks_passed: bool,
    pub added_lines: usize,
    pub deleted_lines: usize,
    pub tests_added: usize,
}

impl RankedVariant {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"rank\":{},\"variantId\":\"{}\",\"runner\":\"{}\",\"model\":\"{}\",\"score\":{:.2},\"sheriffClean\":{},\"checksPassed\":{},\"addedLines\":{},\"deletedLines\":{},\"testsAdded\":{}}}",
            self.rank,
            json_escape(&self.variant_id),
            json_escape(&self.runner),
            json_escape(&self.model),
            self.score,
            self.sheriff_clean,
            self.checks_passed,
            self.added_lines,
            self.deleted_lines,
            self.tests_added
        )
    }
}

/// Deterministically score & rank parallel `x2`/`x3` variant diffs (`f64::total_cmp`, no auto-merge).
pub fn score_variant_candidates(candidates: &[VariantCandidate]) -> Vec<RankedVariant> {
    let mut scored = Vec::with_capacity(candidates.len());
    for c in candidates {
        let diff_audit = audit_unified_diff(&c.unified_diff);
        let sheriff_clean = diff_audit.passed;
        let checks_passed = c.check_exit_code == 0;

        let mut added_lines = 0usize;
        let mut deleted_lines = 0usize;
        let mut tests_added = 0usize;

        for line in c.unified_diff.lines() {
            if line.starts_with("+++") || line.starts_with("---") {
                continue;
            }
            if line.starts_with('+') {
                added_lines += 1;
                if line.contains("#[test]") || line.contains("it(") || line.contains("test(") || line.contains("def test_") {
                    tests_added += 1;
                }
            } else if line.starts_with('-') {
                deleted_lines += 1;
            }
        }

        let mut score = 0.0f64;
        if sheriff_clean {
            score += 50.0;
        } else {
            score -= 100.0;
        }
        if checks_passed {
            score += 35.0;
        } else {
            score -= 40.0;
        }
        score += (tests_added.min(5) as f64) * 2.5;
        let churn = (added_lines + deleted_lines) as f64;
        let compactness_bonus = if churn > 0.0 {
            (10.0 - (churn / 80.0)).clamp(0.0, 10.0)
        } else {
            0.0
        };
        score += compactness_bonus;
        let speed_bonus = (2.5 - (c.duration_ms as f64 / 60_000.0)).clamp(0.0, 2.5);
        score += speed_bonus;

        scored.push(RankedVariant {
            rank: 0,
            variant_id: c.variant_id.clone(),
            runner: c.runner.clone(),
            model: c.model.clone(),
            score,
            sheriff_clean,
            checks_passed,
            added_lines,
            deleted_lines,
            tests_added,
        });
    }

    scored.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.variant_id.cmp(&b.variant_id)));
    for (idx, item) in scored.iter_mut().enumerate() {
        item.rank = idx + 1;
    }
    scored
}
