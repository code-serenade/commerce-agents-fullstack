use std::collections::{HashMap, HashSet};

use db_core::{
    DbContext, PaginatedResponse, PaginationParams,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::{
    entity::student_learning_evidence,
    table::{
        app_users::{AppUser, AppUserService},
        student_learning_evidence::StudentLearningEvidenceService,
        student_learning_probes::{StudentLearningProbe, StudentLearningProbeService},
        student_learning_profiles::{StudentLearningProfile, StudentLearningProfileService},
        student_learning_question_gaps::{
            StudentLearningQuestionGap, StudentLearningQuestionGapService,
        },
    },
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    api::{
        admin::AdminApi,
        learning_progress::{map_evidence_entry, summarize_assessments},
        learning_question_bank::{
            DiagnosticQuestion, QuestionBank, assessment_angle_depth,
            production_approved_question_identities, question_applicability_scope, question_bank,
        },
    },
    dto::learning_progress::{
        LearningAdminMetrics, LearningAdminSortBy, LearningAdminStudentDetail,
        LearningAdminStudentRow, LearningAssessmentSummary, LearningDomainMastery,
        LearningProbeChoiceMetrics, LearningProbePilotMetrics, LearningProbeQuestionMetrics,
        LearningQuestionCoverageGapDimensionMetric, LearningSortOrder,
        ListLearningAdminStudentsRequest,
    },
};

const PERM_LEARNING_READ: &str = "learning:read";

pub struct LearningAdminApi {
    admin_api: AdminApi,
    app_user_svc: AppUserService,
    profile_svc: StudentLearningProfileService,
    evidence_svc: StudentLearningEvidenceService,
    probe_svc: StudentLearningProbeService,
    question_gap_svc: StudentLearningQuestionGapService,
}

impl LearningAdminApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            admin_api: AdminApi::new(db.clone()),
            app_user_svc: AppUserService::new(db.clone()),
            profile_svc: StudentLearningProfileService::new(db.clone()),
            evidence_svc: StudentLearningEvidenceService::new(db.clone()),
            probe_svc: StudentLearningProbeService::new(db.clone()),
            question_gap_svc: StudentLearningQuestionGapService::new(db),
        }
    }

    pub async fn list_students(
        &self,
        current_admin_user_id: String,
        req: ListLearningAdminStudentsRequest,
    ) -> BizResult<PaginatedResponse<LearningAdminStudentRow>> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_LEARNING_READ)
            .await?;
        let profiles = self.profile_svc.list_all().await?;
        let profiles_by_user = profiles
            .iter()
            .map(|profile| (profile.user_id, profile))
            .collect::<HashMap<_, _>>();
        let evidence_by_user = group_evidence_by_user(self.evidence_svc.list_all().await?);
        let retention_review_interval_days =
            framework()?.probe_policy.retention_review_interval_days;
        let users = self.app_user_svc.list_all().await?;
        let mut rows = users
            .into_iter()
            .map(|user| {
                let profile = profiles_by_user.get(&user.user_id);
                let evidence = evidence_by_user
                    .get(&user.user_id)
                    .map_or(&[][..], Vec::as_slice);
                map_row(
                    user,
                    profile.copied(),
                    evidence,
                    retention_review_interval_days,
                )
            })
            .collect::<Vec<_>>();

        if let Some(keyword) = normalize_optional(req.keyword) {
            let keyword = keyword.to_ascii_lowercase();
            rows.retain(|row| {
                row.user_id.to_ascii_lowercase().contains(&keyword)
                    || row.display_id.to_ascii_lowercase().contains(&keyword)
                    || row.display_name.to_ascii_lowercase().contains(&keyword)
                    || row.school_name.to_ascii_lowercase().contains(&keyword)
                    || row.team_name.to_ascii_lowercase().contains(&keyword)
            });
        }

        sort_rows(&mut rows, req.sort_by, req.sort_order);
        Ok(paginate_rows(
            rows,
            PaginationParams::new(req.page, req.page_size),
        ))
    }

    pub async fn get_student_detail(
        &self,
        current_admin_user_id: String,
        user_id: String,
    ) -> BizResult<LearningAdminStudentDetail> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_LEARNING_READ)
            .await?;
        let user_id = parse_user_id(&user_id)?;
        let user = self.app_user_svc.get_by_user_id(user_id).await?;
        let profile = self.profile_svc.get_by_user_id(user_id).await?;
        let evidence = self.evidence_svc.list_by_user(user_id).await?;
        let probes_by_id = self
            .probe_svc
            .list_by_ids(evidence.iter().filter_map(|entry| entry.probe_id).collect())
            .await?
            .into_iter()
            .filter(|probe| probe.user_id == user_id)
            .map(|probe| (probe.id, probe))
            .collect::<HashMap<_, _>>();
        let retention_review_interval_days =
            framework()?.probe_policy.retention_review_interval_days;
        Ok(map_detail(
            user,
            profile,
            evidence,
            &probes_by_id,
            retention_review_interval_days,
        ))
    }

    pub async fn get_metrics(
        &self,
        current_admin_user_id: String,
    ) -> BizResult<LearningAdminMetrics> {
        self.admin_api
            .ensure_current_user_permission(current_admin_user_id, PERM_LEARNING_READ)
            .await?;
        let profiles = self.profile_svc.list_all().await?;
        let users = self.app_user_svc.list_all().await?;
        let evidence = self.evidence_svc.list_all().await?;
        let bank = question_bank()?;
        let angle_depth = assessment_angle_depth(&bank)?;
        let production_approvals = production_approved_question_identities(&bank)?;
        let probe_pilot = summarize_probe_pilot(
            &self.probe_svc.list_all().await?,
            &evidence,
            &self.question_gap_svc.list_all().await?,
            &bank,
            angle_depth,
            &production_approvals,
            time::OffsetDateTime::now_utc(),
        )?;
        let evidence_by_user = group_evidence_by_user(evidence);
        let retention_review_interval_days =
            framework()?.probe_policy.retention_review_interval_days;
        let total = users.len() as u64;
        let disabled_user_ids = profiles
            .iter()
            .filter(|profile| !profile.enabled)
            .map(|profile| profile.user_id)
            .collect::<std::collections::HashSet<_>>();
        let enabled = users
            .iter()
            .filter(|user| !disabled_user_ids.contains(&user.user_id))
            .count() as u64;
        let observed = profiles
            .iter()
            .filter(|profile| mastery_count(&profile.mastery_map) > 0)
            .count() as u64;
        let avg_coverage_rate = if profiles.is_empty() {
            0.0
        } else {
            profiles
                .iter()
                .map(|profile| profile.coverage_rate)
                .sum::<f64>()
                / profiles.len() as f64
        };
        let profiles_by_user = profiles
            .iter()
            .map(|profile| (profile.user_id, profile))
            .collect::<HashMap<_, _>>();
        let assessment_summaries = users
            .iter()
            .map(|user| {
                let evidence = evidence_by_user
                    .get(&user.user_id)
                    .map_or(&[][..], Vec::as_slice);
                assessment_summary(
                    profiles_by_user.get(&user.user_id).copied(),
                    evidence,
                    retention_review_interval_days,
                )
            })
            .collect::<Vec<_>>();
        let assessment_counts = assessment_student_counts(&assessment_summaries);
        Ok(LearningAdminMetrics {
            total,
            enabled,
            observed,
            avg_coverage_rate,
            assessed: assessment_counts.assessed,
            demonstrated: assessment_counts.demonstrated,
            retained: assessment_counts.retained,
            needs_attention: assessment_counts.needs_attention,
            probe_pilot,
        })
    }
}

fn map_row(
    user: AppUser,
    profile: Option<&StudentLearningProfile>,
    evidence: &[student_learning_evidence::Model],
    retention_review_interval_days: i64,
) -> LearningAdminStudentRow {
    let observed_count = profile.map_or(0, |p| mastery_count(&p.mastery_map));
    let assessment_summary = assessment_summary(profile, evidence, retention_review_interval_days);
    LearningAdminStudentRow {
        user_id: user.user_id.to_string(),
        display_id: user.display_id,
        display_name: user.display_name,
        school_name: user.school_name.clone().unwrap_or_default(),
        team_name: user.team_name.clone().unwrap_or_default(),
        coverage_rate: profile.map_or(0.0, |p| p.coverage_rate),
        observed_knowledge_count: observed_count,
        mastery_status: if observed_count > 0 {
            "observed".to_owned()
        } else {
            "unobserved".to_owned()
        },
        assessment_summary,
        enabled: profile.is_none_or(|p| p.enabled),
        last_analyzed_at: profile.and_then(|p| p.last_analyzed_at),
        created_at: user.created_at,
        updated_at: profile.map_or(user.updated_at, |p| p.updated_at),
    }
}

fn map_detail(
    user: Option<AppUser>,
    profile: Option<StudentLearningProfile>,
    evidence: Vec<student_learning_evidence::Model>,
    probes_by_id: &HashMap<Uuid, StudentLearningProbe>,
    retention_review_interval_days: i64,
) -> LearningAdminStudentDetail {
    let user = user.map(|u| {
        (
            u.user_id.to_string(),
            u.display_id,
            u.display_name,
            u.school_name.unwrap_or_default(),
            u.team_name.unwrap_or_default(),
            u.created_at,
            u.updated_at,
        )
    });
    let observed_count = profile
        .as_ref()
        .map_or(0, |p| mastery_count(&p.mastery_map));
    let assessment_summary =
        assessment_summary(profile.as_ref(), &evidence, retention_review_interval_days);
    let domains = framework().map_or_else(|_| Vec::new(), |f| f.domains);
    let mastery_map = profile
        .as_ref()
        .map(|p| p.mastery_map.clone())
        .unwrap_or_else(|| Value::Object(Default::default()));
    let mastery_summary = domains
        .into_iter()
        .map(|domain| {
            let (status, strength) = domain_status_and_strength(&mastery_map, &domain.code);
            LearningDomainMastery {
                knowledge_code: domain.code,
                status,
                strength,
            }
        })
        .collect();
    let evidence_summary = evidence
        .into_iter()
        .map(|entry| {
            let probe = entry
                .probe_id
                .and_then(|probe_id| probes_by_id.get(&probe_id));
            map_evidence_entry(entry, probe)
        })
        .collect();

    LearningAdminStudentDetail {
        user_id: user.as_ref().map_or_else(String::new, |u| u.0.clone()),
        display_id: user.as_ref().map_or_else(String::new, |u| u.1.clone()),
        display_name: user.as_ref().map_or_else(String::new, |u| u.2.clone()),
        school_name: user.as_ref().map_or_else(String::new, |u| u.3.clone()),
        team_name: user.as_ref().map_or_else(String::new, |u| u.4.clone()),
        coverage_rate: profile.as_ref().map_or(0.0, |p| p.coverage_rate),
        observed_knowledge_count: observed_count,
        mastery_status: if observed_count > 0 {
            "observed".to_owned()
        } else {
            "unobserved".to_owned()
        },
        assessment_summary,
        enabled: profile.as_ref().is_some_and(|p| p.enabled),
        last_analyzed_at: profile.as_ref().and_then(|p| p.last_analyzed_at),
        mastery_summary,
        evidence_summary,
    }
}

fn group_evidence_by_user(
    evidence: Vec<student_learning_evidence::Model>,
) -> HashMap<Uuid, Vec<student_learning_evidence::Model>> {
    let mut evidence_by_user = HashMap::new();
    for entry in evidence {
        evidence_by_user
            .entry(entry.user_id)
            .or_insert_with(Vec::new)
            .push(entry);
    }
    evidence_by_user
}

fn assessment_summary(
    profile: Option<&StudentLearningProfile>,
    evidence: &[student_learning_evidence::Model],
    retention_review_interval_days: i64,
) -> LearningAssessmentSummary {
    match profile {
        Some(profile) => summarize_assessments(
            &profile.mastery_map,
            evidence,
            retention_review_interval_days,
        ),
        None => summarize_assessments(&Value::Null, evidence, retention_review_interval_days),
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct AssessmentStudentCounts {
    assessed: u64,
    demonstrated: u64,
    retained: u64,
    needs_attention: u64,
}

fn assessment_student_counts(summaries: &[LearningAssessmentSummary]) -> AssessmentStudentCounts {
    summaries
        .iter()
        .fold(AssessmentStudentCounts::default(), |mut counts, summary| {
            counts.assessed += u64::from(summary.assessed_knowledge_count > 0);
            counts.demonstrated += u64::from(summary.demonstrated_knowledge_count > 0);
            counts.retained += u64::from(summary.retained_knowledge_count > 0);
            counts.needs_attention += u64::from(summary.needs_attention_knowledge_count > 0);
            counts
        })
}

#[derive(Debug, Default)]
struct ProbeMetricAccumulator {
    issued_count: u64,
    active_count: u64,
    answered_count: u64,
    skipped_count: u64,
    expired_count: u64,
    superseded_count: u64,
    correct_count: u64,
    partial_count: u64,
    incorrect_count: u64,
    unscorable_count: u64,
    answer_seconds_total: u128,
    answer_seconds_count: u64,
    choice_counts: HashMap<String, u64>,
    unmapped_choice_count: u64,
}

impl ProbeMetricAccumulator {
    fn record_probe(&mut self, probe: &StudentLearningProbe, now: time::OffsetDateTime) {
        self.issued_count += 1;
        let effectively_expired = matches!(probe.status.as_str(), "offered" | "planned")
            && probe.expires_at.is_some_and(|expires_at| expires_at <= now);
        match probe.status.as_str() {
            "answered" => {
                self.answered_count += 1;
                if let (Some(offered_at), Some(answered_at)) = (probe.offered_at, probe.answered_at)
                    && answered_at >= offered_at
                {
                    self.answer_seconds_total += (answered_at - offered_at).whole_seconds() as u128;
                    self.answer_seconds_count += 1;
                }
            }
            "skipped" => self.skipped_count += 1,
            "expired" => self.expired_count += 1,
            "superseded" => self.superseded_count += 1,
            "offered" | "planned" if effectively_expired => self.expired_count += 1,
            "offered" | "planned" => self.active_count += 1,
            _ => {}
        }
    }

    fn record_outcome(&mut self, outcome: Option<&str>) {
        match outcome {
            Some("correct") => self.correct_count += 1,
            Some("partial") => self.partial_count += 1,
            Some("incorrect") => self.incorrect_count += 1,
            Some("unscorable") => self.unscorable_count += 1,
            _ => {}
        }
    }

    fn record_choice(&mut self, choice_id: Option<&str>, question: &DiagnosticQuestion) {
        let Some(choice_id) = choice_id.map(str::trim).filter(|value| !value.is_empty()) else {
            self.unmapped_choice_count += 1;
            return;
        };
        if question.choices.iter().any(|choice| choice.id == choice_id) {
            *self.choice_counts.entry(choice_id.to_owned()).or_default() += 1;
        } else {
            self.unmapped_choice_count += 1;
        }
    }

    fn answer_rate(&self) -> f64 {
        ratio(
            self.answered_count,
            self.answered_count + self.skipped_count + self.expired_count,
        )
    }

    fn skip_rate(&self) -> f64 {
        ratio(
            self.skipped_count,
            self.answered_count + self.skipped_count + self.expired_count,
        )
    }

    fn correct_rate(&self) -> f64 {
        ratio(
            self.correct_count,
            self.correct_count + self.partial_count + self.incorrect_count,
        )
    }

    fn average_answer_seconds(&self) -> u64 {
        if self.answer_seconds_count == 0 {
            0
        } else {
            u64::try_from(self.answer_seconds_total / u128::from(self.answer_seconds_count))
                .unwrap_or(u64::MAX)
        }
    }
}

fn structured_probe_question_identity(probe: &StudentLearningProbe) -> Option<(&str, u32)> {
    let question_id = probe
        .context_snapshot
        .get("questionId")
        .and_then(Value::as_str)?;
    let question_version = u32::try_from(
        probe
            .context_snapshot
            .get("questionVersion")
            .and_then(Value::as_u64)?,
    )
    .ok()?;
    Some((question_id, question_version))
}

fn map_question_metrics(
    question: &DiagnosticQuestion,
    counts: &ProbeMetricAccumulator,
    production_approvals: &HashSet<(String, u32)>,
) -> BizResult<LearningProbeQuestionMetrics> {
    let applicability = question_applicability_scope(&question.id, question.version)
        .ok_or_else(|| invalid("learning diagnostic applicability scope missing"))?;
    let mut source_publishers = question
        .sources
        .iter()
        .map(|source| source.publisher.trim().to_owned())
        .filter(|publisher| !publisher.is_empty())
        .collect::<Vec<_>>();
    source_publishers.sort();
    source_publishers.dedup();
    let mapped_choice_count = counts.choice_counts.values().sum();
    let choice_metrics = question
        .choices
        .iter()
        .map(|choice| {
            let selected_count = counts.choice_counts.get(&choice.id).copied().unwrap_or(0);
            LearningProbeChoiceMetrics {
                choice_id: choice.id.clone(),
                choice_text: choice.text.clone(),
                selected_count,
                selected_rate: ratio(selected_count, mapped_choice_count),
            }
        })
        .collect();
    Ok(LearningProbeQuestionMetrics {
        question_id: question.id.clone(),
        question_version: question.version,
        knowledge_code: question.domain_code.clone(),
        question_text: question.prompt.clone(),
        jurisdiction_code: applicability.jurisdiction_code.clone(),
        platform_code: applicability.platform_code.clone(),
        business_stage_code: applicability.business_stage_code.clone(),
        source_publishers,
        source_valid_through: question_source_valid_through(question),
        curriculum_review_required: question.review.curriculum_review_required,
        production_approved: production_approvals
            .contains(&(question.id.clone(), question.version)),
        issued_count: counts.issued_count,
        active_count: counts.active_count,
        answered_count: counts.answered_count,
        skipped_count: counts.skipped_count,
        expired_count: counts.expired_count,
        superseded_count: counts.superseded_count,
        correct_count: counts.correct_count,
        partial_count: counts.partial_count,
        incorrect_count: counts.incorrect_count,
        unscorable_count: counts.unscorable_count,
        answer_rate: counts.answer_rate(),
        skip_rate: counts.skip_rate(),
        correct_rate: counts.correct_rate(),
        average_answer_seconds: counts.average_answer_seconds(),
        choice_metrics,
        unmapped_choice_count: counts.unmapped_choice_count,
    })
}

fn summarize_probe_pilot(
    probes: &[StudentLearningProbe],
    evidence: &[student_learning_evidence::Model],
    question_gaps: &[StudentLearningQuestionGap],
    bank: &QuestionBank,
    angle_depth: (u64, u64),
    production_approvals: &HashSet<(String, u32)>,
    now: time::OffsetDateTime,
) -> BizResult<LearningProbePilotMetrics> {
    let questions = &bank.questions;
    let question_indices = questions
        .iter()
        .enumerate()
        .map(|(index, question)| ((question.id.as_str(), question.version), index))
        .collect::<HashMap<_, _>>();
    let mut question_counts = (0..questions.len())
        .map(|_| ProbeMetricAccumulator::default())
        .collect::<Vec<_>>();
    let mut structured_probe_ids = HashSet::new();
    let mut probe_question_indices = HashMap::new();
    let mut probe_selected_choice_ids = HashMap::new();
    let mut total = ProbeMetricAccumulator::default();
    let mut continuation_count = 0;

    for probe in probes.iter().filter(|probe| {
        probe
            .context_snapshot
            .get("answerMode")
            .and_then(Value::as_str)
            == Some("structured_choice")
    }) {
        structured_probe_ids.insert(probe.id);
        total.record_probe(probe, now);
        continuation_count += u64::from(probe.reason_code == "student_continued_assessment");
        if let Some(identity) = structured_probe_question_identity(probe)
            && let Some(index) = question_indices.get(&identity).copied()
        {
            question_counts[index].record_probe(probe, now);
            probe_question_indices.insert(probe.id, index);
            probe_selected_choice_ids.insert(
                probe.id,
                probe
                    .context_snapshot
                    .get("selectedChoiceId")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            );
        }
    }

    for entry in evidence.iter().filter(|entry| {
        entry
            .probe_id
            .is_some_and(|probe_id| structured_probe_ids.contains(&probe_id))
    }) {
        let outcome = entry.rubric_hits.get("outcome").and_then(Value::as_str);
        total.record_outcome(outcome);
        if let Some(probe_id) = entry.probe_id
            && let Some(index) = probe_question_indices.get(&probe_id).copied()
        {
            question_counts[index].record_outcome(outcome);
            let selected_choice_id = entry
                .rubric_hits
                .get("selectedChoiceId")
                .and_then(Value::as_str)
                .or_else(|| {
                    probe_selected_choice_ids
                        .get(&probe_id)
                        .and_then(Option::as_deref)
                });
            question_counts[index].record_choice(selected_choice_id, &questions[index]);
        }
    }

    let question_metrics = questions
        .iter()
        .zip(&question_counts)
        .map(|(question, counts)| map_question_metrics(question, counts, production_approvals))
        .collect::<BizResult<Vec<_>>>()?;
    let coverage_gap_domain_metrics =
        summarize_gap_dimension(question_gaps.iter().map(|gap| gap.knowledge_code.as_str()));
    let coverage_gap_reason_metrics =
        summarize_gap_dimension(question_gaps.iter().map(|gap| gap.reason_code.as_str()));
    Ok(LearningProbePilotMetrics {
        question_bank_version: bank.version.clone(),
        question_count: questions.len() as u64,
        assessment_angle_count: angle_depth.0,
        single_question_angle_count: angle_depth.1,
        production_approved_count: production_approvals.len() as u64,
        curriculum_review_pending_count: questions
            .iter()
            .filter(|question| {
                question.review.curriculum_review_required
                    && !production_approvals.contains(&(question.id.clone(), question.version))
            })
            .count() as u64,
        earliest_source_valid_through: questions
            .iter()
            .filter_map(question_source_valid_through)
            .min(),
        issued_count: total.issued_count,
        active_count: total.active_count,
        answered_count: total.answered_count,
        skipped_count: total.skipped_count,
        expired_count: total.expired_count,
        superseded_count: total.superseded_count,
        continuation_count,
        correct_count: total.correct_count,
        partial_count: total.partial_count,
        incorrect_count: total.incorrect_count,
        unscorable_count: total.unscorable_count,
        answer_rate: total.answer_rate(),
        skip_rate: total.skip_rate(),
        correct_rate: total.correct_rate(),
        average_answer_seconds: total.average_answer_seconds(),
        coverage_gap_count: question_gaps.len() as u64,
        coverage_gap_turn_count: question_gaps
            .iter()
            .map(|gap| gap.assistant_message_id)
            .collect::<HashSet<_>>()
            .len() as u64,
        coverage_gap_domain_metrics,
        coverage_gap_reason_metrics,
        question_metrics,
    })
}

fn summarize_gap_dimension<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Vec<LearningQuestionCoverageGapDimensionMetric> {
    let mut counts = HashMap::<String, u64>::new();
    for value in values {
        *counts.entry(value.to_owned()).or_default() += 1;
    }
    let mut metrics = counts
        .into_iter()
        .map(|(code, count)| LearningQuestionCoverageGapDimensionMetric { code, count })
        .collect::<Vec<_>>();
    metrics.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.code.cmp(&right.code))
    });
    metrics
}

fn question_source_valid_through(question: &DiagnosticQuestion) -> Option<String> {
    question
        .sources
        .iter()
        .map(|source| source.valid_through.trim())
        .filter(|valid_through| !valid_through.is_empty())
        .min()
        .map(str::to_owned)
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn mastery_count(mastery_map: &Value) -> usize {
    mastery_map.as_object().map_or(0, |map| map.len())
}

fn domain_status_and_strength(mastery_map: &Value, code: &str) -> (String, Option<f64>) {
    let Some(entry) = mastery_map.get(code) else {
        return ("unobserved".to_owned(), None);
    };
    match entry {
        Value::Object(map) => {
            let status = map
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("observed")
                .to_owned();
            let strength = map.get("strength").and_then(Value::as_f64);
            (status, strength)
        }
        Value::String(status) => (status.clone(), None),
        Value::Number(number) => ("observed".to_owned(), number.as_f64()),
        _ => ("observed".to_owned(), None),
    }
}

fn sort_rows(
    rows: &mut [LearningAdminStudentRow],
    sort_by: Option<LearningAdminSortBy>,
    sort_order: Option<LearningSortOrder>,
) {
    let Some(sort_by) = sort_by else {
        return;
    };
    match sort_by {
        LearningAdminSortBy::CoverageRate => {
            rows.sort_by(|a, b| a.coverage_rate.total_cmp(&b.coverage_rate))
        }
        LearningAdminSortBy::ObservedKnowledgeCount => {
            rows.sort_by_key(|row| row.observed_knowledge_count)
        }
        LearningAdminSortBy::LastAnalyzedAt => rows.sort_by(|a, b| {
            a.last_analyzed_at
                .cmp(&b.last_analyzed_at)
                .then_with(|| a.updated_at.cmp(&b.updated_at))
        }),
        LearningAdminSortBy::UpdatedAt => rows.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
    }
    if !matches!(sort_order, Some(LearningSortOrder::Asc)) {
        rows.reverse();
    }
}

fn paginate_rows(
    rows: Vec<LearningAdminStudentRow>,
    pagination: PaginationParams,
) -> PaginatedResponse<LearningAdminStudentRow> {
    let page = pagination.validate();
    let limit = page.limit();
    let total = rows.len() as u64;
    let total_pages = if total == 0 { 0 } else { total.div_ceil(limit) };
    let offset = ((page.page - 1) * limit) as usize;
    let items = rows
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .collect::<Vec<_>>();
    PaginatedResponse {
        items,
        page: page.page,
        page_size: limit,
        total,
        total_pages,
        has_next: page.page < total_pages,
        has_prev: page.page > 1 && total_pages > 0,
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

fn parse_user_id(value: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| invalid("invalid user id"))
}

fn invalid(message: &str) -> BizError {
    BizError::new(app_error::LEARNING_REQUEST_INVALID, message.to_owned())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Framework {
    probe_policy: ProbePolicy,
    domains: Vec<FrameworkDomain>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProbePolicy {
    retention_review_interval_days: i64,
}

#[derive(Debug, Deserialize)]
struct FrameworkDomain {
    code: String,
}

fn framework() -> BizResult<Framework> {
    serde_json::from_str(include_str!(
        "../../../../config/json/learning/ecommerce-knowledge-framework.v1.json"
    ))
    .map_err(|_| invalid("learning framework invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo::table::app_users::AppUserStatus;
    use serde_json::json;
    use time::{Duration, OffsetDateTime};

    fn test_user(user_id: Uuid) -> AppUser {
        let now = OffsetDateTime::now_utc();
        AppUser {
            user_id,
            display_id: "student-1".to_owned(),
            display_name: "Student One".to_owned(),
            school_name: None,
            team_name: None,
            remark: None,
            status: AppUserStatus::Enabled,
            created_at: now,
            updated_at: now,
        }
    }

    fn test_profile(user_id: Uuid) -> StudentLearningProfile {
        let now = OffsetDateTime::now_utc();
        StudentLearningProfile {
            id: Uuid::new_v4(),
            user_id,
            framework_version: "ecommerce_v1".to_owned(),
            mastery_map: json!({
                "market": {
                    "status": "observed",
                    "strength": 0.6,
                    "evidenceCount": 1
                }
            }),
            engagement_summary: json!({}),
            coverage_rate: 0.125,
            analysis_status: "done".to_owned(),
            last_analyzed_sequence: 1,
            last_analyzed_at: Some(now),
            last_error_code: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn test_evidence(
        user_id: Uuid,
        outcome: &str,
        question_id: Option<&str>,
        created_at: OffsetDateTime,
    ) -> student_learning_evidence::Model {
        let mut rubric_hits = json!({ "outcome": outcome });
        if let Some(question_id) = question_id {
            rubric_hits["questionId"] = json!(question_id);
        }
        student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id,
            conversation_id: Uuid::new_v4(),
            source_message_id: Some(Uuid::new_v4()),
            probe_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "market".to_owned(),
            evidence_type: "explanation".to_owned(),
            direction: "neutral".to_owned(),
            strength: 0.5,
            confidence: 0.8,
            summary: "Observed in conversation".to_owned(),
            rubric_hits,
            analyzer_version: "test".to_owned(),
            created_at,
        }
    }

    fn test_structured_probe(
        user_id: Uuid,
        conversation_id: Uuid,
        probe_id: Uuid,
        created_at: OffsetDateTime,
    ) -> StudentLearningProbe {
        StudentLearningProbe {
            id: probe_id,
            user_id,
            conversation_id: Some(conversation_id),
            assistant_message_id: Some(Uuid::new_v4()),
            answer_message_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "conversion".to_owned(),
            status: "answered".to_owned(),
            question_text: "用户进入结账流程时，GA4 推荐使用哪个电商事件？".to_owned(),
            reason_code: "context_assessment_turn".to_owned(),
            priority_score: 1.0,
            context_snapshot: json!({
                "answerMode": "structured_choice",
                "questionBankVersion": "ecommerce_diagnostic_v6",
                "questionId": "conversion.ga4_begin_checkout.choice.003",
                "questionVersion": 1,
                "rubricVersion": 1,
                "selectedChoiceId": "begin_checkout"
            }),
            offered_at: Some(created_at),
            answered_at: Some(created_at),
            expires_at: None,
            created_at,
            updated_at: created_at,
        }
    }

    fn test_question_gap(
        knowledge_code: &str,
        reason_code: &str,
        assistant_message_id: Uuid,
    ) -> StudentLearningQuestionGap {
        let now = OffsetDateTime::now_utc();
        StudentLearningQuestionGap {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            assistant_message_id,
            framework_version: "ecommerce_v1".to_owned(),
            selector_version: "ecommerce_diagnostic_question_selector_v1".to_owned(),
            knowledge_code: knowledge_code.to_owned(),
            reason_code: reason_code.to_owned(),
            topic_signals: json!([]),
            scope_signals: json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn admin_row_keeps_observation_out_of_structured_assessment() {
        let user_id = Uuid::new_v4();
        let profile = test_profile(user_id);
        let evidence = vec![test_evidence(
            user_id,
            "observed_unscored",
            None,
            OffsetDateTime::now_utc(),
        )];

        let row = map_row(test_user(user_id), Some(&profile), &evidence, 7);

        assert_eq!(row.observed_knowledge_count, 1);
        assert_eq!(row.mastery_status, "observed");
        assert_eq!(row.assessment_summary, LearningAssessmentSummary::default());
    }

    #[test]
    fn admin_row_uses_the_student_assessment_rules_for_structured_evidence() {
        let user_id = Uuid::new_v4();
        let profile = test_profile(user_id);
        let now = OffsetDateTime::now_utc();
        let evidence = vec![
            test_evidence(
                user_id,
                "correct",
                Some("market.customer_research_mix.choice.007"),
                now,
            ),
            test_evidence(
                user_id,
                "partial",
                Some("market.competitor_analysis_scope.choice.008"),
                now - Duration::days(8),
            ),
        ];

        let row = map_row(test_user(user_id), Some(&profile), &evidence, 7);

        assert_eq!(row.assessment_summary.assessed_knowledge_count, 1);
        assert_eq!(row.assessment_summary.assessment_count, 2);
        assert_eq!(row.assessment_summary.multi_angle_knowledge_count, 1);
        assert_eq!(row.assessment_summary.demonstrated_knowledge_count, 1);
        assert_eq!(row.assessment_summary.retained_knowledge_count, 1);
    }

    #[test]
    fn admin_detail_recovers_structured_fields_from_probe_snapshot() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let probe_id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        let mut evidence = test_evidence(
            user_id,
            "correct",
            Some("conversion.ga4_begin_checkout.choice.003"),
            created_at,
        );
        evidence.conversation_id = conversation_id;
        evidence.source_message_id = None;
        evidence.probe_id = Some(probe_id);
        evidence.knowledge_code = "conversion".to_owned();
        evidence.evidence_type = "probe_answer".to_owned();
        evidence.direction = "positive".to_owned();
        evidence.strength = 1.0;
        evidence.summary = "历史结构化摘要".to_owned();
        evidence.analyzer_version = "deterministic_question_bank_v1".to_owned();
        let probes = HashMap::from([(
            probe_id,
            test_structured_probe(user_id, conversation_id, probe_id, created_at),
        )]);

        let detail = map_detail(
            Some(test_user(user_id)),
            Some(test_profile(user_id)),
            vec![evidence],
            &probes,
            7,
        );
        let entry = &detail.evidence_summary[0];

        assert_eq!(
            entry.question_text.as_deref(),
            Some("用户进入结账流程时，GA4 推荐使用哪个电商事件？")
        );
        assert_eq!(entry.selected_choice_id.as_deref(), Some("begin_checkout"));
        assert_eq!(
            entry.selected_choice_text.as_deref(),
            Some("begin_checkout")
        );
    }

    #[test]
    fn admin_detail_keeps_chat_observation_summary_only() {
        let user_id = Uuid::new_v4();
        let evidence = test_evidence(
            user_id,
            "observed_unscored",
            None,
            OffsetDateTime::now_utc(),
        );

        let detail = map_detail(
            Some(test_user(user_id)),
            Some(test_profile(user_id)),
            vec![evidence],
            &HashMap::new(),
            7,
        );
        let entry = &detail.evidence_summary[0];

        assert_eq!(entry.summary, "Observed in conversation");
        assert!(entry.question_text.is_none());
        assert!(entry.selected_choice_id.is_none());
        assert!(entry.selected_choice_text.is_none());
    }

    #[test]
    fn admin_detail_keeps_missing_legacy_probe_compatible() {
        let user_id = Uuid::new_v4();
        let mut evidence = test_evidence(
            user_id,
            "correct",
            Some("conversion.ga4_begin_checkout.choice.003"),
            OffsetDateTime::now_utc(),
        );
        evidence.probe_id = Some(Uuid::new_v4());
        evidence.evidence_type = "probe_answer".to_owned();
        evidence.summary = "无法补齐 probe 的历史摘要".to_owned();

        let detail = map_detail(
            Some(test_user(user_id)),
            Some(test_profile(user_id)),
            vec![evidence],
            &HashMap::new(),
            7,
        );
        let entry = &detail.evidence_summary[0];

        assert_eq!(entry.summary, "无法补齐 probe 的历史摘要");
        assert!(entry.question_text.is_none());
        assert!(entry.selected_choice_id.is_none());
        assert!(entry.selected_choice_text.is_none());
    }

    #[test]
    fn admin_metrics_count_students_instead_of_evidence_rows() {
        let summaries = vec![
            LearningAssessmentSummary {
                assessed_knowledge_count: 2,
                demonstrated_knowledge_count: 1,
                retained_knowledge_count: 1,
                assessment_count: 8,
                ..LearningAssessmentSummary::default()
            },
            LearningAssessmentSummary {
                assessed_knowledge_count: 1,
                needs_attention_knowledge_count: 1,
                assessment_count: 4,
                incorrect_count: 3,
                ..LearningAssessmentSummary::default()
            },
            LearningAssessmentSummary::default(),
        ];

        assert_eq!(
            assessment_student_counts(&summaries),
            AssessmentStudentCounts {
                assessed: 2,
                demonstrated: 1,
                retained: 1,
                needs_attention: 1,
            }
        );
    }

    #[test]
    fn probe_pilot_metrics_use_structured_resolved_and_scorable_denominators() {
        let now = OffsetDateTime::now_utc();
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let mut probes = Vec::new();
        let mut evidence = Vec::new();

        let mut push_answered =
            |outcome: &str, seconds: i64, continuation: bool, valid_timing: bool| {
                let probe_id = Uuid::new_v4();
                let mut probe = test_structured_probe(user_id, conversation_id, probe_id, now);
                probe.offered_at = Some(now - Duration::seconds(seconds));
                probe.answered_at = Some(if valid_timing {
                    now
                } else {
                    now - Duration::seconds(seconds + 1)
                });
                if continuation {
                    probe.reason_code = "student_continued_assessment".to_owned();
                }
                probes.push(probe);
                let mut entry = test_evidence(user_id, outcome, Some("market.q1"), now);
                entry.probe_id = Some(probe_id);
                entry.source_message_id = None;
                entry.evidence_type = "probe_answer".to_owned();
                evidence.push(entry);
            };

        push_answered("correct", 60, false, true);
        push_answered("partial", 120, false, true);
        push_answered("incorrect", 180, true, true);
        push_answered("unscorable", 30, false, false);

        let mut active = test_structured_probe(user_id, conversation_id, Uuid::new_v4(), now);
        active.status = "offered".to_owned();
        active.answered_at = None;
        active.expires_at = Some(now + Duration::hours(1));
        probes.push(active);

        let mut stale_offered =
            test_structured_probe(user_id, conversation_id, Uuid::new_v4(), now);
        stale_offered.status = "offered".to_owned();
        stale_offered.answered_at = None;
        stale_offered.expires_at = Some(now - Duration::seconds(1));
        probes.push(stale_offered);

        for status in ["skipped", "expired", "superseded"] {
            let mut probe = test_structured_probe(user_id, conversation_id, Uuid::new_v4(), now);
            probe.status = status.to_owned();
            probe.answered_at = None;
            probes.push(probe);
        }

        let open_probe_id = Uuid::new_v4();
        let mut open_probe = test_structured_probe(user_id, conversation_id, open_probe_id, now);
        open_probe.context_snapshot = json!({ "answerMode": "open_response" });
        probes.push(open_probe);
        let mut open_evidence = test_evidence(user_id, "correct", Some("legacy.q1"), now);
        open_evidence.probe_id = Some(open_probe_id);
        evidence.push(open_evidence);

        let bank = question_bank().expect("current question bank");
        let angle_depth = assessment_angle_depth(&bank).expect("assessment angle depth");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");
        let metrics = summarize_probe_pilot(
            &probes,
            &evidence,
            &[],
            &bank,
            angle_depth,
            &production_approvals,
            now,
        )
        .expect("probe pilot metrics");

        assert_eq!(metrics.question_bank_version, "ecommerce_diagnostic_v13");
        assert_eq!(metrics.question_count, 100);
        assert_eq!(metrics.assessment_angle_count, 67);
        assert_eq!(metrics.single_question_angle_count, 44);
        assert_eq!(metrics.production_approved_count, 0);
        assert_eq!(metrics.curriculum_review_pending_count, 100);
        assert_eq!(
            metrics.earliest_source_valid_through.as_deref(),
            Some("2027-02-27")
        );
        assert_eq!(metrics.issued_count, 9);
        assert_eq!(metrics.active_count, 1);
        assert_eq!(metrics.answered_count, 4);
        assert_eq!(metrics.skipped_count, 1);
        assert_eq!(metrics.expired_count, 2);
        assert_eq!(metrics.superseded_count, 1);
        assert_eq!(metrics.continuation_count, 1);
        assert_eq!(metrics.correct_count, 1);
        assert_eq!(metrics.partial_count, 1);
        assert_eq!(metrics.incorrect_count, 1);
        assert_eq!(metrics.unscorable_count, 1);
        assert!((metrics.answer_rate - (4.0 / 7.0)).abs() < f64::EPSILON);
        assert!((metrics.skip_rate - (1.0 / 7.0)).abs() < f64::EPSILON);
        assert!((metrics.correct_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
        assert_eq!(metrics.average_answer_seconds, 120);
        assert_eq!(metrics.question_metrics.len(), 100);
        let question = metrics
            .question_metrics
            .iter()
            .find(|question| question.question_id == "conversion.ga4_begin_checkout.choice.003")
            .expect("current question metrics");
        assert_eq!(question.knowledge_code, "conversion");
        assert_eq!(question.issued_count, 9);
        assert_eq!(question.active_count, 1);
        assert_eq!(question.answered_count, 4);
        assert_eq!(question.skipped_count, 1);
        assert_eq!(question.expired_count, 2);
        assert_eq!(question.superseded_count, 1);
        assert_eq!(question.correct_count, 1);
        assert_eq!(question.partial_count, 1);
        assert_eq!(question.incorrect_count, 1);
        assert_eq!(question.unscorable_count, 1);
        assert!((question.answer_rate - (4.0 / 7.0)).abs() < f64::EPSILON);
        assert!((question.skip_rate - (1.0 / 7.0)).abs() < f64::EPSILON);
        assert!((question.correct_rate - (1.0 / 3.0)).abs() < f64::EPSILON);
        assert_eq!(question.average_answer_seconds, 120);
        assert_eq!(question.choice_metrics.len(), 4);
        let selected = question
            .choice_metrics
            .iter()
            .find(|choice| choice.choice_id == "begin_checkout")
            .expect("selected choice metrics");
        assert_eq!(selected.selected_count, 4);
        assert_eq!(selected.selected_rate, 1.0);
        assert_eq!(question.unmapped_choice_count, 0);
    }

    #[test]
    fn probe_pilot_metrics_return_current_question_catalog_with_stable_zeroes() {
        let bank = question_bank().expect("current question bank");
        let angle_depth = assessment_angle_depth(&bank).expect("assessment angle depth");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");
        let metrics = summarize_probe_pilot(
            &[],
            &[],
            &[],
            &bank,
            angle_depth,
            &production_approvals,
            OffsetDateTime::now_utc(),
        )
        .expect("probe pilot metrics");

        assert_eq!(metrics.question_bank_version, "ecommerce_diagnostic_v13");
        assert_eq!(metrics.question_count, 100);
        assert_eq!(metrics.assessment_angle_count, 67);
        assert_eq!(metrics.single_question_angle_count, 44);
        assert_eq!(metrics.production_approved_count, 0);
        assert_eq!(metrics.curriculum_review_pending_count, 100);
        assert_eq!(
            metrics.earliest_source_valid_through.as_deref(),
            Some("2027-02-27")
        );
        assert_eq!(metrics.issued_count, 0);
        assert_eq!(metrics.correct_count, 0);
        assert_eq!(metrics.answer_rate, 0.0);
        assert_eq!(metrics.average_answer_seconds, 0);
        assert_eq!(metrics.question_metrics.len(), 100);
        assert!(metrics.question_metrics.iter().all(|question| {
            !question.question_id.is_empty()
                && question.question_version == 1
                && !question.knowledge_code.is_empty()
                && !question.question_text.is_empty()
                && !question.source_publishers.is_empty()
                && question.source_valid_through.is_some()
                && question.curriculum_review_required
                && !question.production_approved
                && question.issued_count == 0
                && question.correct_count == 0
                && question.answer_rate == 0.0
                && question.choice_metrics.len() == 4
                && question
                    .choice_metrics
                    .iter()
                    .all(|choice| choice.selected_count == 0 && choice.selected_rate == 0.0)
                && question.unmapped_choice_count == 0
        }));
        let serialized = serde_json::to_value(&metrics).expect("serialized metrics");
        assert_eq!(serialized["assessmentAngleCount"], 67);
        assert_eq!(serialized["singleQuestionAngleCount"], 44);
        let question = &serialized["questionMetrics"][0];
        assert!(question["jurisdictionCode"].is_string());
        assert!(question["platformCode"].is_string());
        assert!(question["businessStageCode"].is_string());
        assert!(question.get("correctChoiceId").is_none());
        assert!(question.get("referenceAnswer").is_none());
        assert!(question.get("rubric").is_none());
        assert!(question.get("assessmentAngleCode").is_none());
        assert!(question.get("reviewerId").is_none());
        assert!(
            question["choiceMetrics"]
                .as_array()
                .is_some_and(|choices| choices.iter().all(|choice| {
                    choice.get("correctChoiceId").is_none() && choice.get("isCorrect").is_none()
                }))
        );
    }

    #[test]
    fn probe_pilot_metrics_summarize_privacy_safe_question_coverage_gaps() {
        let bank = question_bank().expect("current question bank");
        let angle_depth = assessment_angle_depth(&bank).expect("assessment angle depth");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");
        let gaps = vec![
            test_question_gap("product", "no_contextual_question", Uuid::new_v4()),
            test_question_gap("product", "no_deliverable_question", Uuid::new_v4()),
            test_question_gap("traffic", "no_contextual_question", Uuid::new_v4()),
        ];

        let metrics = summarize_probe_pilot(
            &[],
            &[],
            &gaps,
            &bank,
            angle_depth,
            &production_approvals,
            OffsetDateTime::now_utc(),
        )
        .expect("probe pilot metrics");

        assert_eq!(metrics.coverage_gap_count, 3);
        assert_eq!(metrics.coverage_gap_turn_count, 3);
        assert_eq!(
            metrics.coverage_gap_domain_metrics,
            vec![
                LearningQuestionCoverageGapDimensionMetric {
                    code: "product".to_owned(),
                    count: 2,
                },
                LearningQuestionCoverageGapDimensionMetric {
                    code: "traffic".to_owned(),
                    count: 1,
                },
            ]
        );
        assert_eq!(
            metrics.coverage_gap_reason_metrics[0].code,
            "no_contextual_question"
        );
        assert_eq!(metrics.coverage_gap_reason_metrics[0].count, 2);

        let serialized = serde_json::to_value(metrics).expect("serialized metrics");
        assert!(serialized.get("topicSignals").is_none());
        assert!(serialized.get("conversationId").is_none());
        assert!(serialized.get("assistantMessageId").is_none());
    }

    #[test]
    fn choice_distribution_prefers_evidence_and_separates_unmapped_choices() {
        let now = OffsetDateTime::now_utc();
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let mut probes = Vec::new();
        let mut evidence = Vec::new();

        for selected_choice_id in [Some("view_item"), None, Some("unknown_choice")] {
            let probe_id = Uuid::new_v4();
            probes.push(test_structured_probe(
                user_id,
                conversation_id,
                probe_id,
                now,
            ));
            let mut entry = test_evidence(user_id, "incorrect", None, now);
            entry.probe_id = Some(probe_id);
            entry.source_message_id = None;
            entry.evidence_type = "probe_answer".to_owned();
            if let Some(choice_id) = selected_choice_id {
                entry.rubric_hits["selectedChoiceId"] = json!(choice_id);
            }
            evidence.push(entry);
        }

        let bank = question_bank().expect("current question bank");
        let angle_depth = assessment_angle_depth(&bank).expect("assessment angle depth");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");
        let metrics = summarize_probe_pilot(
            &probes,
            &evidence,
            &[],
            &bank,
            angle_depth,
            &production_approvals,
            now,
        )
        .expect("probe pilot metrics");
        let question = metrics
            .question_metrics
            .iter()
            .find(|question| question.question_id == "conversion.ga4_begin_checkout.choice.003")
            .expect("current question metrics");

        assert_eq!(question.answered_count, 3);
        assert_eq!(question.unmapped_choice_count, 1);
        assert_eq!(
            question
                .choice_metrics
                .iter()
                .find(|choice| choice.choice_id == "view_item")
                .map(|choice| (choice.selected_count, choice.selected_rate)),
            Some((1, 0.5))
        );
        assert_eq!(
            question
                .choice_metrics
                .iter()
                .find(|choice| choice.choice_id == "begin_checkout")
                .map(|choice| (choice.selected_count, choice.selected_rate)),
            Some((1, 0.5))
        );
    }

    #[test]
    fn question_governance_dedupes_publishers_and_uses_earliest_source_expiry() {
        let bank = question_bank().expect("current question bank");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");
        let mut question = bank.questions[0].clone();
        let first_source = question.sources[0].clone();
        let expected_publisher = first_source.publisher.clone();
        let expected_valid_through = first_source.valid_through.clone();
        question.sources.push(first_source);

        let metrics = map_question_metrics(
            &question,
            &ProbeMetricAccumulator::default(),
            &production_approvals,
        )
        .expect("question metrics");

        assert_eq!(metrics.source_publishers, vec![expected_publisher]);
        assert_eq!(
            metrics.source_valid_through.as_deref(),
            Some(expected_valid_through.as_str())
        );
        assert!(metrics.curriculum_review_required);
        assert!(!metrics.production_approved);
        let applicability =
            question_applicability_scope(&question.id, question.version).expect("applicability");
        assert_eq!(metrics.jurisdiction_code, applicability.jurisdiction_code);
        assert_eq!(metrics.platform_code, applicability.platform_code);
        assert_eq!(
            metrics.business_stage_code,
            applicability.business_stage_code
        );
    }

    #[test]
    fn unknown_historical_question_stays_in_totals_without_current_question_attribution() {
        let now = OffsetDateTime::now_utc();
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let probe_id = Uuid::new_v4();
        let mut probe = test_structured_probe(user_id, conversation_id, probe_id, now);
        probe.context_snapshot["questionId"] = json!("legacy.unknown.choice.999");
        let mut entry = test_evidence(user_id, "correct", Some("legacy.unknown.choice.999"), now);
        entry.probe_id = Some(probe_id);
        entry.source_message_id = None;
        let bank = question_bank().expect("current question bank");
        let angle_depth = assessment_angle_depth(&bank).expect("assessment angle depth");
        let production_approvals =
            production_approved_question_identities(&bank).expect("production release");

        let metrics = summarize_probe_pilot(
            &[probe],
            &[entry],
            &[],
            &bank,
            angle_depth,
            &production_approvals,
            now,
        )
        .expect("probe pilot metrics");

        assert_eq!(metrics.issued_count, 1);
        assert_eq!(metrics.answered_count, 1);
        assert_eq!(metrics.correct_count, 1);
        assert_eq!(metrics.question_metrics.len(), 100);
        assert_eq!(
            metrics
                .question_metrics
                .iter()
                .map(|question| question.issued_count)
                .sum::<u64>(),
            0
        );
    }
}
