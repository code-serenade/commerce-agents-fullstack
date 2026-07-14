import { GraduationCap } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Button } from "@workspace/ui-components"
import { useOptionalAgentChatContext } from "../agent-chat/context/use-agent-chat-context"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../commerce/components/commerce-ui"
import { DocumentIntroCard } from "../commerce/components/document-section"
import {
  AgentCapabilityBoundaryPanel,
  AgentCapabilityCorePanel,
  AgentCapabilityFlowPanel,
  AgentCapabilityLanePanel,
  AgentCapabilityQaPanel,
} from "./components/agent-capability-panels"

const capabilityEntryLinks = [
  "/commerce",
  "/products",
  "/campaigns",
  "/operations",
  "/operations",
  "/workspaces",
  "/agent-output",
  "/approvals",
]

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
  const resetConversation = () => {
    chat?.startNewConversation()
  }
  const startNewConversation = () => {
    resetConversation()
    navigate("/commerce")
  }
  const capabilityEntryHandlers = [resetConversation]

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
        <DocumentIntroCard
          title={t("agentCapabilities.principle.title")}
          description={t("agentCapabilities.principle.description")}
          icon={<GraduationCap className="size-4" />}
        />

        <div className="space-y-6 [&>section+section]:border-t [&>section+section]:border-(--app-border)/60 [&>section+section]:pt-6">
          <AgentCapabilityCorePanel
            itemLinks={capabilityEntryLinks}
            itemSelectHandlers={capabilityEntryHandlers}
            items={capabilities}
          />
          <AgentCapabilityFlowPanel items={flow} />
          <AgentCapabilityCorePanel
            descriptionKey="agentCapabilities.modes.description"
            items={modes}
            titleKey="agentCapabilities.modes.title"
          />
          <AgentCapabilityQaPanel
            items={qa.map((item) => ({
              title: item.question,
              description: item.answer,
            }))}
          />
          <AgentCapabilityLanePanel items={lanes} />
          <AgentCapabilityBoundaryPanel />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
