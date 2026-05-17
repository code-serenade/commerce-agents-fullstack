import { describe, expect, it, vi } from "vitest"
import { streamAgentSessionApi } from "./agent"

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
  it("uses workspace completion output as chat text when no delta is emitted", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        streamFromSse([
          'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","report":{"output":"Final answer body"},"summary":"Final answer body"}\n\n',
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

  it("deduplicates workspace run and final report artifact fallback output", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        new Response(
          streamFromSse([
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","report":{"output":"Final answer body"},"summary":"Final answer body"}\n\n',
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
            'event: workspace_run_completed\ndata: {"type":"workspace_run_completed","id":"run-1","status":"succeeded","report":{"output":"Final answer body"},"summary":"Final answer body"}\n\n',
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
})
