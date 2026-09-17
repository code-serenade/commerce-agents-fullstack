use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use time::{Date, Month, OffsetDateTime};

use crate::common;

const ANGLE_CATALOG_FILE: &str = "ecommerce-diagnostic-assessment-angles.v4.json";
const SCOPE_CATALOG_FILE: &str = "ecommerce-diagnostic-applicability-scopes.v3.json";
const PRODUCTION_RELEASE_FILE: &str = "ecommerce-diagnostic-question-bank.production-release.json";
const EXPECTED_QUESTION_COUNT: usize = 100;
const EXPECTED_BASE_QUESTION_COUNT_PER_DOMAIN: usize = 10;
const EXPECTED_PRODUCT_QUESTION_COUNT: usize = 21;
const PRODUCTION_RELEASE_SCHEMA_VERSION: u32 = 2;
const CURRICULUM_REVIEWER_ROLE: &str = "ecommerce_curriculum_expert";
const PRODUCT_APPROVER_ROLE: &str = "learning_product_owner";
const CURRICULUM_RELEASE_DECISION: &str = "approved_for_release_review";
const PRODUCTION_RELEASE_DECISION: &str = "approved_for_production";
const REVIEW_CHECK_PASSED: &str = "passed";

const DOMAIN_ORDER: [(&str, &str); 8] = [
    ("market", "市场洞察"),
    ("product", "商品策略"),
    ("content", "内容表达"),
    ("traffic", "流量运营"),
    ("conversion", "转化优化"),
    ("finance", "经营核算"),
    ("service", "客户服务"),
    ("compliance", "合规经营"),
];

type QuestionIdentity = (String, u32);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuestionBank {
    version: String,
    #[serde(default)]
    base_version: Option<String>,
    framework_version: String,
    source_revision: String,
    evaluation_mode: String,
    questions: Vec<Question>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Question {
    id: String,
    version: u32,
    status: String,
    domain_code: String,
    question_type: String,
    prompt: String,
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    correct_choice_id: Option<String>,
    reference_answer: String,
    sources: Vec<Source>,
    review: EngineeringReview,
}

impl Question {
    fn identity(&self) -> QuestionIdentity {
        (self.id.clone(), self.version)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    id: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Source {
    source_id: String,
    authority_level: String,
    publisher: String,
    title: String,
    url: String,
    published_version: String,
    checked_at: String,
    valid_through: String,
    #[serde(default)]
    license_type: Option<String>,
    #[serde(default)]
    use_mode: Option<String>,
    #[serde(default)]
    license_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EngineeringReview {
    author_id: String,
    reviewer_id: String,
    review_scope: String,
    reviewer_type: String,
    curriculum_review_required: bool,
    reviewed_at: String,
    decision: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssessmentAngleCatalog {
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

impl AssessmentAngleMapping {
    fn identity(&self) -> QuestionIdentity {
        (self.question_id.clone(), self.question_version)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplicabilityScopeCatalog {
    catalog_version: String,
    question_bank_version: String,
    mappings: Vec<ApplicabilityScope>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplicabilityScope {
    question_id: String,
    question_version: u32,
    jurisdiction_code: String,
    platform_code: String,
    business_stage_code: String,
}

impl ApplicabilityScope {
    fn identity(&self) -> QuestionIdentity {
        (self.question_id.clone(), self.question_version)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionRelease {
    schema_version: u32,
    question_bank_version: String,
    approvals: Vec<ProductionApproval>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionApproval {
    question_id: String,
    question_version: u32,
    curriculum_review: ProductionCurriculumReview,
    release_approval: ProductReleaseApproval,
}

impl ProductionApproval {
    fn identity(&self) -> QuestionIdentity {
        (self.question_id.clone(), self.question_version)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProductionCurriculumReview {
    reviewer_id: String,
    reviewer_role: String,
    reviewed_at: String,
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
struct ProductReleaseApproval {
    approver_id: String,
    approver_role: String,
    approved_at: String,
    decision: String,
}

struct ReviewPackData {
    bank: QuestionBank,
    angle_catalog: AssessmentAngleCatalog,
    angles: HashMap<QuestionIdentity, AssessmentAngleMapping>,
    scope_catalog: ApplicabilityScopeCatalog,
    scopes: HashMap<QuestionIdentity, ApplicabilityScope>,
    production_approvals: HashSet<QuestionIdentity>,
}

pub fn generate(output: Option<&Path>) -> Result<()> {
    let backend_root = common::project_root()?;
    let data = load_review_pack_data(&backend_root)?;
    let repo_root = repository_root(&backend_root)?;
    let output_path = resolve_output_path(repo_root, output, &data.bank);
    let rendered = render_review_pack(&data)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建审核参考包目录 {} 失败", parent.display()))?;
    }
    fs::write(&output_path, rendered)
        .with_context(|| format!("写入审核参考包 {} 失败", output_path.display()))?;
    println!("已生成内部教研审核参考包：{}", output_path.display());
    println!("注意：该文件包含隐藏答案材料，不得进入学生 API 或前端资源。");
    Ok(())
}

pub fn check(output: Option<&Path>) -> Result<()> {
    let backend_root = common::project_root()?;
    let data = load_review_pack_data(&backend_root)?;
    let repo_root = repository_root(&backend_root)?;
    let output_path = resolve_output_path(repo_root, output, &data.bank);
    let expected = render_review_pack(&data)?;
    let actual = fs::read_to_string(&output_path)
        .with_context(|| format!("读取审核参考包 {} 失败", output_path.display()))?;

    ensure!(
        actual == expected,
        "审核参考包与当前不可变题库不一致，请运行 cargo xtask learning-question-review-pack generate"
    );
    println!(
        "审核参考包一致性检查通过：{} 道题，{}",
        data.bank.questions.len(),
        output_path.display()
    );
    Ok(())
}

fn load_review_pack_data(backend_root: &Path) -> Result<ReviewPackData> {
    let learning_dir = backend_root.join("config/json/learning");
    let angle_catalog: AssessmentAngleCatalog = read_json(&learning_dir.join(ANGLE_CATALOG_FILE))?;
    let scope_catalog: ApplicabilityScopeCatalog =
        read_json(&learning_dir.join(SCOPE_CATALOG_FILE))?;
    let release: ProductionRelease = read_json(&learning_dir.join(PRODUCTION_RELEASE_FILE))?;
    let banks = load_question_banks(&learning_dir)?;
    let bank = compose_bank(&angle_catalog.question_bank_version, &banks)?;

    validate_bank_metadata(&bank, &angle_catalog, &scope_catalog, &release)?;
    let identities = validate_questions(&bank)?;
    let angles = unique_mappings(
        &angle_catalog.mappings,
        AssessmentAngleMapping::identity,
        "课程角度",
    )?;
    let scopes = unique_mappings(
        &scope_catalog.mappings,
        ApplicabilityScope::identity,
        "适用范围",
    )?;
    ensure_identity_coverage(&identities, angles.keys(), "课程角度")?;
    ensure_identity_coverage(&identities, scopes.keys(), "适用范围")?;
    for question in &bank.questions {
        let angle = angles
            .get(&question.identity())
            .context("题目缺少课程角度")?;
        ensure!(
            angle.domain_code == question.domain_code,
            "题目 {} 的课程角度领域不一致",
            question.id
        );
    }

    let production_approvals = validate_production_approvals(&bank, &release)?;
    ensure!(
        production_approvals.is_subset(&identities),
        "生产批准身份校验异常"
    );

    Ok(ReviewPackData {
        bank,
        angle_catalog,
        angles,
        scope_catalog,
        scopes,
        production_approvals,
    })
}

fn validate_production_approvals(
    bank: &QuestionBank,
    release: &ProductionRelease,
) -> Result<HashSet<QuestionIdentity>> {
    ensure!(
        release.schema_version == PRODUCTION_RELEASE_SCHEMA_VERSION
            && release.question_bank_version == bank.version,
        "生产批准清单 schema 或题库版本不正确"
    );
    let today = OffsetDateTime::now_utc().date();
    let mut identities = HashSet::new();
    for approval in &release.approvals {
        let question = bank
            .questions
            .iter()
            .find(|question| question.identity() == approval.identity())
            .with_context(|| {
                format!(
                    "生产批准包含未知题目：{} v{}",
                    approval.question_id, approval.question_version
                )
            })?;
        ensure!(
            production_approval_is_valid(question, approval, today),
            "题目 {} 的双人逐项生产签核无效",
            question.id
        );
        ensure!(
            identities.insert(approval.identity()),
            "生产批准清单包含重复题目身份：{} v{}",
            approval.question_id,
            approval.question_version
        );
    }
    Ok(identities)
}

fn production_approval_is_valid(
    question: &Question,
    approval: &ProductionApproval,
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

fn latest_source_checked_at(question: &Question) -> Option<Date> {
    question
        .sources
        .iter()
        .map(|source| parse_date(&source.checked_at))
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .max()
}

fn load_question_banks(learning_dir: &Path) -> Result<HashMap<String, QuestionBank>> {
    let mut banks = HashMap::new();
    for entry in fs::read_dir(learning_dir)
        .with_context(|| format!("读取题库目录 {} 失败", learning_dir.display()))?
    {
        let path = entry?.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("ecommerce-diagnostic-question-bank.v")
            || !file_name.ends_with(".json")
        {
            continue;
        }
        let bank: QuestionBank = read_json(&path)?;
        ensure!(
            banks.insert(bank.version.clone(), bank).is_none(),
            "题库版本重复：{file_name}"
        );
    }
    ensure!(!banks.is_empty(), "没有找到电商诊断题库版本文件");
    Ok(banks)
}

fn compose_bank(version: &str, banks: &HashMap<String, QuestionBank>) -> Result<QuestionBank> {
    compose_bank_inner(version, banks, &mut HashSet::new())
}

fn compose_bank_inner(
    version: &str,
    banks: &HashMap<String, QuestionBank>,
    resolving: &mut HashSet<String>,
) -> Result<QuestionBank> {
    ensure!(
        resolving.insert(version.to_owned()),
        "题库 baseVersion 存在循环：{version}"
    );
    let mut bank = banks
        .get(version)
        .with_context(|| format!("题库版本不存在：{version}"))?
        .clone();

    if let Some(base_version) = bank.base_version.clone() {
        let mut base = compose_bank_inner(&base_version, banks, resolving)?;
        ensure!(
            base.framework_version == bank.framework_version
                && base.evaluation_mode == bank.evaluation_mode,
            "题库 {version} 与基础版本 {base_version} 元数据不兼容"
        );
        for question in &mut base.questions {
            for source in &mut question.sources {
                source.license_type = Some("official_reference_only".to_owned());
                source.use_mode = Some("source_based_original".to_owned());
                source.license_url = Some(source.url.clone());
            }
        }
        base.questions.append(&mut bank.questions);
        bank.questions = base.questions;
    }
    resolving.remove(version);
    Ok(bank)
}

fn validate_bank_metadata(
    bank: &QuestionBank,
    angles: &AssessmentAngleCatalog,
    scopes: &ApplicabilityScopeCatalog,
    release: &ProductionRelease,
) -> Result<()> {
    ensure!(
        bank.version == angles.question_bank_version
            && bank.version == scopes.question_bank_version
            && bank.version == release.question_bank_version,
        "题库、角度、适用范围或生产清单版本不一致"
    );
    ensure!(
        bank.framework_version == angles.framework_version,
        "题库与课程角度框架版本不一致"
    );
    ensure!(
        bank.evaluation_mode == "deterministic_rubric_v1",
        "当前题库评价模式不是确定性量规"
    );
    ensure!(
        bank.questions.len() == EXPECTED_QUESTION_COUNT,
        "当前合成题库应为 {EXPECTED_QUESTION_COUNT} 道，实际为 {} 道",
        bank.questions.len()
    );
    Ok(())
}

fn validate_questions(bank: &QuestionBank) -> Result<HashSet<QuestionIdentity>> {
    let mut identities = HashSet::new();
    for question in &bank.questions {
        ensure!(
            identities.insert(question.identity()),
            "题目身份重复：{} v{}",
            question.id,
            question.version
        );
        ensure!(
            question.status == "approved"
                && question.question_type == "single_choice"
                && question.choices.len() == 4,
            "题目 {} 不是当前合规的四选一内部灰度题",
            question.id
        );
        let choice_ids = question
            .choices
            .iter()
            .map(|choice| choice.id.as_str())
            .collect::<HashSet<_>>();
        ensure!(
            choice_ids.len() == question.choices.len(),
            "题目 {} 包含重复选项 ID",
            question.id
        );
        ensure!(
            question
                .choices
                .iter()
                .filter(|choice| {
                    Some(choice.id.as_str()) == question.correct_choice_id.as_deref()
                })
                .count()
                == 1,
            "题目 {} 的隐藏答案键不能唯一定位选项",
            question.id
        );
        ensure!(
            question.review.curriculum_review_required,
            "题目 {} 未声明需要公司教研审核",
            question.id
        );
    }
    Ok(identities)
}

fn unique_mappings<T, F>(
    mappings: &[T],
    identity: F,
    label: &str,
) -> Result<HashMap<QuestionIdentity, T>>
where
    T: Clone,
    F: Fn(&T) -> QuestionIdentity,
{
    let mut result = HashMap::new();
    for mapping in mappings {
        let key = identity(mapping);
        ensure!(
            result.insert(key.clone(), mapping.clone()).is_none(),
            "{label}包含重复题目身份：{} v{}",
            key.0,
            key.1
        );
    }
    Ok(result)
}

fn ensure_identity_coverage<'a>(
    questions: &HashSet<QuestionIdentity>,
    mappings: impl Iterator<Item = &'a QuestionIdentity>,
    label: &str,
) -> Result<()> {
    let mappings = mappings.cloned().collect::<HashSet<_>>();
    if &mappings != questions {
        let mut missing = questions.difference(&mappings).cloned().collect::<Vec<_>>();
        let mut extra = mappings.difference(questions).cloned().collect::<Vec<_>>();
        missing.sort();
        extra.sort();
        bail!("{label}身份覆盖不完整，缺失={missing:?}，多余={extra:?}");
    }
    Ok(())
}

fn render_review_pack(data: &ReviewPackData) -> Result<String> {
    let mut output = String::new();
    output.push_str("# 电商诊断题库内部教研审核参考包\n\n");
    output.push_str("> 内部敏感材料：包含服务端隐藏答案。不得放入学生 API、前端静态资源、公开文档或运行日志。\n\n");
    output.push_str("## 使用边界\n\n");
    output.push_str("- 本文件由不可变运行题库、课程角度目录、适用范围目录和生产批准清单确定性生成，不是第二份判分事实。\n");
    output.push_str("- 本文件只帮助公司具名教研逐题查看材料；审核人、岗位、时间和结论请填写同版本教研实名审核清单。\n");
    output.push_str("- 自动工程门禁只能证明结构和材料一致，不能证明拟定答案已经通过公司教研，也不能形成生产批准。\n\n");
    output.push_str("## 目录摘要\n\n");
    output.push_str(&format!("- 题库版本：`{}`\n", data.bank.version));
    output.push_str(&format!(
        "- 基础版本：`{}`\n",
        data.bank.base_version.as_deref().unwrap_or("无")
    ));
    output.push_str(&format!("- 知识框架：`{}`\n", data.bank.framework_version));
    output.push_str(&format!("- 来源修订：`{}`\n", data.bank.source_revision));
    output.push_str(&format!("- 评价模式：`{}`\n", data.bank.evaluation_mode));
    output.push_str(&format!("- 题目数量：{}\n", data.bank.questions.len()));
    output.push_str(&format!("- 角度目录：`{}`\n", data.angle_catalog.version));
    output.push_str(&format!(
        "- 课程角度数量：{}\n",
        unique_angle_count(&data.angle_catalog)
    ));
    output.push_str(&format!(
        "- 适用范围目录：`{}`\n",
        data.scope_catalog.catalog_version
    ));
    output.push_str(&format!(
        "- 生产批准数量：{}\n\n",
        data.production_approvals.len()
    ));

    let mut question_number = 0_usize;
    for (domain_index, (domain_code, domain_label)) in DOMAIN_ORDER.iter().enumerate() {
        output.push_str(&format!(
            "## {}. {} (`{}`)\n\n",
            domain_index + 1,
            domain_label,
            domain_code
        ));
        let mut questions = data
            .bank
            .questions
            .iter()
            .filter(|question| question.domain_code == *domain_code)
            .collect::<Vec<_>>();
        questions.sort_by_key(|question| question.id.as_str());
        let expected_domain_count = match *domain_code {
            "product" => EXPECTED_PRODUCT_QUESTION_COUNT,
            "service" => 11,
            "content" | "traffic" | "conversion" | "finance" => 12,
            _ => EXPECTED_BASE_QUESTION_COUNT_PER_DOMAIN,
        };
        ensure!(
            questions.len() == expected_domain_count,
            "领域 {domain_code} 的题目数量不正确"
        );

        for (index, question) in questions.into_iter().enumerate() {
            question_number += 1;
            let identity = question.identity();
            let angle = data.angles.get(&identity).context("渲染时缺少课程角度")?;
            let scope = data.scopes.get(&identity).context("渲染时缺少适用范围")?;
            let correct_index = question
                .choices
                .iter()
                .position(|choice| {
                    Some(choice.id.as_str()) == question.correct_choice_id.as_deref()
                })
                .context("渲染时答案键未命中")?;
            let correct_choice = &question.choices[correct_index];
            let production_status = if data.production_approvals.contains(&identity) {
                "已批准"
            } else {
                "未批准"
            };

            output.push_str(&format!(
                "<!-- question:{}:{} -->\n### {}.{} `{}` v{}\n\n",
                question.id,
                question.version,
                domain_index + 1,
                index + 1,
                question.id,
                question.version
            ));
            output.push_str(&format!("- 课程角度：`{}`\n", angle.angle_code));
            output.push_str(&format!(
                "- 适用范围：地区 `{}`；平台 `{}`；业务阶段 `{}`\n",
                scope.jurisdiction_code, scope.platform_code, scope.business_stage_code
            ));
            output.push_str(&format!(
                "- 工程状态：`{}` / `{}`；公司教研：待实名复核；生产：{}\n",
                question.review.reviewer_type, question.review.decision, production_status
            ));
            output.push_str(&format!(
                "- 工程记录：作者 `{}`；审核标识 `{}`；范围 `{}`；日期 `{}`\n\n",
                question.review.author_id,
                question.review.reviewer_id,
                question.review.review_scope,
                question.review.reviewed_at
            ));
            output.push_str(&format!("**题干**\n\n{}\n\n", question.prompt));
            output.push_str("| 选项 | 稳定 ID | 内容 | 拟定答案 |\n");
            output.push_str("| --- | --- | --- | --- |\n");
            for (choice_index, choice) in question.choices.iter().enumerate() {
                output.push_str(&format!(
                    "| {} | `{}` | {} | {} |\n",
                    option_letter(choice_index),
                    markdown_cell(&choice.id),
                    markdown_cell(&choice.text),
                    if Some(choice.id.as_str()) == question.correct_choice_id.as_deref() {
                        "是"
                    } else {
                        ""
                    }
                ));
            }
            output.push_str(&format!(
                "\n**拟定正确答案**：{} / `{}` / {}\n\n",
                option_letter(correct_index),
                correct_choice.id,
                correct_choice.text
            ));
            output.push_str(&format!(
                "**参考解释**\n\n{}\n\n",
                question.reference_answer
            ));
            output.push_str("**来源**\n\n");
            for (source_index, source) in question.sources.iter().enumerate() {
                output.push_str(&format!(
                    "{}. [{} - {}]({})\n",
                    source_index + 1,
                    source.publisher,
                    source.title,
                    source.url
                ));
                output.push_str(&format!(
                    "   - ID `{}`；权威级别 `{}`；发布版本 `{}`\n",
                    source.source_id, source.authority_level, source.published_version
                ));
                output.push_str(&format!(
                    "   - 核对 `{}`；有效至 `{}`；许可 `{}`；采用 `{}`\n",
                    source.checked_at,
                    source.valid_through,
                    source.license_type.as_deref().unwrap_or("未记录"),
                    source.use_mode.as_deref().unwrap_or("未记录")
                ));
                if let Some(license_url) = source.license_url.as_deref() {
                    output.push_str(&format!("   - 许可定位：{}\n", license_url));
                }
            }
            output.push_str("\n**教研核对提示**：请在具名审核清单中分别确认答案、干扰项、解释、难度/学段、地区和时效。\n\n");
        }
    }
    ensure!(
        question_number == data.bank.questions.len(),
        "审核参考包没有覆盖全部题目"
    );
    Ok(output)
}

fn unique_angle_count(catalog: &AssessmentAngleCatalog) -> usize {
    catalog
        .mappings
        .iter()
        .map(|mapping| mapping.angle_code.as_str())
        .collect::<HashSet<_>>()
        .len()
}

fn option_letter(index: usize) -> char {
    char::from_u32(u32::from(b'A') + u32::try_from(index).unwrap_or_default()).unwrap_or('?')
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
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

fn read_json<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let source =
        fs::read_to_string(path).with_context(|| format!("读取 JSON {} 失败", path.display()))?;
    serde_json::from_str(&source).with_context(|| format!("解析 JSON {} 失败", path.display()))
}

fn repository_root(backend_root: &Path) -> Result<&Path> {
    backend_root.parent().context("无法解析仓库根目录")
}

fn resolve_output_path(repo_root: &Path, output: Option<&Path>, bank: &QuestionBank) -> PathBuf {
    let default_name = format!(
        "{}_curriculum_review_reference_pack_{}.md",
        bank.version,
        bank.source_revision.replace('-', "")
    );
    let path = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("temp/REQUIREMENTS").join(default_name));
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_production_approval(question: &Question) -> ProductionApproval {
        ProductionApproval {
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
            release_approval: ProductReleaseApproval {
                approver_id: "employee.product.001".to_owned(),
                approver_role: PRODUCT_APPROVER_ROLE.to_owned(),
                approved_at: "2026-08-30".to_owned(),
                decision: PRODUCTION_RELEASE_DECISION.to_owned(),
            },
        }
    }

    #[test]
    fn current_review_pack_covers_the_composed_runtime_catalog() {
        let backend_root = common::project_root().expect("backend root");
        let data = load_review_pack_data(&backend_root).expect("review pack data");

        assert_eq!(data.bank.version, "ecommerce_diagnostic_v13");
        assert_eq!(data.bank.questions.len(), 100);
        assert_eq!(data.angles.len(), 100);
        assert_eq!(unique_angle_count(&data.angle_catalog), 67);
        assert_eq!(data.scopes.len(), 100);
        assert!(data.production_approvals.is_empty());
        assert!(data.bank.questions.iter().all(|question| {
            question
                .choices
                .iter()
                .filter(|choice| Some(choice.id.as_str()) == question.correct_choice_id.as_deref())
                .count()
                == 1
        }));
    }

    #[test]
    fn rendered_pack_contains_every_question_once_and_keeps_approval_honest() {
        let backend_root = common::project_root().expect("backend root");
        let data = load_review_pack_data(&backend_root).expect("review pack data");
        let rendered = render_review_pack(&data).expect("rendered review pack");

        assert_eq!(rendered.matches("<!-- question:").count(), 100);
        for question in &data.bank.questions {
            let marker = format!("<!-- question:{}:{} -->", question.id, question.version);
            assert_eq!(rendered.matches(&marker).count(), 1, "{}", question.id);
            assert!(rendered.contains(&question.prompt));
            assert!(rendered.contains(&question.reference_answer));
            assert!(
                rendered.contains(
                    question
                        .correct_choice_id
                        .as_deref()
                        .expect("current question answer key")
                )
            );
        }
        assert!(rendered.contains("公司教研：待实名复核；生产：未批准"));
        assert!(!rendered.contains("公司教研：已通过"));
    }

    #[test]
    fn production_entries_require_complete_separated_dual_approval() {
        let backend_root = common::project_root().expect("backend root");
        let data = load_review_pack_data(&backend_root).expect("review pack data");
        let question = data.bank.questions.first().expect("current question");
        let approval = valid_production_approval(question);
        let release = ProductionRelease {
            schema_version: PRODUCTION_RELEASE_SCHEMA_VERSION,
            question_bank_version: data.bank.version.clone(),
            approvals: vec![approval],
        };

        let approved =
            validate_production_approvals(&data.bank, &release).expect("complete dual approval");
        assert_eq!(approved.len(), 1);

        let mut failed_check = release.clone();
        failed_check.approvals[0]
            .curriculum_review
            .checks
            .source_currency = "failed".to_owned();
        assert!(validate_production_approvals(&data.bank, &failed_check).is_err());

        let mut missing_evidence = release.clone();
        missing_evidence.approvals[0].curriculum_review.evidence_id = " ".to_owned();
        assert!(validate_production_approvals(&data.bank, &missing_evidence).is_err());

        let mut old_schema = release.clone();
        old_schema.schema_version = 1;
        assert!(validate_production_approvals(&data.bank, &old_schema).is_err());

        let mut same_person = release;
        let reviewer_id = same_person.approvals[0]
            .curriculum_review
            .reviewer_id
            .clone();
        same_person.approvals[0].release_approval.approver_id = reviewer_id;
        assert!(validate_production_approvals(&data.bank, &same_person).is_err());
    }
}
