import { OutputHero } from "./components/output-hero"
import { OutputBackendDeliverables } from "./components/output-backend-deliverables"
import { OutputMetrics } from "./components/output-metrics"
import { OutputPlan } from "./components/output-plan"
import { OutputSections } from "./components/output-sections"
import { useAgentOutputPage } from "./use-agent-output-page"

export default function AgentOutputPage() {
  const page = useAgentOutputPage()

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-5">
      <OutputBackendDeliverables backend={page.backend} />
      <div className="overflow-hidden rounded-lg border border-(--app-border) bg-(--app-bg)">
        <OutputHero prompt={page.prompt} />
        <div className="space-y-5 p-5">
          <OutputMetrics metrics={page.metrics} />
          <OutputPlan actions={page.actions} steps={page.steps} />
          <OutputSections sections={page.sections} />
        </div>
      </div>
    </div>
  )
}
