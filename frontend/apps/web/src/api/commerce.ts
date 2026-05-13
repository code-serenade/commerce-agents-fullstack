import { request } from "@workspace/services/api/base"

export type JsonRecord = Record<string, unknown>

export interface ListQuery {
  keyword?: string
  status?: string
  page?: number
  pageSize?: number
  [key: string]: string | number | boolean | undefined
}

export interface ProductResponse extends JsonRecord {
  id: string
  name: string
  platform?: string | null
  externalId?: string | null
  category?: string | null
  brand?: string | null
  status: string
  description?: string | null
  price?: string | null
  currency?: string | null
  createdAt?: string
  updatedAt?: string
}

interface ProductDetailResponse extends JsonRecord {
  product: ProductResponse
}

export interface ProductDashboardResponse extends JsonRecord {
  total: number
  draft: number
  listing: number
  active: number
  archived: number
}

export interface ProductLifecycleStatusResponse extends JsonRecord {
  status: string
  label: string
  description: string
  terminal: boolean
}

export interface ProductLifecycleTransitionResponse extends JsonRecord {
  from: string
  to: string
  label: string
  requiresReason: boolean
}

export interface ProductLifecycleOptionsResponse extends JsonRecord {
  statuses: ProductLifecycleStatusResponse[]
  transitions: ProductLifecycleTransitionResponse[]
  agentContext: JsonRecord
}

export interface ProductLifecycleResponse extends JsonRecord {
  product: ProductResponse
  currentStatus: string
  allowedNextStatuses: string[]
  transitions: ProductLifecycleTransitionResponse[]
  warnings: string[]
  agentContext: JsonRecord
}

export interface ProductKnowledgeResponse extends JsonRecord {
  id: string
  productId: string
  title?: string | null
  contentType: string
  content: string
  sourceType?: string | null
  confidence?: number | null
  status: string
  createdAt?: string
  updatedAt?: string
}

export interface ProductAssetResponse extends JsonRecord {
  id: string
  productId: string
  assetType: string
  fileName: string
  objectKey?: string
  mimeType?: string | null
  fileSize?: number | null
  description?: string | null
  status: string
  createdAt?: string
  updatedAt?: string
}

export interface ProductAssetUploadSignResponse extends JsonRecord {
  method: "PUT"
  uploadUrl: string
  bucket: string
  objectKey: string
  contentType: string
  expiresSecs: number
}

export interface ProductAssetAccessResponse extends JsonRecord {
  asset: ProductAssetResponse
  method: "GET"
  downloadUrl: string
  expiresSecs: number
}

export interface ProductMaterialResponse extends JsonRecord {
  id: string
  productId: string
  materialType: string
  title?: string | null
  content?: string | null
  version?: number
  status: string
  createdAt?: string
  updatedAt?: string
}

export interface ProductCompetitorResponse extends JsonRecord {
  id: string
  productId: string
  name: string
  platform?: string | null
  url?: string | null
  price?: number | null
  rating?: number | null
  monthlySales?: string | null
  notes?: string | null
  status: string
  lastSearchedAt?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface CompetitorChangeResponse extends JsonRecord {
  id: string
  competitorId: string
  productId: string
  changeType: string
  oldValue?: string | null
  newValue?: string | null
  summary?: string | null
  sourceType?: string | null
  detectedAt?: string | null
  createdAt?: string
}

export interface ProductCompetitorAnalysisCounts extends JsonRecord {
  competitors: number
  pricedCompetitors: number
  ratedCompetitors: number
  monthlySalesCompetitors: number
  recentChanges: number
}

export interface ProductCompetitorPriceAnalysisResponse extends JsonRecord {
  productPrice?: number | null
  minCompetitorPrice?: number | null
  maxCompetitorPrice?: number | null
  avgCompetitorPrice?: number | null
  pricePosition?: string | null
  gapToAvg?: number | null
  gapPercentToAvg?: number | null
  lowerPricedCompetitors: number
  higherPricedCompetitors: number
}

export interface ProductCompetitorRatingAnalysisResponse extends JsonRecord {
  minRating?: number | null
  maxRating?: number | null
  avgRating?: number | null
  ratedCompetitors: number
}

export interface ProductCompetitorPlatformAnalysisResponse extends JsonRecord {
  platform: string
  competitors: number
  avgPrice?: number | null
  avgRating?: number | null
}

export interface ProductCompetitorAnalysisRefResponse extends JsonRecord {
  refType: string
  id: string
  title: string
  refKey: string
}

export interface ProductCompetitorAnalysisProductResponse extends JsonRecord {
  id: string
  name: string
  platform?: string | null
  category?: string | null
  brand?: string | null
  price?: string | null
  numericPrice?: number | null
  currency?: string | null
}

export interface ProductCompetitorAnalysisResponse extends JsonRecord {
  product: ProductCompetitorAnalysisProductResponse
  readiness: string
  summary: string
  counts: ProductCompetitorAnalysisCounts
  price: ProductCompetitorPriceAnalysisResponse
  rating: ProductCompetitorRatingAnalysisResponse
  platforms: ProductCompetitorPlatformAnalysisResponse[]
  topCompetitors: ProductCompetitorResponse[]
  recentChanges: CompetitorChangeResponse[]
  refs: ProductCompetitorAnalysisRefResponse[]
  signals: string[]
  warnings: string[]
  agentContext: JsonRecord
}

export interface CampaignResponse extends JsonRecord {
  id: string
  productId?: string | null
  name: string
  campaignType?: string | null
  description?: string | null
  budget?: number | null
  startDate?: string | null
  endDate?: string | null
  status: string
  createdAt?: string
  updatedAt?: string
}

export interface CampaignListResponse extends JsonRecord {
  campaigns: CampaignResponse[]
  total: number
  page: number
  pageSize: number
}

export interface CampaignDraftSourceResponse extends JsonRecord {
  sourceType: string
  productId?: string | null
  workspaceId?: string | null
  sourceArtifactId?: string | null
  sourceSummary?: string | null
  refKeys: string[]
}

export interface CampaignDraftResponse extends JsonRecord {
  campaign: CampaignResponse
  source: CampaignDraftSourceResponse
  warnings: string[]
  agentContext: JsonRecord
}

export interface WorkspaceResponse extends JsonRecord {
  id: string
  productId?: string | null
  campaignId?: string | null
  title: string
  workspaceType?: string | null
  status: string
  createdAt?: string
  updatedAt?: string
}

export interface WorkspaceListResponse extends JsonRecord {
  workspaces: WorkspaceResponse[]
  total: number
  page: number
  pageSize: number
}

export interface WorkspaceTaskResponse extends JsonRecord {
  id: string
  workspaceId: string
  title: string
  description?: string | null
  ownerRole?: string | null
  status: string
  priority?: number | null
  acceptanceCriteria?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface WorkspaceTaskListResponse extends JsonRecord {
  tasks: WorkspaceTaskResponse[]
  total: number
  page: number
  pageSize: number
}

export interface WorkspaceRunResponse extends JsonRecord {
  id: string
  workspaceId: string
  goal: string
  status: string
  source?: string | null
  summary?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface WorkspaceRunListResponse extends JsonRecord {
  runs: WorkspaceRunResponse[]
  total: number
  page: number
  pageSize: number
}

export interface WorkspaceRunTaskResponse extends JsonRecord {
  id: string
  runId: string
  workspaceId: string
  workspaceTaskId?: string | null
  seq?: number | null
  ownerRole?: string | null
  status: string
  attempts?: number | null
  acceptanceScore?: number | null
  acceptancePassed?: boolean | null
  createdAt?: string
  updatedAt?: string
}

export interface AgentSkillRunResponse extends JsonRecord {
  id: string
  userId: string
  workspaceId: string
  runId: string
  sessionId?: string | null
  workerId: string
  taskId: string
  requestId: string
  skillId: string
  role: string
  status: string
  reason?: string | null
  summary?: string | null
  error?: string | null
  input: JsonRecord
  output: JsonRecord
  executionMode?: string | null
  riskLevel?: string | null
  approvalRequired: boolean
  metadata?: JsonRecord
  requestedAt?: string
  completedAt?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface AgentSkillRunListResponse extends JsonRecord {
  skillRuns: AgentSkillRunResponse[]
  total: number
  page: number
  pageSize: number
  agentContext?: JsonRecord
}

export interface WorkspaceArtifactResponse extends JsonRecord {
  id: string
  workspaceId: string
  runId?: string | null
  workspaceTaskId?: string | null
  runTaskId?: string | null
  artifactType: string
  title: string
  content?: string | null
  preview?: string | null
  mimeType?: string | null
  format: string
  storageKind: string
  bucket?: string | null
  objectKey?: string | null
  fileName?: string | null
  fileSize?: number | null
  sha256?: string | null
  source: string
  status: string
  validationStatus: string
  validationReport?: JsonRecord | null
  metadata?: JsonRecord | null
  createdAt?: string
  updatedAt?: string
}

export interface WorkspaceArtifactListResponse extends JsonRecord {
  artifacts: WorkspaceArtifactResponse[]
  total: number
  page: number
  pageSize: number
}

export interface WorkspaceArtifactExportResponse extends JsonRecord {
  artifactId: string
  workspaceId: string
  title: string
  fileName: string
  mimeType: string
  format: string
  content: string
  contentChars: number
  sourceFormat: string
  sourceStorageKind: string
  metadata?: JsonRecord | null
  warnings: string[]
}

export interface ExecutionQueueResponse extends JsonRecord {
  id: string
  workspaceId?: string | null
  runId?: string | null
  workspaceTaskId?: string | null
  actionType: string
  description: string
  platform?: string | null
  expectedImpact?: string | null
  status: string
  decisionReason?: string | null
  result?: JsonRecord
  createdAt?: string
  decidedAt?: string | null
}

export interface ExecutionQueueListResponse extends JsonRecord {
  items: ExecutionQueueResponse[]
  total: number
  page: number
  pageSize: number
}

export interface PlatformSpecResponse extends JsonRecord {
  platform: string
  display: string
  requiredFields?: JsonRecord[]
  docUrl?: string | null
}

export interface PlatformConnectionResponse extends JsonRecord {
  id: string
  platform: string
  display?: string | null
  appKey?: string | null
  shopId?: string | null
  shopName?: string | null
  enabled: boolean
  status: string
  syncStatus?: string | null
  syncMessage?: string | null
  lastSyncedAt?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface PlatformExecutionAdapterActionResponse extends JsonRecord {
  skillName: string
  display: string
  description: string
  riskLevel: string
  approvalRequired: boolean
  executionMode: string
  adapterStatus: string
  canExecuteNow: boolean
  readiness: string
  readinessMessage: string
  requiredParams: string[]
  optionalParams: string[]
  idempotencyKeyFields: string[]
}

export interface PlatformExecutionAdapterSpecResponse extends JsonRecord {
  platform: string
  display: string
  connected: boolean
  enabled: boolean
  connectionStatus: string
  actions: PlatformExecutionAdapterActionResponse[]
}

export interface PlatformExecutionAdapterCatalogResponse extends JsonRecord {
  adapters: PlatformExecutionAdapterSpecResponse[]
}

export interface ChannelSpecResponse extends JsonRecord {
  channel: string
  display: string
  requiredFields?: JsonRecord[]
  docUrl?: string | null
}

export interface ChannelConnectionResponse extends JsonRecord {
  id: string
  channel: string
  display?: string | null
  enabled: boolean
  status?: string | null
  lastError?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface StoreMetricResponse extends JsonRecord {
  id: string
  platform: string
  metricDate: string
  metricKey: string
  metricValue: number
  dimensionKey?: string | null
  dimensionValue?: string | null
  source?: string | null
  createdAt?: string
}

export interface StoreMetricListResponse extends JsonRecord {
  metrics: StoreMetricResponse[]
  total: number
  page: number
  pageSize: number
}

export interface PlatformSummaryItem extends JsonRecord {
  platform: string
  display: string
  connected: boolean
  enabled: boolean
  gmv7d?: number | null
  orders7d?: number | null
  gmv30d?: number | null
  orders30d?: number | null
  uv30d?: number | null
  adRoi30d?: number | null
}

export interface PlatformSyncStatusItem extends JsonRecord {
  platform: string
  display: string
  connected: boolean
  enabled: boolean
  syncStatus?: string | null
  syncMessage?: string | null
  latestMetricDate?: string | null
}

export interface ToolApprovalResponse extends JsonRecord {
  id: string
  workspaceId?: string | null
  runId?: string | null
  conversationId?: string | null
  requestRole?: string | null
  skillName: string
  toolCallId?: string | null
  riskLevel?: string | null
  status: string
  reason?: string | null
  actionKey?: string | null
  result?: JsonRecord
  approvedBy?: string | null
  approvedAt?: string | null
  executedAt?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface ToolApprovalListResponse extends JsonRecord {
  items: ToolApprovalResponse[]
  total: number
  page: number
  pageSize: number
}

export interface AuditLogResponse extends JsonRecord {
  id: string
  actorUserId?: string | null
  workspaceId?: string | null
  runId?: string | null
  action: string
  targetType: string
  targetId?: string | null
  status?: string | null
  reason?: string | null
  createdAt?: string
}

export interface AuditLogListResponse extends JsonRecord {
  logs: AuditLogResponse[]
  total: number
  page: number
  pageSize: number
}

export interface LongTaskResponse extends JsonRecord {
  id: string
  workspaceId?: string | null
  runId?: string | null
  taskType: string
  title: string
  status: string
  progress?: number | null
  totalSteps?: number | null
  currentStep?: string | null
  role?: string | null
  errorMessage?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface LongTaskListResponse extends JsonRecord {
  tasks: LongTaskResponse[]
  total: number
  page: number
  pageSize: number
}

export interface DailyBriefingResponse extends JsonRecord {
  id: string
  briefingDate: string
  title: string
  summary?: string | null
  source?: string | null
  status?: string | null
  archived?: boolean
  createdAt?: string
  updatedAt?: string
}

export interface DailyBriefingListResponse extends JsonRecord {
  records: DailyBriefingResponse[]
  total: number
  page: number
  pageSize: number
}

export interface DailyBriefingSummaryResponse extends JsonRecord {
  briefing?: DailyBriefingResponse | null
  summary?: string | null
  title?: string | null
  alertSummary?: JsonRecord
  taskSummary?: JsonRecord
}

export interface BackgroundEventResponse extends JsonRecord {
  id: string
  conversationId?: string | null
  eventType: string
  title: string
  consumed: boolean
  createdAt?: string
  consumedAt?: string | null
}

export interface BackgroundEventListResponse extends JsonRecord {
  events: BackgroundEventResponse[]
  total: number
  page: number
  pageSize: number
}

export interface AlertResponse extends JsonRecord {
  id: string
  workspaceId?: string | null
  runId?: string | null
  alertType: string
  severity: string
  message: string
  role?: string | null
  status: string
  snoozedUntil?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface AlertListResponse extends JsonRecord {
  alerts: AlertResponse[]
  total: number
  page: number
  pageSize: number
}

export interface AlertSummaryResponse extends JsonRecord {
  active: number
  bySeverity: JsonRecord
  byType: JsonRecord
}

export interface BatchUpdateResponse extends JsonRecord {
  updated: number
}

export interface PackageResponse extends JsonRecord {
  id?: string
  packageType: string
  packageId: string
  packageName: string
  version: string
  status: string
  manifestPath?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface PackageCatalogResponse extends JsonRecord {
  packages: PackageResponse[]
  sections?: JsonRecord
  total: number
  page: number
  pageSize: number
}

export interface PackageValidationResponse extends JsonRecord {
  valid: boolean
  errors: string[]
  warnings: string[]
  dependencies?: JsonRecord
}

export interface PackageHistoryResponse extends JsonRecord {
  id: string
  packageType: string
  packageId: string
  fromStatus?: string | null
  toStatus?: string | null
  reason?: string | null
  createdAt?: string
}

export interface PackageHistoryListResponse extends JsonRecord {
  history: PackageHistoryResponse[]
}

export interface PackageLockResponse extends JsonRecord {
  id?: string
  workspaceId: string
  runId: string
  packageType: string
  packageId: string
  packageName?: string | null
  lockedVersion: string
  lockedStatus: string
  required: boolean
  lockSource?: string | null
  manifestPath?: string | null
  notes?: string | null
  createdAt?: string
  updatedAt?: string
}

export interface PackageLockListResponse extends JsonRecord {
  locks: PackageLockResponse[]
}

export interface PackageLockDriftResponse extends JsonRecord {
  locks: PackageLockResponse[]
  drifts: JsonRecord[]
  summary: JsonRecord
}

export interface SyncLegacyBuiltinPackagesResponse extends JsonRecord {
  packages: PackageResponse[]
  createdCount: number
  updatedCount: number
  totalCount: number
  manifestBackedCount: number
  derivedCapabilityCount: number
  resetStatus: boolean
}

export interface AgentLearningRecordResponse extends JsonRecord {
  id: string
  userId?: string
  user_id?: string
  workspaceId?: string | null
  workspace_id?: string | null
  runId?: string | null
  run_id?: string | null
  qualityCheckId?: string | null
  quality_check_id?: string | null
  role: string
  category: string
  content: string
  confidence: number
  sourceAction?: string
  source_action?: string
  feedbackRating?: number | null
  feedback_rating?: number | null
  feedbackComment?: string | null
  feedback_comment?: string | null
  status: string
  metadata?: JsonRecord
  createdAt?: string
  created_at?: string
  updatedAt?: string
  updated_at?: string
}

interface AgentRunFeedbackWireResponse extends JsonRecord {
  learningRecord?: AgentLearningRecordResponse
  learning_record?: AgentLearningRecordResponse
}

export interface AgentRunFeedbackResponse extends JsonRecord {
  learningRecord: AgentLearningRecordResponse
  learning_record?: AgentLearningRecordResponse
}

export interface CommerceAgentPlaybookResponse extends JsonRecord {
  id: string
  title: string
  description: string
  category: string
  recommendedStrategy: string
  prompt: string
  requiredContext: string[]
  expectedOutputs: string[]
  roles: string[]
}

export interface CommerceAgentPlaybookListResponse extends JsonRecord {
  playbooks: CommerceAgentPlaybookResponse[]
}

export interface AgentRuntimeChoiceResponse extends JsonRecord {
  id: string
  label: string
  description: string
  isDefault: boolean
}

export interface AgentRuntimeModelOptionResponse extends JsonRecord {
  name: string
  model: string
  isDefault: boolean
}

export interface AgentRuntimeRoleOptionResponse extends JsonRecord {
  id: string
  name: string
  runtimeRole: string
  capabilities: string[]
  isDefault: boolean
}

export interface AgentRuntimeOptionsResponse extends JsonRecord {
  defaultModelName?: string | null
  models: AgentRuntimeModelOptionResponse[]
  strategies: AgentRuntimeChoiceResponse[]
  responseModes: AgentRuntimeChoiceResponse[]
  learningLevels: AgentRuntimeChoiceResponse[]
  collaborationModes: AgentRuntimeChoiceResponse[]
  roles: AgentRuntimeRoleOptionResponse[]
}

const defaultListQuery = (query?: ListQuery): ListQuery => ({
  page: 1,
  pageSize: 100,
  ...query,
})

const normalizeAgentLearningRecord = (
  record: AgentLearningRecordResponse
): AgentLearningRecordResponse => {
  const normalized = { ...record }
  copyIfMissing(normalized, "userId", "user_id")
  copyIfMissing(normalized, "workspaceId", "workspace_id")
  copyIfMissing(normalized, "runId", "run_id")
  copyIfMissing(normalized, "qualityCheckId", "quality_check_id")
  copyIfMissing(normalized, "sourceAction", "source_action")
  copyIfMissing(normalized, "feedbackRating", "feedback_rating")
  copyIfMissing(normalized, "feedbackComment", "feedback_comment")
  copyIfMissing(normalized, "createdAt", "created_at")
  copyIfMissing(normalized, "updatedAt", "updated_at")
  return normalized
}

const copyIfMissing = (
  target: JsonRecord,
  camelKey: string,
  snakeKey: string
) => {
  if (target[camelKey] === undefined && target[snakeKey] !== undefined) {
    target[camelKey] = target[snakeKey]
  }
}

export const listProductsApi = async (
  query?: ListQuery
): Promise<ProductResponse[]> =>
  request<ListQuery, ProductResponse[]>({
    method: "GET",
    url: "/api/app/products",
    body: query,
  })

export const getProductApi = async (
  productId: string
): Promise<ProductResponse> => {
  const detail = await request<undefined, ProductDetailResponse>({
    method: "GET",
    url: `/api/app/products/${productId}`,
  })
  return detail.product
}

export const createProductApi = async (
  body: JsonRecord
): Promise<ProductResponse> =>
  request<JsonRecord, ProductResponse>({
    method: "POST",
    url: "/api/app/products",
    body,
  })

export const updateProductApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductResponse> =>
  request<JsonRecord, ProductResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}`,
    body,
  })

export const archiveProductApi = async (productId: string): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/products/${productId}`,
  })

export const getProductLifecycleOptionsApi =
  async (): Promise<ProductLifecycleOptionsResponse> =>
    request<undefined, ProductLifecycleOptionsResponse>({
      method: "GET",
      url: "/api/app/products/lifecycle/options",
    })

export const getProductLifecycleApi = async (
  productId: string
): Promise<ProductLifecycleResponse> =>
  request<undefined, ProductLifecycleResponse>({
    method: "GET",
    url: `/api/app/products/${productId}/lifecycle`,
  })

export const transitionProductLifecycleApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductLifecycleResponse> =>
  request<JsonRecord, ProductLifecycleResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}/lifecycle`,
    body,
  })

export const getProductDashboardApi =
  async (): Promise<ProductDashboardResponse> =>
    request<undefined, ProductDashboardResponse>({
      method: "GET",
      url: "/api/app/products/dashboard",
    })

export const listCommerceAgentPlaybooksApi =
  async (): Promise<CommerceAgentPlaybookResponse[]> => {
    const response = await request<undefined, CommerceAgentPlaybookListResponse>({
      method: "GET",
      url: "/api/app/agent/playbooks",
    })
    return response.playbooks
  }

export const getAgentRuntimeOptionsApi =
  async (): Promise<AgentRuntimeOptionsResponse> =>
    request<undefined, AgentRuntimeOptionsResponse>({
      method: "GET",
      url: "/api/app/agent/runtime-options",
    })

export const listProductKnowledgeApi = async (
  productId: string,
  query?: ListQuery
): Promise<ProductKnowledgeResponse[]> =>
  request<ListQuery, ProductKnowledgeResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/knowledge`,
    body: query,
  })

export const createProductKnowledgeApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductKnowledgeResponse> =>
  request<JsonRecord, ProductKnowledgeResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/knowledge`,
    body,
  })

export const updateProductKnowledgeApi = async (
  productId: string,
  knowledgeId: string,
  body: JsonRecord
): Promise<ProductKnowledgeResponse> =>
  request<JsonRecord, ProductKnowledgeResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}/knowledge/${knowledgeId}`,
    body,
  })

export const archiveProductKnowledgeApi = async (
  productId: string,
  knowledgeId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/products/${productId}/knowledge/${knowledgeId}`,
  })

export const presignProductAssetUploadApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductAssetUploadSignResponse> =>
  request<JsonRecord, ProductAssetUploadSignResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/assets/presign-upload`,
    body,
  })

export const uploadProductAssetFileApi = async (
  file: File,
  sign: ProductAssetUploadSignResponse
): Promise<Response> => {
  const response = await fetch(sign.uploadUrl, {
    method: sign.method,
    headers: {
      "Content-Type": sign.contentType || file.type,
    },
    body: file,
  })

  if (!response.ok) {
    throw new Error(`Upload failed: ${response.status}`)
  }

  return response
}

export const listProductAssetsApi = async (
  productId: string,
  query?: ListQuery
): Promise<ProductAssetResponse[]> =>
  request<ListQuery, ProductAssetResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/assets`,
    body: query,
  })

export const createProductAssetApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductAssetResponse> =>
  request<JsonRecord, ProductAssetResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/assets`,
    body,
  })

export const accessProductAssetApi = async (
  productId: string,
  assetId: string
): Promise<ProductAssetAccessResponse> =>
  request<undefined, ProductAssetAccessResponse>({
    method: "GET",
    url: `/api/app/products/${productId}/assets/${assetId}/access`,
  })

export const updateProductAssetApi = async (
  productId: string,
  assetId: string,
  body: JsonRecord
): Promise<ProductAssetResponse> =>
  request<JsonRecord, ProductAssetResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}/assets/${assetId}`,
    body,
  })

export const archiveProductAssetApi = async (
  productId: string,
  assetId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/products/${productId}/assets/${assetId}`,
  })

export const listProductMaterialsApi = async (
  productId: string,
  query?: ListQuery
): Promise<ProductMaterialResponse[]> =>
  request<ListQuery, ProductMaterialResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/materials`,
    body: query,
  })

export const getProductMaterialApi = async (
  productId: string,
  materialId: string
): Promise<ProductMaterialResponse> =>
  request<undefined, ProductMaterialResponse>({
    method: "GET",
    url: `/api/app/products/${productId}/materials/${materialId}`,
  })

export const createProductMaterialApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductMaterialResponse> =>
  request<JsonRecord, ProductMaterialResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/materials`,
    body,
  })

export const updateProductMaterialApi = async (
  productId: string,
  materialId: string,
  body: JsonRecord
): Promise<ProductMaterialResponse> =>
  request<JsonRecord, ProductMaterialResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}/materials/${materialId}`,
    body,
  })

export const archiveProductMaterialApi = async (
  productId: string,
  materialId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/products/${productId}/materials/${materialId}`,
  })

export const listProductMaterialVersionsApi = async (
  productId: string,
  materialId: string
): Promise<ProductMaterialResponse[]> =>
  request<undefined, ProductMaterialResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/materials/${materialId}/versions`,
  })

export const listProductCompetitorsApi = async (
  productId: string,
  query?: ListQuery
): Promise<ProductCompetitorResponse[]> =>
  request<ListQuery, ProductCompetitorResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/competitors`,
    body: query,
  })

export const getProductCompetitorAnalysisApi = async (
  productId: string,
  query?: ListQuery
): Promise<ProductCompetitorAnalysisResponse> =>
  request<ListQuery, ProductCompetitorAnalysisResponse>({
    method: "GET",
    url: `/api/app/products/${productId}/competitors/analysis`,
    body: query,
  })

export const getProductCompetitorApi = async (
  productId: string,
  competitorId: string
): Promise<ProductCompetitorResponse> =>
  request<undefined, ProductCompetitorResponse>({
    method: "GET",
    url: `/api/app/products/${productId}/competitors/${competitorId}`,
  })

export const createProductCompetitorApi = async (
  productId: string,
  body: JsonRecord
): Promise<ProductCompetitorResponse> =>
  request<JsonRecord, ProductCompetitorResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/competitors`,
    body,
  })

export const updateProductCompetitorApi = async (
  productId: string,
  competitorId: string,
  body: JsonRecord
): Promise<ProductCompetitorResponse> =>
  request<JsonRecord, ProductCompetitorResponse>({
    method: "PATCH",
    url: `/api/app/products/${productId}/competitors/${competitorId}`,
    body,
  })

export const archiveProductCompetitorApi = async (
  productId: string,
  competitorId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/products/${productId}/competitors/${competitorId}`,
  })

export const listCompetitorChangesApi = async (
  productId: string,
  competitorId?: string,
  query?: ListQuery
): Promise<CompetitorChangeResponse[]> =>
  request<ListQuery, CompetitorChangeResponse[]>({
    method: "GET",
    url: `/api/app/products/${productId}/competitors/changes`,
    body: {
      ...query,
      competitorId,
    },
  })

export const createCompetitorChangeApi = async (
  productId: string,
  competitorId: string,
  body: JsonRecord
): Promise<CompetitorChangeResponse> =>
  request<JsonRecord, CompetitorChangeResponse>({
    method: "POST",
    url: `/api/app/products/${productId}/competitors/${competitorId}/changes`,
    body,
  })

export const listCampaignsApi = async (
  query?: ListQuery
): Promise<CampaignListResponse> =>
  request<ListQuery, CampaignListResponse>({
    method: "GET",
    url: "/api/app/campaigns",
    body: defaultListQuery(query),
  })

export const getCampaignApi = async (
  campaignId: string
): Promise<CampaignResponse> =>
  request<undefined, CampaignResponse>({
    method: "GET",
    url: `/api/app/campaigns/${campaignId}`,
  })

export const createCampaignApi = async (
  body: JsonRecord
): Promise<CampaignResponse> =>
  request<JsonRecord, CampaignResponse>({
    method: "POST",
    url: "/api/app/campaigns",
    body,
  })

export const createCampaignDraftApi = async (
  body: JsonRecord
): Promise<CampaignDraftResponse> =>
  request<JsonRecord, CampaignDraftResponse>({
    method: "POST",
    url: "/api/app/campaigns/draft",
    body,
  })

export const updateCampaignApi = async (
  campaignId: string,
  body: JsonRecord
): Promise<CampaignResponse> =>
  request<JsonRecord, CampaignResponse>({
    method: "PATCH",
    url: `/api/app/campaigns/${campaignId}`,
    body,
  })

export const archiveCampaignApi = async (campaignId: string): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/campaigns/${campaignId}`,
  })

export const listWorkspacesApi = async (
  query?: ListQuery
): Promise<WorkspaceListResponse> =>
  request<ListQuery, WorkspaceListResponse>({
    method: "GET",
    url: "/api/app/workspaces",
    body: defaultListQuery(query),
  })

export const getWorkspaceApi = async (
  workspaceId: string
): Promise<WorkspaceResponse> =>
  request<undefined, WorkspaceResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}`,
  })

export const createWorkspaceApi = async (
  body: JsonRecord
): Promise<WorkspaceResponse> =>
  request<JsonRecord, WorkspaceResponse>({
    method: "POST",
    url: "/api/app/workspaces",
    body,
  })

export const updateWorkspaceApi = async (
  workspaceId: string,
  body: JsonRecord
): Promise<WorkspaceResponse> =>
  request<JsonRecord, WorkspaceResponse>({
    method: "PATCH",
    url: `/api/app/workspaces/${workspaceId}`,
    body,
  })

export const archiveWorkspaceApi = async (
  workspaceId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/workspaces/${workspaceId}`,
  })

export const listWorkspaceTasksApi = async (
  workspaceId: string,
  query?: ListQuery
): Promise<WorkspaceTaskListResponse> =>
  request<ListQuery, WorkspaceTaskListResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/tasks`,
    body: defaultListQuery(query),
  })

export const getWorkspaceTaskApi = async (
  workspaceId: string,
  taskId: string
): Promise<WorkspaceTaskResponse> =>
  request<undefined, WorkspaceTaskResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/tasks/${taskId}`,
  })

export const createWorkspaceTaskApi = async (
  workspaceId: string,
  body: JsonRecord
): Promise<WorkspaceTaskResponse> =>
  request<JsonRecord, WorkspaceTaskResponse>({
    method: "POST",
    url: `/api/app/workspaces/${workspaceId}/tasks`,
    body,
  })

export const updateWorkspaceTaskApi = async (
  workspaceId: string,
  taskId: string,
  body: JsonRecord
): Promise<WorkspaceTaskResponse> =>
  request<JsonRecord, WorkspaceTaskResponse>({
    method: "PATCH",
    url: `/api/app/workspaces/${workspaceId}/tasks/${taskId}`,
    body,
  })

export const archiveWorkspaceTaskApi = async (
  workspaceId: string,
  taskId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/workspaces/${workspaceId}/tasks/${taskId}`,
  })

export const listWorkspaceRunsApi = async (
  workspaceId: string,
  query?: ListQuery
): Promise<WorkspaceRunListResponse> =>
  request<ListQuery, WorkspaceRunListResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs`,
    body: defaultListQuery(query),
  })

export const getWorkspaceRunApi = async (
  workspaceId: string,
  runId: string
): Promise<WorkspaceRunResponse> =>
  request<undefined, WorkspaceRunResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}`,
  })

export const createWorkspaceRunApi = async (
  workspaceId: string,
  body: JsonRecord
): Promise<WorkspaceRunResponse> =>
  request<JsonRecord, WorkspaceRunResponse>({
    method: "POST",
    url: `/api/app/workspaces/${workspaceId}/runs`,
    body,
  })

export const updateWorkspaceRunApi = async (
  workspaceId: string,
  runId: string,
  body: JsonRecord
): Promise<WorkspaceRunResponse> =>
  request<JsonRecord, WorkspaceRunResponse>({
    method: "PATCH",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}`,
    body,
  })

export const submitAgentRunFeedbackApi = async (
  workspaceId: string,
  runId: string,
  body: JsonRecord
): Promise<AgentRunFeedbackResponse> => {
  const response = await request<JsonRecord, AgentRunFeedbackWireResponse>({
    method: "POST",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/feedback`,
    body,
  })
  const learningRecord = response.learningRecord ?? response.learning_record

  if (!learningRecord) {
    throw new Error("Missing learning record in feedback response")
  }

  return {
    ...response,
    learningRecord: normalizeAgentLearningRecord(learningRecord),
  }
}

export const listWorkspaceRunTasksApi = async (
  workspaceId: string,
  runId: string,
  query?: ListQuery
): Promise<WorkspaceRunTaskResponse[]> =>
  request<ListQuery, WorkspaceRunTaskResponse[]>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/tasks`,
    body: query,
  })

export const listAgentSkillRunsApi = async (
  workspaceId: string,
  runId: string,
  query?: ListQuery
): Promise<AgentSkillRunListResponse> =>
  request<ListQuery, AgentSkillRunListResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/skill-runs`,
    body: defaultListQuery(query),
  })

export const createWorkspaceRunTaskApi = async (
  workspaceId: string,
  runId: string,
  body: JsonRecord
): Promise<WorkspaceRunTaskResponse> =>
  request<JsonRecord, WorkspaceRunTaskResponse>({
    method: "POST",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/tasks`,
    body,
  })

export const updateWorkspaceRunTaskApi = async (
  workspaceId: string,
  runId: string,
  runTaskId: string,
  body: JsonRecord
): Promise<WorkspaceRunTaskResponse> =>
  request<JsonRecord, WorkspaceRunTaskResponse>({
    method: "PATCH",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/tasks/${runTaskId}`,
    body,
  })

export const listWorkspaceArtifactsApi = async (
  workspaceId: string,
  query?: ListQuery
): Promise<WorkspaceArtifactListResponse> =>
  request<ListQuery, WorkspaceArtifactListResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/artifacts`,
    body: defaultListQuery(query),
  })

export const getWorkspaceArtifactApi = async (
  workspaceId: string,
  artifactId: string
): Promise<WorkspaceArtifactResponse> =>
  request<undefined, WorkspaceArtifactResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/artifacts/${artifactId}`,
  })

export const exportWorkspaceArtifactApi = async (
  workspaceId: string,
  artifactId: string,
  query?: { format?: string; includeMetadata?: boolean }
): Promise<WorkspaceArtifactExportResponse> =>
  request<{ format?: string; includeMetadata?: boolean }, WorkspaceArtifactExportResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/artifacts/${artifactId}/export`,
    body: query,
  })

export const listExecutionQueueApi = async (
  query?: ListQuery
): Promise<ExecutionQueueListResponse> =>
  request<ListQuery, ExecutionQueueListResponse>({
    method: "GET",
    url: "/api/app/execution-queue",
    body: defaultListQuery(query),
  })

export const getExecutionQueueApi = async (
  executionId: string
): Promise<ExecutionQueueResponse> =>
  request<undefined, ExecutionQueueResponse>({
    method: "GET",
    url: `/api/app/execution-queue/${executionId}`,
  })

export const createExecutionQueueApi = async (
  body: JsonRecord
): Promise<ExecutionQueueResponse> =>
  request<JsonRecord, ExecutionQueueResponse>({
    method: "POST",
    url: "/api/app/execution-queue",
    body,
  })

export const approveExecutionQueueApi = async (
  executionId: string,
  body: JsonRecord
): Promise<ExecutionQueueResponse> =>
  request<JsonRecord, ExecutionQueueResponse>({
    method: "POST",
    url: `/api/app/execution-queue/${executionId}/approve`,
    body,
  })

export const rejectExecutionQueueApi = async (
  executionId: string,
  body: JsonRecord
): Promise<ExecutionQueueResponse> =>
  request<JsonRecord, ExecutionQueueResponse>({
    method: "POST",
    url: `/api/app/execution-queue/${executionId}/reject`,
    body,
  })

export const listSupportedPlatformsApi = async (): Promise<{
  platforms: PlatformSpecResponse[]
}> =>
  request<undefined, { platforms: PlatformSpecResponse[] }>({
    method: "GET",
    url: "/api/app/platform-connect/platforms",
  })

export const listPlatformExecutionAdaptersApi =
  async (): Promise<PlatformExecutionAdapterCatalogResponse> =>
    request<undefined, PlatformExecutionAdapterCatalogResponse>({
      method: "GET",
      url: "/api/app/platform-connect/execution-adapters",
    })

export const listPlatformConnectionsApi = async (
  query?: ListQuery
): Promise<{ connections: PlatformConnectionResponse[] }> =>
  request<ListQuery, { connections: PlatformConnectionResponse[] }>({
    method: "GET",
    url: "/api/app/platform-connect/connections",
    body: query,
  })

export const getPlatformConnectionApi = async (
  connectionId: string
): Promise<PlatformConnectionResponse> =>
  request<undefined, PlatformConnectionResponse>({
    method: "GET",
    url: `/api/app/platform-connect/connections/${connectionId}`,
  })

export const savePlatformConnectionApi = async (
  body: JsonRecord
): Promise<PlatformConnectionResponse> =>
  request<JsonRecord, PlatformConnectionResponse>({
    method: "POST",
    url: "/api/app/platform-connect/connections",
    body,
  })

export const updatePlatformConnectionApi = async (
  connectionId: string,
  body: JsonRecord
): Promise<PlatformConnectionResponse> =>
  request<JsonRecord, PlatformConnectionResponse>({
    method: "PATCH",
    url: `/api/app/platform-connect/connections/${connectionId}`,
    body,
  })

export const disablePlatformConnectionApi = async (
  connectionId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/platform-connect/connections/${connectionId}`,
  })

export const listSupportedChannelsApi = async (): Promise<{
  channels: ChannelSpecResponse[]
}> =>
  request<undefined, { channels: ChannelSpecResponse[] }>({
    method: "GET",
    url: "/api/app/platform-connect/channels",
  })

export const listChannelConnectionsApi = async (
  query?: ListQuery
): Promise<{ connections: ChannelConnectionResponse[] }> =>
  request<ListQuery, { connections: ChannelConnectionResponse[] }>({
    method: "GET",
    url: "/api/app/platform-connect/channels/connections",
    body: query,
  })

export const getChannelConnectionApi = async (
  connectionId: string
): Promise<ChannelConnectionResponse> =>
  request<undefined, ChannelConnectionResponse>({
    method: "GET",
    url: `/api/app/platform-connect/channels/connections/${connectionId}`,
  })

export const saveChannelConnectionApi = async (
  body: JsonRecord
): Promise<ChannelConnectionResponse> =>
  request<JsonRecord, ChannelConnectionResponse>({
    method: "POST",
    url: "/api/app/platform-connect/channels/connections",
    body,
  })

export const updateChannelConnectionApi = async (
  connectionId: string,
  body: JsonRecord
): Promise<ChannelConnectionResponse> =>
  request<JsonRecord, ChannelConnectionResponse>({
    method: "PATCH",
    url: `/api/app/platform-connect/channels/connections/${connectionId}`,
    body,
  })

export const disableChannelConnectionApi = async (
  connectionId: string
): Promise<null> =>
  request<undefined, null>({
    method: "DELETE",
    url: `/api/app/platform-connect/channels/connections/${connectionId}`,
  })

export const listStoreMetricsApi = async (
  query?: ListQuery
): Promise<StoreMetricListResponse> =>
  request<ListQuery, StoreMetricListResponse>({
    method: "GET",
    url: "/api/app/platform-connect/metrics",
    body: defaultListQuery(query),
  })

export const saveStoreMetricApi = async (
  body: JsonRecord
): Promise<StoreMetricResponse> =>
  request<JsonRecord, StoreMetricResponse>({
    method: "POST",
    url: "/api/app/platform-connect/metrics",
    body,
  })

export const getPlatformSyncStatusApi = async (): Promise<{
  platforms: PlatformSyncStatusItem[]
}> =>
  request<undefined, { platforms: PlatformSyncStatusItem[] }>({
    method: "GET",
    url: "/api/app/platform-connect/sync-status",
  })

export const getPlatformSummaryApi = async (): Promise<{
  platforms: PlatformSummaryItem[]
}> =>
  request<undefined, { platforms: PlatformSummaryItem[] }>({
    method: "GET",
    url: "/api/app/platform-connect/summary",
  })

export const listToolApprovalsApi = async (
  query?: ListQuery
): Promise<ToolApprovalListResponse> =>
  request<ListQuery, ToolApprovalListResponse>({
    method: "GET",
    url: "/api/app/approvals/tool-requests",
    body: defaultListQuery(query),
  })

export const getToolApprovalApi = async (
  approvalId: string
): Promise<ToolApprovalResponse> =>
  request<undefined, ToolApprovalResponse>({
    method: "GET",
    url: `/api/app/approvals/tool-requests/${approvalId}`,
  })

export const createToolApprovalApi = async (
  body: JsonRecord
): Promise<ToolApprovalResponse> =>
  request<JsonRecord, ToolApprovalResponse>({
    method: "POST",
    url: "/api/app/approvals/tool-requests",
    body,
  })

export const approveToolApprovalApi = async (
  approvalId: string,
  body: JsonRecord
): Promise<ToolApprovalResponse> =>
  request<JsonRecord, ToolApprovalResponse>({
    method: "POST",
    url: `/api/app/approvals/tool-requests/${approvalId}/approve`,
    body,
  })

export const rejectToolApprovalApi = async (
  approvalId: string,
  body: JsonRecord
): Promise<ToolApprovalResponse> =>
  request<JsonRecord, ToolApprovalResponse>({
    method: "POST",
    url: `/api/app/approvals/tool-requests/${approvalId}/reject`,
    body,
  })

export const listAuditLogsApi = async (
  query?: ListQuery
): Promise<AuditLogListResponse> =>
  request<ListQuery, AuditLogListResponse>({
    method: "GET",
    url: "/api/app/approvals/audit-logs",
    body: defaultListQuery(query),
  })

export const createAuditLogApi = async (
  body: JsonRecord
): Promise<AuditLogResponse> =>
  request<JsonRecord, AuditLogResponse>({
    method: "POST",
    url: "/api/app/approvals/audit-logs",
    body,
  })

export const listLongTasksApi = async (
  query?: ListQuery
): Promise<LongTaskListResponse> =>
  request<ListQuery, LongTaskListResponse>({
    method: "GET",
    url: "/api/app/long-tasks",
    body: defaultListQuery(query),
  })

export const getLongTaskApi = async (
  taskId: string
): Promise<LongTaskResponse> =>
  request<undefined, LongTaskResponse>({
    method: "GET",
    url: `/api/app/long-tasks/${taskId}`,
  })

export const createLongTaskApi = async (
  body: JsonRecord
): Promise<LongTaskResponse> =>
  request<JsonRecord, LongTaskResponse>({
    method: "POST",
    url: "/api/app/long-tasks",
    body,
  })

export const updateLongTaskApi = async (
  taskId: string,
  body: JsonRecord
): Promise<LongTaskResponse> =>
  request<JsonRecord, LongTaskResponse>({
    method: "PATCH",
    url: `/api/app/long-tasks/${taskId}`,
    body,
  })

export const cancelLongTaskApi = async (
  taskId: string,
  body: JsonRecord
): Promise<LongTaskResponse> =>
  request<JsonRecord, LongTaskResponse>({
    method: "POST",
    url: `/api/app/long-tasks/${taskId}/cancel`,
    body,
  })

export const getDailyBriefingSummaryApi = async (
  query?: ListQuery
): Promise<DailyBriefingSummaryResponse> =>
  request<ListQuery, DailyBriefingSummaryResponse>({
    method: "GET",
    url: "/api/app/briefing/daily",
    body: query,
  })

export const listDailyBriefingsApi = async (
  query?: ListQuery
): Promise<DailyBriefingListResponse> =>
  request<ListQuery, DailyBriefingListResponse>({
    method: "GET",
    url: "/api/app/briefing/daily-records",
    body: defaultListQuery(query),
  })

export const getDailyBriefingRecordApi = async (
  briefingId: string
): Promise<DailyBriefingResponse> =>
  request<undefined, DailyBriefingResponse>({
    method: "GET",
    url: `/api/app/briefing/daily-records/${briefingId}`,
  })

export const saveDailyBriefingApi = async (
  body: JsonRecord
): Promise<DailyBriefingResponse> =>
  request<JsonRecord, DailyBriefingResponse>({
    method: "POST",
    url: "/api/app/briefing/daily-records",
    body,
  })

export const archiveDailyBriefingApi = async (
  briefingId: string
): Promise<DailyBriefingResponse> =>
  request<undefined, DailyBriefingResponse>({
    method: "POST",
    url: `/api/app/briefing/daily-records/${briefingId}/archive`,
  })

export const listBackgroundEventsApi = async (
  query?: ListQuery
): Promise<BackgroundEventListResponse> =>
  request<ListQuery, BackgroundEventListResponse>({
    method: "GET",
    url: "/api/app/briefing/notifications",
    body: defaultListQuery(query),
  })

export const createBackgroundEventApi = async (
  body: JsonRecord
): Promise<BackgroundEventResponse> =>
  request<JsonRecord, BackgroundEventResponse>({
    method: "POST",
    url: "/api/app/briefing/notifications",
    body,
  })

export const consumeBackgroundEventApi = async (
  eventId: string
): Promise<BackgroundEventResponse> =>
  request<undefined, BackgroundEventResponse>({
    method: "POST",
    url: `/api/app/briefing/notifications/${eventId}/consume`,
  })

export const consumeAllBackgroundEventsApi = async (
  body: JsonRecord
): Promise<BatchUpdateResponse> =>
  request<JsonRecord, BatchUpdateResponse>({
    method: "POST",
    url: "/api/app/briefing/notifications/consume-all",
    body,
  })

export const listAlertsApi = async (
  query?: ListQuery
): Promise<AlertListResponse> =>
  request<ListQuery, AlertListResponse>({
    method: "GET",
    url: "/api/app/briefing/alerts",
    body: defaultListQuery(query),
  })

export const getAlertApi = async (alertId: string): Promise<AlertResponse> =>
  request<undefined, AlertResponse>({
    method: "GET",
    url: `/api/app/briefing/alerts/${alertId}`,
  })

export const createAlertApi = async (
  body: JsonRecord
): Promise<AlertResponse> =>
  request<JsonRecord, AlertResponse>({
    method: "POST",
    url: "/api/app/briefing/alerts",
    body,
  })

export const getAlertSummaryApi = async (): Promise<AlertSummaryResponse> =>
  request<undefined, AlertSummaryResponse>({
    method: "GET",
    url: "/api/app/briefing/alerts/summary",
  })

export const dismissAllAlertsApi = async (
  body: JsonRecord
): Promise<BatchUpdateResponse> =>
  request<JsonRecord, BatchUpdateResponse>({
    method: "POST",
    url: "/api/app/briefing/alerts/dismiss-all",
    body,
  })

export const snoozeAlertApi = async (
  alertId: string,
  body: JsonRecord
): Promise<AlertResponse> =>
  request<JsonRecord, AlertResponse>({
    method: "POST",
    url: `/api/app/briefing/alerts/${alertId}/snooze`,
    body,
  })

export const dismissAlertApi = async (
  alertId: string,
  body: JsonRecord
): Promise<AlertResponse> =>
  request<JsonRecord, AlertResponse>({
    method: "POST",
    url: `/api/app/briefing/alerts/${alertId}/dismiss`,
    body,
  })

export const actAlertApi = async (
  alertId: string,
  body: JsonRecord
): Promise<AlertResponse> =>
  request<JsonRecord, AlertResponse>({
    method: "POST",
    url: `/api/app/briefing/alerts/${alertId}/act`,
    body,
  })

export const listPackageCatalogApi = async (
  query?: ListQuery
): Promise<PackageCatalogResponse> =>
  request<ListQuery, PackageCatalogResponse>({
    method: "GET",
    url: "/api/app/packages/catalog",
    body: defaultListQuery(query),
  })

export const getPackageApi = async (
  packageType: string,
  packageId: string
): Promise<PackageResponse> =>
  request<undefined, PackageResponse>({
    method: "GET",
    url: `/api/app/packages/${packageType}/${packageId}`,
  })

export const savePackageApi = async (
  body: JsonRecord
): Promise<PackageResponse> =>
  request<JsonRecord, PackageResponse>({
    method: "POST",
    url: "/api/app/packages",
    body,
  })

export const syncLegacyBuiltinPackagesApi = async (
  body: JsonRecord = {}
): Promise<SyncLegacyBuiltinPackagesResponse> =>
  request<JsonRecord, SyncLegacyBuiltinPackagesResponse>({
    method: "POST",
    url: "/api/app/packages/sync-legacy-builtins",
    body,
  })

export const updatePackageApi = async (
  packageType: string,
  packageId: string,
  body: JsonRecord
): Promise<PackageResponse> =>
  request<JsonRecord, PackageResponse>({
    method: "PATCH",
    url: `/api/app/packages/${packageType}/${packageId}`,
    body,
  })

export const validatePackageApi = async (
  packageType: string,
  packageId: string
): Promise<PackageValidationResponse> =>
  request<undefined, PackageValidationResponse>({
    method: "GET",
    url: `/api/app/packages/${packageType}/${packageId}/validate`,
  })

export const activatePackageApi = async (
  packageType: string,
  packageId: string,
  body: JsonRecord
): Promise<PackageResponse> =>
  request<JsonRecord, PackageResponse>({
    method: "POST",
    url: `/api/app/packages/${packageType}/${packageId}/activate`,
    body,
  })

export const deactivatePackageApi = async (
  packageType: string,
  packageId: string,
  body: JsonRecord
): Promise<PackageResponse> =>
  request<JsonRecord, PackageResponse>({
    method: "POST",
    url: `/api/app/packages/${packageType}/${packageId}/deactivate`,
    body,
  })

export const rollbackPackageApi = async (
  packageType: string,
  packageId: string,
  body: JsonRecord
): Promise<PackageResponse> =>
  request<JsonRecord, PackageResponse>({
    method: "POST",
    url: `/api/app/packages/${packageType}/${packageId}/rollback`,
    body,
  })

export const listPackageHistoryApi = async (
  packageType: string,
  packageId: string,
  limit = 50
): Promise<PackageHistoryListResponse> =>
  request<{ limit: number }, PackageHistoryListResponse>({
    method: "GET",
    url: `/api/app/packages/${packageType}/${packageId}/history`,
    body: { limit },
  })

export const listPackageLocksApi = async (
  workspaceId: string,
  runId: string
): Promise<PackageLockListResponse> =>
  request<undefined, PackageLockListResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/package-locks`,
  })

export const savePackageLocksApi = async (
  workspaceId: string,
  runId: string,
  body: JsonRecord
): Promise<PackageLockListResponse> =>
  request<JsonRecord, PackageLockListResponse>({
    method: "POST",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/package-locks`,
    body,
  })

export const checkPackageLockDriftApi = async (
  workspaceId: string,
  runId: string
): Promise<PackageLockDriftResponse> =>
  request<undefined, PackageLockDriftResponse>({
    method: "GET",
    url: `/api/app/workspaces/${workspaceId}/runs/${runId}/package-locks/drift`,
  })
