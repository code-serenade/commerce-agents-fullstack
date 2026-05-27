import { useQuery } from "@tanstack/react-query"
import { bulkDeleteUserFilesApi, deleteUserFileApi, listUserFilesApi } from "@/api"
import type { TrainingFileItem } from "./types"

export function useFilesData(enabled: boolean) {
  const filesQuery = useQuery<TrainingFileItem[]>({
    queryKey: ["commerce", "files", 200],
    queryFn: () => listUserFilesApi({ limit: 200 }),
    enabled,
  })

  const error =
    enabled && filesQuery.error
      ? filesQuery.error instanceof Error
        ? filesQuery.error.message
        : "Failed to load files."
      : null

  return {
    bulkDeleteFiles: bulkDeleteUserFilesApi,
    deleteFile: deleteUserFileApi,
    error: enabled ? error : null,
    files: enabled ? filesQuery.data ?? [] : [],
    loading: enabled ? filesQuery.isLoading : false,
    refetch: filesQuery.refetch,
  }
}
