import {
  trackAnalyticsEventsApi,
  type AnalyticsDeviceType,
  type AnalyticsEventType,
  type AnalyticsSourceKind,
  type TrackAnalyticsEventRequest,
} from "@/api/analytics"

const ANONYMOUS_ID_KEY = "commerce-agents.analytics.anonymousId"
const SESSION_KEY = "commerce-agents.analytics.session"
const SESSION_TIMEOUT_MS = 30 * 60 * 1000
const DEFAULT_PAGE_TITLE = "Commerce Agents"

let fallbackAnonymousId: string | null = null
let fallbackSession: StoredSession | null = null

interface StoredSession {
  id: string
  lastSeenAt: number
}

interface TrackProductEventInput {
  eventType: AnalyticsEventType
  featureKey?: string
  routePath?: string
  pageTitle?: string
  status?: "ok" | "failed"
  metadata?: Record<string, unknown>
}

export function trackProductEvent(input: TrackProductEventInput) {
  if (typeof window === "undefined") return

  const event = safelyBuildAnalyticsEvent(input)
  if (!event) return

  void trackAnalyticsEventsApi({ events: [event] }).catch((error) => {
    console.debug("[analytics] event ignored", error)
  })
}

export function trackProductPageView(pathname: string, search: string) {
  if (typeof window === "undefined") return
  const routePath = `${pathname}${search}`

  trackProductEvent({
    eventType: "page_view",
    routePath,
    pageTitle: document.title || DEFAULT_PAGE_TITLE,
  })
  trackBaiduPageView(routePath)
}

export function trackBaiduPageView(routePath: string) {
  try {
    if (typeof window === "undefined") return
    const hmt = window._hmt
    if (!Array.isArray(hmt)) return
    hmt.push(["_trackPageview", routePath])
  } catch (error) {
    console.debug("[analytics] baidu page view ignored", error)
  }
}

function buildAnalyticsEvent(
  input: TrackProductEventInput
): TrackAnalyticsEventRequest {
  const source = readSource()
  const device = readDevice()
  const url = new URL(window.location.href)

  return {
    eventId: createId("evt"),
    eventType: input.eventType,
    occurredAt: new Date().toISOString(),
    anonymousId: getAnonymousId(),
    sessionId: getSessionId(),
    routePath: input.routePath ?? `${window.location.pathname}${window.location.search}`,
    pageTitle: input.pageTitle ?? document.title ?? null,
    referrer: document.referrer || null,
    sourceKind: source.kind,
    sourceHost: source.host,
    utmSource: normalizeParam(url.searchParams.get("utm_source")),
    utmMedium: normalizeParam(url.searchParams.get("utm_medium")),
    utmCampaign: normalizeParam(url.searchParams.get("utm_campaign")),
    utmContent: normalizeParam(url.searchParams.get("utm_content")),
    utmTerm: normalizeParam(url.searchParams.get("utm_term")),
    featureKey: input.featureKey ?? null,
    deviceType: device.type,
    browserName: device.browserName,
    osName: device.osName,
    viewportWidth: window.innerWidth || null,
    viewportHeight: window.innerHeight || null,
    status: input.status ?? "ok",
    metadata: input.metadata ?? {},
  }
}

function getAnonymousId() {
  const stored = safeReadLocalStorage(ANONYMOUS_ID_KEY)
  if (stored) return stored
  if (fallbackAnonymousId) return fallbackAnonymousId

  const id = createId("anon")
  fallbackAnonymousId = id
  safeWriteLocalStorage(ANONYMOUS_ID_KEY, id)
  return id
}

function getSessionId() {
  const now = Date.now()
  const stored = readStoredSession()
  if (stored && now - stored.lastSeenAt <= SESSION_TIMEOUT_MS) {
    writeStoredSession({ id: stored.id, lastSeenAt: now })
    return stored.id
  }

  const next: StoredSession = { id: createId("session"), lastSeenAt: now }
  writeStoredSession(next)
  trackSessionStart(next.id)
  return next.id
}

function trackSessionStart(sessionId: string) {
  window.setTimeout(() => {
    const event = safelyBuildAnalyticsEvent({
      eventType: "session_start",
      featureKey: "session.start",
    })
    if (!event) return

    void trackAnalyticsEventsApi({
      events: [
        {
          ...event,
          sessionId,
        },
      ],
    }).catch((error) => {
      console.debug("[analytics] session start ignored", error)
    })
  }, 0)
}

function readStoredSession(): StoredSession | null {
  const raw = safeReadSessionStorage(SESSION_KEY)
  if (!raw) return fallbackSession

  try {
    const parsed = JSON.parse(raw) as StoredSession
    if (!parsed.id || !Number.isFinite(parsed.lastSeenAt)) return null
    return parsed
  } catch {
    return null
  }
}

function writeStoredSession(session: StoredSession) {
  fallbackSession = session
  safeWriteSessionStorage(SESSION_KEY, JSON.stringify(session))
}

function safelyBuildAnalyticsEvent(
  input: TrackProductEventInput
): TrackAnalyticsEventRequest | null {
  try {
    return buildAnalyticsEvent(input)
  } catch (error) {
    console.debug("[analytics] event build ignored", error)
    return null
  }
}

function safeReadLocalStorage(key: string) {
  try {
    return window.localStorage.getItem(key)
  } catch {
    return null
  }
}

function safeWriteLocalStorage(key: string, value: string) {
  try {
    window.localStorage.setItem(key, value)
  } catch {
    // Analytics must not block the product when browser storage is unavailable.
  }
}

function safeReadSessionStorage(key: string) {
  try {
    return window.sessionStorage.getItem(key)
  } catch {
    return null
  }
}

function safeWriteSessionStorage(key: string, value: string) {
  try {
    window.sessionStorage.setItem(key, value)
  } catch {
    // Analytics must not block the product when browser storage is unavailable.
  }
}

function readSource(): { kind: AnalyticsSourceKind; host: string | null } {
  const params = new URLSearchParams(window.location.search)
  if (params.get("utm_source") || params.get("utm_medium") || params.get("utm_campaign")) {
    return { kind: "ad", host: normalizeParam(params.get("utm_source")) }
  }
  if (!document.referrer) {
    return { kind: "direct", host: null }
  }

  try {
    const referrer = new URL(document.referrer)
    if (referrer.host === window.location.host) {
      return { kind: "internal", host: referrer.host }
    }
    if (isSearchHost(referrer.host)) {
      return { kind: "search", host: referrer.host }
    }
    if (isSocialHost(referrer.host)) {
      return { kind: "social", host: referrer.host }
    }
    return { kind: "referral", host: referrer.host }
  } catch {
    return { kind: "unknown", host: null }
  }
}

function readDevice(): {
  type: AnalyticsDeviceType
  browserName: string | null
  osName: string | null
} {
  const userAgent = navigator.userAgent
  const lower = userAgent.toLowerCase()
  const type: AnalyticsDeviceType = /bot|spider|crawler/.test(lower)
    ? "bot"
    : /ipad|tablet/.test(lower)
      ? "tablet"
      : /mobile|iphone|android/.test(lower)
        ? "mobile"
        : "desktop"

  return {
    type,
    browserName: readBrowserName(userAgent),
    osName: readOsName(userAgent),
  }
}

function readBrowserName(userAgent: string) {
  if (/Edg\//.test(userAgent)) return "Edge"
  if (/Chrome\//.test(userAgent)) return "Chrome"
  if (/Firefox\//.test(userAgent)) return "Firefox"
  if (/Safari\//.test(userAgent)) return "Safari"
  return null
}

function readOsName(userAgent: string) {
  if (/Windows/i.test(userAgent)) return "Windows"
  if (/Mac OS X/i.test(userAgent)) return "macOS"
  if (/Android/i.test(userAgent)) return "Android"
  if (/iPhone|iPad/i.test(userAgent)) return "iOS"
  if (/Linux/i.test(userAgent)) return "Linux"
  return null
}

function isSearchHost(host: string) {
  return /(^|\.)((baidu|google|bing|sogou|so|sm)\.)/i.test(host)
}

function isSocialHost(host: string) {
  return /(^|\.)(weibo|wechat|douyin|tiktok|xiaohongshu|zhihu|bilibili)\./i.test(host)
}

function normalizeParam(value: string | null) {
  const normalized = value?.trim()
  return normalized ? normalized.slice(0, 128) : null
}

function createId(prefix: string) {
  const random =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`
  return `${prefix}_${random}`
}

declare global {
  interface Window {
    _hmt?: unknown[]
  }
}
