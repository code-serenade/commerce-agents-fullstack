import { useTranslation } from "react-i18next"
import { getStudentTaskEntries } from "../view-model/student-chat-entry-view-model"
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
  const isPlain = surface === "plain"

  return (
    <div
      className={
        isPlain ? "mt-10 w-full space-y-7 lg:mt-12" : "mt-5 w-full space-y-5"
      }
    >
      {isPlain ? null : <AgentRoleClusterHomePanel surface={surface} />}
      <AgentChatWelcomePromptChips
        onSelectExamplePrompt={onSelectExamplePrompt}
        surface={surface}
        taskEntries={taskEntries}
      />
      {isPlain ? <AgentRoleClusterHomePanel surface={surface} /> : null}
    </div>
  )
}

export function AgentChatWelcomePromptChips({
  onSelectExamplePrompt,
  surface = "plain",
  taskEntries,
}: {
  onSelectExamplePrompt?: (prompt: string) => void
  surface?: "card" | "plain"
  taskEntries?: ReturnType<typeof getStudentTaskEntries>
}) {
  const { t } = useTranslation()
  const entries = taskEntries ?? getStudentTaskEntries(t)
  const isPlain = surface === "plain"

  return (
    <div>
      {isPlain ? null : (
        <div className="min-w-0">
          <p className="text-base font-bold text-foreground">
            {t("agentChat.thread.studentTaskEntries.title")}
          </p>
        </div>
      )}
      <div
        className={
          isPlain
            ? "mx-auto flex max-w-[880px] flex-wrap justify-center gap-x-2.5 gap-y-3"
            : "mt-3 flex flex-wrap gap-2"
        }
      >
        {entries.map((entry) => (
          <button
            key={entry.prompt}
            type="button"
            className={
              isPlain
                ? "min-h-10 cursor-pointer rounded-full border border-[#d7e3f2] bg-[linear-gradient(180deg,#fbfdff_0%,#f3f8ff_100%)] px-5 py-2 text-center text-[13px]/5 font-medium text-[#596c80] shadow-[0_1px_2px_rgb(56_88_132/0.06),inset_0_1px_0_rgb(255_255_255/0.95)] transition-colors hover:border-[#a9c8f3] hover:bg-white hover:text-[#596c80] dark:border-[#2d394a] dark:bg-[linear-gradient(180deg,rgb(20_28_39/0.74)_0%,rgb(12_19_30/0.82)_100%)] dark:text-[#c7d0dd] dark:shadow-[0_1px_2px_rgb(0_0_0/0.22),inset_0_1px_0_rgb(255_255_255/0.06)] dark:hover:border-[#46556c] dark:hover:bg-[linear-gradient(180deg,rgb(25_34_47/0.8)_0%,rgb(14_22_34/0.88)_100%)] dark:hover:text-[#c7d0dd]"
                : "cursor-pointer rounded-xl bg-(--agent-chat-accent-soft) px-4 py-2 text-left text-sm/5 text-foreground transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-primary"
            }
            onClick={() => onSelectExamplePrompt?.(entry.prompt)}
          >
            {entry.title}
          </button>
        ))}
      </div>
      <p
        className={
          isPlain
            ? "mx-auto mt-5 max-w-[880px] text-center text-xs/5 text-muted-foreground"
            : "mt-4 text-xs/5 text-muted-foreground"
        }
      >
        {t("agentChat.thread.learningHint")}
      </p>
    </div>
  )
}

export function AgentChatWelcomeRolePanel({
  surface = "plain",
}: {
  surface?: "card" | "plain"
}) {
  return (
    <div className={surface === "plain" ? "mt-9 w-full lg:mt-10" : ""}>
      <AgentRoleClusterHomePanel surface={surface} />
    </div>
  )
}
