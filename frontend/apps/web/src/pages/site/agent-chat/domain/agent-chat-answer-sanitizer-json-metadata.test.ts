import { describe, expect, it } from "vitest"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent chat answer sanitizer JSON metadata cleanup", () => {
  it("removes leaked JSON metadata from persisted answers", () => {
    const raw = [
      textFromCodes([
        32467, 35770, 65306, 36825, 20010, 21830, 21697, 20808, 20570, 27979,
        35797, 12290,
      ]),
      `","agentContext":"${textFromCodes([20869, 37096, 19978, 19979, 25991])}","analysisId":"${textFromCodes([24050, 36873, 21830, 21697])}","promptVersion":"v1","llmName":"doubao-vision"}]`,
      textFromCodes([
        19979, 19968, 27493, 65306, 20808, 34917, 20805, 20215, 26684, 21644,
        30446, 26631, 20154, 32676, 12290,
      ]),
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(textFromCodes([32467, 35770]))
    expect(formatted).toContain(textFromCodes([19979, 19968, 27493]))
    expect(formatted).not.toContain("agentContext")
    expect(formatted).not.toContain("analysisId")
    expect(formatted).not.toContain("promptVersion")
    expect(formatted).not.toContain("llmName")
    expect(formatted).not.toContain("doubao-vision")
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
