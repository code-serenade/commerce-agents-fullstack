import { useMemo, useState } from "react"
import { useNavigate } from "react-router"
import { useTranslation } from "react-i18next"
import { Badge, SearchInput } from "@workspace/ui-components"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import { productStatusTranslationKey } from "./product-context-picker-helpers"
import {
  ProductContextButton,
  ProductPickerContent,
} from "./product-context-picker-parts"

const PRODUCT_PAGE_SIZE = 6

export function ProductContextPicker({
  availableProducts,
  disabled,
  isLoadingProducts,
  onClearProductContext,
  onToggleProductContext,
  selectedProducts,
}: {
  availableProducts: AgentChatProductContextSelection[]
  disabled: boolean
  isLoadingProducts: boolean
  onClearProductContext: () => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  selectedProducts: AgentChatProductContextSelection[]
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [query, setQuery] = useState("")
  const [visibleCount, setVisibleCount] = useState(PRODUCT_PAGE_SIZE)
  const filteredProducts = useMemo(
    () => filterProducts(availableProducts, query),
    [availableProducts, query]
  )
  const visibleProducts = filteredProducts.slice(0, visibleCount)
  const hiddenProductCount = Math.max(
    filteredProducts.length - visibleProducts.length,
    0
  )
  const statusSummary = useMemo(
    () => summarizeProducts(availableProducts),
    [availableProducts]
  )

  return (
    <>
      <div className="mb-3 grid gap-2">
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
          <div className="min-w-0">
            <p className="text-xs font-medium text-foreground">
              {selectedProducts.length
                ? t("agentChat.composer.productContext.selectedCount", {
                    count: selectedProducts.length,
                  })
                : t("agentChat.composer.productContext.noneSelected")}
            </p>
            <p className="mt-0.5 text-[11px] text-muted-foreground">
              {selectedProducts.length
                ? t("agentChat.composer.productContext.selectedHint")
                : t("agentChat.composer.productContext.manualHint")}
            </p>
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-1.5">
            {statusSummary.map(({ count, status }) => (
              <Badge key={status} variant="outline">
                {t(productStatusTranslationKey(status))}
                {count}
              </Badge>
            ))}
          </div>
        </div>
        <SearchInput
          value={query}
          onValueChange={(value) => {
            setQuery(value)
            setVisibleCount(PRODUCT_PAGE_SIZE)
          }}
          placeholder={t("agentChat.composer.productContext.searchPlaceholder")}
          disabled={disabled || isLoadingProducts}
        />
        {selectedProducts.length ? (
          <div className="grid gap-2 rounded-lg border border-border bg-muted/25 p-2">
            <div className="flex min-w-0 items-center justify-between gap-2">
              <p className="text-[11px] font-semibold text-foreground">
                {t("agentChat.composer.productContext.selectedSection")}
              </p>
              <button
                type="button"
                disabled={disabled}
                onClick={onClearProductContext}
                className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              >
                {t("agentChat.composer.productContext.clear")}
              </button>
            </div>
            <div className="grid min-w-0 gap-1.5 sm:grid-cols-2">
              {selectedProducts.map((product) => (
                <ProductContextButton
                  key={product.id}
                  disabled={disabled}
                  onToggleProductContext={onToggleProductContext}
                  product={product}
                  selected
                />
              ))}
            </div>
          </div>
        ) : null}
      </div>
      <div className="grid min-w-0 gap-2 sm:grid-cols-2">
        <ProductPickerContent
          availableProducts={visibleProducts}
          disabled={disabled}
          isLoadingProducts={isLoadingProducts}
          onToggleProductContext={onToggleProductContext}
          onAddProduct={() => navigate("/products")}
          query={query}
          selectedProducts={selectedProducts}
        />
      </div>
      {hiddenProductCount > 0 ? (
        <div className="mt-2 flex justify-center">
          <button
            type="button"
            disabled={disabled || isLoadingProducts}
            onClick={() =>
              setVisibleCount((current) => current + PRODUCT_PAGE_SIZE)
            }
            className="rounded-full border border-border bg-background px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t("agentChat.composer.productContext.showMore", {
              count: Math.min(hiddenProductCount, PRODUCT_PAGE_SIZE),
            })}
          </button>
        </div>
      ) : null}
    </>
  )
}

function filterProducts(
  products: AgentChatProductContextSelection[],
  query: string
) {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return products

  return products.filter((product) =>
    [
      product.name,
      product.brand,
      product.category,
      product.platform,
      product.externalId,
      product.status,
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(normalizedQuery))
  )
}

function summarizeProducts(products: AgentChatProductContextSelection[]) {
  const counts = new Map<string, number>()
  products.forEach((product) => {
    const status = product.status || "unknown"
    counts.set(status, (counts.get(status) ?? 0) + 1)
  })

  return Array.from(counts.entries())
    .map(([status, count]) => ({ count, status }))
    .sort((left, right) => left.status.localeCompare(right.status))
}
