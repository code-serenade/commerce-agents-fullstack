import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Button } from "@workspace/ui-components"
import { useOptionalAgentChatContext } from "../agent-chat/context/use-agent-chat-context"
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
  const chat = useOptionalAgentChatContext()
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
    chat?.startNewConversation()
    navigate("/commerce")
  }

  return (
    <CommerceSurface>
      <CommercePageHeader
        className="min-[1200px]:px-8"
        eyebrow={t("agentCapabilities.header.eyebrow")}
        title={t("agentCapabilities.header.title")}
        description={t("agentCapabilities.header.description")}
        actions={
          <Button size="sm" variant="outline" onClick={startNewConversation}>
            {t("agentCapabilities.header.action")}
          </Button>
        }
      />
      <CommerceBody className="min-[1200px]:px-8">
        <InlineAlert
          variant="success"
          title={t("agentCapabilities.principle.title")}
          description={t("agentCapabilities.principle.description")}
        />

        <div className="space-y-8">
          <AgentCapabilityCorePanel items={capabilities} />
          <AgentCapabilityFlowPanel items={flow} />
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
          <AgentCapabilityLanePanel items={lanes} />
          <AgentCapabilityBoundaryPanel />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
