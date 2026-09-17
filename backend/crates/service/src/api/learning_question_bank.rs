use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use db_core::error::{BizError, BizResult};
use error_code::app as app_error;
use serde::Deserialize;
use serde_json::{Value, json};
use time::{Date, Month, OffsetDateTime};

const QUESTION_BANK_V1_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v1.json");
const QUESTION_BANK_V2_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v2.json");
const QUESTION_BANK_V3_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v3.json");
const QUESTION_BANK_V4_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v4.json");
const QUESTION_BANK_V5_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v5.json");
const QUESTION_BANK_V6_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v6.json");
const QUESTION_BANK_V7_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v7.json");
const QUESTION_BANK_V8_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v8.json");
const QUESTION_BANK_V9_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v9.json");
const QUESTION_BANK_V10_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v10.json");
const QUESTION_BANK_V11_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v11.json");
const QUESTION_BANK_V12_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v12.json");
const QUESTION_BANK_V13_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-question-bank.v13.json");
const ASSESSMENT_ANGLE_CATALOG_JSON: &str =
    include_str!("../../../../config/json/learning/ecommerce-diagnostic-assessment-angles.v4.json");
const APPLICABILITY_SCOPE_CATALOG_JSON: &str = include_str!(
    "../../../../config/json/learning/ecommerce-diagnostic-applicability-scopes.v3.json"
);
const QUESTION_BANK_PRODUCTION_RELEASE_JSON: &str = include_str!(
    "../../../../config/json/learning/ecommerce-diagnostic-question-bank.production-release.json"
);
const CURRENT_QUESTION_BANK_VERSION: &str = "ecommerce_diagnostic_v13";
const CURRENT_ASSESSMENT_ANGLE_VERSION: &str = "ecommerce_assessment_angles_v4";
const CURRENT_APPLICABILITY_SCOPE_VERSION: &str = "ecommerce_diagnostic_applicability_scopes_v3";
const CURRENT_QUESTION_BANK_SIZE: usize = 100;
const CURRENT_BASE_QUESTION_COUNT_PER_DOMAIN: usize = 10;
const CURRENT_PRODUCT_QUESTION_COUNT: usize = 21;
const CURRENT_SERVICE_QUESTION_COUNT: usize = 11;
const CURRENT_ASSESSMENT_ANGLE_COUNT: usize = 67;
const CURRENT_MIN_ANGLE_COUNT_PER_DOMAIN: usize = 7;
const CURRENT_MAX_QUESTIONS_PER_ANGLE: usize = 3;
const PRODUCTION_RELEASE_SCHEMA_VERSION: u32 = 2;
const CURRICULUM_REVIEWER_ROLE: &str = "ecommerce_curriculum_expert";
const PRODUCT_APPROVER_ROLE: &str = "learning_product_owner";
const CURRICULUM_RELEASE_DECISION: &str = "approved_for_release_review";
const PRODUCTION_RELEASE_DECISION: &str = "approved_for_production";
const REVIEW_CHECK_PASSED: &str = "passed";
const ECOMMERCE_DOMAIN_CODES: [&str; 8] = [
    "market",
    "product",
    "content",
    "traffic",
    "conversion",
    "finance",
    "service",
    "compliance",
];
const JURISDICTION_CODES: [&str; 4] = ["not_region_specific", "cn", "eu", "us"];
const PLATFORM_CODES: [&str; 6] = [
    "platform_agnostic",
    "google_trends",
    "google_merchant_center",
    "google_analytics_4",
    "google_search",
    "google_ads",
];
const BUSINESS_STAGE_CODES: [&str; 8] = [
    "market_research",
    "product_operations",
    "content_operations",
    "traffic_acquisition",
    "conversion_optimization",
    "financial_management",
    "customer_service",
    "compliance_governance",
];

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearningQuestionDeliveryMode {
    #[default]
    Disabled,
    InternalPilot,
    Production,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuestionBank {
    pub version: String,
    #[serde(default)]
    pub base_version: Option<String>,
    pub framework_version: String,
    pub source_revision: String,
    pub evaluation_mode: String,
    pub questions: Vec<DiagnosticQuestion>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticQuestion {
    pub id: String,
    pub version: u32,
    pub status: String,
    pub domain_code: String,
    pub question_type: String,
    pub prompt: String,
    #[serde(default)]
    pub choices: Vec<QuestionChoice>,
    #[serde(default)]
    pub correct_choice_id: Option<String>,
    pub reference_answer: String,
    pub rubric: QuestionRubric,
    pub sources: Vec<QuestionSource>,
    pub review: QuestionReview,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub(crate) struct QuestionChoice {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuestionRubric {
    pub version: u32,
    pub criteria: Vec<RubricCriterion>,
    pub contradictions: Vec<RubricCriterion>,
    pub partial_threshold: f64,
    pub correct_threshold: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RubricCriterion {
    pub code: String,
    #[serde(default)]
    pub weight: f64,
    pub concepts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuestionSource {
    pub source_id: String,
    pub authority_level: String,
    pub publisher: String,
    pub title: String,
    pub url: String,
    pub published_version: String,
    pub checked_at: String,
    pub valid_through: String,
    #[serde(default)]
    pub license_type: Option<String>,
    #[serde(default)]
    pub use_mode: Option<String>,
    #[serde(default)]
    pub license_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuestionReview {
    pub author_id: String,
    pub reviewer_id: String,
    pub review_scope: String,
    pub reviewer_type: String,
    pub curriculum_review_required: bool,
    pub reviewed_at: String,
    pub decision: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QuestionBankProductionRelease {
    schema_version: u32,
    question_bank_version: String,
    approvals: Vec<ProductionQuestionApproval>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionQuestionApproval {
    question_id: String,
    question_version: u32,
    curriculum_review: ProductionCurriculumReview,
    release_approval: ProductionReleaseApproval,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionCurriculumReview {
    reviewer_id: String,
    reviewed_at: String,
    reviewer_role: String,
    evidence_id: String,
    checks: ProductionCurriculumChecks,
    decision: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionCurriculumChecks {
    answer: String,
    distractors: String,
    explanation: String,
    difficulty_and_audience: String,
    applicability: String,
    source_currency: String,
}

impl ProductionCurriculumChecks {
    fn all_passed(&self) -> bool {
        [
            self.answer.as_str(),
            self.distractors.as_str(),
            self.explanation.as_str(),
            self.difficulty_and_audience.as_str(),
            self.applicability.as_str(),
            self.source_currency.as_str(),
        ]
        .into_iter()
        .all(|value| value == REVIEW_CHECK_PASSED)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionReleaseApproval {
    approver_id: String,
    approver_role: String,
    approved_at: String,
    decision: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssessmentAngleCatalog {
    schema_version: u32,
    version: String,
    question_bank_version: String,
    framework_version: String,
    mappings: Vec<AssessmentAngleMapping>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssessmentAngleMapping {
    question_id: String,
    question_version: u32,
    domain_code: String,
    angle_code: String,
}

static ASSESSMENT_ANGLE_LOOKUP: OnceLock<Option<HashMap<(String, u32), String>>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplicabilityScopeCatalog {
    schema_version: u32,
    catalog_version: String,
    question_bank_version: String,
    mappings: Vec<QuestionApplicabilityScope>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct QuestionApplicabilityScope {
    pub question_id: String,
    pub question_version: u32,
    pub jurisdiction_code: String,
    pub platform_code: String,
    pub business_stage_code: String,
}

static APPLICABILITY_SCOPE_LOOKUP: OnceLock<
    Option<HashMap<(String, u32), QuestionApplicabilityScope>>,
> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RubricAssessment {
    pub outcome: &'static str,
    pub score: f64,
    pub matched_criteria: Vec<String>,
    pub missing_criteria: Vec<String>,
    pub contradictions: Vec<String>,
    pub direction: &'static str,
    pub contributes_to_mastery: bool,
}

impl RubricAssessment {
    pub fn as_json(&self, bank: &QuestionBank, question: &DiagnosticQuestion) -> Value {
        let mut value = json!({
            "schemaVersion": 1,
            "questionBankVersion": bank.version,
            "questionId": question.id,
            "questionVersion": question.version,
            "rubricVersion": question.rubric.version,
            "outcome": self.outcome,
            "score": self.score,
            "matchedCriteria": self.matched_criteria,
            "missingCriteria": self.missing_criteria,
            "contradictions": self.contradictions,
            "evaluationMode": bank.evaluation_mode
        });
        if let Some(angle_code) = assessment_angle_code(&question.id, question.version) {
            value["assessmentAngleCode"] = json!(angle_code);
        }
        value
    }
}

pub(crate) fn question_bank() -> BizResult<QuestionBank> {
    let bank = question_bank_catalog()?
        .into_iter()
        .find(|bank| bank.version == CURRENT_QUESTION_BANK_VERSION)
        .ok_or_else(|| invalid("current learning diagnostic question bank missing"))?;
    validate_assessment_angle_catalog(&bank)?;
    validate_applicability_scope_catalog(&bank)?;
    Ok(bank)
}

pub(crate) fn assessment_angle_code(
    question_id: &str,
    question_version: u32,
) -> Option<&'static str> {
    ASSESSMENT_ANGLE_LOOKUP
        .get_or_init(|| {
            serde_json::from_str::<AssessmentAngleCatalog>(ASSESSMENT_ANGLE_CATALOG_JSON)
                .ok()
                .map(|catalog| {
                    catalog
                        .mappings
                        .into_iter()
                        .map(|mapping| {
                            (
                                (mapping.question_id, mapping.question_version),
                                mapping.angle_code,
                            )
                        })
                        .collect()
                })
        })
        .as_ref()?
        .get(&(question_id.to_owned(), question_version))
        .map(String::as_str)
}

pub(crate) fn assessment_angle_depth(bank: &QuestionBank) -> BizResult<(u64, u64)> {
    let mut question_counts = HashMap::<&str, u64>::new();
    for question in &bank.questions {
        let angle_code = assessment_angle_code(&question.id, question.version)
            .ok_or_else(|| invalid("assessment angle mapping missing"))?;
        *question_counts.entry(angle_code).or_default() += 1;
    }

    Ok((
        question_counts.len() as u64,
        question_counts
            .values()
            .filter(|count| **count == 1)
            .count() as u64,
    ))
}

pub(crate) fn question_applicability_scope(
    question_id: &str,
    question_version: u32,
) -> Option<&'static QuestionApplicabilityScope> {
    APPLICABILITY_SCOPE_LOOKUP
        .get_or_init(|| {
            serde_json::from_str::<ApplicabilityScopeCatalog>(APPLICABILITY_SCOPE_CATALOG_JSON)
                .ok()
                .map(|catalog| {
                    catalog
                        .mappings
                        .into_iter()
                        .map(|mapping| {
                            (
                                (mapping.question_id.clone(), mapping.question_version),
                                mapping,
                            )
                        })
                        .collect()
                })
        })
        .as_ref()?
        .get(&(question_id.to_owned(), question_version))
}

#[cfg(test)]
fn approved_question_for_domain(
    bank: &QuestionBank,
    domain_code: &str,
    selection_seed: &str,
    excluded_question_id: Option<&str>,
    preferred_question_ids: &[String],
) -> Option<DiagnosticQuestion> {
    deliverable_question_for_domain(
        bank,
        LearningQuestionDeliveryMode::InternalPilot,
        domain_code,
        selection_seed,
        excluded_question_id,
        preferred_question_ids,
    )
    .expect("internal-pilot question selection should be valid")
}

pub(crate) fn deliverable_question_for_domain(
    bank: &QuestionBank,
    mode: LearningQuestionDeliveryMode,
    domain_code: &str,
    selection_seed: &str,
    excluded_question_id: Option<&str>,
    preferred_question_ids: &[String],
) -> BizResult<Option<DiagnosticQuestion>> {
    let production_approvals = match mode {
        LearningQuestionDeliveryMode::Production => {
            Some(production_approved_question_identities(bank)?)
        }
        LearningQuestionDeliveryMode::Disabled | LearningQuestionDeliveryMode::InternalPilot => {
            None
        }
    };
    let candidates = bank
        .questions
        .iter()
        .filter(|question| {
            question.status == "approved"
                && question.domain_code == domain_code
                && sources_are_current(question)
                && match mode {
                    LearningQuestionDeliveryMode::Disabled => false,
                    LearningQuestionDeliveryMode::InternalPilot => true,
                    LearningQuestionDeliveryMode::Production => {
                        production_approvals.as_ref().is_some_and(|approvals| {
                            approvals.contains(&(question.id.clone(), question.version))
                        })
                    }
                }
        })
        .collect::<Vec<_>>();
    Ok(select_question_for_domain(
        candidates,
        selection_seed,
        excluded_question_id,
        preferred_question_ids,
    ))
}

pub(crate) fn question_is_deliverable_for_mode(
    bank: &QuestionBank,
    question: &DiagnosticQuestion,
    mode: LearningQuestionDeliveryMode,
) -> BizResult<bool> {
    if question.status != "approved" || !sources_are_current(question) {
        return Ok(false);
    }
    match mode {
        LearningQuestionDeliveryMode::Disabled => Ok(false),
        LearningQuestionDeliveryMode::InternalPilot => Ok(true),
        LearningQuestionDeliveryMode::Production => Ok(bank.version
            == CURRENT_QUESTION_BANK_VERSION
            && production_approved_question_identities(bank)?
                .contains(&(question.id.clone(), question.version))),
    }
}

fn select_question_for_domain(
    mut candidates: Vec<&DiagnosticQuestion>,
    selection_seed: &str,
    excluded_question_id: Option<&str>,
    preferred_question_ids: &[String],
) -> Option<DiagnosticQuestion> {
    if candidates
        .iter()
        .any(|question| question.question_type == "single_choice")
    {
        candidates.retain(|question| question.question_type == "single_choice");
    }
    if !preferred_question_ids.is_empty() {
        candidates.retain(|question| preferred_question_ids.contains(&question.id));
    }
    candidates.sort_by_key(|question| question.id.as_str());
    if candidates.len() > 1
        && let Some(excluded_question_id) = excluded_question_id
    {
        candidates.retain(|question| question.id != excluded_question_id);
    }
    if candidates.is_empty() {
        return None;
    }
    let index = stable_selection_index(selection_seed, candidates.len());
    candidates.get(index).cloned().cloned()
}

pub(crate) fn production_approved_question_identities(
    bank: &QuestionBank,
) -> BizResult<HashSet<(String, u32)>> {
    let release: QuestionBankProductionRelease =
        serde_json::from_str(QUESTION_BANK_PRODUCTION_RELEASE_JSON)
            .map_err(|_| invalid("learning diagnostic production release invalid"))?;
    validate_production_release(bank, &release)
}

fn validate_production_release(
    bank: &QuestionBank,
    release: &QuestionBankProductionRelease,
) -> BizResult<HashSet<(String, u32)>> {
    if release.schema_version != PRODUCTION_RELEASE_SCHEMA_VERSION
        || release.question_bank_version != bank.version
    {
        return Err(invalid(
            "learning diagnostic production release metadata invalid",
        ));
    }
    let today = OffsetDateTime::now_utc().date();
    let mut identities = HashSet::new();
    for approval in &release.approvals {
        let question = bank.questions.iter().find(|question| {
            question.id == approval.question_id && question.version == approval.question_version
        });
        if approval.question_id.trim().is_empty()
            || approval.question_version == 0
            || question
                .is_none_or(|question| !production_approval_is_valid(question, approval, today))
            || !identities.insert((approval.question_id.clone(), approval.question_version))
        {
            return Err(invalid("learning diagnostic production approval invalid"));
        }
    }
    Ok(identities)
}

fn production_approval_is_valid(
    question: &DiagnosticQuestion,
    approval: &ProductionQuestionApproval,
    today: Date,
) -> bool {
    let curriculum = &approval.curriculum_review;
    let release = &approval.release_approval;
    let Some(reviewed_at) = parse_date(&curriculum.reviewed_at) else {
        return false;
    };
    let Some(approved_at) = parse_date(&release.approved_at) else {
        return false;
    };
    let Some(engineering_reviewed_at) = parse_date(&question.review.reviewed_at) else {
        return false;
    };
    let Some(latest_source_checked_at) = latest_source_checked_at(question) else {
        return false;
    };

    !curriculum.reviewer_id.trim().is_empty()
        && !curriculum.reviewer_id.to_lowercase().contains("automated")
        && curriculum.reviewer_id != question.review.reviewer_id
        && curriculum.reviewer_id != question.review.author_id
        && curriculum.reviewer_role == CURRICULUM_REVIEWER_ROLE
        && !curriculum.evidence_id.trim().is_empty()
        && curriculum.checks.all_passed()
        && curriculum.decision == CURRICULUM_RELEASE_DECISION
        && reviewed_at >= engineering_reviewed_at
        && reviewed_at >= latest_source_checked_at
        && reviewed_at <= today
        && !release.approver_id.trim().is_empty()
        && !release.approver_id.to_lowercase().contains("automated")
        && release.approver_id != curriculum.reviewer_id
        && release.approver_id != question.review.reviewer_id
        && release.approver_id != question.review.author_id
        && release.approver_role == PRODUCT_APPROVER_ROLE
        && release.decision == PRODUCTION_RELEASE_DECISION
        && approved_at >= reviewed_at
        && approved_at <= today
}

fn latest_source_checked_at(question: &DiagnosticQuestion) -> Option<Date> {
    question
        .sources
        .iter()
        .map(|source| parse_date(&source.checked_at))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max()
}

fn validate_assessment_angle_catalog(bank: &QuestionBank) -> BizResult<()> {
    let catalog: AssessmentAngleCatalog = serde_json::from_str(ASSESSMENT_ANGLE_CATALOG_JSON)
        .map_err(|_| invalid("learning diagnostic assessment angle catalog invalid"))?;
    if catalog.schema_version != 1
        || catalog.version != CURRENT_ASSESSMENT_ANGLE_VERSION
        || catalog.question_bank_version != bank.version
        || catalog.framework_version != bank.framework_version
    {
        return Err(invalid(
            "learning diagnostic assessment angle metadata invalid",
        ));
    }

    let mut identities = HashSet::new();
    let mut angle_counts = HashMap::<String, usize>::new();
    let mut angles_by_domain = HashMap::<String, HashSet<String>>::new();
    for mapping in &catalog.mappings {
        let identity = (mapping.question_id.clone(), mapping.question_version);
        let question = bank.questions.iter().find(|question| {
            question.id == mapping.question_id && question.version == mapping.question_version
        });
        let expected_prefix = format!("{}.", mapping.domain_code);
        if !identities.insert(identity)
            || question.is_none()
            || question.is_some_and(|question| question.domain_code != mapping.domain_code)
            || !ECOMMERCE_DOMAIN_CODES.contains(&mapping.domain_code.as_str())
            || !mapping.angle_code.starts_with(&expected_prefix)
            || mapping.angle_code.len() <= expected_prefix.len()
        {
            return Err(invalid(
                "learning diagnostic assessment angle mapping invalid",
            ));
        }
        *angle_counts.entry(mapping.angle_code.clone()).or_default() += 1;
        angles_by_domain
            .entry(mapping.domain_code.clone())
            .or_default()
            .insert(mapping.angle_code.clone());
    }

    if identities.len() != bank.questions.len()
        || angle_counts.len() != CURRENT_ASSESSMENT_ANGLE_COUNT
        || bank
            .questions
            .iter()
            .any(|question| !identities.contains(&(question.id.clone(), question.version)))
        || angle_counts
            .values()
            .any(|count| *count > CURRENT_MAX_QUESTIONS_PER_ANGLE)
        || ECOMMERCE_DOMAIN_CODES.iter().any(|domain| {
            angles_by_domain
                .get(*domain)
                .is_none_or(|angles| angles.len() < CURRENT_MIN_ANGLE_COUNT_PER_DOMAIN)
        })
    {
        return Err(invalid(
            "learning diagnostic assessment angle coverage invalid",
        ));
    }
    Ok(())
}

fn validate_applicability_scope_catalog(bank: &QuestionBank) -> BizResult<()> {
    let catalog: ApplicabilityScopeCatalog = serde_json::from_str(APPLICABILITY_SCOPE_CATALOG_JSON)
        .map_err(|_| invalid("learning diagnostic applicability scope catalog invalid"))?;
    validate_applicability_scope_catalog_data(bank, &catalog)
}

fn validate_applicability_scope_catalog_data(
    bank: &QuestionBank,
    catalog: &ApplicabilityScopeCatalog,
) -> BizResult<()> {
    if catalog.schema_version != 1
        || catalog.catalog_version != CURRENT_APPLICABILITY_SCOPE_VERSION
        || catalog.question_bank_version != bank.version
    {
        return Err(invalid(
            "learning diagnostic applicability scope metadata invalid",
        ));
    }

    let mut identities = HashSet::new();
    for mapping in &catalog.mappings {
        let identity = (mapping.question_id.clone(), mapping.question_version);
        let question = bank.questions.iter().find(|question| {
            question.id == mapping.question_id && question.version == mapping.question_version
        });
        if !identities.insert(identity)
            || question.is_none()
            || !JURISDICTION_CODES.contains(&mapping.jurisdiction_code.as_str())
            || !PLATFORM_CODES.contains(&mapping.platform_code.as_str())
            || !BUSINESS_STAGE_CODES.contains(&mapping.business_stage_code.as_str())
            || question.is_some_and(|question| {
                business_stage_for_domain(&question.domain_code)
                    != Some(mapping.business_stage_code.as_str())
                    || jurisdiction_for_question_id(&question.id) != mapping.jurisdiction_code
            })
        {
            return Err(invalid(
                "learning diagnostic applicability scope mapping invalid",
            ));
        }
    }

    if identities.len() != bank.questions.len()
        || bank
            .questions
            .iter()
            .any(|question| !identities.contains(&(question.id.clone(), question.version)))
    {
        return Err(invalid(
            "learning diagnostic applicability scope coverage invalid",
        ));
    }
    Ok(())
}

fn business_stage_for_domain(domain_code: &str) -> Option<&'static str> {
    match domain_code {
        "market" => Some("market_research"),
        "product" => Some("product_operations"),
        "content" => Some("content_operations"),
        "traffic" => Some("traffic_acquisition"),
        "conversion" => Some("conversion_optimization"),
        "finance" => Some("financial_management"),
        "service" => Some("customer_service"),
        "compliance" => Some("compliance_governance"),
        _ => None,
    }
}

fn jurisdiction_for_question_id(question_id: &str) -> &'static str {
    if question_id.contains(".cn_") {
        "cn"
    } else if question_id.contains(".eu_") {
        "eu"
    } else if question_id.contains(".ftc_") {
        "us"
    } else {
        "not_region_specific"
    }
}

fn stable_selection_index(seed: &str, candidate_count: usize) -> usize {
    let hash = seed
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    (hash % candidate_count as u64) as usize
}

pub(crate) fn question_for_probe(
    snapshot: &Value,
) -> BizResult<Option<(QuestionBank, DiagnosticQuestion)>> {
    let Some(question_id) = snapshot.get("questionId").and_then(Value::as_str) else {
        return Ok(None);
    };
    let Some(question_version) = snapshot.get("questionVersion").and_then(Value::as_u64) else {
        return Ok(None);
    };
    let Some(rubric_version) = snapshot.get("rubricVersion").and_then(Value::as_u64) else {
        return Ok(None);
    };
    let Ok(question_version) = u32::try_from(question_version) else {
        return Ok(None);
    };
    let Ok(rubric_version) = u32::try_from(rubric_version) else {
        return Ok(None);
    };
    let requested_bank_version = snapshot.get("questionBankVersion").and_then(Value::as_str);
    let requested_source_revision = snapshot.get("sourceRevision").and_then(Value::as_str);

    Ok(question_bank_catalog()?.into_iter().find_map(|bank| {
        if requested_bank_version.is_some_and(|version| version != bank.version)
            || requested_source_revision.is_some_and(|revision| revision != bank.source_revision)
        {
            return None;
        }
        let question = bank
            .questions
            .iter()
            .find(|question| {
                question.id == question_id
                    && question.version == question_version
                    && question.rubric.version == rubric_version
            })
            .cloned()?;
        Some((bank, question))
    }))
}

fn question_bank_catalog() -> BizResult<Vec<QuestionBank>> {
    let mut versions = HashSet::new();
    let mut catalog = [
        QUESTION_BANK_V13_JSON,
        QUESTION_BANK_V12_JSON,
        QUESTION_BANK_V11_JSON,
        QUESTION_BANK_V10_JSON,
        QUESTION_BANK_V9_JSON,
        QUESTION_BANK_V8_JSON,
        QUESTION_BANK_V7_JSON,
        QUESTION_BANK_V6_JSON,
        QUESTION_BANK_V5_JSON,
        QUESTION_BANK_V4_JSON,
        QUESTION_BANK_V3_JSON,
        QUESTION_BANK_V2_JSON,
        QUESTION_BANK_V1_JSON,
    ]
    .into_iter()
    .map(|source| {
        let bank: QuestionBank = serde_json::from_str(source)
            .map_err(|_| invalid("learning diagnostic question bank invalid"))?;
        if !versions.insert(bank.version.clone()) {
            return Err(invalid(
                "learning diagnostic question bank version duplicated",
            ));
        }
        Ok(bank)
    })
    .collect::<BizResult<Vec<_>>>()?;

    for index in (0..catalog.len()).rev() {
        let Some(base_version) = catalog[index].base_version.clone() else {
            continue;
        };
        let mut inherited = catalog
            .iter()
            .find(|bank| bank.version == base_version)
            .ok_or_else(|| invalid("learning diagnostic question bank base version missing"))?
            .questions
            .clone();
        for question in &mut inherited {
            for source in &mut question.sources {
                source.license_type = Some("official_reference_only".to_owned());
                source.use_mode = Some("source_based_original".to_owned());
                source.license_url = Some(source.url.clone());
            }
        }
        inherited.append(&mut catalog[index].questions);
        catalog[index].questions = inherited;
    }
    for bank in &catalog {
        validate_question_bank(bank)?;
    }
    Ok(catalog)
}

fn expected_question_count_for_domain(domain: &str) -> usize {
    match domain {
        "product" => CURRENT_PRODUCT_QUESTION_COUNT,
        "service" => CURRENT_SERVICE_QUESTION_COUNT,
        "content" | "traffic" | "conversion" | "finance" => 12,
        _ => CURRENT_BASE_QUESTION_COUNT_PER_DOMAIN,
    }
}

pub(crate) fn evaluate_answer(question: &DiagnosticQuestion, answer: &str) -> RubricAssessment {
    if question.question_type == "single_choice" {
        return evaluate_single_choice(question, answer);
    }
    let normalized = normalize(answer);
    let contradictions = matching_codes(&question.rubric.contradictions, &normalized);
    let matched_criteria = matching_codes(&question.rubric.criteria, &normalized);
    let matched_set = matched_criteria.iter().collect::<HashSet<_>>();
    let missing_criteria = question
        .rubric
        .criteria
        .iter()
        .filter(|criterion| !matched_set.contains(&criterion.code))
        .map(|criterion| criterion.code.clone())
        .collect::<Vec<_>>();
    let score = question
        .rubric
        .criteria
        .iter()
        .filter(|criterion| matched_set.contains(&criterion.code))
        .map(|criterion| criterion.weight)
        .sum::<f64>();

    let (outcome, direction, contributes_to_mastery) = if !contradictions.is_empty() {
        ("incorrect", "negative", true)
    } else if score >= question.rubric.correct_threshold {
        ("correct", "positive", true)
    } else if score >= question.rubric.partial_threshold {
        ("partial", "positive", true)
    } else {
        ("unscorable", "neutral", false)
    };

    RubricAssessment {
        outcome,
        score,
        matched_criteria,
        missing_criteria,
        contradictions,
        direction,
        contributes_to_mastery,
    }
}

pub(crate) fn evaluate_choice_id(
    question: &DiagnosticQuestion,
    choice_id: &str,
) -> BizResult<RubricAssessment> {
    if question.question_type != "single_choice" {
        return Err(invalid("learning probe question is not single choice"));
    }
    let selected = question
        .choices
        .iter()
        .find(|choice| choice.id == choice_id)
        .ok_or_else(|| invalid("learning probe choice invalid"))?;
    let correct = question
        .correct_choice_id
        .as_deref()
        .ok_or_else(|| invalid("learning probe answer key missing"))?;
    Ok(single_choice_assessment(question, selected.id == correct))
}

fn evaluate_single_choice(question: &DiagnosticQuestion, answer: &str) -> RubricAssessment {
    let answer = answer.trim();
    let answer = [
        "我的选择是：",
        "我的选择是:",
        "My choice is:",
        "My choice is：",
    ]
    .into_iter()
    .find_map(|prefix| answer.strip_prefix(prefix))
    .unwrap_or(answer)
    .trim();
    let normalized_answer = normalize(answer);
    let selected = question
        .choices
        .iter()
        .find(|choice| normalize(&choice.text) == normalized_answer);
    let Some(selected) = selected else {
        return RubricAssessment {
            outcome: "unscorable",
            score: 0.0,
            matched_criteria: Vec::new(),
            missing_criteria: question
                .rubric
                .criteria
                .iter()
                .map(|criterion| criterion.code.clone())
                .collect(),
            contradictions: Vec::new(),
            direction: "neutral",
            contributes_to_mastery: false,
        };
    };
    let correct = question.correct_choice_id.as_deref();
    single_choice_assessment(question, correct == Some(selected.id.as_str()))
}

fn single_choice_assessment(question: &DiagnosticQuestion, is_correct: bool) -> RubricAssessment {
    RubricAssessment {
        outcome: if is_correct { "correct" } else { "incorrect" },
        score: if is_correct { 1.0 } else { 0.0 },
        matched_criteria: if is_correct {
            question
                .rubric
                .criteria
                .iter()
                .map(|criterion| criterion.code.clone())
                .collect()
        } else {
            Vec::new()
        },
        missing_criteria: if is_correct {
            Vec::new()
        } else {
            question
                .rubric
                .criteria
                .iter()
                .map(|criterion| criterion.code.clone())
                .collect()
        },
        contradictions: if is_correct {
            Vec::new()
        } else {
            vec!["selected_incorrect_choice".to_owned()]
        },
        direction: if is_correct { "positive" } else { "negative" },
        contributes_to_mastery: true,
    }
}

fn matching_codes(criteria: &[RubricCriterion], normalized: &str) -> Vec<String> {
    criteria
        .iter()
        .filter(|criterion| {
            criterion
                .concepts
                .iter()
                .any(|concept| normalized.contains(&normalize(concept)))
        })
        .map(|criterion| criterion.code.clone())
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn validate_question_bank(bank: &QuestionBank) -> BizResult<()> {
    if bank.version.trim().is_empty()
        || bank.framework_version.trim().is_empty()
        || parse_date(&bank.source_revision).is_none()
        || bank.evaluation_mode != "deterministic_rubric_v1"
    {
        return Err(invalid(
            "learning diagnostic question bank metadata invalid",
        ));
    }
    let mut identities = HashSet::new();
    let mut prompts = HashSet::new();
    let mut choice_sets = HashSet::new();
    for question in &bank.questions {
        if !identities.insert((question.id.as_str(), question.version)) {
            return Err(invalid("learning diagnostic question identity duplicated"));
        }
        if !prompts.insert(normalize(&question.prompt)) {
            return Err(invalid("learning diagnostic question prompt duplicated"));
        }
        let mut normalized_choices = question
            .choices
            .iter()
            .map(|choice| normalize(&choice.text))
            .collect::<Vec<_>>();
        normalized_choices.sort();
        if !normalized_choices.is_empty() && !choice_sets.insert(normalized_choices) {
            return Err(invalid("learning diagnostic question choices duplicated"));
        }
        validate_question(question, bank.version == CURRENT_QUESTION_BANK_VERSION)?;
    }
    if bank.version == CURRENT_QUESTION_BANK_VERSION {
        if bank.questions.len() != CURRENT_QUESTION_BANK_SIZE {
            return Err(invalid(
                "current learning diagnostic question bank size invalid",
            ));
        }
        if bank.questions.iter().any(|question| {
            question.status != "approved"
                || question.question_type != "single_choice"
                || question.choices.len() != 4
        }) {
            return Err(invalid(
                "current learning diagnostic question bank composition invalid",
            ));
        }
        for domain in ECOMMERCE_DOMAIN_CODES {
            let expected_count = expected_question_count_for_domain(domain);
            if bank
                .questions
                .iter()
                .filter(|question| question.domain_code == domain)
                .count()
                != expected_count
            {
                return Err(invalid(
                    "current learning diagnostic question bank domain coverage insufficient",
                ));
            }
        }
        let mut answer_positions = [0_usize; 4];
        for question in &bank.questions {
            let position = question
                .choices
                .iter()
                .position(|choice| {
                    Some(choice.id.as_str()) == question.correct_choice_id.as_deref()
                })
                .ok_or_else(|| invalid("current learning diagnostic answer key missing"))?;
            answer_positions[position] += 1;
        }
        if answer_positions.iter().any(|count| *count < 2)
            || answer_positions
                .iter()
                .any(|count| *count > bank.questions.len().div_ceil(2))
        {
            return Err(invalid(
                "current learning diagnostic answer position distribution invalid",
            ));
        }
    }
    Ok(())
}

fn validate_question(question: &DiagnosticQuestion, require_source_usage: bool) -> BizResult<()> {
    const STATUSES: [&str; 4] = ["draft", "reviewed", "approved", "retired"];
    const QUESTION_TYPES: [&str; 4] = [
        "short_reasoning",
        "scenario_decision",
        "error_diagnosis",
        "single_choice",
    ];
    if !STATUSES.contains(&question.status.as_str())
        || !QUESTION_TYPES.contains(&question.question_type.as_str())
        || !ECOMMERCE_DOMAIN_CODES.contains(&question.domain_code.as_str())
        || question.id.trim().is_empty()
        || question.version == 0
    {
        return Err(invalid("learning diagnostic question metadata invalid"));
    }
    if question.status != "approved" {
        return Ok(());
    }
    let is_single_choice = question.question_type == "single_choice";
    let mut choice_ids = HashSet::new();
    let mut choice_texts = HashSet::new();
    let choice_config_invalid = if is_single_choice {
        question.choices.len() < 2
            || question
                .correct_choice_id
                .as_deref()
                .is_none_or(str::is_empty)
            || question.choices.iter().any(|choice| {
                choice.id.trim().is_empty()
                    || choice.text.trim().is_empty()
                    || !choice_ids.insert(choice.id.as_str())
                    || !choice_texts.insert(normalize(&choice.text))
            })
            || !question
                .choices
                .iter()
                .any(|choice| question.correct_choice_id.as_deref() == Some(choice.id.as_str()))
    } else {
        !question.choices.is_empty() || question.correct_choice_id.is_some()
    };
    if question.prompt.trim().is_empty()
        || question.reference_answer.trim().is_empty()
        || question.rubric.version == 0
        || question.rubric.criteria.is_empty()
        || question.sources.is_empty()
        || question.review.decision != "approved_for_internal_pilot"
        || question.review.author_id == question.review.reviewer_id
        || question.review.review_scope != "source_and_rubric_engineering_review"
        || question.review.reviewer_type != "automated_engineering_gate"
        || !question.review.curriculum_review_required
        || parse_date(&question.review.reviewed_at).is_none()
        || choice_config_invalid
    {
        return Err(invalid("approved learning diagnostic question incomplete"));
    }
    let weight = question
        .rubric
        .criteria
        .iter()
        .map(|criterion| criterion.weight)
        .sum::<f64>();
    let mut codes = HashSet::new();
    if (weight - 1.0).abs() > 0.000_001
        || !(0.0..=1.0).contains(&question.rubric.partial_threshold)
        || !(0.0..=1.0).contains(&question.rubric.correct_threshold)
        || question.rubric.partial_threshold > question.rubric.correct_threshold
        || question.rubric.criteria.iter().any(|criterion| {
            criterion.code.trim().is_empty()
                || criterion.concepts.is_empty()
                || !codes.insert(criterion.code.as_str())
        })
        || question.sources.iter().any(|source| {
            let checked_at = parse_date(&source.checked_at);
            let valid_through = parse_date(&source.valid_through);
            source.source_id.trim().is_empty()
                || !matches!(
                    source.authority_level.as_str(),
                    "primary_official" | "standard" | "textbook" | "internal_expert"
                )
                || source.publisher.trim().is_empty()
                || source.title.trim().is_empty()
                || !source.url.starts_with("https://")
                || source.url.contains("example.invalid")
                || source.published_version.trim().is_empty()
                || checked_at.is_none()
                || valid_through.is_none()
                || checked_at > valid_through
                || (require_source_usage
                    && (!matches!(
                        source.license_type.as_deref(),
                        Some("official_reference_only" | "cc_by_4_0")
                    ) || !matches!(
                        source.use_mode.as_deref(),
                        Some("source_based_original" | "adapted_with_attribution")
                    ) || source
                        .license_url
                        .as_deref()
                        .is_none_or(|url| !url.starts_with("https://"))))
        })
    {
        return Err(invalid(
            "approved learning diagnostic question rubric invalid",
        ));
    }
    Ok(())
}

fn sources_are_current(question: &DiagnosticQuestion) -> bool {
    let today = OffsetDateTime::now_utc().date();
    question.sources.iter().all(|source| {
        parse_date(&source.valid_through).is_some_and(|valid_through| valid_through >= today)
    })
}

fn parse_date(value: &str) -> Option<Date> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = Month::try_from(parts.next()?.parse::<u8>().ok()?).ok()?;
    let day = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Date::from_calendar_date(year, month, day).ok()
}

fn invalid(message: &str) -> BizError {
    BizError::new(app_error::LEARNING_REQUEST_INVALID, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choose(bank: &QuestionBank, domain_code: &str) -> Option<DiagnosticQuestion> {
        approved_question_for_domain(bank, domain_code, "test-seed", None, &[])
    }

    fn open_question_v2(domain_code: &str) -> DiagnosticQuestion {
        question_bank_catalog()
            .expect("question bank catalog should be valid")
            .into_iter()
            .find(|bank| bank.version == "ecommerce_diagnostic_v2")
            .and_then(|bank| {
                bank.questions
                    .into_iter()
                    .find(|question| question.domain_code == domain_code)
            })
            .expect("v2 open question")
    }

    #[test]
    fn approved_bank_has_auditable_answers_and_sources() {
        let bank = question_bank().expect("question bank should be valid");
        assert!(!bank.questions.is_empty());
        assert!(bank.questions.iter().all(|question| {
            !question.reference_answer.is_empty()
                && !question.sources.is_empty()
                && question.review.decision == "approved_for_internal_pilot"
                && question.review.curriculum_review_required
        }));
    }

    #[test]
    fn current_bank_is_v13_and_matches_the_learning_framework() {
        let bank = question_bank().expect("question bank should be valid");
        assert_eq!(bank.version, "ecommerce_diagnostic_v13");
        assert_eq!(bank.framework_version, "ecommerce_v1");
        assert_eq!(bank.source_revision, "2026-09-17");
        assert!(choose(&bank, "content").is_some());
        assert!(choose(&bank, "compliance").is_some());
        assert!(choose(&bank, "product").is_some());
        assert!(choose(&bank, "market").is_some());
        assert!(choose(&bank, "service").is_some());
    }

    #[test]
    fn v11_price_objection_question_has_auditable_answer_and_source() {
        let bank = question_bank().expect("question bank should be valid");
        let question = bank
            .questions
            .iter()
            .find(|question| question.id == "service.price_objection_needs.choice.011")
            .expect("price-objection question");

        assert_eq!(
            question.correct_choice_id.as_deref(),
            Some("ask_need_use_problem_price")
        );
        assert!(question.reference_answer.contains("为什么需要"));
        assert!(question.reference_answer.contains("愿意支付的价格"));
        assert!(question.sources.iter().any(|source| {
            source.publisher == "Australian Government"
                && source.url
                    == "https://business.gov.au/marketing-and-advertising/do-market-research"
        }));
        assert_eq!(
            assessment_angle_code(&question.id, question.version),
            Some("service.price_objection_needs")
        );
    }

    #[test]
    fn current_bank_has_expected_reviewed_choice_questions_per_domain() {
        let bank = question_bank().expect("question bank should be valid");
        assert_eq!(bank.questions.len(), CURRENT_QUESTION_BANK_SIZE);
        for domain in ECOMMERCE_DOMAIN_CODES {
            let expected_count = expected_question_count_for_domain(domain);
            let questions = bank
                .questions
                .iter()
                .filter(|question| question.domain_code == domain)
                .collect::<Vec<_>>();
            assert_eq!(
                questions.len(),
                expected_count,
                "unexpected coverage for {domain}"
            );
            assert!(questions.iter().all(|question| {
                question.question_type == "single_choice"
                    && question.choices.len() == 4
                    && question.correct_choice_id.is_some()
            }));
        }
    }

    #[test]
    fn v13_choices_use_explicit_keys_and_keep_historical_v12_available() {
        let additions: QuestionBank = serde_json::from_str(QUESTION_BANK_V13_JSON).unwrap();
        assert_eq!(additions.questions.len(), 14);
        for question in &additions.questions {
            assert!(question.reference_answer.chars().count() >= 100);
            assert!(assessment_angle_code(&question.id, question.version).is_some());
            assert!(question_applicability_scope(&question.id, question.version).is_some());
            assert!(question.review.curriculum_review_required);
            for choice in &question.choices {
                let result = evaluate_choice_id(question, &choice.id).unwrap();
                let expected = if question.correct_choice_id.as_deref() == Some(&choice.id) {
                    "correct"
                } else {
                    "incorrect"
                };
                assert_eq!(result.outcome, expected, "{} {}", question.id, choice.id);
            }
        }
        let historical = question_bank_catalog()
            .unwrap()
            .into_iter()
            .find(|bank| bank.version == "ecommerce_diagnostic_v12")
            .unwrap();
        assert_eq!(historical.questions.len(), 86);
        assert!(
            historical
                .questions
                .iter()
                .all(|question| { !additions.questions.iter().any(|new| new.id == question.id) })
        );
    }

    #[test]
    fn v13_pricing_answer_keys_match_independent_arithmetic() {
        let bank = question_bank().unwrap();
        let inclusive = bank
            .questions
            .iter()
            .find(|question| question.id == "finance.cost_plus_inclusive.choice.011")
            .unwrap();
        assert_eq!((40 + 10 + 10) * 120 / 100, 72);
        assert_eq!(inclusive.correct_choice_id.as_deref(), Some("b"));
        assert!(inclusive.reference_answer.contains("72元"));
        let margin = bank
            .questions
            .iter()
            .find(|question| question.id == "finance.cost_plus_margin_difference.choice.012")
            .unwrap();
        let price = 80 * 125 / 100;
        assert_eq!(price, 100);
        assert_eq!((price - 80) * 100 / price, 20);
        assert_eq!(margin.correct_choice_id.as_deref(), Some("a"));
        assert!(margin.reference_answer.contains("20%"));
    }

    #[test]
    fn current_bank_has_complete_bounded_assessment_angle_coverage() {
        let bank = question_bank().expect("question bank should be valid");
        assert_eq!(
            assessment_angle_depth(&bank).expect("assessment angle depth"),
            (67, 44)
        );
        let catalog: AssessmentAngleCatalog =
            serde_json::from_str(ASSESSMENT_ANGLE_CATALOG_JSON).expect("assessment angle catalog");
        assert_eq!(catalog.mappings.len(), CURRENT_QUESTION_BANK_SIZE);
        assert_eq!(
            catalog
                .mappings
                .iter()
                .map(|mapping| mapping.angle_code.as_str())
                .collect::<HashSet<_>>()
                .len(),
            CURRENT_ASSESSMENT_ANGLE_COUNT
        );
        for question in &bank.questions {
            assert!(assessment_angle_code(&question.id, question.version).is_some());
        }

        let mut angle_counts = HashMap::<&str, usize>::new();
        let mut angles_by_domain = HashMap::<&str, HashSet<&str>>::new();
        for mapping in &catalog.mappings {
            *angle_counts.entry(mapping.angle_code.as_str()).or_default() += 1;
            angles_by_domain
                .entry(mapping.domain_code.as_str())
                .or_default()
                .insert(mapping.angle_code.as_str());
        }
        assert!(
            angle_counts
                .values()
                .all(|count| *count <= CURRENT_MAX_QUESTIONS_PER_ANGLE)
        );
        for domain in ECOMMERCE_DOMAIN_CODES {
            assert!(
                angles_by_domain
                    .get(domain)
                    .is_some_and(|angles| angles.len() >= CURRENT_MIN_ANGLE_COUNT_PER_DOMAIN),
                "insufficient assessment angles for {domain}"
            );
        }
    }

    #[test]
    fn current_bank_has_complete_machine_readable_applicability_scopes() {
        let bank = question_bank().expect("question bank should be valid");
        let catalog: ApplicabilityScopeCatalog =
            serde_json::from_str(APPLICABILITY_SCOPE_CATALOG_JSON)
                .expect("applicability scope catalog");

        assert_eq!(catalog.mappings.len(), CURRENT_QUESTION_BANK_SIZE);
        let jurisdiction_counts =
            catalog
                .mappings
                .iter()
                .fold(HashMap::<&str, usize>::new(), |mut counts, mapping| {
                    *counts
                        .entry(mapping.jurisdiction_code.as_str())
                        .or_default() += 1;
                    counts
                });
        assert_eq!(jurisdiction_counts.get("not_region_specific"), Some(&78));
        assert_eq!(jurisdiction_counts.get("cn"), Some(&9));
        assert_eq!(jurisdiction_counts.get("eu"), Some(&11));
        assert_eq!(jurisdiction_counts.get("us"), Some(&2));

        for domain in ECOMMERCE_DOMAIN_CODES {
            let expected_stage = business_stage_for_domain(domain).expect("known domain stage");
            let expected_count = expected_question_count_for_domain(domain);
            assert_eq!(
                catalog
                    .mappings
                    .iter()
                    .filter(|mapping| mapping.business_stage_code == expected_stage)
                    .count(),
                expected_count
            );
        }
        for question in &bank.questions {
            assert!(question_applicability_scope(&question.id, question.version).is_some());
        }

        let ga4 = question_applicability_scope("conversion.ga4_begin_checkout.choice.003", 1)
            .expect("GA4 applicability");
        assert_eq!(ga4.jurisdiction_code, "not_region_specific");
        assert_eq!(ga4.platform_code, "google_analytics_4");
        assert_eq!(ga4.business_stage_code, "conversion_optimization");
        assert_eq!(
            question_applicability_scope("content.cn_review_integrity.choice.010", 1)
                .expect("China applicability")
                .jurisdiction_code,
            "cn"
        );
        assert_eq!(
            question_applicability_scope("service.eu_legal_guarantee.choice.002", 1)
                .expect("EU applicability")
                .jurisdiction_code,
            "eu"
        );
        assert_eq!(
            question_applicability_scope("service.ftc_shipping_delay.choice.003", 1)
                .expect("US applicability")
                .jurisdiction_code,
            "us"
        );
    }

    #[test]
    fn applicability_scope_gate_rejects_metadata_coverage_and_enum_regressions() {
        let bank = question_bank().expect("question bank should be valid");
        let catalog: ApplicabilityScopeCatalog =
            serde_json::from_str(APPLICABILITY_SCOPE_CATALOG_JSON)
                .expect("applicability scope catalog");

        let mut wrong_version = catalog.clone();
        wrong_version.question_bank_version = "ecommerce_diagnostic_v8".to_owned();
        assert!(validate_applicability_scope_catalog_data(&bank, &wrong_version).is_err());

        let mut missing = catalog.clone();
        missing.mappings.pop();
        assert!(validate_applicability_scope_catalog_data(&bank, &missing).is_err());

        let mut duplicated = catalog.clone();
        duplicated.mappings.push(duplicated.mappings[0].clone());
        assert!(validate_applicability_scope_catalog_data(&bank, &duplicated).is_err());

        let mut unknown_platform = catalog.clone();
        unknown_platform.mappings[0].platform_code = "unknown_platform".to_owned();
        assert!(validate_applicability_scope_catalog_data(&bank, &unknown_platform).is_err());

        let mut wrong_stage = catalog.clone();
        wrong_stage.mappings[0].business_stage_code = "product_operations".to_owned();
        assert!(validate_applicability_scope_catalog_data(&bank, &wrong_stage).is_err());

        let mut wrong_jurisdiction = catalog;
        let cn_mapping = wrong_jurisdiction
            .mappings
            .iter_mut()
            .find(|mapping| mapping.question_id.contains(".cn_"))
            .expect("China mapping");
        cn_mapping.jurisdiction_code = "not_region_specific".to_owned();
        assert!(validate_applicability_scope_catalog_data(&bank, &wrong_jurisdiction).is_err());
    }

    #[test]
    fn current_bank_runtime_gate_rejects_size_coverage_and_composition_regressions() {
        let bank = question_bank().expect("question bank should be valid");

        let mut missing_question = bank.clone();
        missing_question.questions.pop();
        assert!(validate_question_bank(&missing_question).is_err());

        let mut uneven_coverage = bank.clone();
        let market_question = uneven_coverage
            .questions
            .iter_mut()
            .find(|question| question.domain_code == "market")
            .expect("market question");
        market_question.domain_code = "product".to_owned();
        assert!(validate_question_bank(&uneven_coverage).is_err());

        let mut draft_question = bank;
        draft_question.questions[0].status = "draft".to_owned();
        assert!(validate_question_bank(&draft_question).is_err());
    }

    #[test]
    fn delivery_modes_fail_closed_and_keep_internal_pilot_available() {
        let bank = question_bank().expect("question bank should be valid");
        let domain = "market";
        let seed = "delivery-mode-seed";

        assert!(
            deliverable_question_for_domain(
                &bank,
                LearningQuestionDeliveryMode::Disabled,
                domain,
                seed,
                None,
                &[],
            )
            .expect("disabled delivery mode should be valid")
            .is_none()
        );
        assert!(
            deliverable_question_for_domain(
                &bank,
                LearningQuestionDeliveryMode::InternalPilot,
                domain,
                seed,
                None,
                &[],
            )
            .expect("internal pilot delivery mode should be valid")
            .is_some()
        );
        assert!(
            deliverable_question_for_domain(
                &bank,
                LearningQuestionDeliveryMode::Production,
                domain,
                seed,
                None,
                &[],
            )
            .expect("production delivery mode should be valid")
            .is_none()
        );
        assert!(
            production_approved_question_identities(&bank)
                .expect("production release should be valid")
                .is_empty()
        );
    }

    #[test]
    fn existing_unanswered_question_follows_delivery_mode_without_breaking_history_lookup() {
        let bank = question_bank().expect("question bank should be valid");
        let question = deliverable_question_for_domain(
            &bank,
            LearningQuestionDeliveryMode::InternalPilot,
            "conversion",
            "existing-question-seed",
            None,
            &[],
        )
        .expect("internal-pilot selection should be valid")
        .expect("internal-pilot question");

        assert!(
            question_is_deliverable_for_mode(
                &bank,
                &question,
                LearningQuestionDeliveryMode::InternalPilot,
            )
            .expect("internal-pilot delivery check")
        );
        assert!(
            !question_is_deliverable_for_mode(
                &bank,
                &question,
                LearningQuestionDeliveryMode::Disabled,
            )
            .expect("disabled delivery check")
        );
        assert!(
            !question_is_deliverable_for_mode(
                &bank,
                &question,
                LearningQuestionDeliveryMode::Production,
            )
            .expect("production delivery check")
        );

        let historical = question_for_probe(&json!({
            "questionBankVersion": "ecommerce_diagnostic_v6",
            "sourceRevision": "2026-08-27",
            "questionId": "conversion.ga4_begin_checkout.choice.003",
            "questionVersion": 1,
            "rubricVersion": 1
        }))
        .expect("historical lookup should remain valid");
        assert!(historical.is_some());
    }

    #[test]
    fn delivery_mode_uses_stable_config_codes() {
        assert_eq!(
            serde_json::from_str::<LearningQuestionDeliveryMode>("\"disabled\"")
                .expect("disabled mode"),
            LearningQuestionDeliveryMode::Disabled
        );
        assert_eq!(
            serde_json::from_str::<LearningQuestionDeliveryMode>("\"internal_pilot\"")
                .expect("internal pilot mode"),
            LearningQuestionDeliveryMode::InternalPilot
        );
        assert_eq!(
            serde_json::from_str::<LearningQuestionDeliveryMode>("\"production\"")
                .expect("production mode"),
            LearningQuestionDeliveryMode::Production
        );
        assert!(serde_json::from_str::<LearningQuestionDeliveryMode>("\"内部灰度\"").is_err());
    }

    fn valid_production_approval(question: &DiagnosticQuestion) -> ProductionQuestionApproval {
        ProductionQuestionApproval {
            question_id: question.id.clone(),
            question_version: question.version,
            curriculum_review: ProductionCurriculumReview {
                reviewer_id: "employee.curriculum.001".to_owned(),
                reviewer_role: CURRICULUM_REVIEWER_ROLE.to_owned(),
                reviewed_at: "2026-08-30".to_owned(),
                evidence_id: "curriculum-review-v10-001".to_owned(),
                checks: ProductionCurriculumChecks {
                    answer: REVIEW_CHECK_PASSED.to_owned(),
                    distractors: REVIEW_CHECK_PASSED.to_owned(),
                    explanation: REVIEW_CHECK_PASSED.to_owned(),
                    difficulty_and_audience: REVIEW_CHECK_PASSED.to_owned(),
                    applicability: REVIEW_CHECK_PASSED.to_owned(),
                    source_currency: REVIEW_CHECK_PASSED.to_owned(),
                },
                decision: CURRICULUM_RELEASE_DECISION.to_owned(),
            },
            release_approval: ProductionReleaseApproval {
                approver_id: "employee.product.001".to_owned(),
                approver_role: PRODUCT_APPROVER_ROLE.to_owned(),
                approved_at: "2026-08-30".to_owned(),
                decision: PRODUCTION_RELEASE_DECISION.to_owned(),
            },
        }
    }

    #[test]
    fn production_release_requires_two_named_separated_complete_approvals() {
        let bank = question_bank().expect("question bank should be valid");
        let question = bank.questions.first().expect("current question");
        let approval = valid_production_approval(question);
        let release = QuestionBankProductionRelease {
            schema_version: PRODUCTION_RELEASE_SCHEMA_VERSION,
            question_bank_version: bank.version.clone(),
            approvals: vec![approval.clone()],
        };
        let approved = validate_production_release(&bank, &release)
            .expect("named production approval should be valid");
        assert_eq!(approved.len(), 1);
        assert!(approved.contains(&(question.id.clone(), question.version)));

        let mut automated = release.clone();
        automated.approvals[0].curriculum_review.reviewer_id =
            "automated_curriculum_gate".to_owned();
        assert!(validate_production_release(&bank, &automated).is_err());

        let mut failed_check = release.clone();
        failed_check.approvals[0].curriculum_review.checks.answer = "failed".to_owned();
        assert!(validate_production_release(&bank, &failed_check).is_err());

        let mut missing_evidence = release.clone();
        missing_evidence.approvals[0].curriculum_review.evidence_id = " ".to_owned();
        assert!(validate_production_release(&bank, &missing_evidence).is_err());

        let mut wrong_role = release.clone();
        wrong_role.approvals[0].curriculum_review.reviewer_role = "teacher".to_owned();
        assert!(validate_production_release(&bank, &wrong_role).is_err());

        let mut wrong_curriculum_decision = release.clone();
        wrong_curriculum_decision.approvals[0]
            .curriculum_review
            .decision = "approved".to_owned();
        assert!(validate_production_release(&bank, &wrong_curriculum_decision).is_err());

        let mut author_review = release.clone();
        author_review.approvals[0].curriculum_review.reviewer_id =
            question.review.author_id.clone();
        assert!(validate_production_release(&bank, &author_review).is_err());

        let mut stale_review = release.clone();
        stale_review.approvals[0].curriculum_review.reviewed_at = "2026-08-01".to_owned();
        assert!(validate_production_release(&bank, &stale_review).is_err());

        let mut same_person = release.clone();
        let reviewer_id = same_person.approvals[0]
            .curriculum_review
            .reviewer_id
            .clone();
        same_person.approvals[0].release_approval.approver_id = reviewer_id;
        assert!(validate_production_release(&bank, &same_person).is_err());

        let mut wrong_product_role = release.clone();
        wrong_product_role.approvals[0]
            .release_approval
            .approver_role = "product_manager".to_owned();
        assert!(validate_production_release(&bank, &wrong_product_role).is_err());

        let mut wrong_release_decision = release.clone();
        wrong_release_decision.approvals[0]
            .release_approval
            .decision = "approved".to_owned();
        assert!(validate_production_release(&bank, &wrong_release_decision).is_err());

        let mut automated_product = release.clone();
        automated_product.approvals[0].release_approval.approver_id =
            "automated_product_gate".to_owned();
        assert!(validate_production_release(&bank, &automated_product).is_err());

        let mut early_release = release.clone();
        early_release.approvals[0].release_approval.approved_at = "2026-08-01".to_owned();
        assert!(validate_production_release(&bank, &early_release).is_err());

        let mut old_schema = release.clone();
        old_schema.schema_version = 1;
        assert!(validate_production_release(&bank, &old_schema).is_err());

        let mut future_review = release.clone();
        future_review.approvals[0].curriculum_review.reviewed_at = "2999-01-01".to_owned();
        future_review.approvals[0].release_approval.approved_at = "2999-01-01".to_owned();
        assert!(validate_production_release(&bank, &future_review).is_err());

        let legacy_flat = json!({
            "schemaVersion": PRODUCTION_RELEASE_SCHEMA_VERSION,
            "questionBankVersion": bank.version.clone(),
            "approvals": [{
                "questionId": question.id.clone(),
                "questionVersion": question.version,
                "reviewerId": "employee.curriculum.001",
                "reviewedAt": "2026-08-30",
                "decision": "approved_for_production"
            }]
        });
        assert!(serde_json::from_value::<QuestionBankProductionRelease>(legacy_flat).is_err());

        let mut unknown = release.clone();
        unknown.approvals[0].question_id = "unknown.question".to_owned();
        assert!(validate_production_release(&bank, &unknown).is_err());

        let mut duplicated = release;
        duplicated.approvals.push(approval);
        assert!(validate_production_release(&bank, &duplicated).is_err());
    }

    #[test]
    fn production_release_manifest_contains_no_answer_material() {
        let release: Value = serde_json::from_str(QUESTION_BANK_PRODUCTION_RELEASE_JSON)
            .expect("production release JSON");
        let serialized = release.to_string();
        assert!(!serialized.contains("correctChoiceId"));
        assert!(!serialized.contains("referenceAnswer"));
        assert!(!serialized.contains("rubric"));
    }

    #[test]
    fn current_bank_sources_have_auditable_usage_metadata() {
        let bank = question_bank().expect("question bank should be valid");
        assert!(
            bank.questions
                .iter()
                .flat_map(|question| &question.sources)
                .all(|source| {
                    source.license_type.is_some()
                        && source.use_mode.is_some()
                        && source
                            .license_url
                            .as_deref()
                            .is_some_and(|url| url.starts_with("https://"))
                })
        );
    }

    #[test]
    fn selection_is_stable_and_excludes_the_most_recent_question() {
        let bank = question_bank().expect("question bank should be valid");
        let first = approved_question_for_domain(&bank, "finance", "assistant-a", None, &[])
            .expect("finance question");
        let repeated = approved_question_for_domain(&bank, "finance", "assistant-a", None, &[])
            .expect("stable finance question");
        let next =
            approved_question_for_domain(&bank, "finance", "assistant-a", Some(&first.id), &[])
                .expect("alternate finance question");

        assert_eq!(first.id, repeated.id);
        assert_ne!(first.id, next.id);
    }

    #[test]
    fn selection_prefers_the_requested_assessment_gap_questions() {
        let bank = question_bank().expect("question bank should be valid");
        let finance_ids = bank
            .questions
            .iter()
            .filter(|question| question.domain_code == "finance")
            .map(|question| question.id.clone())
            .collect::<Vec<_>>();
        let preferred = vec![finance_ids[1].clone()];

        let selected =
            approved_question_for_domain(&bank, "finance", "assistant-a", None, &preferred)
                .expect("preferred finance question");

        assert_eq!(selected.id, preferred[0]);
    }

    #[test]
    fn selection_does_not_fallback_when_requested_candidates_are_unavailable() {
        let bank = question_bank().expect("question bank should be valid");
        let unavailable = vec!["compliance.unavailable.choice.999".to_owned()];

        assert!(
            approved_question_for_domain(
                &bank,
                "compliance",
                "assistant-context",
                None,
                &unavailable,
            )
            .is_none()
        );
    }

    #[test]
    fn single_choice_is_scored_by_server_only_answer_key() {
        let bank = question_bank().expect("question bank should be valid");
        let question = choose(&bank, "finance").expect("finance question");
        let correct = question
            .choices
            .iter()
            .find(|choice| Some(choice.id.as_str()) == question.correct_choice_id.as_deref())
            .expect("correct choice");
        let incorrect = question
            .choices
            .iter()
            .find(|choice| Some(choice.id.as_str()) != question.correct_choice_id.as_deref())
            .expect("incorrect choice");

        assert_eq!(
            evaluate_answer(&question, &format!("我的选择是：{}", correct.text)).outcome,
            "correct"
        );
        assert_eq!(
            evaluate_answer(&question, &incorrect.text).outcome,
            "incorrect"
        );
        assert_eq!(
            evaluate_answer(&question, "我还没想好").outcome,
            "unscorable"
        );
    }

    #[test]
    fn structured_choice_is_scored_by_stable_id_and_rejects_unknown_ids() {
        let bank = question_bank().expect("question bank should be valid");
        let question = choose(&bank, "finance").expect("finance question");
        let correct_id = question
            .correct_choice_id
            .as_deref()
            .expect("correct choice id");
        let incorrect_id = question
            .choices
            .iter()
            .find(|choice| choice.id != correct_id)
            .expect("incorrect choice")
            .id
            .as_str();

        assert_eq!(
            evaluate_choice_id(&question, correct_id)
                .expect("correct structured choice")
                .outcome,
            "correct"
        );
        assert_eq!(
            evaluate_choice_id(&question, incorrect_id)
                .expect("incorrect structured choice")
                .outcome,
            "incorrect"
        );
        assert!(evaluate_choice_id(&question, "unknown-choice").is_err());
    }

    #[test]
    fn all_current_choice_answer_keys_score_deterministically() {
        let bank = question_bank().expect("question bank should be valid");
        for question in &bank.questions {
            let correct_id = question
                .correct_choice_id
                .as_deref()
                .expect("approved single-choice question must have an answer key");
            for choice in &question.choices {
                let assessment = evaluate_choice_id(question, &choice.id)
                    .expect("every published choice id should be accepted");
                let expected = if choice.id == correct_id {
                    "correct"
                } else {
                    "incorrect"
                };
                assert_eq!(
                    assessment.outcome, expected,
                    "unexpected outcome for question {} choice {}",
                    question.id, choice.id
                );
                assert_eq!(
                    assessment
                        .as_json(&bank, question)
                        .get("assessmentAngleCode")
                        .and_then(Value::as_str),
                    assessment_angle_code(&question.id, question.version)
                );
            }
        }
    }

    #[test]
    fn v10_product_title_and_highlight_answers_are_source_bounded() {
        let bank = question_bank().expect("question bank should be valid");
        let title = bank
            .questions
            .iter()
            .find(|question| question.id == "product.title_clarity.choice.011")
            .expect("title question");
        let highlight = bank
            .questions
            .iter()
            .find(|question| question.id == "product.highlight_focus.choice.012")
            .expect("highlight question");

        assert_eq!(
            title.correct_choice_id.as_deref(),
            Some("describe_product_and_variant_clearly")
        );
        assert_eq!(
            highlight.correct_choice_id.as_deref(),
            Some("focus_on_product_with_professional_language")
        );
        assert_eq!(
            title.sources[0].source_id,
            "google.merchant.title_specification"
        );
        assert_eq!(
            highlight.sources[0].source_id,
            "google.merchant.product_highlight_specification"
        );
    }

    #[test]
    fn correct_answer_is_scored_by_rubric_not_length() {
        let question = open_question_v2("finance");
        let assessment = evaluate_answer(
            &question,
            "销售额不等于利润，还要看商品成本以及投放成本等成本和费用。",
        );
        assert_eq!(assessment.outcome, "correct");
        assert_eq!(assessment.direction, "positive");
    }

    #[test]
    fn long_but_unsupported_answer_is_not_positive() {
        let question = open_question_v2("finance");
        let assessment = evaluate_answer(
            &question,
            "这是一个很复杂的问题，我认为需要认真分析很多情况以后才能得到最终答案。",
        );
        assert_eq!(assessment.outcome, "unscorable");
        assert_eq!(assessment.direction, "neutral");
        assert!(!assessment.contributes_to_mastery);
    }

    #[test]
    fn explicit_misconception_is_incorrect() {
        let question = open_question_v2("finance");
        let assessment = evaluate_answer(&question, "收入增长利润一定增长，不需要看费用。");
        assert_eq!(assessment.outcome, "incorrect");
        assert_eq!(assessment.direction, "negative");
    }

    #[test]
    fn negated_correct_keywords_do_not_create_positive_evidence() {
        let question = open_question_v2("conversion");
        let assessment = evaluate_answer(&question, "不需要核对埋点，也不用检查支付。");
        assert_eq!(assessment.outcome, "incorrect");
        assert_eq!(assessment.direction, "negative");
    }

    #[test]
    fn invalid_domain_and_review_date_are_rejected() {
        let mut bank = question_bank().expect("question bank should be valid");
        bank.questions[0].domain_code = "unknown".to_owned();
        assert!(validate_question_bank(&bank).is_err());

        let mut bank = question_bank().expect("question bank should be valid");
        bank.questions[0].review.reviewed_at = "not-a-date".to_owned();
        assert!(validate_question_bank(&bank).is_err());
    }

    #[test]
    fn duplicate_choice_text_and_unknown_answer_key_are_rejected() {
        let mut bank = question_bank().expect("question bank should be valid");
        bank.questions[0].choices[1].text = bank.questions[0].choices[0].text.clone();
        assert!(validate_question_bank(&bank).is_err());

        let mut bank = question_bank().expect("question bank should be valid");
        bank.questions[0].correct_choice_id = Some("missing-choice".to_owned());
        assert!(validate_question_bank(&bank).is_err());
    }

    #[test]
    fn expired_source_stops_new_delivery_but_keeps_historical_lookup() {
        let mut bank = question_bank().expect("question bank should be valid");
        for question in bank
            .questions
            .iter_mut()
            .filter(|question| question.domain_code == "finance")
        {
            question.sources[0].valid_through = "2020-01-01".to_owned();
        }
        assert!(choose(&bank, "finance").is_none());
        let snapshot = json!({
            "questionId": "finance.revenue_profit.001",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        assert!(
            question_for_probe(&snapshot)
                .expect("historical lookup")
                .is_some()
        );
    }

    #[test]
    fn retired_question_is_not_delivered_but_keeps_historical_lookup() {
        let mut bank = question_bank().expect("question bank should be valid");
        let question_id = bank
            .questions
            .iter()
            .find(|question| question.domain_code == "finance")
            .expect("finance question")
            .id
            .clone();
        for question in bank
            .questions
            .iter_mut()
            .filter(|question| question.domain_code == "finance")
        {
            question.status = "retired".to_owned();
        }
        assert!(choose(&bank, "finance").is_none());
        let snapshot = json!({
            "questionBankVersion": bank.version,
            "sourceRevision": bank.source_revision,
            "questionId": question_id,
            "questionVersion": 1,
            "rubricVersion": 1
        });
        assert!(
            question_for_probe(&snapshot)
                .expect("historical lookup")
                .is_some()
        );
    }

    #[test]
    fn v1_snapshot_still_resolves_after_v2_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v1",
            "sourceRevision": "2026-08-25",
            "questionId": "conversion.checkout_dropoff.001",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v1 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v1");
        assert_eq!(question.id, "conversion.checkout_dropoff.001");
    }

    #[test]
    fn v3_snapshot_still_resolves_after_v4_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v3",
            "sourceRevision": "2026-08-27",
            "questionId": "content.informative_image_alt.choice.001",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v3 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v3");
        assert_eq!(question.id, "content.informative_image_alt.choice.001");
    }

    #[test]
    fn v5_snapshot_still_resolves_after_v6_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v5",
            "sourceRevision": "2026-08-27",
            "questionId": "market.trends_relative_index.choice.001",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v5 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v5");
        assert_eq!(question.id, "market.trends_relative_index.choice.001");
    }

    #[test]
    fn v6_snapshot_still_resolves_after_v7_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v6",
            "sourceRevision": "2026-08-27",
            "questionId": "compliance.eu_extra_payment_consent.choice.004",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v6 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v6");
        assert_eq!(
            question.id,
            "compliance.eu_extra_payment_consent.choice.004"
        );
    }

    #[test]
    fn v7_snapshot_still_resolves_after_v10_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v7",
            "sourceRevision": "2026-08-28",
            "questionId": "market.trends_not_sales.choice.005",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v7 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v7");
        assert_eq!(question.id, "market.trends_not_sales.choice.005");
    }

    #[test]
    fn v8_snapshot_still_resolves_after_v10_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v8",
            "sourceRevision": "2026-08-29",
            "questionId": "market.customer_research_mix.choice.007",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v8 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v8");
        assert_eq!(question.id, "market.customer_research_mix.choice.007");
    }

    #[test]
    fn v9_snapshot_still_resolves_after_v10_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v9",
            "sourceRevision": "2026-08-30",
            "questionId": "product.stocktake_action.choice.010",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v9 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v9");
        assert_eq!(question.id, "product.stocktake_action.choice.010");
    }

    #[test]
    fn v10_snapshot_still_resolves_after_v11_release() {
        let snapshot = json!({
            "questionBankVersion": "ecommerce_diagnostic_v10",
            "sourceRevision": "2026-08-31",
            "questionId": "product.title_clarity.choice.011",
            "questionVersion": 1,
            "rubricVersion": 1
        });
        let (bank, question) = question_for_probe(&snapshot)
            .expect("catalog should be valid")
            .expect("v10 question should remain available");
        assert_eq!(bank.version, "ecommerce_diagnostic_v10");
        assert_eq!(question.id, "product.title_clarity.choice.011");
    }

    #[test]
    fn content_answer_requires_meaning_and_non_generic_text() {
        let question = open_question_v2("content");
        let assessment = evaluate_answer(
            &question,
            "要传达图片含义并写出防水、20L等关键卖点，不能只写商品图片。",
        );
        assert_eq!(assessment.outcome, "correct");

        let misconception = evaluate_answer(&question, "替代文本只写商品图片即可。");
        assert_eq!(misconception.outcome, "incorrect");
    }

    #[test]
    fn compliance_answer_requires_general_rule_and_exceptions() {
        let question = open_question_v2("compliance");
        let assessment = evaluate_answer(
            &question,
            "远程合同通常有14天撤回期，但存在例外，例如定制商品、易腐商品和数字内容。",
        );
        assert_eq!(assessment.outcome, "correct");

        let misconception = evaluate_answer(&question, "所有商品都无例外，都能无条件退货。");
        assert_eq!(misconception.outcome, "incorrect");
    }
}
