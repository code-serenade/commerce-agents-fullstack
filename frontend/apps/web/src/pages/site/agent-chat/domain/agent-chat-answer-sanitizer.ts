import { normalizeUserFacingAnswer } from "./agent-chat-user-facing-normalizer"
import { restoreLegacyFallbackLayout } from "./agent-chat-legacy-fallback-layout"
import { isUnsupportedBenchmarkClaim } from "./agent-chat-benchmark-claims"
import { removeRepeatedUserFacingContent } from "./agent-chat-answer-dedupe"

export { isUnsupportedBenchmarkClaim } from "./agent-chat-benchmark-claims"

const INTERNAL_FAILURE_PATTERNS = [
  /data role should provide at least two findings/i,
  /data role should mark data or interpretation risks/i,
  /service role output lacks service-specific content/i,
  /service role should provide scripts, SOP, or escalation steps/i,
  /engineering role output lacks technical content/i,
  /engineering role should provide technical risks or rollback boundaries/i,
  /design role output lacks design-specific content/i,
  /design role should provide visual or page recommendations/i,
  /creative role output lacks creative-specific content/i,
  /creative role should provide usable creative directions/i,
  /accounting role output lacks financial content/i,
  /accounting role should provide budget or financial risks/i,
  /web role output lacks web or SEO-specific content/i,
  /web role should provide SEO, keyword, or conversion recommendations/i,
  /ops role should provide at least three recommendations/i,
  /structured role output is missing/i,
  /role output has open questions/i,
  /open questions remain/i,
  /worker reported failed status/i,
  /worker_unavailable/i,
  /llm_provider_error/i,
  /failure_kind/i,
]

const PRESERVED_BLANK_LINE = "__AGENT_CHAT_PRESERVED_BLANK_LINE__"

const INTERNAL_FAILURE_HEADINGS = terms([
  [26410, 23436, 25104, 33410, 28857],
  [26410, 23436, 25104, 25110, 38656, 22797, 26680],
  [26412, 36718, 27809, 26377, 24418, 25104, 23436, 25972, 22238, 31572],
])

const INTERNAL_FAILURE_TEXT_TERMS = terms([
  [37096, 20998, 23703, 20301, 27809, 26377, 25104, 21151, 25191, 34892],
  [26381, 21153, 31471, 27809, 26377, 21487, 29992, 30340, 35813, 23703, 20301, 25191, 34892, 22120],
  [19978, 28216, 20381, 36182, 23703, 20301, 26410, 25104, 21151, 23436, 25104],
  [23703, 20301, 25191, 34892, 22833, 36133],
  [35831, 26816, 26597, 26381, 21153, 31471, 23703, 20301, 37197, 32622],
])

const BOUNDARY_NOTICE_TERMS = terms([
  [19994, 21153, 36793, 30028],
  [26410, 36873, 21830, 21697],
  [26410, 25351, 23450, 21830, 21697],
  [19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697],
  [19981, 27880, 20837, 40664, 35748, 21830, 21697],
])

const LEGACY_FALLBACK_HEADING_REPLACEMENTS = [
  {
    from: cjk(24403, 21069, 21487, 20808, 20132, 20184),
    to: cjk(20808, 32473, 20320, 19968, 29256, 21487, 29992, 21028, 26029),
  },
  {
    from: cjk(20808, 20570, 20160, 20040),
    to: cjk(25509, 19979, 26469, 24590, 20040, 20570),
  },
  {
    from: cjk(26412, 36718, 36793, 30028),
    to: cjk(36824, 35201, 27880, 24847),
  },
]

export interface AgentAnswerSanitizerCopy {
  boundaryNotice: string
}

export function sanitizeAgentAnswerContent(
  content: string,
  copy?: AgentAnswerSanitizerCopy
) {
  const restoredFallback = restoreLegacyFallbackLayout(content)
  if (restoredFallback) return restoredFallback

  const lines = content
    .split("\n")
    .map((line) => {
      const wasBlankLine = !line.trim()
      const trimmedLine = line.trim()
      if (/^skill:[^\n]+/i.test(trimmedLine)) return ""
      const cleanedLine = line
        .replace(/[;；]?\s*skill:[^\n。；;]+(?:[。.;])?/gi, "")
        .trimEnd()
      if (wasBlankLine && !cleanedLine.trim()) return PRESERVED_BLANK_LINE
      return cleanedLine
    })

  const cleaned = removeInternalFailureBlock(lines, copy)
    .join("\n")
    .replaceAll(PRESERVED_BLANK_LINE, "")

  return normalizeLegacyFallbackHeadings(
    removeRepeatedUserFacingContent(normalizeUserFacingAnswer(cleaned))
  )
    .replace(/\n{3,}/g, "\n\n")
    .trim()
}

function normalizeLegacyFallbackHeadings(content: string) {
  return LEGACY_FALLBACK_HEADING_REPLACEMENTS.reduce(
    (current, replacement) =>
      current.replaceAll(`## ${replacement.from}`, `## ${replacement.to}`),
    content
  )
}

function removeInternalFailureBlock(
  lines: string[],
  copy?: AgentAnswerSanitizerCopy
) {
  const result: string[] = []
  let skippedInternalFailure = false
  let insideInternalFailureBlock = false

  for (const line of lines) {
    const trimmedLine = line.trim()

    if (isInternalFailureHeading(trimmedLine)) {
      insideInternalFailureBlock = true
      skippedInternalFailure = true
      continue
    }

    if (insideInternalFailureBlock) {
      if (trimmedLine.startsWith("#") && !isInternalFailureHeading(trimmedLine)) {
        insideInternalFailureBlock = false
      } else {
        skippedInternalFailure = true
        continue
      }
    }

    if (hasInternalFailureText(trimmedLine)) {
      skippedInternalFailure = true
      continue
    }

    if (isUnsupportedBenchmarkClaim(trimmedLine)) {
      continue
    }

    result.push(line)
  }

  if (!skippedInternalFailure) return result.filter(hasVisibleUserFacingLine)

  const output = result.filter(hasVisibleUserFacingLine)
  if (!hasBoundaryNotice(output.join("\n"))) {
    output.push(
      copy?.boundaryNotice ||
        "Some roles hit a business boundary. The interface treats it as reviewable context instead of a user-facing failure."
    )
  }
  return output
}

function hasVisibleUserFacingLine(line: string) {
  return line === PRESERVED_BLANK_LINE || Boolean(line.trim())
}

function isInternalFailureHeading(value: string) {
  const heading = value
    .replace(/^#{0,6}\s*/, "")
    .replace(/[:：]\s*$/, "")
    .trim()
  return INTERNAL_FAILURE_HEADINGS.includes(heading)
}

function hasInternalFailureText(value: string) {
  if (!value) return false
  if (/^-\s*.+\(.+\)\s*failed:/i.test(value)) return true
  if (/^-\s*(?:data|ops|creative|service|engineering|accounting|design|web|search|chat)\s*[:：]/i.test(value)) return true
  if (containsAnyTerm(value, INTERNAL_FAILURE_TEXT_TERMS)) return true
  return INTERNAL_FAILURE_PATTERNS.some((pattern) => pattern.test(value))
}

function hasBoundaryNotice(value: string) {
  return containsAnyTerm(value, BOUNDARY_NOTICE_TERMS)
}

function containsAnyTerm(value: string, termsToFind: string[]) {
  return termsToFind.some((term) => value.includes(term))
}

function terms(codePointGroups: number[][]) {
  return codePointGroups.map((codes) => cjk(...codes))
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
