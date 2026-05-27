import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import type { TrainingFileItem } from "../types"
import {
  fileConversationUseState,
  type FileConversationUseState,
} from "./logic"

export type TrainingFileUsageFilter = "all" | FileConversationUseState

export function filterTrainingFilesByUsage({
  attachedObjectKeys,
  files,
  selectedSources,
  usage,
}: {
  attachedObjectKeys: Set<string>
  files: TrainingFileItem[]
  selectedSources: AgentChatSourceContextSelection[]
  usage: TrainingFileUsageFilter
}) {
  if (usage === "all") return files

  return files.filter(
    (file) =>
      fileConversationUseState({
        attachedObjectKeys,
        file,
        selectedSources,
      }) === usage
  )
}
