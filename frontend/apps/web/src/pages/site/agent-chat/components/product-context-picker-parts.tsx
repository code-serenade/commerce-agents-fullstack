import { Check, PackageSearch, Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import { productStatusTranslationKey } from "./product-context-picker-helpers"
import {
  ComposerPanelEmptyState,
  composerPanelSecondaryButtonClassName,
} from "./composer-panel-layout"

export function ProductPickerContent({
  availableProducts,
  disabled,
  isLoadingProducts,
  onToggleProductContext,
  onAddProduct,
  query,
  selectedProducts,
}: {
  availableProducts: AgentChatProductContextSelection[]
  disabled: boolean
  isLoadingProducts: boolean
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onAddProduct: () => void
  query: string
  selectedProducts: AgentChatProductContextSelection[]
}) {
  const { t } = useTranslation()

  if (isLoadingProducts) {
    return (
      <span className="rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 py-1.5 text-xs text-muted-foreground">
        {t("agentChat.composer.productContext.loading")}
      </span>
    )
  }

  if (!availableProducts.length) {
    return (
      <ComposerPanelEmptyState
        action={
          <button
            type="button"
            disabled={disabled}
            onClick={onAddProduct}
            className={composerPanelSecondaryButtonClassName}
          >
            <Plus className="size-3.5" aria-hidden="true" />
            <span>{t("agentChat.composer.productContext.addProduct")}</span>
          </button>
        }
      >
          {query.trim()
            ? t("agentChat.composer.productContext.noSearchResult")
            : t("agentChat.composer.productContext.empty")}
      </ComposerPanelEmptyState>
    )
  }

  return (
    <>
      {selectedProducts.length === 0 ? (
        <ComposerPanelEmptyState icon={<PackageSearch className="size-4" />}>
          {t("agentChat.composer.productContext.noneHint")}
        </ComposerPanelEmptyState>
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

export function ProductContextButton({
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
          ? "border-primary/50 bg-primary/10 text-primary"
          : "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-foreground"
      }`}
      title={product.name}
    >
      {selected ? (
        <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-primary bg-primary text-primary-foreground">
          <Check className="size-3" />
        </span>
      ) : (
        <span className="mt-0.5 size-4 shrink-0 rounded-[4px] border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-inner" />
      )}
      <span className="min-w-0">
        <span className="block truncate font-medium text-foreground">
          {product.name}
        </span>
        <span className="mt-0.5 flex min-w-0 flex-wrap gap-1">
          {readProductMeta(product, t).map((item) => (
            <span
              key={item}
              className="rounded-full bg-(--agent-chat-subtle-surface) px-1.5 py-0.5 text-[10px] text-muted-foreground"
            >
              {item}
            </span>
          ))}
        </span>
      </span>
    </button>
  )
}

function readProductMeta(
  product: AgentChatProductContextSelection,
  t: ReturnType<typeof useTranslation>["t"]
) {
  return [
    product.status ? t(productStatusTranslationKey(product.status)) : null,
    product.platform,
    product.category,
    product.brand,
    product.price,
  ]
    .filter((value): value is string => Boolean(value))
    .slice(0, 4)
}
