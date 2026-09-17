use crate::entity::student_learning_evidence;
use db_core::{
    DbContext, Repository,
    error::{BizResult, Error},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use time::OffsetDateTime;
use uuid::Uuid;

use super::dto::CreateStudentLearningEvidence;
db_core::impl_repository!(
    StudentLearningEvidenceRepo,
    student_learning_evidence::Entity,
    student_learning_evidence::Model
);
pub struct StudentLearningEvidenceService {
    repo: StudentLearningEvidenceRepo,
}
impl StudentLearningEvidenceService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StudentLearningEvidenceRepo::new(db),
        }
    }

    pub async fn list_all(&self) -> BizResult<Vec<student_learning_evidence::Model>> {
        Ok(self
            .repo
            .select_all(
                self.repo
                    .query()
                    .order_by_desc(student_learning_evidence::Column::CreatedAt),
            )
            .await?)
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
    ) -> BizResult<Vec<student_learning_evidence::Model>> {
        Ok(self
            .repo
            .select_all(
                self.repo
                    .query()
                    .filter(student_learning_evidence::Column::UserId.eq(user_id))
                    .order_by_desc(student_learning_evidence::Column::CreatedAt),
            )
            .await?)
    }

    pub async fn list_by_user_since(
        &self,
        user_id: Uuid,
        since: Option<OffsetDateTime>,
        limit: u64,
    ) -> BizResult<Vec<student_learning_evidence::Model>> {
        let query = self
            .repo
            .query()
            .filter(student_learning_evidence::Column::UserId.eq(user_id));
        let query = if let Some(since) = since {
            query.filter(student_learning_evidence::Column::CreatedAt.gte(since))
        } else {
            query
        };
        Ok(self
            .repo
            .select_all(
                query
                    .order_by_desc(student_learning_evidence::Column::CreatedAt)
                    .limit(limit),
            )
            .await?)
    }

    pub async fn get_by_probe_id(
        &self,
        probe_id: Uuid,
    ) -> BizResult<Option<student_learning_evidence::Model>> {
        Ok(self
            .repo
            .select_one(
                self.repo
                    .query()
                    .filter(student_learning_evidence::Column::ProbeId.eq(probe_id))
                    .order_by_desc(student_learning_evidence::Column::CreatedAt),
            )
            .await?)
    }

    pub async fn create_if_absent(&self, input: CreateStudentLearningEvidence) -> BizResult<bool> {
        let query = if let Some(probe_id) = input.probe_id {
            self.repo
                .query()
                .filter(student_learning_evidence::Column::ProbeId.eq(probe_id))
        } else {
            let source_message_id = input.source_message_id.ok_or_else(|| {
                db_core::error::BizError::new(
                    error_code::app::LEARNING_REQUEST_INVALID,
                    "learning evidence source missing".to_owned(),
                )
            })?;
            self.repo
                .query()
                .filter(student_learning_evidence::Column::SourceMessageId.eq(source_message_id))
                .filter(student_learning_evidence::Column::KnowledgeCode.eq(&input.knowledge_code))
                .filter(
                    student_learning_evidence::Column::AnalyzerVersion.eq(&input.analyzer_version),
                )
        };
        let existing = self.repo.select_one(query).await?;
        if existing.is_some() {
            return Ok(false);
        }

        student_learning_evidence::Entity::insert(student_learning_evidence::ActiveModel {
            id: Set(input.id),
            user_id: Set(input.user_id),
            conversation_id: Set(input.conversation_id),
            source_message_id: Set(input.source_message_id),
            probe_id: Set(input.probe_id),
            framework_version: Set(input.framework_version),
            knowledge_code: Set(input.knowledge_code),
            evidence_type: Set(input.evidence_type),
            direction: Set(input.direction),
            strength: Set(input.strength),
            confidence: Set(input.confidence),
            summary: Set(input.summary),
            rubric_hits: Set(input.rubric_hits),
            analyzer_version: Set(input.analyzer_version),
            created_at: Set(input.created_at),
        })
        .exec(self.repo.db())
        .await
        .map_err(Error::from)?;
        Ok(true)
    }
}
