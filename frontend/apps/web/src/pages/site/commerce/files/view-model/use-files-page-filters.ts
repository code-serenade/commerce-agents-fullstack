import { useCallback, useMemo, useState } from "react"
import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import {
  filterTrainingFiles,
  hasActiveTrainingFileFilters,
  summarizeTrainingFilesFilterResult,
  type TrainingFileKindFilter,
  type TrainingFileReadStatusFilter,
} from "../domain/logic"
import {
  filterTrainingFilesByUsage,
  type TrainingFileUsageFilter,
} from "../domain/usage-filter"
import type { TrainingFileItem } from "../types"

export function useFilesPageFilters({
  attachedObjectKeys,
  files,
  selectedSources,
}: {
  attachedObjectKeys: Set<string>
  files: TrainingFileItem[]
  selectedSources: AgentChatSourceContextSelection[]
}) {
  const [keyword, setKeyword] = useState("")
  const [kindFilter, setKindFilter] = useState<TrainingFileKindFilter>("all")
  const [readStatusFilter, setReadStatusFilter] =
    useState<TrainingFileReadStatusFilter>("all")
  const [usageFilter, setUsageFilter] = useState<TrainingFileUsageFilter>("all")
  const filters = useMemo(
    () => ({
      keyword,
      kind: kindFilter,
      readStatus: readStatusFilter,
      usage: usageFilter,
    }),
    [keyword, kindFilter, readStatusFilter, usageFilter]
  )
  const hasActiveFilters = useMemo(
    () => hasActiveTrainingFileFilters(filters),
    [filters]
  )
  const filteredFiles = useMemo(() => {
    const filteredByBasicFields = filterTrainingFiles(files, filters)

    return filterTrainingFilesByUsage({
      attachedObjectKeys,
      files: filteredByBasicFields,
      selectedSources,
      usage: usageFilter,
    })
  }, [attachedObjectKeys, files, filters, selectedSources, usageFilter])
  const filterSummary = useMemo(
    () =>
      summarizeTrainingFilesFilterResult({
        filters,
        filteredFiles,
        totalFiles: files,
      }),
    [files, filteredFiles, filters]
  )

  const clearFilters = useCallback(() => {
    setKeyword("")
    setKindFilter("all")
    setReadStatusFilter("all")
    setUsageFilter("all")
  }, [])

  return {
    clearFilters,
    files: filteredFiles,
    filterSummary,
    hasActiveFilters,
    kindFilter,
    keyword,
    readStatusFilter,
    setKindFilter,
    setKeyword,
    setReadStatusFilter,
    setUsageFilter,
    usageFilter,
  }
}
