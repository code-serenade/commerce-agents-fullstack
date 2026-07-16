import { Bot, ClipboardList, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Button } from "@workspace/ui-components/stable/button"
import type { AgentOutputReport } from "../types"

export interface OutputHeroProps {
  report: AgentOutputReport
}

export function OutputHero({ report }: OutputHeroProps) {
  const { t } = useTranslation()

  const downloadReport = () => {
    if (!report.content) return
    const url = URL.createObjectURL(
      new Blob([report.content], { type: "text/plain;charset=utf-8" })
    )
    const anchor = document.createElement("a")
    anchor.href = url
    anchor.download = report.fileName
    anchor.click()
    URL.revokeObjectURL(url)
  }

  return (
    <section className="grid gap-4 rounded-t-lg border-b border-(--document-intro-card-border) bg-(image:--document-intro-card-background) p-5 lg:grid-cols-[1fr_18rem]">
      <div className="min-w-0 space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary">{t("agentOutput.structured.badge")}</Badge>
          {report.badges.map((badge) => (
            <Badge key={badge} variant="outline">
              {badge}
            </Badge>
          ))}
        </div>
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold text-(--app-text) sm:text-3xl">
            {report.title}
          </h1>
          <p className="max-w-3xl text-sm/6 text-(--app-muted-text)">
            {report.description ||
              t("agentOutput.structured.defaultDescription")}
          </p>
        </div>
        {report.prompt ? (
          <div className="rounded-lg border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md">
            <div className="mb-2 flex items-center gap-2 text-sm font-medium text-(--app-muted-text)">
              <Sparkles className="size-4" />
              {t("agentOutput.hero.promptLabel")}
            </div>
            <p className="text-sm/6 text-(--app-text)">{report.prompt}</p>
          </div>
        ) : null}
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
              {report.collaboration ||
                t("agentOutput.structured.defaultCollaboration")}
            </p>
          </div>
        </div>
        <Button
          variant="outline"
          size="sm"
          disabled={!report.content}
          onClick={downloadReport}
        >
          <ClipboardList className="size-4" />
          {t("agentOutput.hero.export")}
        </Button>
      </aside>
    </section>
  )
}
