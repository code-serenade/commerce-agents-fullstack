import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import {
  trackBaiduPageView,
  trackProductEvent,
  trackProductPageView,
} from "./product-analytics"

const analyticsApiState = vi.hoisted(() => ({
  trackAnalyticsEventsApi: vi.fn(),
}))

vi.mock("@/api/analytics", () => ({
  trackAnalyticsEventsApi: analyticsApiState.trackAnalyticsEventsApi,
}))

describe("product analytics", () => {
  const originalLocalStorage = window.localStorage
  const originalSessionStorage = window.sessionStorage

  beforeEach(() => {
    analyticsApiState.trackAnalyticsEventsApi.mockResolvedValue(null)
    vi.spyOn(console, "debug").mockImplementation(() => undefined)
    window.history.replaceState({}, "", "/agent-chat?utm_source=test")
    document.title = "Commerce Agents"
  })

  afterEach(() => {
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: originalLocalStorage,
    })
    Object.defineProperty(window, "sessionStorage", {
      configurable: true,
      value: originalSessionStorage,
    })
    vi.restoreAllMocks()
  })

  it("does not throw when browser storage rejects analytics writes", () => {
    const brokenStorage = {
      getItem: vi.fn(() => {
        throw new Error("storage unavailable")
      }),
      setItem: vi.fn(() => {
        throw new Error("storage unavailable")
      }),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 0,
    }
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: brokenStorage,
    })
    Object.defineProperty(window, "sessionStorage", {
      configurable: true,
      value: brokenStorage,
    })

    expect(() =>
      trackProductEvent({
        eventType: "agent_question_submit",
        featureKey: "agent_chat.submit",
      })
    ).not.toThrow()

    expect(analyticsApiState.trackAnalyticsEventsApi).toHaveBeenCalledTimes(1)
  })

  it("does not throw when route analytics storage or Baidu queue fails", () => {
    Object.defineProperty(window, "sessionStorage", {
      configurable: true,
      value: {
        getItem: vi.fn(() => {
          throw new Error("session unavailable")
        }),
        setItem: vi.fn(() => {
          throw new Error("session unavailable")
        }),
        removeItem: vi.fn(),
        clear: vi.fn(),
        key: vi.fn(),
        length: 0,
      },
    })

    const throwingQueue: unknown[] = []
    vi.spyOn(throwingQueue, "push").mockImplementation(() => {
      throw new Error("baidu unavailable")
    })
    window._hmt = throwingQueue

    expect(() => trackProductPageView("/agent-chat", "?q=1")).not.toThrow()
    expect(() => trackBaiduPageView("/agent-chat")).not.toThrow()
    expect(analyticsApiState.trackAnalyticsEventsApi).toHaveBeenCalled()
  })

  it("swallows product analytics API failures", async () => {
    analyticsApiState.trackAnalyticsEventsApi.mockRejectedValueOnce(
      new Error("network down")
    )

    expect(() =>
      trackProductEvent({
        eventType: "process_panel_open",
        featureKey: "agent_chat.process_panel.open",
      })
    ).not.toThrow()

    await new Promise((resolve) => window.setTimeout(resolve, 0))
    expect(console.debug).toHaveBeenCalledWith(
      "[analytics] event ignored",
      expect.any(Error)
    )
  })

  it("pushes SPA route page views to the Baidu analytics queue", () => {
    window._hmt = []

    expect(() => trackBaiduPageView("/commerce?tab=overview")).not.toThrow()
    expect(window._hmt).toContainEqual([
      "_trackPageview",
      "/commerce?tab=overview",
    ])
  })
})
