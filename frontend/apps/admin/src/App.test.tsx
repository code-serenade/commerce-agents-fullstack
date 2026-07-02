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
  PageLoading: ({ label }: { label?: string }) =>
    label ?? "Admin restoring session",
  addAuthProfileUserUpdatedListener: () => () => undefined,
}))

vi.mock("@workspace/ui-components/stable/toast", () => ({
  ToastProvider: () => null,
}))

vi.mock("@/routes/publicRoutes", () => ({
  publicRoutes: {
    path: "/",
    element: "Admin login",
  },
}))

vi.mock("@/routes/protectedRoutes", () => ({
  protectedRoutes: {
    path: "/",
    element: "Admin dashboard",
  },
}))

const adminApiState = vi.hoisted(() => ({
  getCurrentUserPermissionsApi: vi.fn(),
  meApi: vi.fn(),
}))

vi.mock("./api", () => ({
  getCurrentUserPermissionsApi: adminApiState.getCurrentUserPermissionsApi,
  meApi: adminApiState.meApi,
}))

describe("Admin App", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    window.history.replaceState({}, "", "/")
    adminApiState.meApi.mockReset()
    adminApiState.getCurrentUserPermissionsApi.mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("does not trap admins on the loading screen or erase tokens when session restore hangs", async () => {
    localStorage.setItem("token", "stale-admin-token")
    localStorage.setItem("refreshToken", "stale-admin-refresh")
    adminApiState.meApi.mockReturnValue(new Promise(() => undefined))
    adminApiState.getCurrentUserPermissionsApi.mockReturnValue(
      new Promise(() => undefined)
    )

    render(<TestProviders><App /></TestProviders>)

    expect(screen.getByText("admin.loading.app")).toBeInTheDocument()

    await act(async () => {
      await vi.advanceTimersByTimeAsync(12_000)
    })

    expect(screen.getByText("Admin login")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBe("stale-admin-token")
    expect(localStorage.getItem("refreshToken")).toBe("stale-admin-refresh")
  })

  it("preserves stored tokens when admin session restore fails from a temporary backend outage", async () => {
    vi.useRealTimers()
    localStorage.setItem("token", "hot-update-admin-token")
    localStorage.setItem("refreshToken", "hot-update-admin-refresh")
    adminApiState.meApi.mockRejectedValue({
      response: {
        data: { code: 503, message: "backend unavailable" },
        status: 503,
      },
    })

    render(<TestProviders><App /></TestProviders>)

    expect(await screen.findByText("Admin login")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBe("hot-update-admin-token")
    expect(localStorage.getItem("refreshToken")).toBe(
      "hot-update-admin-refresh"
    )
  })

  it("clears stored tokens when the backend reports disabled admin access", async () => {
    vi.useRealTimers()
    localStorage.setItem("token", "disabled-admin-token")
    localStorage.setItem("refreshToken", "disabled-admin-refresh")
    adminApiState.getCurrentUserPermissionsApi.mockRejectedValue({
      response: {
        data: { code: -11001, message: "admin user is disabled" },
        status: 403,
      },
    })
    adminApiState.meApi.mockResolvedValue({
      id: "1",
      display_id: "admin_1",
      email: "admin@example.com",
      name: "Admin",
      avatar: "",
    })

    render(<TestProviders><App /></TestProviders>)

    expect(await screen.findByText("Admin login")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBeNull()
    expect(localStorage.getItem("refreshToken")).toBeNull()
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
      adminAccess: accessReducer,
      requestError: requestErrorReducer,
    },
  })

  return (
    <QueryClientProvider client={queryClient}>
      <Provider store={store}>{children}</Provider>
    </QueryClientProvider>
  )
}
