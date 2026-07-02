import { useCallback } from "react"
import { trackProductEvent } from "@/analytics/product-analytics"
import type { AgentProcessTarget } from "./agent-process-visibility-view-model"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
  AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatAttachmentItem } from "../types"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"

export function useAgentChatAnalytics(input: {
  attachments: AgentChatAttachmentItem[]
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  strategyId: AgentChatStrategyId
}) {
  const {
    attachments,
    selectedCampaigns,
    selectedProducts,
    selectedSources,
    strategyId,
  } = input

  const trackQuestionSubmit = useCallback(() => {
    trackProductEvent({
      eventType: "agent_question_submit",
      featureKey: "agent_chat.submit",
      metadata: {
        hasAttachments: attachments.length > 0,
        selectedSourceCount: selectedSources.length,
        selectedProductCount: selectedProducts.length,
        selectedCampaignCount: selectedCampaigns.length,
        strategyId,
      },
    })
  }, [
    attachments.length,
    selectedCampaigns.length,
    selectedProducts.length,
    selectedSources.length,
    strategyId,
  ])

  const trackProcessPanelOpen = useCallback((target: AgentProcessTarget) => {
    trackProductEvent({
      eventType: "process_panel_open",
      featureKey: "agent_chat.process_panel.open",
      metadata: {
        messageId: target.messageId,
        task: target.task,
      },
    })
  }, [])

  const trackFileUpload = useCallback((files: File[]) => {
    trackProductEvent({
      eventType: "file_upload",
      featureKey: "agent_chat.attachments.upload",
      metadata: {
        fileCount: files.length,
        totalSize: files.reduce((total, file) => total + file.size, 0),
      },
    })
  }, [])

  return {
    trackFileUpload,
    trackProcessPanelOpen,
    trackQuestionSubmit,
  }
}
