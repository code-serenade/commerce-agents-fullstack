import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Button } from "@workspace/ui-components"
import { useAgentChatContext } from "../agent-chat/context/use-agent-chat-context"
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
  const chat = useAgentChatContext()
  const navigate = useNavigate()
  const capabilities = t("agentCapabilities.capabilities.items", {
    returnObjects: true,
  }) as Array<{ title: string; description: string }>
  const lanes = t("agentCapabilities.lanes.items", {
    returnObjects: true,
  }) as Array<{ title: string; description: string }>
  const flow = t("agentCapabilities.flow.items", {
    returnObjects: true,
  }) as string[]
  const modes = t("agentCapabilities.modes.items", {
    returnObjects: true,
  }) as Array<{ title: string; description: string }>
  const qa = t("agentCapabilities.qa.items", {
    returnObjects: true,
  }) as Array<{ question: string; answer: string }>
  const startNewConversation = () => {
    chat.startNewConversation()
    navigate("/commerce")
  }

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("agentCapabilities.header.eyebrow")}
        title={t("agentCapabilities.header.title")}
        description={t("agentCapabilities.header.description")}
        actions={
          <Button size="sm" variant="outline" onClick={startNewConversation}>
            {t("agentCapabilities.header.action")}
          </Button>
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

        <div className="grid gap-4 lg:grid-cols-[0.95fr_1.05fr]">
          <AgentCapabilityCorePanel
            descriptionKey="agentCapabilities.modes.description"
            items={modes}
            titleKey="agentCapabilities.modes.title"
          />
          <AgentCapabilityCorePanel
            descriptionKey="agentCapabilities.qa.description"
            items={qa.map((item) => ({
              title: item.question,
              description: item.answer,
            }))}
            titleKey="agentCapabilities.qa.title"
          />
        </div>

        <AgentCapabilityLanePanel items={lanes} />
        <AgentCapabilityBoundaryPanel />
      </CommerceBody>
    </CommerceSurface>
  )
}
