type RequestErrorKind = "api" | "network" | "unknown"

interface RequestErrorOptions {
  kind: RequestErrorKind
  code?: number | null
  status?: number
  cause?: unknown
}

export class RequestError extends Error {
  kind: RequestErrorKind
  code: number | null
  status?: number
  cause?: unknown

  constructor(message: string, options: RequestErrorOptions) {
    super(message)
    this.name = "RequestError"
    this.kind = options.kind
    this.code = options.code ?? null
    this.status = options.status
    this.cause = options.cause
  }
}

interface HttpErrorLike {
  response?: {
    data?: unknown
    status?: number
  }
}

export function toRequestError(input: unknown): RequestError {
  if (input instanceof RequestError) return input

  const httpLike = input as HttpErrorLike | undefined
  const status = httpLike?.response?.status
  const code = extractBackendErrorCode(httpLike?.response?.data)

  if (typeof status === "number") {
    return new RequestError("API request failed", {
      kind: "api",
      code,
      status,
      cause: input,
    })
  }

  if (input instanceof TypeError || input instanceof DOMException) {
    return new RequestError("Network request failed", {
      kind: "network",
      cause: input,
    })
  }

  return new RequestError("Unknown error", {
    kind: "unknown",
    cause: input,
  })
}

function extractBackendErrorCode(payload: unknown): number | null {
  if (!payload || typeof payload !== "object") return null
  const data = payload as { error_code?: unknown; code?: unknown }

  if (typeof data.error_code === "number") return data.error_code
  if (typeof data.code === "number") return data.code

  return null
}
