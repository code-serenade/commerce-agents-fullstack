import type { ChangeEvent, KeyboardEvent } from "react"
import type { RefObject } from "react"
import { useTranslation } from "react-i18next"
import { AI_READABLE_FILE_ACCEPT } from "../domain/agent-chat-upload-validation"

export function AgentChatComposerInput({
  canSend,
  input,
  isLoggedIn,
  isStreaming,
  onAttachmentChange,
  onInputChange,
  onSend,
  placeholder,
  size = "default",
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
  placeholder?: string
  size?: "default" | "welcome"
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
        accept={AI_READABLE_FILE_ACCEPT}
        className="hidden"
        multiple
        type="file"
        onChange={onAttachmentChange}
      />
      <div className="min-h-10">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(event) => onInputChange(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={
            placeholder ??
            t(
              !isLoggedIn
                ? "agentChat.composer.loginPlaceholder"
                : "agentChat.composer.placeholder"
            )
          }
          disabled={isStreaming}
          rows={1}
          className={`max-h-44 w-full resize-none border-0 bg-transparent px-3 text-sm/6 text-foreground outline-none placeholder:text-(--agent-chat-placeholder) disabled:cursor-not-allowed disabled:opacity-60 ${
            size === "welcome" ? "min-h-10 py-2 sm:px-4" : "min-h-10 py-2"
          }`}
        />
      </div>
    </>
  )
}
