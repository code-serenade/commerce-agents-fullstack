import { act, renderHook, waitFor } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import {
  getLearningProbeHistoryApi,
  getLearningProbeResultApi,
  getNextLearningProbeApi,
} from "@/api"
import {
  CONVERSATION_ID,
  createProbe,
  createResult,
  createWrapper,
  PROBE_ID,
  QUESTION,
  resetLearningProbeMocks,
  STORAGE_KEY,
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

describe("useStudentLearningProbe restoration", () => {
  beforeEach(resetLearningProbeMocks)

  it("restores completed questions from the server without duplicating the current one", async () => {
    const currentProbe = createProbe({ engaged: true, id: "probe-current" })
    const previousProbe = createProbe({
      engaged: true,
      id: "probe-previous",
      questionText: "Which title detail should be verified first?",
    })
    vi.mocked(getNextLearningProbeApi).mockResolvedValue(currentProbe)
    vi.mocked(getLearningProbeResultApi).mockResolvedValue(
      createResult({ id: currentProbe.id })
    )
    vi.mocked(getLearningProbeHistoryApi).mockResolvedValue([
      {
        probe: previousProbe,
        result: createResult({ id: previousProbe.id }),
      },
      {
        probe: currentProbe,
        result: createResult({ id: currentProbe.id }),
      },
    ])

    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-history",
        }),
      { wrapper: createWrapper() }
    )

    await waitFor(() =>
      expect(
        hook.result.current.completedProbes.map((item) => item.probe.id)
      ).toEqual([previousProbe.id])
    )
    expect(getLearningProbeHistoryApi).toHaveBeenCalledWith({
      assistantMessageId: "assistant-history",
      conversationId: CONVERSATION_ID,
    })
  })

  it("restores an engaged probe after the original assistant block unmounts", async () => {
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(
      createProbe({
        choices: [
          { id: "compare_funnel", text: "Compare funnel stages" },
          { id: "increase_clicks", text: "Only increase clicks" },
        ],
        reasonCode: "context_unobserved",
      })
    )
    const first = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-a",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(first.result.current.probe?.id).toBe(PROBE_ID))
    await act(async () => {
      expect(await first.result.current.onEngage(PROBE_ID)).toBe(true)
    })
    const messageStorageKey = `${STORAGE_KEY}:assistant-a`
    expect(window.sessionStorage.getItem(messageStorageKey)).toContain(PROBE_ID)
    expect(window.sessionStorage.getItem(messageStorageKey)).toContain(
      "compare_funnel"
    )
    first.unmount()

    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(null)
    const second = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-a",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(second.result.current.probe?.id).toBe(PROBE_ID))
    expect(second.result.current.probe?.choices).toHaveLength(2)
    await waitFor(() =>
      expect(getLearningProbeResultApi).toHaveBeenCalledWith(PROBE_ID)
    )

    act(() => second.result.current.onDismissResult())
    expect(window.sessionStorage.getItem(messageStorageKey)).toBeNull()
    expect(second.result.current.probe).toBeNull()
  })

  it("keeps the in-memory flow usable when session storage is unavailable", async () => {
    vi.mocked(getNextLearningProbeApi).mockResolvedValueOnce(
      createProbe({ choices: [], questionType: "open_response" })
    )
    const setItem = vi
      .spyOn(Storage.prototype, "setItem")
      .mockImplementationOnce(() => {
        throw new DOMException("storage blocked", "SecurityError")
      })
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-storage-blocked",
        }),
      { wrapper: createWrapper() }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))
    await act(async () => {
      expect(await hook.result.current.onEngage(PROBE_ID)).toBe(true)
    })
    expect(hook.result.current.probe?.id).toBe(PROBE_ID)
    setItem.mockRestore()

    const removeItem = vi
      .spyOn(Storage.prototype, "removeItem")
      .mockImplementationOnce(() => {
        throw new DOMException("storage blocked", "SecurityError")
      })
    expect(() => act(() => hook.result.current.onDismissResult())).not.toThrow()
    expect(hook.result.current.probe).toBeNull()
    removeItem.mockRestore()
  })

  it("keeps legacy stored probes compatible as open responses", async () => {
    window.sessionStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ id: PROBE_ID, questionText: QUESTION })
    )
    vi.mocked(getNextLearningProbeApi).mockResolvedValue(null)
    const hook = renderHook(
      () =>
        useStudentLearningProbe({
          conversationId: CONVERSATION_ID,
          messageId: "assistant-legacy",
        }),
      { wrapper: createWrapper() }
    )

    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))
    expect(hook.result.current.probe?.questionType).toBe("open_response")
    expect(hook.result.current.probe?.choices).toEqual([])
  })

  it("switches restored feedback with the active conversation", async () => {
    const secondConversationId = "conversation-b"
    const secondProbeId = "probe-b"
    window.sessionStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ id: PROBE_ID, questionText: QUESTION })
    )
    window.sessionStorage.setItem(
      `learning-engaged-probe:${secondConversationId}`,
      JSON.stringify({ id: secondProbeId, questionText: "Second question" })
    )
    vi.mocked(getNextLearningProbeApi).mockResolvedValue(null)
    const hook = renderHook(
      ({ conversationId }) =>
        useStudentLearningProbe({
          conversationId,
          messageId: "assistant-latest",
        }),
      {
        initialProps: { conversationId: CONVERSATION_ID },
        wrapper: createWrapper(),
      }
    )
    await waitFor(() => expect(hook.result.current.probe?.id).toBe(PROBE_ID))

    hook.rerender({ conversationId: secondConversationId })
    await waitFor(() =>
      expect(hook.result.current.probe?.id).toBe(secondProbeId)
    )
  })
})
