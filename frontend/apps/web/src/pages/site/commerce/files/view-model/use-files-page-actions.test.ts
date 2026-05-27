import { act, renderHook } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { useFilesPageActions } from "./use-files-page-actions"
import { file } from "../domain/test-helpers"

describe("useFilesPageActions", () => {
  it("tracks row-level material use state and prevents duplicate attach clicks", async () => {
    let resolveAttach: (() => void) | undefined
    const attachExistingSource = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          resolveAttach = resolve
        })
    )

    const { result } = renderHook(() =>
      useFilesPageActions({
        attachExistingSource,
        availableSources: [],
        bulkDeleteFiles: vi.fn(),
        isLoggedIn: true,
        isStreaming: false,
        refreshFileViews: vi.fn(),
        removeSourcesByObjects: vi.fn(),
        selectedFiles: [],
        setSelectedFileIds: vi.fn(),
        toggleSourceContext: vi.fn(),
        uploadAttachments: vi.fn(),
      })
    )
    const target = file({ id: "file-source" })
    let firstUse: Promise<void> | undefined

    await act(async () => {
      firstUse = result.current.toggleFileForCurrentTurn(target)
    })

    expect(result.current.usingFileIds.has("file-source")).toBe(true)

    await act(async () => {
      void result.current.toggleFileForCurrentTurn(target)
    })

    expect(attachExistingSource).toHaveBeenCalledTimes(1)

    await act(async () => {
      resolveAttach?.()
      await firstUse
    })

    expect(result.current.usingFileIds.has("file-source")).toBe(false)
  })

  it("marks an already attached material while selecting it for this turn", async () => {
    const toggleSourceContext = vi.fn()
    const { result } = renderHook(() =>
      useFilesPageActions({
        attachExistingSource: vi.fn(),
        availableSources: [
          {
            bucket: "agent-files",
            id: "file-attached",
            kind: "conversation_file",
            objectKey: "chat/user/conversation/attached.pdf",
            readable: true,
            ref: "conversation_file:file-attached",
            status: "uploaded",
            title: ["attached", "pdf"].join("."),
          },
        ],
        bulkDeleteFiles: vi.fn(),
        isLoggedIn: true,
        isStreaming: false,
        refreshFileViews: vi.fn(),
        removeSourcesByObjects: vi.fn(),
        selectedFiles: [],
        setSelectedFileIds: vi.fn(),
        toggleSourceContext,
        uploadAttachments: vi.fn(),
      })
    )

    await act(async () => {
      await result.current.toggleFileForCurrentTurn(
        file({
          bucket: "agent-files",
          id: "file-attached",
          objectKey: "chat/user/conversation/attached.pdf",
        })
      )
    })

    expect(toggleSourceContext).toHaveBeenCalledTimes(1)
    expect(result.current.usingFileIds.has("file-attached")).toBe(false)
  })

  it("allows removing an already selected material from this turn", async () => {
    const toggleSourceContext = vi.fn()
    const source = {
      bucket: "agent-files",
      id: "file-selected",
      kind: "conversation_file" as const,
      objectKey: "chat/user/conversation/selected.pdf",
      readable: true,
      ref: "conversation_file:file-selected",
      status: "uploaded" as const,
      title: ["selected", "pdf"].join("."),
    }
    const { result } = renderHook(() =>
      useFilesPageActions({
        attachExistingSource: vi.fn(),
        availableSources: [source],
        bulkDeleteFiles: vi.fn(),
        isLoggedIn: true,
        isStreaming: false,
        refreshFileViews: vi.fn(),
        removeSourcesByObjects: vi.fn(),
        selectedFiles: [],
        setSelectedFileIds: vi.fn(),
        toggleSourceContext,
        uploadAttachments: vi.fn(),
      })
    )

    await act(async () => {
      await result.current.toggleFileForCurrentTurn(
        file({
          bucket: "agent-files",
          id: "file-selected",
          objectKey: "chat/user/conversation/selected.pdf",
        })
      )
    })

    expect(toggleSourceContext).toHaveBeenCalledWith(source)
  })
})
