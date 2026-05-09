import { ShoppingBag } from "lucide-react"
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

  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2">
      <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <ShoppingBag className="size-3.5" />
        {t("agentChat.composer.productContext.label")}
      </span>
      {isLoadingProducts ? (
        <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
          {t("agentChat.composer.productContext.loading")}
        </span>
      ) : availableProducts.length ? (
        availableProducts.slice(0, 5).map((product) => (
          <ProductContextButton
            key={product.id}
            disabled={disabled}
            onToggleProductContext={onToggleProductContext}
            product={product}
            selected={selectedProducts.some((item) => item.id === product.id)}
          />
        ))
      ) : (
        <span className="rounded-full border border-dashed border-border bg-background/60 px-3 py-1.5 text-xs text-muted-foreground">
          {t("agentChat.composer.productContext.empty")}
        </span>
      )}
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
      <span className="size-1.5 shrink-0 rounded-full bg-current" />
      <span className="truncate">{product.name}</span>
    </button>
  )
}
