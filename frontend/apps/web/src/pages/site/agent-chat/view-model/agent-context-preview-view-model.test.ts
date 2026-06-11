import { describe, expect, it } from "vitest"
import type { AgentContextPreviewResponse } from "../agent-chat-data"
import { getAgentContextPreviewModel } from "./agent-context-preview-view-model"

describe("getAgentContextPreviewModel", () => {
  it("summarizes safe preview metadata without exposing memory content", () => {
    const preview = {
      budget: {},
      includedRefs: ["product:p1", "conversation_file:f1"],
      memoryRecords: [
        {
          charCount: 180,
          key: "buyer-tone",
          kind: "preference",
          tags: ["tone", "  ", "campaign"],
          truncated: true,
        },
      ],
      metadataOnlyRefs: ["conversation_file:f2"],
      omittedRefs: ["campaign:c1"],
      warnings: ["  file f2 only has metadata  "],
      workspaceContext: {},
      workspacePack: {},
    } as AgentContextPreviewResponse

    const model = getAgentContextPreviewModel(preview)

    expect(model.readCounts).toEqual({
      included: 2,
      memory: 1,
      metadataOnly: 1,
      omitted: 1,
    })
    expect(model.refRows.map((row) => [row.ref, row.status])).toEqual([
      ["product:p1", "included"],
      ["conversation_file:f1", "included"],
      ["conversation_file:f2", "metadataOnly"],
      ["campaign:c1", "omitted"],
    ])
    expect(model.memoryRows).toEqual([
      {
        charCount: 180,
        id: "preference:buyer-tone",
        kind: "preference",
        name: "buyer-tone",
        tags: ["tone", "campaign"],
        truncated: true,
      },
    ])
    expect(JSON.stringify(model)).not.toContain("content")
    expect(model.warnings).toEqual(["file f2 only has metadata"])
  })

  it("summarizes audit fields without exposing raw context bodies", () => {
    const textKey = "text"
    const summaryKey = "summary"
    const titleKey = "title"
    const hiddenHistoryText = "hidden history body"
    const hiddenOlderSummary = "hidden older summary"
    const hiddenMemorySummary = "hidden memory rolling summary"
    const hiddenMemoryFact = "hidden memory fact"
    const hiddenMemoryDecision = "hidden memory decision"
    const hiddenMemoryConstraint = "hidden memory constraint"
    const hiddenMemoryQuestion = "hidden memory question"
    const hiddenLastTurn = "hidden last turn output"
    const hiddenSourceMapValue = "hidden source map value"
    const hiddenWorkspaceSummary = "hidden workspace summary"
    const hiddenPackText = "hidden workspace pack text"
    const hiddenPackSummary = "hidden pack summary"
    const preview = {
      budget: {
        conversationHistoryChars: 240,
        conversationMemoryChars: 80,
        historyMessageCount: 3,
        historyTotalMessageCount: 9,
        inputChars: 18,
        workspacePackChars: 360,
        workspacePackItems: 2,
      },
      conversationHistory: {
        messageCount: 3,
        olderSummary: hiddenOlderSummary,
        relevantOlderCount: 2,
        text: hiddenHistoryText,
        totalMessageCount: 9,
        truncated: true,
      },
      conversationMemory: {
        constraints: [{ [textKey]: hiddenMemoryConstraint }],
        decisions: [{ [textKey]: hiddenMemoryDecision }],
        facts: [{ [textKey]: hiddenMemoryFact }],
        lastAssistantOutput: hiddenLastTurn,
        openQuestions: [{ [textKey]: hiddenMemoryQuestion }],
        rollingSummary: hiddenMemorySummary,
        sourceMap: {
          secret: hiddenSourceMapValue,
        },
      },
      includedRefs: [],
      memoryRecords: [],
      metadataOnlyRefs: [],
      omittedRefs: [],
      warnings: [],
      workspaceContext: {
        attachmentSources: [
          {
            id: "file-1",
            refKey: "conversation_file:file-1",
            sourceType: "conversation_file",
            status: "readable",
            [titleKey]: "brief",
          },
        ],
        counts: {
          conversationFiles: 1,
          productMaterials: 1,
        },
        missing: ["campaign:missing"],
        readiness: "partial",
        [summaryKey]: hiddenWorkspaceSummary,
        warnings: ["workspace warning"],
      },
      workspacePack: {
        items: [
          {
            id: "file-1",
            refKey: "conversation_file:file-1",
            text: hiddenPackText,
            textChars: 280,
          },
          {
            id: "file-2",
            refKey: "conversation_file:file-2",
            textChars: 0,
          },
        ],
        omittedDetails: [{ reason: "too_large", refKey: "file-3" }],
        readiness: "partial",
        [summaryKey]: hiddenPackSummary,
        warnings: ["pack warning"],
      },
    } as AgentContextPreviewResponse

    const model = getAgentContextPreviewModel(preview)

    expect(model.auditSummary).toEqual({
      budget: {
        inputChars: 18,
        totalContextChars: 680,
      },
      conversation: {
        hasHistory: true,
        historyChars: 240,
        memoryChars: 80,
        messageCount: 3,
        relevantOlderCount: 2,
        totalMessageCount: 9,
        truncated: true,
      },
      conversationMemory: {
        available: true,
        constraintCount: 1,
        decisionCount: 1,
        factCount: 1,
        openQuestionCount: 1,
        summaryAvailable: true,
      },
      workspace: {
        missingCount: 1,
        readiness: "partial",
        sourceCount: 2,
        warningCount: 1,
      },
      workspacePack: {
        charCount: 360,
        itemCount: 2,
        omittedCount: 1,
        readiness: "partial",
        readableItemCount: 1,
        warningCount: 1,
      },
    })
    const serialized = JSON.stringify(model)
    expect(serialized).not.toContain(hiddenHistoryText)
    expect(serialized).not.toContain(hiddenOlderSummary)
    expect(serialized).not.toContain(hiddenMemorySummary)
    expect(serialized).not.toContain(hiddenMemoryFact)
    expect(serialized).not.toContain(hiddenMemoryDecision)
    expect(serialized).not.toContain(hiddenMemoryConstraint)
    expect(serialized).not.toContain(hiddenMemoryQuestion)
    expect(serialized).not.toContain(hiddenLastTurn)
    expect(serialized).not.toContain(hiddenSourceMapValue)
    expect(serialized).not.toContain(hiddenWorkspaceSummary)
    expect(serialized).not.toContain(hiddenPackText)
    expect(serialized).not.toContain(hiddenPackSummary)
  })

  it("summarizes audit fields from the safe backend response shape", () => {
    const preview = {
      budget: {
        conversationHistoryChars: 120,
        conversationMemoryChars: 90,
        historyMessageCount: 2,
        historyTotalMessageCount: 4,
        inputChars: 16,
        workspacePackChars: 999,
        workspacePackItems: 99,
      },
      conversationHistory: {
        conversationId: "conversation-1",
        messageCount: 3,
        relevantOlderCount: 2,
        totalMessageCount: 9,
        truncated: true,
      },
      conversationMemory: {
        constraintCount: 3,
        decisionCount: 2,
        factCount: 1,
        kind: "conversation_memory_summary",
        openQuestionCount: 4,
        rollingSummaryAvailable: true,
      },
      includedRefs: [],
      memoryRecords: [],
      metadataOnlyRefs: [],
      omittedRefs: [],
      warnings: [],
      workspaceContext: {
        counts: {
          conversationFiles: 20,
          productMaterials: 20,
        },
        missing: [],
        readiness: "ready",
        sourceCount: 5,
        warnings: [],
      },
      workspacePack: {
        counts: {
          conversationFiles: 20,
        },
        itemCount: 6,
        omitted: [],
        omittedDetails: [],
        readiness: "ready",
        readableItemCount: 4,
        textChars: 700,
        warnings: [],
      },
    } as AgentContextPreviewResponse

    const model = getAgentContextPreviewModel(preview)

    expect(model.auditSummary).toEqual({
      budget: {
        inputChars: 16,
        totalContextChars: 910,
      },
      conversation: {
        hasHistory: true,
        historyChars: 120,
        memoryChars: 90,
        messageCount: 3,
        relevantOlderCount: 2,
        totalMessageCount: 9,
        truncated: true,
      },
      conversationMemory: {
        available: true,
        constraintCount: 3,
        decisionCount: 2,
        factCount: 1,
        openQuestionCount: 4,
        summaryAvailable: true,
      },
      workspace: {
        missingCount: 0,
        readiness: "ready",
        sourceCount: 5,
        warningCount: 0,
      },
      workspacePack: {
        charCount: 700,
        itemCount: 6,
        omittedCount: 0,
        readiness: "ready",
        readableItemCount: 4,
        warningCount: 0,
      },
    })
  })
})
