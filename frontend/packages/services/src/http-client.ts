import { resolveUrl, type UrlGroup } from "./url"
import { RequestError } from "./errors/request-error"

const DEFAULT_TIMEOUT = 60000
const REFRESH_TIMEOUT = 10000

interface FetchOptions extends RequestInit {
  data?: unknown
  timeout?: number
  token?: string | null
  url: string
  method: string
  group?: UrlGroup
  suppressGlobalError?: boolean
}

class HttpError extends Error {
  response: { data: unknown; status: number }
  constructor(message: string, response: { data: unknown; status: number }) {
    super(message)
    this.name = "HttpError"
    this.response = response
  }
}

type HttpClientErrorHandler = (error: unknown) => void
type TokenRefreshResult = "refreshed" | "invalid" | "unavailable"

let globalHttpClientErrorHandler: HttpClientErrorHandler | null = null

export function setHttpClientErrorHandler(
  handler: HttpClientErrorHandler | null
) {
  globalHttpClientErrorHandler = handler
}

const httpClient = {
  async request<T>({
    url,
    method,
    data,
    timeout = DEFAULT_TIMEOUT,
    token,
    group = "api",
    suppressGlobalError = false,
  }: FetchOptions): Promise<{ data: T }> {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), timeout)
    const hasExplicitToken = token !== undefined

    try {
      let res = await makeRequest({
        url,
        method,
        data,
        controller,
        token,
        group,
      })

      if (res.status === 401 && !hasExplicitToken) {
        const refreshResult = await tryRefreshToken()
        if (refreshResult === "refreshed") {
          res = await makeRequest({ url, method, data, controller, group })
        } else if (refreshResult === "invalid") {
          redirectToLogin("Refresh token invalid")
        } else {
          throw new HttpError("Token refresh service unavailable", {
            data: { message: "Token refresh service unavailable" },
            status: 503,
          })
        }
      }

      clearTimeout(timer)
      const json = await safeJson(res)

      if (!res.ok) {
        throw new HttpError(readHttpErrorMessage(json) ?? "HTTP error", {
          data: json,
          status: res.status,
        })
      }

      return { data: json }
    } catch (err) {
      clearTimeout(timer)
      const error = isAbortError(err) ? createRequestTimeoutError(err) : err
      if (!suppressGlobalError) {
        globalHttpClientErrorHandler?.(error)
      }
      throw error
    }
  },

  async stream(
    url: string,
    data: Record<string, unknown>,
    onChunk: (chunk: string) => void,
    onDone?: () => void,
    onError?: (err: Error) => void
  ) {
    try {
      const token = localStorage.getItem("token")

      const finalUrl = resolveUrl(url)

      const res = await fetch(finalUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify(data),
      })

      if (!res.ok || !res.body) {
        const msg = `SSE connection failed: ${res.status}`
        onError?.(new Error(msg))
        return
      }

      const reader = res.body.getReader()
      const decoder = new TextDecoder("utf-8")
      let buffer = ""

      while (true) {
        const { value, done } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })

        if (processBuffer(buffer, onChunk, onDone)) return
        buffer = ""
      }

      onDone?.()
    } catch (err) {
      onError?.(err instanceof Error ? err : new Error("connection error"))
    }
  },

  async uploadFile(
    fullUrl: string,
    formData: FormData,
    onError?: (err: Error) => void
  ): Promise<Response> {
    try {
      const res = await fetch(fullUrl, {
        method: "POST",
        headers: {},
        body: formData,
      })

      if (!res.ok) {
        const err = new Error(`Upload failed: ${res.status}`)
        onError?.(err)
        throw err
      }

      return res
    } catch (err) {
      const error = err instanceof Error ? err : new Error("upload error")
      onError?.(error)
      throw error
    }
  },
}

export default httpClient

function isAbortError(error: unknown) {
  return error instanceof DOMException && error.name === "AbortError"
}

function createRequestTimeoutError(cause: unknown) {
  return new RequestError("Request timed out", {
    kind: "network",
    cause,
  })
}

function readHttpErrorMessage(data: unknown): string | null {
  if (!data || typeof data !== "object") return null
  const message = (data as { message?: unknown }).message
  return typeof message === "string" && message.trim().length > 0
    ? message
    : null
}

function processBuffer(
  buffer: string,
  onChunk: (chunk: string) => void,
  onDone?: () => void
): boolean {
  const lines = buffer.split("\n")
  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed?.startsWith("data:")) continue

    const content = trimmed.replace(/^data:\s*/, "")
    if (content === "[DONE]") {
      onDone?.()
      return true
    }

    try {
      const parsed = JSON.parse(content)
      onChunk(parsed.text ?? content)
    } catch {
      onChunk(content)
    }
  }
  return false
}

async function makeRequest({
  url,
  method,
  data,
  controller,
  token,
  group = "api",
}: {
  url: string
  method: string
  data?: unknown
  controller: AbortController
  token?: string | null
  group?: UrlGroup
}): Promise<Response> {
  const authToken = token === undefined ? localStorage.getItem("token") : token

  const finalUrl = resolveUrl(url, group)
  const isBlobBody = data instanceof Blob

  return fetch(finalUrl, {
    method,
    headers: {
      ...(isBlobBody ? {} : { "Content-Type": "application/json" }),
      ...(authToken ? { Authorization: `Bearer ${authToken}` } : {}),
    },
    body:
      method === "GET" || method === "DELETE"
        ? undefined
        : isBlobBody
          ? data
          : JSON.stringify(data),
    signal: controller.signal,
  })
}

async function tryRefreshToken(): Promise<TokenRefreshResult> {
  const refresh = localStorage.getItem("refreshToken")
  if (!refresh) {
    return "invalid"
  }

  const finalUrl = resolveUrl("/auth/session/refresh_token", "auth")
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), REFRESH_TIMEOUT)

  try {
    const res = await fetch(finalUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refreshToken: refresh }),
      signal: controller.signal,
    })

    if (res.ok) {
      const newTokenData = await res.json()
      const token =
        newTokenData.data?.accessToken ?? newTokenData.data?.access_token
      if (token) {
        localStorage.setItem("token", token)
        return "refreshed"
      }
    }
    if (isInvalidRefreshTokenStatus(res.status)) {
      return "invalid"
    }
  } catch (error) {
    if (!isAbortError(error)) {
      console.warn("Refresh token request failed:", error)
    }
  } finally {
    clearTimeout(timer)
  }

  return "unavailable"
}

function redirectToLogin(reason: string) {
  console.warn("Redirecting to login:", reason)
  localStorage.removeItem("token")
  localStorage.removeItem("refreshToken")
  window.location.href = "/login"
}

function isInvalidRefreshTokenStatus(status: number) {
  return status === 400 || status === 401 || status === 403
}

async function safeJson(res: Response): Promise<any> {
  try {
    return await res.json()
  } catch {
    return {}
  }
}
