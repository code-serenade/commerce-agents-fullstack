use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LearningSortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LearningStudentSortBy {
    CoverageRate,
    ObservedKnowledgeCount,
    LastAnalyzedAt,
    UpdatedAt,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListLearningStudentsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    pub sort_by: Option<LearningStudentSortBy>,
    pub sort_order: Option<LearningSortOrder>,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}
