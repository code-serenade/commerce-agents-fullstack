import { Check, PackageSearch } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Badge, SearchInput } from "@workspace/ui-components"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"

const PRODUCT_STATUS_KEYS = new Set(["active", "draft", "listing", "archived"])

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
  const [query, setQuery] = useState("")
  const filteredProducts = useMemo(
    () => filterProducts(availableProducts, query),
    [availableProducts, query]
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
          onValueChange={setQuery}
          placeholder={t("agentChat.composer.productContext.searchPlaceholder")}
          disabled={disabled || isLoadingProducts}
        />
        {selectedProducts.length ? (
          <div className="flex justify-end">
            <button
              type="button"
              disabled={disabled}
              onClick={onClearProductContext}
              className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            >
              {t("agentChat.composer.productContext.clear")}
            </button>
          </div>
        ) : null}
      </div>
      <div className="grid max-h-72 min-w-0 gap-2 overflow-y-auto pr-1 sm:grid-cols-2">
        <ProductPickerContent
          availableProducts={filteredProducts}
          disabled={disabled}
          isLoadingProducts={isLoadingProducts}
          onToggleProductContext={onToggleProductContext}
          query={query}
          selectedProducts={selectedProducts}
        />
      </div>
    </>
  )
}

function ProductPickerContent({
  availableProducts,
  disabled,
  isLoadingProducts,
  onToggleProductContext,
  query,
  selectedProducts,
}: {
  availableProducts: AgentChatProductContextSelection[]
  disabled: boolean
  isLoadingProducts: boolean
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  query: string
  selectedProducts: AgentChatProductContextSelection[]
}) {
  const { t } = useTranslation()

  if (isLoadingProducts) {
    return (
      <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
        {t("agentChat.composer.productContext.loading")}
      </span>
    )
  }

  if (!availableProducts.length) {
    return (
      <div className="col-span-full rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
        {query.trim()
          ? t("agentChat.composer.productContext.noSearchResult")
          : t("agentChat.composer.productContext.empty")}
      </div>
    )
  }

  return (
    <>
      {selectedProducts.length === 0 ? (
        <div className="col-span-full flex items-start gap-2 rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
          <PackageSearch className="mt-0.5 size-4 shrink-0" />
          <span>{t("agentChat.composer.productContext.noneHint")}</span>
        </div>
      ) : null}
      {availableProducts.map((product) => (
        <ProductContextButton
          key={product.id}
          disabled={disabled}
          onToggleProductContext={onToggleProductContext}
          product={product}
          selected={selectedProducts.some((item) => item.id === product.id)}
        />
      ))}
    </>
  )
}

function ProductContextButton({
  disabled,
  onToggleProductContext,
  product,
  selected,
}: {
  disabled: boolean
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  product: AgentChatProductContextSelection
  selected: boolean
}) {
  const { t } = useTranslation()

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleProductContext(product)}
      aria-pressed={selected}
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-emerald-500/70 bg-emerald-50 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-200"
          : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={product.name}
    >
      {selected ? (
        <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-emerald-500 bg-emerald-500 text-white">
          <Check className="size-3" />
        </span>
      ) : (
        <span className="mt-0.5 size-4 shrink-0 rounded-[4px] border border-border bg-background shadow-inner" />
      )}
      <span className="min-w-0">
        <span className="block truncate font-medium text-foreground">
          {product.name}
        </span>
        <span className="mt-1 flex min-w-0 flex-wrap gap-1.5">
          {readProductMeta(product, t).map((item) => (
            <span
              key={item}
              className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
            >
              {item}
            </span>
          ))}
        </span>
      </span>
    </button>
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

function readProductMeta(
  product: AgentChatProductContextSelection,
  t: ReturnType<typeof useTranslation>["t"]
) {
  return [
    product.status
      ? t(productStatusTranslationKey(product.status))
      : null,
    product.platform,
    product.category,
    product.brand,
    product.price,
  ]
    .filter((value): value is string => Boolean(value))
    .slice(0, 4)
}

function productStatusTranslationKey(status: string) {
  const normalized = status.trim().toLowerCase()
  return `agentChat.composer.productContext.status.${
    PRODUCT_STATUS_KEYS.has(normalized) ? normalized : "unknown"
  }`
}
