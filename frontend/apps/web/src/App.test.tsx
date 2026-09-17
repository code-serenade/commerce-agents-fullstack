import { configureStore } from "@reduxjs/toolkit"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { act, render, screen } from "@testing-library/react"
import type { ReactNode } from "react"
import { Provider } from "react-redux"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import accessReducer from "@/store/accessSlice"
import authReducer from "@/store/authSlice"
import requestErrorReducer from "@/store/requestErrorSlice"
import App from "./App"

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock("@workspace/app-kit", () => ({
  GlobalRequestError: ({ message }: { message?: string | null }) =>
    message ?? null,
  PageLoading: () => "App restoring session",
  addAuthProfileUserUpdatedListener: () => () => undefined,
}))

vi.mock("@workspace/ui-components/stable/toast", () => ({
  ToastProvider: () => null,
}))

vi.mock("@/routes/siteRoutes", () => ({
  siteRoutes: {
    path: "/",
    element: "Commerce home",
  },
}))

vi.mock("@/routes/authRoutes", () => ({
  authRoutes: {
    path: "/",
    children: [
      {
        path: "login",
        element: "Login page",
      },
    ],
  },
}))

const appApiState = vi.hoisted(() => ({
  ensureCurrentAppUserPermissions: vi.fn(),
  meApi: vi.fn(),
}))

vi.mock("./api", () => ({
  ensureCurrentAppUserPermissions: appApiState.ensureCurrentAppUserPermissions,
  meApi: appApiState.meApi,
}))

describe("App", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    window.history.replaceState({}, "", "/")
    appApiState.meApi.mockReset()
    appApiState.ensureCurrentAppUserPermissions.mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("does not trap users on the loading screen or erase tokens when session restore hangs", async () => {
    localStorage.setItem("token", "stale-token")
    localStorage.setItem("refreshToken", "stale-refresh")
    appApiState.meApi.mockReturnValue(new Promise(() => undefined))

    render(<TestProviders><App /></TestProviders>)

    expect(appApiState.meApi).toHaveBeenCalledWith({
      suppressGlobalError: true,
    })
    expect(screen.getByText("App restoring session")).toBeInTheDocument()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(12_000)
    })

    expect(screen.getByText("Commerce home")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBe("stale-token")
    expect(localStorage.getItem("refreshToken")).toBe("stale-refresh")
  })

  it("preserves stored tokens when session restore fails from a temporary backend outage", async () => {
    vi.useRealTimers()
    localStorage.setItem("token", "hot-update-token")
    localStorage.setItem("refreshToken", "hot-update-refresh")
    appApiState.meApi.mockRejectedValue({
      response: {
        data: { code: 503, message: "backend unavailable" },
        status: 503,
      },
    })

    render(<TestProviders><App /></TestProviders>)

    expect(await screen.findByText("Commerce home")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBe("hot-update-token")
    expect(localStorage.getItem("refreshToken")).toBe("hot-update-refresh")
  })
})

function TestProviders({ children }: { children: ReactNode }) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })
  const store = configureStore({
    reducer: {
      auth: authReducer,
      webAccess: accessReducer,
      requestError: requestErrorReducer,
    },
  })

  return (
    <QueryClientProvider client={queryClient}>
      <Provider store={store}>{children}</Provider>
    </QueryClientProvider>
  )
}
