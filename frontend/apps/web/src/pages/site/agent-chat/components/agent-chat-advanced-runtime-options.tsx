import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeChoice,
} from "../domain/agent-runtime-options"
import { choiceToSelectOption } from "../view-model/agent-chat-runtime-options-view-model"
import { RuntimeSelect } from "./agent-chat-runtime-shared"

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
    <div className="flex min-w-0 flex-wrap items-center gap-2 border-t border-border pt-3">
      <span className="shrink-0 text-xs font-semibold text-foreground">
        {t("agentChat.composer.runtime.advanced")}
      </span>
      <RuntimeSelect
        disabled={disabled}
        label={t("agentChat.composer.runtime.explanationDepth")}
        onValueChange={onLearningLevelChange}
        options={learningLevels.map(choiceToSelectOption)}
        value={runtimeSelection.learningLevel}
      />
    </div>
  )
}
