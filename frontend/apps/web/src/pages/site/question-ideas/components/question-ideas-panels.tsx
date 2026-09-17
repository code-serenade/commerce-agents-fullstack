import { ArrowRight, ListChecks, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  StudentPromptRecipeItem,
  StudentTaskEntry,
} from "../../agent-chat/view-model/student-chat-entry-view-model"
import { DocumentSection } from "../../commerce/components/document-section"

export function QuestionIdeasTaskPanel({
  items,
  onSelectPrompt,
}: {
  items: StudentTaskEntry[]
  onSelectPrompt: (prompt: string) => void
}) {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("questionIdeas.tasks.title")}
      description={t("questionIdeas.tasks.description")}
      icon={<Sparkles className="size-4" />}
    >
      <div className="grid gap-3">
        {items.map((item) => (
          <button
            key={item.title}
            type="button"
            className="group min-w-0 rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 text-left shadow-(--document-card-shadow) backdrop-blur-md transition-colors hover:bg-(--shell-sidebar-item-hover-background)"
            onClick={() => onSelectPrompt(item.prompt)}
          >
            <span className="flex min-w-0 items-center gap-2 text-sm font-semibold text-foreground">
              <Sparkles className="size-4 shrink-0 text-primary" />
              <span className="min-w-0 flex-1">{item.title}</span>
              <ArrowRight className="size-4 shrink-0 text-muted-foreground transition group-hover:translate-x-0.5 group-hover:text-foreground" />
            </span>
            <span className="mt-1 block text-xs/5 text-muted-foreground">
              {item.detail}
            </span>
            <span className="mt-2 block rounded-md bg-muted/60 px-3 py-2 text-xs/5 text-foreground">
              {item.prompt}
            </span>
          </button>
        ))}
      </div>
    </DocumentSection>
  )
}

export function QuestionIdeasRecipePanel({
  items,
}: {
  items: StudentPromptRecipeItem[]
}) {
  const { t } = useTranslation()

  return (
    <DocumentSection
      title={t("questionIdeas.recipe.title")}
      description={t("questionIdeas.recipe.description")}
      icon={<ListChecks className="size-4" />}
    >
      <div className="grid max-w-5xl gap-3 md:grid-cols-2">
        {items.map((item, index) => (
          <div
            key={item.label}
            className="group grid min-h-24 min-w-0 grid-cols-[2.25rem_minmax(0,1fr)] gap-3 rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md transition-colors hover:border-primary/30 hover:bg-(--shell-sidebar-item-hover-background)"
          >
            <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-primary/10 text-xs font-semibold text-primary">
              {String(index + 1).padStart(2, "0")}
            </span>
            <span className="min-w-0">
              <span className="block text-sm/5 font-semibold text-foreground">
                {item.label}
              </span>
              <span className="mt-1.5 block text-sm/6 text-muted-foreground">
                {item.value}
              </span>
            </span>
          </div>
        ))}
      </div>
    </DocumentSection>
  )
}
