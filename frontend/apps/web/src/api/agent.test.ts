import { describe, expect, it, vi } from "vitest"
import {
  buildMemoryLoadedEventFromContextPreview,
  previewAgentContextApi,
  streamAgentSessionApi,
} from "./agent"

function streamFromSse(blocks: string[]) {
  const encoder = new TextEncoder()
  return new ReadableStream<Uint8Array>({
    start(controller) {
      for (const block of blocks) {
        controller.enqueue(encoder.encode(block))
      }
      controller.close()
    },
  })
}

describe("streamAgentSessionApi", () => {
  it("uses output events as chat text when no delta is emitted", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        streamFromSse([
          'event: output\ndata: {"type":"output","content":"Final answer body"}\n\n',
          'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","summaryAvailable":true,"reportAvailable":true,"metadataAvailable":true}\n\n',
          "event: done\ndata: {}\n\n",
        ]),
        {
          status: 200,
          headers: { "Content-Type": "text/event-stream" },
        }
      )
    )
    vi.stubGlobal("fetch", fetchMock)

    const deltas: string[] = []
    const onDone = vi.fn()
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onAgentEvent: vi.fn(),
        onDelta: (content) => deltas.push(content),
        onDone,
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  it("uses event-name output as chat text even when the payload has no type", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: output\ndata: {"content":"Startup fallback body"}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const onDone = vi.fn()
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onDelta: (content) => deltas.push(content),
        onDone,
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Startup fallback body"])
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  it("waits for output after a workspace failure event before completing the chat UI", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: workspace_run_failed\ndata: {"type":"workspace_run_failed","id":"run-1","status":"failed"}\n\n',
            'event: output\ndata: {"content":"Startup fallback body"}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const onDone = vi.fn()
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onDelta: (content) => deltas.push(content),
        onDone,
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Startup fallback body"])
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  it("releases the chat UI when the final output event arrives without a done event", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: output\ndata: {"type":"output","content":"Final answer body"}\n\n',
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const onDone = vi.fn()
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onDelta: (content) => deltas.push(content),
        onDone,
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  it("deduplicates workspace run and final report artifact fallback output", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: output\ndata: {"type":"output","content":"Final answer body"}\n\n',
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","summaryAvailable":true,"reportAvailable":true,"metadataAvailable":true}\n\n',
            'event: workspace_artifact_created\ndata: {"type":"workspace_artifact_created","id":"artifact-1","title":"Agent final report","status":"active","artifactType":"final_report","content":"Final answer body","preview":"Final answer body"}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onDelta: (content) => deltas.push(content),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
  })

  it("releases the chat UI when the agent finished event arrives before workspace persistence", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: delta\ndata: {"content":"Final answer body"}\n\n',
            'event: finished\ndata: {"type":"finished","session_id":"session-1"}\n\n',
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","summaryAvailable":true,"reportAvailable":true,"metadataAvailable":true}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const onDone = vi.fn()
    await streamAgentSessionApi(
      { input: "diagnose a livestream funnel" },
      {
        onDelta: (content) => deltas.push(content),
        onDone,
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
    expect(onDone).toHaveBeenCalledTimes(1)
  })

  it("accepts safe conversation memory updated events", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: conversation_memory_updated\ndata: {"type":"conversation_memory_updated","conversationId":"conversation-1","persisted":true,"memorySummary":{"kind":"conversation_memory_summary","rollingSummaryAvailable":true,"factCount":2,"decisionCount":1,"constraintCount":1,"openQuestionCount":0,"lastTurnAvailable":true,"lastUserInputAvailable":true,"lastAssistantOutputAvailable":true,"sourceMapAvailable":true}}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const events: unknown[] = []
    await streamAgentSessionApi(
      { input: "continue the review" },
      {
        onAgentEvent: (event) => events.push(event),
        onDelta: vi.fn(),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(events).toEqual([
      expect.objectContaining({
        conversationId: "conversation-1",
        memorySummary: expect.objectContaining({
          factCount: 2,
          lastTurnAvailable: true,
          rollingSummaryAvailable: true,
          sourceMapAvailable: true,
        }),
        persisted: true,
        type: "conversation_memory_updated",
      }),
    ])
    expect(JSON.stringify(events)).not.toContain('"rollingSummary":')
    expect(JSON.stringify(events)).not.toContain('"lastUserInput":')
    expect(JSON.stringify(events)).not.toContain('"sourceMap":')
  })

  it("accepts safe context and rag context events", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: context_loaded\ndata: {"type":"context_loaded","session_id":"session-1","product_ids":["product-1"],"refs":["product:product-1"],"product_count":1,"knowledge_count":2,"source_count":3,"summary_available":true,"sources_available":true}\n\n',
            'event: rag_context_loaded\ndata: {"type":"rag_context_loaded","mode":"local_product_knowledge","queryTerms":["coffee"],"productIds":["product-1"],"knowledgeCount":2,"sourceCount":3,"summaryAvailable":true,"sourcesAvailable":true,"degraded":false,"externalSearchAvailable":true}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const events: unknown[] = []
    await streamAgentSessionApi(
      { input: "continue the review" },
      {
        onAgentEvent: (event) => events.push(event),
        onDelta: vi.fn(),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(events).toEqual([
      expect.objectContaining({
        productIds: ["product-1"],
        refs: ["product:product-1"],
        source_count: 3,
        sourcesAvailable: true,
        summaryAvailable: true,
        type: "context_loaded",
      }),
      expect.objectContaining({
        mode: "local_product_knowledge",
        productIds: ["product-1"],
        queryTerms: ["coffee"],
        sourceCount: 3,
        sourcesAvailable: true,
        summaryAvailable: true,
        type: "rag_context_loaded",
      }),
    ])
    expect(JSON.stringify(events)).not.toContain('"summary":')
    expect(JSON.stringify(events)).not.toContain('"sources":')
    expect(JSON.stringify(events)).not.toContain('"sources_json":')
  })

  it("accepts safe process and persistence events without raw payload fields", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: skill_result\ndata: {"type":"skill_result","session_id":"session-1","worker_id":"worker.search","task_id":"task-1","request_id":"req-1","skill_id":"search_competitor","role":"search","status":"success","summary":"search completed","output_available":true,"output_summary":{"kind":"object","available":true,"adapter_status":"live","evidence_packet":{"available":true,"sources":1,"usable_claims":1,"gaps":0,"confidence":0.72,"search_strength":"standard"}},"execution_mode":"adapter","risk_level":"low","approval_required":false}\n\n',
            'event: task_node_reported\ndata: {"type":"task_node_reported","session_id":"session-1","graph_id":"graph-1","node_id":"data","worker_id":"worker.data","report":{"graph_id":"graph-1","node_id":"data","task_id":"task-1","worker_id":"worker.data","role":"data","status":"completed","content_available":true,"role_output":{"summary_available":true,"finding_count":2,"recommendation_count":1,"evidence_count":1,"risk_count":0,"open_question_count":0},"evidence_count":3,"risk_count":0,"open_question_count":0,"diagnostic_count":1,"diagnostic_kinds":["search_evidence"]}}\n\n',
            'event: workspace_artifact_created\ndata: {"type":"workspace_artifact_created","id":"artifact-1","workspace_id":"workspace-1","run_id":"run-1","artifact_type":"search_evidence","title":"Search evidence","content_available":true,"preview_available":true,"format":"json","storage_kind":"inline","file_name":"evidence.json","file_size":128,"source":"agent_skill_result","status":"active","validation_status":"unchecked","validation_report_available":true,"metadata_available":true}\n\n',
            'event: agent_learning_record_created\ndata: {"type":"agent_learning_record_created","id":"learning-1","workspace_id":"workspace-1","run_id":"run-1","role":"ops","category":"decision","content_available":true,"confidence":0.88,"source_action":"coordination_save_memory","feedback_comment_available":true,"status":"active","metadata_available":true}\n\n',
            'event: approval_required\ndata: {"type":"approval_required","skill_id":"platform_update_price","request_id":"req-2","role":"ops","risk_level":"L3","reason":"needs approval","approval":{"id":"approval-1","workspace_id":"workspace-1","run_id":"run-1","request_role":"ops","skill_name":"platform_update_price","tool_call_id":"req-2","risk_level":"L3","status":"pending","reason":"needs approval","args_available":true,"result_available":true},"alert":{"id":"alert-1","workspace_id":"workspace-1","run_id":"run-1","alert_type":"agent_approval_required","severity":"warning","message":"approval required","role":"ops","status":"active","payload_available":true}}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const events: unknown[] = []
    await streamAgentSessionApi(
      { input: "continue the review" },
      {
        onAgentEvent: (event) => events.push(event),
        onDelta: vi.fn(),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(events).toEqual([
      expect.objectContaining({
        outputAvailable: true,
        outputSummary: expect.objectContaining({
          adapterStatus: "live",
          evidencePacket: expect.objectContaining({
            searchStrength: "standard",
            usableClaims: 1,
          }),
        }),
        type: "skill_result",
      }),
      expect.objectContaining({
        report: expect.objectContaining({
          contentAvailable: true,
          diagnosticKinds: ["search_evidence"],
          evidenceCount: 3,
          roleOutput: expect.objectContaining({
            findingCount: 2,
            summaryAvailable: true,
          }),
        }),
        type: "task_node_reported",
      }),
      expect.objectContaining({
        contentAvailable: true,
        metadataAvailable: true,
        type: "workspace_artifact_created",
        validationReportAvailable: true,
      }),
      expect.objectContaining({
        contentAvailable: true,
        feedbackCommentAvailable: true,
        metadataAvailable: true,
        type: "agent_learning_record_created",
      }),
      expect.objectContaining({
        approval: expect.objectContaining({
          argsAvailable: true,
          resultAvailable: true,
        }),
        alert: expect.objectContaining({
          payloadAvailable: true,
        }),
        riskLevel: "L3",
        type: "approval_required",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain('"output":')
    expect(serialized).not.toContain('"content":')
    expect(serialized).not.toContain('"metadata":')
    expect(serialized).not.toContain('"objectKey":')
    expect(serialized).not.toContain('"args":')
    expect(serialized).not.toContain('"payload":')
  })

  it("accepts safe completion and quality events without raw report or quality details", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: output\ndata: {"type":"output","content":"Final answer body"}\n\n',
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","workspace_id":"workspace-1","status":"succeeded","source":"agent_session","summary_available":true,"report_available":true,"metadata_available":true}\n\n',
            'event: workspace_artifact_created\ndata: {"type":"workspace_artifact_created","id":"artifact-1","workspace_id":"workspace-1","run_id":"run-1","artifact_type":"final_report","title":"Agent final report","content_available":true,"preview_available":true,"format":"markdown","storage_kind":"inline","source":"agent_session","status":"active","validation_status":"unchecked","validation_report_available":false,"metadata_available":true}\n\n',
            'event: quality_checked\ndata: {"type":"quality_checked","quality_check":{"id":"quality-1","workspace_id":"workspace-1","run_id":"run-1","artifact_id":"artifact-1","role":"agent","score":0.82,"passed":false,"source":"agent_session","dimension_count":2,"issue_count":1,"suggestion_count":1,"dimensions_available":true,"issues_available":true,"suggestions_available":true,"metadata_available":true},"trust_update":{"role":"agent","skill_id":null,"previous_score":0.5,"new_score":0.66,"previous_level":"moderate","new_level":"moderate","delta":0.16,"reason_available":true},"learning_record_count":1,"learning_records_available":true}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const events: unknown[] = []
    await streamAgentSessionApi(
      { input: "continue the review" },
      {
        onAgentEvent: (event) => events.push(event),
        onDelta: (content) => deltas.push(content),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
    expect(events).toEqual([
      expect.objectContaining({
        content: "Final answer body",
        type: "output",
      }),
      expect.objectContaining({
        metadataAvailable: true,
        reportAvailable: true,
        summaryAvailable: true,
        type: "workspace_run_completed",
      }),
      expect.objectContaining({
        artifactType: "final_report",
        contentAvailable: true,
        metadataAvailable: true,
        previewAvailable: true,
        type: "workspace_artifact_created",
      }),
      expect.objectContaining({
        learningRecordCount: 1,
        learningRecordsAvailable: true,
        qualityCheck: expect.objectContaining({
          dimensionCount: 2,
          dimensionsAvailable: true,
          issueCount: 1,
          issuesAvailable: true,
          metadataAvailable: true,
          suggestionCount: 1,
          suggestionsAvailable: true,
        }),
        trustUpdate: expect.objectContaining({
          newLevel: "moderate",
          newScore: 0.66,
          reasonAvailable: true,
        }),
        type: "quality_checked",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain('"report":')
    expect(serialized).not.toContain('"summary":')
    expect(serialized).not.toContain('"requirements":')
    expect(serialized).not.toContain('"dimensions":')
    expect(serialized).not.toContain('"issues":')
    expect(serialized).not.toContain('"suggestions":')
    expect(serialized).not.toContain('"learningRecords":')
    expect(serialized).not.toContain('"reason":')
  })

  it("keeps final output when post-stream quality persistence reports a safe error", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: output\ndata: {"type":"output","content":"Final answer body"}\n\n',
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","workspace_id":"workspace-1","status":"succeeded","source":"agent_session","summary_available":true,"report_available":true,"metadata_available":true}\n\n',
            'event: workspace_artifact_created\ndata: {"type":"workspace_artifact_created","id":"artifact-1","workspace_id":"workspace-1","run_id":"run-1","artifact_type":"final_report","title":"Agent final report","content_available":true,"preview_available":true,"format":"markdown","storage_kind":"inline","source":"agent_session","status":"active","validation_status":"unchecked","validation_report_available":false,"metadata_available":true}\n\n',
            'event: workspace_run_persistence_error\ndata: {"type":"workspace_run_persistence_error","action":"evaluate_agent_run","code":-12027,"message":"quality check persistence failed"}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const deltas: string[] = []
    const events: unknown[] = []
    await streamAgentSessionApi(
      { input: "continue the review" },
      {
        onAgentEvent: (event) => events.push(event),
        onDelta: (content) => deltas.push(content),
        onDone: vi.fn(),
        onError: (message) => {
          throw new Error(message)
        },
      }
    )

    expect(deltas).toEqual(["Final answer body"])
    expect(events).toEqual([
      expect.objectContaining({
        content: "Final answer body",
        type: "output",
      }),
      expect.objectContaining({
        reportAvailable: true,
        summaryAvailable: true,
        type: "workspace_run_completed",
      }),
      expect.objectContaining({
        artifactType: "final_report",
        contentAvailable: true,
        type: "workspace_artifact_created",
      }),
      expect.objectContaining({
        action: "evaluate_agent_run",
        code: -12027,
        message: "quality check persistence failed",
        type: "workspace_run_persistence_error",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain('"qualityCheck"')
    expect(serialized).not.toContain('"dimensions"')
    expect(serialized).not.toContain('"issues"')
    expect(serialized).not.toContain('"suggestions"')
  })
})

describe("previewAgentContextApi", () => {
  it("posts context preview requests and keeps safe memory record summaries", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          code: 0,
          data: {
            workspaceContext: {
              counts: {},
              readiness: "ready",
              sourceCount: 1,
            },
            workspacePack: {
              counts: {},
              itemCount: 0,
              omitted: [],
              omittedDetails: [],
              readiness: "ready",
              readableItemCount: 0,
              textChars: 0,
              warnings: [],
            },
            conversationHistory: null,
            conversationMemory: null,
            memoryRecords: [
              {
                kind: "conversation_fact",
                key: "memory_card:facts",
                tags: ["memory_card", "entity:product-1"],
                charCount: 86,
                truncated: false,
              },
            ],
            budget: { strategy: "preview" },
            includedRefs: ["conversation_file:file-1"],
            metadataOnlyRefs: [],
            omittedRefs: [],
            warnings: [],
          },
          message: "ok",
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }
      )
    )
    vi.stubGlobal("fetch", fetchMock)

    const preview = await previewAgentContextApi({
      contextRefs: ["conversation_file:file-1"],
      input: "continue the review",
      workspaceId: "workspace-1",
    })

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/app/agent/context-preview",
      expect.objectContaining({
        body: JSON.stringify({
          contextRefs: ["conversation_file:file-1"],
          input: "continue the review",
          workspaceId: "workspace-1",
        }),
        method: "POST",
      })
    )
    expect(preview.memoryRecords).toEqual([
      {
        kind: "conversation_fact",
        key: "memory_card:facts",
        tags: ["memory_card", "entity:product-1"],
        charCount: 86,
        truncated: false,
      },
    ])
  })

  it("maps preview memory records to the existing memory loaded event shape", () => {
    const event = buildMemoryLoadedEventFromContextPreview({
      memoryRecords: [
        {
          charCount: 86,
          key: "memory_card:facts",
          kind: "conversation_fact",
          tags: ["memory_card"],
          truncated: false,
        },
        {
          char_count: 12,
          key: "memory_card:decisions",
          kind: "conversation_fact",
          tags: ["memory_card"],
          truncated: true,
        },
      ],
    })

    expect(event).toEqual({
      type: "memory_loaded",
      count: 2,
      kinds: ["conversation_fact"],
      records: [
        {
          charCount: 86,
          key: "memory_card:facts",
          kind: "conversation_fact",
          tags: ["memory_card"],
          truncated: false,
        },
        {
          charCount: 12,
          key: "memory_card:decisions",
          kind: "conversation_fact",
          tags: ["memory_card"],
          truncated: true,
        },
      ],
    })
  })
})
