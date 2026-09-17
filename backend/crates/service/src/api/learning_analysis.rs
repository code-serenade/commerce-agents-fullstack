use std::collections::HashMap;

use db_core::{
    DbContext,
    error::{BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::{
    student_learning_evidence::{CreateStudentLearningEvidence, StudentLearningEvidenceService},
    student_learning_probes::{
        MarkStudentLearningProbeAnswered, StudentLearningProbe, StudentLearningProbeService,
    },
    student_learning_profiles::{
        CreateStudentLearningProfile, StudentLearningProfile, StudentLearningProfileService,
        UpdateStudentLearningAnalysis,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::api::learning_question_bank::{evaluate_answer, question_for_probe};

pub struct LearningAnalysisApi {
    profile_svc: StudentLearningProfileService,
    probe_svc: StudentLearningProbeService,
    evidence_svc: StudentLearningEvidenceService,
}

impl LearningAnalysisApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            profile_svc: StudentLearningProfileService::new(db.clone()),
            probe_svc: StudentLearningProbeService::new(db.clone()),
            evidence_svc: StudentLearningEvidenceService::new(db),
        }
    }

    pub async fn analyze_student_message(
        &self,
        user_id: Uuid,
        conversation_id: Uuid,
        message_id: Uuid,
        sequence: i64,
        content: Option<String>,
    ) -> BizResult<()> {
        let profile = self.get_or_create_profile(user_id).await?;
        if !profile.enabled {
            return Ok(());
        }
        let Some(content) = content
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };

        let offered_probes = self
            .probe_svc
            .list_offered_by_conversation(conversation_id)
            .await?;
        let engaged_probe = offered_probes
            .iter()
            .find(|probe| probe.context_snapshot.get("engagedAt").is_some());
        if let Some(engaged_probe) = engaged_probe
            && !is_eligible_probe_answer(
                &content,
                crate::api::learning_progress::minimum_probe_answer_characters()?,
            )
        {
            self.probe_svc
                .update_status(
                    repo::table::student_learning_probes::UpdateStudentLearningProbeStatus {
                        id: engaged_probe.id,
                        status: "skipped".to_owned(),
                    },
                )
                .await?;
            return Ok(());
        }
        let classifier = classifier()?;
        let framework = framework()?;
        let Some(class) = classify(&content, engaged_probe, &framework, &classifier)? else {
            return Ok(());
        };

        let now = OffsetDateTime::now_utc();
        if class.probe_id.is_some()
            && let Some(probe) = engaged_probe
        {
            self.probe_svc
                .mark_answered(MarkStudentLearningProbeAnswered {
                    id: probe.id,
                    answer_message_id: message_id,
                    answered_at: now,
                })
                .await?;
        }

        let assessment_outcome = class
            .rubric_hits
            .get("outcome")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let inserted = self
            .evidence_svc
            .create_if_absent(CreateStudentLearningEvidence {
                id: Uuid::new_v4(),
                user_id,
                conversation_id,
                source_message_id: Some(message_id),
                probe_id: class.probe_id,
                framework_version: profile.framework_version.clone(),
                knowledge_code: class.knowledge_code.clone(),
                evidence_type: class.evidence_type,
                direction: class.direction,
                strength: class.strength,
                confidence: class.confidence,
                summary: class.summary,
                rubric_hits: class.rubric_hits,
                analyzer_version: classifier.analyzer_version.clone(),
                created_at: now,
            })
            .await?;

        if !inserted {
            return Ok(());
        }

        let mastery_map = if class.contributes_to_mastery {
            merge_mastery(
                &profile.mastery_map,
                &class.knowledge_code,
                class.strength,
                assessment_outcome.as_deref(),
                now,
            )
        } else {
            profile.mastery_map.clone()
        };
        let coverage_rate = mastery_map.as_object().map_or(0, |map| map.len()) as f64
            / framework.domains.len() as f64;
        let engagement_summary =
            merge_engagement(&profile.engagement_summary, &class.knowledge_code, now);
        self.profile_svc
            .update_analysis(UpdateStudentLearningAnalysis {
                id: profile.id,
                mastery_map,
                engagement_summary,
                coverage_rate,
                analysis_status: "done".to_owned(),
                last_analyzed_sequence: sequence,
                last_analyzed_at: now,
                last_error_code: None,
            })
            .await?;
        Ok(())
    }

    async fn get_or_create_profile(&self, user_id: Uuid) -> BizResult<StudentLearningProfile> {
        match self.profile_svc.get_by_user_id(user_id).await? {
            Some(profile) => Ok(profile),
            None => {
                self.profile_svc
                    .create(CreateStudentLearningProfile {
                        id: Uuid::new_v4(),
                        user_id,
                        framework_version: framework()?.version,
                    })
                    .await
            }
        }
    }
}

struct ClassifiedEvidence {
    knowledge_code: String,
    probe_id: Option<Uuid>,
    evidence_type: String,
    direction: String,
    strength: f64,
    confidence: f64,
    summary: String,
    rubric_hits: Value,
    contributes_to_mastery: bool,
}

fn classify(
    content: &str,
    probe: Option<&StudentLearningProbe>,
    framework: &Framework,
    classifier: &Classifier,
) -> BizResult<Option<ClassifiedEvidence>> {
    let lowered = content.to_lowercase();
    let summary = truncate(content, 140);

    if let Some(probe) = probe {
        let Some((bank, question)) = question_for_probe(&probe.context_snapshot)? else {
            return Ok(Some(ClassifiedEvidence {
                knowledge_code: probe.knowledge_code.clone(),
                probe_id: Some(probe.id),
                evidence_type: "probe_answer".to_owned(),
                direction: "neutral".to_owned(),
                strength: 0.0,
                confidence: 1.0,
                summary,
                rubric_hits: json!({
                    "schemaVersion": 1,
                    "outcome": "legacy_unscored",
                    "evaluationMode": "none"
                }),
                contributes_to_mastery: false,
            }));
        };
        let assessment = evaluate_answer(&question, content);
        return Ok(Some(ClassifiedEvidence {
            knowledge_code: probe.knowledge_code.clone(),
            probe_id: Some(probe.id),
            evidence_type: "probe_answer".to_owned(),
            direction: assessment.direction.to_owned(),
            strength: assessment.score,
            confidence: 1.0,
            summary,
            rubric_hits: assessment.as_json(&bank, &question),
            contributes_to_mastery: assessment.contributes_to_mastery,
        }));
    }

    let mut best_domain: Option<(usize, String, usize)> = None;
    for (index, domain) in framework.domains.iter().enumerate() {
        let keywords = classifier
            .domains
            .get(&domain.code)
            .map_or(&[][..], |config| config.keywords.as_slice());
        let score = keywords
            .iter()
            .filter(|keyword| lowered.contains(&**keyword))
            .count();
        if score == 0 {
            continue;
        }
        let replace = match &best_domain {
            Some((best_score, _, best_index)) => {
                score > *best_score || (score == *best_score && index < *best_index)
            }
            None => true,
        };
        if replace {
            best_domain = Some((score, domain.code.clone(), index));
        }
    }
    let Some((_, knowledge_code, _)) = best_domain else {
        return Ok(None);
    };

    let (evidence_type, type_strength) = pick_evidence_type(&lowered, classifier);
    Ok(Some(ClassifiedEvidence {
        knowledge_code,
        probe_id: None,
        evidence_type: evidence_type.clone(),
        direction: "neutral".to_owned(),
        strength: strength_with_length(type_strength, content),
        confidence: 0.7,
        summary,
        rubric_hits: json!({
            "schemaVersion": 1,
            "outcome": "observed_unscored",
            "evaluationMode": "rule_v2_observation"
        }),
        contributes_to_mastery: true,
    }))
}

fn pick_evidence_type(lowered: &str, classifier: &Classifier) -> (String, f64) {
    const EVIDENCE_TYPE_ORDER: [&str; 3] = ["correction", "application", "explanation"];
    let mut best: Option<(&str, usize, f64)> = None;
    for name in EVIDENCE_TYPE_ORDER {
        let Some(config) = classifier.evidence_types.get(name) else {
            continue;
        };
        let score = config
            .keywords
            .iter()
            .filter(|keyword| lowered.contains(&**keyword))
            .count();
        if score == 0 {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|(_, best_score, _)| score > *best_score)
        {
            best = Some((name, score, config.strength));
        }
    }
    best.map_or_else(
        || ("explanation".to_owned(), 0.6),
        |(name, _, strength)| (name.to_owned(), strength),
    )
}

fn strength_with_length(base: f64, content: &str) -> f64 {
    let length_boost = 0.2 * ((content.chars().count() as f64) / 200.0).min(1.0);
    (base + length_boost).min(1.0)
}

fn is_eligible_probe_answer(content: &str, minimum_characters: usize) -> bool {
    let normalized = content.trim().to_lowercase();
    if normalized.chars().count() < minimum_characters {
        return false;
    }
    const NON_ANSWERS: [&str; 8] = [
        "不知道",
        "不清楚",
        "不会",
        "没想法",
        "跳过",
        "不想回答",
        "i don't know",
        "skip",
    ];
    !NON_ANSWERS.iter().any(|phrase| {
        normalized == *phrase || normalized.trim_end_matches(['。', '.', '！', '!']) == *phrase
    })
}

fn truncate(content: &str, max: usize) -> String {
    let mut text = content.chars().take(max).collect::<String>();
    if content.chars().count() > max {
        text.push('…');
    }
    text
}

pub(crate) fn merge_mastery(
    mastery_map: &Value,
    code: &str,
    strength: f64,
    assessment_outcome: Option<&str>,
    now: OffsetDateTime,
) -> Value {
    let mut map = mastery_map.as_object().cloned().unwrap_or_default();
    let entry = map.entry(code.to_owned()).or_insert_with(|| {
        json!({
            "status": "observed",
            "evidenceCount": 0,
            "strength": 0.0
        })
    });
    if let Some(obj) = entry.as_object_mut() {
        obj.insert("status".to_owned(), json!("observed"));
        let count = obj
            .get("evidenceCount")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            + 1;
        obj.insert("evidenceCount".to_owned(), json!(count));
        let previous = obj.get("strength").and_then(Value::as_f64).unwrap_or(0.0);
        obj.insert(
            "strength".to_owned(),
            json!(previous.max(strength).min(1.0)),
        );
        if let Some(outcome @ ("correct" | "partial" | "incorrect")) = assessment_outcome {
            let assessed_count = obj
                .get("assessedCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                + 1;
            let outcome_count_key = format!("{outcome}Count");
            let outcome_count = obj
                .get(&outcome_count_key)
                .and_then(Value::as_u64)
                .unwrap_or(0)
                + 1;
            let evaluation_status = match outcome {
                "correct" => "assessed",
                "partial" => "developing",
                "incorrect" => "needs_attention",
                _ => unreachable!(),
            };
            obj.insert("assessedCount".to_owned(), json!(assessed_count));
            obj.insert(outcome_count_key, json!(outcome_count));
            obj.insert("lastAssessmentOutcome".to_owned(), json!(outcome));
            obj.insert("evaluationStatus".to_owned(), json!(evaluation_status));
            obj.insert("lastAssessedAt".to_owned(), json!(rfc3339(now)));
        }
        obj.insert("lastObservedAt".to_owned(), json!(rfc3339(now)));
    }
    Value::Object(map)
}

pub(crate) fn merge_engagement(engagement: &Value, code: &str, now: OffsetDateTime) -> Value {
    let mut map = engagement.as_object().cloned().unwrap_or_default();
    let analyzed = map
        .get("analyzedMessages")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let evidence_total = map
        .get("evidenceCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    let mut domains = map
        .get("domainsEngaged")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !domains.iter().any(|value| value.as_str() == Some(code)) {
        domains.push(json!(code));
    }
    map.insert("analyzedMessages".to_owned(), json!(analyzed));
    map.insert("evidenceCount".to_owned(), json!(evidence_total));
    map.insert("domainsEngaged".to_owned(), json!(domains));
    map.insert("updatedAt".to_owned(), json!(rfc3339(now)));
    Value::Object(map)
}

fn rfc3339(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).unwrap_or_else(|_| value.to_string())
}

fn invalid(message: &str) -> BizError {
    BizError::new(app_error::LEARNING_REQUEST_INVALID, message.to_owned())
}

#[derive(Debug, Deserialize)]
struct Framework {
    version: String,
    domains: Vec<FrameworkDomain>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Classifier {
    analyzer_version: String,
    evidence_types: HashMap<String, EvidenceTypeConfig>,
    domains: HashMap<String, DomainClassifier>,
}

#[derive(Debug, Deserialize)]
struct EvidenceTypeConfig {
    strength: f64,
    keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainClassifier {
    keywords: Vec<String>,
}

fn classifier() -> BizResult<Classifier> {
    serde_json::from_str(include_str!(
        "../../../../config/json/learning/evidence-classifier.v1.json"
    ))
    .map_err(|_| invalid("learning evidence classifier invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn framework() -> Framework {
        super::framework().expect("framework should parse")
    }

    fn classifier() -> Classifier {
        super::classifier().expect("classifier should parse")
    }

    #[test]
    fn probe_answer_creates_probe_evidence() {
        let probe = StudentLearningProbe {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_id: Some(Uuid::new_v4()),
            assistant_message_id: None,
            answer_message_id: None,
            framework_version: "ecommerce_v1".to_owned(),
            knowledge_code: "market".to_owned(),
            status: "offered".to_owned(),
            question_text: "test".to_owned(),
            reason_code: "core_unobserved".to_owned(),
            priority_score: 1.0,
            context_snapshot: json!({}),
            offered_at: Some(OffsetDateTime::now_utc()),
            answered_at: None,
            expires_at: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        let class = classify(
            "我会先看市场规模和竞争情况",
            Some(&probe),
            &framework(),
            &classifier(),
        )
        .expect("probe classification should succeed")
        .expect("probe answer should classify");
        assert_eq!(class.knowledge_code, "market");
        assert_eq!(class.evidence_type, "probe_answer");
        assert_eq!(class.probe_id, Some(probe.id));
    }

    #[test]
    fn domain_keywords_select_best_domain() {
        let class = classify(
            "我觉得点击率下降的原因是因为转化环节，我的判断依据是转化率数据",
            None,
            &framework(),
            &classifier(),
        )
        .expect("classification should succeed")
        .expect("should classify");
        assert!(matches!(
            class.knowledge_code.as_str(),
            "traffic" | "conversion"
        ));
        assert_eq!(class.evidence_type, "explanation");
        assert_eq!(class.direction, "neutral");
    }

    #[test]
    fn ga4_checkout_turn_is_observed_as_conversion_without_overmatching_other_domains() {
        let classifier = classifier();
        let framework = framework();
        let cases = [
            (
                "某商品详情页访问量正常，但很多用户进入结账流程后没有购买。请告诉我应该检查哪些 GA4 电商事件和转化环节。",
                "conversion",
            ),
            (
                "我会整理商品详情页的标题、卖点、材质和尺寸，让顾客看懂商品信息。",
                "product",
            ),
            (
                "顾客购买后申请退货，客服需要核对售后规则并及时回复。",
                "service",
            ),
            (
                "使用个性化推荐会处理个人信息和算法推荐偏好，因此要提供隐私设置和便捷关闭方式，尊重用户选择权；即使可能影响转化率和下单，也必须守住合规底线。",
                "compliance",
            ),
            (
                "正式购买前，我会先研究市场需求、竞争情况和品类趋势。",
                "market",
            ),
        ];

        for (content, expected_domain) in cases {
            let class = classify(content, None, &framework, &classifier)
                .expect("classification should succeed")
                .expect("commerce content should classify");
            assert_eq!(class.knowledge_code, expected_domain, "content: {content}");
            assert_eq!(class.direction, "neutral");
        }
    }

    #[test]
    fn correction_keyword_uses_neutral_direction() {
        let class = classify(
            "我之前说错了，主图优化应该先看点击率再改标题，我修正一下判断",
            None,
            &framework(),
            &classifier(),
        )
        .expect("classification should succeed")
        .expect("should classify");
        assert_eq!(class.evidence_type, "correction");
        assert_eq!(class.direction, "neutral");
        assert!(class.strength > 0.0 && class.strength <= 1.0);
    }

    #[test]
    fn no_domain_match_returns_none() {
        let class = classify("今天天气不错", None, &framework(), &classifier())
            .expect("classification should succeed");
        assert!(class.is_none());
    }

    #[test]
    fn short_or_opt_out_probe_answers_are_not_eligible() {
        assert!(!is_eligible_probe_answer("不知道", 6));
        assert!(!is_eligible_probe_answer("跳过。", 6));
        assert!(!is_eligible_probe_answer("看看", 6));
        assert!(is_eligible_probe_answer("我会先检查流量人群是否精准", 6));
    }

    #[test]
    fn incorrect_assessment_is_counted_without_increasing_mastery_strength() {
        let merged = merge_mastery(
            &json!({}),
            "finance",
            0.0,
            Some("incorrect"),
            OffsetDateTime::now_utc(),
        );
        let entry = &merged["finance"];
        assert_eq!(entry["strength"], json!(0.0));
        assert_eq!(entry["assessedCount"], json!(1));
        assert_eq!(entry["incorrectCount"], json!(1));
        assert_eq!(entry["evaluationStatus"], json!("needs_attention"));
    }
}
