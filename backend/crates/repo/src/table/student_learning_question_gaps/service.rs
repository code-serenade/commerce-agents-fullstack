use crate::{
    entity::student_learning_question_gaps,
    table::student_learning_question_gaps::{
        CreateStudentLearningQuestionGap, StudentLearningQuestionGap,
    },
};
use db_core::{
    DbContext, Repository,
    error::{BIZ_INTERNAL_ERROR, BizError, BizResult},
};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, sea_query::OnConflict};

db_core::impl_repository!(
    StudentLearningQuestionGapRepo,
    student_learning_question_gaps::Entity,
    student_learning_question_gaps::Model
);

pub struct StudentLearningQuestionGapService {
    repo: StudentLearningQuestionGapRepo,
}

impl StudentLearningQuestionGapService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StudentLearningQuestionGapRepo::new(db),
        }
    }

    pub async fn create_idempotent(
        &self,
        input: CreateStudentLearningQuestionGap,
    ) -> BizResult<StudentLearningQuestionGap> {
        student_learning_question_gaps::Entity::insert(
            student_learning_question_gaps::ActiveModel {
                id: Set(input.id),
                user_id: Set(input.user_id),
                conversation_id: Set(input.conversation_id),
                assistant_message_id: Set(input.assistant_message_id),
                framework_version: Set(input.framework_version),
                selector_version: Set(input.selector_version),
                knowledge_code: Set(input.knowledge_code),
                reason_code: Set(input.reason_code),
                topic_signals: Set(input.topic_signals),
                scope_signals: Set(input.scope_signals),
                ..Default::default()
            },
        )
        .on_conflict(
            OnConflict::column(student_learning_question_gaps::Column::AssistantMessageId)
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(self.repo.db())
        .await
        .map_err(db_core::error::Error::from)?;

        self.repo
            .select_one(
                self.repo.query().filter(
                    student_learning_question_gaps::Column::AssistantMessageId
                        .eq(input.assistant_message_id),
                ),
            )
            .await?
            .map(Self::map)
            .transpose()?
            .ok_or_else(|| {
                BizError::new(
                    BIZ_INTERNAL_ERROR,
                    "learning question gap missing after insert".to_owned(),
                )
            })
    }

    pub async fn list_all(&self) -> BizResult<Vec<StudentLearningQuestionGap>> {
        self.repo
            .select_all(self.repo.query())
            .await?
            .into_iter()
            .map(Self::map)
            .collect()
    }

    fn map(model: student_learning_question_gaps::Model) -> BizResult<StudentLearningQuestionGap> {
        Ok(StudentLearningQuestionGap {
            id: model.id,
            user_id: model.user_id,
            conversation_id: model.conversation_id,
            assistant_message_id: model.assistant_message_id,
            framework_version: model.framework_version,
            selector_version: model.selector_version,
            knowledge_code: model.knowledge_code,
            reason_code: model.reason_code,
            topic_signals: model.topic_signals,
            scope_signals: model.scope_signals,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
