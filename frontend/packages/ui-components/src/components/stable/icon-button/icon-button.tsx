import type { MouseEventHandler, ReactNode } from "react"
import { Button as CoreButton } from "@workspace/ui-core/components/button"

export type IconButtonVariant =
  | "default"
  | "primary"
  | "info"
  | "success"
  | "warning"
  | "destructive"

export interface IconButtonProps {
  children: ReactNode
  label: ReactNode
  variant?: IconButtonVariant
  type?: "button" | "submit" | "reset"
  disabled?: boolean
  onClick?: MouseEventHandler<HTMLButtonElement>
}

const VARIANT_CLASS_NAMES: Record<IconButtonVariant, string> = {
  default:
    "bg-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
  primary:
    "bg-transparent text-primary hover:bg-primary/10 hover:text-primary",
  info: "bg-transparent text-[var(--info)] hover:bg-[color-mix(in_oklab,var(--info)_12%,transparent)] hover:text-[var(--info)]",
  success:
    "bg-transparent text-[var(--success)] hover:bg-[color-mix(in_oklab,var(--success)_12%,transparent)] hover:text-[var(--success)]",
  warning:
    "bg-transparent text-[var(--warning)] hover:bg-[color-mix(in_oklab,var(--warning)_12%,transparent)] hover:text-[var(--warning)]",
  destructive:
    "bg-transparent text-destructive hover:bg-destructive/10 hover:text-destructive",
}

export function IconButton({
  children,
  label,
  variant = "default",
  type = "button",
  disabled = false,
  onClick,
}: IconButtonProps) {
  return (
    <CoreButton
      type={type}
      variant="ghost"
      size="icon"
      disabled={disabled}
      onClick={onClick}
      className={VARIANT_CLASS_NAMES[variant]}
    >
      {children}
      <span className="sr-only">{label}</span>
    </CoreButton>
  )
}
