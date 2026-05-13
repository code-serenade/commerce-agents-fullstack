import { Check, ChevronDown, ShoppingBag } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"

export interface AgentChatProductContextBarProps {
  availableProducts: AgentChatProductContextSelection[]
  disabled: boolean
  isLoadingProducts: boolean
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  selectedProducts: AgentChatProductContextSelection[]
}

export function AgentChatProductContextBar({
  availableProducts,
  disabled,
  isLoadingProducts,
  onToggleProductContext,
  selectedProducts,
}: AgentChatProductContextBarProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const selectedProductNames = selectedProducts.map((product) => product.name)
  const selectedProductSummary =
    selectedProducts.length > 0
      ? t("agentChat.composer.productContext.selectedCount", {
          count: selectedProducts.length,
        })
      : t("agentChat.composer.productContext.noneSelected")

  return (
    <div className="rounded-xl border border-border bg-background/90 p-2 shadow-sm">
      <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
            <ShoppingBag className="size-4" />
          </span>
          <div className="min-w-0">
            <div className="flex min-w-0 flex-wrap items-center gap-2">
              <span className="text-xs font-medium text-foreground">
                {t("agentChat.composer.productContext.label")}
              </span>
              <span
                className={`rounded-full px-2 py-0.5 text-[11px] font-medium ${
                  selectedProducts.length
                    ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-200"
                    : "bg-muted text-muted-foreground"
                }`}
              >
                {selectedProductSummary}
              </span>
            </div>
            <p className="mt-0.5 truncate text-[11px] text-muted-foreground">
              {selectedProducts.length
                ? selectedProductNames.join(" / ")
                : t("agentChat.composer.productContext.noneHint")}
            </p>
          </div>
        </div>
        <button
          type="button"
          onClick={() => setOpen((current) => !current)}
          className="flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          aria-expanded={open}
        >
          {open
            ? t("agentChat.composer.productContext.collapse")
            : t("agentChat.composer.productContext.choose")}
          <ChevronDown
            className={`size-3.5 transition-transform ${open ? "rotate-180" : ""}`}
          />
        </button>
      </div>
      {open ? (
        <div className="mt-2 border-t border-border pt-2">
          <p className="mb-2 text-[11px] text-muted-foreground">
            {selectedProducts.length
              ? t("agentChat.composer.productContext.selectedHint")
              : t("agentChat.composer.productContext.manualHint")}
          </p>
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            {isLoadingProducts ? (
              <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
                {t("agentChat.composer.productContext.loading")}
              </span>
            ) : availableProducts.length ? (
              <>
                {selectedProducts.length === 0 ? (
                  <span className="rounded-full border border-dashed border-border bg-background/60 px-3 py-1.5 text-xs text-muted-foreground">
                    {t("agentChat.composer.productContext.noneSelected")}
                  </span>
                ) : null}
                {availableProducts.slice(0, 8).map((product) => (
                  <ProductContextButton
                    key={product.id}
                    disabled={disabled}
                    onToggleProductContext={onToggleProductContext}
                    product={product}
                    selected={selectedProducts.some((item) => item.id === product.id)}
                  />
                ))}
                {availableProducts.length > 8 ? (
                  <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
                    {t("agentChat.composer.productContext.moreCount", {
                      count: availableProducts.length - 8,
                    })}
                  </span>
                ) : null}
              </>
            ) : (
              <span className="rounded-full border border-dashed border-border bg-background/60 px-3 py-1.5 text-xs text-muted-foreground">
                {t("agentChat.composer.productContext.empty")}
              </span>
            )}
          </div>
        </div>
      ) : null}
    </div>
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
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleProductContext(product)}
      className={`flex max-w-44 min-w-0 items-center gap-2 rounded-full border px-3 py-1.5 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-emerald-500/70 bg-emerald-50 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-200"
          : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={product.name}
    >
      {selected ? (
        <Check className="size-3.5 shrink-0" />
      ) : (
        <span className="size-1.5 shrink-0 rounded-full bg-current" />
      )}
      <span className="truncate">{product.name}</span>
    </button>
  )
}
