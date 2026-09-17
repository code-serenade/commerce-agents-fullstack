import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  getLearningEvidenceApi,
  getLearningProfileApi,
  updateLearningSettingsApi,
} from "@/api"
import type { LearningEvidenceRange } from "@/api"

export const learningProfileQueryKey = ["learning", "profile"] as const

export function useLearningProfile() {
  return useQuery({
    queryKey: learningProfileQueryKey,
    queryFn: getLearningProfileApi,
    retry: false,
  })
}

export function useUpdateLearningSettings() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (enabled: boolean) =>
      updateLearningSettingsApi({ enabled }),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: learningProfileQueryKey,
      })
    },
  })
}

export function useLearningEvidence(range: LearningEvidenceRange) {
  return useQuery({
    queryKey: ["learning", "evidence", range],
    queryFn: () => getLearningEvidenceApi({ range }),
    retry: false,
  })
}
