import { describe, expect, it } from "vitest"
import { formatAgentUiMessage } from "./agent-chat-message-mapping"

const copy = {
  boundaryNotice: "boundary notice",
  missingStructuredOutput: "missing structured output",
  streamTimedOut: "stream timed out",
}

describe("agent answer repeated visible content", () => {
  it("removes repeated trailing bullets from final answers", () => {
    const title = textFromCodes([
      22799, 23395, 20912, 19997, 38450, 26194, 34915, 22899, 38450, 32043,
      22806, 32447, 36879, 27668, 34180, 27454, 32, 25143, 22806, 39569,
      36710, 36830, 24125, 36974, 38451, 22806, 22871, 32, 26085, 24120,
      30334, 25645, 27454,
    ])
    const repeatedRisk = textFromCodes([
      22240, 32570, 23569, 21830, 21697, 30495, 23454, 26448, 36136, 12289,
      29256, 22411, 12289, 36866, 29992, 20154, 32676, 31561, 20449, 24687,
      65292, 21442, 32771, 26631, 39064, 19981, 33021, 23436, 20840, 21305,
      37197, 21830, 21697, 29420, 26377, 23646, 24615, 65292, 30452, 25509,
      20351, 29992, 21487, 33021, 36798, 19981, 21040, 26368, 20248, 28857,
      20987, 29575,
    ])
    const originalTitleQuestion = textFromCodes([
      24403, 21069, 21407, 26631, 39064, 30340, 23436, 25972, 20869, 23481,
      26159, 24590, 26679, 30340, 65311,
    ])
    const raw = [
      `## ${textFromCodes([21487, 30452, 25509, 20351, 29992, 20869, 23481])}`,
      title,
      "",
      `## ${textFromCodes([39118, 38505, 36793, 30028])}`,
      repeatedRisk,
      "",
      `## ${textFromCodes([19979, 19968, 27493])}`,
      originalTitleQuestion,
      `- ${repeatedRisk}`,
      `- ${originalTitleQuestion}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(title)
    expect(countOccurrences(formatted, repeatedRisk)).toBe(1)
    expect(countOccurrences(formatted, originalTitleQuestion)).toBe(1)
  })

  it("compacts repeated sunscreen title boundary bullets", () => {
    const boundary = textFromCodes([
      24403, 21069, 26080, 35813, 38450, 26194, 34915, 19987, 23646, 25968,
      25454, 65292, 20165, 33021, 22522, 20110, 36890, 29992, 30005, 21830,
      38450, 26194, 34915, 26631, 39064, 28857, 20987, 29575, 36923, 36753,
      32473, 20986, 20248, 21270, 26041, 21521, 12290,
    ])
    const direction = textFromCodes([
      21407, 26631, 39064, 26410, 21069, 32622, 29992, 25143, 25628, 38450,
      26194, 34915, 26102, 30340, 26680, 24515, 38656, 27714, 35789, 65292,
      23548, 33268, 28857, 20987, 29575, 20302, 65292, 38656, 25226, 29992,
      25143, 25628, 38450, 26194, 34915, 26102, 30340, 26680, 24515, 38656,
      27714, 35789, 25918, 22312, 26631, 39064, 26368, 21069, 38754, 65307,
    ])
    const title = textFromCodes([
      26085, 24120, 25143, 22806, 36731, 34180, 36879, 27668, 38450, 26194,
      34915,
    ])
    const repeatedBoundary = textFromCodes([
      24403, 21069, 26080, 35813, 38450, 26194, 34915, 19987, 23646, 25968,
      25454, 65292, 20165, 33021, 22522, 20110, 36890, 29992, 30005, 21830,
      38450, 26194, 34915, 26631, 39064, 28857, 20987, 29575, 36923, 36753,
      21028, 26029, 20248, 21270, 26041, 21521,
    ])
    const repeatedNeed = textFromCodes([
      38450, 26194, 34915, 26631, 39064, 28857, 20987, 29575, 20302, 30340,
      26680, 24515, 21407, 22240, 26159, 26410, 25226, 29992, 25143, 25628,
      38450, 26194, 34915, 26102, 31532, 19968, 30524, 25214, 30340, 26680,
      24515, 38656, 27714, 35789, 25918, 22312, 26631, 39064, 26368, 21069,
      38754,
    ])
    const repeatedFollowUp = textFromCodes([
      21518, 32493, 21487, 34917, 20805, 35813, 38450, 26194, 34915, 30340,
      19987, 23646, 26333, 20809, 12289, 28857, 20987, 31561, 25968, 25454,
      65292, 36827, 19968, 27493, 20248, 21270, 26631, 39064,
    ])
    const repeatedVerification = textFromCodes([
      26080, 35813, 38450, 26194, 34915, 19987, 23646, 25968, 25454, 65292,
      20248, 21270, 26041, 21521, 21450, 26631, 39064, 20165, 20026, 36890,
      29992, 36923, 36753, 19979, 30340, 24314, 35758, 65292, 23454, 38469,
      25928, 26524, 38656, 32467, 21512, 19987, 23646, 25968, 25454, 39564,
      35777,
    ])
    const raw = [
      boundary,
      `## ${textFromCodes([20248, 21270, 26041, 21521])}`,
      direction,
      `## ${textFromCodes([21487, 30452, 25509, 25913, 29992, 30340, 33258, 28982, 26631, 39064])}`,
      `- ${title}`,
      `- ${repeatedBoundary}`,
      `- ${repeatedNeed}`,
      `- ${repeatedFollowUp}`,
      `- ${repeatedVerification}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(title)
    expect(formatted).toContain(direction)
    expect(formatted).toContain(repeatedFollowUp)
    expect(formatted).not.toContain(repeatedBoundary)
    expect(formatted).not.toContain(repeatedNeed)
    expect(formatted).not.toContain(repeatedVerification)
  })

  it("keeps one sunscreen title answer structure when backend returns two similar sections", () => {
    const firstDirection = textFromCodes([
      26368, 20248, 20808, 20248, 21270, 26041, 21521, 21028, 26029,
    ])
    const firstTitleHeading = textFromCodes([
      32473, 20320, 19968, 29256, 36890, 29992, 33258, 28982, 26631,
      39064, 21442, 32771,
    ])
    const firstTesting = textFromCodes([
      21518, 32493, 25968, 25454, 35760, 24405, 24314, 35758,
    ])
    const secondDirection = textFromCodes([
      20248, 20808, 20248, 21270, 26041, 21521, 21028, 26029,
    ])
    const secondTitleHeading = textFromCodes([
      36890, 29992, 33258, 28982, 26631, 39064, 21442, 32771,
    ])
    const whyHeading = textFromCodes([
      20026, 20160, 20040, 36825, 20040, 25913, 65311,
    ])
    const secondTesting = textFromCodes([
      27979, 35797, 21644, 22797, 30424, 26041, 27861,
    ])
    const riskHeading = textFromCodes([
      27880, 24847, 39118, 38505, 21644, 24453, 34917, 20449, 24687,
    ])
    const title = textFromCodes([
      20912, 19997, 20937, 24863, 38450, 26194, 34915, 22899, 50, 48,
      50, 53, 26032, 27454, 38450, 32043, 22806, 32447, 22799, 23395,
      36879, 27668, 34180, 27454, 117, 112, 102, 53, 48, 43, 38450,
      26194, 26381, 22806, 22871,
    ])
    const oldDirectionBody = textFromCodes([
      28857, 20987, 29575, 20302, 22823, 27010, 29575, 26159, 26631,
      39064, 27809, 36393, 20013, 29992, 25143, 25628, 38450, 26194,
      34915, 26102, 30340, 39640, 39057, 38656, 27714, 35789,
    ])
    const finalDirectionBody = textFromCodes([
      20320, 29616, 22312, 33021, 31435, 21051, 35843, 25972, 12289,
      35797, 38169, 25104, 26412, 26368, 20302, 30340, 23601, 26159,
      26631, 39064,
    ])
    const testBody = textFromCodes([
      25913, 26631, 39064, 30340, 26102, 20505, 20215, 26684, 12289,
      20027, 22270, 20808, 21035, 21160, 65292, 36991, 20813, 22810,
      20010, 21464, 37327, 21516, 26102, 25913, 19981, 30693, 36947,
      26159, 35841, 36215, 30340, 20316, 29992,
    ])
    const riskBody = textFromCodes([
      26631, 39064, 37324, 30340, 20869, 23481, 24517, 39035, 26159,
      20320, 23478, 21830, 21697, 30495, 30340, 26377, 30340,
    ])
    const raw = [
      `## ${firstDirection}`,
      oldDirectionBody,
      `## ${firstTitleHeading}`,
      title,
      `## ${firstTesting}`,
      testBody,
      `## ${secondDirection}`,
      finalDirectionBody,
      `## ${secondTitleHeading}`,
      title,
      `## ${whyHeading}`,
      oldDirectionBody,
      `## ${secondTesting}`,
      testBody,
      `## ${riskHeading}`,
      riskBody,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).not.toContain(firstDirection)
    expect(formatted).not.toContain(firstTitleHeading)
    expect(formatted).not.toContain(firstTesting)
    expect(formatted).toContain(secondDirection)
    expect(formatted).toContain(secondTitleHeading)
    expect(formatted).toContain(whyHeading)
    expect(formatted).toContain(secondTesting)
    expect(formatted).toContain(riskHeading)
    expect(countOccurrences(formatted, title)).toBe(1)
  })

  it("truncates raw role-output bullets after a structured follow-up field list", () => {
    const naturalTitle = textFromCodes([
      22799, 23395, 20912, 19997, 38450, 26194, 34915, 22899, 38450, 32043,
      22806, 32447, 36879, 27668, 34180, 27454, 25143, 22806, 39569,
      34892, 38450, 26194, 34915,
    ])
    const followupHeading = textFromCodes([
      21518, 32493, 20320, 21487, 20197, 34917, 20805, 36825, 20123,
      23383, 27573, 26469, 20570, 26356, 31934, 20934, 30340, 20998,
      26512,
    ])
    const recordScope = textFromCodes([
      35760, 24405, 33539, 22260, 65306, 36817, 49, 45, 50, 21608,
      35813, 38450, 26194, 34915, 30340, 26631, 39064, 12289, 20215,
      26684, 12289, 20027, 22270, 12289, 28857, 20987, 29575, 25968,
      25454, 65307,
    ])
    const metricCalculation = textFromCodes([
      25351, 26631, 35745, 31639, 65306, 28857, 20987, 29575, 61,
      28857, 20987, 37327, 47, 26333, 20809, 37327, 65307,
    ])
    const sampleBoundary = textFromCodes([
      26679, 26412, 36793, 30028, 65306, 21462, 36817, 49, 45, 50,
      21608, 30340, 24179, 21488, 33258, 28982, 26333, 20809, 25968,
      25454, 65307,
    ])
    const missingData = textFromCodes([
      32570, 22833, 25968, 25454, 65306, 30446, 21069, 32570, 23569,
      26333, 20809, 37327, 12289, 28857, 20987, 37327, 12289, 29992,
      25143, 25628, 32034, 20851, 38190, 35789, 36825, 20123, 26680,
      24515, 25968, 25454,
    ])
    const rawDataBoundary = textFromCodes([
      24403, 21069, 26080, 26377, 25928, 21830, 21697, 25968, 25454,
      25110, 34920, 26684, 65292, 20165, 33021, 22522, 20110, 36890,
      29992, 30005, 21830, 22330, 26223, 32473, 20986, 21028, 26029,
    ])
    const rawKeywordReason = textFromCodes([
      38450, 26194, 34915, 26631, 39064, 28857, 20987, 29575, 20302,
      30340, 26680, 24515, 36890, 29992, 21407, 22240, 26159, 26631,
      39064, 32570, 22833, 29992, 25143, 39640, 20851, 27880, 30340,
      25628, 32034, 20851, 38190, 35789,
    ])
    const rawMetricGap = textFromCodes([
      32570, 23569, 26333, 20809, 37327, 12289, 28857, 20987, 37327,
      12289, 29992, 25143, 25628, 32034, 20851, 38190, 35789, 31561,
      26680, 24515, 25968, 25454, 65292, 26080, 27861, 20570, 31934,
      20934, 25351, 26631, 35745, 31639, 21644, 32467, 35770, 25512,
      23548,
    ])
    const rawQuestion = textFromCodes([
      26159, 21542, 26377, 35813, 38450, 26194, 34915, 30340, 24179,
      21488, 25628, 32034, 20851, 38190, 35789, 25968, 25454, 65292,
      29992, 20110, 34917, 20805, 26631, 39064, 30340, 39640, 28857,
      20987, 20851, 38190, 35789, 65311,
    ])
    const raw = [
      `${textFromCodes([20249, 20276])}，${textFromCodes([21681, 20204])}${naturalTitle}`,
      `### ${followupHeading}`,
      `- ${recordScope}`,
      `- ${metricCalculation}`,
      `- ${sampleBoundary}`,
      `- ${missingData}`,
      `- ${rawDataBoundary}`,
      `- ${rawKeywordReason}`,
      `- ${rawMetricGap}`,
      `- ${rawQuestion}`,
    ].join("\n")

    const formatted = formatAgentUiMessage(raw, copy)

    expect(formatted).toContain(naturalTitle)
    expect(formatted).toContain(recordScope)
    expect(formatted).toContain(metricCalculation)
    expect(formatted).toContain(missingData)
    expect(formatted).not.toContain(rawDataBoundary)
    expect(formatted).not.toContain(rawKeywordReason)
    expect(formatted).not.toContain(rawMetricGap)
    expect(formatted).not.toContain(rawQuestion)
  })
})

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}

function countOccurrences(value: string, search: string) {
  return value.split(search).length - 1
}
