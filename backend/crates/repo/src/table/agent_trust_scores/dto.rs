use db_core::error::{BIZ_INTERNAL_ERROR, BizError, BizResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTrustLevel {
    High,
    Moderate,
    Low,
    None,
}

impl AgentTrustLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Moderate => "moderate",
            Self::Low => "low",
            Self::None => "none",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::High
        } else if score >= 0.5 {
            Self::Moderate
        } else if score >= 0.3 {
            Self::Low
        } else {
            Self::None
        }
    }
}

impl TryFrom<&str> for AgentTrustLevel {
    type Error = BizError;

    fn try_from(value: &str) -> BizResult<Self> {
        match value {
            "high" => Ok(Self::High),
            "moderate" => Ok(Self::Moderate),
            "low" => Ok(Self::Low),
            "none" => Ok(Self::None),
            _ => Err(BizError::new(
                BIZ_INTERNAL_ERROR,
                format!("invalid agent trust level: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrustScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub skill_id: Option<String>,
    pub score: f64,
    pub trust_level: AgentTrustLevel,
    pub previous_score: Option<f64>,
    pub previous_level: Option<String>,
    pub delta: f64,
    pub reason: Option<String>,
    pub source: String,
    pub quality_check_id: Option<Uuid>,
    pub metadata: Value,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentTrustScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub skill_id: Option<String>,
    pub score: f64,
    pub trust_level: AgentTrustLevel,
    pub previous_score: Option<f64>,
    pub previous_level: Option<String>,
    pub delta: f64,
    pub reason: Option<String>,
    pub source: String,
    pub quality_check_id: Option<Uuid>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAgentTrustScore {
    pub id: Uuid,
    pub score: f64,
    pub trust_level: AgentTrustLevel,
    pub previous_score: Option<f64>,
    pub previous_level: Option<String>,
    pub delta: f64,
    pub reason: Option<String>,
    pub source: String,
    pub quality_check_id: Option<Uuid>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AgentTrustScoreFilter {
    pub user_id: Uuid,
    pub role: Option<String>,
    pub skill_id: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
