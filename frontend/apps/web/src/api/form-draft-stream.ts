import type {
  CampaignFormDraftRequest,
  CampaignFormDraftResponse,
  ProductDraftResponse,
  ProductFormDraftRequest,
} from "./commerce"

const FORM_DRAFT_STREAM_URL = "/api/sse/form-draft/stream"
const FORM_DRAFT_SAFE_ERROR_MESSAGE =
  "Form draft generation failed before form fill."

export type FormDraftStreamFormType = "campaign" | "product"

export interface FormDraftStreamProcessEvent {
  agentContext?: Record<string, unknown>
  failureKind?: string
  formType?: FormDraftStreamFormType
  generationMode?: string
  messageAvailable?: boolean
  stage: string
}

export type FormDraftStreamResponse =
  | CampaignFormDraftResponse
  | ProductDraftResponse

export interface ProductFormDraftStreamRequest {
  formType: "product"
  product: ProductFormDraftRequest
}

export interface CampaignFormDraftStreamRequest {
  campaign: CampaignFormDraftRequest
  formType: "campaign"
}

export type FormDraftStreamRequest =
  | CampaignFormDraftStreamRequest
  | ProductFormDraftStreamRequest

type FormDraftStreamResponseFor<T extends FormDraftStreamRequest> =
  T extends ProductFormDraftStreamRequest
    ? ProductDraftResponse
    : CampaignFormDraftResponse

export async function streamFormDraftApi<T extends FormDraftStreamRequest>({
  onProcess,
  request,
  signal,
}: {
  onProcess?: (event: FormDraftStreamProcessEvent) => void
  request: T
  signal?: AbortSignal
}): Promise<FormDraftStreamResponseFor<T>> {
  const response = await fetchFormDraftStream(request, signal)
  if (!response.ok || !response.body) {
    throw formDraftStreamError()
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder("utf-8")
  let buffer = ""
  let result: FormDraftStreamResponse | null = null

  while (true) {
    const { value, done } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })
    const boundary = buffer.lastIndexOf("\n\n")
    if (boundary === -1) continue

    const completed = buffer.slice(0, boundary)
    buffer = buffer.slice(boundary + 2)

    for (const block of completed.split("\n\n")) {
      const handled = handleFormDraftSseMessage(parseSseMessage(block), onProcess)
      if (handled.response) result = handled.response
    }
  }

  if (buffer.trim()) {
    const handled = handleFormDraftSseMessage(parseSseMessage(buffer), onProcess)
    if (handled.response) result = handled.response
  }

  if (!result) {
    throw formDraftStreamError()
  }

  return result as FormDraftStreamResponseFor<T>
}

function fetchFormDraftStream(
  body: FormDraftStreamRequest,
  signal?: AbortSignal
) {
  const token = localStorage.getItem("token")

  return fetch(FORM_DRAFT_STREAM_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(body),
    signal,
  })
}

function parseSseMessage(block: string): { data: string; event: string } {
  let event = "message"
  const data: string[] = []

  for (const line of block.split("\n")) {
    const trimmed = line.trimEnd()
    if (trimmed.startsWith("event:")) {
      event = trimmed.slice("event:".length).trim()
    }
    if (trimmed.startsWith("data:")) {
      data.push(trimmed.slice("data:".length).trimStart())
    }
  }

  return { data: data.join("\n"), event }
}

function handleFormDraftSseMessage(
  message: { data: string; event: string },
  onProcess?: (event: FormDraftStreamProcessEvent) => void
): { response: FormDraftStreamResponse | null } {
  if (!message.data || message.event === "done") {
    return { response: null }
  }

  const parsed = readJson(message.data)
  if (!parsed) return { response: null }

  if (message.event === "error") {
    throw formDraftStreamError()
  }

  if (isFormDraftCompleted(parsed)) {
    const response = parsed.response
    if (isFormDraftResponse(response)) return { response }
    throw formDraftStreamError()
  }

  if (isFormDraftProcessEvent(parsed)) {
    onProcess?.(toFormDraftProcessEvent(parsed))
  }

  return { response: null }
}

function isFormDraftCompleted(
  value: Record<string, unknown>
): value is Record<string, unknown> & { response: unknown; type: "form_draft_completed" } {
  return value.type === "form_draft_completed"
}

function isFormDraftProcessEvent(value: Record<string, unknown>) {
  return typeof value.type === "string" && value.type.startsWith("form_draft_")
}

function toFormDraftProcessEvent(
  value: Record<string, unknown>
): FormDraftStreamProcessEvent {
  return {
    agentContext: safeAgentContextFromProcess(value),
    failureKind: readString(value.failureKind),
    formType: readFormType(value.formType),
    generationMode: readString(value.generationMode),
    messageAvailable: readBoolean(value.messageAvailable),
    stage: String(value.type),
  }
}

function safeAgentContextFromProcess(
  value: Record<string, unknown>
): Record<string, unknown> {
  return {
    directWrite: value.directWrite,
    draftQuality: value.draftQuality,
    failureKind: readString(value.failureKind),
    generationSource: "agent_llm",
    inputSummary: safeInputSummaryFromProcess(value.inputSummary),
    llm: { used: true },
    messageAvailable: readBoolean(value.messageAvailable),
    requiresUserReview: value.requiresUserReview,
    saveBoundary: value.saveBoundary,
  }
}

function safeInputSummaryFromProcess(value: unknown): Record<string, unknown> | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined
  const source = value as Record<string, unknown>

  return {
    contextSummaryAvailable: readBoolean(source.contextSummaryAvailable),
    existingFieldCount: readNumber(source.existingFieldCount),
    userInputAvailable: readBoolean(source.userInputAvailable),
  }
}

function isFormDraftResponse(value: unknown): value is FormDraftStreamResponse {
  return (
    value !== null &&
    typeof value === "object" &&
    "draft" in value &&
    "agentContext" in value &&
    "generationMode" in value
  )
}

function readJson(data: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(data)
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : null
  } catch {
    return null
  }
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined
}

function readBoolean(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined
}

function readNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined
}

function readFormType(value: unknown): FormDraftStreamFormType | undefined {
  return value === "campaign" || value === "product" ? value : undefined
}

function formDraftStreamError() {
  return new Error(FORM_DRAFT_SAFE_ERROR_MESSAGE)
}
