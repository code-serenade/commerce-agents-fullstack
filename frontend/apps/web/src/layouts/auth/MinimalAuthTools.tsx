import { LanguageSwitch, ThemeToggle } from "@workspace/app-kit"
import { CircleHelp } from "lucide-react"

export interface MinimalAuthToolsProps {
  helpLabel: string
  helpSteps: string[]
}

const authToolClassName =
  "size-9 cursor-pointer rounded-xl border border-[rgba(209,220,240,0.96)] bg-white/80 text-[#475467] shadow-[0_12px_28px_rgba(38,76,160,0.08)] backdrop-blur-xl hover:bg-white/90 dark:border-[rgba(159,179,217,0.22)] dark:bg-[rgba(11,18,38,0.58)] dark:text-[#d6e2ff] dark:shadow-[0_16px_34px_rgba(0,0,0,0.22)] dark:hover:bg-[rgba(16,25,50,0.78)]"

export function MinimalAuthTools({
  helpLabel,
  helpSteps,
}: MinimalAuthToolsProps) {
  return (
    <>
      <ThemeToggle className={authToolClassName} />
      <LanguageSwitch className={authToolClassName} />
      <details className="group relative">
        <summary
          className={`${authToolClassName} inline-flex cursor-pointer list-none items-center justify-center [&::-webkit-details-marker]:hidden`}
          aria-label={helpLabel}
          title={helpLabel}
        >
          <CircleHelp aria-hidden="true" className="size-5" strokeWidth={2.2} />
        </summary>
        <div className="invisible absolute top-12 right-0 z-40 w-[min(330px,calc(100vw-40px))] rounded-2xl border border-[rgba(209,220,240,0.96)] bg-white/90 p-[18px] text-left opacity-0 shadow-[0_24px_60px_rgba(38,76,160,0.16)] backdrop-blur-xl transition group-open:visible group-open:opacity-100 sm:w-[360px] dark:border-[rgba(159,179,217,0.24)] dark:bg-[rgba(11,18,38,0.86)] dark:shadow-[0_26px_70px_rgba(0,0,0,0.34)]">
          <ol className="m-0 grid gap-3 pl-5">
            {helpSteps.map((step) => (
              <li
                key={step}
                className="text-[13px] leading-[1.75] text-[#344054] dark:text-[#d8e4ff]"
              >
                {step}
              </li>
            ))}
          </ol>
        </div>
      </details>
    </>
  )
}
