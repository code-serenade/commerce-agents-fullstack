import type { CSSProperties } from "react"

export const homeStudentPathImageSources = {
  assistant: "/home/student-path-01-assistant.png",
  products: "/home/student-path-02-products.png",
  campaigns: "/home/student-path-03-campaign.png",
  tasks: "/home/student-path-04-tasks.png",
  reports: "/home/student-path-05-report.png",
} as const

export const homeStepCardStyle: CSSProperties = {
  background:
    "linear-gradient(180deg, color-mix(in oklch, var(--background) 98%, var(--primary) 2%) 0%, color-mix(in oklch, var(--background) 94%, var(--primary) 6%) 54%, color-mix(in oklch, var(--background) 90%, var(--primary) 10%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 98%, transparent)",
  boxShadow:
    "0 22px 52px color-mix(in oklch, var(--primary) 9%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 98%, transparent)",
}
