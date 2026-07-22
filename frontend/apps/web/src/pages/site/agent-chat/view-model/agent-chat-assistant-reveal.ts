export function createAssistantRevealController({
  onVisibleContentChange,
}: {
  onVisibleContentChange: (content: string) => void
}) {
  let visibleContent = ""

  return {
    flush() {
      // Deltas are rendered when received, so no delayed text remains to flush.
    },
    getVisibleContent() {
      return visibleContent
    },
    queue(chunk: string) {
      if (!chunk) return
      visibleContent += chunk
      onVisibleContentChange(visibleContent)
    },
    waitForIdle() {
      return Promise.resolve()
    },
  }
}
