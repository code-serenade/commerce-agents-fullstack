import {
  ArrowRight,
  ClipboardCheck,
  FileText,
  ListChecks,
  Workflow,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentSection } from "../../commerce/components/document-section"
import {
  getUsageGuideContent,
  type UsageGuideJourneyStep,
} from "../usage-guide-data"

const destinationIcons = [ClipboardCheck, FileText, ListChecks]

function UsageGuideScreenshot({ step }: { step: UsageGuideJourneyStep }) {
  if (!step.screenshot || !step.screenshotAlt) return null

  return (
    <figure className="ml-11 w-[calc(100%-2.75rem)] max-w-[800px] min-w-0 overflow-hidden rounded-lg border border-(--app-border)/80 bg-background">
      <div className="relative min-w-0 overflow-hidden">
        <img
          src={step.screenshot}
          alt={step.screenshotAlt}
          className="h-auto w-full"
        />
        {step.pins?.map((pin) => (
          <span
            key={`${step.id}-${pin.marker}`}
            className="absolute flex size-7 -translate-1/2 items-center justify-center rounded-full border-2 border-background bg-destructive text-xs font-semibold text-white shadow-md"
            style={{ left: `${pin.left}%`, top: `${pin.top}%` }}
          >
            {pin.marker}
          </span>
        ))}
      </div>
      {step.note ? (
        <figcaption className="border-t border-(--app-border)/80 bg-muted/15 px-3 py-2 text-xs/5 text-muted-foreground">
          {step.note}
        </figcaption>
      ) : null}
    </figure>
  )
}

function UsageGuideRecipe({
  description,
  items,
  prompt,
  promptLabel,
  title,
}: {
  description: string
  items: Array<{ title: string; detail: string }>
  prompt: string
  promptLabel: string
  title: string
}) {
  return (
    <div className="ml-11 w-[calc(100%-2.75rem)] space-y-3">
      <div className="overflow-hidden rounded-lg border border-(--app-border) bg-background">
        <div className="border-b border-(--app-border) bg-muted/15 px-4 py-3">
          <h4 className="text-sm font-semibold text-foreground">{title}</h4>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {description}
          </p>
        </div>
        <ol className="grid sm:grid-cols-2 xl:grid-cols-4">
          {items.map((item, index) => (
            <li
              key={item.title}
              className="flex min-w-0 gap-3 border-b border-(--app-border) p-3 last:border-b-0 sm:odd:border-r sm:nth-last-[-n+2]:border-b-0 xl:border-r xl:border-b-0 xl:last:border-r-0"
            >
              <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
                {index + 1}
              </span>
              <div className="min-w-0">
                <h5 className="text-sm font-semibold text-foreground">
                  {item.title}
                </h5>
                <p className="mt-1 text-xs/5 text-muted-foreground">
                  {item.detail}
                </p>
              </div>
            </li>
          ))}
        </ol>
      </div>
      <div className="rounded-r-lg border-l-3 border-primary/70 bg-muted/15 px-4 py-3">
        <p className="text-xs font-semibold text-primary">{promptLabel}</p>
        <p className="mt-1 text-sm/6 text-foreground">{prompt}</p>
      </div>
    </div>
  )
}

function UsageGuideDestinations({
  items,
  title,
}: {
  items: Array<{ purpose: string; title: string; detail: string }>
  title: string
}) {
  return (
    <div className="mt-3 ml-11 w-[calc(100%-2.75rem)] overflow-hidden rounded-lg border border-(--app-border) bg-background">
      <div className="border-b border-(--app-border) bg-muted/15 px-4 py-3">
        <h4 className="text-sm font-semibold text-foreground">{title}</h4>
      </div>
      <ul className="divide-y divide-(--app-border)">
        {items.map((item, index) => {
          const Icon = destinationIcons[index] ?? ClipboardCheck
          return (
            <li
              key={item.title}
              className="grid min-w-0 gap-3 p-3 sm:grid-cols-[minmax(0,0.75fr)_auto_minmax(0,1.5fr)] sm:items-center"
            >
              <div className="flex min-w-0 items-center gap-3">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary/10">
                  <Icon className="size-4 text-primary" />
                </span>
                <p className="text-sm font-medium text-foreground">
                  {item.purpose}
                </p>
              </div>
              <ArrowRight className="hidden size-4 text-muted-foreground sm:block" />
              <div className="min-w-0 pl-11 sm:pl-0">
                <h5 className="text-sm font-semibold text-primary">
                  {item.title}
                </h5>
                <p className="mt-1 text-xs/5 text-muted-foreground">
                  {item.detail}
                </p>
              </div>
            </li>
          )
        })}
      </ul>
    </div>
  )
}

export function UsageGuideJourney() {
  const { i18n, t } = useTranslation()
  const screenshotLanguage = i18n.resolvedLanguage
    ?.toLowerCase()
    .startsWith("en")
    ? "en"
    : "zh-CN"
  const content = getUsageGuideContent(t, screenshotLanguage).journey

  return (
    <DocumentSection
      title={content.title}
      description={content.description}
      icon={<Workflow className="size-4" />}
      contentClassName="pl-0 sm:pl-9"
    >
      <div className="grid min-w-0 gap-5 xl:grid-cols-[10rem_minmax(0,1fr)] xl:gap-7">
        <nav
          aria-label={content.title}
          className="flex min-w-0 gap-1 overflow-x-auto border-b border-(--app-border) pb-2 xl:sticky xl:top-0 xl:block xl:self-start xl:overflow-visible xl:border-b-0 xl:pb-0"
        >
          {content.steps.map((step, index) => (
            <a
              key={step.id}
              href={`#usage-guide-${step.id}`}
              className="flex min-h-10 min-w-32 shrink-0 items-center gap-2 border-b-2 border-transparent px-2 text-sm text-muted-foreground no-underline hover:border-primary hover:text-primary xl:min-w-0 xl:border-b-0 xl:border-l-2 xl:border-(--app-border) xl:hover:border-primary"
            >
              <span className="flex size-6 shrink-0 items-center justify-center rounded-full border border-current text-xs">
                {index + 1}
              </span>
              {step.navLabel}
            </a>
          ))}
        </nav>

        <div className="min-w-0">
          {content.steps.map((step, index) => (
            <article
              key={step.id}
              id={`usage-guide-${step.id}`}
              className="min-w-0 scroll-mt-4 border-b border-(--app-border) pb-6 last:border-b-0 last:pb-0 [&+article]:pt-6"
            >
              <div className="mb-3 flex min-w-0 items-start gap-3">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-semibold text-primary-foreground">
                  {index + 1}
                </span>
                <div className="min-w-0">
                  <h3 className="text-base/6 font-semibold text-foreground">
                    {step.title}
                  </h3>
                  <p className="mt-1 text-sm/6 text-muted-foreground">
                    {step.description}
                  </p>
                </div>
              </div>

              {step.id === "prompt" ? (
                <UsageGuideRecipe
                  description={content.recipeDescription}
                  items={content.recipe}
                  prompt={content.prompt}
                  promptLabel={content.promptLabel}
                  title={content.recipeTitle}
                />
              ) : null}
              <UsageGuideScreenshot step={step} />
              {step.id === "save" ? (
                <UsageGuideDestinations
                  items={content.destinations}
                  title={content.destinationsTitle}
                />
              ) : null}
            </article>
          ))}
        </div>
      </div>
    </DocumentSection>
  )
}
