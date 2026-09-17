import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo, useRef, useState } from "react"
import {
  engageLearningProbeApi,
  getLearningProbeResultApi,
  getNextLearningProbeApi,
  skipLearningProbeApi,
  submitLearningProbeChoiceApi,
} from "@/api"
import { useStudentLearningProbeHistory } from "../student-learning-probe-history-data"
import {
  mergeCompletedLearningProbes,
  type CompletedLearningProbe,
  type DisplayableProbe,
} from "./student-learning-probe-history-view-model"
import {
  clearStoredProbe,
  readStoredProbe,
  toStoredProbe,
  writeStoredProbe,
} from "./student-learning-probe-storage"
import type { StoredEngagedProbe } from "./student-learning-probe-storage"

export function useStudentLearningProbe({
  conversationId,
  messageId,
}: {
  conversationId?: string
  messageId?: string
}) {
  const queryClient = useQueryClient()
  const submitChoiceInFlightRef = useRef(false)
  const skipInFlightRef = useRef(false)
  const continueInFlightRef = useRef(false)
  const [continuationState, setContinuationState] = useState<{
    attempt: number
    conversationId?: string
    fromProbeId: string | null
    retainedProbe: DisplayableProbe | null
  }>({
    attempt: 0,
    conversationId,
    fromProbeId: null,
    retainedProbe: null,
  })
  const [dismissedProbeState, setDismissedProbeState] = useState<{
    conversationId?: string
    probeId: string | null
  }>({ conversationId, probeId: null })
  const [engagedProbeState, setEngagedProbeState] = useState<{
    conversationId?: string
    probeId: string | null
  }>({ conversationId, probeId: null })
  const [submittedProbeState, setSubmittedProbeState] = useState<{
    conversationId?: string
    probeId: string | null
  }>({ conversationId, probeId: null })
  const [completedProbeState, setCompletedProbeState] = useState<{
    conversationId?: string
    messageId?: string
    items: CompletedLearningProbe[]
  }>({ conversationId, messageId, items: [] })
  const [storedProbeState, setStoredProbeState] = useState<{
    conversationId?: string
    probe: StoredEngagedProbe | null
  }>(() => ({
    conversationId,
    probe: readStoredProbe(conversationId, messageId),
  }))
  const engagedProbeId =
    engagedProbeState.conversationId === conversationId
      ? engagedProbeState.probeId
      : null
  const storedProbe =
    storedProbeState.conversationId === conversationId
      ? storedProbeState.probe
      : readStoredProbe(conversationId, messageId)
  const activeContinuation =
    continuationState.conversationId === conversationId
      ? continuationState
      : null
  const localCompletedProbes = useMemo(
    () =>
      completedProbeState.conversationId === conversationId &&
      completedProbeState.messageId === messageId
        ? completedProbeState.items
        : [],
    [completedProbeState, conversationId, messageId]
  )
  const continueFromProbeId = activeContinuation?.fromProbeId ?? null
  const queryKey = [
    "learning",
    "probe",
    conversationId ?? "",
    messageId ?? "",
    continueFromProbeId ?? "",
    activeContinuation?.attempt ?? 0,
  ] as const

  const probeQuery = useQuery({
    queryKey,
    queryFn: () =>
      getNextLearningProbeApi({
        conversationId,
        assistantMessageId: messageId,
        continueFromProbeId: continueFromProbeId ?? undefined,
      }),
    enabled: Boolean(conversationId && messageId),
    retry: false,
  })

  const historyQuery = useStudentLearningProbeHistory(conversationId, messageId)

  const skipMutation = useMutation({
    mutationFn: (probeId: string) => skipLearningProbeApi(probeId),
    onSuccess: (_, probeId) => {
      setDismissedProbeState({ conversationId, probeId })
      clearStoredProbe(conversationId, messageId)
      setStoredProbeState({ conversationId, probe: null })
    },
  })

  const engageMutation = useMutation({
    mutationFn: (probeId: string) => engageLearningProbeApi(probeId),
    onSuccess: (_, probeId) => {
      setEngagedProbeState({ conversationId, probeId })
      if (!probeQuery.data) return
      const stored = toStoredProbe(probeQuery.data)
      setStoredProbeState({ conversationId, probe: stored })
      writeStoredProbe(conversationId, messageId, stored)
    },
  })

  const submitChoiceMutation = useMutation({
    mutationFn: ({
      probeId,
      choiceId,
    }: {
      probeId: string
      choiceId: string
    }) => submitLearningProbeChoiceApi(probeId, { choiceId }),
    onSuccess: (result, { probeId }) => {
      setSubmittedProbeState({ conversationId, probeId })
      queryClient.setQueryData(["learning", "probe-result", probeId], result)
      void queryClient.invalidateQueries({
        queryKey: [
          "learning",
          "probe-history",
          conversationId ?? "",
          messageId ?? "",
        ],
      })
    },
  })

  const retainedProbe = activeContinuation?.retainedProbe ?? null
  const availableProbe = probeQuery.data ?? storedProbe ?? retainedProbe
  const dismissedProbeId =
    dismissedProbeState.conversationId === conversationId
      ? dismissedProbeState.probeId
      : null
  const visibleProbe =
    availableProbe?.id === dismissedProbeId ? null : availableProbe
  const completedProbes = useMemo(() => {
    return mergeCompletedLearningProbes(
      historyQuery.data ?? [],
      localCompletedProbes,
      visibleProbe?.id
    )
  }, [historyQuery.data, localCompletedProbes, visibleProbe])

  const resultProbeId = useMemo(() => {
    if (!visibleProbe) return null
    if (
      submittedProbeState.conversationId === conversationId &&
      submittedProbeState.probeId === visibleProbe.id
    ) {
      return submittedProbeState.probeId
    }
    if (engagedProbeId === visibleProbe.id) return engagedProbeId
    if (visibleProbe.engaged) return visibleProbe.id
    return null
  }, [conversationId, engagedProbeId, submittedProbeState, visibleProbe])

  const resultQuery = useQuery({
    queryKey: ["learning", "probe-result", resultProbeId ?? ""],
    queryFn: () => getLearningProbeResultApi(resultProbeId ?? ""),
    enabled: Boolean(resultProbeId),
    retry: false,
    refetchInterval: (query) => (query.state.data?.outcome ? false : 2000),
  })

  const continuing = Boolean(continueFromProbeId && probeQuery.isFetching)

  useEffect(() => {
    if (!continuing) continueInFlightRef.current = false
  }, [continuing])

  return {
    completedProbes,
    probe: visibleProbe,
    result: resultQuery.data ?? null,
    resultFailed: Boolean(resultProbeId && resultQuery.isError),
    ready:
      Boolean(visibleProbe) ||
      (!probeQuery.isLoading && !probeQuery.isFetching),
    continuing,
    continueFailed: Boolean(continueFromProbeId && probeQuery.isError),
    continueUnavailable: Boolean(
      continueFromProbeId && probeQuery.isSuccess && probeQuery.data === null
    ),
    skipping: skipMutation.isPending,
    submittingChoice: submitChoiceMutation.isPending,
    onEngage: async (probeId: string) => {
      try {
        await engageMutation.mutateAsync(probeId)
        return true
      } catch {
        return false
      }
    },
    onSkip: async (probeId: string) => {
      if (skipInFlightRef.current || submitChoiceInFlightRef.current) {
        return false
      }
      skipInFlightRef.current = true
      try {
        await skipMutation.mutateAsync(probeId)
        return true
      } catch {
        return false
      } finally {
        skipInFlightRef.current = false
      }
    },
    onSubmitChoice: async (probeId: string, choiceId: string) => {
      if (submitChoiceInFlightRef.current || skipInFlightRef.current) {
        return false
      }
      submitChoiceInFlightRef.current = true
      try {
        await submitChoiceMutation.mutateAsync({ probeId, choiceId })
        return true
      } catch {
        return false
      } finally {
        submitChoiceInFlightRef.current = false
      }
    },
    onRetryResult: () => {
      void resultQuery.refetch()
    },
    onContinue: (probeId: string) => {
      if (continueInFlightRef.current) return
      continueInFlightRef.current = true
      if (visibleProbe?.id === probeId && resultQuery.data?.outcome) {
        setCompletedProbeState((current) => {
          const items =
            current.conversationId === conversationId &&
            current.messageId === messageId
              ? current.items
              : []
          if (items.some((item) => item.probe.id === probeId)) {
            return { conversationId, messageId, items }
          }
          return {
            conversationId,
            messageId,
            items: [
              ...items,
              { probe: visibleProbe, result: resultQuery.data },
            ],
          }
        })
      }
      setContinuationState((current) => ({
        attempt:
          current.conversationId === conversationId ? current.attempt + 1 : 1,
        conversationId,
        fromProbeId: probeId,
        retainedProbe:
          visibleProbe?.id === probeId
            ? visibleProbe
            : current.conversationId === conversationId
              ? current.retainedProbe
              : null,
      }))
    },
    onDismissResult: () => {
      if (visibleProbe) {
        setDismissedProbeState({ conversationId, probeId: visibleProbe.id })
      }
      void queryClient.cancelQueries({
        queryKey: ["learning", "probe", conversationId ?? "", messageId ?? ""],
      })
      clearStoredProbe(conversationId, messageId)
      setStoredProbeState({ conversationId, probe: null })
      setEngagedProbeState({ conversationId, probeId: null })
      setSubmittedProbeState({ conversationId, probeId: null })
      setCompletedProbeState({ conversationId, messageId, items: [] })
      setContinuationState({
        attempt: 0,
        conversationId,
        fromProbeId: null,
        retainedProbe: null,
      })
      continueInFlightRef.current = false
    },
  }
}
