import { describe, expect, it } from "vitest"
import { isUnsupportedBenchmarkClaim } from "./agent-chat-answer-sanitizer"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent chat answer sanitizer", () => {
  it("hides backend skill marker text from persisted assistant answers", () => {
    expect(
      formatAgentUiMessage(
        "Suggestion: improve the entry point;skill:data_refund_decomposition status=Deferred reason=data skill executed\nskill:creative_copy status=ok\nNext step: add more data",
        copy
      )
    ).toBe("Suggestion: improve the entry point\nNext step: add more data")
  })

  it("keeps paragraph breaks from backend final answer formatting", () => {
    const conclusion = textFromCodes([32467, 35770, 20808, 30475, 21830, 21697, 36164, 26009, 12290])
    const coreHeading = textFromCodes([12304, 26680, 24515, 19994, 21153, 32467, 35770, 19982, 23450, 20215, 21028, 26029, 12305])
    const productFit = textFromCodes([36825, 27454, 21830, 21697, 36866, 21512, 20808, 20570, 36873, 21697, 23450, 20301, 12290])
    const sourceHeading = textFromCodes([12304, 20381, 25454, 35828, 26126, 12305])
    const sourceText = textFromCodes([24403, 21069, 20381, 25454, 26469, 33258, 21830, 21697, 36164, 26009, 12290])
    const formattedAnswer = [
      conclusion,
      coreHeading,
      productFit,
      sourceHeading,
      sourceText,
    ].join("\n\n")

    expect(
      formatAgentUiMessage(
        formattedAnswer,
        copy
      )
    ).toBe(formattedAnswer)
  })

  it("hides task graph shells, role labels, placeholders, and note labels", () => {
    const raw = [
      `Task graph completed. Task: ${textFromCodes([36825, 20010, 21654, 21857, 22914, 20309, 20570, 33829, 38144])}`,
      "Passed worker reports",
      `- ${textFromCodes([20869, 23481, 21019, 24847, 40, 99, 114, 101, 97, 116, 105, 118, 101, 41])}: ${textFromCodes([12304, 21487, 30452, 25509, 27979, 35797, 30340, 21315, 20048, 21654, 21857, 33829, 38144, 21019, 24847, 65288, 21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687, 65289, 12305])}`,
      textFromCodes([49, 46, 32, 20027, 25512, 26631, 39064, 65306, 21315, 20048, 21654, 21857, 12304, 30495, 23454, 21697, 31867, 65288, 22914, 25346, 32819, 47, 36895, 28342, 47, 33014, 22218, 47, 21654, 21857, 35910, 65289, 12305, 26085, 24120, 21475, 31918, 27454]),
      textFromCodes([22791, 27880, 65306, 26367, 25442, 30495, 23454, 21697, 31867, 21518, 21363, 21487, 19978, 32447, 65292, 27979, 35797, 26680, 24515, 25351, 26631, 21442, 32771, 28857, 20987, 29575]),
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)
    const lower = formatted.toLowerCase()

    expect(formatted).toContain(
      [textFromCodes([21315, 20048, 21654, 21857]), textFromCodes([30495, 23454, 21697, 31867, 26085, 24120, 21475, 31918, 27454])].join("")
    )
    expect(formatted).toContain(
      textFromCodes([35828, 26126, 65306, 26367, 25442, 30495, 23454, 21697, 31867, 21518, 21363, 21487, 19978, 32447])
    )
    expect(lower).not.toContain("task graph")
    expect(lower).not.toContain("passed worker")
    expect(lower).not.toContain("worker")
    expect(formatted).not.toContain(textFromCodes([20869, 23481, 21019, 24847, 40, 99, 114, 101, 97, 116, 105, 118, 101, 41]))
    expect(formatted).not.toContain(textFromCodes([21344, 20301, 31526]))
    expect(formatted).not.toContain(textFromCodes([22791, 27880, 65306]))
    expect(formatted).not.toContain(textFromCodes([12304]))
    expect(formatted).not.toContain(textFromCodes([12305]))
  })

  it("hides legacy internal role quality failures from persisted answers", () => {
    const raw = [
      textFromCodes([22810, 26234, 33021, 20307, 20219, 21153, 22270, 24050, 23436, 25104, 12290]),
      textFromCodes([36890, 36807, 30340, 35282, 33394, 20135, 29289, 65306]),
      `- ${textFromCodes([36816, 33829, 26041, 26696])}(ops): ${textFromCodes([24050, 24418, 25104, 31867, 30446, 32423, 21028, 26029, 12290])}`,
      textFromCodes([26410, 23436, 25104, 33410, 28857, 65306]),
      `- ${textFromCodes([25968, 25454, 35786, 26029])}(data) failed: data role should provide at least two findings`,
      `- ${textFromCodes([26381, 21153, 26041, 26696])}(service) failed: service role output lacks service-specific content`,
    ].join("\n")
    const formatted = formatAgentUiMessage(raw, copy)
    const opsRoleLabel = [textFromCodes([36816, 33829, 26041, 26696]), "(ops)"].join("")
    const opsContent = textFromCodes([
      24050, 24418, 25104, 31867, 30446, 32423, 21028, 26029, 12290,
    ])

    expect(formatted).toContain(opsContent)
    expect(formatted).toContain(copy.boundaryNotice)
    expect(formatted).not.toContain(opsRoleLabel)
    expect(formatted).not.toContain("data role should provide")
    expect(formatted).not.toContain("service role output")
  })

  it("removes legacy unfinished-node blocks even when several internal failures are mixed", () => {
    const suggestion = textFromCodes([
      24314, 35758, 65306, 20808, 25353, 21654, 21857, 31867, 30446, 20570,
      29992, 25143, 12289, 22330, 26223, 12289, 20215, 26684, 24102, 21644,
      28192, 36947, 25286, 35299, 12290,
    ])
    const unfinishedHeading = textFromCodes([
      26410, 23436, 25104, 25110, 38656, 22797, 26680, 65306,
    ])
    const dataName = textFromCodes([25968, 25454, 35786, 26029])
    const serviceName = textFromCodes([26381, 21153, 26041, 26696])
    const engineeringName = textFromCodes([24037, 31243, 21028, 26029])
    const raw = [
      suggestion,
      unfinishedHeading,
      `- ${dataName}(data) failed: data role should provide at least two findings`,
      `- ${serviceName}(service) failed: service role output lacks service-specific content`,
      `- ${engineeringName}(engineering) failed: engineering role should provide technical risks or rollback boundaries`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(suggestion)
    expect(formatted).toContain(copy.boundaryNotice)
    expect(formatted).not.toContain(unfinishedHeading)
    expect(formatted).not.toContain("data role should")
    expect(formatted).not.toContain("service role output")
    expect(formatted).not.toContain("engineering role should")
  })

  it("removes role executor failure blocks with plain internal role ids", () => {
    const raw = [
      textFromCodes([
        26412, 36718, 27809, 26377, 24418, 25104, 23436, 25972, 22238, 31572,
        65292, 21407, 22240, 26159, 37096, 20998, 23703, 20301, 27809, 26377,
        25104, 21151, 25191, 34892, 12290,
      ]),
      "",
      `- data: ${textFromCodes([
        26381, 21153, 31471, 27809, 26377, 21487, 29992, 30340, 35813, 23703,
        20301, 25191, 34892, 22120, 25110, 23703, 20301, 25191, 34892, 22833,
        36133,
      ])}`,
      `- creative: ${textFromCodes([
        19978, 28216, 20381, 36182, 23703, 20301, 26410, 25104, 21151, 23436,
        25104,
      ])}`,
      `- ops: ${textFromCodes([
        19978, 28216, 20381, 36182, 23703, 20301, 26410, 25104, 21151, 23436,
        25104,
      ])}`,
      "",
      textFromCodes([
        35831, 26816, 26597, 26381, 21153, 31471, 23703, 20301, 37197, 32622,
        12289, 27169, 22411, 37197, 32622, 25110, 24037, 20855, 36830, 25509,
        29366, 24577, 21518, 37325, 35797, 12290,
      ]),
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toBe(copy.boundaryNotice)
    expect(formatted).not.toContain("data:")
    expect(formatted).not.toContain("creative:")
    expect(formatted).not.toContain("ops:")
    expect(formatted).not.toContain(textFromCodes([23703, 20301]))
    expect(formatted).not.toContain(textFromCodes([25191, 34892, 22120]))
  })

  it("removes unsupported industry benchmark ranges but keeps benchmark gaps", () => {
    const unsupportedIndustryRange = textFromCodes([21457, 29616, 65306, 26333, 20809, 21040, 28857, 20987, 36716, 21270, 29575, 56, 37, 20302, 20110, 34892, 19994, 53, 45, 49, 48, 37, 21306, 38388, 65292, 25552, 31034, 27969, 37327, 25110, 26631, 39064, 47, 23553, 38754, 21560, 24341, 21147, 19981, 36275, 12290])
    const unsupportedCommonClaim = textFromCodes([21457, 29616, 65306, 28857, 20987, 21040, 21152, 36141, 29575, 32422, 50, 53, 46, 48, 37, 65292, 31526, 21512, 22810, 25968, 30005, 21830, 30452, 25773, 30340, 21152, 36141, 36716, 21270, 29575, 12290])
    const unsupportedAttractionClaim = textFromCodes([21457, 29616, 65306, 26333, 20809, 21040, 28857, 20987, 29575, 32422, 32, 56, 46, 48, 37, 65292, 31526, 21512, 19968, 33324, 30452, 25773, 23553, 38754, 47, 26631, 39064, 21560, 24341, 21147, 39044, 26399, 12290])
    const allowedBenchmarkGap = textFromCodes([21457, 29616, 65306, 28857, 20987, 21040, 21152, 36141, 32422, 50, 53, 37, 65292, 38656, 35201, 25910, 38598, 34892, 19994, 22522, 20934, 21518, 20877, 21028, 26029, 26159, 21542, 20581, 24247, 12290])
    const allowedReviewGap = textFromCodes([39118, 38505, 65306, 38656, 35201, 34917, 40784, 34892, 19994, 22522, 20934, 12289, 23458, 21333, 20215, 21644, 25237, 25918, 25104, 26412, 21518, 22797, 26680, 12290])
    const unsupportedComparison = textFromCodes([24314, 35758, 65306, 20808, 23545, 27604, 34892, 19994, 51, 48, 45, 52, 48, 37, 21306, 38388, 20877, 20915, 23450, 26159, 21542, 25918, 37327, 12290])
    const raw = [
      unsupportedIndustryRange,
      unsupportedCommonClaim,
      unsupportedAttractionClaim,
      allowedBenchmarkGap,
      allowedReviewGap,
      unsupportedComparison,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).not.toContain(textFromCodes([34892, 19994, 53, 45, 49, 48, 37]))
    expect(formatted).not.toContain(textFromCodes([34892, 19994, 51, 48, 45, 52, 48, 37]))
    expect(formatted).not.toContain(textFromCodes([20302, 20110, 34892, 19994]))
    expect(formatted).not.toContain(textFromCodes([31526, 21512, 22810, 25968, 30005, 21830, 30452, 25773]))
    expect(formatted).not.toContain(textFromCodes([31526, 21512, 19968, 33324, 30452, 25773]))
    expect(formatted).toContain(textFromCodes([38656, 35201, 25910, 38598, 34892, 19994, 22522, 20934, 21518, 20877, 21028, 26029, 26159, 21542, 20581, 24247]))
    expect(formatted).toContain(textFromCodes([38656, 35201, 34917, 40784, 34892, 19994, 22522, 20934, 12289, 23458, 21333, 20215, 21644, 25237, 25918, 25104, 26412, 21518, 22797, 26680]))
  })

  it("classifies unsupported benchmark claims without hiding sourced or gap wording", () => {
    expect(
      isUnsupportedBenchmarkClaim(textFromCodes([26333, 20809, 21040, 28857, 20987, 36716, 21270, 29575, 56, 37, 20302, 20110, 34892, 19994, 53, 45, 49, 48, 37, 21306, 38388]))
    ).toBe(true)
    expect(
      isUnsupportedBenchmarkClaim(
        textFromCodes([28857, 20987, 21040, 21152, 36141, 29575, 32422, 50, 53, 46, 48, 37, 65292, 31526, 21512, 22810, 25968, 30005, 21830, 30452, 25773, 30340, 21152, 36141, 36716, 21270, 29575, 12290])
      )
    ).toBe(true)
    expect(
      isUnsupportedBenchmarkClaim(
        textFromCodes([26333, 20809, 21040, 28857, 20987, 29575, 32422, 32, 56, 46, 48, 37, 65292, 31526, 21512, 19968, 33324, 30452, 25773, 23553, 38754, 47, 26631, 39064, 21560, 24341, 21147, 39044, 26399, 12290])
      )
    ).toBe(true)
    expect(
      isUnsupportedBenchmarkClaim(textFromCodes([28857, 20987, 21040, 21152, 36141, 32422, 50, 53, 37, 65292, 38656, 35201, 25910, 38598, 34892, 19994, 22522, 20934, 21518, 20877, 21028, 26029, 26159, 21542, 20581, 24247]))
    ).toBe(false)
    expect(
      isUnsupportedBenchmarkClaim(textFromCodes([26469, 28304, 65306, 20844, 24320, 25253, 21578, 26174, 31034, 34892, 19994, 53, 45, 49, 48, 37, 21306, 38388]))
    ).toBe(false)
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}
