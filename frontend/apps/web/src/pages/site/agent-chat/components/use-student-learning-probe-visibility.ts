import { useEffect, type RefObject } from "react"
import type { LearningProbeResultResponse } from "@/api"

export function useStudentLearningProbeVisibility({
  cardRef,
  probeId,
  ready,
  result,
}: {
  cardRef: RefObject<HTMLDivElement | null>
  probeId?: string
  ready: boolean
  result?: LearningProbeResultResponse | null
}) {
  useEffect(() => {
    if (!probeId || !ready) return

    const card = cardRef.current
    const scrollRoot = card?.closest<HTMLElement>(
      '[data-agent-chat-thread-scroll-root="true"]'
    )
    if (!card || !scrollRoot) return

    const distanceFromBottom =
      scrollRoot.scrollHeight - scrollRoot.scrollTop - scrollRoot.clientHeight
    if (distanceFromBottom > Math.max(240, card.offsetHeight + 96)) return

    const resultBlock = card.querySelector<HTMLElement>(
      '[data-learning-probe-result="true"]'
    )
    if (result && resultBlock) {
      let revealFrame = 0
      const settleFrame = requestAnimationFrame(() => {
        revealFrame = requestAnimationFrame(() => {
          const currentResultBlock = card.querySelector<HTMLElement>(
            '[data-learning-probe-result="true"]'
          )
          if (!currentResultBlock) return

          const rootRect = scrollRoot.getBoundingClientRect()
          const resultRect = currentResultBlock.getBoundingClientRect()
          const resultTop =
            scrollRoot.scrollTop + resultRect.top - rootRect.top - 12
          scrollRoot.scrollTo({
            behavior: "auto",
            top: Math.max(0, resultTop),
          })
        })
      })
      return () => {
        cancelAnimationFrame(settleFrame)
        cancelAnimationFrame(revealFrame)
      }
    }

    scrollRoot.scrollTo({ behavior: "smooth", top: scrollRoot.scrollHeight })
  }, [cardRef, probeId, ready, result])
}
