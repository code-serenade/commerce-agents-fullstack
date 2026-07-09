import { ChevronDown, ShoppingBag } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import { ProductContextPicker } from "./product-context-picker"

export interface AgentChatProductContextBarProps {
  availableProducts: AgentChatProductContextSelection[]
  disabled: boolean
  isOpen?: boolean
  isLoadingProducts: boolean
  onClearProductContext: () => void
  onOpenChange?: (open: boolean) => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  selectedProducts: AgentChatProductContextSelection[]
  variant?: "bar" | "compact" | "panel"
}

export function AgentChatProductContextBar({
  availableProducts,
  disabled,
  isOpen,
  isLoadingProducts,
  onClearProductContext,
  onOpenChange,
  onToggleProductContext,
  selectedProducts,
  variant = "bar",
}: AgentChatProductContextBarProps) {
  const { t } = useTranslation()
  const [internalOpen, setInternalOpen] = useState(false)
  const open = isOpen ?? internalOpen
  const selectedProductNames = selectedProducts.map((product) => product.name)
  const selectedProductSummary =
    selectedProducts.length > 0
      ? t("agentChat.composer.productContext.selectedCount", {
          count: selectedProducts.length,
        })
      : t("agentChat.composer.productContext.noneSelected")
  const title = selectedProducts.length
    ? `${t("agentChat.composer.productContext.compactTitle")} - ${selectedProductNames.join(" / ")}`
    : `${t("agentChat.composer.productContext.compactTitle")} - ${selectedProductSummary}`

  const toggleOpen = () => {
    const nextOpen = !open
    if (isOpen === undefined) setInternalOpen(nextOpen)
    onOpenChange?.(nextOpen)
  }

  if (variant === "panel") {
    return (
      <ProductContextPicker
        availableProducts={availableProducts}
        disabled={disabled}
        isLoadingProducts={isLoadingProducts}
        onClearProductContext={onClearProductContext}
        onToggleProductContext={onToggleProductContext}
        selectedProducts={selectedProducts}
      />
    )
  }

  if (variant === "compact") {
    return (
      <div className="relative shrink-0" data-composer-interactive="true">
        <button
          type="button"
          onClick={toggleOpen}
          className={`flex h-7 max-w-28 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors sm:max-w-36 ${
            open
              ? "border-primary/30 bg-(--agent-chat-accent-soft) text-(--agent-chat-accent-foreground) shadow-sm"
              : "border-(--agent-chat-soft-border) bg-(--agent-chat-muted-control) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-(--agent-chat-accent-foreground)"
          }`}
          aria-expanded={open}
          aria-label={title}
          title={title}
        >
          <ShoppingBag className="size-3.5 shrink-0" />
          <span className="truncate">
            {selectedProducts.length
              ? t("agentChat.composer.productContext.compactLabel")
              : t("agentChat.composer.productContext.noneCompactLabel")}
          </span>
          {selectedProducts.length ? (
            <span className="flex size-4 shrink-0 items-center justify-center rounded-full bg-current/10 text-[10px]">
              {selectedProducts.length}
            </span>
          ) : null}
          <ChevronDown
            className={`size-3.5 shrink-0 transition-transform ${
              open ? "rotate-180" : ""
            }`}
          />
        </button>
      </div>
    )
  }

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
                    ? "bg-(--agent-chat-accent-soft) text-(--agent-chat-accent-foreground)"
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
          onClick={toggleOpen}
          className="flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
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
          <ProductContextPicker
            availableProducts={availableProducts}
            disabled={disabled}
            isLoadingProducts={isLoadingProducts}
            onClearProductContext={onClearProductContext}
            onToggleProductContext={onToggleProductContext}
            selectedProducts={selectedProducts}
          />
        </div>
      ) : null}
    </div>
  )
}
