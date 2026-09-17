import { act, renderHook, waitFor } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import {
  engageLearningProbeApi,
  getLearningProbeResultApi,
  getNextLearningProbeApi,
  skipLearningProbeApi,
  submitLearningProbeChoiceApi,
} from "@/api"
import type { LearningProbeResultResponse } from "@/api"
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

describe("useStudentLearningProbe", () => {
  beforeEach(resetLearningProbeMocks)

  it("submits a stable choice id and exposes the structured result", async () => {
    vi.mocked(getLearningProbeResultApi).mockResolvedValueOnce(createResult())
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(createProbe())
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-choice",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))

    await act(async () => {
      expect(
        await hook.result.current.onSubmitChoice(PROBE_ID, "compare_funnel")
      ).toBe(true)
    })

    expect(submitLearningProbeChoiceApi).toHaveBeenCalledWith(PROBE_ID, {
      choiceId: "compare_funnel",
    })
    expect(engageLearningProbeApi).not.toHaveBeenCalled()
    await waitFor(() =>
      expect(hook.result.current.result?.outcome).toBe("correct")
    )
    expect(hook.result.current.result?.selectedChoiceId).toBe("compare_funnel")
    expect(hook.result.current.result?.selectedChoiceText).toBe(
      "Compare funnel stages"
    )
  })

  it("keeps the probe mounted and deduplicates rapid choice submissions", async () => {
    const submission = createDeferred<LearningProbeResultResponse>()
    vi.mocked(submitLearningProbeChoiceApi).mockReturnValueOnce(
      submission.promise
    )
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(createProbe())
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-choice-pending",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))

    let firstSubmission!: Promise<boolean>
    let duplicateSubmission!: Promise<boolean>
    act(() => {
      firstSubmission = hook.result.current.onSubmitChoice(
        PROBE_ID,
        "compare_funnel"
      )
      duplicateSubmission = hook.result.current.onSubmitChoice(
        PROBE_ID,
        "compare_funnel"
      )
    })

    await waitFor(() => expect(hook.result.current.submittingChoice).toBe(true))
    expect(hook.result.current.ready).toBe(true)
    expect(hook.result.current.probe?.id).toBe(PROBE_ID)
    expect(submitLearningProbeChoiceApi).toHaveBeenCalledTimes(1)
    await expect(duplicateSubmission).resolves.toBe(false)

    submission.resolve(createResult())
    await act(async () => expect(await firstSubmission).toBe(true))
  })

  it("keeps the probe mounted while postponing and hides it only after success", async () => {
    const skip = createDeferred<{ id: string; status: string }>()
    vi.mocked(skipLearningProbeApi).mockReturnValueOnce(skip.promise)
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(createProbe())
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-skip-pending",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))

    let skipRequest!: Promise<boolean>
    act(() => {
      skipRequest = hook.result.current.onSkip(PROBE_ID)
    })
    await waitFor(() => expect(hook.result.current.skipping).toBe(true))
    expect(hook.result.current.probe?.id).toBe(PROBE_ID)

    skip.resolve({ id: PROBE_ID, status: "skipped" })
    await act(async () => expect(await skipRequest).toBe(true))
    await waitFor(() => expect(hook.result.current.probe).toBeNull())
    await act(async () => Promise.resolve())
    expect(getNextLearningProbeApi).toHaveBeenCalledTimes(1)
  })

  it("does not start postponing while a structured answer is in flight", async () => {
    const submission = createDeferred<LearningProbeResultResponse>()
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(createProbe())
    vi.mocked(submitLearningProbeChoiceApi).mockReturnValueOnce(
      submission.promise
    )
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-submit-skip-race",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))

    let submissionRequest!: Promise<boolean>
    let skipRequest!: Promise<boolean>
    act(() => {
      submissionRequest = hook.result.current.onSubmitChoice(
        PROBE_ID,
        "compare_funnel"
      )
      skipRequest = hook.result.current.onSkip(PROBE_ID)
    })

    await expect(skipRequest).resolves.toBe(false)
    expect(skipLearningProbeApi).not.toHaveBeenCalled()
    submission.resolve(createResult())
    await act(async () => expect(await submissionRequest).toBe(true))
  })
})
