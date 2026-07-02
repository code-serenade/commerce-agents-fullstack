import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { act, renderHook, waitFor } from "@testing-library/react"
import { createElement, type ReactNode } from "react"
import type { i18n as I18nInstance } from "i18next"
import { afterEach, describe, expect, it, vi } from "vitest"
import { RequestError } from "@workspace/services/errors/request-error"
import { currentTeamUsageQuotaQueryKey } from "@/api"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import * as profileData from "./profile-data"
import { useProfilePage } from "./use-profile-page"

vi.mock("./profile-data", async () => {
  const actual =
    await vi.importActual<typeof import("./profile-data")>("./profile-data")

  return {
    ...actual,
    loadAppProfile: vi.fn(),
    saveAppProfile: vi.fn(),
  }
})

describe("useProfilePage", () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it("refreshes the account quota cache after school and team are saved", async () => {
    const i18n = await createAgentChatTestI18n()
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries")

    vi.mocked(profileData.loadAppProfile).mockResolvedValue({
      userId: "user-1",
      displayId: "student-1",
      displayName: "Student 1",
      schoolName: null,
      teamName: null,
      isSchoolTeamBound: false,
    })
    vi.mocked(profileData.saveAppProfile).mockResolvedValue({
      userId: "user-1",
      displayId: "student-1",
      displayName: "Student 1",
      schoolName: "Commerce School",
      teamName: "Team A",
      isSchoolTeamBound: true,
    })

    const { result } = renderHook(() => useProfilePage(), {
      wrapper: ({ children }: { children: ReactNode }) =>
        createTestWrapper({ children, i18n, queryClient }),
    })

    await waitFor(() =>
      expect(result.current.appProfile.isLoading).toBe(false)
    )

    act(() => {
      result.current.appProfile.setSchoolName("Commerce School")
      result.current.appProfile.setTeamName("Team A")
    })
    await act(async () => {
      await result.current.appProfile.save()
    })

    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: currentTeamUsageQuotaQueryKey,
    })
    await waitFor(() =>
      expect(result.current.appProfile.profile?.isSchoolTeamBound).toBe(true)
    )
  })

  it("allows a partial school and team profile while keeping the account unbound", async () => {
    const i18n = await createAgentChatTestI18n()
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries")

    vi.mocked(profileData.loadAppProfile).mockResolvedValue({
      userId: "user-1",
      displayId: "student-1",
      displayName: "Student 1",
      schoolName: null,
      teamName: null,
      isSchoolTeamBound: false,
    })
    vi.mocked(profileData.saveAppProfile).mockResolvedValue({
      userId: "user-1",
      displayId: "student-1",
      displayName: "Student 1",
      schoolName: "Commerce School",
      teamName: null,
      isSchoolTeamBound: false,
    })

    const { result } = renderHook(() => useProfilePage(), {
      wrapper: ({ children }: { children: ReactNode }) =>
        createTestWrapper({ children, i18n, queryClient }),
    })

    await waitFor(() =>
      expect(result.current.appProfile.isLoading).toBe(false)
    )

    act(() => {
      result.current.appProfile.setSchoolName("Commerce School")
      result.current.appProfile.setTeamName("")
    })
    await act(async () => {
      await result.current.appProfile.save()
    })

    expect(profileData.saveAppProfile).toHaveBeenCalledWith({
      schoolName: "Commerce School",
      teamName: "",
    })
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: currentTeamUsageQuotaQueryKey,
    })
    await waitFor(() =>
      expect(result.current.appProfile.profile?.isSchoolTeamBound).toBe(false)
    )
  })

  it("shows a clear initialization hint when the app profile is missing", async () => {
    const i18n = await createAgentChatTestI18n()
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })

    vi.mocked(profileData.loadAppProfile).mockRejectedValue(
      new RequestError("missing app user", {
        kind: "api",
        code: -12000,
        status: 404,
      })
    )

    const { result } = renderHook(() => useProfilePage(), {
      wrapper: ({ children }: { children: ReactNode }) =>
        createTestWrapper({ children, i18n, queryClient }),
    })

    await waitFor(() =>
      expect(result.current.appProfile.isLoading).toBe(false)
    )

    expect(result.current.appProfile.loadError).toBe(
      "This account has not initialized its training profile yet. Open the Q&A page once, then come back to add school and team."
    )
  })
})

function createTestWrapper({
  children,
  i18n,
  queryClient,
}: {
  children: ReactNode
  i18n: I18nInstance
  queryClient: QueryClient
}) {
  return createElement(
    QueryClientProvider,
    { client: queryClient },
    createElement(
      AgentChatTestI18nProvider,
      { i18n, children },
    )
  )
}
