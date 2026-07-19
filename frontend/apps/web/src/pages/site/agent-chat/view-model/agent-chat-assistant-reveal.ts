export function createAssistantRevealController({
  chunkSize,
  intervalMs,
  onVisibleContentChange,
}: {
  chunkSize: number
  intervalMs: number
  onVisibleContentChange: (content: string) => void
}) {
  let visibleContent = ""
  let pendingText = ""
  let pendingReveal: ReturnType<typeof setTimeout> | null = null
  const revealResolvers: Array<() => void> = []

  function resolveIdle() {
    if (pendingText || pendingReveal) return
    const resolvers = revealResolvers.splice(0)
    for (const resolve of resolvers) resolve()
  }

  function cancelReveal() {
    if (!pendingReveal) return
    clearTimeout(pendingReveal)
    pendingReveal = null
  }

  function takeNextChunk() {
    return Array.from(pendingText).slice(0, chunkSize).join("")
  }

  function revealNextChunk() {
    pendingReveal = null
    if (!pendingText) {
      resolveIdle()
      return
    }

    const chunk = takeNextChunk()
    visibleContent += chunk
    pendingText = pendingText.slice(chunk.length)
    onVisibleContentChange(visibleContent)

    if (pendingText) {
      scheduleReveal()
      return
    }
    resolveIdle()
  }

  function scheduleReveal() {
    if (!pendingText || pendingReveal) return
    pendingReveal = setTimeout(revealNextChunk, intervalMs)
  }

  return {
    flush() {
      cancelReveal()
      if (pendingText) {
        visibleContent += pendingText
        pendingText = ""
        onVisibleContentChange(visibleContent)
      }
      resolveIdle()
    },
    getVisibleContent() {
      return visibleContent
    },
    queue(chunk: string) {
      pendingText += chunk
      scheduleReveal()
    },
    waitForIdle() {
      if (!pendingText && !pendingReveal) return Promise.resolve()
      return new Promise<void>((resolve) => {
        revealResolvers.push(resolve)
      })
    },
  }
}
