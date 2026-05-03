import type { QueryParams, QueryParamValue } from "./types"

export type NormalizedQueryParams = Readonly<Record<string, QueryParamValue>>

export function normalizeQueryParams(params: QueryParams = {}): NormalizedQueryParams {
  const normalized: Record<string, QueryParamValue> = {}

  const entries = Object.entries(params).sort(([left], [right]) =>
    left.localeCompare(right)
  )

  for (const [key, value] of entries) {
    if (value === undefined) continue
    normalized[key] = Array.isArray(value) ? [...value] : value
  }

  return Object.freeze(normalized)
}

export function createQueryKeys(resource: string) {
  return {
    all: [resource] as const,
    lists: () => [resource, "list"] as const,
    list: (params: QueryParams = {}) =>
      [resource, "list", normalizeQueryParams(params)] as const,
    details: () => [resource, "detail"] as const,
    detail: (id: string | number) => [resource, "detail", id] as const,
  }
}
