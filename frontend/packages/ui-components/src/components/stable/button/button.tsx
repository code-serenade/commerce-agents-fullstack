import type { ButtonHTMLAttributes, ReactNode } from "react"
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
export type ButtonClassNameMode = "merge" | "replace"

export interface ButtonProps extends Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "className" | "disabled" | "type"
> {
  children: ReactNode
  className?: string
  variant?: ButtonVariant
  size?: ButtonSize
  classNameMode?: ButtonClassNameMode
  fullWidth?: boolean
  type?: ButtonType
  disabled?: boolean
}

export function Button({
  children,
  className,
  classNameMode = "merge",
  variant = "default",
  size = "default",
  fullWidth = false,
  type = "button",
  disabled = false,
  onClick,
  ...buttonProps
}: ButtonProps) {
  return (
    <CoreButton
      variant={variant}
      size={size}
      className={cn(
        !disabled && "cursor-pointer",
        fullWidth && "w-full",
        className
      )}
      classNameMode={classNameMode}
      type={type}
      disabled={disabled}
      onClick={onClick}
      {...buttonProps}
    >
      {children}
    </CoreButton>
  )
}
