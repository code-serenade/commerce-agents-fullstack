import { describe, expect, it } from "vitest"
import {
  fileDisplayName,
  fileConversationUseState,
  fileObjectKey,
  fileReadableState,
  formatFileSize,
  summarizeTrainingFilesDeleteImpact,
  summarizeTrainingFilesUsage,
} from "./logic"
import { file } from "./test-helpers"

describe("commerce files logic", () => {
  it("derives file names, object keys, sizes, and readable state", () => {
    const readable = file({
      fileName: null,
      objectKey: "chat/user/conversation/report.docx",
      fileSize: 1536,
      metadata: { extraction: { status: "ready" } },
    })

    expect(fileDisplayName(readable)).toBe("report.docx")
    expect(fileObjectKey(readable)).toBe(
      "agent-files:chat/user/conversation/report.docx"
    )
    expect(formatFileSize(readable.fileSize)).toBe("1.5 KB")
    expect(fileReadableState(readable)).toBe("readable")
  })

  it("marks failed and unsupported extraction states distinctly", () => {
    expect(
      fileReadableState(file({ metadata: { extractionStatus: "failed" } }))
    ).toBe("failed")
    expect(
      fileReadableState(file({ metadata: { extractionStatus: "unsupported" } }))
    ).toBe("unsupported")
    expect(fileReadableState(file({ metadata: {} }))).toBe("pending")
  })

  it("distinguishes available, attached, and selected-for-turn files", () => {
    const sourceTitle = ["source", "pdf"].join(".")
    const sourceFile = file({
      bucket: "agent-files",
      id: "file-source",
      objectKey: "chat/user/conversation/source.pdf",
    })
    const selectedSources = [
      {
        bucket: sourceFile.bucket,
        id: sourceFile.id,
        kind: "conversation_file" as const,
        objectKey: sourceFile.objectKey,
        readable: true,
        ref: `conversation_file:${sourceFile.id}`,
        status: "uploaded" as const,
        title: sourceTitle,
      },
    ]

    expect(
      fileConversationUseState({
        attachedObjectKeys: new Set(),
        file: sourceFile,
        selectedSources: [],
      })
    ).toBe("available")
    expect(
      fileConversationUseState({
        attachedObjectKeys: new Set([fileObjectKey(sourceFile)]),
        file: sourceFile,
        selectedSources: [],
      })
    ).toBe("attached")
    expect(
      fileConversationUseState({
        attachedObjectKeys: new Set([fileObjectKey(sourceFile)]),
        file: sourceFile,
        selectedSources,
      })
    ).toBe("selectedForTurn")
  })

  it("summarizes reusable material usage for the file manager", () => {
    const selected = file({
      bucket: "agent-files",
      id: "selected",
      objectKey: "chat/user/conversation/selected.pdf",
      metadata: { extractionStatus: "ready" },
    })
    const attached = file({
      bucket: "agent-files",
      id: "attached",
      objectKey: "chat/user/conversation/attached.pdf",
      metadata: { extractionStatus: "ready" },
    })
    const available = file({
      bucket: "agent-files",
      id: "available",
      objectKey: "chat/user/conversation/available.pdf",
      metadata: { extractionStatus: "failed" },
    })

    expect(
      summarizeTrainingFilesUsage({
        attachedObjectKeys: new Set([
          fileObjectKey(selected),
          fileObjectKey(attached),
        ]),
        files: [selected, attached, available],
        selectedSources: [
          {
            bucket: selected.bucket,
            id: selected.id,
            kind: "conversation_file" as const,
            objectKey: selected.objectKey,
            readable: true,
            ref: `conversation_file:${selected.id}`,
            status: "uploaded" as const,
            title: ["selected", "pdf"].join("."),
          },
        ],
      })
    ).toEqual({
      attached: 2,
      readable: 2,
      selectedForTurn: 1,
      total: 3,
    })
  })

  it("summarizes delete impact for selected material safety copy", () => {
    const selected = file({
      bucket: "agent-files",
      fileName: "selected.pdf",
      id: "selected",
      objectKey: "chat/user/conversation/selected.pdf",
      metadata: { extractionStatus: "ready" },
    })
    const attached = file({
      bucket: "agent-files",
      fileName: "attached.pdf",
      id: "attached",
      objectKey: "chat/user/conversation/attached.pdf",
      metadata: { extractionStatus: "ready" },
    })
    const available = file({
      bucket: "agent-files",
      fileName: "available.pdf",
      id: "available",
      objectKey: "chat/user/conversation/available.pdf",
      metadata: { extractionStatus: "failed" },
    })

    expect(
      summarizeTrainingFilesDeleteImpact({
        attachedObjectKeys: new Set([
          fileObjectKey(selected),
          fileObjectKey(attached),
        ]),
        files: [selected, attached, available],
        maxTargetNames: 2,
        selectedSources: [
          {
            bucket: selected.bucket,
            id: selected.id,
            kind: "conversation_file" as const,
            objectKey: selected.objectKey,
            readable: true,
            ref: `conversation_file:${selected.id}`,
            status: "uploaded" as const,
            title: ["selected", "pdf"].join("."),
          },
        ],
      })
    ).toEqual({
      attached: 2,
      hiddenNames: 1,
      readable: 2,
      selectedForTurn: 1,
      targetNames: ["selected.pdf", "attached.pdf"],
      total: 3,
    })
  })
})
