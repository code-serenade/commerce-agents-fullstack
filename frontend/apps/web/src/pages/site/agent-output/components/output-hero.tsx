import { Bot, ClipboardList, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Button } from "@workspace/ui-components/stable/button"

export interface OutputHeroProps {
  prompt: string
}

export function OutputHero({ prompt }: OutputHeroProps) {
  const { t } = useTranslation()

  return (
    <section className="grid gap-4 rounded-t-lg border-b border-(--document-intro-card-border) bg-(image:--document-intro-card-background) p-5 lg:grid-cols-[1fr_18rem]">
      <div className="min-w-0 space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary">{t("agentOutput.hero.badges.output")}</Badge>
          <Badge variant="outline">{t("agentOutput.hero.badges.category")}</Badge>
          <Badge variant="outline">{t("agentOutput.hero.badges.platform")}</Badge>
        </div>
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold text-(--app-text) sm:text-3xl">
            {t("agentOutput.hero.title")}
          </h1>
          <p className="max-w-3xl text-sm/6 text-(--app-muted-text)">
            {t("agentOutput.hero.description")}
          </p>
        </div>
        <div className="rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md">
          <div className="mb-2 flex items-center gap-2 text-sm font-medium text-(--app-muted-text)">
            <Sparkles className="size-4" />
            {t("agentOutput.hero.promptLabel")}
          </div>
          <p className="text-sm/6 text-(--app-text)">{prompt}</p>
        </div>
      </div>
      <aside className="flex flex-col justify-between gap-4 rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md">
        <div className="space-y-3">
          <div className="flex size-10 items-center justify-center rounded-md bg-(--app-active-bg) text-(--app-active-text)">
            <Bot className="size-5" />
          </div>
          <div>
            <p className="text-sm font-semibold text-(--app-text)">
              {t("agentOutput.hero.dispatchTitle")}
            </p>
            <p className="mt-1 text-sm/6 text-(--app-muted-text)">
              {t("agentOutput.hero.dispatchDescription")}
            </p>
          </div>
        </div>
        <Button variant="outline" size="sm">
          <ClipboardList className="size-4" />
          {t("agentOutput.hero.export")}
        </Button>
      </aside>
    </section>
  )
}
