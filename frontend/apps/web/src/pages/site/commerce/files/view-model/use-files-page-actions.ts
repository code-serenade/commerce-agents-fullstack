import { useCallback, useMemo, useRef, useState } from "react"
import type { ChangeEvent } from "react"
import type { TrainingFileItem } from "../types"
import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"

export function useFilesPageActions({
  attachExistingSource,
  availableSources,
  bulkDeleteFiles,
  isLoggedIn,
  isStreaming,
  refreshFileViews,
  removeSourcesByObjects,
  selectedFiles,
  setSelectedFileIds,
  toggleSourceContext,
  uploadAttachments,
}: {
  attachExistingSource: (fileId: string) => Promise<void>
  availableSources: AgentChatSourceContextSelection[]
  bulkDeleteFiles: (input: { fileIds: string[] }) => Promise<unknown>
  isLoggedIn: boolean
  isStreaming: boolean
  refreshFileViews: () => Promise<void>
  removeSourcesByObjects: (
    objects: Array<{ bucket: string; objectKey: string }>
  ) => void
  selectedFiles: TrainingFileItem[]
  setSelectedFileIds: (updater: (current: string[]) => string[]) => void
  toggleSourceContext: (source: AgentChatSourceContextSelection) => void
  uploadAttachments: (files: File[]) => Promise<void>
}) {
  const [deleting, setDeleting] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [usingFileIds, setUsingFileIds] = useState<string[]>([])
  const usingFileIdsRef = useRef(new Set<string>())
  const usingFileIdSet = useMemo(() => new Set(usingFileIds), [usingFileIds])

  const setFileUsing = useCallback((fileId: string, using: boolean) => {
    const next = new Set(usingFileIdsRef.current)
    if (using) {
      if (next.has(fileId)) return
      next.add(fileId)
    } else {
      next.delete(fileId)
    }
    usingFileIdsRef.current = next
    setUsingFileIds(Array.from(next))
  }, [])

  const uploadFiles = useCallback(
    async (files: File[]) => {
      if (!files.length || uploading || !isLoggedIn) return
      setUploading(true)
      try {
        await uploadAttachments(files)
        await refreshFileViews()
      } finally {
        setUploading(false)
      }
    },
    [isLoggedIn, refreshFileViews, uploadAttachments, uploading]
  )

  const handleUploadChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(event.target.files ?? [])
      event.target.value = ""
      void uploadFiles(files)
    },
    [uploadFiles]
  )

  const toggleFileForCurrentTurn = useCallback(
    async (file: TrainingFileItem) => {
      if (
        !isLoggedIn ||
        isStreaming ||
        deleting ||
        uploading ||
        usingFileIdsRef.current.has(file.id)
      ) {
        return
      }
      const source = availableSources.find(
        (item) =>
          item.id === file.id ||
          (item.bucket === file.bucket && item.objectKey === file.objectKey)
      )
      setFileUsing(file.id, true)
      try {
        if (source) {
          toggleSourceContext(source)
          return
        }
        await attachExistingSource(file.id)
      } finally {
        setFileUsing(file.id, false)
      }
    },
    [
      attachExistingSource,
      availableSources,
      deleting,
      isLoggedIn,
      isStreaming,
      setFileUsing,
      toggleSourceContext,
      uploading,
    ]
  )

  const deleteSelectedFiles = useCallback(async () => {
    if (!selectedFiles.length || deleting) return
    setDeleting(true)
    try {
      await bulkDeleteFiles({
        fileIds: selectedFiles.map((file) => file.id),
      })
      removeSourcesByObjects(
        selectedFiles.map((file) => ({
          bucket: file.bucket,
          objectKey: file.objectKey,
        }))
      )
      setSelectedFileIds(() => [])
      await refreshFileViews()
    } finally {
      setDeleting(false)
    }
  }, [
    bulkDeleteFiles,
    deleting,
    refreshFileViews,
    removeSourcesByObjects,
    selectedFiles,
    setSelectedFileIds,
  ])

  return {
    deleting,
    deleteSelectedFiles,
    handleUploadChange,
    uploadFiles,
    uploading,
    usingFileIds: usingFileIdSet,
    toggleFileForCurrentTurn,
  }
}
