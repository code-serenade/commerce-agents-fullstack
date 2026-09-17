import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { createElement, type ReactNode } from "react"
import { vi } from "vitest"
import {
  engageLearningProbeApi,
  getLearningProbeHistoryApi,
  getLearningProbeResultApi,
  skipLearningProbeApi,
  submitLearningProbeChoiceApi,
} from "@/api"
import type { LearningProbeResponse, LearningProbeResultResponse } from "@/api"

export const CONVERSATION_ID = "conversation-a"
export const PROBE_ID = "probe-a"
export const STORAGE_KEY = `learning-engaged-probe:${CONVERSATION_ID}`
export const QUESTION = "How would you diagnose the funnel?"

export function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children)
  }
}

export function createDeferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve
    reject = promiseReject
  })
  return { promise, reject, resolve }
}

export function createProbe(
  overrides: Partial<LearningProbeResponse> = {}
): LearningProbeResponse {
  return {
    choices: [{ id: "compare_funnel", text: "Compare funnel stages" }],
    engaged: false,
    expiresAt: "2026-08-28T00:00:00Z",
    id: PROBE_ID,
    interactionMode: "contextual_micro",
    knowledgeCode: "traffic",
    questionText: QUESTION,
    questionType: "single_choice",
    reasonCode: "context_assessment_turn",
    rewardCode: "progress_feedback",
    ...overrides,
  }
}

export function createResult(
  overrides: Partial<LearningProbeResultResponse> = {}
): LearningProbeResultResponse {
  return {
    demonstratedPoints: ["compare_funnel"],
    id: PROBE_ID,
    outcome: "correct",
    score: 1,
    selectedChoiceId: "compare_funnel",
    selectedChoiceText: "Compare funnel stages",
    sourceLabels: [],
    status: "answered",
    ...overrides,
  }
}

export function resetLearningProbeMocks() {
  window.sessionStorage.clear()
  vi.clearAllMocks()
  vi.mocked(engageLearningProbeApi).mockResolvedValue({
    id: PROBE_ID,
    status: "offered",
  })
  vi.mocked(getLearningProbeHistoryApi).mockResolvedValue([])
  vi.mocked(getLearningProbeResultApi).mockResolvedValue(
    createResult({
      demonstratedPoints: [],
      outcome: undefined,
      status: "offered",
    })
  )
  vi.mocked(skipLearningProbeApi).mockResolvedValue({
    id: PROBE_ID,
    status: "skipped",
  })
  vi.mocked(submitLearningProbeChoiceApi).mockResolvedValue(createResult())
}
