import { useQuery } from "@tanstack/react-query"
import { getLearningStudentDetailApi } from "@/api"
import { learningStudentDetailQueryKey } from "./constants"

export function useLearningStudentDetail(userId: string | undefined) {
  return useQuery({
    queryKey: learningStudentDetailQueryKey(userId ?? ""),
    queryFn: () => getLearningStudentDetailApi(userId ?? ""),
    enabled: Boolean(userId),
    retry: false,
  })
}
