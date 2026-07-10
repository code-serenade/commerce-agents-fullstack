import { createContext, useContext, useState, type ReactNode } from "react"
import { useLocation, useNavigate } from "react-router"
import { LoginDialog } from "./LoginDialog"

type LoginTarget = { pathname: string; search?: string; hash?: string } | null
interface AuthDialogContextValue {
  requestLogin: (target?: LoginTarget) => void
}

const LOGIN_DIALOG_DESKTOP_MIN_WIDTH = 796

const AuthDialogContext = createContext<AuthDialogContextValue | null>(null)

export function AuthDialogProvider({ children }: { children: ReactNode }) {
  const navigate = useNavigate()
  const location = useLocation()
  const [open, setOpen] = useState(false)
  const [target, setTarget] = useState<LoginTarget>(null)

  const requestLogin = (nextTarget?: LoginTarget) => {
    if (
      typeof window !== "undefined" &&
      window.innerWidth < LOGIN_DIALOG_DESKTOP_MIN_WIDTH
    ) {
      const from = nextTarget ?? {
        pathname: location.pathname,
        search: location.search,
        hash: location.hash,
      }
      navigate("/login", { state: { from } })
      return
    }

    setTarget(nextTarget ?? null)
    setOpen(true)
  }

  const handleSuccess = () => {
    setOpen(false)
    if (target) {
      navigate(`${target.pathname}${target.search ?? ""}${target.hash ?? ""}`)
    }
    setTarget(null)
  }

  return (
    <AuthDialogContext.Provider value={{ requestLogin }}>
      {children}
      <LoginDialog
        open={open}
        onOpenChange={(nextOpen) => {
          setOpen(nextOpen)
          if (!nextOpen) setTarget(null)
        }}
        onSuccess={handleSuccess}
      />
    </AuthDialogContext.Provider>
  )
}

export function useAuthDialog() {
  const context = useContext(AuthDialogContext)
  if (!context)
    throw new Error("useAuthDialog must be used within AuthDialogProvider")
  return context
}
