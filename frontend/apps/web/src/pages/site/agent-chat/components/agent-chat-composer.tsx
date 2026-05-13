import {
  type ChangeEvent,
  type KeyboardEvent,
  type MouseEvent,
  useEffect,
  useRef,
  useState,
} from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatComposerProps } from "./agent-chat-composer-types"
import { AgentChatAttachmentStrip } from "./agent-chat-attachment-strip"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import {
  AgentChatComposerSendButton,
  AgentChatComposerToolbar,
} from "./agent-chat-composer-toolbar"

export function AgentChatComposer({
  attachments,
  availableProducts,
  canSend,
  input,
  isLoggedIn,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onInputChange,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimeModelChange,
  onRuntimePrimaryRoleChange,
  onToggleRuntimeRole,
  onSelectPlaybook,
  onSend,
  onStop,
  onStrategyChange,
  onClearProductContext,
  onToggleProductContext,
  onUploadAttachments,
  selectedProducts,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: AgentChatComposerProps) {
  const { t } = useTranslation()
  const composerRootRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const [productContextOpen, setProductContextOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)

  useEffect(() => {
    if (!productContextOpen && !settingsOpen) return

    const closePanelsOnOutsideEvent = (event: Event) => {
      const target = event.target
      if (!(target instanceof Node)) return
      if (composerRootRef.current?.contains(target)) return
      setProductContextOpen(false)
      setSettingsOpen(false)
    }

    document.addEventListener("pointerdown", closePanelsOnOutsideEvent, true)
    document.addEventListener("click", closePanelsOnOutsideEvent, true)
    return () => {
      document.removeEventListener(
        "pointerdown",
        closePanelsOnOutsideEvent,
        true
      )
      document.removeEventListener("click", closePanelsOnOutsideEvent, true)
    }
  }, [productContextOpen, settingsOpen])

  useEffect(() => {
    const textarea = textareaRef.current
    if (!textarea) return
    textarea.style.height = "auto"
    textarea.style.height = `${Math.min(textarea.scrollHeight, 180)}px`
  }, [input])

  const sendAndClosePanels = () => {
    setProductContextOpen(false)
    setSettingsOpen(false)
    onSend()
  }

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (
      event.key !== "Enter" ||
      event.shiftKey ||
      event.nativeEvent.isComposing
    )
      return
    event.preventDefault()
    if (canSend) sendAndClosePanels()
  }

  const focusComposer = (event: MouseEvent<HTMLDivElement>) => {
    if (isStreaming) return
    const target = event.target
    if (!(target instanceof HTMLElement)) return
    if (target.closest("button, select, textarea, [data-composer-interactive='true']"))
      return
    textareaRef.current?.focus()
  }

  const handleAttachmentChange = (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    event.target.value = ""
    if (!files.length) return
    onUploadAttachments(files)
  }

  return (
    <footer className="relative z-30 shrink-0 bg-linear-to-t from-background via-background/95 to-background/70 px-3 pb-3 pt-2 sm:px-5">
      <div
        ref={composerRootRef}
        className="mx-auto flex w-full max-w-4xl flex-col gap-2 rounded-xl border border-border bg-background p-1 shadow-lg shadow-black/10 dark:bg-card"
        onMouseDown={focusComposer}
      >
        <div className="relative">
          <AgentChatComposerPanels
            availableProducts={availableProducts}
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingProducts={isLoadingProducts}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isStreaming={isStreaming}
            onFocusComposer={() => textareaRef.current?.focus()}
            onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
            onRuntimeLearningLevelChange={onRuntimeLearningLevelChange}
            onRuntimeModelChange={onRuntimeModelChange}
            onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
            onSelectPlaybook={onSelectPlaybook}
            onStrategyChange={onStrategyChange}
            onClearProductContext={onClearProductContext}
            onToggleProductContext={onToggleProductContext}
            onToggleRuntimeRole={onToggleRuntimeRole}
            playbooks={playbooks}
            productContextOpen={productContextOpen}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            selectedProducts={selectedProducts}
            settingsOpen={settingsOpen}
            strategyId={strategyId}
          />

          <div className="grid gap-1 rounded-lg bg-background p-1.5 shadow-sm">
            <input
              ref={fileInputRef}
              className="hidden"
              multiple
              type="file"
              onChange={handleAttachmentChange}
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
                  onSend={sendAndClosePanels}
                  onStop={onStop}
                />
              </div>
            </div>

            <AgentChatAttachmentStrip attachments={attachments} />

            <AgentChatComposerToolbar
              availableProducts={availableProducts}
              canSend={canSend}
              isLoggedIn={isLoggedIn}
              isLoadingPlaybooks={isLoadingPlaybooks}
              isLoadingProducts={isLoadingProducts}
              isLoadingRuntimeOptions={isLoadingRuntimeOptions}
              isStreaming={isStreaming}
              onAttach={() => fileInputRef.current?.click()}
              onRuntimeCollaborationModeChange={onRuntimeCollaborationModeChange}
              onRuntimeLearningLevelChange={onRuntimeLearningLevelChange}
              onRuntimeModelChange={onRuntimeModelChange}
              onRuntimePrimaryRoleChange={onRuntimePrimaryRoleChange}
              onSelectPlaybook={(playbook) => {
                onSelectPlaybook(playbook)
                queueMicrotask(() => textareaRef.current?.focus())
              }}
              onSend={sendAndClosePanels}
              onStop={onStop}
              onStrategyChange={onStrategyChange}
              onClearProductContext={onClearProductContext}
              onToggleProductContext={onToggleProductContext}
              onToggleProductPanel={(open) => {
                setProductContextOpen(open)
                if (open) setSettingsOpen(false)
              }}
              onToggleRuntimeRole={onToggleRuntimeRole}
              onToggleSettingsPanel={() => {
                setSettingsOpen((open) => {
                  if (!open) setProductContextOpen(false)
                  return !open
                })
              }}
              playbooks={playbooks}
              productContextOpen={productContextOpen}
              runtimeOptions={runtimeOptions}
              runtimeSelection={runtimeSelection}
              selectedProducts={selectedProducts}
              settingsOpen={settingsOpen}
              strategyId={strategyId}
            />
          </div>
        </div>
      </div>
    </footer>
  )
}
