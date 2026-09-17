import { act, renderHook, waitFor } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { getLearningProbeResultApi, getNextLearningProbeApi } from "@/api"
import type { LearningProbeResponse } from "@/api"
import {
  CONVERSATION_ID,
  createDeferred,
  createProbe,
  createResult,
  createWrapper,
  PROBE_ID,
  resetLearningProbeMocks,
} from "./use-student-learning-probe-test-utils"
import { useStudentLearningProbe } from "./use-student-learning-probe"

vi.mock("@/api", () => ({
  engageLearningProbeApi: vi.fn(),
  getLearningProbeHistoryApi: vi.fn(),
  getLearningProbeResultApi: vi.fn(),
  getNextLearningProbeApi: vi.fn(),
  skipLearningProbeApi: vi.fn(),
  submitLearningProbeChoiceApi: vi.fn(),
}))

describe("useStudentLearningProbe continuation", () => {
  beforeEach(resetLearningProbeMocks)

  it("keeps completed feedback while requesting multiple voluntary continuations", async () => {
    vi.mocked(getNextLearningProbeApi)
      .mockResolvedValueOnce(createProbe({ engaged: true }))
      .mockResolvedValueOnce(
        createProbe({
          choices: [{ id: "next", text: "Check the next funnel angle" }],
          engaged: true,
          id: "probe-next",
          questionText: "What would you check next?",
          reasonCode: "student_continued_assessment",
        })
      )
      .mockResolvedValueOnce(
        createProbe({
          id: "probe-third",
          questionText: "Which related signal would you verify third?",
          reasonCode: "student_continued_assessment",
        })
      )
    vi.mocked(getLearningProbeResultApi)
      .mockResolvedValueOnce(createResult({ canContinue: true }))
      .mockResolvedValueOnce(
        createResult({
          canContinue: true,
          id: "probe-next",
          selectedChoiceId: "next",
          selectedChoiceText: "Check the next funnel angle",
        })
      )
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-feedback",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() =>
      expect(hook.result.current.result?.outcome).toBe("correct")
    )

    act(() => hook.result.current.onContinue(PROBE_ID))
    await waitFor(() =>
      expect(getNextLearningProbeApi).toHaveBeenLastCalledWith({
        assistantMessageId: "assistant-feedback",
        continueFromProbeId: PROBE_ID,
        conversationId: CONVERSATION_ID,
      })
    )
    await waitFor(() =>
      expect(hook.result.current.probe?.id).toBe("probe-next")
    )
    await waitFor(() =>
      expect(hook.result.current.result?.id).toBe("probe-next")
    )
    expect(
      hook.result.current.completedProbes.map((item) => item.probe.id)
    ).toEqual([PROBE_ID])

    act(() => hook.result.current.onContinue("probe-next"))
    await waitFor(() =>
      expect(hook.result.current.probe?.id).toBe("probe-third")
    )
    expect(
      hook.result.current.completedProbes.map((item) => item.probe.id)
    ).toEqual([PROBE_ID, "probe-next"])
    expect(getNextLearningProbeApi).toHaveBeenLastCalledWith({
      assistantMessageId: "assistant-feedback",
      continueFromProbeId: "probe-next",
      conversationId: CONVERSATION_ID,
    })
  })

  it("retains completed feedback while continuation loads, fails, and retries", async () => {
    const continuation = createDeferred<LearningProbeResponse | null>()
    vi.mocked(getNextLearningProbeApi)
      .mockResolvedValueOnce(createProbe({ engaged: true }))
      .mockReturnValueOnce(continuation.promise)
      .mockResolvedValueOnce(null)
    vi.mocked(getLearningProbeResultApi).mockResolvedValueOnce(
      createResult({ canContinue: true })
    )
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-continuation",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() =>
      expect(hook.result.current.result?.outcome).toBe("correct")
    )

    act(() => hook.result.current.onContinue(PROBE_ID))
    await waitFor(() => expect(hook.result.current.continuing).toBe(true))
    expect(hook.result.current.probe?.id).toBe(PROBE_ID)
    expect(hook.result.current.result?.outcome).toBe("correct")

    continuation.reject(new Error("temporary failure"))
    await waitFor(() => expect(hook.result.current.continueFailed).toBe(true))
    expect(hook.result.current.result?.outcome).toBe("correct")

    act(() => hook.result.current.onContinue(PROBE_ID))
    await waitFor(() =>
      expect(hook.result.current.continueUnavailable).toBe(true)
    )
    expect(getNextLearningProbeApi).toHaveBeenCalledTimes(3)
    expect(hook.result.current.probe?.id).toBe(PROBE_ID)
  })

  it("keeps a dismissed result closed when an in-flight continuation finishes", async () => {
    const continuation = createDeferred<LearningProbeResponse | null>()
    vi.mocked(getNextLearningProbeApi)
      .mockResolvedValueOnce(createProbe({ engaged: true }))
      .mockReturnValueOnce(continuation.promise)
    vi.mocked(getLearningProbeResultApi).mockResolvedValueOnce(
      createResult({ canContinue: true })
    )
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-dismiss-continuation",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() =>
      expect(hook.result.current.result?.outcome).toBe("correct")
    )

    act(() => hook.result.current.onContinue(PROBE_ID))
    await waitFor(() => expect(hook.result.current.continuing).toBe(true))
    act(() => hook.result.current.onDismissResult())
    expect(hook.result.current.probe).toBeNull()

    continuation.resolve(
      createProbe({
        id: "probe-next-after-dismiss",
        reasonCode: "student_continued_assessment",
      })
    )
    await act(async () => Promise.resolve())
    expect(hook.result.current.probe).toBeNull()
  })

  it("retries a failed restored-result request without reopening choices", async () => {
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(
      createProbe({ engaged: true })
    )
    vi.mocked(getLearningProbeResultApi)
      .mockRejectedValueOnce(new Error("temporary result failure"))
      .mockResolvedValueOnce(createResult())
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-result-retry",
        }),
      { wrapper: createWrapper() }
    )

    await waitFor(() => expect(hook.result.current.resultFailed).toBe(true))
    expect(hook.result.current.probe?.engaged).toBe(true)
    act(() => hook.result.current.onRetryResult())
    await waitFor(() =>
      expect(hook.result.current.result?.outcome).toBe("correct")
    )
    expect(hook.result.current.resultFailed).toBe(false)
    expect(getLearningProbeResultApi).toHaveBeenCalledTimes(2)
  })
})
