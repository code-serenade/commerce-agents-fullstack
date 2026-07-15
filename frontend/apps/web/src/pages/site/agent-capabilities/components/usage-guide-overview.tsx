import { useState } from "react"
import { GraduationCap, Route, Sparkles } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Tabs, TabsList, TabsTrigger } from "@workspace/ui-components"
import { DocumentIntroCard, DocumentSection } from "../../commerce/components/document-section"
import { HomeStudentPathBackdrop } from "../../home/components/home-student-path"
import {
  homeStepCardStyle,
  homeStudentPathImageSources,
} from "../../home/components/home-student-path-presentation"
import { getUsageGuideContent } from "../usage-guide-data"

const usageGuideFlowImages = [
  homeStudentPathImageSources.products,
  homeStudentPathImageSources.assistant,
  homeStudentPathImageSources.campaigns,
  homeStudentPathImageSources.tasks,
  homeStudentPathImageSources.reports,
]

export function UsageGuideOverview() {
  const { t } = useTranslation()
  const content = getUsageGuideContent(t)
  const [activeTask, setActiveTask] = useState(content.tasks.items[0]?.id ?? "campaign")
  const selectedTask =
    content.tasks.items.find((task) => task.id === activeTask) ??
    content.tasks.items[0]

  return (
    <>
      <DocumentIntroCard
        title={content.intro.title}
        description={content.intro.description}
        icon={<GraduationCap />}
      />

      <DocumentSection
        title={content.flow.title}
        description={content.flow.description}
        icon={<Route className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <div className="mx-auto w-full max-w-[1120px] lg:w-[calc(100%+1rem)]">
          <div className="relative px-0 lg:px-2">
            <HomeStudentPathBackdrop showCornerDecoration={false} />
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5 lg:gap-9">
              {content.flow.items.map((item, index) => (
                <article
                  key={item.title}
                  className="relative z-10 flex min-h-[150px] min-w-0 flex-row overflow-hidden rounded-lg border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(180deg,color-mix(in_oklch,var(--background)_98%,var(--primary)_2%)_0%,color-mix(in_oklch,var(--background)_94%,var(--primary)_6%)_54%,color-mix(in_oklch,var(--background)_90%,var(--primary)_10%)_100%)] text-left shadow-[0_22px_52px_color-mix(in_oklch,var(--primary)_9%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl sm:min-h-[260px] sm:flex-col"
                  style={homeStepCardStyle}
                >
                  <span className="pointer-events-none absolute top-2.5 left-2.5 z-10 text-xl leading-none font-semibold text-(--app-muted-text)/58 italic sm:text-2xl">
                    {String(index + 1).padStart(2, "0")}
                  </span>
                  <span className="flex w-[42%] max-w-36 shrink-0 items-center justify-center overflow-hidden pt-5 sm:block sm:h-[126px] sm:w-full sm:max-w-none sm:pt-0">
                    <img
                      src={usageGuideFlowImages[index]}
                      alt=""
                      aria-hidden="true"
                      className="size-full max-h-30 object-contain p-3 sm:max-h-none sm:pb-0"
                      loading="lazy"
                    />
                  </span>
                  <span className="flex min-w-0 flex-1 flex-col px-3.5 pt-8 pb-3.5 sm:pt-3">
                    <span className="block text-xs font-semibold text-(--app-text)">
                      {item.title}
                    </span>
                    <span className="mt-1.5 block min-h-10 text-[11px]/5 text-(--app-muted-text)">
                      {item.detail}
                    </span>
                    <span className="mt-auto block pt-2 text-[11px] font-semibold text-primary">
                      {item.stage}
                    </span>
                  </span>
                </article>
              ))}
            </div>
          </div>
        </div>
      </DocumentSection>

      <DocumentSection
        title={content.tasks.title}
        description={content.tasks.description}
        icon={<Sparkles className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <Tabs value={activeTask} onValueChange={setActiveTask}>
          <div className="min-h-64 md:min-h-20">
            <TabsList className="grid h-auto w-full grid-cols-1 gap-2 bg-transparent p-0 md:grid-cols-3">
              {content.tasks.items.map((task) => (
                <TabsTrigger
                  key={task.id}
                  value={task.id}
                  className="h-auto min-w-0 justify-start gap-3 whitespace-normal rounded-lg border border-(--app-border) bg-(--app-bg) p-2 text-left data-[state=active]:border-primary data-[state=active]:bg-primary/10"
                >
                  <img
                    src={task.cover}
                    alt=""
                    className="aspect-4/3 w-20 shrink-0 rounded-md object-cover"
                  />
                  <span className="min-w-0">
                    <strong className="block text-sm/5 text-foreground">
                      {task.title}
                    </strong>
                    <small className="mt-0.5 block text-xs/5 text-muted-foreground">
                      {task.shortDescription}
                    </small>
                  </span>
                </TabsTrigger>
              ))}
            </TabsList>
          </div>

          {selectedTask ? (
            <div className="mt-3 grid min-w-0 gap-3 rounded-lg border border-primary/20 bg-primary/5 p-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
              <div className="min-w-0">
                <h3 className="text-base font-semibold text-foreground">
                  {selectedTask.title}
                </h3>
                <p className="mt-1 text-sm/6 text-muted-foreground">
                  {selectedTask.description}
                </p>
              </div>
              <div className="min-w-0 border-t border-primary/15 pt-3 sm:max-w-72 sm:border-t-0 sm:border-l sm:pt-0 sm:pl-4">
                <p className="text-xs text-muted-foreground">
                  {content.tasks.resultLabel}
                </p>
                <p className="mt-1 text-sm font-semibold text-foreground">
                  {selectedTask.result}
                </p>
              </div>
            </div>
          ) : null}
        </Tabs>
      </DocumentSection>
    </>
  )
}
