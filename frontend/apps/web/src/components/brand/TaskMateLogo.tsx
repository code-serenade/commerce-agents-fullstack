import type { HTMLAttributes } from "react"

export interface TaskMateLogoProps extends HTMLAttributes<HTMLDivElement> {
  size?: "sm" | "md" | "lg"
}

const sizeClassNames = {
  sm: "size-9 rounded-lg",
  md: "size-10 rounded-xl",
  lg: "size-12 rounded-2xl",
} as const

export function TaskMateLogo({
  className,
  size = "md",
  ...props
}: TaskMateLogoProps) {
  return (
    <div
      aria-hidden="true"
      className={[
        "flex shrink-0 items-center justify-center overflow-hidden bg-(--app-surface) shadow-sm ring-1 ring-(--app-border)",
        sizeClassNames[size],
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      {...props}
    >
      <img
        src="/favicon.svg"
        alt=""
        className="size-full object-cover"
        draggable={false}
      />
    </div>
  )
}
