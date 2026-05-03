import httpClient from "../http-client"
import type { UrlGroup } from "../url"

export interface RequestParams<T = Record<string, unknown>> {
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
  url: string
  body?: T
  group?: UrlGroup
  suppressGlobalError?: boolean
}

interface SuccessResponse<T> {
  code: 0
  data: T
  message: string
}

export async function request<T = unknown, R = unknown>({
  method,
  url,
  body,
  group = "api",
  suppressGlobalError = false,
}: RequestParams<T>): Promise<R> {
  let finalUrl = url

  if (
    ["GET", "DELETE"].includes(method) &&
    body &&
    typeof body === "object" &&
    !Array.isArray(body)
  ) {
    const queryParams = new URLSearchParams(
      Object.entries(body).reduce(
        (acc, [k, v]) => {
          if (v != null) acc[k] = String(v)
          return acc
        },
        {} as Record<string, string>
      )
    ).toString()

    finalUrl = `${url}?${queryParams}`
  }

  const response = await httpClient.request<SuccessResponse<R>>({
    method,
    url: finalUrl,
    data: ["POST", "PUT", "PATCH"].includes(method) ? body : undefined,
    group,
    suppressGlobalError,
  })

  return response.data.data
}

export function streamRequest<
  T extends Record<string, unknown> = Record<string, unknown>,
>({
  url,
  body,
  onChunk,
  onDone,
  onError,
}: {
  url: string
  body: T
  onChunk: (text: string) => void
  onDone?: () => void
  onError?: (err: Error) => void
}) {
  httpClient.stream(url, body, onChunk, onDone, onError)
}

export async function upload(
  fullUrl: string,
  formData: FormData,
  onError?: (err: Error) => void
): Promise<Response> {
  return httpClient.uploadFile(fullUrl, formData, onError)
}
