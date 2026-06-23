import { describe, expect, it } from "vitest"
import type { AgentWorkerReport } from "@/api"
import {
  compactPreviewLines,
  isInternalRuntimeLine,
  readReportLines,
} from "./swarm-report-lines-view-model"

const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.swarm.reportLines.question") {
    return `question: ${String(params?.question)}`
  }
  if (key === "agentChat.swarm.reportLines.risk") {
    return `risk: ${String(params?.risk)}`
  }
  return key
}) as never

describe("swarm report lines view model", () => {
  it("reads camelCase role output without exposing internal lines", () => {
    const report = {
      content: "",
      evidence: [],
      graph_id: "graph-1",
      node_id: "creative",
      openQuestions: ["How should the title avoid overpromising?"],
      risks: ["Avoid unsupported benchmark claims."],
      role: "creative",
      roleOutput: {
        evidence: ["Compared the title against the available selling points."],
        findings: [
          "The current title has a clear benefit but lacks a concrete usage scenario.",
          "json template validation failed at $: missing key 'role_output'",
        ],
        openQuestions: [],
        recommendations: ["Add one scenario phrase before the main benefit."],
        risks: [],
        summary: "The title direction is usable after one specificity pass.",
      },
      status: "completed",
      task_id: "task-1",
      worker_id: "worker.creative",
    } satisfies AgentWorkerReport

    const lines = readReportLines(report, t)
    const visibleText = lines.join("\n")

    expect(visibleText).toContain("The title direction is usable")
    expect(visibleText).toContain("Add one scenario phrase")
    expect(visibleText).toContain("question: How should the title avoid overpromising?")
    expect(visibleText).not.toContain("json template validation")
    expect(visibleText).not.toContain("role_output")
  })

  it("filters internal runtime rules from process previews", () => {
    const title = textFromCodes([35, 32, 25968, 25454, 35786, 26029])
    const runtimeRule = textFromCodes([39564, 35777, 35268, 21017, 32, 49, 65306, 99, 111, 110, 116, 101, 110, 116, 32, 109, 117, 115, 116, 32, 98, 101, 32, 115, 112, 101, 99, 105, 102, 105, 99, 32, 97, 110, 100, 32, 110, 111, 110, 45, 101, 109, 112, 116, 121])
    const finding = textFromCodes([21457, 29616, 65306, 24050, 35745, 31639, 26333, 20809, 21040, 28857, 20987, 12289, 28857, 20987, 21040, 21152, 36141, 12289, 21152, 36141, 21040, 19979, 21333, 21644, 19979, 21333, 21040, 20184, 27454, 12290])
    const lines = compactPreviewLines([
      title,
      runtimeRule,
      "content must be specific and non-empty",
      "data role should provide at least two findings",
      "json template validation failed at $: missing key 'role_output'",
      finding,
    ])

    expect(lines).toEqual([
      title,
      finding,
    ])
  })

  it("classifies internal runtime lines", () => {
    expect(isInternalRuntimeLine("content must be specific and non-empty")).toBe(true)
    expect(isInternalRuntimeLine("worker reported failed status")).toBe(true)
    expect(isInternalRuntimeLine("structured role output is missing")).toBe(true)
    expect(isInternalRuntimeLine("llm provider error: upstream model service unavailable")).toBe(true)
    expect(
      isInternalRuntimeLine(textFromCodes([38656, 35201, 34917, 40784, 34892, 19994, 22522, 20934, 21518, 20877, 22797, 26680]))
    ).toBe(false)
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
