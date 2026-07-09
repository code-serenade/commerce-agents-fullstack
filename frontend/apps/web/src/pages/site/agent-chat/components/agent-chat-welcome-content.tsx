import { useTranslation } from "react-i18next"
import { getStudentTaskEntries } from "../view-model/student-next-actions-view-model"
import { AgentRoleClusterHomePanel } from "./agent-role-cluster-home-panel"

export function AgentChatWelcomeContent({
  onSelectExamplePrompt,
  surface = "plain",
}: {
  onSelectExamplePrompt?: (prompt: string) => void
  surface?: "card" | "plain"
}) {
  const { t } = useTranslation()
  const taskEntries = getStudentTaskEntries(t)

  return (
    <div className="mt-5 w-full space-y-5">
      <AgentRoleClusterHomePanel surface={surface} />
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
              className="cursor-pointer rounded-xl bg-(--agent-chat-accent-soft) px-4 py-2 text-left text-sm/5 text-foreground transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-primary"
              onClick={() => onSelectExamplePrompt?.(entry.prompt)}
            >
              {entry.prompt}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
