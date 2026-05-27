import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import type { TrainingFileItem } from "../types"
import { fileConversationUseState, fileDisplayName } from "./logic"

export interface TrainingFilesTurnSelectionSummary {
  hiddenNames: number
  selectedNames: string[]
  total: number
}

export function summarizeTrainingFilesTurnSelection({
  attachedObjectKeys,
  files,
  maxNames = 3,
  selectedSources,
}: {
  attachedObjectKeys: Set<string>
  files: TrainingFileItem[]
  maxNames?: number
  selectedSources: AgentChatSourceContextSelection[]
}): TrainingFilesTurnSelectionSummary {
  const selectedNames = files
    .filter(
      (file) =>
        fileConversationUseState({
          attachedObjectKeys,
          file,
          selectedSources,
        }) === "selectedForTurn"
    )
    .map(fileDisplayName)

  return {
    hiddenNames: Math.max(selectedNames.length - maxNames, 0),
    selectedNames: selectedNames.slice(0, maxNames),
    total: selectedNames.length,
  }
}
