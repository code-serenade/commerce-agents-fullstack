import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { I18nextProvider } from "react-i18next"
import i18next from "i18next"
import { describe, expect, it, vi } from "vitest"
import { WelcomeCard } from "./thread-message-blocks"

describe("WelcomeCard", () => {
  it("lets students choose an example prompt to start", async () => {
    const onSelectExamplePrompt = vi.fn()
    const i18n = i18next.createInstance()
    await i18n.init({
      lng: "en",
      resources: {
        en: {
          translation: {
            agentChat: {
              thread: {
                agentName: "AI training assistant",
                examplePrompts: [
                  "Judge whether this product is suitable for training product selection and explain why",
                ],
                startHere: "Click an example to start",
                welcomeCapabilities: [
                  "Describe your task first",
                  "Process opens automatically",
                  "Results become deliverables",
                ],
              },
            },
          },
        },
      },
    })

    render(
      <I18nextProvider i18n={i18n}>
        <WelcomeCard
          content="Hi, I am the AI training assistant."
          onSelectExamplePrompt={onSelectExamplePrompt}
        />
      </I18nextProvider>
    )

    await userEvent.click(
      screen.getByRole("button", {
        name: "Judge whether this product is suitable for training product selection and explain why",
      })
    )

    expect(onSelectExamplePrompt).toHaveBeenCalledWith(
      "Judge whether this product is suitable for training product selection and explain why"
    )
  })
})
