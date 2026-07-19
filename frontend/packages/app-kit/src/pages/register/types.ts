import type { ReactNode } from "react"

export interface AuthRegisterExtraFieldDefinition {
  name: string
  label: string
  placeholder?: string
  helpText?: string
  required?: boolean
  maxLength?: number
}

export interface AuthRegisterCompletedValues {
  username: string
  password: string
  displayName: string
  email: string
  extraFields: Record<string, string>
}

export interface AuthRegisterPageProps {
  brandLogo?: ReactNode
  extraFields?: AuthRegisterExtraFieldDefinition[]
  showProfileFields?: boolean
  loginPath?: string
  footerExtraAction?: ReactNode
  onLoginClick?: () => void
  redirectAfterRegister?: boolean
  variant?: "split" | "minimal"
  onRegistered?: (values: AuthRegisterCompletedValues) => void | Promise<void>
}
