import { ArrowRight } from "lucide-react"
import type { CSSProperties } from "react"
import type { TFunction } from "i18next"

interface StudentPathItem {
  titleKey: string
  descriptionKey: string
  actionKey: string
  imageSrc: string
  path: string
  requiresLogin: boolean
}

const studentPathItems: StudentPathItem[] = [
  {
    titleKey: "home.studentPath.steps.assistant.title",
    descriptionKey: "home.studentPath.steps.assistant.description",
    actionKey: "home.studentPath.steps.assistant.action",
    imageSrc: "/home/student-path-01-assistant.png",
    path: "/commerce",
    requiresLogin: false,
  },
  {
    titleKey: "home.studentPath.steps.products.title",
    descriptionKey: "home.studentPath.steps.products.description",
    actionKey: "home.studentPath.steps.products.action",
    imageSrc: "/home/student-path-02-products.png",
    path: "/products",
    requiresLogin: true,
  },
  {
    titleKey: "home.studentPath.steps.campaigns.title",
    descriptionKey: "home.studentPath.steps.campaigns.description",
    actionKey: "home.studentPath.steps.campaigns.action",
    imageSrc: "/home/student-path-03-campaign.png",
    path: "/campaigns",
    requiresLogin: true,
  },
  {
    titleKey: "home.studentPath.steps.tasks.title",
    descriptionKey: "home.studentPath.steps.tasks.description",
    actionKey: "home.studentPath.steps.tasks.action",
    imageSrc: "/home/student-path-04-tasks.png",
    path: "/workspaces",
    requiresLogin: true,
  },
  {
    titleKey: "home.studentPath.steps.reports.title",
    descriptionKey: "home.studentPath.steps.reports.description",
    actionKey: "home.studentPath.steps.reports.action",
    imageSrc: "/home/student-path-05-report.png",
    path: "/agent-output",
    requiresLogin: true,
  },
]

const routeSegments: CSSProperties[] = [
  {
    left: "-10%",
    top: "48%",
    width: "17%",
    transform: "rotate(-15deg)",
  },
  {
    left: "11%",
    top: "44%",
    width: "16%",
    transform: "rotate(7deg)",
  },
  {
    left: "28%",
    top: "46%",
    width: "18%",
    transform: "rotate(-4deg)",
  },
  {
    left: "46%",
    top: "44%",
    width: "17%",
    transform: "rotate(5deg)",
  },
  {
    left: "72%",
    top: "44.5%",
    width: "15%",
    transform: "rotate(10deg)",
  },
  {
    left: "93.5%",
    top: "44.5%",
    width: "8%",
    transform: "rotate(12deg)",
  },
]

const homeStepCardStyle: CSSProperties = {
  background:
    "linear-gradient(180deg, color-mix(in oklch, var(--background) 98%, var(--primary) 2%) 0%, color-mix(in oklch, var(--background) 94%, var(--primary) 6%) 54%, color-mix(in oklch, var(--background) 90%, var(--primary) 10%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 98%, transparent)",
  boxShadow:
    "0 22px 52px color-mix(in oklch, var(--primary) 9%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 98%, transparent)",
}

export interface HomeStudentPathProps {
  t: TFunction
  navigateToPath: (path: string, requiresLogin?: boolean) => void
}

export function HomeStudentPath({ t, navigateToPath }: HomeStudentPathProps) {
  return (
    <section className="mx-auto w-full max-w-[1120px] space-y-5">
      <div className="space-y-1.5">
        <p className="text-[11px] font-semibold text-(--app-muted-text)">
          {t("home.studentPath.eyebrow")}
        </p>
        <h2 className="text-xl leading-tight font-semibold text-(--app-text)">
          {t("home.studentPath.title")}
        </h2>
        <p className="max-w-3xl text-xs/5 font-medium text-(--app-muted-text)">
          {t("home.studentPath.description")}
        </p>
      </div>

      <div className="relative px-0 lg:px-2">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute right-8 -bottom-10 hidden size-12 rotate-45 bg-[color-mix(in_oklch,var(--app-surface)_70%,transparent)] [clip-path:polygon(50%_0,62%_38%,100%_50%,62%_62%,50%_100%,38%_62%,0_50%,38%_38%)] lg:block"
        />
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0"
        >
          {routeSegments.map((style, index) => (
            <span
              key={index}
              className="absolute hidden h-[3px] origin-left rounded-full bg-[linear-gradient(90deg,transparent_0%,color-mix(in_oklch,var(--primary)_22%,transparent)_18%,color-mix(in_oklch,var(--primary)_24%,transparent)_82%,transparent_100%)] lg:block"
              style={style}
            />
          ))}
          <span className="absolute top-[calc(45.5%+6px)] right-[-1%] hidden size-4 rotate-45 border-t-[3px] border-r-[3px] border-[color-mix(in_oklch,var(--primary)_26%,transparent)] lg:block" />
        </div>

        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5 lg:gap-9">
          {studentPathItems.map((item, index) => (
            <button
              key={item.path}
              type="button"
              className="group relative z-10 flex min-h-[260px] cursor-pointer flex-col overflow-hidden rounded-lg border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(180deg,color-mix(in_oklch,var(--background)_98%,var(--primary)_2%)_0%,color-mix(in_oklch,var(--background)_94%,var(--primary)_6%)_54%,color-mix(in_oklch,var(--background)_90%,var(--primary)_10%)_100%)] text-left shadow-[0_22px_52px_color-mix(in_oklch,var(--primary)_9%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl transition hover:border-[color-mix(in_oklch,var(--background)_82%,var(--primary))]"
              style={homeStepCardStyle}
              onClick={() => navigateToPath(item.path, item.requiresLogin)}
            >
              <span className="pointer-events-none absolute top-2.5 left-2.5 z-10 text-2xl leading-none font-semibold text-(--app-muted-text)/58 italic">
                {String(index + 1).padStart(2, "0")}
              </span>
              <span className="block h-[126px] w-full overflow-hidden">
                <img
                  src={item.imageSrc}
                  alt=""
                  aria-hidden="true"
                  className="size-full object-contain p-3 pb-0"
                  loading="lazy"
                />
              </span>
              <span className="flex flex-1 flex-col px-3.5 pt-3 pb-3.5">
                <span className="block text-xs font-semibold text-(--app-text)">
                  {t(item.titleKey)}
                </span>
                <span className="mt-1.5 block min-h-10 text-[11px]/5 text-(--app-muted-text)">
                  {t(item.descriptionKey)}
                </span>
                <span className="mt-auto flex items-center gap-1 pt-2 text-[11px] font-semibold text-(--app-text)">
                  {t(item.actionKey)}
                  <ArrowRight className="size-3 transition group-hover:translate-x-0.5" />
                </span>
              </span>
            </button>
          ))}
        </div>
      </div>
    </section>
  )
}
