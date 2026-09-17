use std::sync::OnceLock;

use service::api::LearningQuestionDeliveryMode;

use crate::error::{Error, Result};

static QUESTION_DELIVERY_MODE: OnceLock<LearningQuestionDeliveryMode> = OnceLock::new();

pub fn init_learning_runtime(mode: LearningQuestionDeliveryMode) -> Result<()> {
    QUESTION_DELIVERY_MODE
        .set(mode)
        .map_err(|_| Error::Custom("Learning runtime already initialized".to_owned()))
}

pub fn get_question_delivery_mode() -> LearningQuestionDeliveryMode {
    QUESTION_DELIVERY_MODE.get().copied().unwrap_or_default()
}
