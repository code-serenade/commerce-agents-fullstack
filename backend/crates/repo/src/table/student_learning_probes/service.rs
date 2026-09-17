use crate::{
    entity::student_learning_probes,
    table::student_learning_probes::{
        CreateStudentLearningProbe, MarkStudentLearningProbeAnswered,
        MarkStudentLearningProbeStructuredAnswer, StudentLearningProbe,
        UpdateStudentLearningProbeContext, UpdateStudentLearningProbeStatus,
    },
};
use db_core::{DbContext, Repository, error::BizResult};
use sea_orm::{
    ActiveValue::{Set, Unchanged},
    ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use time::OffsetDateTime;
use uuid::Uuid;

db_core::impl_repository!(
    StudentLearningProbeRepo,
    student_learning_probes::Entity,
    student_learning_probes::Model
);

pub struct StudentLearningProbeService {
    repo: StudentLearningProbeRepo,
}

impl StudentLearningProbeService {
    pub fn new(db: DbContext) -> Self {
        Self {
            repo: StudentLearningProbeRepo::new(db),
        }
    }

    pub async fn create(
        &self,
        input: CreateStudentLearningProbe,
    ) -> BizResult<StudentLearningProbe> {
        Self::map(
            self.repo
                .insert(student_learning_probes::ActiveModel {
                    id: Set(input.id),
                    user_id: Set(input.user_id),
                    conversation_id: Set(input.conversation_id),
                    assistant_message_id: Set(input.assistant_message_id),
                    framework_version: Set(input.framework_version),
                    knowledge_code: Set(input.knowledge_code),
                    status: Set("offered".to_owned()),
                    question_text: Set(input.question_text),
                    reason_code: Set(input.reason_code),
                    priority_score: Set(input.priority_score),
                    context_snapshot: Set(input.context_snapshot),
                    offered_at: Set(Some(input.offered_at)),
                    expires_at: Set(Some(input.expires_at)),
                    ..Default::default()
                })
                .await?,
        )
    }

    pub async fn get_by_id(&self, id: Uuid) -> BizResult<Option<StudentLearningProbe>> {
        self.repo.find_by_id(id).await?.map(Self::map).transpose()
    }

    pub async fn list_by_ids(&self, ids: Vec<Uuid>) -> BizResult<Vec<StudentLearningProbe>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.repo
            .select_all(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::Id.is_in(ids)),
            )
            .await?
            .into_iter()
            .map(Self::map)
            .collect()
    }

    pub async fn list_all(&self) -> BizResult<Vec<StudentLearningProbe>> {
        self.repo
            .select_all(self.repo.query())
            .await?
            .into_iter()
            .map(Self::map)
            .collect()
    }

    pub async fn get_active_by_user(
        &self,
        user_id: Uuid,
    ) -> BizResult<Option<StudentLearningProbe>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::UserId.eq(user_id))
                    .filter(
                        Condition::any()
                            .add(student_learning_probes::Column::Status.eq("planned"))
                            .add(student_learning_probes::Column::Status.eq("offered")),
                    )
                    .order_by_desc(student_learning_probes::Column::CreatedAt),
            )
            .await?
            .map(Self::map)
            .transpose()
    }

    pub async fn get_latest_by_user(
        &self,
        user_id: Uuid,
    ) -> BizResult<Option<StudentLearningProbe>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::UserId.eq(user_id))
                    .order_by_desc(student_learning_probes::Column::CreatedAt),
            )
            .await?
            .map(Self::map)
            .transpose()
    }

    pub async fn get_latest_by_user_and_assistant_message(
        &self,
        user_id: Uuid,
        assistant_message_id: Uuid,
    ) -> BizResult<Option<StudentLearningProbe>> {
        self.repo
            .select_one(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::UserId.eq(user_id))
                    .filter(
                        student_learning_probes::Column::AssistantMessageId
                            .eq(assistant_message_id),
                    )
                    .order_by_desc(student_learning_probes::Column::CreatedAt),
            )
            .await?
            .map(Self::map)
            .transpose()
    }

    pub async fn list_recent_answered_by_user_and_assistant_message(
        &self,
        user_id: Uuid,
        assistant_message_id: Uuid,
        limit: u64,
    ) -> BizResult<Vec<StudentLearningProbe>> {
        self.repo
            .select_all(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::UserId.eq(user_id))
                    .filter(
                        student_learning_probes::Column::AssistantMessageId
                            .eq(assistant_message_id),
                    )
                    .filter(student_learning_probes::Column::Status.eq("answered"))
                    .order_by_desc(student_learning_probes::Column::CreatedAt)
                    .limit(limit),
            )
            .await?
            .into_iter()
            .map(Self::map)
            .collect()
    }

    pub async fn update_status(
        &self,
        input: UpdateStudentLearningProbeStatus,
    ) -> BizResult<StudentLearningProbe> {
        Self::map(
            self.repo
                .update(student_learning_probes::ActiveModel {
                    id: Unchanged(input.id),
                    status: Set(input.status),
                    ..Default::default()
                })
                .await?,
        )
    }

    pub async fn update_context_snapshot(
        &self,
        input: UpdateStudentLearningProbeContext,
    ) -> BizResult<StudentLearningProbe> {
        Self::map(
            self.repo
                .update(student_learning_probes::ActiveModel {
                    id: Unchanged(input.id),
                    context_snapshot: Set(input.context_snapshot),
                    ..Default::default()
                })
                .await?,
        )
    }

    pub async fn list_offered_by_conversation(
        &self,
        conversation_id: Uuid,
    ) -> BizResult<Vec<StudentLearningProbe>> {
        let now = OffsetDateTime::now_utc();
        self.repo
            .select_all(
                self.repo
                    .query()
                    .filter(student_learning_probes::Column::ConversationId.eq(conversation_id))
                    .filter(student_learning_probes::Column::Status.eq("offered"))
                    .filter(
                        Condition::any()
                            .add(student_learning_probes::Column::ExpiresAt.is_null())
                            .add(student_learning_probes::Column::ExpiresAt.gt(now)),
                    )
                    .order_by_desc(student_learning_probes::Column::CreatedAt),
            )
            .await?
            .into_iter()
            .map(Self::map)
            .collect()
    }

    pub async fn mark_answered(
        &self,
        input: MarkStudentLearningProbeAnswered,
    ) -> BizResult<StudentLearningProbe> {
        Self::map(
            self.repo
                .update(student_learning_probes::ActiveModel {
                    id: Unchanged(input.id),
                    status: Set("answered".to_owned()),
                    answer_message_id: Set(Some(input.answer_message_id)),
                    answered_at: Set(Some(input.answered_at)),
                    ..Default::default()
                })
                .await?,
        )
    }

    pub async fn try_mark_structured_answer(
        &self,
        input: MarkStudentLearningProbeStructuredAnswer,
    ) -> BizResult<bool> {
        let result = student_learning_probes::Entity::update_many()
            .set(student_learning_probes::ActiveModel {
                status: Set("answered".to_owned()),
                context_snapshot: Set(input.context_snapshot),
                answered_at: Set(Some(input.answered_at)),
                ..Default::default()
            })
            .filter(student_learning_probes::Column::Id.eq(input.id))
            .filter(student_learning_probes::Column::Status.eq("offered"))
            .exec(self.repo.db())
            .await
            .map_err(db_core::error::Error::from)?;
        Ok(result.rows_affected == 1)
    }

    fn map(m: student_learning_probes::Model) -> BizResult<StudentLearningProbe> {
        Ok(StudentLearningProbe {
            id: m.id,
            user_id: m.user_id,
            conversation_id: m.conversation_id,
            assistant_message_id: m.assistant_message_id,
            answer_message_id: m.answer_message_id,
            framework_version: m.framework_version,
            knowledge_code: m.knowledge_code,
            status: m.status,
            question_text: m.question_text,
            reason_code: m.reason_code,
            priority_score: m.priority_score,
            context_snapshot: m.context_snapshot,
            offered_at: m.offered_at,
            answered_at: m.answered_at,
            expires_at: m.expires_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        })
    }
}
