import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import type { AgentOutputReport } from "../types"
import { EmptyState } from "./output-backend-deliverable-blocks"

export function OutputStructuredReport({
  children,
  report,
}: {
  children: ReactNode
  report: AgentOutputReport | null
}) {
  if (!report) {
    return <StructuredReportEmptyState />
  }

  return (
    <section className="overflow-visible rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) shadow-(--document-card-shadow) backdrop-blur-md">
      {children}
    </section>
  )
}

function StructuredReportEmptyState() {
  const { t } = useTranslation()

  return (
    <section className="rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-5 shadow-(--document-card-shadow) backdrop-blur-md">
      <EmptyState
        title={t("agentOutput.structured.emptyTitle")}
        detail={t("agentOutput.structured.emptyDescription")}
      />
    </section>
  )
}
