import { createRef } from "react"
import { renderHook, waitFor } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { useStudentLearningProbeVisibility } from "./use-student-learning-probe-visibility"

describe("useStudentLearningProbeVisibility", () => {
  it("keeps newly returned feedback in view instead of scrolling past it", async () => {
    const scrollRoot = document.createElement("section")
    const card = document.createElement("div")
    const resultBlock = document.createElement("div")
    const scrollTo = vi.fn()
    scrollRoot.dataset.agentChatThreadScrollRoot = "true"
    resultBlock.dataset.learningProbeResult = "true"
    card.append(resultBlock)
    scrollRoot.append(card)
    Object.defineProperties(scrollRoot, {
      clientHeight: { configurable: true, value: 600 },
      scrollHeight: { configurable: true, value: 1200 },
      scrollTop: { configurable: true, value: 700 },
      scrollTo: { configurable: true, value: scrollTo },
    })
    const getBoundingClientRect = vi
      .spyOn(HTMLElement.prototype, "getBoundingClientRect")
      .mockImplementation(function (this: HTMLElement) {
        if (this === scrollRoot) {
          return { bottom: 700, top: 100 } as DOMRect
        }
        if (this === resultBlock) {
          return { bottom: 120, top: -100 } as DOMRect
        }
        return { bottom: 0, top: 0 } as DOMRect
      })
    document.body.append(scrollRoot)
    const cardRef = createRef<HTMLDivElement>()
    cardRef.current = card

    renderHook(() =>
      useStudentLearningProbeVisibility({
        cardRef,
        probeId: "probe-feedback-position",
        ready: true,
        result: {
          demonstratedPoints: [],
          id: "probe-feedback-position",
          outcome: "correct",
          sourceLabels: [],
          status: "answered",
        },
      })
    )

    await waitFor(() =>
      expect(scrollTo).toHaveBeenCalledWith({
        behavior: "auto",
        top: 488,
      })
    )

    getBoundingClientRect.mockRestore()
    scrollRoot.remove()
  })
})
