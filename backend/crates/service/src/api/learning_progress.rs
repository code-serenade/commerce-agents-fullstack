use crate::api::{
    learning_analysis::{merge_engagement, merge_mastery},
    learning_question_bank::{
        DiagnosticQuestion, LearningQuestionDeliveryMode, QuestionBank, assessment_angle_code,
        deliverable_question_for_domain, evaluate_choice_id, question_applicability_scope,
        question_bank, question_for_probe, question_is_deliverable_for_mode,
    },
};
use crate::dto::learning_progress::{
    LearningAssessmentSummary, LearningAssessmentTimelinePoint, LearningDomainAssessment,
    LearningDomainEvidenceSummary, LearningEvidenceEntry, LearningEvidenceRange,
    LearningProbeChoice, LearningProbeHistoryEntry, LearningProbeResponse,
    LearningProbeResultResponse, LearningProbeSourceLabel, LearningProbeStatusResponse,
    LearningProfileResponse, UpdateLearningSettingsRequest,
};
use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    conversation_messages::ConversationMessageService,
    conversations::ConversationService,
    student_learning_evidence::{CreateStudentLearningEvidence, StudentLearningEvidenceService},
    student_learning_probes::{
        CreateStudentLearningProbe, MarkStudentLearningProbeStructuredAnswer, StudentLearningProbe,
        StudentLearningProbeService, UpdateStudentLearningProbeStatus,
    },
    student_learning_profiles::{
        CreateStudentLearningProfile, StudentLearningProfile, StudentLearningProfileService,
        UpdateStudentLearningAnalysis, UpdateStudentLearningProfileSettings,
    },
    student_learning_question_gaps::{
        CreateStudentLearningQuestionGap, StudentLearningQuestionGapService,
    },
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime, Time, UtcOffset, Weekday};
use uuid::Uuid;

const PROBE_REASON_CONTEXT_ASSESSMENT_TURN: &str = "context_assessment_turn";
const PROBE_REASON_STUDENT_CONTINUED_ASSESSMENT: &str = "student_continued_assessment";
const PROBE_REASON_SPACED_RETENTION_REVIEW: &str = "spaced_retention_review";
const ASSESSMENT_TIMELINE_WEEKS: i64 = 13;
const CHAT_PROBE_HISTORY_LIMIT: u64 = 50;
const QUESTION_SELECTOR_SCHEMA_VERSION: u32 = 1;
const QUESTION_SELECTOR_VERSION: &str = "ecommerce_diagnostic_question_selector_v2";
const QUESTION_SELECTOR_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-selector.v2.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Framework {
    version: String,
    probe_policy: ProbePolicy,
    domains: Vec<FrameworkDomain>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ProbePolicy {
    interaction_mode: String,
    reward_code: String,
    delivery_cadence: ProbeDeliveryCadence,
    max_continuation_depth: u64,
    retention_review_interval_days: i64,
    minimum_answer_characters: usize,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProbeDeliveryCadence {
    EveryEligibleTurn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DomainAssessmentNeed {
    InitialOrCorrection,
    SpacedRetentionReview,
    OngoingReinforcement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PracticePurpose {
    InitialCoverage,
    CorrectionReview,
    ReinforcementReview,
}

impl PracticePurpose {
    fn code(self) -> &'static str {
        match self {
            Self::InitialCoverage => "initial_coverage",
            Self::CorrectionReview => "correction_review",
            Self::ReinforcementReview => "reinforcement_review",
        }
    }
}

#[derive(Debug, Clone)]
struct LatestQuestionAssessment {
    question_id: String,
    outcome: String,
    created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
struct FrameworkDomain {
    code: String,
    importance: f64,
}

#[derive(Debug, Deserialize)]
struct ContextClassifier {
    domains: HashMap<String, DomainClassifier>,
}

#[derive(Debug, Deserialize)]
struct DomainClassifier {
    keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QuestionSelectorConfig {
    schema_version: u32,
    version: String,
    question_bank_version: String,
    jurisdictions: HashMap<String, Vec<String>>,
    platforms: HashMap<String, Vec<String>>,
    topic_keywords: HashMap<String, Vec<String>>,
    topic_question_ids: HashMap<String, HashMap<String, Vec<String>>>,
    fallback_suppression_keywords: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Eq)]
enum SelectionSignalMatch {
    None,
    Unique(String),
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionGapSignals {
    topic_signals: Vec<String>,
    jurisdiction: String,
    platform: String,
}

struct QuestionGapRecordContext<'a> {
    user_id: Uuid,
    conversation_id: Uuid,
    assistant_message_id: Uuid,
    framework_version: &'a str,
    knowledge_code: &'a str,
    reason_code: &'a str,
    signals: Option<&'a QuestionGapSignals>,
}

pub struct LearningProgressApi {
    db: DbContext,
    question_delivery_mode: LearningQuestionDeliveryMode,
    profiles: StudentLearningProfileService,
    probes: StudentLearningProbeService,
    question_gaps: StudentLearningQuestionGapService,
    evidence: StudentLearningEvidenceService,
    conversations: ConversationService,
    messages: ConversationMessageService,
}

impl LearningProgressApi {
    pub fn new(db: DbContext) -> Self {
        Self::new_with_question_delivery_mode(db, LearningQuestionDeliveryMode::Disabled)
    }

    pub fn new_with_question_delivery_mode(
        db: DbContext,
        question_delivery_mode: LearningQuestionDeliveryMode,
    ) -> Self {
        Self {
            db: db.clone(),
            question_delivery_mode,
            profiles: StudentLearningProfileService::new(db.clone()),
            probes: StudentLearningProbeService::new(db.clone()),
            question_gaps: StudentLearningQuestionGapService::new(db.clone()),
            conversations: ConversationService::new(db.clone()),
            messages: ConversationMessageService::new(db.clone()),
            evidence: StudentLearningEvidenceService::new(db),
        }
    }

    pub async fn list_evidence(
        &self,
        current_user_id: String,
        range: LearningEvidenceRange,
        limit: u64,
    ) -> BizResult<Vec<LearningEvidenceEntry>> {
        let user_id = parse_user_id(&current_user_id)?;
        let since = match range {
            LearningEvidenceRange::Recent => None,
            LearningEvidenceRange::Days30 => Some(OffsetDateTime::now_utc() - Duration::days(30)),
            LearningEvidenceRange::Days90 => Some(OffsetDateTime::now_utc() - Duration::days(90)),
        };
        let entries = self
            .evidence
            .list_by_user_since(user_id, since, limit)
            .await?;
        let probe_ids = entries
            .iter()
            .filter_map(|entry| entry.probe_id)
            .collect::<Vec<_>>();
        let probes = self
            .probes
            .list_by_ids(probe_ids)
            .await?
            .into_iter()
            .filter(|probe| probe.user_id == user_id)
            .map(|probe| (probe.id, probe))
            .collect::<HashMap<_, _>>();
        Ok(entries
            .into_iter()
            .map(|entry| {
                let probe = entry.probe_id.and_then(|id| probes.get(&id));
                map_evidence_entry(entry, probe)
            })
            .collect())
    }

    pub async fn list_probe_history(
        &self,
        current_user_id: String,
        conversation_id: String,
        assistant_message_id: String,
    ) -> BizResult<Vec<LearningProbeHistoryEntry>> {
        let user_id = parse_user_id(&current_user_id)?;
        let (_, assistant_message) = self
            .validate_probe_context(user_id, Some(conversation_id), Some(assistant_message_id))
            .await?;
        let assistant_message =
            assistant_message.ok_or_else(|| invalid("learning assistant message not found"))?;
        let mut probes = self
            .probes
            .list_recent_answered_by_user_and_assistant_message(
                user_id,
                assistant_message.id,
                CHAT_PROBE_HISTORY_LIMIT,
            )
            .await?
            .into_iter()
            .filter(|probe| {
                probe
                    .context_snapshot
                    .get("answerMode")
                    .and_then(Value::as_str)
                    == Some("structured_choice")
            })
            .collect::<Vec<_>>();
        probes.reverse();

        let mut entries = Vec::with_capacity(probes.len());
        for probe in probes {
            let mut result = self
                .get_probe_result(current_user_id.clone(), probe.id.to_string())
                .await?;
            result.can_continue = false;
            entries.push(LearningProbeHistoryEntry {
                probe: map_probe(probe),
                result,
            });
        }
        Ok(entries)
    }

    pub async fn get_profile(&self, current_user_id: String) -> BizResult<LearningProfileResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let profile = self.get_or_create_profile(user_id).await?;
        let evidence = self.evidence.list_by_user(user_id).await?;
        let framework = framework()?;
        Ok(map_profile(profile, &evidence, &framework))
    }

    pub async fn update_settings(
        &self,
        current_user_id: String,
        req: UpdateLearningSettingsRequest,
    ) -> BizResult<LearningProfileResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let profile = self.get_or_create_profile(user_id).await?;
        let profile = self
            .profiles
            .update_settings(UpdateStudentLearningProfileSettings {
                id: profile.id,
                enabled: req.enabled,
            })
            .await?;
        let evidence = self.evidence.list_by_user(user_id).await?;
        let framework = framework()?;
        Ok(map_profile(profile, &evidence, &framework))
    }

    pub async fn get_next_probe(
        &self,
        current_user_id: String,
        conversation_id: Option<String>,
        assistant_message_id: Option<String>,
        continue_from_probe_id: Option<String>,
    ) -> BizResult<Option<LearningProbeResponse>> {
        let user_id = parse_user_id(&current_user_id)?;
        let profile = self.get_or_create_profile(user_id).await?;
        if !profile.enabled {
            return Ok(None);
        }

        let (conversation_id, assistant_message) = self
            .validate_probe_context(user_id, conversation_id, assistant_message_id)
            .await?;
        let (Some(conversation_id), Some(assistant_message)) = (conversation_id, assistant_message)
        else {
            return Ok(None);
        };

        let framework = framework()?;
        if framework.probe_policy.delivery_cadence != ProbeDeliveryCadence::EveryEligibleTurn {
            return Err(invalid("learning probe delivery cadence unsupported"));
        }
        let now = OffsetDateTime::now_utc();
        let mut turn_probe = self
            .probes
            .get_latest_by_user_and_assistant_message(user_id, assistant_message.id)
            .await?;
        if continue_from_probe_id.is_none()
            && let Some(active) = turn_probe.as_ref()
            && matches!(active.status.as_str(), "planned" | "offered")
        {
            let expired = active
                .expires_at
                .is_some_and(|expires_at| expires_at <= now);
            let engaged = active.context_snapshot.get("engagedAt").is_some();
            if !expired {
                let deliverable = question_for_probe(&active.context_snapshot)?
                    .map(|(bank, question)| {
                        question_is_deliverable_for_mode(
                            &bank,
                            &question,
                            self.question_delivery_mode,
                        )
                    })
                    .transpose()?
                    .unwrap_or(false);
                let context_matches = if deliverable
                    && active
                        .context_snapshot
                        .get("questionType")
                        .and_then(Value::as_str)
                        == Some("single_choice")
                {
                    let question_bank = question_bank()?;
                    let question_selector = question_selector(&question_bank)?;
                    let classifier = context_classifier()?;
                    let conversation_messages = self
                        .messages
                        .list_by_conversation_id(conversation_id)
                        .await?;
                    active_probe_matches_turn_context(
                        active,
                        direct_preceding_user_content(&conversation_messages, &assistant_message),
                        assistant_message
                            .content_text
                            .as_deref()
                            .unwrap_or_default(),
                        &question_bank,
                        &question_selector,
                        &framework,
                        &classifier,
                    )
                } else {
                    true
                };
                if deliverable && (context_matches || engaged) {
                    return Ok(Some(map_probe(active.clone())));
                }
                if engaged {
                    return Ok(None);
                }
                self.probes
                    .update_status(UpdateStudentLearningProbeStatus {
                        id: active.id,
                        status: "superseded".to_owned(),
                    })
                    .await?;
                turn_probe = None;
            } else {
                self.probes
                    .update_status(UpdateStudentLearningProbeStatus {
                        id: active.id,
                        status: "expired".to_owned(),
                    })
                    .await?;
                return Ok(None);
            }
        }

        let latest_probe = self.probes.get_latest_by_user(user_id).await?;
        let evidence = self.evidence.list_by_user(user_id).await?;
        let latest_question_assessments = latest_question_assessments_newest_first(
            evidence
                .iter()
                .filter_map(evidence_timed_assessment_outcome),
        );
        let continuation_probe = match continue_from_probe_id {
            Some(probe_id) => {
                let probe_id =
                    Uuid::parse_str(&probe_id).map_err(|_| invalid("invalid probe id"))?;
                let probe = self
                    .probes
                    .get_by_id(probe_id)
                    .await?
                    .filter(|probe| {
                        probe.user_id == user_id
                            && probe.conversation_id == Some(conversation_id)
                            && probe.status == "answered"
                    })
                    .ok_or_else(|| invalid("learning continuation probe invalid"))?;
                if let Some(answer_message_id) = probe.answer_message_id {
                    let answer_message = self
                        .messages
                        .get_by_id(answer_message_id)
                        .await?
                        .filter(|message| {
                            message.user_id == user_id
                                && message.conversation_id == conversation_id
                                && message.role == "user"
                        })
                        .ok_or_else(|| invalid("learning continuation answer invalid"))?;
                    if !is_direct_probe_answer_reply(&probe, &answer_message, &assistant_message) {
                        return Err(invalid("learning continuation context invalid"));
                    }
                } else if probe
                    .context_snapshot
                    .get("answerMode")
                    .and_then(Value::as_str)
                    != Some("structured_choice")
                    || probe.assistant_message_id != Some(assistant_message.id)
                {
                    return Err(invalid("learning continuation context invalid"));
                }
                Some(probe)
            }
            None => None,
        };

        if continuation_probe.is_none()
            && let Some(probe) = turn_probe.as_ref()
        {
            return Ok(existing_turn_probe_response(probe, &assistant_message));
        }

        if continuation_probe.is_none()
            && let Some(latest) = latest_probe.as_ref()
            && latest.status == "answered"
            && let Some(answer_message_id) = latest.answer_message_id
            && let Some(answer_message) = self.messages.get_by_id(answer_message_id).await?
            && is_direct_probe_answer_reply(latest, &answer_message, &assistant_message)
        {
            return Ok(None);
        }

        let question_bank = question_bank()?;
        if question_bank.framework_version != framework.version {
            return Err(invalid(
                "learning diagnostic question bank framework mismatch",
            ));
        }
        let question_selector = question_selector(&question_bank)?;
        let mut question_gap_signals = None;
        let (
            domain,
            context_match_count,
            continuation_depth,
            reason_code,
            previous_question_id,
            contextual_question_ids,
        ) = if let Some(probe) = continuation_probe.as_ref() {
            let Some(continuation_depth) =
                next_probe_continuation_depth(probe, framework.probe_policy.max_continuation_depth)
            else {
                return Ok(None);
            };
            let Some(domain) = framework
                .domains
                .iter()
                .find(|domain| domain.code == probe.knowledge_code)
                .cloned()
            else {
                return Ok(None);
            };
            if domain_assessment_need(
                latest_question_assessments.get(&domain.code),
                now,
                framework.probe_policy.retention_review_interval_days,
            )
            .is_none()
            {
                return Ok(None);
            }
            (
                domain,
                probe
                    .context_snapshot
                    .get("contextMatchCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(1) as usize,
                continuation_depth,
                PROBE_REASON_STUDENT_CONTINUED_ASSESSMENT,
                probe
                    .context_snapshot
                    .get("questionId")
                    .and_then(Value::as_str),
                probe_contextual_question_ids(probe),
            )
        } else {
            let classifier = context_classifier()?;
            let conversation_messages = self
                .messages
                .list_by_conversation_id(conversation_id)
                .await?;
            let user_content =
                direct_preceding_user_content(&conversation_messages, &assistant_message);
            let assistant_content = assistant_message
                .content_text
                .as_deref()
                .unwrap_or_default();
            let Some((domain, context_match_count, assessment_need)) = select_turn_context_domain(
                user_content,
                assistant_content,
                &latest_question_assessments,
                &framework.domains,
                &classifier,
                now,
                framework.probe_policy.retention_review_interval_days,
            ) else {
                return Ok(None);
            };
            let reason_code = match assessment_need {
                DomainAssessmentNeed::InitialOrCorrection => PROBE_REASON_CONTEXT_ASSESSMENT_TURN,
                DomainAssessmentNeed::SpacedRetentionReview => PROBE_REASON_SPACED_RETENTION_REVIEW,
                DomainAssessmentNeed::OngoingReinforcement => PROBE_REASON_CONTEXT_ASSESSMENT_TURN,
            };
            let contextual_question_ids = contextual_question_ids_for_turn(
                user_content,
                assistant_content,
                &domain.code,
                &question_bank,
                &question_selector,
            );
            question_gap_signals = Some(question_gap_signals_for_turn(
                user_content,
                assistant_content,
                &domain.code,
                &question_selector,
            ));
            (
                domain,
                context_match_count,
                0,
                reason_code,
                latest_probe.as_ref().and_then(|probe| {
                    probe
                        .context_snapshot
                        .get("questionId")
                        .and_then(Value::as_str)
                }),
                contextual_question_ids,
            )
        };
        let selection_seed = format!("{}:{continuation_depth}", assistant_message.id);
        let assessment_question_ids = preferred_question_ids_for_domain(
            &question_bank,
            &domain.code,
            latest_question_assessments.get(&domain.code),
        );
        let Some(preferred_question_ids) = question_pool_for_request(
            continuation_probe.is_some(),
            contextual_question_ids.clone(),
            assessment_question_ids,
        ) else {
            if continuation_probe.is_none() {
                self.record_question_gap(QuestionGapRecordContext {
                    user_id,
                    conversation_id,
                    assistant_message_id: assistant_message.id,
                    framework_version: &framework.version,
                    knowledge_code: &domain.code,
                    reason_code: "no_contextual_question",
                    signals: question_gap_signals.as_ref(),
                })
                .await;
            }
            return Ok(None);
        };
        let continuation_question_ids = continuation_probe
            .as_ref()
            .map(probe_continuation_question_ids)
            .unwrap_or_default();
        let (mut preferred_question_ids, mut continuation_question_ids, restarted_cycle) =
            if continuation_probe.is_some() {
                continuation_candidates_for_next_cycle(
                    preferred_question_ids,
                    &continuation_question_ids,
                )
            } else {
                (preferred_question_ids, continuation_question_ids, false)
            };
        if preferred_question_ids.is_empty() {
            return Ok(None);
        }
        let practice_cycle = if let Some(probe) = continuation_probe.as_ref() {
            probe_practice_cycle(probe)
                .checked_add(u64::from(restarted_cycle))
                .ok_or_else(|| invalid("learning practice cycle overflow"))?
        } else {
            0
        };
        if practice_cycle > 0 {
            preferred_question_ids = prioritize_review_candidates(
                preferred_question_ids,
                latest_question_assessments.get(&domain.code),
                &question_bank,
            );
        }
        let excluded_question_id = if continuation_probe.is_none() || restarted_cycle {
            previous_question_id
        } else {
            None
        };
        let Some(question) = deliverable_question_for_domain(
            &question_bank,
            self.question_delivery_mode,
            &domain.code,
            &selection_seed,
            excluded_question_id,
            &preferred_question_ids,
        )?
        else {
            if continuation_probe.is_none() {
                self.record_question_gap(QuestionGapRecordContext {
                    user_id,
                    conversation_id,
                    assistant_message_id: assistant_message.id,
                    framework_version: &framework.version,
                    knowledge_code: &domain.code,
                    reason_code: "no_deliverable_question",
                    signals: question_gap_signals.as_ref(),
                })
                .await;
            }
            return Ok(None);
        };
        if continuation_probe.is_some() && continuation_question_ids.contains(&question.id) {
            return Ok(None);
        }

        continuation_question_ids.push(question.id.clone());

        let angle_code = assessment_angle_code(&question.id, question.version)
            .ok_or_else(|| invalid("learning diagnostic assessment angle missing"))?;
        let practice_purpose = practice_purpose_for_question(
            practice_cycle,
            angle_code,
            latest_question_assessments.get(&domain.code),
        );
        let create = CreateStudentLearningProbe {
            id: Uuid::new_v4(),
            user_id,
            conversation_id: Some(conversation_id),
            assistant_message_id: Some(assistant_message.id),
            framework_version: framework.version,
            knowledge_code: domain.code,
            question_text: question.prompt,
            reason_code: reason_code.to_owned(),
            priority_score: domain.importance,
            context_snapshot: json!({
                "interactionMode": framework.probe_policy.interaction_mode,
                "rewardCode": framework.probe_policy.reward_code,
                "questionBankVersion": question_bank.version,
                "sourceRevision": question_bank.source_revision,
                "questionId": question.id,
                "questionVersion": question.version,
                "assessmentAngleCode": angle_code,
                "questionType": question.question_type,
                "choices": question.choices,
                "rubricVersion": question.rubric.version,
                "contextMatchCount": context_match_count,
                "continuationDepth": continuation_depth,
                "continuationQuestionIds": continuation_question_ids,
                "contextualQuestionIds": contextual_question_ids,
                "practiceCycle": practice_cycle,
                "practicePurposeCode": practice_purpose.code(),
                "sourceSequence": assistant_message.sequence
            }),
            offered_at: now,
            expires_at: now + Duration::days(1),
        };
        let probe = self.probes.create(create).await?;
        Ok(Some(map_probe(probe)))
    }

    async fn record_question_gap(&self, context: QuestionGapRecordContext<'_>) {
        let (topic_signals, scope_signals) = context.signals.map_or_else(
            || (json!([]), json!({})),
            |signals| {
                (
                    json!(signals.topic_signals),
                    json!({
                        "jurisdiction": signals.jurisdiction,
                        "platform": signals.platform,
                    }),
                )
            },
        );
        if let Err(error) = self
            .question_gaps
            .create_idempotent(CreateStudentLearningQuestionGap {
                id: Uuid::new_v4(),
                user_id: context.user_id,
                conversation_id: context.conversation_id,
                assistant_message_id: context.assistant_message_id,
                framework_version: context.framework_version.to_owned(),
                selector_version: QUESTION_SELECTOR_VERSION.to_owned(),
                knowledge_code: context.knowledge_code.to_owned(),
                reason_code: context.reason_code.to_owned(),
                topic_signals,
                scope_signals,
            })
            .await
        {
            tracing::warn!(
                assistant_message_id = %context.assistant_message_id,
                knowledge_code = context.knowledge_code,
                reason_code = context.reason_code,
                error = ?error,
                "failed to record learning question coverage gap"
            );
        }
    }

    pub async fn submit_choice(
        &self,
        current_user_id: String,
        probe_id: String,
        choice_id: String,
    ) -> BizResult<LearningProbeResultResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let id = Uuid::parse_str(&probe_id).map_err(|_| invalid("invalid probe id"))?;
        let probe = self.probes.get_by_id(id).await?.ok_or_else(|| {
            BizError::new(
                app_error::LEARNING_PROBE_NOT_FOUND,
                "learning probe not found".to_owned(),
            )
        })?;
        if probe.user_id != user_id {
            return Err(BizError::new(
                app_error::LEARNING_PROBE_FORBIDDEN,
                "learning probe forbidden".to_owned(),
            ));
        }
        if probe.status == "answered" {
            if self.evidence.get_by_probe_id(probe.id).await?.is_some() {
                return self.get_probe_result(current_user_id, probe_id).await;
            }
            return Err(invalid("learning probe result missing"));
        }
        let now = OffsetDateTime::now_utc();
        if probe.expires_at.is_some_and(|expires_at| expires_at <= now) {
            return Err(invalid("learning probe is expired"));
        }
        if probe.status != "offered" {
            return Err(invalid("learning probe is not offered"));
        }
        let conversation_id = probe
            .conversation_id
            .ok_or_else(|| invalid("learning probe conversation missing"))?;
        let (bank, question) = question_for_probe(&probe.context_snapshot)?
            .ok_or_else(|| invalid("learning probe question missing"))?;
        let assessment = evaluate_choice_id(&question, choice_id.trim())?;
        let selected_choice = question
            .choices
            .iter()
            .find(|choice| choice.id == choice_id.trim())
            .ok_or_else(|| invalid("learning probe choice invalid"))?;
        let profile = self.get_or_create_profile(user_id).await?;
        let framework = framework()?;
        let mut context = probe
            .context_snapshot
            .as_object()
            .cloned()
            .unwrap_or_else(Map::new);
        context.insert("answerMode".to_owned(), json!("structured_choice"));
        context.insert("selectedChoiceId".to_owned(), json!(selected_choice.id));
        context.insert("submittedAt".to_owned(), json!(rfc3339(now)));
        let mut rubric_hits = assessment.as_json(&bank, &question);
        if let Some(details) = rubric_hits.as_object_mut() {
            details.insert("probeId".to_owned(), json!(probe.id));
            details.insert("questionText".to_owned(), json!(probe.question_text));
            details.insert("selectedChoiceId".to_owned(), json!(selected_choice.id));
            details.insert("selectedChoiceText".to_owned(), json!(selected_choice.text));
        }
        let mastery_map = if assessment.contributes_to_mastery {
            merge_mastery(
                &profile.mastery_map,
                &probe.knowledge_code,
                assessment.score,
                Some(assessment.outcome),
                now,
            )
        } else {
            profile.mastery_map.clone()
        };
        let coverage_rate = mastery_map.as_object().map_or(0, |map| map.len()) as f64
            / framework.domains.len() as f64;
        let engagement_summary =
            merge_engagement(&profile.engagement_summary, &probe.knowledge_code, now);

        let tx = self.db.begin().await?;
        let result: BizResult<()> = async {
            let probes = StudentLearningProbeService::new(tx.clone());
            let evidence = StudentLearningEvidenceService::new(tx.clone());
            let profiles = StudentLearningProfileService::new(tx.clone());
            let claimed = probes
                .try_mark_structured_answer(MarkStudentLearningProbeStructuredAnswer {
                    id: probe.id,
                    context_snapshot: Value::Object(context),
                    answered_at: now,
                })
                .await?;
            if !claimed {
                return Err(invalid("learning probe already answered"));
            }
            let inserted = evidence
                .create_if_absent(CreateStudentLearningEvidence {
                    id: Uuid::new_v4(),
                    user_id,
                    conversation_id,
                    source_message_id: None,
                    probe_id: Some(probe.id),
                    framework_version: profile.framework_version.clone(),
                    knowledge_code: probe.knowledge_code.clone(),
                    evidence_type: "probe_answer".to_owned(),
                    direction: assessment.direction.to_owned(),
                    strength: assessment.score,
                    confidence: 1.0,
                    summary: format!("{}：{}", probe.question_text, selected_choice.text),
                    rubric_hits,
                    analyzer_version: "deterministic_question_bank_v1".to_owned(),
                    created_at: now,
                })
                .await?;
            if !inserted {
                return Err(invalid("learning probe already answered"));
            }
            profiles
                .update_analysis(UpdateStudentLearningAnalysis {
                    id: profile.id,
                    mastery_map,
                    engagement_summary,
                    coverage_rate,
                    analysis_status: "done".to_owned(),
                    last_analyzed_sequence: profile.last_analyzed_sequence,
                    last_analyzed_at: now,
                    last_error_code: None,
                })
                .await?;
            Ok(())
        }
        .await;
        match commit_or_rollback(tx, result).await {
            Ok(()) => self.get_probe_result(current_user_id, probe_id).await,
            Err(error) => {
                if self.evidence.get_by_probe_id(probe.id).await?.is_some() {
                    self.get_probe_result(current_user_id, probe_id).await
                } else {
                    Err(error)
                }
            }
        }
    }

    pub async fn engage_probe(
        &self,
        current_user_id: String,
        probe_id: String,
    ) -> BizResult<LearningProbeStatusResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let id = Uuid::parse_str(&probe_id).map_err(|_| invalid("invalid probe id"))?;
        let probe = self.probes.get_by_id(id).await?.ok_or_else(|| {
            BizError::new(
                app_error::LEARNING_PROBE_NOT_FOUND,
                "learning probe not found".to_owned(),
            )
        })?;
        if probe.user_id != user_id {
            return Err(BizError::new(
                app_error::LEARNING_PROBE_FORBIDDEN,
                "learning probe forbidden".to_owned(),
            ));
        }
        if probe.context_snapshot.get("engagedAt").is_some() {
            return Ok(LearningProbeStatusResponse {
                id: probe.id.to_string(),
                status: probe.status,
            });
        }
        let now = OffsetDateTime::now_utc();
        if probe.expires_at.is_some_and(|expires_at| expires_at <= now) {
            self.probes
                .update_status(UpdateStudentLearningProbeStatus {
                    id: probe.id,
                    status: "expired".to_owned(),
                })
                .await?;
            return Err(invalid("learning probe is expired"));
        }
        if probe.status != "offered" {
            return Err(invalid("learning probe is not offered"));
        }

        let mut context = probe
            .context_snapshot
            .as_object()
            .cloned()
            .unwrap_or_else(Map::new);
        context.insert("engagedAt".to_owned(), json!(rfc3339(now)));
        let updated = self
            .probes
            .update_context_snapshot(
                repo::table::student_learning_probes::UpdateStudentLearningProbeContext {
                    id: probe.id,
                    context_snapshot: Value::Object(context),
                },
            )
            .await?;
        Ok(LearningProbeStatusResponse {
            id: updated.id.to_string(),
            status: updated.status,
        })
    }

    pub async fn get_probe_result(
        &self,
        current_user_id: String,
        probe_id: String,
    ) -> BizResult<LearningProbeResultResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let id = Uuid::parse_str(&probe_id).map_err(|_| invalid("invalid probe id"))?;
        let probe = self.probes.get_by_id(id).await?.ok_or_else(|| {
            BizError::new(
                app_error::LEARNING_PROBE_NOT_FOUND,
                "learning probe not found".to_owned(),
            )
        })?;
        if probe.user_id != user_id {
            return Err(BizError::new(
                app_error::LEARNING_PROBE_FORBIDDEN,
                "learning probe forbidden".to_owned(),
            ));
        }

        let evidence = self.evidence.get_by_probe_id(probe.id).await?;
        let outcome = evidence
            .as_ref()
            .and_then(|entry| entry.rubric_hits.get("outcome"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let score = evidence
            .as_ref()
            .and_then(|entry| entry.rubric_hits.get("score"))
            .and_then(Value::as_f64);
        let demonstrated_points = evidence
            .as_ref()
            .and_then(|entry| entry.rubric_hits.get("matchedCriteria"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let next_point = evidence
            .as_ref()
            .and_then(|entry| entry.rubric_hits.get("missingCriteria"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        let question = if probe.status == "answered" {
            crate::api::learning_question_bank::question_for_probe(&probe.context_snapshot)?
        } else {
            None
        };
        let feedback_code = outcome
            .as_deref()
            .map(|value| format!("learning_probe_{value}"));
        let source_labels = question
            .as_ref()
            .map(|(_, item)| {
                item.sources
                    .iter()
                    .map(|source| LearningProbeSourceLabel {
                        source_id: source.source_id.clone(),
                        title: source.title.clone(),
                        publisher: source.publisher.clone(),
                        url: source.url.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let (selected_choice_id, selected_choice_text) = selected_choice_for_result(
            &probe,
            evidence.as_ref().map(|entry| &entry.rubric_hits),
            question.as_ref().map(|(_, item)| item),
        );
        let correct_choice_text = answered_correct_choice_text(
            &probe,
            evidence.as_ref(),
            question.as_ref().map(|(_, item)| item),
        );
        let answer_comparison_code = answer_comparison_code(
            evidence.as_ref(),
            selected_choice_id.as_deref(),
            question.as_ref().map(|(_, item)| item),
        );
        let knowledge_points = if evidence.is_some() {
            question
                .as_ref()
                .map(|(_, item)| question_knowledge_points(item))
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let reference_explanation = question.as_ref().map(|(_, item)| {
            instructional_reference_explanation(
                item,
                outcome.as_deref(),
                selected_choice_text.as_deref(),
                correct_choice_text.as_deref(),
                &knowledge_points,
            )
        });
        let can_continue = if probe.status == "answered"
            && outcome
                .as_deref()
                .is_some_and(|value| matches!(value, "correct" | "partial" | "incorrect"))
            && probe_can_continue(&probe, framework()?.probe_policy.max_continuation_depth)
        {
            let all_evidence = self.evidence.list_by_user(user_id).await?;
            let latest_question_assessments = latest_question_assessments_newest_first(
                all_evidence
                    .iter()
                    .filter_map(evidence_timed_assessment_outcome),
            );
            if domain_assessment_need(
                latest_question_assessments.get(&probe.knowledge_code),
                OffsetDateTime::now_utc(),
                framework()?.probe_policy.retention_review_interval_days,
            )
            .is_some()
            {
                let bank = question_bank()?;
                let assessment_question_ids = preferred_question_ids_for_domain(
                    &bank,
                    &probe.knowledge_code,
                    latest_question_assessments.get(&probe.knowledge_code),
                );
                if let Some(preferred_question_ids) = question_pool_for_request(
                    true,
                    probe_contextual_question_ids(&probe),
                    assessment_question_ids,
                ) {
                    let answered_question_ids = probe_continuation_question_ids(&probe);
                    let (candidate_question_ids, _, restarted_cycle) =
                        continuation_candidates_for_next_cycle(
                            preferred_question_ids,
                            &answered_question_ids,
                        );
                    !candidate_question_ids.is_empty()
                        && deliverable_question_for_domain(
                            &bank,
                            self.question_delivery_mode,
                            &probe.knowledge_code,
                            &probe.id.to_string(),
                            restarted_cycle
                                .then(|| {
                                    probe
                                        .context_snapshot
                                        .get("questionId")
                                        .and_then(Value::as_str)
                                })
                                .flatten(),
                            &candidate_question_ids,
                        )?
                        .is_some()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        Ok(LearningProbeResultResponse {
            id: probe.id.to_string(),
            status: probe.status,
            can_continue,
            outcome,
            score,
            feedback_code,
            selected_choice_id,
            selected_choice_text,
            correct_choice_text,
            answer_comparison_code,
            knowledge_points,
            demonstrated_points,
            next_point,
            reference_explanation,
            source_labels,
        })
    }

    pub async fn skip_probe(
        &self,
        current_user_id: String,
        probe_id: String,
    ) -> BizResult<LearningProbeStatusResponse> {
        let user_id = parse_user_id(&current_user_id)?;
        let id = Uuid::parse_str(&probe_id).map_err(|_| invalid("invalid probe id"))?;
        let probe = self.probes.get_by_id(id).await?.ok_or_else(|| {
            BizError::new(
                app_error::LEARNING_PROBE_NOT_FOUND,
                "learning probe not found".to_owned(),
            )
        })?;
        if probe.user_id != user_id {
            return Err(BizError::new(
                app_error::LEARNING_PROBE_FORBIDDEN,
                "learning probe forbidden".to_owned(),
            ));
        }
        if probe.status == "skipped" {
            return Ok(LearningProbeStatusResponse {
                id: probe.id.to_string(),
                status: probe.status,
            });
        }
        if probe.status != "planned" && probe.status != "offered" {
            return Err(invalid("learning probe is not active"));
        }
        let updated = self
            .probes
            .update_status(UpdateStudentLearningProbeStatus {
                id,
                status: "skipped".to_owned(),
            })
            .await?;
        Ok(LearningProbeStatusResponse {
            id: updated.id.to_string(),
            status: updated.status,
        })
    }

    async fn get_or_create_profile(&self, user_id: Uuid) -> BizResult<StudentLearningProfile> {
        match self.profiles.get_by_user_id(user_id).await? {
            Some(profile) => Ok(profile),
            None => {
                self.profiles
                    .create(CreateStudentLearningProfile {
                        id: Uuid::new_v4(),
                        user_id,
                        framework_version: framework()?.version,
                    })
                    .await
            }
        }
    }

    async fn validate_probe_context(
        &self,
        user_id: Uuid,
        conversation_id: Option<String>,
        assistant_message_id: Option<String>,
    ) -> BizResult<(
        Option<Uuid>,
        Option<repo::table::conversation_messages::ConversationMessage>,
    )> {
        let conversation_id = parse_optional_uuid(conversation_id)?;
        let assistant_message_id = parse_optional_uuid(assistant_message_id)?;

        if let Some(conversation_id) = conversation_id {
            let conversation = self
                .conversations
                .get_by_id(conversation_id)
                .await?
                .filter(|conversation| {
                    conversation.user_id == user_id && conversation.status != "deleted"
                })
                .ok_or_else(|| invalid("learning conversation not found"))?;

            if let Some(message_id) = assistant_message_id {
                let message = self
                    .messages
                    .get_by_id(message_id)
                    .await?
                    .filter(|message| {
                        message.user_id == user_id
                            && message.conversation_id == conversation.id
                            && message.role == "assistant"
                    })
                    .ok_or_else(|| invalid("learning assistant message not found"))?;
                return Ok((Some(conversation.id), Some(message)));
            }

            return Ok((Some(conversation.id), None));
        }

        if let Some(message_id) = assistant_message_id {
            let message = self
                .messages
                .get_by_id(message_id)
                .await?
                .filter(|message| message.user_id == user_id && message.role == "assistant")
                .ok_or_else(|| invalid("learning assistant message not found"))?;
            return Ok((Some(message.conversation_id), Some(message)));
        }

        Ok((None, None))
    }
}

fn framework() -> BizResult<Framework> {
    serde_json::from_str(include_str!(
        "../../../../config/json/learning/ecommerce-knowledge-framework.v1.json"
    ))
    .map_err(|_| invalid("learning framework invalid"))
}

fn context_classifier() -> BizResult<ContextClassifier> {
    serde_json::from_str(include_str!(
        "../../../../config/json/learning/evidence-classifier.v1.json"
    ))
    .map_err(|_| invalid("learning evidence classifier invalid"))
}

fn question_selector(bank: &QuestionBank) -> BizResult<QuestionSelectorConfig> {
    let selector: QuestionSelectorConfig = serde_json::from_str(QUESTION_SELECTOR_JSON)
        .map_err(|_| invalid("learning diagnostic question selector invalid"))?;
    validate_question_selector(bank, &selector)?;
    Ok(selector)
}

fn validate_question_selector(
    bank: &QuestionBank,
    selector: &QuestionSelectorConfig,
) -> BizResult<()> {
    let domain_codes = bank
        .questions
        .iter()
        .map(|question| question.domain_code.as_str())
        .collect::<HashSet<_>>();
    if selector.schema_version != QUESTION_SELECTOR_SCHEMA_VERSION
        || selector.version != QUESTION_SELECTOR_VERSION
        || selector.question_bank_version != bank.version
        || !signal_groups_are_valid(&selector.jurisdictions, &["cn", "eu", "us"])
        || !signal_groups_are_valid(
            &selector.platforms,
            &[
                "google_trends",
                "google_merchant_center",
                "google_analytics_4",
                "google_search",
                "google_ads",
            ],
        )
        || selector.topic_keywords.len() != domain_codes.len()
        || !domain_codes
            .iter()
            .all(|domain| selector.topic_keywords.contains_key(*domain))
        || selector
            .fallback_suppression_keywords
            .iter()
            .any(|(domain_code, keywords)| {
                !domain_codes.contains(domain_code.as_str()) || !signals_are_valid(keywords)
            })
    {
        return Err(invalid(
            "learning diagnostic question selector metadata invalid",
        ));
    }

    if selector
        .topic_question_ids
        .iter()
        .any(|(domain_code, mappings)| {
            let Some(domain_keywords) = selector.topic_keywords.get(domain_code) else {
                return true;
            };
            let normalized_domain_keywords = domain_keywords
                .iter()
                .map(|keyword| keyword.trim().to_lowercase())
                .collect::<HashSet<_>>();
            mappings.is_empty()
                || mappings.iter().any(|(keyword, question_ids)| {
                    let normalized_keyword = keyword.trim().to_lowercase();
                    let mut unique = HashSet::new();
                    normalized_keyword.is_empty()
                        || !normalized_domain_keywords.contains(&normalized_keyword)
                        || question_ids.is_empty()
                        || question_ids.iter().any(|question_id| {
                            !unique.insert(question_id)
                                || !bank.questions.iter().any(|question| {
                                    question.id == *question_id
                                        && question.domain_code == *domain_code
                                })
                        })
                })
        })
    {
        return Err(invalid(
            "learning diagnostic question selector topic mappings invalid",
        ));
    }

    for (domain_code, keywords) in &selector.topic_keywords {
        if !signals_are_valid(keywords)
            || keywords.iter().any(|keyword| {
                let lowered = keyword.to_lowercase();
                let appears_in_question = bank.questions.iter().any(|question| {
                    question.domain_code == *domain_code
                        && question_search_text(question).contains(&lowered)
                });
                let has_valid_explicit_mapping = selector
                    .topic_question_ids
                    .get(domain_code)
                    .is_some_and(|mappings| {
                        mappings
                            .keys()
                            .any(|mapped_keyword| mapped_keyword.trim().to_lowercase() == lowered)
                    });
                !appears_in_question && !has_valid_explicit_mapping
            })
        {
            return Err(invalid(
                "learning diagnostic question selector topics invalid",
            ));
        }
    }
    Ok(())
}

fn signal_groups_are_valid(groups: &HashMap<String, Vec<String>>, expected_codes: &[&str]) -> bool {
    groups.len() == expected_codes.len()
        && expected_codes.iter().all(|code| groups.contains_key(*code))
        && groups.values().all(|signals| signals_are_valid(signals))
}

fn signals_are_valid(signals: &[String]) -> bool {
    let mut unique = HashSet::new();
    !signals.is_empty()
        && signals.iter().all(|signal| {
            let normalized = signal.trim().to_lowercase();
            !normalized.is_empty() && unique.insert(normalized)
        })
}

fn direct_preceding_user_content<'a>(
    messages: &'a [repo::table::conversation_messages::ConversationMessage],
    assistant_message: &repo::table::conversation_messages::ConversationMessage,
) -> Option<&'a str> {
    let user_sequence = assistant_message.sequence.checked_sub(1)?;
    messages
        .iter()
        .find(|message| {
            message.conversation_id == assistant_message.conversation_id
                && message.user_id == assistant_message.user_id
                && message.role == "user"
                && message.status == "completed"
                && message.sequence == user_sequence
        })
        .and_then(|message| message.content_text.as_deref())
        .map(str::trim)
        .filter(|content| !content.is_empty())
}

fn select_turn_context_domain(
    user_content: Option<&str>,
    assistant_content: &str,
    latest_question_assessments: &HashMap<String, HashMap<String, LatestQuestionAssessment>>,
    domains: &[FrameworkDomain],
    classifier: &ContextClassifier,
    now: OffsetDateTime,
    retention_review_interval_days: i64,
) -> Option<(FrameworkDomain, usize, DomainAssessmentNeed)> {
    user_content
        .and_then(|content| {
            select_context_domain(
                content,
                latest_question_assessments,
                domains,
                classifier,
                now,
                retention_review_interval_days,
            )
        })
        .or_else(|| {
            select_context_domain(
                assistant_content,
                latest_question_assessments,
                domains,
                classifier,
                now,
                retention_review_interval_days,
            )
        })
}

fn contextual_question_ids_for_turn(
    user_content: Option<&str>,
    assistant_content: &str,
    domain_code: &str,
    bank: &QuestionBank,
    selector: &QuestionSelectorConfig,
) -> Option<Vec<String>> {
    user_content
        .and_then(|content| {
            contextual_question_ids_for_content(content, domain_code, bank, selector)
        })
        .or_else(|| {
            contextual_question_ids_for_content(assistant_content, domain_code, bank, selector)
        })
        .or_else(|| Some(Vec::new()))
}

fn question_gap_signals_for_turn(
    user_content: Option<&str>,
    assistant_content: &str,
    domain_code: &str,
    selector: &QuestionSelectorConfig,
) -> QuestionGapSignals {
    user_content
        .and_then(|content| question_gap_signals_for_content(content, domain_code, selector))
        .or_else(|| question_gap_signals_for_content(assistant_content, domain_code, selector))
        .unwrap_or_else(|| QuestionGapSignals {
            topic_signals: Vec::new(),
            jurisdiction: "none".to_owned(),
            platform: "none".to_owned(),
        })
}

fn question_gap_signals_for_content(
    content: &str,
    domain_code: &str,
    selector: &QuestionSelectorConfig,
) -> Option<QuestionGapSignals> {
    let lowered = content.to_lowercase();
    let jurisdiction = selection_signal_match(&lowered, &selector.jurisdictions);
    let platform = selection_signal_match(&lowered, &selector.platforms);
    let topic_signals = matched_topic_keywords(&lowered, domain_code, selector);
    if jurisdiction == SelectionSignalMatch::None
        && platform == SelectionSignalMatch::None
        && topic_signals.is_empty()
    {
        return None;
    }
    Some(QuestionGapSignals {
        topic_signals,
        jurisdiction: selection_signal_label(&jurisdiction),
        platform: selection_signal_label(&platform),
    })
}

fn selection_signal_label(signal: &SelectionSignalMatch) -> String {
    match signal {
        SelectionSignalMatch::None => "none".to_owned(),
        SelectionSignalMatch::Unique(code) => code.clone(),
        SelectionSignalMatch::Ambiguous => "ambiguous".to_owned(),
    }
}

fn matched_topic_keywords(
    lowered_content: &str,
    domain_code: &str,
    selector: &QuestionSelectorConfig,
) -> Vec<String> {
    selector
        .topic_keywords
        .get(domain_code)
        .map(|keywords| {
            keywords
                .iter()
                .filter(|keyword| lowered_content.contains(&keyword.to_lowercase()))
                .map(|keyword| keyword.to_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn active_probe_matches_turn_context(
    probe: &StudentLearningProbe,
    user_content: Option<&str>,
    assistant_content: &str,
    bank: &QuestionBank,
    selector: &QuestionSelectorConfig,
    framework: &Framework,
    classifier: &ContextClassifier,
) -> bool {
    if probe
        .context_snapshot
        .get("questionType")
        .and_then(Value::as_str)
        != Some("single_choice")
    {
        return true;
    }
    let Some(question_id) = probe
        .context_snapshot
        .get("questionId")
        .and_then(Value::as_str)
    else {
        return false;
    };

    let domain_matches = select_turn_context_domain(
        user_content,
        assistant_content,
        &HashMap::new(),
        &framework.domains,
        classifier,
        OffsetDateTime::now_utc(),
        framework.probe_policy.retention_review_interval_days,
    )
    .is_some_and(|(domain, _, _)| domain.code == probe.knowledge_code);
    if !domain_matches {
        return false;
    }

    contextual_question_ids_for_turn(
        user_content,
        assistant_content,
        &probe.knowledge_code,
        bank,
        selector,
    )
    .is_none_or(|candidate_ids| {
        candidate_ids
            .iter()
            .any(|candidate| candidate == question_id)
    })
}

fn contextual_question_ids_for_content(
    content: &str,
    domain_code: &str,
    bank: &QuestionBank,
    selector: &QuestionSelectorConfig,
) -> Option<Vec<String>> {
    let lowered = content.to_lowercase();
    let jurisdiction = selection_signal_match(&lowered, &selector.jurisdictions);
    let platform = selection_signal_match(&lowered, &selector.platforms);
    let topic_keywords = matched_topic_keywords(&lowered, domain_code, selector);
    let has_scope_signal =
        jurisdiction != SelectionSignalMatch::None || platform != SelectionSignalMatch::None;
    if !has_scope_signal && topic_keywords.is_empty() {
        return None;
    }
    if jurisdiction == SelectionSignalMatch::Ambiguous
        || platform == SelectionSignalMatch::Ambiguous
    {
        return Some(Vec::new());
    }

    let mut candidates = bank
        .questions
        .iter()
        .filter(|question| question.domain_code == domain_code)
        .collect::<Vec<_>>();
    if let SelectionSignalMatch::Unique(code) = &jurisdiction {
        candidates.retain(|question| {
            question_applicability_scope(&question.id, question.version)
                .is_some_and(|scope| scope.jurisdiction_code == *code)
        });
        if candidates.is_empty() {
            return Some(Vec::new());
        }
    }
    if let SelectionSignalMatch::Unique(code) = &platform {
        candidates.retain(|question| {
            question_applicability_scope(&question.id, question.version)
                .is_some_and(|scope| scope.platform_code == *code)
        });
        if candidates.is_empty() {
            return Some(Vec::new());
        }
    }

    if !topic_keywords.is_empty() {
        let mut explicit_question_scores = HashMap::<&str, usize>::new();
        if let Some(mappings) = selector.topic_question_ids.get(domain_code) {
            for question_id in topic_keywords
                .iter()
                .filter_map(|keyword| mappings.get(keyword))
                .flatten()
            {
                *explicit_question_scores
                    .entry(question_id.as_str())
                    .or_default() += 1;
            }
        }
        if explicit_question_scores.is_empty() {
            let scored = candidates
                .into_iter()
                .map(|question| {
                    let text = question_search_text(question);
                    let score = topic_keywords
                        .iter()
                        .filter(|keyword| text.contains(keyword.as_str()))
                        .count();
                    (question, score)
                })
                .collect::<Vec<_>>();
            let max_score = scored
                .iter()
                .map(|(_, score)| *score)
                .max()
                .unwrap_or_default();
            if max_score == 0 {
                return Some(Vec::new());
            }
            candidates = scored
                .into_iter()
                .filter(|(_, score)| *score == max_score)
                .map(|(question, _)| question)
                .collect();
        } else {
            let max_score = explicit_question_scores
                .values()
                .copied()
                .max()
                .unwrap_or_default();
            candidates.retain(|question| {
                explicit_question_scores
                    .get(question.id.as_str())
                    .is_some_and(|score| *score == max_score)
            });
            if candidates.is_empty() {
                return Some(Vec::new());
            }
        }
    }

    Some(
        candidates
            .into_iter()
            .map(|question| question.id.clone())
            .collect(),
    )
}

fn selection_signal_match(
    content: &str,
    signal_groups: &HashMap<String, Vec<String>>,
) -> SelectionSignalMatch {
    let matched_codes = signal_groups
        .iter()
        .filter(|(_, signals)| {
            signals
                .iter()
                .any(|signal| content.contains(&signal.to_lowercase()))
        })
        .map(|(code, _)| code.clone())
        .collect::<Vec<_>>();
    match matched_codes.as_slice() {
        [] => SelectionSignalMatch::None,
        [code] => SelectionSignalMatch::Unique(code.clone()),
        _ => SelectionSignalMatch::Ambiguous,
    }
}

fn question_search_text(question: &DiagnosticQuestion) -> String {
    let mut text = format!("{} {}", question.prompt, question.reference_answer);
    for choice in &question.choices {
        text.push(' ');
        text.push_str(&choice.text);
    }
    for criterion in question
        .rubric
        .criteria
        .iter()
        .chain(&question.rubric.contradictions)
    {
        for concept in &criterion.concepts {
            text.push(' ');
            text.push_str(concept);
        }
    }
    text.to_lowercase()
}

fn select_context_domain(
    content: &str,
    latest_question_assessments: &HashMap<String, HashMap<String, LatestQuestionAssessment>>,
    domains: &[FrameworkDomain],
    classifier: &ContextClassifier,
    now: OffsetDateTime,
    retention_review_interval_days: i64,
) -> Option<(FrameworkDomain, usize, DomainAssessmentNeed)> {
    let lowered = content.to_lowercase();
    let mut best: Option<(usize, f64, usize, FrameworkDomain, DomainAssessmentNeed)> = None;
    for (index, domain) in domains.iter().enumerate() {
        let Some(assessment_need) = domain_assessment_need(
            latest_question_assessments.get(&domain.code),
            now,
            retention_review_interval_days,
        ) else {
            continue;
        };
        let score = classifier
            .domains
            .get(&domain.code)
            .map(|config| {
                config
                    .keywords
                    .iter()
                    .filter(|keyword| lowered.contains(&**keyword))
                    .count()
            })
            .unwrap_or_default();
        if score == 0 {
            continue;
        }
        let should_replace =
            best.as_ref()
                .is_none_or(|(best_score, best_importance, best_index, _, _)| {
                    score > *best_score
                        || (score == *best_score && domain.importance > *best_importance)
                        || (score == *best_score
                            && domain.importance == *best_importance
                            && index < *best_index)
                });
        if should_replace {
            best = Some((
                score,
                domain.importance,
                index,
                domain.clone(),
                assessment_need,
            ));
        }
    }
    best.map(|(score, _, _, domain, assessment_need)| (domain, score, assessment_need))
}

fn evidence_assessment_outcome(
    entry: &repo::entity::student_learning_evidence::Model,
) -> Option<(&str, &str, Option<&str>, String)> {
    entry
        .rubric_hits
        .get("outcome")
        .and_then(Value::as_str)
        .map(|outcome| {
            let question_id = entry.rubric_hits.get("questionId").and_then(Value::as_str);
            (
                entry.knowledge_code.as_str(),
                outcome,
                question_id,
                evidence_assessment_angle_code(entry, question_id),
            )
        })
}

fn evidence_timed_assessment_outcome(
    entry: &repo::entity::student_learning_evidence::Model,
) -> Option<(&str, &str, Option<&str>, String, OffsetDateTime)> {
    evidence_assessment_outcome(entry).map(|(knowledge_code, outcome, question_id, angle_code)| {
        (
            knowledge_code,
            outcome,
            question_id,
            angle_code,
            entry.created_at,
        )
    })
}

fn evidence_assessment_angle_code(
    entry: &repo::entity::student_learning_evidence::Model,
    question_id: Option<&str>,
) -> String {
    let expected_prefix = format!("{}.", entry.knowledge_code);
    if let Some(angle_code) = entry
        .rubric_hits
        .get("assessmentAngleCode")
        .and_then(Value::as_str)
        .filter(|angle_code| {
            angle_code.starts_with(&expected_prefix) && angle_code.len() > expected_prefix.len()
        })
    {
        return angle_code.to_owned();
    }
    let question_version = entry
        .rubric_hits
        .get("questionVersion")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .unwrap_or(1);
    if let Some(angle_code) = question_id
        .and_then(|question_id| assessment_angle_code(question_id, question_version))
        .filter(|angle_code| angle_code.starts_with(&expected_prefix))
    {
        return angle_code.to_owned();
    }
    format!("{}.legacy_unmapped", entry.knowledge_code)
}

fn latest_question_assessments_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>, String, OffsetDateTime)>,
) -> HashMap<String, HashMap<String, LatestQuestionAssessment>> {
    let mut assessments_by_domain = HashMap::new();
    for (knowledge_code, outcome, question_id, angle_code, created_at) in outcomes {
        if !matches!(outcome, "correct" | "partial" | "incorrect") {
            continue;
        }
        let Some(question_id) = question_id else {
            continue;
        };
        assessments_by_domain
            .entry(knowledge_code.to_owned())
            .or_insert_with(HashMap::new)
            .entry(angle_code)
            .or_insert_with(|| LatestQuestionAssessment {
                question_id: question_id.to_owned(),
                outcome: outcome.to_owned(),
                created_at,
            });
    }
    assessments_by_domain
}

fn retained_knowledge_count_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>, String, OffsetDateTime)>,
    retention_review_interval_days: i64,
) -> usize {
    latest_question_assessments_newest_first(outcomes)
        .values()
        .filter(|latest_assessments| {
            domain_has_retained_evidence(latest_assessments, retention_review_interval_days)
        })
        .count()
}

fn latest_question_outcomes_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>, &'a str)>,
) -> HashMap<String, HashMap<String, String>> {
    let mut outcomes_by_domain = HashMap::new();
    for (knowledge_code, outcome, question_id, _) in outcomes {
        if !matches!(outcome, "correct" | "partial" | "incorrect") {
            continue;
        }
        let Some(question_id) = question_id else {
            continue;
        };
        outcomes_by_domain
            .entry(knowledge_code.to_owned())
            .or_insert_with(HashMap::new)
            .entry(question_id.to_owned())
            .or_insert_with(|| outcome.to_owned());
    }
    outcomes_by_domain
}

fn latest_angle_outcomes_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>, &'a str)>,
) -> HashMap<String, HashMap<String, String>> {
    let mut outcomes_by_domain = HashMap::new();
    for (knowledge_code, outcome, question_id, angle_code) in outcomes {
        if !matches!(outcome, "correct" | "partial" | "incorrect") || question_id.is_none() {
            continue;
        }
        outcomes_by_domain
            .entry(knowledge_code.to_owned())
            .or_insert_with(HashMap::new)
            .entry(angle_code.to_owned())
            .or_insert_with(|| outcome.to_owned());
    }
    outcomes_by_domain
}

fn domain_assessment_need(
    latest_assessments: Option<&HashMap<String, LatestQuestionAssessment>>,
    now: OffsetDateTime,
    retention_review_interval_days: i64,
) -> Option<DomainAssessmentNeed> {
    let Some(latest_assessments) = latest_assessments else {
        return Some(DomainAssessmentNeed::InitialOrCorrection);
    };
    let demonstrates_understanding = latest_assessments.len() >= 2
        && latest_assessments
            .values()
            .any(|assessment| assessment.outcome == "correct")
        && latest_assessments
            .values()
            .all(|assessment| matches!(assessment.outcome.as_str(), "correct" | "partial"));
    if !demonstrates_understanding {
        return Some(DomainAssessmentNeed::InitialOrCorrection);
    }

    let latest = latest_assessments
        .values()
        .map(|assessment| assessment.created_at)
        .max()?;
    let review_interval = Duration::days(retention_review_interval_days);
    if !domain_has_retained_evidence(latest_assessments, retention_review_interval_days)
        && now - latest >= review_interval
    {
        return Some(DomainAssessmentNeed::SpacedRetentionReview);
    }
    Some(DomainAssessmentNeed::OngoingReinforcement)
}

fn domain_has_retained_evidence(
    latest_assessments: &HashMap<String, LatestQuestionAssessment>,
    retention_review_interval_days: i64,
) -> bool {
    let demonstrates_understanding = latest_assessments.len() >= 2
        && latest_assessments
            .values()
            .any(|assessment| assessment.outcome == "correct")
        && latest_assessments
            .values()
            .all(|assessment| matches!(assessment.outcome.as_str(), "correct" | "partial"));
    if !demonstrates_understanding {
        return false;
    }
    let Some(earliest) = latest_assessments
        .values()
        .map(|assessment| assessment.created_at)
        .min()
    else {
        return false;
    };
    let Some(latest) = latest_assessments
        .values()
        .map(|assessment| assessment.created_at)
        .max()
    else {
        return false;
    };
    latest - earliest >= Duration::days(retention_review_interval_days)
}

fn preferred_question_ids_for_domain(
    bank: &crate::api::learning_question_bank::QuestionBank,
    domain_code: &str,
    latest_assessments: Option<&HashMap<String, LatestQuestionAssessment>>,
) -> Vec<String> {
    let Some(latest_assessments) = latest_assessments else {
        return Vec::new();
    };
    if latest_assessments.len() >= 2 {
        let incorrect = latest_assessments
            .values()
            .filter(|assessment| assessment.outcome == "incorrect")
            .map(|assessment| assessment.question_id.clone())
            .collect::<Vec<_>>();
        if !incorrect.is_empty() {
            return incorrect;
        }
    }
    bank.questions
        .iter()
        .filter(|question| {
            question.domain_code == domain_code
                && assessment_angle_code(&question.id, question.version)
                    .is_some_and(|angle_code| !latest_assessments.contains_key(angle_code))
        })
        .map(|question| question.id.clone())
        .collect()
}

fn merge_question_preferences(
    contextual_question_ids: Option<Vec<String>>,
    assessment_question_ids: Vec<String>,
) -> Option<Vec<String>> {
    let Some(contextual_question_ids) = contextual_question_ids else {
        return Some(assessment_question_ids);
    };
    if contextual_question_ids.is_empty() {
        return None;
    }
    if assessment_question_ids.is_empty() {
        return Some(contextual_question_ids);
    }
    let assessment_question_ids = assessment_question_ids.into_iter().collect::<HashSet<_>>();
    let intersection = contextual_question_ids
        .iter()
        .filter(|question_id| assessment_question_ids.contains(*question_id))
        .cloned()
        .collect::<Vec<_>>();
    Some(if intersection.is_empty() {
        contextual_question_ids
    } else {
        intersection
    })
}

fn question_pool_for_request(
    is_continuation: bool,
    contextual_question_ids: Option<Vec<String>>,
    assessment_question_ids: Vec<String>,
) -> Option<Vec<String>> {
    if !is_continuation {
        return merge_question_preferences(contextual_question_ids, assessment_question_ids);
    }
    match contextual_question_ids {
        Some(question_ids) if question_ids.is_empty() => None,
        Some(question_ids) => Some(question_ids),
        None => Some(assessment_question_ids),
    }
}

fn prioritize_review_candidates(
    candidate_question_ids: Vec<String>,
    latest_assessments: Option<&HashMap<String, LatestQuestionAssessment>>,
    bank: &QuestionBank,
) -> Vec<String> {
    let priority = |question_id: &str| {
        let outcome = bank
            .questions
            .iter()
            .find(|question| question.id == question_id)
            .and_then(|question| assessment_angle_code(&question.id, question.version))
            .and_then(|angle_code| latest_assessments.and_then(|items| items.get(angle_code)))
            .filter(|assessment| assessment.question_id == question_id)
            .map(|assessment| assessment.outcome.as_str());
        match outcome {
            Some("incorrect") => 0,
            Some("partial") => 1,
            None => 2,
            Some("correct") => 3,
            Some(_) => 4,
        }
    };
    let best_priority = candidate_question_ids
        .iter()
        .map(|question_id| priority(question_id))
        .min();
    candidate_question_ids
        .into_iter()
        .filter(|question_id| Some(priority(question_id)) == best_priority)
        .collect()
}

fn probe_contextual_question_ids(probe: &StudentLearningProbe) -> Option<Vec<String>> {
    let value = probe.context_snapshot.get("contextualQuestionIds")?;
    let Some(values) = value.as_array() else {
        return Some(Vec::new());
    };
    let question_ids = values
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if question_ids.len() == values.len() {
        Some(question_ids)
    } else {
        Some(Vec::new())
    }
}

fn probe_continuation_depth(probe: &StudentLearningProbe) -> u64 {
    probe
        .context_snapshot
        .get("continuationDepth")
        .and_then(Value::as_u64)
        .unwrap_or_default()
}

fn probe_practice_cycle(probe: &StudentLearningProbe) -> u64 {
    probe
        .context_snapshot
        .get("practiceCycle")
        .and_then(Value::as_u64)
        .unwrap_or_default()
}

fn practice_purpose_for_question(
    practice_cycle: u64,
    assessment_angle_code: &str,
    latest_assessments: Option<&HashMap<String, LatestQuestionAssessment>>,
) -> PracticePurpose {
    if practice_cycle == 0 {
        return PracticePurpose::InitialCoverage;
    }
    match latest_assessments
        .and_then(|items| items.get(assessment_angle_code))
        .map(|assessment| assessment.outcome.as_str())
    {
        Some("incorrect" | "partial") => PracticePurpose::CorrectionReview,
        _ => PracticePurpose::ReinforcementReview,
    }
}

fn probe_continuation_question_ids(probe: &StudentLearningProbe) -> Vec<String> {
    let mut question_ids = probe
        .context_snapshot
        .get("continuationQuestionIds")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(question_id) = probe
        .context_snapshot
        .get("questionId")
        .and_then(Value::as_str)
    {
        question_ids.push(question_id.to_owned());
    }
    question_ids.sort();
    question_ids.dedup();
    question_ids
}

fn next_probe_continuation_depth(
    probe: &StudentLearningProbe,
    max_continuation_depth: u64,
) -> Option<u64> {
    let next_depth = probe_continuation_depth(probe).checked_add(1)?;
    (max_continuation_depth == 0 || next_depth <= max_continuation_depth).then_some(next_depth)
}

fn probe_can_continue(probe: &StudentLearningProbe, max_continuation_depth: u64) -> bool {
    next_probe_continuation_depth(probe, max_continuation_depth).is_some()
}

fn continuation_candidates_for_next_cycle(
    preferred_question_ids: Vec<String>,
    answered_question_ids: &[String],
) -> (Vec<String>, Vec<String>, bool) {
    let remaining_question_ids = preferred_question_ids
        .iter()
        .filter(|question_id| !answered_question_ids.contains(question_id))
        .cloned()
        .collect::<Vec<_>>();
    if remaining_question_ids.is_empty() && !preferred_question_ids.is_empty() {
        (preferred_question_ids, Vec::new(), true)
    } else {
        (
            remaining_question_ids,
            answered_question_ids.to_vec(),
            false,
        )
    }
}

fn is_direct_probe_answer_reply(
    probe: &StudentLearningProbe,
    answer_message: &repo::table::conversation_messages::ConversationMessage,
    assistant_message: &repo::table::conversation_messages::ConversationMessage,
) -> bool {
    probe.answer_message_id == Some(answer_message.id)
        && probe.conversation_id == Some(answer_message.conversation_id)
        && answer_message.role == "user"
        && assistant_message.conversation_id == answer_message.conversation_id
        && assistant_message.role == "assistant"
        && assistant_message.sequence == answer_message.sequence + 1
}

pub(crate) fn minimum_probe_answer_characters() -> BizResult<usize> {
    Ok(framework()?.probe_policy.minimum_answer_characters)
}

fn rfc3339(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).unwrap_or_else(|_| value.to_string())
}

async fn commit_or_rollback<T>(tx: DbContext, result: BizResult<T>) -> BizResult<T> {
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(err) => {
            tx.rollback().await?;
            Err(err)
        }
    }
}
fn parse_user_id(value: &str) -> BizResult<Uuid> {
    Uuid::parse_str(value).map_err(|_| invalid("invalid current user id"))
}
fn parse_optional_uuid(value: Option<String>) -> BizResult<Option<Uuid>> {
    value
        .map(|v| Uuid::parse_str(&v).map_err(|_| invalid("invalid uuid")))
        .transpose()
}
fn invalid(message: &str) -> BizError {
    BizError::new(app_error::LEARNING_REQUEST_INVALID, message.to_owned())
}
fn map_profile(
    p: StudentLearningProfile,
    evidence: &[repo::entity::student_learning_evidence::Model],
    framework: &Framework,
) -> LearningProfileResponse {
    let retention_review_interval_days = framework.probe_policy.retention_review_interval_days;
    let assessment_summary =
        summarize_assessments(&p.mastery_map, evidence, retention_review_interval_days);
    let assessment_timeline = build_assessment_timeline(
        evidence,
        retention_review_interval_days,
        OffsetDateTime::now_utc(),
    );
    let domain_assessments = build_domain_assessments(
        &p.mastery_map,
        evidence,
        &framework.domains,
        retention_review_interval_days,
    );
    LearningProfileResponse {
        framework_version: p.framework_version,
        mastery_map: p.mastery_map,
        engagement_summary: p.engagement_summary,
        coverage_rate: p.coverage_rate,
        assessment_summary,
        assessment_timeline,
        domain_assessments,
        analysis_status: p.analysis_status,
        last_analyzed_at: p.last_analyzed_at,
        enabled: p.enabled,
    }
}

fn build_domain_assessments(
    mastery_map: &Value,
    evidence: &[repo::entity::student_learning_evidence::Model],
    domains: &[FrameworkDomain],
    retention_review_interval_days: i64,
) -> Vec<LearningDomainAssessment> {
    domains
        .iter()
        .map(|domain| {
            let domain_evidence = evidence
                .iter()
                .filter(|entry| entry.knowledge_code == domain.code)
                .collect::<Vec<_>>();
            let observation_count = domain_evidence
                .iter()
                .filter(|entry| {
                    entry.rubric_hits.get("outcome").and_then(Value::as_str)
                        == Some("observed_unscored")
                })
                .count() as u64;
            let mut assessments = domain_evidence
                .iter()
                .filter_map(|entry| {
                    let (_, outcome, question_id, angle_code, created_at) =
                        evidence_timed_assessment_outcome(entry)?;
                    if !matches!(outcome, "correct" | "partial" | "incorrect") {
                        return None;
                    }
                    Some((*entry, outcome, question_id, angle_code, created_at))
                })
                .collect::<Vec<_>>();
            assessments.sort_by(|left, right| right.4.cmp(&left.4));

            let assessment_count = assessments.len() as u64;
            let correct_count = assessments
                .iter()
                .filter(|(_, outcome, _, _, _)| *outcome == "correct")
                .count() as u64;
            let partial_count = assessments
                .iter()
                .filter(|(_, outcome, _, _, _)| *outcome == "partial")
                .count() as u64;
            let incorrect_count = assessments
                .iter()
                .filter(|(_, outcome, _, _, _)| *outcome == "incorrect")
                .count() as u64;
            let distinct_question_count = assessments
                .iter()
                .filter_map(|(_, _, question_id, _, _)| *question_id)
                .collect::<HashSet<_>>()
                .len();
            let latest_assessments =
                latest_question_assessments_newest_first(assessments.iter().map(
                    |(_, outcome, question_id, angle_code, created_at)| {
                        (
                            domain.code.as_str(),
                            *outcome,
                            *question_id,
                            angle_code.clone(),
                            *created_at,
                        )
                    },
                ))
                .remove(&domain.code)
                .unwrap_or_default();
            let assessment_angle_count = latest_assessments.len();
            let demonstrates_understanding = assessment_angle_count >= 2
                && latest_assessments
                    .values()
                    .any(|assessment| assessment.outcome == "correct")
                && latest_assessments
                    .values()
                    .all(|assessment| matches!(assessment.outcome.as_str(), "correct" | "partial"));
            let retention_verified =
                domain_has_retained_evidence(&latest_assessments, retention_review_interval_days);
            let latest_outcome = assessments
                .first()
                .map(|(_, outcome, _, _, _)| (*outcome).to_owned());
            let latest_assessed_at = assessments.first().map(|(_, _, _, _, at)| *at);

            let (status_code, reason_code) = if assessment_count == 0 {
                ("not_assessed", "no_structured_assessment")
            } else if latest_outcome.as_deref() == Some("incorrect") {
                ("needs_attention", "latest_answer_incorrect")
            } else if assessment_angle_count < 2 {
                ("collecting_evidence", "single_angle_only")
            } else if retention_verified {
                ("retained", "spaced_positive_evidence")
            } else if demonstrates_understanding {
                ("demonstrated", "multi_angle_positive_evidence")
            } else {
                ("developing", "mixed_or_partial_evidence")
            };

            let mut missing_evidence_codes = Vec::new();
            if assessment_count == 0 {
                missing_evidence_codes.push("first_assessment".to_owned());
            }
            if assessment_count > 0 && assessment_angle_count < 2 {
                missing_evidence_codes.push("another_assessment_angle".to_owned());
            }
            if assessment_count > 0 && !demonstrates_understanding {
                missing_evidence_codes.push("positive_correction".to_owned());
            }
            if demonstrates_understanding && !retention_verified {
                missing_evidence_codes.push("spaced_review".to_owned());
            }

            let next_action_code = match status_code {
                "not_assessed" => "answer_first_context_question",
                "needs_attention" => "correct_latest_gap",
                "collecting_evidence" => "answer_another_angle",
                "developing" => "strengthen_mixed_results",
                "demonstrated" => "repeat_after_interval",
                "retained" => "continue_context_practice",
                _ => "continue_context_practice",
            };

            let recent_evidence = assessments
                .iter()
                .take(3)
                .map(
                    |(entry, outcome, question_id, _, created_at)| LearningDomainEvidenceSummary {
                        evidence_id: entry.id.to_string(),
                        conversation_id: entry.conversation_id.to_string(),
                        question_id: question_id.map(ToOwned::to_owned),
                        question_text: evidence_string(&entry.rubric_hits, "questionText"),
                        selected_choice_text: evidence_string(
                            &entry.rubric_hits,
                            "selectedChoiceText",
                        ),
                        outcome: (*outcome).to_owned(),
                        created_at: *created_at,
                    },
                )
                .collect();

            LearningDomainAssessment {
                knowledge_code: domain.code.clone(),
                status_code: status_code.to_owned(),
                reason_code: reason_code.to_owned(),
                observed: mastery_map.get(&domain.code).is_some() || !domain_evidence.is_empty(),
                observation_count,
                assessment_count,
                distinct_question_count,
                assessment_angle_count,
                correct_count,
                partial_count,
                incorrect_count,
                latest_outcome,
                latest_assessed_at,
                retention_verified,
                missing_evidence_codes,
                next_action_code: next_action_code.to_owned(),
                recent_evidence,
            }
        })
        .collect()
}

fn build_assessment_timeline(
    evidence: &[repo::entity::student_learning_evidence::Model],
    retention_review_interval_days: i64,
    now: OffsetDateTime,
) -> Vec<LearningAssessmentTimelinePoint> {
    let current_week_start = utc_week_start(now);
    let empty_mastery = json!({});

    (0..ASSESSMENT_TIMELINE_WEEKS)
        .map(|index| {
            let weeks_ago = ASSESSMENT_TIMELINE_WEEKS - index - 1;
            let period_start = current_week_start - Duration::weeks(weeks_ago);
            let period_end = period_start + Duration::weeks(1);
            let mut snapshot = evidence
                .iter()
                .filter(|entry| entry.created_at < period_end)
                .cloned()
                .collect::<Vec<_>>();
            snapshot.sort_by(|left, right| right.created_at.cmp(&left.created_at));
            let summary =
                summarize_assessments(&empty_mastery, &snapshot, retention_review_interval_days);

            LearningAssessmentTimelinePoint {
                period_start: period_start.date().to_string(),
                assessed_knowledge_count: summary.assessed_knowledge_count,
                demonstrated_knowledge_count: summary.demonstrated_knowledge_count,
                retained_knowledge_count: summary.retained_knowledge_count,
            }
        })
        .collect()
}

fn utc_week_start(value: OffsetDateTime) -> OffsetDateTime {
    let utc = value.to_offset(UtcOffset::UTC).replace_time(Time::MIDNIGHT);
    let days_from_monday = match utc.weekday() {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };
    utc - Duration::days(days_from_monday)
}

pub(crate) fn summarize_assessments(
    mastery_map: &Value,
    evidence: &[repo::entity::student_learning_evidence::Model],
    retention_review_interval_days: i64,
) -> LearningAssessmentSummary {
    let outcomes = evidence
        .iter()
        .filter_map(evidence_assessment_outcome)
        .collect::<Vec<_>>();
    let mut evidence_summary =
        summarize_assessment_outcomes_with_angles_newest_first(outcomes.iter().map(
            |(knowledge_code, outcome, question_id, angle_code)| {
                (*knowledge_code, *outcome, *question_id, angle_code.as_str())
            },
        ));
    if evidence_summary.assessment_count > 0 {
        evidence_summary.retained_knowledge_count = retained_knowledge_count_newest_first(
            evidence
                .iter()
                .filter_map(evidence_timed_assessment_outcome),
            retention_review_interval_days,
        );
        return evidence_summary;
    }

    summarize_mastery_assessments(mastery_map)
}

#[cfg(test)]
fn summarize_assessment_outcomes_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>)>,
) -> LearningAssessmentSummary {
    let outcomes = outcomes.into_iter().collect::<Vec<_>>();
    let angle_codes = outcomes
        .iter()
        .map(|(knowledge_code, _, question_id)| question_id.unwrap_or(knowledge_code).to_string())
        .collect::<Vec<_>>();
    summarize_assessment_outcomes_with_angles_newest_first(outcomes.iter().zip(&angle_codes).map(
        |((knowledge_code, outcome, question_id), angle_code)| {
            (*knowledge_code, *outcome, *question_id, angle_code.as_str())
        },
    ))
}

fn summarize_assessment_outcomes_with_angles_newest_first<'a>(
    outcomes: impl IntoIterator<Item = (&'a str, &'a str, Option<&'a str>, &'a str)>,
) -> LearningAssessmentSummary {
    let outcomes = outcomes.into_iter().collect::<Vec<_>>();
    let latest_outcome_by_question_by_domain =
        latest_question_outcomes_newest_first(outcomes.iter().copied());
    let latest_outcome_by_angle_by_domain =
        latest_angle_outcomes_newest_first(outcomes.iter().copied());
    let mut summary = LearningAssessmentSummary::default();
    let mut latest_outcome_by_domain = HashMap::new();
    for (knowledge_code, outcome, _, _) in outcomes {
        match outcome {
            "correct" => summary.correct_count += 1,
            "partial" => summary.partial_count += 1,
            "incorrect" => summary.incorrect_count += 1,
            _ => continue,
        }
        summary.assessment_count += 1;
        latest_outcome_by_domain
            .entry(knowledge_code)
            .or_insert(outcome);
    }
    summary.assessed_knowledge_count = latest_outcome_by_domain.len();
    summary.distinct_assessment_question_count = latest_outcome_by_question_by_domain
        .values()
        .map(HashMap::len)
        .sum();
    summary.multi_angle_knowledge_count = latest_outcome_by_angle_by_domain
        .values()
        .filter(|latest_outcomes| latest_outcomes.len() >= 2)
        .count();
    summary.demonstrated_knowledge_count = latest_outcome_by_angle_by_domain
        .values()
        .filter(|latest_outcomes| {
            latest_outcomes.len() >= 2
                && latest_outcomes.values().any(|outcome| outcome == "correct")
                && latest_outcomes
                    .values()
                    .all(|outcome| matches!(outcome.as_str(), "correct" | "partial"))
        })
        .count();
    summary.needs_attention_knowledge_count = latest_outcome_by_domain
        .values()
        .filter(|outcome| **outcome == "incorrect")
        .count();
    summary
}

fn summarize_mastery_assessments(mastery_map: &Value) -> LearningAssessmentSummary {
    mastery_map
        .as_object()
        .map(|domains| {
            domains.values().fold(
                LearningAssessmentSummary::default(),
                |mut summary, value| {
                    let Some(domain) = value.as_object() else {
                        return summary;
                    };
                    let assessed_count = domain
                        .get("assessedCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    if assessed_count > 0 {
                        summary.assessed_knowledge_count += 1;
                    }
                    summary.assessment_count += assessed_count;
                    summary.correct_count += domain
                        .get("correctCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    summary.partial_count += domain
                        .get("partialCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    summary.incorrect_count += domain
                        .get("incorrectCount")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    if domain.get("evaluationStatus").and_then(Value::as_str)
                        == Some("needs_attention")
                    {
                        summary.needs_attention_knowledge_count += 1;
                    }
                    summary
                },
            )
        })
        .unwrap_or_default()
}
fn map_probe(p: StudentLearningProbe) -> LearningProbeResponse {
    let engaged = p.status == "answered" || p.context_snapshot.get("engagedAt").is_some();
    let question_id = p
        .context_snapshot
        .get("questionId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let question_version = p
        .context_snapshot
        .get("questionVersion")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok());
    let choices = p
        .context_snapshot
        .get("choices")
        .and_then(Value::as_array)
        .map(|choices| {
            choices
                .iter()
                .filter_map(|choice| {
                    Some(LearningProbeChoice {
                        id: choice.get("id")?.as_str()?.to_owned(),
                        text: choice.get("text")?.as_str()?.to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let question_type = p
        .context_snapshot
        .get("questionType")
        .and_then(Value::as_str)
        .filter(|question_type| *question_type == "single_choice")
        .unwrap_or("open_response")
        .to_owned();
    let interaction_mode = p
        .context_snapshot
        .get("interactionMode")
        .and_then(Value::as_str)
        .unwrap_or("contextual_micro")
        .to_owned();
    let reward_code = p
        .context_snapshot
        .get("rewardCode")
        .and_then(Value::as_str)
        .unwrap_or("progress_feedback")
        .to_owned();
    LearningProbeResponse {
        id: p.id.to_string(),
        question_id,
        question_version,
        question_type,
        choices,
        engaged,
        knowledge_code: p.knowledge_code,
        question_text: p.question_text,
        reason_code: p.reason_code,
        interaction_mode,
        reward_code,
        expires_at: p.expires_at.unwrap_or_else(OffsetDateTime::now_utc),
    }
}

fn is_structured_probe_answer_for_assistant(
    probe: &StudentLearningProbe,
    assistant_message: &repo::table::conversation_messages::ConversationMessage,
) -> bool {
    probe.status == "answered"
        && probe.answer_message_id.is_none()
        && probe.assistant_message_id == Some(assistant_message.id)
        && probe
            .context_snapshot
            .get("answerMode")
            .and_then(Value::as_str)
            == Some("structured_choice")
}

fn existing_turn_probe_response(
    probe: &StudentLearningProbe,
    assistant_message: &repo::table::conversation_messages::ConversationMessage,
) -> Option<LearningProbeResponse> {
    is_structured_probe_answer_for_assistant(probe, assistant_message)
        .then(|| map_probe(probe.clone()))
}

pub(crate) fn map_evidence_entry(
    entry: repo::entity::student_learning_evidence::Model,
    probe: Option<&StudentLearningProbe>,
) -> LearningEvidenceEntry {
    let assessment_outcome = entry
        .rubric_hits
        .get("outcome")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let question_id = entry
        .rubric_hits
        .get("questionId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let rubric_version = entry
        .rubric_hits
        .get("rubricVersion")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok());
    let probe_id = entry.probe_id.map(|id| id.to_string());
    let question_text = evidence_string(&entry.rubric_hits, "questionText")
        .or_else(|| probe.map(|probe| probe.question_text.clone()));
    let selected_choice_id =
        evidence_string(&entry.rubric_hits, "selectedChoiceId").or_else(|| {
            probe.and_then(|probe| {
                probe
                    .context_snapshot
                    .get("selectedChoiceId")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
        });
    let selected_choice_text =
        evidence_string(&entry.rubric_hits, "selectedChoiceText").or_else(|| {
            let probe = probe?;
            let choice_id = selected_choice_id.as_deref()?;
            let (_, question) = question_for_probe(&probe.context_snapshot).ok().flatten()?;
            question
                .choices
                .iter()
                .find(|choice| choice.id == choice_id)
                .map(|choice| choice.text.clone())
        });
    LearningEvidenceEntry {
        id: entry.id.to_string(),
        conversation_id: entry.conversation_id.to_string(),
        probe_id,
        question_text,
        selected_choice_id,
        selected_choice_text,
        knowledge_code: entry.knowledge_code,
        evidence_type: entry.evidence_type,
        direction: entry.direction,
        strength: entry.strength,
        summary: entry.summary,
        assessment_outcome,
        question_id,
        rubric_version,
        created_at: entry.created_at,
    }
}

fn evidence_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn selected_choice_for_result(
    probe: &StudentLearningProbe,
    evidence_rubric_hits: Option<&Value>,
    question: Option<&DiagnosticQuestion>,
) -> (Option<String>, Option<String>) {
    let Some(rubric_hits) = evidence_rubric_hits else {
        return (None, None);
    };
    if probe.status != "answered"
        || probe
            .context_snapshot
            .get("answerMode")
            .and_then(Value::as_str)
            != Some("structured_choice")
        || question.is_none_or(|item| item.question_type != "single_choice")
    {
        return (None, None);
    }

    let selected_choice_id = evidence_string(rubric_hits, "selectedChoiceId")
        .or_else(|| evidence_string(&probe.context_snapshot, "selectedChoiceId"));
    let selected_choice_text = evidence_string(rubric_hits, "selectedChoiceText").or_else(|| {
        let choice_id = selected_choice_id.as_deref()?;
        question?
            .choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .map(|choice| choice.text.clone())
    });

    (selected_choice_id, selected_choice_text)
}

fn answered_correct_choice_text(
    probe: &StudentLearningProbe,
    evidence: Option<&repo::entity::student_learning_evidence::Model>,
    question: Option<&DiagnosticQuestion>,
) -> Option<String> {
    if probe.status != "answered" || evidence.is_none() {
        return None;
    }
    let question = question.filter(|item| item.question_type == "single_choice")?;
    let correct_choice_id = question.correct_choice_id.as_deref()?;
    question
        .choices
        .iter()
        .find(|choice| choice.id == correct_choice_id)
        .map(|choice| choice.text.clone())
}

fn answer_comparison_code(
    evidence: Option<&repo::entity::student_learning_evidence::Model>,
    selected_choice_id: Option<&str>,
    question: Option<&DiagnosticQuestion>,
) -> Option<String> {
    evidence?;
    let question = question.filter(|item| item.question_type == "single_choice")?;
    let code = match (selected_choice_id, question.correct_choice_id.as_deref()) {
        (Some(selected), Some(correct)) if selected == correct => "matches_reference",
        (Some(_), Some(_)) => "differs_from_reference",
        _ => "not_available",
    };
    Some(code.to_owned())
}

fn question_knowledge_points(question: &DiagnosticQuestion) -> Vec<String> {
    let mut seen = HashSet::new();
    question
        .rubric
        .criteria
        .iter()
        .flat_map(|criterion| criterion.concepts.iter())
        .filter(|concept| seen.insert((*concept).clone()))
        .cloned()
        .collect()
}

fn instructional_reference_explanation(
    question: &DiagnosticQuestion,
    outcome: Option<&str>,
    selected_choice_text: Option<&str>,
    correct_choice_text: Option<&str>,
    knowledge_points: &[String],
) -> String {
    let mut explanation = question.reference_answer.trim().to_owned();
    if knowledge_points.is_empty() {
        return explanation;
    }

    explanation.push_str(" 本题的核心判断条件包括：");
    explanation.push_str(&knowledge_points.join("；"));
    explanation.push_str("。这些条件需要结合起来判断，不能只抓住某个数字、术语或单一动作。");

    match (outcome, selected_choice_text, correct_choice_text) {
        (Some("incorrect"), Some(selected), Some(correct)) => {
            explanation.push_str(" 你选择的“");
            explanation.push_str(selected);
            explanation.push_str("”没有完整满足上述条件；应对照“");
            explanation.push_str(correct);
            explanation.push_str(
                "”逐项检查补上了哪些必要信息。遇到类似场景时，先确认已知事实、判断条件和适用边界，再决定解释或行动，避免把局部信号直接当成最终结论。",
            );
        }
        (Some("correct"), _, Some(correct)) => {
            explanation.push_str(" 你的选择“");
            explanation.push_str(correct);
            explanation.push_str(
                "”与上述条件一致。遇到类似场景时，不要只记选项文字，应先找出已知信息、判断条件和适用边界，再说明结论为什么成立、其他做法缺少什么。",
            );
        }
        _ => explanation.push_str(
            " 复盘时应先找出已知信息、判断条件和适用边界，再说明结论为什么成立，并把同一判断方法迁移到新的商品或运营场景。",
        ),
    }

    explanation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_questions_produce_detailed_source_bounded_teaching_explanations() {
        let bank = question_bank().expect("question bank should parse");

        for question in &bank.questions {
            let knowledge_points = question_knowledge_points(question);
            let correct = question
                .choices
                .iter()
                .find(|choice| Some(choice.id.as_str()) == question.correct_choice_id.as_deref())
                .expect("current choice question should have a correct choice");
            let correct_explanation = instructional_reference_explanation(
                question,
                Some("correct"),
                Some(&correct.text),
                Some(&correct.text),
                &knowledge_points,
            );

            assert!(
                correct_explanation.starts_with(question.reference_answer.trim()),
                "reference answer changed for {}",
                question.id
            );
            assert!(correct_explanation.contains("本题的核心判断条件包括"));
            assert!(correct_explanation.contains(&correct.text));
            assert!(
                correct_explanation.chars().count() >= 140,
                "teaching explanation remains too short for {}",
                question.id
            );

            let incorrect = question
                .choices
                .iter()
                .find(|choice| choice.id != correct.id)
                .expect("current choice question should have a distractor");
            let incorrect_explanation = instructional_reference_explanation(
                question,
                Some("incorrect"),
                Some(&incorrect.text),
                Some(&correct.text),
                &knowledge_points,
            );
            assert!(incorrect_explanation.contains(&incorrect.text));
            assert!(incorrect_explanation.contains(&correct.text));
            assert!(incorrect_explanation.contains("没有完整满足上述条件"));
        }
    }

    #[test]
    fn evidence_response_keeps_its_source_conversation() {
        let conversation_id = Uuid::new_v4();
        let entry = repo::entity::student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id,
            source_message_id: Some(Uuid::new_v4()),
            probe_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "conversion".to_owned(),
            evidence_type: "explanation".to_owned(),
            direction: "neutral".to_owned(),
            strength: 0.5,
            confidence: 0.8,
            summary: "test evidence".to_owned(),
            rubric_hits: json!({ "outcome": "observed_unscored" }),
            analyzer_version: "test".to_owned(),
            created_at: OffsetDateTime::now_utc(),
        };

        let response = map_evidence_entry(entry, None);

        assert_eq!(response.conversation_id, conversation_id.to_string());
    }

    #[test]
    fn structured_evidence_recovers_question_and_selection_from_probe() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let probe_id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        let entry = repo::entity::student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id,
            conversation_id,
            source_message_id: None,
            probe_id: Some(probe_id),
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "conversion".to_owned(),
            evidence_type: "probe_answer".to_owned(),
            direction: "positive".to_owned(),
            strength: 1.0,
            confidence: 1.0,
            summary: "legacy structured summary".to_owned(),
            rubric_hits: json!({
                "questionId": "conversion.ga4_begin_checkout.choice.003",
                "outcome": "correct"
            }),
            analyzer_version: "deterministic_question_bank_v1".to_owned(),
            created_at,
        };
        let probe = StudentLearningProbe {
            id: probe_id,
            user_id,
            conversation_id: Some(conversation_id),
            assistant_message_id: Some(Uuid::new_v4()),
            answer_message_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "conversion".to_owned(),
            status: "answered".to_owned(),
            question_text: "Which event starts checkout?".to_owned(),
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
        };

        let response = map_evidence_entry(entry, Some(&probe));

        assert_eq!(
            response.probe_id.as_deref(),
            Some(probe_id.to_string().as_str())
        );
        assert_eq!(
            response.question_text.as_deref(),
            Some("Which event starts checkout?")
        );
        assert_eq!(
            response.selected_choice_id.as_deref(),
            Some("begin_checkout")
        );
        assert_eq!(
            response.selected_choice_text.as_deref(),
            Some("begin_checkout")
        );
    }

    #[test]
    fn probe_result_selection_prefers_evidence_and_recovers_legacy_text() {
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        probe.context_snapshot = json!({
            "answerMode": "structured_choice",
            "questionBankVersion": "ecommerce_diagnostic_v6",
            "sourceRevision": "2026-08-27",
            "questionId": "conversion.ga4_begin_checkout.choice.003",
            "questionVersion": 1,
            "rubricVersion": 1,
            "selectedChoiceId": "begin_checkout"
        });
        let (_, question) = question_for_probe(&probe.context_snapshot)
            .expect("question lookup")
            .expect("historical structured question");

        let legacy_evidence = json!({ "outcome": "correct" });
        let recovered = selected_choice_for_result(&probe, Some(&legacy_evidence), Some(&question));
        assert_eq!(recovered.0.as_deref(), Some("begin_checkout"));
        assert_eq!(recovered.1.as_deref(), Some("begin_checkout"));

        let current_evidence = json!({
            "outcome": "incorrect",
            "selectedChoiceId": "purchase",
            "selectedChoiceText": "purchase"
        });
        let current = selected_choice_for_result(&probe, Some(&current_evidence), Some(&question));
        assert_eq!(current.0.as_deref(), Some("purchase"));
        assert_eq!(current.1.as_deref(), Some("purchase"));

        assert_eq!(
            selected_choice_for_result(&probe, None, Some(&question)),
            (None, None)
        );
        probe.status = "offered".to_owned();
        assert_eq!(
            selected_choice_for_result(&probe, Some(&current_evidence), Some(&question)),
            (None, None)
        );
    }

    #[test]
    fn probe_result_serializes_learning_explanation_without_answer_key_id() {
        let response = LearningProbeResultResponse {
            id: Uuid::new_v4().to_string(),
            status: "answered".to_owned(),
            can_continue: false,
            outcome: Some("incorrect".to_owned()),
            score: Some(0.0),
            feedback_code: Some("learning_probe_incorrect".to_owned()),
            selected_choice_id: Some("purchase".to_owned()),
            selected_choice_text: Some("purchase".to_owned()),
            correct_choice_text: Some("begin_checkout".to_owned()),
            answer_comparison_code: Some("differs_from_reference".to_owned()),
            knowledge_points: vec!["区分开始结账与购买完成".to_owned()],
            demonstrated_points: Vec::new(),
            next_point: None,
            reference_explanation: Some("Reviewed explanation".to_owned()),
            source_labels: Vec::new(),
        };

        let serialized = serde_json::to_value(response).expect("serialize probe result");
        assert_eq!(serialized["selectedChoiceId"], "purchase");
        assert_eq!(serialized["selectedChoiceText"], "purchase");
        assert!(serialized.get("correctChoiceId").is_none());
        assert_eq!(serialized["correctChoiceText"], "begin_checkout");
        assert_eq!(serialized["answerComparisonCode"], "differs_from_reference");
        assert_eq!(serialized["knowledgePoints"][0], "区分开始结账与购买完成");
        assert!(serialized.get("rubric").is_none());
    }

    fn domain(code: &str, importance: f64) -> FrameworkDomain {
        FrameworkDomain {
            code: code.to_owned(),
            importance,
        }
    }

    fn assessment(outcome: &str, created_at: OffsetDateTime) -> LatestQuestionAssessment {
        LatestQuestionAssessment {
            question_id: "synthetic-question".to_owned(),
            outcome: outcome.to_owned(),
            created_at,
        }
    }

    fn test_probe(status: &str, created_at: OffsetDateTime) -> StudentLearningProbe {
        StudentLearningProbe {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Some(Uuid::new_v4()),
            assistant_message_id: Some(Uuid::new_v4()),
            answer_message_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "traffic".to_owned(),
            status: status.to_owned(),
            question_text: "test".to_owned(),
            reason_code: "context_unobserved".to_owned(),
            priority_score: 1.0,
            context_snapshot: json!({}),
            offered_at: Some(created_at),
            answered_at: None,
            expires_at: None,
            created_at,
            updated_at: created_at,
        }
    }

    fn test_message(
        conversation_id: Uuid,
        user_id: Uuid,
        role: &str,
        sequence: i64,
    ) -> repo::table::conversation_messages::ConversationMessage {
        repo::table::conversation_messages::ConversationMessage {
            id: Uuid::new_v4(),
            conversation_id,
            user_id,
            role: role.to_owned(),
            content_text: Some("test".to_owned()),
            content_json: None,
            status: "completed".to_owned(),
            sequence,
            token_count: None,
            metadata: json!({}),
            strategy_id: None,
            response_mode: None,
            routing_mode: None,
            collaboration_mode: None,
            learning_level: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn framework_and_classifier_configs_parse() {
        let framework = framework().expect("framework should parse");
        let classifier = context_classifier().expect("classifier should parse");
        let question_bank = question_bank().expect("question bank should parse");
        assert_eq!(framework.probe_policy.interaction_mode, "contextual_micro");
        assert_eq!(framework.probe_policy.reward_code, "progress_feedback");
        assert_eq!(framework.probe_policy.retention_review_interval_days, 7);
        assert_eq!(question_bank.framework_version, framework.version);
        assert!(classifier.domains.contains_key("traffic"));
    }

    #[test]
    fn framework_rejects_silent_probe_policy_fields() {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../../config/json/learning/ecommerce-knowledge-framework.v1.json"
        ))
        .expect("framework JSON");
        value["probePolicy"]["answeredCooldownHours"] = json!(72);

        assert!(serde_json::from_value::<Framework>(value).is_err());
    }

    #[test]
    fn question_selector_config_matches_the_current_bank() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");

        assert_eq!(selector.schema_version, QUESTION_SELECTOR_SCHEMA_VERSION);
        assert_eq!(selector.version, QUESTION_SELECTOR_VERSION);
        assert_eq!(selector.question_bank_version, bank.version);
        assert_eq!(
            selector
                .jurisdictions
                .keys()
                .cloned()
                .collect::<HashSet<_>>(),
            HashSet::from(["cn".to_owned(), "eu".to_owned(), "us".to_owned()])
        );
        assert_eq!(selector.topic_keywords.len(), 8);
        assert!(selector.fallback_suppression_keywords.is_empty());
        assert_eq!(
            selector
                .topic_question_ids
                .get("product")
                .and_then(|mappings| mappings.get("标题")),
            Some(&vec![
                "product.title_key_details.choice.013".to_owned(),
                "product.title_promotion_boundary.choice.014".to_owned(),
                "product.title_fact_consistency.choice.016".to_owned(),
                "product.title_variant_identity.choice.017".to_owned(),
                "product.title_claim_evidence.choice.018".to_owned(),
            ])
        );
        assert_eq!(
            selector
                .topic_question_ids
                .get("service")
                .and_then(|mappings| mappings.get("价格异议")),
            Some(&vec!["service.price_objection_needs.choice.011".to_owned()])
        );
        assert!(
            selector
                .topic_keywords
                .values()
                .all(|keywords| !keywords.is_empty())
        );
    }

    #[test]
    fn contextual_selector_keeps_china_personalization_out_of_eu_questions() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let selected = contextual_question_ids_for_content(
            "中国电商平台使用个性化推荐时，怎样保障消费者选择权？",
            "compliance",
            &bank,
            &selector,
        )
        .expect("fine-grained context should produce candidates");

        assert_eq!(
            selected,
            vec!["compliance.cn_personalized_search_option.choice.007".to_owned()]
        );
        assert!(selected.iter().all(|question_id| {
            question_applicability_scope(question_id, 1)
                .is_some_and(|scope| scope.jurisdiction_code == "cn")
        }));
    }

    #[test]
    fn contextual_selector_isolates_explicit_eu_and_us_jurisdictions() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let eu = contextual_question_ids_for_content(
            "欧盟线上购买的撤回权和退回商品规则是什么？",
            "compliance",
            &bank,
            &selector,
        )
        .expect("EU context should produce candidates");
        assert!(!eu.is_empty());
        assert!(eu.iter().all(|question_id| {
            question_applicability_scope(question_id, 1)
                .is_some_and(|scope| scope.jurisdiction_code == "eu")
        }));

        let us = contextual_question_ids_for_content(
            "美国 FTC 规则下无法按承诺时间发货时怎样处理？",
            "service",
            &bank,
            &selector,
        )
        .expect("US context should produce candidates");
        assert_eq!(us, vec!["service.ftc_shipping_delay.choice.003".to_owned()]);
    }

    #[test]
    fn contextual_selector_uses_platform_and_topic_together() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let selected = contextual_question_ids_for_content(
            "GA4 中用户开始结账时应该发送 begin_checkout 吗？",
            "conversion",
            &bank,
            &selector,
        )
        .expect("GA4 context should produce candidates");

        assert_eq!(
            selected,
            vec!["conversion.ga4_begin_checkout.choice.003".to_owned()]
        );
        assert!(selected.iter().all(|question_id| {
            question_applicability_scope(question_id, 1)
                .is_some_and(|scope| scope.platform_code == "google_analytics_4")
        }));
    }

    #[test]
    fn contextual_selector_prefers_the_google_trends_concept_discussed_in_the_turn() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");

        let relative_index = contextual_question_ids_for_turn(
            Some("Google Trends 里的 100 具体代表多少次搜索？能不能直接用这个数预测销量？"),
            "Google Trends 的数值是相对搜索热度，不能直接当作销量。",
            "market",
            &bank,
            &selector,
        )
        .expect("relative-index context should produce candidates");
        assert_eq!(
            relative_index,
            vec!["market.trends_relative_index.choice.001".to_owned()]
        );

        let zero_value = contextual_question_ids_for_turn(
            Some("Google Trends 图表中某个时点显示为 0，应该怎样解释？"),
            "零值不一定表示完全没有搜索。",
            "market",
            &bank,
            &selector,
        )
        .expect("zero-value context should produce candidates");
        assert_eq!(
            zero_value,
            vec!["market.trends_zero_value.choice.003".to_owned()]
        );
    }

    #[test]
    fn contextual_selector_prefers_student_signals_and_uses_assistant_only_as_fallback() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let student_selected = contextual_question_ids_for_turn(
            Some("中国电商平台的个性化推荐怎样保障选择权？"),
            "欧盟撤回权要求消费者退回商品。",
            "compliance",
            &bank,
            &selector,
        )
        .expect("student context should produce candidates");
        assert_eq!(
            student_selected,
            vec!["compliance.cn_personalized_search_option.choice.007".to_owned()]
        );

        let assistant_selected = contextual_question_ids_for_turn(
            Some("请继续说明。"),
            "GA4 中开始结账应发送 begin_checkout。",
            "conversion",
            &bank,
            &selector,
        )
        .expect("assistant context should provide the fallback candidates");
        assert_eq!(
            assistant_selected,
            vec!["conversion.ga4_begin_checkout.choice.003".to_owned()]
        );
    }

    #[test]
    fn contextual_selector_uses_mapped_domain_signals_instead_of_random_fallback() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");

        assert_eq!(
            contextual_question_ids_for_turn(
                Some("请从合规角度分析这个风险。"),
                "需要遵守相关合规规则。",
                "compliance",
                &bank,
                &selector,
            ),
            Some(vec![
                "compliance.cn_personal_info_minimum.choice.008".to_owned(),
                "compliance.cn_default_bundling.choice.009".to_owned(),
            ])
        );
    }

    #[test]
    fn contextual_selector_matches_title_and_selling_point_questions_without_inventory() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let selected = contextual_question_ids_for_turn(
            Some("帮我优化咖啡这个商品的标题和卖点"),
            "可以先梳理人群、场景和核心表达。",
            "product",
            &bank,
            &selector,
        )
        .expect("supported product intent should produce contextual candidates");

        assert_eq!(
            selected.iter().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                "product.title_key_details.choice.013".to_owned(),
                "product.title_promotion_boundary.choice.014".to_owned(),
                "product.highlight_verifiable.choice.015".to_owned(),
                "product.title_fact_consistency.choice.016".to_owned(),
                "product.title_variant_identity.choice.017".to_owned(),
                "product.title_claim_evidence.choice.018".to_owned(),
                "product.highlight_test_conditions.choice.019".to_owned(),
                "product.highlight_comparison_evidence.choice.020".to_owned(),
                "product.highlight_usage_evidence.choice.021".to_owned(),
            ])
        );
        assert!(!selected.iter().any(|question_id| {
            question_id == "product.stocktake_action.choice.010"
                || question_id == "product.reorder_point_inputs.choice.009"
        }));

        let title = contextual_question_ids_for_turn(
            Some("帮我把咖啡商品标题写清楚"),
            "标题需要与商品一致。",
            "product",
            &bank,
            &selector,
        )
        .expect("title intent should produce candidates");
        assert_eq!(
            title,
            vec![
                "product.title_key_details.choice.013".to_owned(),
                "product.title_promotion_boundary.choice.014".to_owned(),
                "product.title_fact_consistency.choice.016".to_owned(),
                "product.title_variant_identity.choice.017".to_owned(),
                "product.title_claim_evidence.choice.018".to_owned(),
            ]
        );

        let selling_point = contextual_question_ids_for_turn(
            Some("这款咖啡的卖点应该怎样提炼？"),
            "先梳理商品亮点。",
            "product",
            &bank,
            &selector,
        )
        .expect("selling-point intent should produce candidates");
        assert_eq!(
            selling_point,
            vec![
                "product.highlight_verifiable.choice.015".to_owned(),
                "product.highlight_test_conditions.choice.019".to_owned(),
                "product.highlight_comparison_evidence.choice.020".to_owned(),
                "product.highlight_usage_evidence.choice.021".to_owned(),
            ]
        );
    }

    #[test]
    fn every_chat_domain_keyword_has_a_non_empty_question_mapping() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let classifier = context_classifier().expect("context classifier should parse");

        for (domain_code, domain) in classifier.domains {
            for keyword in domain.keywords {
                let selected =
                    contextual_question_ids_for_content(&keyword, &domain_code, &bank, &selector)
                        .unwrap_or_else(|| {
                            panic!("missing selector mapping for {domain_code}:{keyword}")
                        });
                assert!(
                    !selected.is_empty(),
                    "empty selector mapping for {domain_code}:{keyword}"
                );
            }
        }
    }

    #[test]
    fn representative_chat_topics_retrieve_related_questions_across_all_domains() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let framework = framework().expect("framework should parse");
        let classifier = context_classifier().expect("context classifier should parse");
        let cases = [
            (
                "market",
                "怎样分析主要竞争对手？",
                "market.competitor_analysis_scope.choice.008",
            ),
            (
                "product",
                "畅销商品的补货点怎样设置？",
                "product.reorder_point_inputs.choice.009",
            ),
            (
                "content",
                "直播脚本怎样写得更有用？",
                "content.people_first_purpose.choice.007",
            ),
            (
                "traffic",
                "广告渠道怎样归因真实销售？",
                "traffic.promotion_sales_attribution.choice.009",
            ),
            (
                "conversion",
                "怎样衡量购物车放弃？",
                "conversion.cart_abandonment_measurement.choice.009",
            ),
            (
                "finance",
                "成本加成定价应怎样计算？",
                "finance.cost_plus_pricing.choice.009",
            ),
            (
                "service",
                "客户嫌贵时客服应怎样回应？",
                "service.price_objection_needs.choice.011",
            ),
            (
                "compliance",
                "个性化推荐怎样保障选择权？",
                "compliance.cn_personalized_search_option.choice.007",
            ),
        ];

        for (domain_code, content, expected_question_id) in cases {
            let (selected_domain, _, _) = select_turn_context_domain(
                Some(content),
                "",
                &HashMap::new(),
                &framework.domains,
                &classifier,
                OffsetDateTime::now_utc(),
                framework.probe_policy.retention_review_interval_days,
            )
            .expect("representative chat topic should select a domain");
            assert_eq!(
                selected_domain.code, domain_code,
                "{content} selected the wrong domain"
            );
            let selected = contextual_question_ids_for_content(
                content,
                &selected_domain.code,
                &bank,
                &selector,
            )
            .expect("representative chat topic should be indexed");
            assert!(
                selected
                    .iter()
                    .any(|question_id| question_id == expected_question_id),
                "{content} did not retrieve {expected_question_id}: {selected:?}"
            );
        }
    }

    #[test]
    fn conversational_aliases_retrieve_related_questions_across_all_domains() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let framework = framework().expect("framework should parse");
        let classifier = context_classifier().expect("context classifier should parse");
        let cases = [
            (
                "market",
                "想看看同行都在卖什么",
                "market.competitor_analysis_scope.choice.008",
            ),
            (
                "product",
                "不同款式怎么避免混成同一个 SKU",
                "product.variant_item_group.choice.001",
            ),
            (
                "content",
                "直播带货的宣传语怎么写",
                "content.people_first_purpose.choice.007",
            ),
            (
                "traffic",
                "几个渠道一起引流，怎么判断谁带来销售",
                "traffic.promotion_sales_attribution.choice.009",
            ),
            (
                "conversion",
                "用户老是弃购，该看哪个环节",
                "conversion.cart_abandonment_measurement.choice.009",
            ),
            (
                "finance",
                "广告投产比高就一定赚不赚钱吗",
                "finance.roas_not_profit.choice.005",
            ),
            (
                "service",
                "买家说到货慢并发起客诉怎么办",
                "service.cn_complaint_mechanism.choice.008",
            ),
            (
                "compliance",
                "增值服务默认勾选会有风险吗",
                "compliance.cn_default_bundling.choice.009",
            ),
        ];

        for (domain_code, content, expected_question_id) in cases {
            let (domain, _, _) = select_turn_context_domain(
                Some(content),
                "",
                &HashMap::new(),
                &framework.domains,
                &classifier,
                OffsetDateTime::now_utc(),
                framework.probe_policy.retention_review_interval_days,
            )
            .expect("alias should select a domain");
            assert_eq!(domain.code, domain_code, "wrong domain for {content}");
            let selected =
                contextual_question_ids_for_content(content, domain_code, &bank, &selector)
                    .expect("alias should produce candidates");
            assert!(
                selected.iter().any(|id| id == expected_question_id),
                "{content} did not retrieve {expected_question_id}: {selected:?}"
            );
        }
    }

    #[test]
    fn question_gap_signals_contain_only_controlled_selector_values() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let signals = question_gap_signals_for_turn(
            Some("我的私密店铺备注是 123，请分析欧盟和中国的默认勾选"),
            "",
            "compliance",
            &selector,
        );

        assert_eq!(signals.topic_signals, vec!["默认勾选"]);
        assert_eq!(signals.jurisdiction, "ambiguous");
        assert_eq!(signals.platform, "none");
    }

    #[test]
    fn unsupported_chat_topic_fails_closed_instead_of_selecting_a_domain_randomly() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");

        assert_eq!(
            contextual_question_ids_for_turn(
                Some("请继续说明。"),
                "这里暂时没有可核验的细分知识点。",
                "product",
                &bank,
                &selector,
            ),
            Some(Vec::new())
        );
    }

    #[test]
    fn active_structured_probe_is_hidden_when_current_turn_no_longer_supports_it() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let framework = framework().expect("framework should parse");
        let classifier = context_classifier().expect("classifier should parse");
        let mut probe = test_probe("offered", OffsetDateTime::now_utc());
        probe.knowledge_code = "product".to_owned();
        probe.context_snapshot = json!({
            "questionType": "single_choice",
            "questionId": "product.stocktake_action.choice.010"
        });

        assert!(!active_probe_matches_turn_context(
            &probe,
            Some("帮我优化咖啡这个商品的标题和卖点"),
            "可以先梳理人群、场景和核心表达。",
            &bank,
            &selector,
            &framework,
            &classifier,
        ));
        assert!(active_probe_matches_turn_context(
            &probe,
            Some("商品做库存盘点时，发现账实不一致应先怎么办？"),
            "先核对差异来源。",
            &bank,
            &selector,
            &framework,
            &classifier,
        ));

        probe.knowledge_code = "market".to_owned();
        probe.context_snapshot = json!({
            "questionType": "single_choice",
            "questionId": "market.trends_compare_context.choice.002"
        });
        assert!(!active_probe_matches_turn_context(
            &probe,
            Some("客户嫌贵时，客服怎样回应并追问需求，避免机械降价？"),
            "先接住价格异议，再追问场景和预算。",
            &bank,
            &selector,
            &framework,
            &classifier,
        ));

        probe.context_snapshot = json!({ "questionType": "open_response" });
        assert!(active_probe_matches_turn_context(
            &probe,
            Some("帮我优化咖啡这个商品的标题和卖点"),
            "可以先梳理人群、场景和核心表达。",
            &bank,
            &selector,
            &framework,
            &classifier,
        ));
    }

    #[test]
    fn contextual_selector_fails_closed_for_ambiguous_jurisdictions() {
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let selected = contextual_question_ids_for_content(
            "比较中国和欧盟的个性化推荐规则。",
            "compliance",
            &bank,
            &selector,
        )
        .expect("ambiguous fine-grained context should be explicit");

        assert!(selected.is_empty());
    }

    #[test]
    fn contextual_candidates_remain_primary_when_assessment_preferences_do_not_intersect() {
        let contextual = vec!["compliance.cn_personalized_search_option.choice.007".to_owned()];
        assert_eq!(
            merge_question_preferences(
                Some(contextual.clone()),
                vec!["compliance.eu_distance_withdrawal.choice.001".to_owned()],
            ),
            Some(contextual.clone())
        );
        assert_eq!(
            merge_question_preferences(Some(contextual.clone()), contextual.clone()),
            Some(contextual.clone())
        );
        assert_eq!(
            merge_question_preferences(None, contextual.clone()),
            Some(contextual)
        );
        assert!(merge_question_preferences(Some(Vec::new()), Vec::new()).is_none());
    }

    #[test]
    fn continuation_context_candidates_are_restored_and_malformed_snapshots_fail_closed() {
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        assert!(probe_contextual_question_ids(&probe).is_none());

        probe.context_snapshot = json!({
            "contextualQuestionIds": [
                "compliance.cn_personalized_search_option.choice.007",
                "compliance.cn_personal_info_minimum.choice.008"
            ]
        });
        assert_eq!(
            probe_contextual_question_ids(&probe),
            Some(vec![
                "compliance.cn_personalized_search_option.choice.007".to_owned(),
                "compliance.cn_personal_info_minimum.choice.008".to_owned(),
            ])
        );

        probe.context_snapshot = json!({ "contextualQuestionIds": "not-an-array" });
        assert_eq!(probe_contextual_question_ids(&probe), Some(Vec::new()));
        probe.context_snapshot = json!({ "contextualQuestionIds": ["valid", 7] });
        assert_eq!(probe_contextual_question_ids(&probe), Some(Vec::new()));
    }

    #[test]
    fn context_domain_requires_a_match_and_keeps_demonstrated_domains_eligible() {
        let classifier = context_classifier().expect("classifier should parse");
        let domains = vec![domain("traffic", 1.0), domain("conversion", 1.0)];
        let now = OffsetDateTime::now_utc();
        let latest_question_assessments = HashMap::from([(
            "traffic".to_owned(),
            HashMap::from([
                ("traffic.q1".to_owned(), assessment("correct", now)),
                ("traffic.q2".to_owned(), assessment("partial", now)),
            ]),
        )]);
        let selected = select_context_domain(
            "广告点击和流量都上涨，但付款转化没有变化",
            &latest_question_assessments,
            &domains,
            &classifier,
            now,
            7,
        )
        .expect("a demonstrated domain should remain eligible for light reinforcement");
        assert_eq!(selected.0.code, "traffic");
        assert!(selected.1 > 0);

        assert!(
            select_context_domain(
                "今天天气不错",
                &HashMap::new(),
                &domains,
                &classifier,
                now,
                7,
            )
            .is_none()
        );
    }

    #[test]
    fn context_domain_matches_ga4_checkout_turn_as_conversion() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let selected = select_context_domain(
            "某商品详情页访问量正常，但很多用户进入结账流程后没有购买。请告诉我应该检查哪些 GA4 电商事件和转化环节。",
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("GA4 checkout content should select a diagnostic domain");

        assert_eq!(selected.0.code, "conversion");
        assert!(selected.1 >= 3);
    }

    #[test]
    fn google_trends_follow_up_stays_in_market_domain() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let selected = select_turn_context_domain(
            Some("Google Trends 里的 100 具体代表多少次搜索？能不能直接用这个数预测销量？"),
            "Google Trends 的数值是相对搜索热度，不能直接当作销量。",
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("Google Trends follow-up should remain eligible for a market probe");

        assert_eq!(selected.0.code, "market");
        assert!(selected.1 >= 1);
    }

    #[test]
    fn context_domain_matches_personalized_recommendation_turn_as_compliance() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let selected = select_context_domain(
            "使用个性化推荐会处理个人信息和算法推荐偏好，因此要提供隐私设置和便捷关闭方式，尊重用户选择权；即使可能影响转化率和下单，也必须守住合规底线。",
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("personalized recommendation content should select a diagnostic domain");

        assert_eq!(selected.0.code, "compliance");
        assert!(selected.1 >= 6);
    }

    #[test]
    fn turn_context_prefers_student_service_question_over_assistant_conversion_terms() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let now = OffsetDateTime::now_utc();
        let assistant_content =
            "消费者下单和购买时可能影响转化率；从客服售后纠纷处理看，还要说明押金退还程序。";
        let assistant_only = select_context_domain(
            assistant_content,
            &HashMap::new(),
            &framework.domains,
            &classifier,
            now,
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("assistant content should expose the former conflict");
        assert_eq!(assistant_only.0.code, "conversion");

        let selected = select_turn_context_domain(
            Some("请从客服售后角度说明押金退还方式、程序和不合理限制。"),
            assistant_content,
            &HashMap::new(),
            &framework.domains,
            &classifier,
            now,
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("student service question should select a diagnostic domain");

        assert_eq!(selected.0.code, "service");
        assert!(selected.1 >= 2);
    }

    #[test]
    fn price_objection_turn_prefers_service_and_selects_source_backed_question() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let student_content =
            "客户说咖啡太贵时，客服应先怎样回应，怎样追问需求，才能避免只会机械降价？";
        let assistant_content = "先接住客户的价格异议，再追问使用场景和预算，不要直接机械降价。";

        let selected_domain = select_turn_context_domain(
            Some(student_content),
            assistant_content,
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("a price objection should select the customer service domain");
        assert_eq!(selected_domain.0.code, "service");
        assert!(selected_domain.1 >= 4);

        assert_eq!(
            contextual_question_ids_for_turn(
                Some(student_content),
                assistant_content,
                "service",
                &bank,
                &selector,
            ),
            Some(vec!["service.price_objection_needs.choice.011".to_owned()]),
            "a price objection should select the explicitly mapped source-backed question"
        );
    }

    #[test]
    fn refund_dispute_turn_prefers_service_evidence_question_over_product_context() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let bank = question_bank().expect("question bank should parse");
        let selector = question_selector(&bank).expect("question selector should parse");
        let student_content = "客户收到商品后反馈与描述不符并要求退款，我应该先核对哪些证据、怎样判断责任并制定客服处理步骤？";
        let assistant_content =
            "先核对订单、商品描述、沟通和履约记录，再判断争议责任并给出处理步骤。";

        let selected_domain = select_turn_context_domain(
            Some(student_content),
            assistant_content,
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("a refund evidence dispute should select the customer service domain");
        assert_eq!(selected_domain.0.code, "service");

        let candidates = contextual_question_ids_for_turn(
            Some(student_content),
            assistant_content,
            "service",
            &bank,
            &selector,
        )
        .expect("refund evidence and responsibility signals should select related questions");
        assert_eq!(
            candidates,
            vec![
                "service.cn_return_reasonable_inspection.choice.007".to_owned(),
                "service.cn_complaint_mechanism.choice.008".to_owned(),
                "service.cn_dispute_records.choice.010".to_owned(),
            ]
        );
    }

    #[test]
    fn turn_context_falls_back_to_assistant_when_student_text_has_no_domain_signal() {
        let classifier = context_classifier().expect("classifier should parse");
        let framework = framework().expect("framework should parse");
        let selected = select_turn_context_domain(
            Some("请继续说明。"),
            "用户进入结账流程后没有购买，需要检查 GA4 转化漏斗。",
            &HashMap::new(),
            &framework.domains,
            &classifier,
            OffsetDateTime::now_utc(),
            framework.probe_policy.retention_review_interval_days,
        )
        .expect("assistant content should provide the fallback domain");

        assert_eq!(selected.0.code, "conversion");
    }

    #[test]
    fn direct_preceding_user_content_requires_same_turn_identity() {
        let conversation_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut user_message = test_message(conversation_id, user_id, "user", 4);
        user_message.content_text = Some("  请分析客服售后。  ".to_owned());
        let assistant_message = test_message(conversation_id, user_id, "assistant", 5);

        assert_eq!(
            direct_preceding_user_content(&[user_message.clone()], &assistant_message),
            Some("请分析客服售后。")
        );

        user_message.sequence = 3;
        assert!(direct_preceding_user_content(&[user_message], &assistant_message).is_none());

        let mut foreign_user_message = test_message(conversation_id, Uuid::new_v4(), "user", 4);
        foreign_user_message.content_text = Some("客服售后".to_owned());
        assert!(
            direct_preceding_user_content(&[foreign_user_message], &assistant_message).is_none()
        );
    }

    #[test]
    fn observed_domain_remains_eligible_until_assessment_is_demonstrated() {
        let classifier = context_classifier().expect("classifier should parse");
        let domains = vec![domain("traffic", 1.0)];
        let now = OffsetDateTime::now_utc();

        let selected = select_context_domain(
            "广告流量和点击成本需要怎么分析",
            &HashMap::new(),
            &domains,
            &classifier,
            now,
            7,
        )
        .expect("chat observation must not suppress the diagnostic");

        assert_eq!(selected.0.code, "traffic");
    }

    #[test]
    fn assessment_need_requires_positive_multi_angle_evidence() {
        let now = OffsetDateTime::now_utc();
        assert_eq!(
            domain_assessment_need(None, now, 7),
            Some(DomainAssessmentNeed::InitialOrCorrection)
        );
        assert_eq!(
            domain_assessment_need(
                Some(&HashMap::from([(
                    "q1".to_owned(),
                    assessment("correct", now),
                )])),
                now,
                7,
            ),
            Some(DomainAssessmentNeed::InitialOrCorrection)
        );
        assert_eq!(
            domain_assessment_need(
                Some(&HashMap::from([
                    ("q1".to_owned(), assessment("partial", now)),
                    ("q2".to_owned(), assessment("partial", now)),
                ])),
                now,
                7,
            ),
            Some(DomainAssessmentNeed::InitialOrCorrection)
        );
        assert_eq!(
            domain_assessment_need(
                Some(&HashMap::from([
                    ("q1".to_owned(), assessment("correct", now)),
                    ("q2".to_owned(), assessment("incorrect", now)),
                ])),
                now,
                7,
            ),
            Some(DomainAssessmentNeed::InitialOrCorrection)
        );
        assert_eq!(
            domain_assessment_need(
                Some(&HashMap::from([
                    ("q1".to_owned(), assessment("correct", now)),
                    ("q2".to_owned(), assessment("partial", now)),
                ])),
                now,
                7,
            ),
            Some(DomainAssessmentNeed::OngoingReinforcement)
        );
    }

    #[test]
    fn demonstrated_domain_marks_spacing_and_continues_light_reinforcement() {
        let now = OffsetDateTime::now_utc();
        let eight_days_ago = now - Duration::days(8);
        let same_old_session = HashMap::from([
            ("q1".to_owned(), assessment("correct", eight_days_ago)),
            ("q2".to_owned(), assessment("partial", eight_days_ago)),
        ]);
        assert_eq!(
            domain_assessment_need(Some(&same_old_session), now, 7),
            Some(DomainAssessmentNeed::SpacedRetentionReview)
        );

        let retained = HashMap::from([
            ("q1".to_owned(), assessment("correct", eight_days_ago)),
            ("q2".to_owned(), assessment("partial", now)),
        ]);
        assert_eq!(
            domain_assessment_need(Some(&retained), now, 7),
            Some(DomainAssessmentNeed::OngoingReinforcement)
        );
        assert!(domain_has_retained_evidence(&retained, 7));
    }

    #[test]
    fn question_priority_fills_new_angles_then_returns_to_incorrect_answers() {
        let bank = question_bank().expect("question bank should parse");
        let cart_question = bank
            .questions
            .iter()
            .find(|question| question.id == "traffic.ga4_add_to_cart_event.choice.002")
            .expect("cart question");
        let cart_angle = assessment_angle_code(&cart_question.id, cart_question.version)
            .expect("cart assessment angle");
        let one_attempt = HashMap::from([(
            cart_angle.to_owned(),
            LatestQuestionAssessment {
                question_id: cart_question.id.clone(),
                outcome: "incorrect".to_owned(),
                created_at: OffsetDateTime::now_utc(),
            },
        )]);
        let new_angles = preferred_question_ids_for_domain(&bank, "traffic", Some(&one_attempt));
        assert!(!new_angles.contains(&cart_question.id));
        assert!(!new_angles.contains(&"traffic.ga4_view_cart.choice.006".to_owned()));
        assert!(new_angles.iter().all(|question_id| {
            bank.questions
                .iter()
                .find(|question| question.id == *question_id)
                .and_then(|question| assessment_angle_code(&question.id, question.version))
                != Some(cart_angle)
        }));

        let list_question = bank
            .questions
            .iter()
            .find(|question| question.id == "traffic.ga4_view_item_list.choice.003")
            .expect("list question");
        let list_angle = assessment_angle_code(&list_question.id, list_question.version)
            .expect("list assessment angle");
        let two_attempts = HashMap::from([
            (cart_angle.to_owned(), one_attempt[cart_angle].clone()),
            (
                list_angle.to_owned(),
                LatestQuestionAssessment {
                    question_id: list_question.id.clone(),
                    outcome: "correct".to_owned(),
                    created_at: OffsetDateTime::now_utc(),
                },
            ),
        ]);
        let correction = preferred_question_ids_for_domain(&bank, "traffic", Some(&two_attempts));
        assert_eq!(correction, vec![cart_question.id.clone()]);
    }

    #[test]
    fn continuation_depth_defaults_to_zero_and_reads_snapshot() {
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        assert_eq!(probe_continuation_depth(&probe), 0);
        probe.context_snapshot = json!({ "continuationDepth": 2 });
        assert_eq!(probe_continuation_depth(&probe), 2);
    }

    #[test]
    fn continuation_policy_allows_voluntary_practice_without_a_fixed_depth_limit() {
        let max_continuation_depth = framework()
            .expect("learning framework should load")
            .probe_policy
            .max_continuation_depth;
        assert_eq!(max_continuation_depth, 0);

        let mut initial_probe = test_probe("answered", OffsetDateTime::now_utc());
        assert!(probe_can_continue(&initial_probe, max_continuation_depth));
        assert_eq!(
            next_probe_continuation_depth(&initial_probe, max_continuation_depth),
            Some(1)
        );

        initial_probe.context_snapshot = json!({ "continuationDepth": 10_000 });
        assert!(probe_can_continue(&initial_probe, max_continuation_depth));
        assert_eq!(
            next_probe_continuation_depth(&initial_probe, max_continuation_depth),
            Some(10_001)
        );
    }

    #[test]
    fn continuation_restarts_only_the_frozen_contextual_cycle() {
        let preferred = vec!["service.return".to_owned(), "service.dispute".to_owned()];
        let (remaining, history, restarted) = continuation_candidates_for_next_cycle(
            preferred.clone(),
            &["service.return".to_owned()],
        );
        assert_eq!(remaining, vec!["service.dispute"]);
        assert_eq!(history, vec!["service.return"]);
        assert!(!restarted);

        let (remaining, history, restarted) =
            continuation_candidates_for_next_cycle(preferred.clone(), &preferred);
        assert_eq!(remaining, preferred);
        assert!(history.is_empty());
        assert!(restarted);
    }

    #[test]
    fn continuation_completes_the_frozen_contextual_pool_before_correction_review() {
        let contextual = vec![
            "product.title_key_details.choice.013".to_owned(),
            "product.title_promotion_boundary.choice.014".to_owned(),
            "product.highlight_verifiable.choice.015".to_owned(),
        ];
        let correction_only = vec!["product.title_promotion_boundary.choice.014".to_owned()];

        assert_eq!(
            question_pool_for_request(false, Some(contextual.clone()), correction_only.clone()),
            Some(correction_only)
        );
        let continuation_pool = question_pool_for_request(
            true,
            Some(contextual.clone()),
            vec!["product.title_promotion_boundary.choice.014".to_owned()],
        )
        .expect("continuation should preserve the frozen contextual pool");
        let (remaining, _, restarted) = continuation_candidates_for_next_cycle(
            continuation_pool,
            &[
                "product.title_key_details.choice.013".to_owned(),
                "product.title_promotion_boundary.choice.014".to_owned(),
            ],
        );
        assert_eq!(remaining, vec!["product.highlight_verifiable.choice.015"]);
        assert!(!restarted);
        assert_eq!(
            question_pool_for_request(true, Some(Vec::new()), Vec::new()),
            None
        );
    }

    #[test]
    fn review_cycle_prioritizes_incorrect_then_partial_then_unassessed_questions() {
        let bank = question_bank().expect("question bank should parse");
        let question_ids = [
            "product.stocktake_action.choice.010",
            "product.reorder_point_inputs.choice.009",
            "product.highlight_verifiable.choice.015",
            "product.feed_landing_price.choice.003",
        ];
        let angle_for = |question_id: &str| {
            let question = bank
                .questions
                .iter()
                .find(|question| question.id == question_id)
                .expect("test question should exist");
            assessment_angle_code(&question.id, question.version)
                .expect("test question should have an assessment angle")
                .to_owned()
        };
        let now = OffsetDateTime::now_utc();
        let latest = HashMap::from([
            (
                angle_for(question_ids[0]),
                LatestQuestionAssessment {
                    question_id: question_ids[0].to_owned(),
                    outcome: "correct".to_owned(),
                    created_at: now,
                },
            ),
            (
                angle_for(question_ids[1]),
                LatestQuestionAssessment {
                    question_id: question_ids[1].to_owned(),
                    outcome: "incorrect".to_owned(),
                    created_at: now,
                },
            ),
            (
                angle_for(question_ids[3]),
                LatestQuestionAssessment {
                    question_id: question_ids[3].to_owned(),
                    outcome: "partial".to_owned(),
                    created_at: now,
                },
            ),
        ]);

        assert_eq!(
            prioritize_review_candidates(
                question_ids.iter().map(|id| (*id).to_owned()).collect(),
                Some(&latest),
                &bank,
            ),
            vec![question_ids[1].to_owned()]
        );
        assert_eq!(
            prioritize_review_candidates(
                vec![
                    question_ids[0].to_owned(),
                    question_ids[2].to_owned(),
                    question_ids[3].to_owned(),
                ],
                Some(&latest),
                &bank,
            ),
            vec![question_ids[3].to_owned()]
        );
        assert_eq!(
            prioritize_review_candidates(
                vec![question_ids[0].to_owned(), question_ids[2].to_owned()],
                Some(&latest),
                &bank,
            ),
            vec![question_ids[2].to_owned()]
        );
    }

    #[test]
    fn practice_cycle_and_purpose_distinguish_coverage_from_review() {
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        assert_eq!(probe_practice_cycle(&probe), 0);
        probe.context_snapshot = json!({ "practiceCycle": 2 });
        assert_eq!(probe_practice_cycle(&probe), 2);

        let latest = HashMap::from([(
            "product.title".to_owned(),
            assessment("partial", OffsetDateTime::now_utc()),
        )]);
        assert_eq!(
            practice_purpose_for_question(0, "product.title", Some(&latest)),
            PracticePurpose::InitialCoverage
        );
        assert_eq!(
            practice_purpose_for_question(1, "product.title", Some(&latest)),
            PracticePurpose::CorrectionReview
        );
        assert_eq!(
            practice_purpose_for_question(1, "product.other", Some(&latest)),
            PracticePurpose::ReinforcementReview
        );
    }

    #[test]
    fn continuation_question_history_is_deduplicated_and_supports_legacy_snapshots() {
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        probe.context_snapshot = json!({ "questionId": "product.title.choice.001" });
        assert_eq!(
            probe_continuation_question_ids(&probe),
            vec!["product.title.choice.001"]
        );

        probe.context_snapshot = json!({
            "questionId": "product.title.choice.003",
            "continuationQuestionIds": [
                "product.title.choice.002",
                "product.title.choice.001",
                "product.title.choice.002"
            ]
        });
        assert_eq!(
            probe_continuation_question_ids(&probe),
            vec![
                "product.title.choice.001",
                "product.title.choice.002",
                "product.title.choice.003"
            ]
        );
    }

    #[test]
    fn only_the_direct_assistant_reply_suppresses_automatic_next_question() {
        let conversation_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let answer = test_message(conversation_id, user_id, "user", 4);
        let direct_reply = test_message(conversation_id, user_id, "assistant", 5);
        let later_reply = test_message(conversation_id, user_id, "assistant", 7);
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        probe.user_id = user_id;
        probe.conversation_id = Some(conversation_id);
        probe.answer_message_id = Some(answer.id);

        assert!(is_direct_probe_answer_reply(&probe, &answer, &direct_reply));
        assert!(!is_direct_probe_answer_reply(&probe, &answer, &later_reply));
    }

    #[test]
    fn structured_answer_restores_only_on_its_original_assistant_turn() {
        let conversation_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let original_assistant = test_message(conversation_id, user_id, "assistant", 3);
        let later_assistant = test_message(conversation_id, user_id, "assistant", 5);
        let mut probe = test_probe("answered", OffsetDateTime::now_utc());
        probe.user_id = user_id;
        probe.conversation_id = Some(conversation_id);
        probe.assistant_message_id = Some(original_assistant.id);
        probe.context_snapshot = json!({ "answerMode": "structured_choice" });

        assert!(is_structured_probe_answer_for_assistant(
            &probe,
            &original_assistant
        ));
        assert!(existing_turn_probe_response(&probe, &original_assistant).is_some());
        assert!(!is_structured_probe_answer_for_assistant(
            &probe,
            &later_assistant
        ));

        probe.status = "offered".to_owned();
        assert!(!is_structured_probe_answer_for_assistant(
            &probe,
            &original_assistant
        ));
        assert!(existing_turn_probe_response(&probe, &original_assistant).is_none());

        probe.status = "skipped".to_owned();
        assert!(existing_turn_probe_response(&probe, &original_assistant).is_none());
    }

    #[test]
    fn assessment_summary_separates_observation_from_rubric_results() {
        let summary = summarize_mastery_assessments(&json!({
            "market": {
                "status": "observed",
                "evidenceCount": 3,
                "strength": 0.7
            },
            "conversion": {
                "assessedCount": 2,
                "correctCount": 1,
                "partialCount": 1,
                "evaluationStatus": "developing"
            },
            "finance": {
                "assessedCount": 1,
                "incorrectCount": 1,
                "evaluationStatus": "needs_attention"
            }
        }));

        assert_eq!(summary.assessed_knowledge_count, 2);
        assert_eq!(summary.assessment_count, 3);
        assert_eq!(summary.correct_count, 1);
        assert_eq!(summary.partial_count, 1);
        assert_eq!(summary.incorrect_count, 1);
        assert_eq!(summary.needs_attention_knowledge_count, 1);
    }

    #[test]
    fn assessment_summary_uses_evidence_for_legacy_profiles() {
        let summary = summarize_assessment_outcomes_newest_first([
            ("conversion", "correct", Some("conversion.q1")),
            ("conversion", "incorrect", Some("conversion.q2")),
            ("traffic", "partial", Some("traffic.q1")),
            ("market", "unscored", Some("market.q1")),
        ]);

        assert_eq!(summary.assessed_knowledge_count, 2);
        assert_eq!(summary.assessment_count, 3);
        assert_eq!(summary.correct_count, 1);
        assert_eq!(summary.partial_count, 1);
        assert_eq!(summary.incorrect_count, 1);
        assert_eq!(summary.needs_attention_knowledge_count, 0);
        assert_eq!(summary.distinct_assessment_question_count, 3);
        assert_eq!(summary.multi_angle_knowledge_count, 1);
        assert_eq!(summary.demonstrated_knowledge_count, 0);
    }

    #[test]
    fn multi_angle_summary_ignores_legacy_and_unscorable_questions() {
        let summary = summarize_assessment_outcomes_newest_first([
            ("product", "correct", Some("product.q1")),
            ("product", "partial", Some("product.q1")),
            ("product", "incorrect", Some("product.q2")),
            ("service", "legacy_unscored", Some("service.q1")),
            ("finance", "correct", None),
        ]);

        assert_eq!(summary.assessment_count, 4);
        assert_eq!(summary.assessed_knowledge_count, 2);
        assert_eq!(summary.distinct_assessment_question_count, 2);
        assert_eq!(summary.multi_angle_knowledge_count, 1);
        assert_eq!(summary.demonstrated_knowledge_count, 0);
    }

    #[test]
    fn same_curriculum_angle_does_not_inflate_multi_angle_mastery() {
        let summary = summarize_assessment_outcomes_with_angles_newest_first([
            (
                "content",
                "correct",
                Some("content.informative_image_alt.choice.001"),
                "content.informative_image_accessibility",
            ),
            (
                "content",
                "correct",
                Some("content.complex_chart_description.choice.005"),
                "content.informative_image_accessibility",
            ),
        ]);

        assert_eq!(summary.distinct_assessment_question_count, 2);
        assert_eq!(summary.multi_angle_knowledge_count, 0);
        assert_eq!(summary.demonstrated_knowledge_count, 0);
    }

    #[test]
    fn historical_evidence_without_angle_code_uses_the_versioned_catalog() {
        let entry = repo::entity::student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            source_message_id: None,
            probe_id: Some(Uuid::new_v4()),
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "content".to_owned(),
            evidence_type: "probe_answer".to_owned(),
            direction: "positive".to_owned(),
            strength: 1.0,
            confidence: 1.0,
            summary: "historical structured evidence".to_owned(),
            rubric_hits: json!({
                "questionId": "content.complex_chart_description.choice.005",
                "questionVersion": 1,
                "outcome": "correct"
            }),
            analyzer_version: "deterministic_question_bank_v1".to_owned(),
            created_at: OffsetDateTime::now_utc(),
        };

        let (_, _, _, angle_code) =
            evidence_assessment_outcome(&entry).expect("assessment evidence");

        assert_eq!(angle_code, "content.informative_image_accessibility");
    }

    #[test]
    fn unmapped_historical_questions_collapse_to_one_conservative_angle() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        let first = repo::entity::student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id,
            conversation_id,
            source_message_id: None,
            probe_id: Some(Uuid::new_v4()),
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "market".to_owned(),
            evidence_type: "probe_answer".to_owned(),
            direction: "positive".to_owned(),
            strength: 1.0,
            confidence: 1.0,
            summary: "unmapped historical evidence".to_owned(),
            rubric_hits: json!({
                "questionId": "market.legacy_unknown.001",
                "outcome": "correct"
            }),
            analyzer_version: "legacy".to_owned(),
            created_at,
        };
        let mut second = first.clone();
        second.id = Uuid::new_v4();
        second.probe_id = Some(Uuid::new_v4());
        second.rubric_hits = json!({
            "questionId": "market.legacy_unknown.002",
            "outcome": "correct"
        });

        let summary = summarize_assessments(&json!({}), &[first, second], 7);

        assert_eq!(summary.distinct_assessment_question_count, 2);
        assert_eq!(summary.multi_angle_knowledge_count, 0);
        assert_eq!(summary.demonstrated_knowledge_count, 0);
    }

    #[test]
    fn demonstrated_summary_uses_the_latest_result_for_each_question() {
        let summary = summarize_assessment_outcomes_newest_first([
            ("content", "correct", Some("content.q1")),
            ("content", "partial", Some("content.q2")),
            ("content", "incorrect", Some("content.q1")),
            ("traffic", "partial", Some("traffic.q1")),
            ("traffic", "partial", Some("traffic.q2")),
            ("conversion", "incorrect", Some("conversion.q1")),
            ("conversion", "correct", Some("conversion.q2")),
            ("conversion", "correct", Some("conversion.q1")),
        ]);

        assert_eq!(summary.multi_angle_knowledge_count, 3);
        assert_eq!(summary.demonstrated_knowledge_count, 1);
    }

    #[test]
    fn retention_summary_requires_positive_latest_results_across_the_interval() {
        let now = OffsetDateTime::now_utc();
        let eight_days_ago = now - Duration::days(8);
        let retained_count = retained_knowledge_count_newest_first(
            [
                (
                    "content",
                    "partial",
                    Some("content.q2"),
                    "content.angle2".to_owned(),
                    now,
                ),
                (
                    "content",
                    "correct",
                    Some("content.q1"),
                    "content.angle1".to_owned(),
                    eight_days_ago,
                ),
                (
                    "traffic",
                    "correct",
                    Some("traffic.q1"),
                    "traffic.angle1".to_owned(),
                    now,
                ),
                (
                    "traffic",
                    "partial",
                    Some("traffic.q2"),
                    "traffic.angle2".to_owned(),
                    now,
                ),
                (
                    "finance",
                    "incorrect",
                    Some("finance.q1"),
                    "finance.angle1".to_owned(),
                    now,
                ),
                (
                    "finance",
                    "correct",
                    Some("finance.q2"),
                    "finance.angle2".to_owned(),
                    eight_days_ago,
                ),
            ],
            7,
        );

        assert_eq!(retained_count, 1);
    }

    #[test]
    fn assessment_timeline_tracks_structured_status_without_chat_observations() {
        let now = OffsetDateTime::parse("2026-08-30T12:00:00Z", &Rfc3339).unwrap();
        let evidence = vec![
            timeline_evidence(
                "incorrect",
                Some("conversion.ga4_payment_info.choice.004"),
                Some("conversion.checkout_event_measurement"),
                "2026-08-25T09:00:00Z",
            ),
            timeline_evidence(
                "partial",
                Some("conversion.ga4_payment_info.choice.004"),
                Some("conversion.checkout_event_measurement"),
                "2026-08-18T09:00:00Z",
            ),
            timeline_evidence(
                "correct",
                Some("conversion.checkout_dropoff.choice.001"),
                Some("conversion.checkout_funnel"),
                "2026-08-10T09:00:00Z",
            ),
            timeline_evidence("observed_unscored", None, None, "2026-08-01T09:00:00Z"),
        ];

        let timeline = build_assessment_timeline(&evidence, 7, now);

        assert_eq!(timeline.len(), 13);
        assert_eq!(timeline.first().unwrap().period_start, "2026-06-01");
        assert_eq!(timeline.last().unwrap().period_start, "2026-08-24");

        let initial = timeline
            .iter()
            .find(|point| point.period_start == "2026-08-10")
            .unwrap();
        assert_eq!(initial.assessed_knowledge_count, 1);
        assert_eq!(initial.demonstrated_knowledge_count, 0);
        assert_eq!(initial.retained_knowledge_count, 0);

        let retained = timeline
            .iter()
            .find(|point| point.period_start == "2026-08-17")
            .unwrap();
        assert_eq!(retained.assessed_knowledge_count, 1);
        assert_eq!(retained.demonstrated_knowledge_count, 1);
        assert_eq!(retained.retained_knowledge_count, 1);

        let corrected_back = timeline.last().unwrap();
        assert_eq!(corrected_back.assessed_knowledge_count, 1);
        assert_eq!(corrected_back.demonstrated_knowledge_count, 0);
        assert_eq!(corrected_back.retained_knowledge_count, 0);
    }

    #[test]
    fn assessment_timeline_serializes_stable_camel_case_fields() {
        let value = serde_json::to_value(LearningAssessmentTimelinePoint {
            period_start: "2026-08-24".to_owned(),
            assessed_knowledge_count: 3,
            demonstrated_knowledge_count: 2,
            retained_knowledge_count: 1,
        })
        .unwrap();

        assert_eq!(value["periodStart"], "2026-08-24");
        assert_eq!(value["assessedKnowledgeCount"], 3);
        assert_eq!(value["demonstratedKnowledgeCount"], 2);
        assert_eq!(value["retainedKnowledgeCount"], 1);
        assert!(value.get("correctChoiceId").is_none());
    }

    #[test]
    fn domain_assessment_explains_latest_error_without_counting_chat_as_mastery() {
        let mut observation =
            timeline_evidence("observed_unscored", None, None, "2026-09-08T07:00:00Z");
        observation.source_message_id = Some(Uuid::new_v4());
        let first = timeline_evidence(
            "correct",
            Some("conversion.question.001"),
            Some("conversion.angle.001"),
            "2026-09-01T08:00:00Z",
        );
        let mut latest = timeline_evidence(
            "incorrect",
            Some("conversion.question.002"),
            Some("conversion.angle.002"),
            "2026-09-08T08:00:00Z",
        );
        latest.rubric_hits["questionText"] = json!("Which checkout event is correct?");
        latest.rubric_hits["selectedChoiceText"] = json!("purchase");

        let result = build_domain_assessments(
            &json!({ "conversion": { "status": "observed" } }),
            &[latest.clone(), observation, first],
            &[domain("conversion", 1.0)],
            7,
        );
        let assessment = &result[0];

        assert_eq!(assessment.status_code, "needs_attention");
        assert_eq!(assessment.reason_code, "latest_answer_incorrect");
        assert_eq!(assessment.observation_count, 1);
        assert_eq!(assessment.assessment_count, 2);
        assert_eq!(assessment.distinct_question_count, 2);
        assert_eq!(assessment.assessment_angle_count, 2);
        assert_eq!(assessment.correct_count, 1);
        assert_eq!(assessment.incorrect_count, 1);
        assert_eq!(assessment.latest_outcome.as_deref(), Some("incorrect"));
        assert_eq!(
            assessment.recent_evidence[0].evidence_id,
            latest.id.to_string()
        );
        assert_eq!(
            assessment.recent_evidence[0].question_text.as_deref(),
            Some("Which checkout event is correct?")
        );
        assert!(
            assessment
                .missing_evidence_codes
                .contains(&"positive_correction".to_owned())
        );
        assert_eq!(assessment.next_action_code, "correct_latest_gap");
    }

    #[test]
    fn domain_assessment_distinguishes_observation_demonstration_and_retention() {
        let observation =
            timeline_evidence("observed_unscored", None, None, "2026-09-08T07:00:00Z");
        let observed_only = build_domain_assessments(
            &json!({ "conversion": { "status": "observed" } }),
            std::slice::from_ref(&observation),
            &[domain("conversion", 1.0)],
            7,
        );
        assert_eq!(observed_only[0].status_code, "not_assessed");
        assert_eq!(observed_only[0].assessment_count, 0);

        let old = timeline_evidence(
            "correct",
            Some("conversion.question.001"),
            Some("conversion.angle.001"),
            "2026-08-31T08:00:00Z",
        );
        let recent = timeline_evidence(
            "partial",
            Some("conversion.question.002"),
            Some("conversion.angle.002"),
            "2026-09-08T08:00:00Z",
        );
        let retained =
            build_domain_assessments(&json!({}), &[recent, old], &[domain("conversion", 1.0)], 7);
        assert_eq!(retained[0].status_code, "retained");
        assert_eq!(retained[0].reason_code, "spaced_positive_evidence");
        assert!(retained[0].retention_verified);
        assert!(retained[0].missing_evidence_codes.is_empty());
    }

    fn timeline_evidence(
        outcome: &str,
        question_id: Option<&str>,
        assessment_angle_code: Option<&str>,
        created_at: &str,
    ) -> repo::entity::student_learning_evidence::Model {
        repo::entity::student_learning_evidence::Model {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            source_message_id: None,
            probe_id: question_id.map(|_| Uuid::new_v4()),
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "conversion".to_owned(),
            evidence_type: if question_id.is_some() {
                "probe_answer".to_owned()
            } else {
                "explanation".to_owned()
            },
            direction: if matches!(outcome, "correct" | "partial") {
                "positive".to_owned()
            } else if outcome == "incorrect" {
                "negative".to_owned()
            } else {
                "neutral".to_owned()
            },
            strength: 1.0,
            confidence: 1.0,
            summary: "timeline evidence".to_owned(),
            rubric_hits: json!({
                "outcome": outcome,
                "questionId": question_id,
                "assessmentAngleCode": assessment_angle_code
            }),
            analyzer_version: "timeline_test".to_owned(),
            created_at: OffsetDateTime::parse(created_at, &Rfc3339).unwrap(),
        }
    }
}
