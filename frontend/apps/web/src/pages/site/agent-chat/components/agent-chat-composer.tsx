import { Loader2, Send, Square } from "lucide-react"
import { type KeyboardEvent } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components/stable/button"
import { Textarea } from "@workspace/ui-components/stable/textarea"

export interface AgentChatComposerProps {
  canSend: boolean
  disabled: boolean
  input: string
  isStreaming: boolean
  onInputChange: (value: string) => void
  onSend: () => void
  onStop: () => void
}

export function AgentChatComposer({
  canSend,
  disabled,
  input,
  isStreaming,
  onInputChange,
  onSend,
  onStop,
}: AgentChatComposerProps) {
  const { t } = useTranslation()

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "Enter" || event.shiftKey || event.nativeEvent.isComposing) return
    event.preventDefault()
    if (canSend) onSend()
  }

  return (
    <footer className="border-t border-(--app-border) bg-(--app-bg) p-4 sm:px-6">
      <div
        className="mx-auto flex w-full max-w-3xl flex-col gap-3 rounded-lg border border-(--app-border) bg-(--app-surface) p-3 shadow-(--ui-shadow-soft)"
        onKeyDown={handleKeyDown}
      >
        <Textarea
          value={input}
          onValueChange={onInputChange}
          placeholder={t(
            disabled
              ? "agentChat.composer.loginPlaceholder"
              : "agentChat.composer.placeholder"
          )}
          disabled={disabled || isStreaming}
          rows={3}
        />
        <div className="flex items-center justify-between gap-3">
          <p className="text-xs text-(--app-muted-text)">
            {t("agentChat.composer.hint")}
          </p>
          {isStreaming ? (
            <Button variant="outline" size="sm" onClick={onStop}>
              <Square className="size-4" />
              {t("agentChat.composer.stop")}
            </Button>
          ) : (
            <Button size="sm" disabled={!canSend} onClick={onSend}>
              {canSend ? <Send className="size-4" /> : <Loader2 className="size-4" />}
              {t("agentChat.composer.send")}
            </Button>
          )}
        </div>
      </div>
    </footer>
  )
}
