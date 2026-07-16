import {
  ArrowRight,
  CheckCircle2,
  CircleHelp,
  Route,
  Settings2,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentSection } from "../../commerce/components/document-section"
import { homeStudentPathImageSources } from "../../home/components/home-student-path-presentation"
import flowCollaborationIllustration from "../assets/usage-guide/flow-collaboration.webp"
import flowPrepareIllustration from "../assets/usage-guide/flow-prepare.webp"
import { getUsageGuideContent } from "../usage-guide-data"

const usageGuideFlowImages = [
  flowPrepareIllustration,
  homeStudentPathImageSources.assistant,
  flowCollaborationIllustration,
  homeStudentPathImageSources.tasks,
  homeStudentPathImageSources.reports,
]

export function UsageGuideOverview() {
  const { t } = useTranslation()
  const content = getUsageGuideContent(t)

  return (
    <>
      <DocumentSection
        title={content.intro.title}
        description={content.intro.description}
        icon={<CircleHelp className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <div className="overflow-hidden rounded-lg border border-(--app-border) bg-background">
          <div className="hidden grid-cols-[minmax(0,0.9fr)_2rem_minmax(0,1.25fr)_2rem_minmax(0,1fr)] items-center border-b border-(--app-border) bg-muted/15 px-4 py-2.5 text-xs font-semibold text-muted-foreground md:grid">
            <span>{content.intro.columns.problem}</span>
            <span aria-hidden="true" />
            <span>{content.intro.columns.action}</span>
            <span aria-hidden="true" />
            <span>{content.intro.columns.result}</span>
          </div>

          <ol className="divide-y divide-(--app-border)">
            {content.intro.items.map((item, index) => (
              <li
                key={item.problem}
                className="grid min-w-0 gap-3 px-4 py-4 md:grid-cols-[minmax(0,0.9fr)_2rem_minmax(0,1.25fr)_2rem_minmax(0,1fr)] md:items-center md:gap-0"
              >
                <div className="flex min-w-0 items-start gap-3">
                  <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold text-foreground">
                    {index + 1}
                  </span>
                  <div className="min-w-0">
                    <p className="text-xs font-semibold text-muted-foreground md:hidden">
                      {content.intro.columns.problem}
                    </p>
                    <p className="mt-1 text-sm/6 font-semibold text-foreground md:mt-0">
                      {item.problem}
                    </p>
                  </div>
                </div>

                <ArrowRight className="hidden size-4 justify-self-center text-muted-foreground/60 md:block" />

                <div className="min-w-0 border-l-2 border-(--app-border) pl-4 md:border-l-0 md:pl-0">
                  <p className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground md:hidden">
                    <Settings2 className="size-3.5" />
                    {content.intro.columns.action}
                  </p>
                  <p className="mt-1 text-sm/6 text-foreground md:mt-0">
                    {item.action}
                  </p>
                </div>

                <ArrowRight className="hidden size-4 justify-self-center text-muted-foreground/60 md:block" />

                <div className="min-w-0 border-l-2 border-primary/30 pl-4 md:border-l-0 md:pl-0">
                  <p className="flex items-center gap-1.5 text-xs font-semibold text-primary md:hidden">
                    <CheckCircle2 className="size-3.5" />
                    {content.intro.columns.result}
                  </p>
                  <p className="mt-1 text-sm/6 font-medium text-foreground md:mt-0">
                    {item.result}
                  </p>
                </div>
              </li>
            ))}
          </ol>
        </div>
      </DocumentSection>

      <DocumentSection
        title={content.flow.title}
        description={content.flow.description}
        icon={<Route className="size-4" />}
        contentClassName="pl-0 sm:pl-9"
      >
        <div className="mx-auto w-full max-w-[1120px] lg:w-[calc(100%+1rem)]">
          <div className="relative px-0 lg:px-2">
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5 lg:gap-4">
              {content.flow.items.map((item, index) => (
                <article
                  key={item.title}
                  className="relative z-10 flex min-h-[150px] min-w-0 flex-row overflow-hidden rounded-lg border border-(--app-border) bg-background text-left shadow-sm sm:min-h-[248px] sm:flex-col"
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
    </>
  )
}
