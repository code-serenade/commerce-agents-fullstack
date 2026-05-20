import type { QueryClient, QueryKey } from "@tanstack/react-query"
import type { ProductResponse } from "@/api"
import { productsQueryKey } from "./use-product-queries"

export function syncProductCache(
  queryClient: QueryClient,
  product: ProductResponse
) {
  queryClient.setQueryData([...productsQueryKey, product.id, "detail"], product)
  queryClient
    .getQueryCache()
    .findAll({ queryKey: [...productsQueryKey, "list"] })
    .forEach((query) => {
      const params = readActiveProductListParams(query.queryKey)
      if (!params) return

      queryClient.setQueryData<ProductResponse[]>(
        query.queryKey,
        (current) => syncProductList(current, product, params.keyword)
      )
    })
}

function readActiveProductListParams(queryKey: QueryKey) {
  if (queryKey.length !== productsQueryKey.length + 2) return null
  if (queryKey[0] !== productsQueryKey[0]) return null
  if (queryKey[1] !== productsQueryKey[1]) return null
  if (queryKey[2] !== "list") return null

  const params = queryKey[3]
  if (!isProductListParams(params)) return null

  return params.status === "active" ? params : null
}

function isProductListParams(
  value: unknown
): value is { keyword: string; status: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "keyword" in value &&
    "status" in value &&
    typeof value.keyword === "string" &&
    typeof value.status === "string"
  )
}

function syncProductList(
  current: ProductResponse[] | undefined,
  product: ProductResponse,
  keyword: string
) {
  if (!current) return current
  const withoutProduct = current.filter((item) => item.id !== product.id)
  if (product.status && product.status !== "active") return withoutProduct
  if (!productMatchesKeyword(product, keyword)) return withoutProduct
  return [product, ...withoutProduct]
}

function productMatchesKeyword(product: ProductResponse, keyword: string) {
  const normalizedKeyword = keyword.trim().toLowerCase()
  if (!normalizedKeyword) return true

  return [
    product.name,
    product.brand,
    product.category,
    product.platform,
    product.externalId,
    product.status,
  ]
    .filter(Boolean)
    .some((value) => String(value).toLowerCase().includes(normalizedKeyword))
}
