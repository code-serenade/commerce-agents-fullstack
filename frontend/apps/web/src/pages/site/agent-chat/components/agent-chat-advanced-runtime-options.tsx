import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeChoice,
} from "../domain/agent-runtime-options"
import { choiceToOption } from "../view-model/agent-chat-runtime-options-view-model"
import { OptionSegment } from "./agent-chat-runtime-shared"

export function AdvancedRuntimeOptions({
  disabled,
  learningLevels,
  onLearningLevelChange,
  runtimeSelection,
}: {
  disabled: boolean
  learningLevels: AgentRuntimeChoice[]
  onLearningLevelChange: (value: string) => void
  runtimeSelection: AgentChatRuntimeSelection
}) {
  const { t } = useTranslation()

  return (
    <OptionSegment
      disabled={disabled}
      label={t("agentChat.composer.runtime.explanationDepth")}
      onValueChange={onLearningLevelChange}
      options={learningLevels.map((choice) =>
        choiceToOption(choice, t, "learningLevels")
      )}
      value={runtimeSelection.learningLevel}
    />
  )
}
