import type { HTMLAttributes } from "react"

export interface CommerceAgentLogoProps extends HTMLAttributes<HTMLDivElement> {
  size?: "sm" | "md" | "lg"
}

const sizeClassNames = {
  sm: "size-9 rounded-lg",
  md: "size-10 rounded-xl",
  lg: "size-12 rounded-2xl",
} as const

export function CommerceAgentLogo({
  className,
  size = "md",
  ...props
}: CommerceAgentLogoProps) {
  return (
    <div
      aria-hidden="true"
      className={[
        "flex shrink-0 items-center justify-center overflow-hidden bg-(--shell-logo-background) shadow-[var(--shell-logo-shadow)] ring-1 ring-(--shell-logo-ring)",
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
