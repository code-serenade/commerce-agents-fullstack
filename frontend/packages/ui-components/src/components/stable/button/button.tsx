import type { MouseEventHandler, ReactNode } from "react"
import { Button as CoreButton } from "@workspace/ui-core/components/button"
import { cn } from "../../../lib/utils"

export type ButtonVariant =
  | "default"
  | "primary"
  | "secondary"
  | "destructive"
  | "outline"
  | "ghost"
  | "link"

export type ButtonSize = "default" | "xs" | "sm" | "lg" | "icon"

export type ButtonType = "button" | "submit" | "reset"

export interface ButtonProps {
  children: ReactNode
  className?: string
  variant?: ButtonVariant
  size?: ButtonSize
  fullWidth?: boolean
  type?: ButtonType
  disabled?: boolean
  onClick?: MouseEventHandler<HTMLButtonElement>
}

export function Button({
  children,
  className,
  variant = "default",
  size = "default",
  fullWidth = false,
  type = "button",
  disabled = false,
  onClick,
}: ButtonProps) {
  return (
    <CoreButton
      variant={variant}
      size={size}
      className={cn(fullWidth && "w-full", className)}
      type={type}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </CoreButton>
  )
}
