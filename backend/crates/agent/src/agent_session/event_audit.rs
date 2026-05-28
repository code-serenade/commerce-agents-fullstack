use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::protocol::{SessionEvent, WorkerReport};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventRoleAudit {
    pub route_mode: Option<String>,
    pub primary_runtime_role: Option<String>,
    pub support_runtime_roles: Vec<String>,
    pub expected_roles: Vec<String>,
    pub route_roles: Vec<String>,
    pub planned_roles: Vec<String>,
    pub reported_roles: Vec<String>,
    pub finished_roles: Vec<String>,
    pub skipped_roles: Vec<String>,
    pub failed_roles: Vec<String>,
    pub incomplete_roles: Vec<String>,
    pub pending_roles: Vec<String>,
    pub status_by_role: BTreeMap<String, Vec<String>>,
    pub missing_expected_planned_roles: Vec<String>,
    pub missing_expected_reported_roles: Vec<String>,
    pub missing_expected_finished_roles: Vec<String>,
    pub missing_route_planned_roles: Vec<String>,
    pub missing_reported_roles: Vec<String>,
    pub missing_finished_roles: Vec<String>,
    pub planned_not_routed_roles: Vec<String>,
    pub reported_not_planned_roles: Vec<String>,
    pub finished_not_planned_roles: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionEventDiagnosticAudit {
    pub failure_count: usize,
    pub search_evidence_count: usize,
    pub skill_failure_count: usize,
    pub context_pack_failure_count: usize,
    pub legacy_failure_marker_count: usize,
    pub failure_kind_counts: BTreeMap<String, usize>,
    pub failure_kinds_by_role: BTreeMap<String, Vec<String>>,
    pub skill_failures: Vec<SessionEventSkillFailure>,
    pub context_pack_failures: Vec<SessionEventContextPackFailure>,
    pub search_diagnostics: Vec<SessionEventSearchDiagnostic>,
    pub legacy_failure_markers: Vec<SessionEventLegacyFailureMarker>,
}

impl SessionEventDiagnosticAudit {
    pub fn has_failures(&self) -> bool {
        self.failure_count > 0 || self.legacy_failure_marker_count > 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventDiagnosticConsistencyOptions {
    pub expect_search_evidence: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventDiagnosticConsistencyAudit {
    pub ok: bool,
    pub issue_count: usize,
    pub issues: Vec<SessionEventDiagnosticConsistencyIssue>,
}

impl SessionEventDiagnosticConsistencyAudit {
    pub fn is_clean(&self) -> bool {
        self.ok && self.issues.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventDiagnosticConsistencyIssue {
    pub code: String,
    pub severity: String,
    pub reason: String,
    pub role: Option<String>,
    pub node_id: Option<String>,
    pub skill: Option<String>,
    pub failure_kind: Option<String>,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventSkillFailure {
    pub role: String,
    pub node_id: String,
    pub skill: String,
    pub status: String,
    pub failure_kind: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventContextPackFailure {
    pub role: String,
    pub node_id: String,
    pub readiness: String,
    pub failure_kind: String,
    pub item_count: usize,
    pub omitted_count: usize,
    pub extraction_pending_count: usize,
    pub extraction_failed_count: usize,
    pub extraction_unsupported_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEventSearchDiagnostic {
    pub role: String,
    pub node_id: String,
    pub skill: String,
    pub status: String,
    pub adapter_status: String,
    pub sources: usize,
    pub usable_claims: usize,
    pub gaps: usize,
    pub stale_filtered_count: usize,
    pub missing_url_filtered_count: usize,
    pub untraceable_domain_filtered_count: usize,
    pub duplicate_filtered_count: usize,
    pub total_filtered_count: usize,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEventLegacyFailureMarker {
    pub role: String,
    pub node_id: String,
    pub failure_kind: String,
}

impl SessionEventRoleAudit {
    pub fn is_clean(&self) -> bool {
        self.missing_expected_planned_roles.is_empty()
            && self.missing_expected_reported_roles.is_empty()
            && self.missing_expected_finished_roles.is_empty()
            && self.missing_route_planned_roles.is_empty()
            && self.missing_reported_roles.is_empty()
            && self.missing_finished_roles.is_empty()
            && self.reported_not_planned_roles.is_empty()
            && self.finished_not_planned_roles.is_empty()
    }

    pub fn has_task_graph(&self) -> bool {
        !self.planned_roles.is_empty() || !self.finished_roles.is_empty()
    }
}

pub fn audit_session_event_roles(events: &[SessionEvent]) -> SessionEventRoleAudit {
    audit_session_event_roles_against::<&str>(events, &[])
}

pub fn audit_session_event_diagnostics(events: &[SessionEvent]) -> SessionEventDiagnosticAudit {
    let mut audit = SessionEventDiagnosticAudit::default();
    for event in events {
        let SessionEvent::TaskNodeReported { report, .. } = event else {
            continue;
        };
        audit_worker_report_diagnostics(&mut audit, report);
    }
    audit
}

pub fn render_session_event_diagnostic_audit(audit: &SessionEventDiagnosticAudit) -> String {
    let mut lines = vec![format!(
        "Agent diagnostic audit: failures={}, search={}, skill_failures={}, \
         context_pack_failures={}, legacy_markers={}",
        audit.failure_count,
        audit.search_evidence_count,
        audit.skill_failure_count,
        audit.context_pack_failure_count,
        audit.legacy_failure_marker_count
    )];
    if !audit.failure_kind_counts.is_empty() {
        let counts = audit
            .failure_kind_counts
            .iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("failure_kind_counts: {counts}"));
    }
    if !audit.failure_kinds_by_role.is_empty() {
        let roles = audit
            .failure_kinds_by_role
            .iter()
            .map(|(role, kinds)| format!("{role}=[{}]", kinds.join(", ")))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("failure_kinds_by_role: {roles}"));
    }
    for item in &audit.skill_failures {
        lines.push(format!(
            "skill_failure: role={} node={} skill={} status={} kind={}",
            item.role, item.node_id, item.skill, item.status, item.failure_kind
        ));
    }
    for item in &audit.context_pack_failures {
        lines.push(format!(
            "context_pack_failure: role={} node={} readiness={} kind={} items={} omitted={} \
             pending={} failed={} unsupported={}",
            item.role,
            item.node_id,
            item.readiness,
            item.failure_kind,
            item.item_count,
            item.omitted_count,
            item.extraction_pending_count,
            item.extraction_failed_count,
            item.extraction_unsupported_count
        ));
    }
    for item in &audit.search_diagnostics {
        lines.push(format!(
            "search_diagnostic: role={} node={} skill={} status={} adapter={} sources={} \
             usable_claims={} gaps={} filtered={} stale={} missing_url={} untraceable={} \
             duplicate={} confidence={}",
            item.role,
            item.node_id,
            item.skill,
            item.status,
            item.adapter_status,
            item.sources,
            item.usable_claims,
            item.gaps,
            item.total_filtered_count,
            item.stale_filtered_count,
            item.missing_url_filtered_count,
            item.untraceable_domain_filtered_count,
            item.duplicate_filtered_count,
            item.confidence
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }
    for item in &audit.legacy_failure_markers {
        lines.push(format!(
            "legacy_failure_marker: role={} node={} kind={}",
            item.role, item.node_id, item.failure_kind
        ));
    }
    lines.join("\n")
}

pub fn audit_session_event_diagnostic_consistency(
    events: &[SessionEvent],
    options: SessionEventDiagnosticConsistencyOptions,
) -> SessionEventDiagnosticConsistencyAudit {
    let output = session_event_output_text(events);
    let diagnostic_audit = audit_session_event_diagnostics(events);
    audit_diagnostic_consistency(&output, &diagnostic_audit, options)
}

pub fn audit_diagnostic_consistency(
    output: &str,
    diagnostics: &SessionEventDiagnosticAudit,
    options: SessionEventDiagnosticConsistencyOptions,
) -> SessionEventDiagnosticConsistencyAudit {
    let mut issues = Vec::new();
    let preview = truncate_chars(output.trim(), 500);
    if preview.is_empty() {
        return SessionEventDiagnosticConsistencyAudit {
            ok: true,
            issue_count: 0,
            issues,
        };
    }

    if diagnostics.context_pack_failure_count > 0
        && contains_any(output, CONTEXT_FULL_READ_CLAIMS)
        && !contains_any(output, CONTEXT_LIMITATION_ACKS)
    {
        let failure = diagnostics.context_pack_failures.first();
        issues.push(SessionEventDiagnosticConsistencyIssue {
            code: "context_pack_failure_overclaimed_read".to_string(),
            severity: "high".to_string(),
            reason: "context_pack_* diagnostic exists, but the final answer claims full \
                     document/file reading"
                .to_string(),
            role: failure.map(|item| item.role.clone()),
            node_id: failure.map(|item| item.node_id.clone()),
            skill: None,
            failure_kind: failure.map(|item| item.failure_kind.clone()),
            preview: preview.clone(),
        });
    }

    for failure in &diagnostics.skill_failures {
        let patterns = skill_completion_claim_patterns(&failure.skill);
        if patterns.is_empty() {
            continue;
        }
        if contains_any(output, patterns) && !contains_any(output, SKILL_LIMITATION_ACKS) {
            issues.push(SessionEventDiagnosticConsistencyIssue {
                code: "skill_failure_overclaimed_execution".to_string(),
                severity: "high".to_string(),
                reason: "skill_* failure diagnostic exists, but the final answer claims an \
                         external action completed"
                    .to_string(),
                role: Some(failure.role.clone()),
                node_id: Some(failure.node_id.clone()),
                skill: Some(failure.skill.clone()),
                failure_kind: Some(failure.failure_kind.clone()),
                preview: preview.clone(),
            });
        }
    }

    if options.expect_search_evidence
        && diagnostics.search_evidence_count == 0
        && contains_any(output, LIVE_SEARCH_CLAIMS)
        && !contains_any(output, SEARCH_LIMITATION_ACKS)
    {
        issues.push(SessionEventDiagnosticConsistencyIssue {
            code: "missing_search_evidence_overclaimed_live_search".to_string(),
            severity: "medium".to_string(),
            reason: "search evidence was expected and no typed search evidence was observed, but \
                     the final answer claims live search"
                .to_string(),
            role: None,
            node_id: None,
            skill: None,
            failure_kind: None,
            preview,
        });
    }

    SessionEventDiagnosticConsistencyAudit {
        ok: issues.is_empty(),
        issue_count: issues.len(),
        issues,
    }
}

pub fn render_session_event_diagnostic_consistency_audit(
    audit: &SessionEventDiagnosticConsistencyAudit,
) -> String {
    let mut lines = vec![format!(
        "Agent diagnostic consistency audit: {} issues={}",
        if audit.is_clean() { "ok" } else { "mismatch" },
        audit.issue_count
    )];
    for issue in &audit.issues {
        lines.push(format!(
            "diagnostic_consistency_issue: code={} severity={} role={} node={} skill={} kind={} \
             reason={}",
            issue.code,
            issue.severity,
            issue.role.as_deref().unwrap_or("<none>"),
            issue.node_id.as_deref().unwrap_or("<none>"),
            issue.skill.as_deref().unwrap_or("<none>"),
            issue.failure_kind.as_deref().unwrap_or("<none>"),
            issue.reason
        ));
    }
    lines.join("\n")
}

pub fn audit_session_event_roles_against<S: AsRef<str>>(
    events: &[SessionEvent],
    expected_roles: &[S],
) -> SessionEventRoleAudit {
    let mut route_mode = None;
    let mut primary_runtime_role = None;
    let mut support_runtime_roles = Vec::new();
    let mut route_roles = BTreeSet::new();
    let mut planned_roles = BTreeSet::new();
    let mut reported_roles = BTreeSet::new();
    let mut finished_roles = BTreeSet::new();
    let mut skipped_roles = BTreeSet::new();
    let mut failed_roles = BTreeSet::new();
    let mut incomplete_roles = BTreeSet::new();
    let mut pending_roles = BTreeSet::new();
    let mut status_by_role = BTreeMap::<String, Vec<String>>::new();

    for event in events {
        match event {
            SessionEvent::RouteDecided {
                route_mode: next_route_mode,
                primary_runtime_role: next_primary_runtime_role,
                support_runtime_roles: next_support_runtime_roles,
                ..
            } => {
                route_mode = Some(next_route_mode.clone());
                primary_runtime_role = next_primary_runtime_role.clone();
                support_runtime_roles = next_support_runtime_roles.clone();
                route_roles.clear();
                if let Some(role) = next_primary_runtime_role {
                    insert_role(&mut route_roles, role);
                }
                for role in next_support_runtime_roles {
                    insert_role(&mut route_roles, role);
                }
            }
            SessionEvent::TaskGraphPlanned { graph, .. } => {
                for node in &graph.nodes {
                    insert_role(&mut planned_roles, &node.role);
                }
            }
            SessionEvent::TaskNodeReported { report, .. } => {
                insert_role(&mut reported_roles, &report.role);
            }
            SessionEvent::TaskGraphFinished { process, .. } => {
                for role in &process.roles {
                    insert_role_status(&mut status_by_role, &role.role, &role.status);
                    match role.status.as_str() {
                        "passed" => {
                            insert_role(&mut finished_roles, &role.role);
                        }
                        "failed" => {
                            insert_role(&mut finished_roles, &role.role);
                            insert_role(&mut failed_roles, &role.role);
                        }
                        "skipped" => {
                            insert_role(&mut finished_roles, &role.role);
                            insert_role(&mut skipped_roles, &role.role);
                        }
                        _ => {
                            insert_role(&mut pending_roles, &role.role);
                        }
                    }
                }
                for gap in &process.incomplete {
                    insert_role(&mut incomplete_roles, &gap.role);
                    match gap.state.as_str() {
                        "failed" => {
                            insert_role(&mut failed_roles, &gap.role);
                        }
                        "skipped" => {
                            insert_role(&mut skipped_roles, &gap.role);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let expected = expected_roles
        .iter()
        .map(|role| role.as_ref().trim())
        .filter(|role| !role.is_empty())
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let report_or_skip = union(&reported_roles, &skipped_roles);

    SessionEventRoleAudit {
        route_mode,
        primary_runtime_role,
        support_runtime_roles,
        expected_roles: sorted_vec(&expected),
        route_roles: sorted_vec(&route_roles),
        planned_roles: sorted_vec(&planned_roles),
        reported_roles: sorted_vec(&reported_roles),
        finished_roles: sorted_vec(&finished_roles),
        skipped_roles: sorted_vec(&skipped_roles),
        failed_roles: sorted_vec(&failed_roles),
        incomplete_roles: sorted_vec(&incomplete_roles),
        pending_roles: sorted_vec(&pending_roles),
        status_by_role,
        missing_expected_planned_roles: sorted_vec(&difference(&expected, &planned_roles)),
        missing_expected_reported_roles: sorted_vec(&difference(&expected, &report_or_skip)),
        missing_expected_finished_roles: sorted_vec(&difference(&expected, &finished_roles)),
        missing_route_planned_roles: sorted_vec(&difference(&route_roles, &planned_roles)),
        missing_reported_roles: sorted_vec(&difference(&planned_roles, &report_or_skip)),
        missing_finished_roles: sorted_vec(&difference(&planned_roles, &finished_roles)),
        planned_not_routed_roles: if route_roles.is_empty() {
            Vec::new()
        } else {
            sorted_vec(&difference(&planned_roles, &route_roles))
        },
        reported_not_planned_roles: sorted_vec(&difference(&reported_roles, &planned_roles)),
        finished_not_planned_roles: sorted_vec(&difference(&finished_roles, &planned_roles)),
    }
}

pub fn render_session_event_role_audit(audit: &SessionEventRoleAudit) -> String {
    let mut lines = vec![format!(
        "Agent role audit: {}",
        if audit.is_clean() { "ok" } else { "mismatch" }
    )];
    lines.push(format!(
        "route: mode={}, primary={}, support=[{}]",
        audit.route_mode.as_deref().unwrap_or("<none>"),
        audit.primary_runtime_role.as_deref().unwrap_or("<none>"),
        audit.support_runtime_roles.join(", ")
    ));
    push_roles(&mut lines, "expected", &audit.expected_roles);
    push_roles(&mut lines, "route_roles", &audit.route_roles);
    push_roles(&mut lines, "planned", &audit.planned_roles);
    push_roles(&mut lines, "reported", &audit.reported_roles);
    push_roles(&mut lines, "finished", &audit.finished_roles);
    push_roles(&mut lines, "skipped", &audit.skipped_roles);
    push_roles(&mut lines, "failed", &audit.failed_roles);
    push_roles(&mut lines, "pending", &audit.pending_roles);
    push_roles(
        &mut lines,
        "missing_expected_planned",
        &audit.missing_expected_planned_roles,
    );
    push_roles(
        &mut lines,
        "missing_expected_reported_or_skipped",
        &audit.missing_expected_reported_roles,
    );
    push_roles(
        &mut lines,
        "missing_expected_finished",
        &audit.missing_expected_finished_roles,
    );
    push_roles(
        &mut lines,
        "missing_route_planned",
        &audit.missing_route_planned_roles,
    );
    push_roles(
        &mut lines,
        "missing_reported_or_skipped",
        &audit.missing_reported_roles,
    );
    push_roles(
        &mut lines,
        "missing_finished",
        &audit.missing_finished_roles,
    );
    push_roles(
        &mut lines,
        "planned_not_routed",
        &audit.planned_not_routed_roles,
    );
    push_roles(
        &mut lines,
        "reported_not_planned",
        &audit.reported_not_planned_roles,
    );
    push_roles(
        &mut lines,
        "finished_not_planned",
        &audit.finished_not_planned_roles,
    );
    if !audit.status_by_role.is_empty() {
        let statuses = audit
            .status_by_role
            .iter()
            .map(|(role, statuses)| format!("{role}={}", statuses.join("|")))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("status_by_role: {statuses}"));
    }
    lines.join("\n")
}

fn audit_worker_report_diagnostics(audit: &mut SessionEventDiagnosticAudit, report: &WorkerReport) {
    let mut has_typed_failure = false;
    let mut has_typed_search_evidence = false;
    for diagnostic in &report.diagnostics {
        match diagnostic.kind.as_str() {
            "failure" => {
                has_typed_failure = true;
                let failure_kind = diagnostic
                    .fields
                    .get("failure_kind")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(diagnostic.source.as_str())
                    .to_string();
                audit.failure_count += 1;
                increment_count(&mut audit.failure_kind_counts, &failure_kind);
                push_unique_string(
                    audit
                        .failure_kinds_by_role
                        .entry(report.role.clone())
                        .or_default(),
                    failure_kind.clone(),
                );
                if failure_kind.starts_with("skill_") {
                    audit.skill_failure_count += 1;
                    audit.skill_failures.push(SessionEventSkillFailure {
                        role: report.role.clone(),
                        node_id: report.node_id.0.clone(),
                        skill: diagnostic_field(diagnostic, "skill", "unknown_skill"),
                        status: diagnostic_field(diagnostic, "status", "unknown"),
                        failure_kind,
                        reason: diagnostic.fields.get("reason").cloned(),
                    });
                } else if failure_kind.starts_with("context_pack_") {
                    audit.context_pack_failure_count += 1;
                    audit
                        .context_pack_failures
                        .push(SessionEventContextPackFailure {
                            role: report.role.clone(),
                            node_id: report.node_id.0.clone(),
                            readiness: diagnostic_field(diagnostic, "readiness", "unknown"),
                            failure_kind,
                            item_count: diagnostic_usize(diagnostic, "item_count"),
                            omitted_count: diagnostic_usize(diagnostic, "omitted_count"),
                            extraction_pending_count: diagnostic_usize(
                                diagnostic,
                                "extraction_pending_count",
                            ),
                            extraction_failed_count: diagnostic_usize(
                                diagnostic,
                                "extraction_failed_count",
                            ),
                            extraction_unsupported_count: diagnostic_usize(
                                diagnostic,
                                "extraction_unsupported_count",
                            ),
                        });
                }
            }
            "search_evidence" => {
                has_typed_search_evidence = true;
                push_search_diagnostic(audit, typed_search_diagnostic(report, diagnostic));
            }
            _ => {}
        }
    }

    if !has_typed_search_evidence {
        for diagnostic in legacy_search_diagnostics(report) {
            push_search_diagnostic(audit, diagnostic);
        }
    }

    if !has_typed_failure {
        for marker in legacy_failure_markers(report) {
            audit.legacy_failure_marker_count += 1;
            increment_count(&mut audit.failure_kind_counts, &marker);
            push_unique_string(
                audit
                    .failure_kinds_by_role
                    .entry(report.role.clone())
                    .or_default(),
                marker.clone(),
            );
            audit
                .legacy_failure_markers
                .push(SessionEventLegacyFailureMarker {
                    role: report.role.clone(),
                    node_id: report.node_id.0.clone(),
                    failure_kind: marker,
                });
        }
    }
}

fn typed_search_diagnostic(
    report: &WorkerReport,
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
) -> SessionEventSearchDiagnostic {
    let stale_filtered_count = diagnostic_usize_any(diagnostic, &["stale_filtered_count", "stale"]);
    let missing_url_filtered_count =
        diagnostic_usize_any(diagnostic, &["missing_url_filtered_count", "missing_url"]);
    let untraceable_domain_filtered_count = diagnostic_usize_any(
        diagnostic,
        &["untraceable_domain_filtered_count", "untraceable"],
    );
    let duplicate_filtered_count =
        diagnostic_usize_any(diagnostic, &["duplicate_filtered_count", "duplicate"]);
    let total_filtered_count =
        diagnostic_usize_any(diagnostic, &["total_filtered_count", "filtered"]).max(
            stale_filtered_count
                + missing_url_filtered_count
                + untraceable_domain_filtered_count
                + duplicate_filtered_count,
        );

    SessionEventSearchDiagnostic {
        role: report.role.clone(),
        node_id: report.node_id.0.clone(),
        skill: diagnostic
            .fields
            .get("skill")
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(diagnostic.source.as_str())
            .to_string(),
        status: diagnostic_field(diagnostic, "status", "unknown"),
        adapter_status: diagnostic
            .fields
            .get("adapter_status")
            .or_else(|| diagnostic.fields.get("status"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        sources: diagnostic_usize(diagnostic, "sources"),
        usable_claims: diagnostic_usize(diagnostic, "usable_claims"),
        gaps: diagnostic_usize(diagnostic, "gaps"),
        stale_filtered_count,
        missing_url_filtered_count,
        untraceable_domain_filtered_count,
        duplicate_filtered_count,
        total_filtered_count,
        confidence: diagnostic
            .fields
            .get("confidence")
            .and_then(|value| value.parse::<f32>().ok()),
    }
}

fn push_search_diagnostic(
    audit: &mut SessionEventDiagnosticAudit,
    diagnostic: SessionEventSearchDiagnostic,
) {
    if audit
        .search_diagnostics
        .iter()
        .any(|item| item == &diagnostic)
    {
        return;
    }
    audit.search_evidence_count += 1;
    audit.search_diagnostics.push(diagnostic);
}

fn diagnostic_field(
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
    key: &str,
    fallback: &str,
) -> String {
    diagnostic
        .fields
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn diagnostic_usize(diagnostic: &crate::core::protocol::WorkerDiagnostic, key: &str) -> usize {
    diagnostic
        .fields
        .get(key)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn diagnostic_usize_any(
    diagnostic: &crate::core::protocol::WorkerDiagnostic,
    keys: &[&str],
) -> usize {
    keys.iter()
        .find_map(|key| diagnostic.fields.get(*key))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn legacy_failure_markers(report: &WorkerReport) -> Vec<String> {
    let mut markers = Vec::new();
    for item in report_evidence_items(report) {
        for part in item.split_whitespace() {
            let marker = part
                .trim_matches(|ch: char| {
                    matches!(ch, ',' | ';' | '"' | '\'' | ')' | '(' | '[' | ']')
                })
                .strip_prefix("failure_kind:");
            if let Some(kind) = marker.map(str::trim).filter(|value| !value.is_empty()) {
                push_unique_string(&mut markers, kind.to_string());
            }
        }
    }
    markers
}

fn legacy_search_diagnostics(report: &WorkerReport) -> Vec<SessionEventSearchDiagnostic> {
    let items = report_evidence_items(report);
    let mut traces = Vec::new();
    for item in &items {
        if let Some(trace) = item
            .find("search_evidence_trace:")
            .map(|index| &item[index + "search_evidence_trace:".len() ..])
            && let Some(diagnostic) = legacy_search_trace_diagnostic(report, trace)
        {
            push_unique_search_diagnostic(&mut traces, diagnostic);
        }
    }
    if !traces.is_empty() {
        return traces;
    }

    let mut packets = Vec::new();
    for item in &items {
        if let Some(packet) = item
            .find("search_evidence_packet:")
            .map(|index| &item[index + "search_evidence_packet:".len() ..])
            && let Some(diagnostic) = legacy_search_packet_diagnostic(report, packet)
        {
            push_unique_search_diagnostic(&mut packets, diagnostic);
        }
    }
    packets
}

fn legacy_search_trace_diagnostic(
    report: &WorkerReport,
    trace: &str,
) -> Option<SessionEventSearchDiagnostic> {
    let fields = parse_legacy_search_trace_fields(trace);
    if fields.is_empty() {
        return None;
    }
    let stale_filtered_count = legacy_usize_any(&fields, &["stale_filtered_count", "stale"]);
    let missing_url_filtered_count =
        legacy_usize_any(&fields, &["missing_url_filtered_count", "missing_url"]);
    let untraceable_domain_filtered_count = legacy_usize_any(
        &fields,
        &["untraceable_domain_filtered_count", "untraceable"],
    );
    let duplicate_filtered_count =
        legacy_usize_any(&fields, &["duplicate_filtered_count", "duplicate"]);
    let total_filtered_count = legacy_usize_any(&fields, &["total_filtered_count", "filtered"])
        .max(
            stale_filtered_count
                + missing_url_filtered_count
                + untraceable_domain_filtered_count
                + duplicate_filtered_count,
        );
    Some(SessionEventSearchDiagnostic {
        role: report.role.clone(),
        node_id: report.node_id.0.clone(),
        skill: legacy_field(&fields, "skill", "search_evidence_trace"),
        status: legacy_field(&fields, "status", "unknown"),
        adapter_status: fields
            .get("adapter_status")
            .or_else(|| fields.get("status"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        sources: legacy_usize(&fields, "sources"),
        usable_claims: legacy_usize(&fields, "usable_claims"),
        gaps: legacy_usize(&fields, "gaps"),
        stale_filtered_count,
        missing_url_filtered_count,
        untraceable_domain_filtered_count,
        duplicate_filtered_count,
        total_filtered_count,
        confidence: fields
            .get("confidence")
            .and_then(|value| value.parse::<f32>().ok()),
    })
}

fn legacy_search_packet_diagnostic(
    report: &WorkerReport,
    packet: &str,
) -> Option<SessionEventSearchDiagnostic> {
    let value: Value = serde_json::from_str(packet.trim()).ok()?;
    let adapter_status = json_string(&value, "adapter_status")
        .or_else(|| json_string(&value, "source_state"))
        .unwrap_or_else(|| "unknown".to_string());
    let stale_filtered_count = json_usize_deep(&value, "stale_filtered_count");
    let missing_url_filtered_count = json_usize_deep(&value, "missing_url_filtered_count");
    let untraceable_domain_filtered_count =
        json_usize_deep(&value, "untraceable_domain_filtered_count");
    let duplicate_filtered_count = json_usize_deep(&value, "duplicate_filtered_count");
    let total_filtered_count = json_usize_deep(&value, "total_filtered_count").max(
        stale_filtered_count
            + missing_url_filtered_count
            + untraceable_domain_filtered_count
            + duplicate_filtered_count,
    );
    Some(SessionEventSearchDiagnostic {
        role: report.role.clone(),
        node_id: report.node_id.0.clone(),
        skill: json_string(&value, "skill").unwrap_or_else(|| "search_evidence_packet".to_string()),
        status: json_string(&value, "status").unwrap_or_else(|| {
            if adapter_status == "live" {
                "success".to_string()
            } else if adapter_status == "unknown" {
                "unknown".to_string()
            } else {
                "deferred".to_string()
            }
        }),
        adapter_status,
        sources: json_array_len(&value, "sources")
            .or_else(|| json_array_len(&value, "evidence"))
            .unwrap_or(0),
        usable_claims: json_array_len(&value, "usable_claims").unwrap_or(0),
        gaps: json_array_len(&value, "gaps").unwrap_or(0),
        stale_filtered_count,
        missing_url_filtered_count,
        untraceable_domain_filtered_count,
        duplicate_filtered_count,
        total_filtered_count,
        confidence: value
            .get("confidence")
            .and_then(Value::as_f64)
            .map(|value| value as f32),
    })
}

fn parse_legacy_search_trace_fields(trace: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for token in trace.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = value
            .trim()
            .trim_matches(|ch: char| matches!(ch, ',' | ';' | '"' | '\''))
            .to_string();
        fields.insert(key.to_string(), value);
    }
    fields
}

fn legacy_field(fields: &BTreeMap<String, String>, key: &str, fallback: &str) -> String {
    fields
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn legacy_usize(fields: &BTreeMap<String, String>, key: &str) -> usize {
    fields
        .get(key)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn legacy_usize_any(fields: &BTreeMap<String, String>, keys: &[&str]) -> usize {
    keys.iter()
        .find_map(|key| fields.get(*key))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_array_len(value: &Value, key: &str) -> Option<usize> {
    value.get(key).and_then(Value::as_array).map(Vec::len)
}

fn json_usize_deep(value: &Value, key: &str) -> usize {
    value
        .pointer(&format!("/filter_stats/{key}"))
        .or_else(|| value.get(key))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(0)
}

fn push_unique_search_diagnostic(
    items: &mut Vec<SessionEventSearchDiagnostic>,
    diagnostic: SessionEventSearchDiagnostic,
) {
    if !items.iter().any(|item| item == &diagnostic) {
        items.push(diagnostic);
    }
}

fn report_evidence_items(report: &WorkerReport) -> Vec<String> {
    let mut items = report.evidence.clone();
    if let Some(role_output) = report.role_output.as_ref() {
        items.extend(role_output.evidence.clone());
    }
    items
}

const CONTEXT_FULL_READ_CLAIMS: &[&str] = &[
    "已读取全文",
    "已读全文",
    "完整读取",
    "读取所有资料",
    "读取所有文件",
    "读取所有附件",
    "所有资料已读取",
    "所有文件已读取",
    "所有附件已读取",
    "全部资料",
    "全部文件",
    "read full file",
    "read full document",
    "read entire file",
    "read entire document",
    "all files read",
    "all documents read",
];

const CONTEXT_LIMITATION_ACKS: &[&str] = &[
    "未能读取全文",
    "没有读取全文",
    "不能当作已读取",
    "解析失败",
    "解析中",
    "解析不支持",
    "未进入上下文",
    "仅元数据",
    "部分资料",
    "not fully read",
    "not read",
    "extraction failed",
    "extraction pending",
    "extraction unsupported",
];

const SKILL_LIMITATION_ACKS: &[&str] = &[
    "未执行",
    "无法执行",
    "执行失败",
    "未接入",
    "不可用",
    "待审批",
    "待确认",
    "需要人工",
    "not executed",
    "execution failed",
    "adapter unavailable",
    "pending approval",
];

const PRICE_UPDATE_CLAIMS: &[&str] = &[
    "已改价",
    "已调价",
    "已更新价格",
    "改价完成",
    "调价完成",
    "价格更新完成",
    "price updated",
    "price changed",
];

const PUBLISH_CLAIMS: &[&str] = &[
    "已发布",
    "已上架",
    "已提交",
    "已同步",
    "发布完成",
    "上架完成",
    "提交完成",
    "同步完成",
    "published successfully",
    "submitted successfully",
    "synced successfully",
    "listed successfully",
];

const AD_ACTION_CLAIMS: &[&str] = &[
    "已投放",
    "已创建广告",
    "已开启广告",
    "投放完成",
    "广告完成",
    "campaign created",
    "campaign launched",
];

const LIVE_SEARCH_CLAIMS: &[&str] = &[
    "已搜索",
    "已检索",
    "实时搜索结果",
    "实时搜索来源",
    "根据实时搜索",
    "根据网页搜索",
    "根据公开搜索",
    "live search",
    "real-time search",
    "real time search",
];

const SEARCH_LIMITATION_ACKS: &[&str] = &[
    "未搜索",
    "没有搜索",
    "无法搜索",
    "搜索失败",
    "缺少实时来源",
    "没有实时来源",
    "证据缺口",
    "no live sources",
    "search failed",
];

fn session_event_output_text(events: &[SessionEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            SessionEvent::Output { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn skill_completion_claim_patterns(skill: &str) -> &'static [&'static str] {
    let skill = skill.to_ascii_lowercase();
    if skill.contains("search") || skill.contains("crawl") || skill.contains("web") {
        LIVE_SEARCH_CLAIMS
    } else if skill.contains("price")
        || skill.contains("sku")
        || skill.contains("platform")
        || skill.contains("update")
    {
        PRICE_UPDATE_CLAIMS
    } else if skill.contains("publish")
        || skill.contains("listing")
        || skill.contains("product")
        || skill.contains("upload")
        || skill.contains("sync")
    {
        PUBLISH_CLAIMS
    } else if skill.contains("ad")
        || skill.contains("ads")
        || skill.contains("douyin")
        || skill.contains("campaign")
        || skill.contains("promotion")
    {
        AD_ACTION_CLAIMS
    } else {
        &[]
    }
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    let lower = text.to_lowercase();
    patterns.iter().any(|pattern| lower.contains(pattern))
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut output = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

fn increment_count(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}

fn push_roles(lines: &mut Vec<String>, label: &str, roles: &[String]) {
    if !roles.is_empty() {
        lines.push(format!("{label}: [{}]", roles.join(", ")));
    }
}

fn insert_role(set: &mut BTreeSet<String>, role: &str) {
    let role = role.trim();
    if !role.is_empty() {
        set.insert(role.to_string());
    }
}

fn insert_role_status(
    status_by_role: &mut BTreeMap<String, Vec<String>>,
    role: &str,
    status: &str,
) {
    let role = role.trim();
    let status = status.trim();
    if role.is_empty() || status.is_empty() {
        return;
    }
    let statuses = status_by_role.entry(role.to_string()).or_default();
    if !statuses.iter().any(|item| item == status) {
        statuses.push(status.to_string());
    }
}

fn push_unique_string(items: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !items.iter().any(|item| item == &value) {
        items.push(value);
    }
}

fn union(left: &BTreeSet<String>, right: &BTreeSet<String>) -> BTreeSet<String> {
    left.union(right).cloned().collect()
}

fn difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> BTreeSet<String> {
    left.difference(right).cloned().collect()
}

fn sorted_vec(values: &BTreeSet<String>) -> Vec<String> {
    values.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::protocol::{
        RoleWorkOutput, SessionId, TaskGraph, TaskGraphId, TaskGraphProcessGap,
        TaskGraphProcessPanel, TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNode,
        TaskNodeId, WorkerDiagnostic, WorkerId, WorkerReport, WorkerReportStatus,
    };

    #[test]
    fn audit_is_clean_when_planned_reported_and_finished_roles_match() {
        let events = vec![
            route_event("task_graph", "ops", &["search"]),
            planned_event(&["search", "ops"]),
            reported_event("search"),
            reported_event("ops"),
            finished_event(&[("search", "passed"), ("ops", "passed")], &[]),
        ];

        let audit = audit_session_event_roles_against(&events, &["search", "ops"]);

        assert!(
            audit.is_clean(),
            "{}",
            render_session_event_role_audit(&audit)
        );
        assert_eq!(audit.route_roles, vec!["ops", "search"]);
        assert_eq!(audit.planned_roles, vec!["ops", "search"]);
        assert_eq!(audit.reported_roles, vec!["ops", "search"]);
        assert_eq!(audit.finished_roles, vec!["ops", "search"]);
    }

    #[test]
    fn audit_detects_planned_role_without_report_or_terminal_status() {
        let events = vec![
            route_event("task_graph", "ops", &["search"]),
            planned_event(&["search", "ops"]),
            reported_event("search"),
            finished_event(&[("search", "passed")], &[]),
        ];

        let audit = audit_session_event_roles_against(&events, &["search", "ops"]);

        assert!(!audit.is_clean());
        assert_eq!(audit.missing_reported_roles, vec!["ops"]);
        assert_eq!(audit.missing_finished_roles, vec!["ops"]);
        assert_eq!(audit.missing_expected_reported_roles, vec!["ops"]);
        assert_eq!(audit.missing_expected_finished_roles, vec!["ops"]);
    }

    #[test]
    fn audit_accepts_explicitly_skipped_role_without_report() {
        let events = vec![
            route_event("task_graph", "ops", &["search"]),
            planned_event(&["search", "ops"]),
            reported_event("search"),
            finished_event(
                &[("search", "passed"), ("ops", "skipped")],
                &[("ops", "skipped")],
            ),
        ];

        let audit = audit_session_event_roles_against(&events, &["search", "ops"]);

        assert!(
            audit.is_clean(),
            "{}",
            render_session_event_role_audit(&audit)
        );
        assert_eq!(audit.skipped_roles, vec!["ops"]);
        assert!(audit.missing_reported_roles.is_empty());
    }

    #[test]
    fn audit_groups_typed_diagnostic_failures_by_kind_and_role() {
        let events = vec![
            reported_event_with(
                "ops",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "skill_validation_error".to_string(),
                    fields: [
                        ("failure_kind", "skill_validation_error"),
                        ("skill", "platform_update_price"),
                        ("status", "validation_error"),
                        ("reason", "missing sku"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
            reported_event_with(
                "data",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "context_pack_extraction_failed".to_string(),
                    fields: [
                        ("failure_kind", "context_pack_extraction_failed"),
                        ("readiness", "partial"),
                        ("item_count", "3"),
                        ("omitted_count", "1"),
                        ("extraction_pending_count", "0"),
                        ("extraction_failed_count", "1"),
                        ("extraction_unsupported_count", "0"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
            reported_event_with(
                "search",
                vec![WorkerDiagnostic {
                    kind: "search_evidence".to_string(),
                    source: "search_web".to_string(),
                    fields: [
                        ("skill", "search_web"),
                        ("status", "success"),
                        ("adapter_status", "mock"),
                        ("sources", "2"),
                        ("usable_claims", "4"),
                        ("gaps", "1"),
                        ("stale_filtered_count", "2"),
                        ("missing_url_filtered_count", "1"),
                        ("untraceable_domain_filtered_count", "1"),
                        ("duplicate_filtered_count", "1"),
                        ("total_filtered_count", "5"),
                        ("confidence", "0.72"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
        ];

        let audit = audit_session_event_diagnostics(&events);

        assert!(audit.has_failures());
        assert_eq!(audit.failure_count, 2);
        assert_eq!(audit.skill_failure_count, 1);
        assert_eq!(audit.context_pack_failure_count, 1);
        assert_eq!(audit.search_evidence_count, 1);
        assert_eq!(
            audit.failure_kind_counts.get("skill_validation_error"),
            Some(&1)
        );
        assert_eq!(
            audit
                .failure_kind_counts
                .get("context_pack_extraction_failed"),
            Some(&1)
        );
        assert_eq!(
            audit.failure_kinds_by_role.get("ops"),
            Some(&vec!["skill_validation_error".to_string()])
        );
        assert_eq!(audit.skill_failures[0].skill, "platform_update_price");
        assert_eq!(audit.context_pack_failures[0].readiness, "partial");
        assert_eq!(audit.context_pack_failures[0].item_count, 3);
        assert_eq!(audit.search_diagnostics[0].sources, 2);
        assert_eq!(audit.search_diagnostics[0].stale_filtered_count, 2);
        assert_eq!(audit.search_diagnostics[0].missing_url_filtered_count, 1);
        assert_eq!(
            audit.search_diagnostics[0].untraceable_domain_filtered_count,
            1
        );
        assert_eq!(audit.search_diagnostics[0].duplicate_filtered_count, 1);
        assert_eq!(audit.search_diagnostics[0].total_filtered_count, 5);
        assert_eq!(audit.search_diagnostics[0].confidence, Some(0.72));
        let rendered = render_session_event_diagnostic_audit(&audit);
        assert!(rendered.contains("skill_failure: role=ops"));
        assert!(rendered.contains("context_pack_failure: role=data"));
        assert!(rendered.contains("search_diagnostic: role=search"));
        assert!(rendered.contains("filtered=5 stale=2 missing_url=1 untraceable=1 duplicate=1"));
    }

    #[test]
    fn audit_uses_legacy_failure_marker_only_when_typed_failure_is_absent() {
        let events = vec![
            reported_event_with(
                "ops",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "skill_validation_error".to_string(),
                    fields: [("failure_kind", "skill_validation_error")]
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect(),
                }],
                vec!["failure_kind:legacy_should_not_count".to_string()],
                Vec::new(),
            ),
            reported_event_with(
                "data",
                Vec::new(),
                vec!["trace failure_kind:legacy_context_issue".to_string()],
                vec!["nested failure_kind:legacy_skill_issue".to_string()],
            ),
        ];

        let audit = audit_session_event_diagnostics(&events);

        assert_eq!(audit.failure_count, 1);
        assert_eq!(audit.legacy_failure_marker_count, 2);
        assert!(
            !audit
                .failure_kind_counts
                .contains_key("legacy_should_not_count")
        );
        assert_eq!(
            audit.failure_kind_counts.get("skill_validation_error"),
            Some(&1)
        );
        assert_eq!(
            audit.failure_kind_counts.get("legacy_context_issue"),
            Some(&1)
        );
        assert_eq!(
            audit.failure_kind_counts.get("legacy_skill_issue"),
            Some(&1)
        );
        assert_eq!(
            audit
                .legacy_failure_markers
                .iter()
                .map(|item| item.failure_kind.as_str())
                .collect::<Vec<_>>(),
            vec!["legacy_context_issue", "legacy_skill_issue"]
        );
    }

    #[test]
    fn audit_counts_legacy_search_trace_when_typed_search_diagnostic_is_absent() {
        let events = vec![reported_event_with(
            "search",
            Vec::new(),
            vec![
                "search_evidence_trace: skill=search_competitor status=success \
                 adapter_status=live intent=competitor_price_or_benchmark strength=standard \
                 sources=2 usable_claims=1 gaps=0 confidence=0.72 filtered=4 stale=1 \
                 missing_url=1 untraceable=1 duplicate=1 query=\"校园冷萃咖啡 竞品价格\""
                    .to_string(),
            ],
            Vec::new(),
        )];

        let audit = audit_session_event_diagnostics(&events);

        assert_eq!(audit.search_evidence_count, 1);
        assert_eq!(audit.search_diagnostics[0].skill, "search_competitor");
        assert_eq!(audit.search_diagnostics[0].status, "success");
        assert_eq!(audit.search_diagnostics[0].adapter_status, "live");
        assert_eq!(audit.search_diagnostics[0].sources, 2);
        assert_eq!(audit.search_diagnostics[0].usable_claims, 1);
        assert_eq!(audit.search_diagnostics[0].gaps, 0);
        assert_eq!(audit.search_diagnostics[0].stale_filtered_count, 1);
        assert_eq!(audit.search_diagnostics[0].missing_url_filtered_count, 1);
        assert_eq!(
            audit.search_diagnostics[0].untraceable_domain_filtered_count,
            1
        );
        assert_eq!(audit.search_diagnostics[0].duplicate_filtered_count, 1);
        assert_eq!(audit.search_diagnostics[0].total_filtered_count, 4);
        assert_eq!(audit.search_diagnostics[0].confidence, Some(0.72));
    }

    #[test]
    fn audit_counts_legacy_search_packet_when_trace_is_absent() {
        let events = vec![reported_event_with(
            "search",
            Vec::new(),
            vec![
                r#"search_evidence_packet:{"skill":"search_market_info","status":"deferred","adapter_status":"degraded","sources":[],"usable_claims":[],"gaps":["provider timeout"],"filter_stats":{"missing_url_filtered_count":1,"untraceable_domain_filtered_count":2,"duplicate_filtered_count":0,"stale_filtered_count":3,"total_filtered_count":6},"confidence":0.0}"#
                    .to_string(),
            ],
            Vec::new(),
        )];

        let audit = audit_session_event_diagnostics(&events);

        assert_eq!(audit.search_evidence_count, 1);
        assert_eq!(audit.search_diagnostics[0].skill, "search_market_info");
        assert_eq!(audit.search_diagnostics[0].status, "deferred");
        assert_eq!(audit.search_diagnostics[0].adapter_status, "degraded");
        assert_eq!(audit.search_diagnostics[0].sources, 0);
        assert_eq!(audit.search_diagnostics[0].usable_claims, 0);
        assert_eq!(audit.search_diagnostics[0].gaps, 1);
        assert_eq!(audit.search_diagnostics[0].stale_filtered_count, 3);
        assert_eq!(audit.search_diagnostics[0].missing_url_filtered_count, 1);
        assert_eq!(
            audit.search_diagnostics[0].untraceable_domain_filtered_count,
            2
        );
        assert_eq!(audit.search_diagnostics[0].duplicate_filtered_count, 0);
        assert_eq!(audit.search_diagnostics[0].total_filtered_count, 6);
        assert_eq!(audit.search_diagnostics[0].confidence, Some(0.0));
    }

    #[test]
    fn diagnostic_consistency_accepts_legacy_search_trace_as_search_evidence() {
        let events = vec![
            reported_event_with(
                "search",
                Vec::new(),
                vec![
                    "search_evidence_trace: skill=search_competitor status=success \
                     adapter_status=live sources=1 usable_claims=1 gaps=0 confidence=0.80"
                        .to_string(),
                ],
                Vec::new(),
            ),
            output_event("根据实时搜索结果，竞品价格需要继续核验。"),
        ];

        let audit = audit_session_event_diagnostic_consistency(
            &events,
            SessionEventDiagnosticConsistencyOptions {
                expect_search_evidence: true,
            },
        );

        assert!(audit.is_clean(), "{audit:?}");
    }

    #[test]
    fn diagnostic_consistency_flags_context_pack_full_read_overclaim() {
        let events = vec![
            reported_event_with(
                "data",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "context_pack_extraction_failed".to_string(),
                    fields: [
                        ("failure_kind", "context_pack_extraction_failed"),
                        ("readiness", "partial"),
                        ("item_count", "3"),
                        ("extraction_failed_count", "1"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
            output_event("我已读取全文，并基于所有资料给出结论。"),
        ];

        let audit = audit_session_event_diagnostic_consistency(
            &events,
            SessionEventDiagnosticConsistencyOptions::default(),
        );

        assert!(!audit.is_clean());
        assert_eq!(audit.issue_count, 1);
        assert_eq!(
            audit.issues[0].code,
            "context_pack_failure_overclaimed_read"
        );
        assert_eq!(audit.issues[0].severity, "high");
    }

    #[test]
    fn diagnostic_consistency_flags_skill_execution_overclaim() {
        let events = vec![
            reported_event_with(
                "ops",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "skill_adapter_unavailable".to_string(),
                    fields: [
                        ("failure_kind", "skill_adapter_unavailable"),
                        ("skill", "platform_update_price"),
                        ("status", "deferred"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
            output_event("已更新价格，后续可以观察转化率。"),
        ];

        let audit = audit_session_event_diagnostic_consistency(
            &events,
            SessionEventDiagnosticConsistencyOptions::default(),
        );

        assert!(!audit.is_clean());
        assert_eq!(audit.issue_count, 1);
        assert_eq!(audit.issues[0].code, "skill_failure_overclaimed_execution");
        assert_eq!(
            audit.issues[0].skill.as_deref(),
            Some("platform_update_price")
        );
    }

    #[test]
    fn diagnostic_consistency_allows_explicit_limitation_ack() {
        let events = vec![
            reported_event_with(
                "ops",
                vec![WorkerDiagnostic {
                    kind: "failure".to_string(),
                    source: "skill_adapter_unavailable".to_string(),
                    fields: [
                        ("failure_kind", "skill_adapter_unavailable"),
                        ("skill", "platform_update_price"),
                        ("status", "deferred"),
                    ]
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect(),
                }],
                Vec::new(),
                Vec::new(),
            ),
            output_event("价格更新未执行，因为当前 adapter 不可用；下面只给操作建议。"),
        ];

        let audit = audit_session_event_diagnostic_consistency(
            &events,
            SessionEventDiagnosticConsistencyOptions::default(),
        );

        assert!(audit.is_clean(), "{audit:?}");
    }

    fn route_event(route_mode: &str, primary: &str, support: &[&str]) -> SessionEvent {
        SessionEvent::RouteDecided {
            session_id: session_id(),
            strategy_id: None,
            response_mode: None,
            collaboration_mode: None,
            learning_level: None,
            route_mode: route_mode.to_string(),
            primary_runtime_role: Some(primary.to_string()),
            support_runtime_roles: support.iter().map(|role| role.to_string()).collect(),
        }
    }

    fn planned_event(roles: &[&str]) -> SessionEvent {
        SessionEvent::TaskGraphPlanned {
            session_id: session_id(),
            graph: TaskGraph {
                graph_id: graph_id(),
                root_task: "audit test".to_string(),
                nodes: roles
                    .iter()
                    .map(|role| TaskNode {
                        id: TaskNodeId((*role).to_string()),
                        role: (*role).to_string(),
                        worker_id: WorkerId(format!("worker.{role}")),
                        title: (*role).to_string(),
                        objective: "audit".to_string(),
                        llm_usage: None,
                        skill_autosuggest: None,
                        timeout_sec: None,
                        input_refs: Vec::new(),
                        acceptance: Vec::new(),
                        max_attempts: 1,
                    })
                    .collect(),
            },
        }
    }

    fn reported_event(role: &str) -> SessionEvent {
        reported_event_with(role, Vec::new(), Vec::new(), Vec::new())
    }

    fn reported_event_with(
        role: &str,
        diagnostics: Vec<WorkerDiagnostic>,
        evidence: Vec<String>,
        role_output_evidence: Vec<String>,
    ) -> SessionEvent {
        SessionEvent::TaskNodeReported {
            session_id: session_id(),
            graph_id: graph_id(),
            node_id: TaskNodeId(role.to_string()),
            worker_id: WorkerId(format!("worker.{role}")),
            report: Box::new(WorkerReport {
                graph_id: graph_id(),
                node_id: TaskNodeId(role.to_string()),
                task_id: TaskId("task.audit".to_string()),
                worker_id: WorkerId(format!("worker.{role}")),
                role: role.to_string(),
                content: "done".to_string(),
                role_output: Some(RoleWorkOutput {
                    evidence: role_output_evidence,
                    ..RoleWorkOutput::default()
                }),
                evidence,
                risks: Vec::new(),
                open_questions: Vec::new(),
                diagnostics,
                status: WorkerReportStatus::Completed,
            }),
        }
    }

    fn output_event(content: &str) -> SessionEvent {
        SessionEvent::Output {
            session_id: session_id(),
            content: content.to_string(),
        }
    }

    fn finished_event(roles: &[(&str, &str)], gaps: &[(&str, &str)]) -> SessionEvent {
        SessionEvent::TaskGraphFinished {
            session_id: session_id(),
            graph_id: graph_id(),
            process: Box::new(TaskGraphProcessSummary {
                graph_id: graph_id(),
                root_task: "audit test".to_string(),
                primary: None,
                roles: roles
                    .iter()
                    .map(|(role, status)| process_role(role, status))
                    .collect(),
                handoffs: Vec::new(),
                next_actions: Vec::new(),
                check_items: Vec::new(),
                deliverables: Vec::new(),
                risks: Vec::new(),
                open_questions: Vec::new(),
                incomplete: gaps
                    .iter()
                    .map(|(role, state)| TaskGraphProcessGap {
                        node_id: TaskNodeId((*role).to_string()),
                        role: (*role).to_string(),
                        role_label: (*role).to_string(),
                        title: (*role).to_string(),
                        state: (*state).to_string(),
                        reason: "test gap".to_string(),
                    })
                    .collect(),
                panel: TaskGraphProcessPanel {
                    default_collapsed: false,
                    anchor: "task_graph_process".to_string(),
                    label: "process".to_string(),
                    reason: "test".to_string(),
                },
            }),
        }
    }

    fn process_role(role: &str, status: &str) -> TaskGraphProcessRole {
        TaskGraphProcessRole {
            node_id: TaskNodeId(role.to_string()),
            task_id: Some(TaskId("task.audit".to_string())),
            worker_id: WorkerId(format!("worker.{role}")),
            role: role.to_string(),
            role_label: role.to_string(),
            title: role.to_string(),
            objective: "audit".to_string(),
            status: status.to_string(),
            attempt: 1,
            is_primary: false,
            summary: None,
            finding: None,
            recommendation: None,
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
        }
    }

    fn session_id() -> SessionId {
        SessionId("session.audit".to_string())
    }

    fn graph_id() -> TaskGraphId {
        TaskGraphId("graph.audit".to_string())
    }
}
