type Primitive = string | number | boolean | null

export type QueryParamValue = Primitive | readonly Primitive[]

export type QueryParams = Record<string, QueryParamValue | undefined>

export interface PaginationParams {
  page?: number
  pageSize?: number
}

export interface PaginatedParams extends PaginationParams, QueryParams {}

export interface CollectionResult<TItem> {
  items: TItem[]
  total: number
}

export interface PaginatedResult<TItem> extends CollectionResult<TItem> {
  page: number
  pageSize: number
}

export interface SelectOption<TValue extends Primitive = string> {
  label: string
  value: TValue
}
