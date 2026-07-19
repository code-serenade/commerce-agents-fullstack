import { useQuery } from "@tanstack/react-query"
import { getConversationFileAccessApi } from "@/api"

const ACCESS_URL_STALE_TIME_MS = 5 * 60 * 1000

export function useFileImageAccess(fileId: string) {
  return useQuery({
    queryKey: ["commerce", "files", "access", fileId],
    queryFn: () => getConversationFileAccessApi(fileId),
    retry: 1,
    staleTime: ACCESS_URL_STALE_TIME_MS,
  })
}
