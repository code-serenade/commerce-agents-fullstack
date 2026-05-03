import { resolveUrl, type UrlGroup } from "./url"

const DEFAULT_TIMEOUT = 10000

interface FetchOptions extends RequestInit {
  data?: unknown
  timeout?: number
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
    group = "api",
    suppressGlobalError = false,
  }: FetchOptions): Promise<{ data: T }> {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), timeout)

    try {
      let res = await makeRequest({ url, method, data, controller, group })

      if (res.status === 401 && (await tryRefreshToken())) {
        res = await makeRequest({ url, method, data, controller, group })
      }

      clearTimeout(timer)
      const json = await safeJson(res)

      if (!res.ok) {
        throw new HttpError("HTTP error", {
          data: json,
          status: res.status,
        })
      }

      return { data: json }
    } catch (err) {
      clearTimeout(timer)
      if (!isAbortError(err) && !suppressGlobalError) {
        globalHttpClientErrorHandler?.(err)
      }
      throw err
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
  group = "api",
}: {
  url: string
  method: string
  data?: unknown
  controller: AbortController
  group?: UrlGroup
}): Promise<Response> {
  const token = localStorage.getItem("token")

  const finalUrl = resolveUrl(url, group)

  return fetch(finalUrl, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body:
      method === "GET" || method === "DELETE"
        ? undefined
        : JSON.stringify(data),
    signal: controller.signal,
  })
}

async function tryRefreshToken(): Promise<boolean> {
  const refresh = localStorage.getItem("refreshToken")
  if (!refresh) {
    redirectToLogin("No refresh token")
    return false
  }

  const finalUrl = resolveUrl("/auth/session/refresh_token", "auth")

  const res = await fetch(finalUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ refreshToken: refresh }),
  })

  if (res.ok) {
    const newTokenData = await res.json()
    const token =
      newTokenData.data?.accessToken ?? newTokenData.data?.access_token
    if (token) {
      localStorage.setItem("token", token)
      return true
    }
  }

  redirectToLogin("Refresh token expired")
  return false
}

function redirectToLogin(reason: string) {
  console.warn("Redirecting to login:", reason)
  localStorage.removeItem("token")
  localStorage.removeItem("refreshToken")
  window.location.href = "/login"
}

async function safeJson(res: Response): Promise<any> {
  try {
    return await res.json()
  } catch {
    return {}
  }
}
