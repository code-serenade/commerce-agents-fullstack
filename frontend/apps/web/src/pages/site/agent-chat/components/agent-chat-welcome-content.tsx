import { useTranslation } from "react-i18next"
import { getStudentTaskEntries } from "../view-model/student-next-actions-view-model"

export function AgentChatWelcomeContent({
  onSelectExamplePrompt,
}: {
  onSelectExamplePrompt?: (prompt: string) => void
}) {
  const { t } = useTranslation()
  const taskEntries = getStudentTaskEntries(t)

  return (
    <div className="mt-5 w-full space-y-5">
      <div>
        <div className="min-w-0">
          <p className="text-base font-bold text-foreground">
            {t("agentChat.thread.studentTaskEntries.title")}
          </p>
        </div>
        <div className="mt-3 flex flex-wrap gap-2">
          {taskEntries.map((entry) => (
            <button
              key={entry.prompt}
              type="button"
              className="cursor-pointer rounded-xl bg-(--agent-chat-suggestion-background) px-4 py-2 text-left text-sm/5 text-(--agent-chat-suggestion-foreground) transition-colors hover:bg-(--agent-chat-suggestion-hover-background) hover:text-(--agent-chat-suggestion-foreground)"
              onClick={() => onSelectExamplePrompt?.(entry.prompt)}
            >
              {entry.title}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
