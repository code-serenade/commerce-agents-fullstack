import { useTranslation } from "react-i18next"
import { getStudentPromptRecipeItems } from "../view-model/student-chat-entry-view-model"

export function StudentPromptRecipe() {
  const { t } = useTranslation()
  const promptRecipeItems = getStudentPromptRecipeItems(t)

  return (
    <div className="mt-3 rounded-lg border border-border bg-background">
      <div className="border-b border-border px-3 py-2">
        <p className="text-xs font-semibold text-foreground">
          {t("agentChat.thread.studentPromptRecipe.title")}
        </p>
        <p className="mt-1 text-[11px]/5 text-muted-foreground">
          {t("agentChat.thread.studentPromptRecipe.subtitle")}
        </p>
      </div>
      <div className="grid gap-1 p-3 sm:grid-cols-2">
        {promptRecipeItems.map((item) => (
          <div
            key={item.label}
            className="min-w-0 rounded-md border border-border bg-card px-3 py-2"
          >
            <p className="text-[11px] font-semibold text-foreground">
              {item.label}
            </p>
            <p className="mt-1 text-[11px]/5 text-muted-foreground">
              {item.value}
            </p>
          </div>
        ))}
      </div>
    </div>
  )
}
