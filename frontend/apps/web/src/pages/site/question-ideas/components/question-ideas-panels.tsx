import { ArrowRight, ListChecks, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  StudentPromptRecipeItem,
  StudentTaskEntry,
} from "../../agent-chat/view-model/student-next-actions-view-model"
import {
  DocumentCard,
  DocumentSection,
} from "../../commerce/components/document-section"

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
      <div className="grid gap-3">
        {items.map((item) => (
          <DocumentCard key={item.label}>
            <p className="text-xs font-semibold text-foreground">
              {item.label}
            </p>
            <p className="mt-1 text-sm/6 text-muted-foreground">{item.value}</p>
          </DocumentCard>
        ))}
      </div>
    </DocumentSection>
  )
}
