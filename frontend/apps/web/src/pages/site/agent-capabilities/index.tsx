import { Link } from "react-router"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../commerce/components/commerce-ui"
import {
  AgentCapabilityBoundaryPanel,
  AgentCapabilityCorePanel,
  AgentCapabilityFlowPanel,
  AgentCapabilityLanePanel,
} from "./components/agent-capability-panels"

export default function AgentCapabilitiesPage() {
  const { t } = useTranslation()
  const capabilities = t("agentCapabilities.capabilities.items", {
    returnObjects: true,
  }) as Array<{ title: string; description: string }>
  const lanes = t("agentCapabilities.lanes.items", {
    returnObjects: true,
  }) as Array<{ title: string; description: string }>
  const flow = t("agentCapabilities.flow.items", {
    returnObjects: true,
  }) as string[]

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("agentCapabilities.header.eyebrow")}
        title={t("agentCapabilities.header.title")}
        description={t("agentCapabilities.header.description")}
        actions={
          <Link to="/commerce">
            <Button size="sm" variant="outline">
              {t("agentCapabilities.header.action")}
            </Button>
          </Link>
        }
      />
      <CommerceBody>
        <InlineAlert
          variant="success"
          title={t("agentCapabilities.principle.title")}
          description={t("agentCapabilities.principle.description")}
        />

        <div className="grid gap-4 lg:grid-cols-[1.05fr_0.95fr]">
          <AgentCapabilityCorePanel items={capabilities} />
          <AgentCapabilityFlowPanel items={flow} />
        </div>

        <AgentCapabilityLanePanel items={lanes} />
        <AgentCapabilityBoundaryPanel />
      </CommerceBody>
    </CommerceSurface>
  )
}
