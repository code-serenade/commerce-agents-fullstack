use crate::{
    entity::student_learning_profiles,
    table::student_learning_profiles::dto::{
        CreateStudentLearningProfile, StudentLearningProfile, UpdateStudentLearningAnalysis,
        UpdateStudentLearningProfileSettings,
    },
};
use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, QueryFilter, QueryOrder,
};
use serde_json::json;
use uuid::Uuid;
db_core::impl_repository!(
    StudentLearningProfileRepo,
    student_learning_profiles::Entity,
    student_learning_profiles::Model
);
pub struct StudentLearningProfileService {
    repo: StudentLearningProfileRepo,
}
impl StudentLearningProfileService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StudentLearningProfileRepo::new(db),
        }
    }
    pub async fn get_by_user_id(&self, user_id: Uuid) -> BizResult<Option<StudentLearningProfile>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(student_learning_profiles::Column::UserId.eq(user_id)),
            )
            .await?
            .map(Self::map)
            .transpose()
    }
    pub async fn list_all(&self) -> BizResult<Vec<StudentLearningProfile>> {
        let models = self
            .repo
            .select_all(
                self.repo
                    .query()
                    .order_by_desc(student_learning_profiles::Column::UpdatedAt),
            )
            .await?;
        models.into_iter().map(Self::map).collect()
    }
    pub async fn create(
        &self,
        input: CreateStudentLearningProfile,
    ) -> BizResult<StudentLearningProfile> {
        Self::map(
            self.repo
                .insert(student_learning_profiles::ActiveModel {
                    id: Set(input.id),
                    user_id: Set(input.user_id),
                    framework_version: Set(input.framework_version),
                    mastery_map: Set(json!({})),
                    engagement_summary: Set(json!({})),
                    ..Default::default()
                })
                .await?,
        )
    }
    pub async fn update_settings(
        &self,
        input: UpdateStudentLearningProfileSettings,
    ) -> BizResult<StudentLearningProfile> {
        Self::map(
            self.repo
                .update(student_learning_profiles::ActiveModel {
                    id: Unchanged(input.id),
                    enabled: Set(input.enabled),
                    ..Default::default()
                })
                .await?,
        )
    }
    pub async fn update_analysis(
        &self,
        input: UpdateStudentLearningAnalysis,
    ) -> BizResult<StudentLearningProfile> {
        Self::map(
            self.repo
                .update(student_learning_profiles::ActiveModel {
                    id: Unchanged(input.id),
                    mastery_map: Set(input.mastery_map),
                    engagement_summary: Set(input.engagement_summary),
                    coverage_rate: Set(input.coverage_rate),
                    analysis_status: Set(input.analysis_status),
                    last_analyzed_sequence: Set(input.last_analyzed_sequence),
                    last_analyzed_at: Set(Some(input.last_analyzed_at)),
                    last_error_code: Set(input.last_error_code),
                    ..Default::default()
                })
                .await?,
        )
    }
    fn map(m: student_learning_profiles::Model) -> BizResult<StudentLearningProfile> {
        Ok(StudentLearningProfile {
            id: m.id,
            user_id: m.user_id,
            framework_version: m.framework_version,
            mastery_map: m.mastery_map,
            engagement_summary: m.engagement_summary,
            coverage_rate: m.coverage_rate,
            analysis_status: m.analysis_status,
            last_analyzed_sequence: m.last_analyzed_sequence,
            last_analyzed_at: m.last_analyzed_at,
            last_error_code: m.last_error_code,
            enabled: m.enabled,
            created_at: m.created_at,
            updated_at: m.updated_at,
        })
    }
}
