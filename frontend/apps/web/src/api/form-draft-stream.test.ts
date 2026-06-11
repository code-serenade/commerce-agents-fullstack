import { describe, expect, it, vi } from "vitest"
import { streamFormDraftApi } from "./form-draft-stream"

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

describe("streamFormDraftApi", () => {
  const safeError = "Form draft generation failed before form fill."
  const rawCampaignName = "raw-campaign-name"
  const rawProductName = "raw-product-name"

  it("returns the completed draft and emits safe process stages", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: form_draft_started\ndata: {"type":"form_draft_started","formType":"product","generationMode":"agent_llm","requiresUserReview":true}\n\n',
            `event: form_draft_input_preflighted\ndata: {"type":"form_draft_input_preflighted","formType":"product","generationMode":"agent_llm","directWrite":false,"requiresUserReview":true,"inputSummary":{"existingFieldCount":3,"userInputAvailable":true,"contextSummaryAvailable":true},"name":"${rawProductName}","prompt":"zh/form_drafts/product_form_draft.md","model":"hidden-model","baseUrl":"http://127.0.0.1:11434"}\n\n`,
            'event: form_draft_agent_started\ndata: {"type":"form_draft_agent_started","formType":"product","systemAgentId":"form_draft_generator","llm":{"used":true}}\n\n',
            'event: form_draft_output_validation_started\ndata: {"type":"form_draft_output_validation_started","formType":"product","draftQuality":{"invalidModelJsonPolicy":"reject_before_form_fill"}}\n\n',
            'event: form_draft_output_validation_passed\ndata: {"type":"form_draft_output_validation_passed","formType":"product","draftQuality":{"invalidModelJsonPolicy":"reject_before_form_fill","validated":true}}\n\n',
            'event: form_draft_review_ready\ndata: {"type":"form_draft_review_ready","formType":"product","generationMode":"agent_llm","directWrite":false,"requiresUserReview":true,"saveBoundary":{"directWrite":false,"requiresUserReview":true},"draft":{"name":"raw-draft"},"model":"hidden-model","prompt":"zh/form_drafts/product_form_draft.md"}\n\n',
            'event: form_draft_completed\ndata: {"type":"form_draft_completed","formType":"product","response":{"draft":{"name":"Agent product","status":"active"},"generationMode":"agent_llm","warnings":[],"agentContext":{"generationSource":"agent_llm","llm":{"used":true},"saveBoundary":{"requiresUserReview":true}}}}\n\n',
            "event: done\ndata: {}\n\n",
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const stages: string[] = []
    const result = await streamFormDraftApi({
      onProcess: (event) => stages.push(event.stage),
      request: {
        formType: "product",
        product: {
          generationMode: "agent_llm",
          name: "Agent product",
        },
      },
    })

    expect(result.draft.name).toBe("Agent product")
    expect(result.generationMode).toBe("agent_llm")
    expect(stages).toEqual([
      "form_draft_started",
      "form_draft_input_preflighted",
      "form_draft_agent_started",
      "form_draft_output_validation_started",
      "form_draft_output_validation_passed",
      "form_draft_review_ready",
    ])
  })

  it("emits the review-ready stage without raw draft internals", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: form_draft_review_ready\ndata: {"type":"form_draft_review_ready","formType":"product","generationMode":"agent_llm","directWrite":false,"requiresUserReview":true,"saveBoundary":{"directWrite":false,"requiresUserReview":true},"draft":{"name":"raw-draft"},"model":"hidden-model","prompt":"zh/form_drafts/product_form_draft.md"}\n\n',
            'event: form_draft_completed\ndata: {"type":"form_draft_completed","formType":"product","response":{"draft":{"name":"Agent product","status":"active"},"generationMode":"agent_llm","warnings":[],"agentContext":{"generationSource":"agent_llm","llm":{"used":true},"saveBoundary":{"requiresUserReview":true}}}}\n\n',
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const events: unknown[] = []
    const result = await streamFormDraftApi({
      onProcess: (event) => events.push(event),
      request: {
        formType: "product",
        product: {
          generationMode: "agent_llm",
          name: "Agent product",
        },
      },
    })

    expect(result.draft.name).toBe("Agent product")
    expect(events).toEqual([
      expect.objectContaining({
        agentContext: expect.objectContaining({
          directWrite: false,
          requiresUserReview: true,
          saveBoundary: {
            directWrite: false,
            requiresUserReview: true,
          },
        }),
        formType: "product",
        generationMode: "agent_llm",
        stage: "form_draft_review_ready",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain("raw-draft")
    expect(serialized).not.toContain("hidden-model")
    expect(serialized).not.toContain("product_form_draft")
  })

  it("normalizes the input preflight event to a safe summary", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            `event: form_draft_input_preflighted\ndata: {"type":"form_draft_input_preflighted","formType":"campaign","generationMode":"agent_llm","directWrite":false,"requiresUserReview":true,"inputSummary":{"existingFieldCount":4,"userInputAvailable":true,"contextSummaryAvailable":true},"name":"${rawCampaignName}","prompt":"zh/form_drafts/campaign_form_draft.md","model":"hidden-model","baseUrl":"http://127.0.0.1:11434"}\n\n`,
            'event: form_draft_completed\ndata: {"type":"form_draft_completed","formType":"campaign","response":{"draft":{"name":"Agent campaign","status":"draft"},"generationMode":"agent_llm","warnings":[],"agentContext":{"generationSource":"agent_llm","llm":{"used":true},"saveBoundary":{"requiresUserReview":true}}}}\n\n',
          ]),
          {
            status: 200,
            headers: { "Content-Type": "text/event-stream" },
          }
        )
      )
    )

    const events: unknown[] = []
    const result = await streamFormDraftApi({
      onProcess: (event) => events.push(event),
      request: {
        campaign: {
          generationMode: "agent_llm",
          name: "Agent campaign",
        },
        formType: "campaign",
      },
    })

    expect(result.draft.name).toBe("Agent campaign")
    expect(events).toEqual([
      expect.objectContaining({
        agentContext: expect.objectContaining({
          directWrite: false,
          inputSummary: {
            contextSummaryAvailable: true,
            existingFieldCount: 4,
            userInputAvailable: true,
          },
          requiresUserReview: true,
        }),
        formType: "campaign",
        generationMode: "agent_llm",
        stage: "form_draft_input_preflighted",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain(rawCampaignName)
    expect(serialized).not.toContain("hidden-model")
    expect(serialized).not.toContain("127.0.0.1")
    expect(serialized).not.toContain("campaign_form_draft")
  })

  it("fails when the stream ends without a completed draft", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(streamFromSse(["event: done\ndata: {}\n\n"]), {
          status: 200,
          headers: { "Content-Type": "text/event-stream" },
        })
      )
    )

    await expect(
      streamFormDraftApi({
        request: {
          campaign: {
            generationMode: "agent_llm",
            name: "Agent campaign",
          },
          formType: "campaign",
        },
      })
    ).rejects.toThrow(safeError)
  })

  it("emits a safe failed stage before rejecting the stream error", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: form_draft_started\ndata: {"type":"form_draft_started","formType":"product","generationMode":"agent_llm","requiresUserReview":true}\n\n',
            'event: form_draft_failed\ndata: {"type":"form_draft_failed","formType":"product","generationMode":"agent_llm","failureKind":"agent_run_failed","messageAvailable":true,"directWrite":false,"requiresUserReview":true,"draftQuality":{"invalidModelJsonPolicy":"reject_before_form_fill","validated":false},"saveBoundary":{"directWrite":false,"requiresUserReview":true},"model":"hidden-model","baseUrl":"http://127.0.0.1:11434","promptKey":"zh/form_drafts/product_form_draft.md"}\n\n',
            'event: error\ndata: {"message":"provider raw error model=hidden-model base_url=http://127.0.0.1:11434 prompt=zh/form_drafts/product_form_draft.md"}\n\n',
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
    await expect(
      streamFormDraftApi({
        onProcess: (event) => events.push(event),
        request: {
          formType: "product",
          product: {
            generationMode: "agent_llm",
            name: "Agent product",
          },
        },
      })
    ).rejects.toThrow(safeError)

    expect(events).toEqual([
      expect.objectContaining({
        formType: "product",
        generationMode: "agent_llm",
        stage: "form_draft_started",
      }),
      expect.objectContaining({
        agentContext: expect.objectContaining({
          directWrite: false,
          failureKind: "agent_run_failed",
          messageAvailable: true,
          requiresUserReview: true,
        }),
        failureKind: "agent_run_failed",
        formType: "product",
        generationMode: "agent_llm",
        messageAvailable: true,
        stage: "form_draft_failed",
      }),
    ])
    const serialized = JSON.stringify(events)
    expect(serialized).not.toContain("hidden-model")
    expect(serialized).not.toContain("127.0.0.1")
    expect(serialized).not.toContain("product_form_draft")
  })

  it("uses a safe error when the stream response itself fails", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          JSON.stringify({
            message:
              "provider raw error model=hidden-model base_url=http://127.0.0.1:11434 prompt=zh/form_drafts/product_form_draft.md",
          }),
          { status: 502 }
        )
      )
    )

    let thrown: unknown
    try {
      await streamFormDraftApi({
        request: {
          formType: "product",
          product: {
            generationMode: "agent_llm",
            name: "Agent product",
          },
        },
      })
    } catch (error) {
      thrown = error
    }

    expect(thrown).toBeInstanceOf(Error)
    const message = thrown instanceof Error ? thrown.message : String(thrown)
    expect(message).toBe(safeError)
    expect(message).not.toContain("hidden-model")
    expect(message).not.toContain("127.0.0.1")
    expect(message).not.toContain("product_form_draft")
  })
})
