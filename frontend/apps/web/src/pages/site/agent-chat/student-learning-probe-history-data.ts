import { useQuery } from "@tanstack/react-query"
import { getLearningProbeHistoryApi } from "@/api"

export function useStudentLearningProbeHistory(
  conversationId?: string,
  messageId?: string
) {
  return useQuery({
    queryKey: [
      "learning",
      "probe-history",
      conversationId ?? "",
      messageId ?? "",
    ],
    queryFn: () =>
      getLearningProbeHistoryApi({
        conversationId: conversationId ?? "",
        assistantMessageId: messageId ?? "",
      }),
    enabled: Boolean(conversationId && messageId),
    retry: false,
  })
}
