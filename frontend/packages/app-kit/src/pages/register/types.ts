import type { ReactNode } from "react"

export interface AuthRegisterPageProps {
  brandLogo?: ReactNode
  loginPath?: string
  onRegistered?: () => void | Promise<void>
}
