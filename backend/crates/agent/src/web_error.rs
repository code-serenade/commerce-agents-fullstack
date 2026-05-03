use db_core::error::BizError;
use error_code::agent as agent_error;

use crate::agent_error::{Error, ErrorKind};

pub fn to_biz_error(err: Error) -> BizError {
    BizError::new(code_for_error(&err), err.to_string())
}

pub fn code_for_error(err: &Error) -> i16 {
    match err.kind() {
        ErrorKind::Cognition => agent_error::AGENT_COGNITION_ERROR,
        ErrorKind::Llm => agent_error::AGENT_LLM_ERROR,
        ErrorKind::Perception => agent_error::AGENT_PERCEPTION_ERROR,
        ErrorKind::Prompt => agent_error::AGENT_PROMPT_ERROR,
        ErrorKind::Settings => agent_error::AGENT_SETTINGS_ERROR,
        ErrorKind::External => agent_error::AGENT_EXTERNAL_ERROR,
        ErrorKind::Internal => agent_error::AGENT_INTERNAL_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use error_code::agent as agent_error;

    use super::{code_for_error, to_biz_error};
    use crate::agent_error::{Error, PromptError};

    #[test]
    fn maps_prompt_error_to_agent_error_code() {
        let err = Error::from(PromptError::not_found("prompts/missing.md"));

        assert_eq!(code_for_error(&err), agent_error::AGENT_PROMPT_ERROR);

        let biz = to_biz_error(err);
        assert_eq!(biz.code(), agent_error::AGENT_PROMPT_ERROR);
        assert!(biz.message().contains("Prompt not found"));
    }
}
