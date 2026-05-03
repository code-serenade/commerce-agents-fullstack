import { useTranslation } from "react-i18next"
import type { AgentOutputSection } from "../types"

export interface OutputSectionsProps {
  sections: AgentOutputSection[]
}

export function OutputSections({ sections }: OutputSectionsProps) {
  const { t } = useTranslation()

  return (
    <section className="grid gap-4 lg:grid-cols-[16rem_1fr]">
      <nav className="rounded-lg border border-(--app-border) bg-(--app-surface) p-4 lg:sticky lg:top-4 lg:self-start">
        <p className="text-sm font-medium text-(--app-muted-text)">
          {t("agentOutput.sectionsNavigation")}
        </p>
        <div className="mt-3 flex flex-col gap-2">
          {sections.map((section, index) => (
            <a
              key={section.id}
              href={`#${section.id}`}
              className="rounded-md px-3 py-2 text-sm text-(--app-muted-text) hover:bg-(--app-active-bg) hover:text-(--app-active-text)"
            >
              {index + 1}. {section.title}
            </a>
          ))}
        </div>
      </nav>

      <div className="space-y-4">
        {sections.map((section) => (
          <article
            key={section.id}
            id={section.id}
            className="scroll-mt-4 rounded-lg border border-(--app-border) bg-(--app-surface) p-5"
          >
            <p className="text-sm font-medium text-(--app-muted-text)">
              {section.eyebrow}
            </p>
            <h2 className="mt-1 text-xl font-semibold text-(--app-text)">
              {section.title}
            </h2>
            <p className="mt-3 text-sm/6 text-(--app-text)">
              {section.content}
            </p>
            <ul className="mt-4 space-y-2">
              {section.bullets.map((bullet) => (
                <li
                  key={bullet}
                  className="rounded-md border border-(--app-border) bg-(--app-bg) px-3 py-2 text-sm/6 text-(--app-muted-text)"
                >
                  {bullet}
                </li>
              ))}
            </ul>
          </article>
        ))}
      </div>
    </section>
  )
}
