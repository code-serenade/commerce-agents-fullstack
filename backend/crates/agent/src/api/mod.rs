//! Public API for applications embedding the commerce agent runtime.
//!
//! Web servers should depend on this module instead of internal runtime modules.

pub use crate::{
    agent_error::{Error, Result},
    agent_manager::{AgentManager, AgentManagerConfig, AgentRequest},
    agent_session::{
        CognitionFactory, DeterministicMemoryCompactor, FileMemoryStore, FileMemoryStoreError,
        InMemoryMemoryStore, MemoryBudget, MemoryBundle, MemoryCompactionPolicy,
        MemoryCompactionStats, MemoryCompactor, MemoryId, MemoryKind, MemoryQuery, MemoryRecord,
        MemoryScope, MemorySource, MemoryStore, MemoryWriteOutcome, NoopMemoryStore,
        RoleCognitionFactory, RoleMemorySummary, SessionEventContextPackFailure,
        SessionEventDiagnosticAudit, SessionEventDiagnosticConsistencyAudit,
        SessionEventDiagnosticConsistencyIssue, SessionEventDiagnosticConsistencyOptions,
        SessionEventLegacyFailureMarker, SessionEventRoleAudit, SessionEventRx,
        SessionEventSearchDiagnostic, SessionEventSkillFailure, SessionEventTx, SessionResult,
        SessionRuntimeConfig, SessionSnapshot, TaskGraphMemorySummary, WorkerDetail,
        audit_diagnostic_consistency, audit_session_event_diagnostic_consistency,
        audit_session_event_diagnostics, audit_session_event_roles,
        audit_session_event_roles_against, render_session_event_diagnostic_audit,
        render_session_event_diagnostic_consistency_audit, render_session_event_role_audit,
    },
    cognition::{
        Cognition, CognitionEngine, CognitionFailure, CognitionInput, CognitionOutput,
        CognitionResult, Context, Fact, FailureReason, Intent, IntentKind, JsonTemplateDecoder,
    },
    core::protocol::{
        AgentId, DispatchContextBudget, DispatchPlannedStep, SessionEvent, SessionId,
        TaskGraphHandoff, TaskGraphId, TaskGraphProcessGap, TaskGraphProcessPanel,
        TaskGraphProcessRole, TaskGraphProcessSummary, TaskId, TaskNodeId, UserId,
        WorkerDiagnostic, WorkerId, WorkerReportStatus, WorkspaceId,
    },
    llm_routing::{AgentLlmCfg, AgentLlmUsageCfg},
    roles::{
        RoleAction, RoleAssignment, RoleCatalog, RoleCollaborationPlan, RoleContribution, RoleId,
        RoleProfile, RolePromptBuilder, RolePromptInput, RoleRoute, RoleRouteInput, RoleRouter,
        RoleScore, RoleSkillCatalog,
    },
    runtime::{AgentRuntimeCfg, build_agent_manager},
    skills::{
        SearchAdapterConfig, SkillAdapterKind, SkillCategory, SkillExecutionMode, SkillInputField,
        SkillPriority, SkillRegistry, SkillRiskLevel, SkillSpec, adapter_kinds_for_spec,
        execution_mode_for_spec, risk_level_for_spec,
    },
    system_agents::vision_preprocess::{
        VisionPreprocessAgent, VisionPreprocessAgentConfig, VisionPreprocessInput,
        VisionPreprocessOutput, VisionPreprocessResult,
    },
    web_error::{code_for_error, to_biz_error},
};
