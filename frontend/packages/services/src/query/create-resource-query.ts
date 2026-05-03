import {
  keepPreviousData,
  queryOptions,
  useQuery as useTanStackQuery,
  type UseQueryResult,
} from "@tanstack/react-query"
import { toRequestError, type RequestError } from "../errors/request-error"
import { createQueryKeys } from "./keys"
import type { QueryParams } from "./types"

async function withRequestError<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn()
  } catch (error) {
    throw toRequestError(error)
  }
}

interface CreateListResourceQueryConfig<TParams extends QueryParams, TResult> {
  resource: string
  queryFn: (params: TParams) => Promise<TResult>
  staleTime?: number
  keepPreviousResult?: boolean
}

interface CreateDetailResourceQueryConfig<TId extends string | number, TResult> {
  resource: string
  queryFn: (id: TId) => Promise<TResult>
  staleTime?: number
}

export function createListResourceQuery<
  TParams extends QueryParams,
  TResult,
>({
  resource,
  queryFn,
  staleTime,
  keepPreviousResult = true,
}: CreateListResourceQueryConfig<TParams, TResult>) {
  const keys = createQueryKeys(resource)

  const getKey = (params: TParams) => keys.list(params)

  const getOptions = (params: TParams) =>
    queryOptions<TResult, RequestError>({
      queryKey: getKey(params),
      queryFn: () => withRequestError(() => queryFn(params)),
      staleTime,
      placeholderData: keepPreviousResult ? keepPreviousData : undefined,
    })

  const useQuery = (params: TParams): UseQueryResult<TResult, RequestError> =>
    useTanStackQuery(getOptions(params))

  return {
    keys,
    getKey,
    getOptions,
    useQuery,
  }
}

export function createDetailResourceQuery<
  TId extends string | number,
  TResult,
>({
  resource,
  queryFn,
  staleTime,
}: CreateDetailResourceQueryConfig<TId, TResult>) {
  const keys = createQueryKeys(resource)

  const getKey = (id: TId) => keys.detail(id)

  const getOptions = (id: TId) =>
    queryOptions<TResult, RequestError>({
      queryKey: getKey(id),
      queryFn: () => withRequestError(() => queryFn(id)),
      staleTime,
    })

  const useQuery = (id: TId): UseQueryResult<TResult, RequestError> =>
    useTanStackQuery(getOptions(id))

  return {
    keys,
    getKey,
    getOptions,
    useQuery,
  }
}
