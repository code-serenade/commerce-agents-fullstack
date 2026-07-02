use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppUserStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppUserSortBy {
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAppUsersQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    pub status: Option<AppUserStatus>,
    #[validate(length(max = 64))]
    pub created_at_from: Option<String>,
    #[validate(length(max = 64))]
    pub created_at_to: Option<String>,
    #[validate(length(max = 64))]
    pub updated_at_from: Option<String>,
    #[validate(length(max = 64))]
    pub updated_at_to: Option<String>,
    pub sort_by: Option<AppUserSortBy>,
    pub sort_order: Option<SortOrder>,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAppUserRequest {
    #[validate(length(min = 1, max = 64))]
    pub display_id: String,
    #[validate(length(min = 1, max = 64))]
    pub display_name: String,
    #[validate(length(max = 128))]
    pub school_name: Option<String>,
    #[validate(length(max = 128))]
    pub team_name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCurrentAppUserProfileRequest {
    #[validate(length(max = 128))]
    pub school_name: Option<String>,
    #[validate(length(max = 128))]
    pub team_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentAppUserProfileResponse {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub is_school_team_bound: bool,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppUserRequest {
    #[validate(length(min = 1, max = 255))]
    pub remark: Option<String>,
    pub status: AppUserStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUserResponse {
    pub user_id: String,
    pub display_id: String,
    pub display_name: String,
    pub school_name: Option<String>,
    pub team_name: Option<String>,
    pub remark: Option<String>,
    pub status: AppUserStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub roles: Vec<RoleResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUserMetricsResponse {
    pub total: u64,
    pub enabled: u64,
    pub disabled: u64,
    pub multi_role: u64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    #[validate(length(min = 1, max = 64))]
    pub code: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleResponse {
    pub id: i64,
    pub name: String,
    pub code: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePermissionsRequest {
    pub permission_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRolesRequest {
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleOptionResponse {
    pub id: i64,
    pub name: String,
    pub code: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionTreeNode {
    pub id: i64,
    pub name: String,
    pub kind: PermissionKind,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub children: Vec<PermissionTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionTreeNode {
    pub id: i64,
    pub name: String,
    pub kind: PermissionKind,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub checked: bool,
    pub children: Vec<RolePermissionTreeNode>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Group,
    Action,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserPermissionsResponse {
    pub user_id: String,
    pub role_codes: Vec<String>,
    pub permission_codes: Vec<String>,
}

pub type CommerceAgentPlaybookListResponse =
    service::dto::commerce_playbook::CommerceAgentPlaybookListResponse;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeOptionsResponse {
    pub strategies: Vec<AgentRuntimeChoiceResponse>,
    pub response_modes: Vec<AgentRuntimeChoiceResponse>,
    pub routing_modes: Vec<AgentRuntimeChoiceResponse>,
    pub learning_levels: Vec<AgentRuntimeChoiceResponse>,
    pub collaboration_modes: Vec<AgentRuntimeChoiceResponse>,
    pub roles: Vec<AgentRuntimeRoleOptionResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeChoiceResponse {
    pub id: String,
    pub label: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeRoleOptionResponse {
    pub id: String,
    pub name: String,
    pub runtime_role: String,
    pub capabilities: Vec<String>,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityCatalogResponse {
    pub source: String,
    pub version: String,
    pub summary: AgentCapabilityCatalogSummaryResponse,
    pub roles: Vec<AgentCapabilityRoleResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityCatalogSummaryResponse {
    pub role_count: usize,
    pub skill_count: usize,
    pub native_skill_count: usize,
    pub adapter_skill_count: usize,
    pub approval_skill_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilityRoleResponse {
    pub id: String,
    pub name: String,
    pub runtime_role: String,
    pub priority: u32,
    pub domains: Vec<String>,
    pub keywords: Vec<String>,
    pub preferred_actions: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub optional_capabilities: Vec<String>,
    pub skill_count: usize,
    pub skills: Vec<AgentCapabilitySkillResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilitySkillResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub priority: String,
    pub execution_mode: String,
    pub risk_level: String,
    pub approval_required: bool,
    pub adapter_requirements: Vec<String>,
    pub tags: Vec<String>,
    pub inputs: Vec<AgentCapabilitySkillInputResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilitySkillInputResponse {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub schema: Value,
    pub default: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 20000))]
    pub first_message: Option<String>,
    #[validate(length(max = 128))]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddConversationMessageRequest {
    #[validate(length(min = 1, max = 32))]
    pub role: String,
    #[validate(length(max = 20000))]
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
    #[validate(length(max = 32))]
    pub strategy_id: Option<String>,
    #[validate(length(max = 32))]
    pub response_mode: Option<String>,
    #[validate(length(max = 32))]
    pub routing_mode: Option<String>,
    #[validate(length(max = 32))]
    pub collaboration_mode: Option<String>,
    #[validate(length(max = 32))]
    pub learning_level: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConversationRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterConversationFileRequest {
    #[validate(length(min = 1, max = 128))]
    pub message_id: Option<String>,
    #[validate(length(min = 1, max = 1024))]
    pub object_key: String,
    #[validate(length(max = 255))]
    pub file_name: Option<String>,
    #[validate(length(max = 128))]
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    #[validate(length(max = 128))]
    pub sha256: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConversationFileExtractionRequest {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 100000))]
    pub text: Option<String>,
    #[validate(length(max = 32))]
    pub content_kind: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 64))]
    pub parser: Option<String>,
    #[validate(length(max = 32))]
    pub language: Option<String>,
    #[validate(length(max = 2000))]
    pub failure_reason: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TriggerConversationFileExtractionJobRequest {
    #[validate(length(max = 32))]
    pub mode: Option<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListUserFilesQuery {
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AttachConversationFileRequest {
    #[validate(length(min = 1, max = 128))]
    pub conversation_id: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BulkDeleteUserFilesRequest {
    #[validate(length(min = 1, max = 100))]
    pub file_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PresignConversationFileUploadRequest {
    #[validate(length(min = 1, max = 32))]
    pub extension: String,
    #[validate(length(max = 128))]
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignConversationFileUploadResponse {
    pub method: String,
    pub upload_url: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub expires_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationResponse {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub status: String,
    pub source: String,
    pub agent_id: Option<String>,
    pub model_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDetailResponse {
    pub conversation: ConversationResponse,
    pub messages: Vec<ConversationMessageResponse>,
    pub files: Vec<ConversationFileResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub user_id: String,
    pub role: String,
    pub content_text: Option<String>,
    pub content_json: Option<Value>,
    pub status: String,
    pub sequence: i64,
    pub strategy_id: Option<String>,
    pub response_mode: Option<String>,
    pub routing_mode: Option<String>,
    pub collaboration_mode: Option<String>,
    pub learning_level: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFileResponse {
    pub id: String,
    pub conversation_id: String,
    pub message_id: Option<String>,
    pub bucket: String,
    pub object_key: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFileAccessResponse {
    pub file: ConversationFileResponse,
    pub method: String,
    pub download_url: String,
    pub expires_secs: u64,
}

pub type ConversationFileExtractionStatusResponse =
    service::dto::conversation::ConversationFileExtractionStatusResponse;
pub type ConversationFileExtractionJobResponse =
    service::dto::conversation::ConversationFileExtractionJobResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 128))]
    pub external_id: Option<String>,
    #[validate(length(max = 128))]
    pub category: Option<String>,
    #[validate(length(max = 128))]
    pub brand: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(length(max = 64))]
    pub price: Option<String>,
    #[validate(length(max = 16))]
    pub currency: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductDraftRequest {
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 128))]
    pub category: Option<String>,
    #[validate(length(max = 128))]
    pub brand: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(length(max = 64))]
    pub price: Option<String>,
    #[validate(length(max = 16))]
    pub currency: Option<String>,
    #[validate(length(max = 2000))]
    pub prompt: Option<String>,
    #[validate(length(max = 64))]
    pub generation_mode: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ImportProductTemplateRequest {
    #[validate(length(min = 1, max = 64))]
    pub template_id: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 128))]
    pub external_id: Option<String>,
    #[validate(length(max = 128))]
    pub category: Option<String>,
    #[validate(length(max = 128))]
    pub brand: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(length(max = 64))]
    pub price: Option<String>,
    #[validate(length(max = 16))]
    pub currency: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransitionProductLifecycleRequest {
    #[validate(length(min = 1, max = 32))]
    pub target_status: String,
    #[validate(length(max = 500))]
    pub reason: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub external_id: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub status: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetailResponse {
    pub product: ProductResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDashboardResponse {
    pub total: u64,
    pub draft: u64,
    pub listing: u64,
    pub active: u64,
    pub archived: u64,
}

pub type ProductLifecycleOptionsResponse = service::dto::product::ProductLifecycleOptionsResponse;
pub type ProductLifecycleResponse = service::dto::product::ProductLifecycleResponse;
pub type ProductDraftResponse = service::dto::product::ProductDraftResponse;
pub type ProductTemplateResponse = service::dto::product::ProductTemplateResponse;
pub type ProductTemplateImportResponse = service::dto::product::ProductTemplateImportResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListCampaignsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_type: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaignRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 64))]
    pub campaign_type: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub budget: Option<f64>,
    #[validate(length(max = 32))]
    pub start_date: Option<String>,
    #[validate(length(max = 32))]
    pub end_date: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub metadata: Option<Value>,
    pub output: Option<Value>,
    pub action_metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaignFormDraftRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_type: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub budget: Option<f64>,
    #[validate(length(max = 32))]
    pub start_date: Option<String>,
    #[validate(length(max = 32))]
    pub end_date: Option<String>,
    #[validate(length(max = 2000))]
    pub goal: Option<String>,
    #[validate(length(max = 1000))]
    pub target_audience: Option<String>,
    #[validate(length(max = 16))]
    pub channels: Option<Vec<String>>,
    #[validate(length(max = 2000))]
    pub prompt: Option<String>,
    #[validate(length(max = 64))]
    pub generation_mode: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ImportCampaignTemplateRequest {
    #[validate(length(min = 1, max = 64))]
    pub template_id: String,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaignDraftRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub source_artifact_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 64))]
    pub campaign_type: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub budget: Option<f64>,
    #[validate(length(max = 32))]
    pub start_date: Option<String>,
    #[validate(length(max = 32))]
    pub end_date: Option<String>,
    #[validate(length(max = 2000))]
    pub goal: Option<String>,
    #[validate(length(max = 1000))]
    pub target_audience: Option<String>,
    #[validate(length(max = 16))]
    pub channels: Option<Vec<String>>,
    #[validate(length(max = 20000))]
    pub strategy: Option<String>,
    #[validate(length(max = 64))]
    pub source_type: Option<String>,
    #[validate(length(max = 20000))]
    pub source_summary: Option<String>,
    pub draft: Option<Value>,
    pub metadata: Option<Value>,
    pub action_metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCampaignRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_type: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(range(min = 0.0))]
    pub budget: Option<f64>,
    #[validate(length(max = 32))]
    pub start_date: Option<String>,
    #[validate(length(max = 32))]
    pub end_date: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub metadata: Option<Value>,
    pub output: Option<Value>,
    pub action_metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignListResponse {
    pub campaigns: Vec<CampaignResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDraftResponse {
    pub campaign: CampaignResponse,
    pub source: CampaignDraftSourceResponse,
    pub warnings: Vec<String>,
    pub agent_context: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDraftSourceResponse {
    pub source_type: String,
    pub product_id: Option<String>,
    pub workspace_id: Option<String>,
    pub source_artifact_id: Option<String>,
    pub source_summary: Option<String>,
    pub ref_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: Option<String>,
    pub name: String,
    pub campaign_type: Option<String>,
    pub description: Option<String>,
    pub budget: f64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub output: Value,
    pub action_metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

pub type CampaignFormDraftResponse = service::dto::campaign::CampaignFormDraftResponse;
pub type CampaignTemplateResponse = service::dto::campaign::CampaignTemplateResponse;
pub type CampaignTemplateImportResponse = service::dto::campaign::CampaignTemplateImportResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspacesQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_type: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 64))]
    pub workspace_type: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRequest {
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub campaign_id: Option<String>,
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_type: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceTasksQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub owner_role: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceTaskRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(length(max = 64))]
    pub owner_role: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 0, max = 10))]
    pub priority: Option<i32>,
    pub depends_on: Option<Value>,
    #[validate(length(max = 20000))]
    pub acceptance_criteria: Option<String>,
    #[validate(length(max = 40000))]
    pub result: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceTaskRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    #[validate(length(max = 64))]
    pub owner_role: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 0, max = 10))]
    pub priority: Option<i32>,
    pub depends_on: Option<Value>,
    #[validate(length(max = 20000))]
    pub acceptance_criteria: Option<String>,
    #[validate(length(max = 40000))]
    pub result: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceRunsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRunRequest {
    #[validate(length(min = 1, max = 40000))]
    pub goal: String,
    pub requirements: Option<Value>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 40000))]
    pub summary: Option<String>,
    pub report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRunRequest {
    #[validate(length(max = 40000))]
    pub goal: Option<String>,
    pub requirements: Option<Value>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 40000))]
    pub summary: Option<String>,
    pub report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceRunTasksQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAgentSkillRunsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 128))]
    pub skill_id: Option<String>,
    #[validate(length(max = 64))]
    pub role: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspaceArtifactsQuery {
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_task_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_task_id: Option<String>,
    #[validate(length(max = 64))]
    pub artifact_type: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceArtifactQuery {
    #[validate(length(max = 32))]
    pub format: Option<String>,
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAgentQualityChecksQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateAgentRunQualityRequest {
    #[validate(length(max = 64))]
    pub artifact_id: Option<String>,
    #[validate(length(max = 100000))]
    pub output: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAgentRunFeedbackRequest {
    #[validate(range(min = 1, max = 5))]
    pub rating: i32,
    #[validate(length(max = 40000))]
    pub comment: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAgentLearningRecordsQuery {
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub role: Option<String>,
    #[validate(length(max = 32))]
    pub category: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAgentTrustScoresQuery {
    #[validate(length(max = 64))]
    pub role: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRunTaskRequest {
    #[validate(length(max = 64))]
    pub workspace_task_id: Option<String>,
    pub seq: Option<i32>,
    #[validate(length(max = 64))]
    pub owner_role: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 0))]
    pub attempts: Option<i32>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub acceptance_score: Option<f64>,
    pub acceptance_passed: Option<bool>,
    #[validate(length(max = 20000))]
    pub acceptance_notes: Option<String>,
    #[validate(length(max = 40000))]
    pub execution_notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRunTaskRequest {
    pub seq: Option<i32>,
    #[validate(length(max = 64))]
    pub owner_role: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 0))]
    pub attempts: Option<i32>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub acceptance_score: Option<f64>,
    pub acceptance_passed: Option<bool>,
    #[validate(length(max = 20000))]
    pub acceptance_notes: Option<String>,
    #[validate(length(max = 40000))]
    pub execution_notes: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceArtifactRequest {
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_task_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_task_id: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub artifact_type: String,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 100000))]
    pub content: Option<String>,
    #[validate(length(max = 4000))]
    pub preview: Option<String>,
    #[validate(length(max = 128))]
    pub mime_type: Option<String>,
    #[validate(length(max = 64))]
    pub format: Option<String>,
    #[validate(length(max = 32))]
    pub storage_kind: Option<String>,
    #[validate(length(max = 128))]
    pub bucket: Option<String>,
    #[validate(length(max = 2048))]
    pub object_key: Option<String>,
    #[validate(length(max = 255))]
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    #[validate(length(max = 128))]
    pub sha256: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 32))]
    pub validation_status: Option<String>,
    pub validation_report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceArtifactRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 100000))]
    pub content: Option<String>,
    #[validate(length(max = 4000))]
    pub preview: Option<String>,
    #[validate(length(max = 128))]
    pub mime_type: Option<String>,
    #[validate(length(max = 64))]
    pub format: Option<String>,
    #[validate(length(max = 32))]
    pub storage_kind: Option<String>,
    #[validate(length(max = 128))]
    pub bucket: Option<String>,
    #[validate(length(max = 2048))]
    pub object_key: Option<String>,
    #[validate(length(max = 255))]
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    #[validate(length(max = 128))]
    pub sha256: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 32))]
    pub validation_status: Option<String>,
    pub validation_report: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListExecutionQueueQuery {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 128))]
    pub action_type: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateExecutionQueueRequest {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_task_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub action_type: String,
    #[validate(length(min = 1, max = 40000))]
    pub description: String,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    pub payload: Option<Value>,
    #[validate(length(max = 40000))]
    pub expected_impact: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DecideExecutionQueueRequest {
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListToolApprovalsQuery {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 128))]
    pub skill_name: Option<String>,
    #[validate(length(max = 16))]
    pub risk_level: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateToolApprovalRequest {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 64))]
    pub request_role: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub skill_name: String,
    #[validate(length(max = 255))]
    pub tool_call_id: Option<String>,
    #[validate(length(max = 16))]
    pub risk_level: Option<String>,
    pub args: Option<Value>,
    #[validate(length(max = 255))]
    pub action_key: Option<String>,
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DecideToolApprovalRequest {
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsQuery {
    #[validate(length(max = 64))]
    pub actor_user_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub execution_queue_id: Option<String>,
    #[validate(length(max = 64))]
    pub tool_approval_id: Option<String>,
    #[validate(length(max = 128))]
    pub action: Option<String>,
    #[validate(length(max = 64))]
    pub target_type: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuditLogRequest {
    #[validate(length(max = 64))]
    pub actor_user_id: Option<String>,
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub execution_queue_id: Option<String>,
    #[validate(length(max = 64))]
    pub tool_approval_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub action: String,
    #[validate(length(min = 1, max = 64))]
    pub target_type: String,
    #[validate(length(max = 255))]
    pub target_id: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
    #[validate(length(max = 255))]
    pub request_id: Option<String>,
    #[validate(length(max = 255))]
    pub trace_id: Option<String>,
    pub metadata: Option<Value>,
}

pub type ToolApprovalListResponse = service::dto::approval_audit::ToolApprovalListResponse;
pub type ToolApprovalResponse = service::dto::approval_audit::ToolApprovalResponse;
pub type AuditLogListResponse = service::dto::approval_audit::AuditLogListResponse;
pub type AuditLogResponse = service::dto::approval_audit::AuditLogResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListLongTasksQuery {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 128))]
    pub task_type: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateLongTaskRequest {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub task_type: String,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(range(min = 0, max = 100))]
    pub progress: Option<i32>,
    #[validate(range(min = 1))]
    pub total_steps: Option<i32>,
    #[validate(length(max = 255))]
    pub current_step: Option<String>,
    #[validate(length(max = 64))]
    pub role: Option<String>,
    pub input_params: Option<Value>,
    pub result: Option<Value>,
    #[validate(length(max = 40000))]
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLongTaskRequest {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(range(min = 0, max = 100))]
    pub progress: Option<i32>,
    #[validate(range(min = 1))]
    pub total_steps: Option<i32>,
    #[validate(length(max = 255))]
    pub current_step: Option<String>,
    #[validate(length(max = 64))]
    pub role: Option<String>,
    pub result: Option<Value>,
    #[validate(length(max = 40000))]
    pub error_message: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CancelLongTaskRequest {
    #[validate(length(max = 40000))]
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DailyBriefingQuery {
    #[validate(length(max = 32))]
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListDailyBriefingRecordsQuery {
    pub archived: Option<bool>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 32))]
    pub date_from: Option<String>,
    #[validate(length(max = 32))]
    pub date_to: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertDailyBriefingRecordRequest {
    #[validate(length(min = 1, max = 32))]
    pub briefing_date: String,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 40000))]
    pub summary: Option<String>,
    pub sections: Option<Value>,
    pub product_summary: Option<Value>,
    pub alert_summary: Option<Value>,
    pub task_summary: Option<Value>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListBackgroundEventsQuery {
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 128))]
    pub event_type: Option<String>,
    pub consumed: Option<bool>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackgroundEventRequest {
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(min = 1, max = 128))]
    pub event_type: String,
    #[validate(length(max = 255))]
    pub title: Option<String>,
    pub data: Option<Value>,
    pub consumed: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeBackgroundEventsRequest {
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 128))]
    pub event_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListAlertsQuery {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub alert_type: Option<String>,
    #[validate(length(max = 32))]
    pub severity: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub include_snoozed: Option<bool>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertRequest {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub alert_type: String,
    #[validate(length(max = 32))]
    pub severity: Option<String>,
    #[validate(length(min = 1, max = 40000))]
    pub message: String,
    #[validate(length(max = 64))]
    pub role: Option<String>,
    #[validate(length(max = 255))]
    pub dedup_key: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub snoozed_until: Option<OffsetDateTime>,
    pub payload: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SnoozeAlertRequest {
    #[serde(with = "time::serde::rfc3339")]
    pub snoozed_until: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DismissAlertRequest {
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ActAlertRequest {
    pub result: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DismissAllAlertsRequest {
    #[validate(length(max = 64))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 64))]
    pub alert_type: Option<String>,
    #[validate(length(max = 32))]
    pub severity: Option<String>,
}

pub type LongTaskListResponse = service::dto::briefing_ops::LongTaskListResponse;
pub type LongTaskResponse = service::dto::briefing_ops::LongTaskResponse;
pub type DailyBriefingSummaryResponse = service::dto::briefing_ops::DailyBriefingSummaryResponse;
pub type DailyBriefingListResponse = service::dto::briefing_ops::DailyBriefingListResponse;
pub type DailyBriefingResponse = service::dto::briefing_ops::DailyBriefingResponse;
pub type BackgroundEventListResponse = service::dto::briefing_ops::BackgroundEventListResponse;
pub type BackgroundEventResponse = service::dto::briefing_ops::BackgroundEventResponse;
pub type BatchUpdateResponse = service::dto::briefing_ops::BatchUpdateResponse;
pub type AlertListResponse = service::dto::briefing_ops::AlertListResponse;
pub type AlertSummaryResponse = service::dto::briefing_ops::AlertSummaryResponse;
pub type AlertResponse = service::dto::briefing_ops::AlertResponse;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTaskListResponse {
    pub tasks: Vec<WorkspaceTaskResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRunListResponse {
    pub runs: Vec<WorkspaceRunResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceArtifactListResponse {
    pub artifacts: Vec<WorkspaceArtifactResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

pub type AgentQualityCheckListResponse = service::dto::workspace::AgentQualityCheckListResponse;
pub type AgentQualityEvaluationResponse = service::dto::workspace::AgentQualityEvaluationResponse;
pub type AgentRunFeedbackResponse = service::dto::workspace::AgentRunFeedbackResponse;
pub type AgentLearningRecordListResponse = service::dto::workspace::AgentLearningRecordListResponse;
pub type AgentTrustScoreListResponse = service::dto::workspace::AgentTrustScoreListResponse;
pub type AgentSkillRunListResponse = service::dto::workspace::AgentSkillRunListResponse;
pub type WorkspaceArtifactExportResponse = service::dto::workspace::WorkspaceArtifactExportResponse;
pub type WorkspaceContextSummaryResponse = service::dto::workspace::WorkspaceContextSummaryResponse;
pub type WorkspaceContextPackResponse = service::dto::workspace::WorkspaceContextPackResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextSummaryQuery {
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 3400))]
    pub context_refs: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextPackQuery {
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(max = 64))]
    pub run_id: Option<String>,
    #[validate(length(max = 3400))]
    pub context_refs: Option<String>,
    #[validate(range(min = 1, max = 50))]
    pub max_items: Option<usize>,
    #[validate(range(min = 1, max = 24000))]
    pub max_chars: Option<usize>,
    #[validate(length(max = 20000))]
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewRequest {
    #[validate(length(min = 1, max = 128))]
    pub workspace_id: Option<String>,
    #[validate(length(max = 64))]
    pub conversation_id: Option<String>,
    #[validate(length(max = 64))]
    pub product_id: Option<String>,
    #[validate(length(min = 1, max = 20000))]
    pub input: String,
    #[serde(default)]
    pub context_refs: Vec<String>,
    #[validate(range(min = 1, max = 50))]
    pub max_items: Option<usize>,
    #[validate(range(min = 1, max = 24000))]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewResponse {
    pub workspace_context: AgentContextPreviewWorkspaceContextResponse,
    pub workspace_pack: AgentContextPreviewWorkspacePackResponse,
    pub conversation_history: Option<AgentContextPreviewConversationHistoryResponse>,
    pub conversation_memory: Option<AgentContextPreviewConversationMemoryResponse>,
    pub memory_records: Vec<AgentContextPreviewMemoryRecordResponse>,
    pub budget: Value,
    pub included_refs: Vec<String>,
    pub metadata_only_refs: Vec<String>,
    pub omitted_refs: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewWorkspaceContextResponse {
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub run_id: Option<String>,
    pub readiness: String,
    pub counts: service::dto::workspace::WorkspaceContextSummaryCounts,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
    pub source_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewWorkspacePackResponse {
    pub workspace_id: String,
    pub conversation_id: Option<String>,
    pub product_id: Option<String>,
    pub run_id: Option<String>,
    pub readiness: String,
    pub counts: service::dto::workspace::WorkspaceContextSummaryCounts,
    pub item_count: usize,
    pub readable_item_count: usize,
    pub text_chars: usize,
    pub omitted: Vec<String>,
    pub omitted_details: Vec<service::dto::workspace::WorkspaceContextPackOmittedResponse>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewConversationHistoryResponse {
    pub conversation_id: Option<String>,
    pub message_count: usize,
    pub total_message_count: usize,
    pub truncated: bool,
    pub relevant_older_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewConversationMemoryResponse {
    pub kind: String,
    pub rolling_summary_available: bool,
    pub fact_count: usize,
    pub decision_count: usize,
    pub constraint_count: usize,
    pub open_question_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentContextPreviewMemoryRecordResponse {
    pub kind: String,
    pub key: Option<String>,
    pub tags: Vec<String>,
    pub char_count: usize,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionQueueListResponse {
    pub items: Vec<ExecutionQueueResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: Option<String>,
    pub campaign_id: Option<String>,
    pub title: String,
    pub workspace_type: String,
    pub status: String,
    pub config: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTaskResponse {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_role: Option<String>,
    pub status: String,
    pub priority: i32,
    pub depends_on: Value,
    pub acceptance_criteria: Option<String>,
    pub result: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRunResponse {
    pub id: String,
    pub workspace_id: String,
    pub user_id: String,
    pub goal: String,
    pub requirements: Value,
    pub status: String,
    pub source: String,
    pub summary: Option<String>,
    pub report: Value,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRunTaskResponse {
    pub id: String,
    pub run_id: String,
    pub workspace_id: String,
    pub workspace_task_id: Option<String>,
    pub user_id: String,
    pub seq: i32,
    pub owner_role: Option<String>,
    pub status: String,
    pub attempts: i32,
    pub acceptance_score: f64,
    pub acceptance_passed: bool,
    pub acceptance_notes: Option<String>,
    pub execution_notes: Option<String>,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceArtifactResponse {
    pub id: String,
    pub workspace_id: String,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub run_task_id: Option<String>,
    pub user_id: String,
    pub artifact_type: String,
    pub title: String,
    pub content: Option<String>,
    pub preview: Option<String>,
    pub mime_type: Option<String>,
    pub format: String,
    pub storage_kind: String,
    pub bucket: Option<String>,
    pub object_key: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub sha256: Option<String>,
    pub source: String,
    pub status: String,
    pub validation_status: String,
    pub validation_report: Value,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionQueueResponse {
    pub id: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub run_id: Option<String>,
    pub workspace_task_id: Option<String>,
    pub action_type: String,
    pub description: String,
    pub platform: Option<String>,
    pub payload: Value,
    pub expected_impact: Option<String>,
    pub status: String,
    pub decision_reason: Option<String>,
    pub result: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub decided_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListPlatformConnectionsQuery {
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PlatformExecutionPreflightRequest {
    #[validate(length(min = 1, max = 128))]
    pub skill_name: String,
    pub args: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SavePlatformConnectionRequest {
    #[validate(length(min = 1, max = 64))]
    pub platform: String,
    #[validate(length(max = 255))]
    pub app_key: Option<String>,
    #[validate(length(max = 40000))]
    pub app_secret: Option<String>,
    #[validate(length(max = 40000))]
    pub access_token: Option<String>,
    #[validate(length(max = 40000))]
    pub refresh_token: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub token_expires_at: Option<OffsetDateTime>,
    #[validate(length(max = 255))]
    pub shop_id: Option<String>,
    #[validate(length(max = 255))]
    pub shop_name: Option<String>,
    pub enabled: Option<bool>,
    pub test_result: Option<Value>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_synced_at: Option<OffsetDateTime>,
    #[validate(length(max = 32))]
    pub sync_status: Option<String>,
    #[validate(length(max = 40000))]
    pub sync_message: Option<String>,
    pub extra: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlatformConnectionRequest {
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 255))]
    pub app_key: Option<String>,
    #[validate(length(max = 40000))]
    pub app_secret: Option<String>,
    #[validate(length(max = 40000))]
    pub access_token: Option<String>,
    #[validate(length(max = 40000))]
    pub refresh_token: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub token_expires_at: Option<OffsetDateTime>,
    #[validate(length(max = 255))]
    pub shop_id: Option<String>,
    #[validate(length(max = 255))]
    pub shop_name: Option<String>,
    pub enabled: Option<bool>,
    pub test_result: Option<Value>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_synced_at: Option<OffsetDateTime>,
    #[validate(length(max = 32))]
    pub sync_status: Option<String>,
    #[validate(length(max = 40000))]
    pub sync_message: Option<String>,
    pub extra: Option<Value>,
}

pub type PlatformCatalogResponse = service::dto::platform_connect::PlatformCatalogResponse;
pub type PlatformConnectionListResponse =
    service::dto::platform_connect::PlatformConnectionListResponse;
pub type PlatformConnectionResponse = service::dto::platform_connect::PlatformConnectionResponse;
pub type PlatformExecutionAdapterCatalogResponse =
    service::dto::platform_connect::PlatformExecutionAdapterCatalogResponse;
pub type PlatformExecutionPreflightResponse =
    service::dto::platform_connect::PlatformExecutionPreflightResponse;
pub type PlatformSyncStatusListResponse =
    service::dto::platform_connect::PlatformSyncStatusListResponse;
pub type PlatformSummaryResponse = service::dto::platform_connect::PlatformSummaryResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelConnectionsQuery {
    #[validate(length(max = 64))]
    pub channel: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SaveChannelConnectionRequest {
    #[validate(length(min = 1, max = 64))]
    pub channel: String,
    #[validate(length(max = 40000))]
    pub webhook_url: Option<String>,
    #[validate(length(max = 40000))]
    pub bot_token: Option<String>,
    #[validate(length(max = 255))]
    pub chat_id: Option<String>,
    #[validate(length(max = 40000))]
    pub secret: Option<String>,
    pub extra_credentials: Option<Value>,
    pub enabled: Option<bool>,
    #[validate(length(max = 40000))]
    pub last_error: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelConnectionRequest {
    #[validate(length(max = 64))]
    pub channel: Option<String>,
    #[validate(length(max = 40000))]
    pub webhook_url: Option<String>,
    #[validate(length(max = 40000))]
    pub bot_token: Option<String>,
    #[validate(length(max = 255))]
    pub chat_id: Option<String>,
    #[validate(length(max = 40000))]
    pub secret: Option<String>,
    pub extra_credentials: Option<Value>,
    pub enabled: Option<bool>,
    #[validate(length(max = 40000))]
    pub last_error: Option<String>,
}

pub type ChannelCatalogResponse = service::dto::platform_connect::ChannelCatalogResponse;
pub type ChannelConnectionListResponse =
    service::dto::platform_connect::ChannelConnectionListResponse;
pub type ChannelConnectionResponse = service::dto::platform_connect::ChannelConnectionResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListPackageCatalogQuery {
    #[validate(length(max = 32))]
    pub package_type: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    pub include_disabled: Option<bool>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPackageRequest {
    #[validate(length(min = 1, max = 32))]
    pub package_type: String,
    #[validate(length(min = 1, max = 128))]
    pub package_id: String,
    #[validate(length(max = 255))]
    pub package_name: Option<String>,
    #[validate(length(max = 64))]
    pub version: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 2048))]
    pub manifest_path: Option<String>,
    pub dependencies: Option<Value>,
    pub compatibility: Option<Value>,
    pub metadata: Option<Value>,
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePackageRequest {
    #[validate(length(max = 255))]
    pub package_name: Option<String>,
    #[validate(length(max = 64))]
    pub version: Option<String>,
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 2048))]
    pub manifest_path: Option<String>,
    pub dependencies: Option<Value>,
    pub compatibility: Option<Value>,
    pub metadata: Option<Value>,
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PackageStatusChangeRequest {
    #[validate(length(max = 40000))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SyncLegacyBuiltinPackagesRequest {
    pub reset_status: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListPackageHistoryQuery {
    #[validate(range(min = 1, max = 500))]
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SavePackageLocksRequest {
    pub entries: Vec<PackageLockInput>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PackageLockInput {
    #[validate(length(min = 1, max = 32))]
    pub package_type: String,
    #[validate(length(min = 1, max = 128))]
    pub package_id: String,
    #[validate(length(max = 255))]
    pub package_name: Option<String>,
    #[validate(length(max = 64))]
    pub locked_version: Option<String>,
    #[validate(length(max = 32))]
    pub locked_status: Option<String>,
    pub required: Option<bool>,
    #[validate(length(max = 128))]
    pub lock_source: Option<String>,
    #[validate(length(max = 2048))]
    pub manifest_path: Option<String>,
    #[validate(length(max = 40000))]
    pub notes: Option<String>,
    pub metadata: Option<Value>,
}

pub type PackageCatalogResponse = service::dto::package_governance::PackageCatalogResponse;
pub type PackageResponse = service::dto::package_governance::PackageResponse;
pub type PackageValidationResponse = service::dto::package_governance::PackageValidationResponse;
pub type PackageHistoryListResponse = service::dto::package_governance::PackageHistoryListResponse;
pub type PackageLockListResponse = service::dto::package_governance::PackageLockListResponse;
pub type PackageLockDriftResponse = service::dto::package_governance::PackageLockDriftResponse;
pub type SyncLegacyBuiltinPackagesResponse =
    service::dto::package_governance::SyncLegacyBuiltinPackagesResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListStoreMetricsQuery {
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 128))]
    pub metric_key: Option<String>,
    #[validate(length(max = 32))]
    pub metric_date_from: Option<String>,
    #[validate(length(max = 32))]
    pub metric_date_to: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SaveStoreMetricRequest {
    #[validate(length(min = 1, max = 64))]
    pub platform: String,
    #[validate(length(min = 1, max = 32))]
    pub metric_date: String,
    #[validate(length(min = 1, max = 128))]
    pub metric_key: String,
    pub metric_value: f64,
    #[validate(length(max = 128))]
    pub dimension_key: Option<String>,
    #[validate(length(max = 255))]
    pub dimension_value: Option<String>,
    #[validate(length(max = 64))]
    pub source: Option<String>,
}

pub type StoreMetricListResponse = service::dto::platform_connect::StoreMetricListResponse;
pub type StoreMetricResponse = service::dto::platform_connect::StoreMetricResponse;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListProductKnowledgeQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub content_type: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductKnowledgeRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 64))]
    pub content_type: Option<String>,
    #[validate(length(min = 1, max = 20000))]
    pub content: String,
    #[validate(length(max = 64))]
    pub source_type: Option<String>,
    #[validate(length(max = 128))]
    pub source_id: Option<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductKnowledgeRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 64))]
    pub content_type: Option<String>,
    #[validate(length(max = 20000))]
    pub content: Option<String>,
    #[validate(length(max = 64))]
    pub source_type: Option<String>,
    #[validate(length(max = 128))]
    pub source_id: Option<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductKnowledgeResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub title: Option<String>,
    pub content_type: String,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub confidence: f64,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListProductAssetsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub asset_type: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PresignProductAssetUploadRequest {
    #[validate(length(min = 1, max = 32))]
    pub extension: String,
    #[validate(length(max = 128))]
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignProductAssetUploadResponse {
    pub method: String,
    pub upload_url: String,
    pub bucket: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub expires_secs: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterProductAssetRequest {
    #[validate(length(min = 1, max = 128))]
    pub bucket: String,
    #[validate(length(min = 1, max = 1024))]
    pub object_key: String,
    #[validate(length(max = 64))]
    pub asset_type: Option<String>,
    #[validate(length(max = 255))]
    pub file_name: Option<String>,
    #[validate(length(max = 128))]
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductAssetRequest {
    #[validate(length(max = 64))]
    pub asset_type: Option<String>,
    #[validate(length(max = 255))]
    pub file_name: Option<String>,
    #[validate(length(max = 128))]
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    #[validate(length(max = 20000))]
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductAssetResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub bucket: String,
    pub object_key: String,
    pub asset_type: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub description: Option<String>,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductAssetAccessResponse {
    pub asset: ProductAssetResponse,
    pub method: String,
    pub download_url: String,
    pub expires_secs: u64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListProductMaterialsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub material_type: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductMaterialRequest {
    #[validate(length(min = 1, max = 64))]
    pub material_type: String,
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 40000))]
    pub content: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductMaterialRequest {
    #[validate(length(max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 40000))]
    pub content: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductMaterialResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub material_type: String,
    pub title: Option<String>,
    pub version: i32,
    pub content: String,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListProductCompetitorsQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 255))]
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorAnalysisQuery {
    #[validate(length(max = 32))]
    pub status: Option<String>,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(range(min = 1, max = 500))]
    pub change_limit: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductCompetitorRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 2048))]
    pub url: Option<String>,
    #[validate(range(min = 0.0))]
    pub price: Option<f64>,
    #[validate(range(min = 0.0, max = 5.0))]
    pub rating: Option<f64>,
    #[validate(length(max = 64))]
    pub monthly_sales: Option<String>,
    #[validate(length(max = 20000))]
    pub notes: Option<String>,
    pub search_results: Option<Value>,
    pub metadata: Option<Value>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductCompetitorRequest {
    #[validate(length(max = 255))]
    pub name: Option<String>,
    #[validate(length(max = 64))]
    pub platform: Option<String>,
    #[validate(length(max = 2048))]
    pub url: Option<String>,
    #[validate(range(min = 0.0))]
    pub price: Option<f64>,
    #[validate(range(min = 0.0, max = 5.0))]
    pub rating: Option<f64>,
    #[validate(length(max = 64))]
    pub monthly_sales: Option<String>,
    #[validate(length(max = 20000))]
    pub notes: Option<String>,
    pub search_results: Option<Value>,
    pub metadata: Option<Value>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_searched_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ListCompetitorChangesQuery {
    #[validate(length(max = 64))]
    pub competitor_id: Option<String>,
    #[validate(length(max = 64))]
    pub change_type: Option<String>,
    #[validate(range(min = 1, max = 500))]
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompetitorChangeRequest {
    #[validate(length(min = 1, max = 64))]
    pub change_type: String,
    #[validate(length(max = 20000))]
    pub old_value: Option<String>,
    #[validate(length(max = 20000))]
    pub new_value: Option<String>,
    #[validate(length(max = 20000))]
    pub summary: Option<String>,
    #[validate(length(max = 64))]
    pub source_type: Option<String>,
    pub metadata: Option<Value>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub detected_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCompetitorResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub name: String,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub rating: Option<f64>,
    pub monthly_sales: Option<String>,
    pub notes: Option<String>,
    pub search_results: Value,
    pub status: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_searched_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitorChangeResponse {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub competitor_id: String,
    pub competitor_name: Option<String>,
    pub competitor_platform: Option<String>,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub summary: Option<String>,
    pub source_type: String,
    pub metadata: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub detected_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub type ProductCompetitorAnalysisResponse =
    service::dto::product_competitor::ProductCompetitorAnalysisResponse;
