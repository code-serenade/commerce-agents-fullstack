import type { TFunction } from "i18next"
import type { AgentWorkerReport } from "@/api"
import { isUnsupportedBenchmarkClaim } from "../domain/agent-chat-answer-sanitizer"
import { isInternalRuntimeReason } from "./swarm-runtime-reason-view-model"

export function readReportDocumentLines(
  report: AgentWorkerReport | undefined,
  t: TFunction
) {
  if (!report) return []
  return compactPreviewLines([report.content, ...readReportLines(report, t)])
}

export function compactPreviewLines(lines: Array<string | undefined | null>) {
  const seen = new Set<string>()
  return lines
    .flatMap((line) => String(line ?? "").split(/\r?\n/))
    .map((line) => line.trim())
    .filter((line) => {
      if (
        !line ||
        seen.has(line) ||
        isUnsupportedBenchmarkClaim(line) ||
        isInternalRuntimeLine(line)
      ) {
        return false
      }
      seen.add(line)
      return true
    })
    .slice(0, 18)
}

export function readReportLines(
  report: AgentWorkerReport | undefined,
  t: TFunction
) {
  if (!report) return []
  const roleOutput = report.role_output ?? report.roleOutput
  return [
    roleOutput?.summary,
    ...(roleOutput?.findings ?? []),
    ...(roleOutput?.recommendations ?? []),
    ...(roleOutput?.evidence ?? []),
    ...(report.risks ?? []).map((risk) =>
      t("agentChat.swarm.reportLines.risk", { risk })
    ),
    ...readOpenQuestions(report).map((question) =>
      t("agentChat.swarm.reportLines.question", { question })
    ),
  ].flatMap((line) => {
    const value = line?.trim()
    if (!value) return []
    if (isUnsupportedBenchmarkClaim(value) || isInternalRuntimeLine(value)) {
      return []
    }
    return [value]
  })
}

export function isInternalRuntimeLine(value: string) {
  const text = value.trim()
  if (!text) return false
  if (isInternalRuntimeReason(text)) return true
  return [
    "content must be specific and non-empty",
    "data role should provide",
    "service role output lacks",
    "engineering role output lacks",
    "worker reported failed status",
    "structured role output is missing",
    "llm provider error",
    "upstream model service unavailable",
    "failure_kind",
    "worker_unavailable",
    "llm_provider_error",
    "llm access denied",
    "json template validation",
    "decode failed",
    "cognition failure",
    "raw=",
    "missing key",
  ].some((term) => text.toLowerCase().includes(term))
}

function readOpenQuestions(report: AgentWorkerReport) {
  return report.open_questions ?? report.openQuestions ?? []
}
