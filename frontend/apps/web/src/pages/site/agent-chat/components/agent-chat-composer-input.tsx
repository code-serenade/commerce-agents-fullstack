import type { ChangeEvent, KeyboardEvent } from "react"
import type { RefObject } from "react"
import { useTranslation } from "react-i18next"
import { AgentChatComposerSendButton } from "./agent-chat-composer-toolbar"

export function AgentChatComposerInput({
  canSend,
  input,
  isLoggedIn,
  isStreaming,
  onAttachmentChange,
  onInputChange,
  onSend,
  onStop,
  fileInputRef,
  textareaRef,
}: {
  canSend: boolean
  input: string
  isLoggedIn: boolean
  isStreaming: boolean
  onAttachmentChange: (event: ChangeEvent<HTMLInputElement>) => void
  onInputChange: (value: string) => void
  onSend: () => void
  onStop: () => void
  fileInputRef: RefObject<HTMLInputElement | null>
  textareaRef: RefObject<HTMLTextAreaElement | null>
}) {
  const { t } = useTranslation()

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (
      event.key !== "Enter" ||
      event.shiftKey ||
      event.nativeEvent.isComposing
    )
      return
    event.preventDefault()
    if (canSend) onSend()
  }

  return (
    <>
      <input
        ref={fileInputRef}
        className="hidden"
        multiple
        type="file"
        onChange={onAttachmentChange}
      />
      <div className="grid min-h-10 grid-cols-1 gap-1 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(event) => onInputChange(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t(
            !isLoggedIn
              ? "agentChat.composer.loginPlaceholder"
              : "agentChat.composer.placeholder"
          )}
          disabled={isStreaming}
          rows={1}
          className="max-h-44 min-h-10 w-full resize-none border-0 bg-transparent px-3 py-2 text-sm/6 outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-60"
        />
        <div className="hidden min-h-10 items-center sm:flex">
          <AgentChatComposerSendButton
            canSend={canSend}
            isLoggedIn={isLoggedIn}
            isStreaming={isStreaming}
            onSend={onSend}
            onStop={onStop}
          />
        </div>
      </div>
    </>
  )
}
