import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { StudentNextActions } from "./thread-message-blocks"

describe("StudentNextActions contextual suggestions", () => {
  it("keeps product follow-up prompts tied to the current product question without a redundant category badge", async () => {
    const i18n = await createAgentChatTestI18n()
    const sunscreen = textFromCodes(38450, 26194, 34915)
    const student = textFromCodes(22823, 23398, 29983)
    const reportPrompts = i18n.t("agentChat.thread.nextActions.groups.report", {
      returnObjects: true,
    }) as string[]
    const productFollowupLabel = i18n.t(
      "agentChat.thread.nextActions.meta.productFollowup.label"
    )

    render(
      <AgentChatTestI18nProvider i18n={i18n}>
        <StudentNextActions
          message={{
            id: "assistant-sunscreen",
            role: "assistant",
            content: textFromCodes(
              20808,
              25353,
              24403,
              21069,
              38382,
              39064,
              12289,
              24050,
              36873,
              19978,
              19979,
              25991,
              21644,
              24050,
              26377,
              20449,
              24687,
              32473,
              20986,
              21487,
              25191,
              34892,
              33609,
              26696,
              65292,
              21518,
              32493,
              20063,
              21487,
              20197,
              25972,
              29702,
              25104,
              25253,
              21578,
              12290
            ),
          }}
          previousUserContent={
            textFromCodes(25105, 24819, 21334, 19968, 27454, 22799, 23395) +
            sunscreen +
            textFromCodes(65292, 20215, 26684) +
            "129" +
            textFromCodes(20803, 65292, 38754, 21521) +
            student +
            textFromCodes(
              65292,
              20027,
              22270,
              27604,
              36739,
              26222,
              36890,
              12290,
              35831,
              20687,
              24179,
              31561,
              20249,
              20276,
              19968,
              26679,
              20808,
              21028,
              26029,
              33021,
              19981,
              33021,
              20570,
              12290
            )
          }
        />
      </AgentChatTestI18nProvider>
    )

    expect(screen.getAllByText(new RegExp(sunscreen)).length).toBeGreaterThan(0)
    expect(
      screen.getAllByText(new RegExp(`129\\s*${textFromCodes(20803)}`)).length
    ).toBeGreaterThan(0)
    expect(screen.getAllByText(new RegExp(student)).length).toBeGreaterThan(0)
    expect(screen.getByText(/main image expression/i)).toBeInTheDocument()
    expect(screen.queryByText(reportPrompts[0]!)).not.toBeInTheDocument()
    expect(screen.queryByText(productFollowupLabel)).not.toBeInTheDocument()
  })
})

function textFromCodes(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
