import { useEffect } from "react"
import { trackProductEvent } from "@/analytics/product-analytics"
import { OutputBackendDeliverables } from "./components/output-backend-deliverables"
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
    <div className="flex size-full min-h-0 flex-1 bg-white dark:bg-background">
      <OutputBackendDeliverables backend={page.backend} />
    </div>
  )
}
