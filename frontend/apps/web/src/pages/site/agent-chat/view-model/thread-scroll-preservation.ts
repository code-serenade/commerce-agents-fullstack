export interface ThreadScrollSnapshot {
  activeElement: HTMLElement | null
  anchorOffsetTop?: number
  anchorSelector?: string
  root: HTMLElement
  scrollTop: number
}

export function createThreadScrollSnapshot(
  root: HTMLElement | null,
  messageId: string | null
): ThreadScrollSnapshot | null {
  if (!root) return null
  const anchorSelector = messageId
    ? `[data-message-id="${escapeCssAttributeValue(messageId)}"]`
    : undefined
  const anchor = anchorSelector
    ? root.querySelector<HTMLElement>(anchorSelector)
    : null
  const activeElement = document.activeElement

  return {
    activeElement:
      activeElement instanceof HTMLElement && root.contains(activeElement)
        ? activeElement
        : null,
    anchorOffsetTop: anchor
      ? anchor.getBoundingClientRect().top - root.getBoundingClientRect().top
      : undefined,
    anchorSelector,
    root,
    scrollTop: root.scrollTop,
  }
}

export function restoreThreadScroll(snapshot: ThreadScrollSnapshot | null) {
  if (!snapshot) return

  const restore = () => {
    const { activeElement, anchorOffsetTop, anchorSelector, root, scrollTop } =
      snapshot
    activeElement?.focus({ preventScroll: true })
    const anchor = anchorSelector
      ? root.querySelector<HTMLElement>(anchorSelector)
      : null
    if (anchor && anchorOffsetTop !== undefined) {
      const nextAnchorOffsetTop =
        anchor.getBoundingClientRect().top - root.getBoundingClientRect().top
      root.scrollTop += nextAnchorOffsetTop - anchorOffsetTop
      return
    }
    root.scrollTop = scrollTop
  }

  restore()
  requestAnimationFrame(() => {
    restore()
    requestAnimationFrame(() => {
      restore()
      window.setTimeout(restore, 0)
    })
  })
}

function escapeCssAttributeValue(value: string) {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") {
    return CSS.escape(value)
  }
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')
}
