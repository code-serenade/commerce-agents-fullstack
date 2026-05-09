import type { TFunction } from "i18next"
import type { AgentWorkerReport } from "@/api"

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
      if (!line || seen.has(line)) return false
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
  const roleOutput = report.role_output
  return [
    roleOutput?.summary,
    ...(roleOutput?.findings ?? []),
    ...(roleOutput?.recommendations ?? []),
    ...(roleOutput?.evidence ?? []),
    ...(report.risks ?? []).map((risk) =>
      t("agentChat.swarm.reportLines.risk", { risk })
    ),
    ...(report.open_questions ?? []).map((question) =>
      t("agentChat.swarm.reportLines.question", { question })
    ),
  ].filter((line): line is string => Boolean(line))
}
