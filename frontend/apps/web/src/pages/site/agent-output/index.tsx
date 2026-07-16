import { useEffect } from "react"
import { trackProductEvent } from "@/analytics/product-analytics"
import { OutputHero } from "./components/output-hero"
import { OutputBackendDeliverables } from "./components/output-backend-deliverables"
import { OutputMetrics } from "./components/output-metrics"
import { OutputPlan } from "./components/output-plan"
import { OutputSections } from "./components/output-sections"
import { OutputStructuredReport } from "./components/output-structured-report"
import { useAgentOutputPage } from "./use-agent-output-page"

export default function AgentOutputPage() {
  const page = useAgentOutputPage()

  useEffect(() => {
    trackProductEvent({
      eventType: "report_view",
      featureKey: "agent_output.view",
    })
  }, [])

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-5">
      <OutputBackendDeliverables backend={page.backend} />
      <OutputStructuredReport report={page.report}>
        {page.report ? (
          <>
            <OutputHero report={page.report} />
            <div className="space-y-5 p-5">
              <OutputMetrics metrics={page.report.metrics} />
              <OutputPlan
                actions={page.report.actions}
                steps={page.report.steps}
              />
              <OutputSections sections={page.report.sections} />
            </div>
          </>
        ) : null}
      </OutputStructuredReport>
    </div>
  )
}
