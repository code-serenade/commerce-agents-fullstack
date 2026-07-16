import { GraduationCap, Route } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard, DocumentSection } from "../../commerce/components/document-section"
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
      <DocumentIntroCard
        className="border-(--app-border) bg-background bg-none shadow-none backdrop-blur-none before:hidden"
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
