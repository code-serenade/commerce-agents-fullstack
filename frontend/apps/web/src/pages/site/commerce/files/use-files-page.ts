import { useCallback, useMemo, useState } from "react"
import { useSelector } from "react-redux"
import { useQueryClient } from "@tanstack/react-query"
import type { RootState } from "@/store"
import { useAgentChatContext } from "@/pages/site/agent-chat/context/use-agent-chat-context"
import {
  fileConversationUseState,
  fileDisplayName,
  fileObjectKey,
  formatFileSize,
  summarizeTrainingFilesDeleteImpact,
  summarizeTrainingFilesUsage,
} from "./domain/logic"
import { summarizeTrainingFilesTurnSelection } from "./domain/turn-selection-summary"
import { useFilesData } from "./files-data"
import { useFilesPageActions } from "./view-model/use-files-page-actions"
import { useFilesPageFilters } from "./view-model/use-files-page-filters"

export function useFilesPage() {
  const user = useSelector((state: RootState) => state.auth.user)
  const queryClient = useQueryClient()
  const chat = useAgentChatContext()
  const {
    attachExistingSource,
    availableSources,
    clearSelectedSources,
    isLoggedIn,
    isStreaming,
    removeSourcesByObjects,
    selectedSources,
    toggleSourceContext,
    uploadAttachments,
  } = chat
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [detailFileId, setDetailFileId] = useState<string | null>(null)
  const [selectedFileIds, setSelectedFileIds] = useState<string[]>([])
  const data = useFilesData(Boolean(user))
  const attachedObjectKeys = useMemo(
    () =>
      new Set(
        availableSources
          .filter((source) => source.bucket && source.objectKey)
          .map((source) => `${source.bucket}:${source.objectKey}`)
      ),
    [availableSources]
  )
  const filters = useFilesPageFilters({
    attachedObjectKeys,
    files: data.files,
    selectedSources,
  })
  const files = filters.files
  const selectedFileIdSet = useMemo(
    () => new Set(selectedFileIds),
    [selectedFileIds]
  )
  const selectedFiles = useMemo(
    () => data.files.filter((file) => selectedFileIdSet.has(file.id)),
    [data.files, selectedFileIdSet]
  )
  const detailFile = useMemo(
    () => data.files.find((file) => file.id === detailFileId) ?? null,
    [data.files, detailFileId]
  )
  const usageSummary = useMemo(
    () =>
      summarizeTrainingFilesUsage({
        attachedObjectKeys,
        files: data.files,
        selectedSources,
      }),
    [attachedObjectKeys, data.files, selectedSources]
  )
  const turnSelectionSummary = useMemo(
    () =>
      summarizeTrainingFilesTurnSelection({
        attachedObjectKeys,
        files: data.files,
        selectedSources,
      }),
    [attachedObjectKeys, data.files, selectedSources]
  )
  const deleteImpact = useMemo(
    () =>
      summarizeTrainingFilesDeleteImpact({
        attachedObjectKeys,
        files: selectedFiles,
        selectedSources,
      }),
    [attachedObjectKeys, selectedFiles, selectedSources]
  )
  const toggleFileSelection = useCallback((fileId: string) => {
    setSelectedFileIds((current) =>
      current.includes(fileId)
        ? current.filter((id) => id !== fileId)
        : [...current, fileId]
    )
  }, [])

  const clearSelection = useCallback(() => {
    setSelectedFileIds([])
  }, [])

  const openFileDetail = useCallback((file: (typeof files)[number]) => {
    setDetailFileId(file.id)
  }, [])

  const closeFileDetail = useCallback(() => {
    setDetailFileId(null)
  }, [])

  const refreshFileViews = useCallback(async () => {
    await Promise.all([
      data.refetch(),
      queryClient.invalidateQueries({
        queryKey: ["agent-chat", "source-library"],
      }),
      queryClient.invalidateQueries({ queryKey: ["commerce", "files"] }),
    ])
  }, [data, queryClient])

  const actions = useFilesPageActions({
    attachExistingSource,
    availableSources,
    bulkDeleteFiles: data.bulkDeleteFiles,
    isLoggedIn,
    isStreaming,
    refreshFileViews,
    removeSourcesByObjects,
    selectedFiles,
    setSelectedFileIds,
    toggleSourceContext,
    uploadAttachments,
  })

  const deleteSelectedFiles = useCallback(async () => {
    await actions.deleteSelectedFiles()
    setDeleteDialogOpen(false)
  }, [actions])

  return {
    attachedObjectKeys,
    clearFilters: filters.clearFilters,
    clearSelection,
    dataFileCount: data.files.length,
    deleteSelectedFiles,
    deleteImpact,
    deleteDialogOpen,
    deleting: actions.deleting,
    detailFile,
    error: data.error,
    filterSummary: filters.filterSummary,
    files,
    hasActiveFilters: filters.hasActiveFilters,
    canUploadFiles:
      isLoggedIn && !isStreaming && !actions.deleting && !actions.uploading,
    canClearSelectedForTurn: !isStreaming && selectedSources.length > 0,
    clearSelectedForTurn: clearSelectedSources,
    getFileConversationUseState: (file: (typeof files)[number]) =>
      fileConversationUseState({
        attachedObjectKeys,
        file,
        selectedSources,
      }),
    formatFileSize,
    getFileName: fileDisplayName,
    getFileObjectKey: fileObjectKey,
    kindFilter: filters.kindFilter,
    keyword: filters.keyword,
    loading: data.loading,
    readStatusFilter: filters.readStatusFilter,
    selectedFileIds: selectedFileIdSet,
    selectedFiles,
    setDeleteDialogOpen,
    setKindFilter: filters.setKindFilter,
    setKeyword: filters.setKeyword,
    setReadStatusFilter: filters.setReadStatusFilter,
    setUsageFilter: filters.setUsageFilter,
    closeFileDetail,
    openFileDetail,
    uploadFiles: actions.uploadFiles,
    uploading: actions.uploading,
    usageFilter: filters.usageFilter,
    usingFileIds: actions.usingFileIds,
    usageSummary,
    handleUploadChange: actions.handleUploadChange,
    toggleFileSelection,
    turnSelectionSummary,
    toggleFileForCurrentTurn: actions.toggleFileForCurrentTurn,
  }
}
