export type AnalyticsEventType =
  | "page_view"
  | "session_start"
  | "session_end"
  | "feature_click"
  | "agent_question_submit"
  | "agent_answer_view"
  | "process_panel_open"
  | "report_view"
  | "report_copy"
  | "report_download"
  | "file_upload"
  | "login_click"
  | "login_success"
  | "register_success"
  | "frontend_error"
  | "api_error"

export type AnalyticsSourceKind =
  | "direct"
  | "search"
  | "referral"
  | "social"
  | "ad"
  | "internal"
  | "unknown"

export type AnalyticsDeviceType =
  | "desktop"
  | "mobile"
  | "tablet"
  | "bot"
  | "unknown"

export type AnalyticsEventStatus = "ok" | "failed"

export interface TrackAnalyticsEventRequest {
  eventId: string
  eventType: AnalyticsEventType
  occurredAt: string
  anonymousId?: string | null
  sessionId: string
  routePath?: string | null
  pageTitle?: string | null
  referrer?: string | null
  sourceKind: AnalyticsSourceKind
  sourceHost?: string | null
  utmSource?: string | null
  utmMedium?: string | null
  utmCampaign?: string | null
  utmContent?: string | null
  utmTerm?: string | null
  featureKey?: string | null
  deviceType: AnalyticsDeviceType
  browserName?: string | null
  osName?: string | null
  viewportWidth?: number | null
  viewportHeight?: number | null
  status?: AnalyticsEventStatus
  metadata?: Record<string, unknown>
}

export interface TrackAnalyticsEventsRequest {
  events: TrackAnalyticsEventRequest[]
}

export interface TrackAnalyticsEventsResponse {
  accepted: number
  duplicated: number
  rejected: number
}

export async function trackAnalyticsEventsApi(
  body: TrackAnalyticsEventsRequest,
  options?: { token?: string | null; signal?: AbortSignal }
): Promise<TrackAnalyticsEventsResponse | null> {
  const token = options?.token ?? localStorage.getItem("token")
  const response = await fetch(
    token ? "/api/app/analytics/events" : "/api/analytics/events",
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(body),
      signal: options?.signal,
      keepalive: body.events.length <= 3,
    }
  )

  if (!response.ok) return null

  const json = (await response.json().catch(() => null)) as
    | { data?: TrackAnalyticsEventsResponse }
    | null
  return json?.data ?? null
}
