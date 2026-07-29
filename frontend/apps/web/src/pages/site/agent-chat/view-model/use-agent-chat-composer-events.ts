import type { ChangeEvent, MouseEvent, RefObject } from "react"

export function useAgentChatComposerEvents({
  closePanels,
  fileInputRef,
  isStreaming,
  onSend,
  onUploadAttachments,
  textareaRef,
}: {
  closePanels: () => void
  fileInputRef: RefObject<HTMLInputElement | null>
  isStreaming: boolean
  onSend: () => void
  onUploadAttachments: (files: File[]) => void
  textareaRef: RefObject<HTMLTextAreaElement | null>
}) {
  const sendAndClosePanels = () => {
    closePanels()
    onSend()
  }

  const focusComposer = (event: MouseEvent<HTMLDivElement>) => {
    if (isStreaming) return
    const target = event.target
    if (!(target instanceof HTMLElement)) return
    if (
      target.closest("button, select, textarea, [data-composer-interactive='true']")
    ) {
      return
    }
    textareaRef.current?.focus()
  }

  const handleAttachmentChange = (event: ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files ?? [])
    event.target.value = ""
    if (!files.length) return
    onUploadAttachments(files)
  }

  return {
    focusComposer,
    handleAttachmentChange,
    openUploadFilePicker: () => fileInputRef.current?.click(),
    sendAndClosePanels,
  }
}
