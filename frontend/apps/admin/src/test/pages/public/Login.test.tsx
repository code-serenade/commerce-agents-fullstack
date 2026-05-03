import { beforeEach, describe, expect, it, vi } from "vitest"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { Provider } from "react-redux"
import { configureStore } from "@reduxjs/toolkit"
import { createMemoryRouter, RouterProvider } from "react-router"
import authReducer from "@/store/authSlice"
import LoginPage from "@/pages/public/Login"

const { getCurrentUserPermissionsApiMock, loginApiMock, meApiMock } =
  vi.hoisted(() => ({
    loginApiMock: vi.fn(),
    meApiMock: vi.fn(),
    getCurrentUserPermissionsApiMock: vi.fn(),
  }))

vi.mock("@/api", () => ({
  loginApi: loginApiMock,
  meApi: meApiMock,
  getCurrentUserPermissionsApi: getCurrentUserPermissionsApiMock,
}))

vi.mock("react-i18next", async (importOriginal) => {
  const actual = await importOriginal<typeof import("react-i18next")>()
  return {
    ...actual,
    useTranslation: () => ({
      t: (key: string) => key,
      i18n: {
        language: "en",
        changeLanguage: vi.fn(),
      },
    }),
  }
})

function renderLoginPage() {
  const store = configureStore({
    reducer: {
      auth: authReducer,
    },
  })

  const router = createMemoryRouter(
    [
      { path: "/login", element: <LoginPage /> },
      { path: "/", element: <div>HOME_PAGE</div> },
    ],
    { initialEntries: ["/login"] }
  )

  render(
    <Provider store={store}>
      <RouterProvider router={router} />
    </Provider>
  )

  return { store }
}

describe("LoginPage", () => {
  beforeEach(() => {
    loginApiMock.mockResolvedValue({
      accessToken: "test-access-token",
      refreshToken: "test-refresh-token",
    })
    meApiMock.mockResolvedValue({
      id: "1",
      display_id: "u_test_1001",
      email: "tester@example.com",
      name: "Tester",
      avatar: "",
    })
    getCurrentUserPermissionsApiMock.mockResolvedValue({
      userId: "1",
      roleCodes: ["super_admin"],
      permissionCodes: ["admin:user", "admin:access"],
    })
  })

  it("shows validation errors for empty submit", async () => {
    const user = userEvent.setup()
    renderLoginPage()

    await user.click(screen.getByRole("button", { name: "login.form.submit" }))

    expect(
      await screen.findAllByText("validation.required")
    ).toHaveLength(2)
  })

  it("logs in and navigates to home", async () => {
    const user = userEvent.setup()
    const { store } = renderLoginPage()

    await user.type(
      screen.getByPlaceholderText("login.form.identifier.placeholder"),
      "tester"
    )
    await user.type(
      screen.getByPlaceholderText("login.form.password.placeholder"),
      "12345678"
    )
    await user.click(screen.getByRole("button", { name: "login.form.submit" }))

    expect(await screen.findByText("HOME_PAGE")).toBeInTheDocument()
    expect(localStorage.getItem("token")).toBe("test-access-token")
    expect(localStorage.getItem("refreshToken")).toBe("test-refresh-token")

    await waitFor(() => {
      expect(store.getState().auth.isLogin).toBe(true)
      expect(store.getState().auth.user?.name).toBe("Tester")
    })
  })
})
