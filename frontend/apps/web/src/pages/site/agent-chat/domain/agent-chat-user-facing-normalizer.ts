import { normalizeLegacyProcessCopyLine } from "./agent-chat-legacy-process-copy"

const NOTE_PREFIX = cjk(22791, 27880, 65306)
const NOTE_ASCII_PREFIX = cjk(22791, 27880, 58)
const EXPLANATION_PREFIX = cjk(35828, 26126, 65306)
const PLACEHOLDER = cjk(21344, 20301, 31526)
const NEEDS_REPLACE = cjk(38656, 26367, 25442)
const REPLACE_WITH_REAL = cjk(26367, 25442, 20026, 30495, 23454)
const REAL_CATEGORY = cjk(30495, 23454, 21697, 31867)
const REAL_SPEC = cjk(30495, 23454, 35268, 26684)
const REAL_PRICE = cjk(30495, 23454, 20215, 26684)
const REAL_SALE_PRICE = cjk(30495, 23454, 21806, 20215)
const ACTUAL_PRICE = cjk(21040, 25163, 20215)
const PENDING_FILL = cjk(24453, 22635, 20889)
const PENDING_SUPPLEMENT = cjk(24453, 34917, 20805)
const INTERNAL_GRAPH = cjk(20219, 21153, 22270)
const INTERNAL_REPORT = cjk(23703, 20301, 25253, 21578)
const TASK_DONE = cjk(20219, 21153, 24050, 23436, 25104)
const SUPPORT_DONE = cjk(25903, 25345, 35282, 33394, 24050, 23436, 25104)
const CJK_PAUSE = cjk(12289)
const CJK_PERIOD = cjk(12290)
const CJK_COLON = cjk(65306)
const CJK_SEMICOLON = cjk(65307)
const CJK_COMMA = cjk(65292)
const CJK_EXCLAMATION = cjk(65281)
const CJK_QUESTION = cjk(65311)
const THINKING = cjk(27491, 22312, 24605, 32771, 20013)
const MISSING_USER_QUESTION = cjk(29992, 25143, 26410, 25552, 20379, 20855, 20307, 38382, 39064)
const LEGACY_INCOMPLETE_ANSWER = cjk(26412, 36718, 27809, 26377, 24418, 25104, 23436, 25972, 22238, 31572)
const PARTIAL_ROLE_FAILURE = cjk(37096, 20998, 23703, 20301, 27809, 26377, 25104, 21151, 25191, 34892)
const ROLE_EXECUTOR_UNAVAILABLE = cjk(26381, 21153, 31471, 27809, 26377, 21487, 29992, 30340, 35813, 23703, 20301, 25191, 34892, 22120)
const UPSTREAM_ROLE_INCOMPLETE = cjk(19978, 28216, 20381, 36182, 23703, 20301, 26410, 25104, 21151, 23436, 25104)
const CHECK_SERVER_ROLE_CONFIG = cjk(35831, 26816, 26597, 26381, 21153, 31471, 23703, 20301, 37197, 32622)
const ROLE_EXECUTION_FAILED = cjk(23703, 20301, 25191, 34892, 22833, 36133)
const UPSTREAM_ROLE_OUTPUT = cjk(19978, 28216, 23703, 20301, 36755, 20986)
const EXISTING_INFO = cjk(24050, 26377, 20449, 24687)
const COLLABORATION_STEP = cjk(21327, 20316, 27493, 39588)
const PROCESS_STEP = cjk(22788, 29702, 27493, 39588)
const INTERNAL_FAILURE_DETAIL = cjk(20869, 37096, 22833, 36133, 32454, 33410)
const INTERNAL_FAILURE_EXPOSURE_AVOIDED = cjk(24050, 36991, 20813, 26292, 38706, 20869, 37096, 22833, 36133, 32454, 33410)

const PLACEHOLDER_PHRASES = [
  cjk(65288, 21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687, 65289),
  cjk(40, 21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687, 41),
  cjk(12304, 21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687, 12305),
  cjk(91, 21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687, 93),
  cjk(21344, 20301, 31526, 38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687),
  cjk(21344, 20301, 31526, 38656, 26367, 25442),
  cjk(38656, 26367, 25442, 20026, 30495, 23454, 20449, 24687),
  cjk(35831, 26367, 25442, 20026, 30495, 23454, 20449, 24687),
  "placeholder",
  "PLACEHOLDER",
]

export function normalizeUserFacingAnswer(content: string) {
  return content
    .split("\n")
    .map(cleanUserFacingLine)
    .map(normalizeLegacyProcessCopyLine)
    .filter((line) => {
      const trimmedLine = line.trim()
      return line === "" || (Boolean(trimmedLine) && !isInternalNoiseLine(trimmedLine))
    })
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
}

function cleanUserFacingLine(line: string) {
  const withoutRole = stripRolePrefix(line.trim())
  const withoutPlaceholders = removePlaceholderPhrases(
    removePlaceholderBrackets(withoutRole)
  )
  const rewrittenInternalTerms = withoutPlaceholders
    .replaceAll(UPSTREAM_ROLE_OUTPUT, EXISTING_INFO)
    .replaceAll(COLLABORATION_STEP, PROCESS_STEP)
  const rewrittenNote = rewrittenInternalTerms
    .replaceAll(NOTE_PREFIX, EXPLANATION_PREFIX)
    .replaceAll(NOTE_ASCII_PREFIX, EXPLANATION_PREFIX)
  const compacted = compactLineSpacing(rewrittenNote)
  return isEmptyListArtifact(compacted) ? "" : compacted
}

function stripRolePrefix(line: string) {
  const { prefix, body } = splitListPrefix(line)
  const colon = firstColon(body)
  if (!colon) return line.trim()

  const label = body.slice(0, colon.index).trim()
  if (!looksLikeInternalRoleLabel(label)) return line.trim()

  return `${prefix}${body.slice(colon.index + colon.width).trimStart()}`.trim()
}

function splitListPrefix(line: string) {
  const match = line.match(/^(\s*(?:[-*•]\s+))(.*)$/)
  if (!match) return { prefix: "", body: line }
  return { prefix: match[1] ?? "", body: match[2] ?? "" }
}

function firstColon(value: string) {
  const colonIndex = value.search(/[:：]/)
  if (colonIndex < 0) return null
  return { index: colonIndex, width: 1 }
}

function looksLikeInternalRoleLabel(label: string) {
  const lower = label.toLowerCase()
  if (isInternalRuntimeRoleLabel(lower)) return true
  if (
    lower.startsWith("primary ") ||
    lower.startsWith("support ") ||
    lower.startsWith("worker.") ||
    lower.startsWith("role ")
  ) {
    return true
  }

  const match = label.match(/\(([^)]+)\)/)
  if (!match) return false
  return /^[a-z0-9_.-]+$/.test(match[1] ?? "")
}

function isInternalRuntimeRoleLabel(label: string) {
  const normalized = label.trim().replace(/_final$/i, "")
  return [
    "chat",
    "ops",
    "data",
    "creative",
    "service",
    "engineering",
    "accounting",
    "design",
    "web",
    "search",
  ].includes(normalized)
}

function removePlaceholderBrackets(input: string) {
  return replacePlaceholderPairs(
    replacePlaceholderPairs(input, "【", "】"),
    "[",
    "]"
  )
}

function replacePlaceholderPairs(input: string, open: string, close: string) {
  let output = ""
  let rest = input

  while (true) {
    const start = rest.indexOf(open)
    if (start < 0) return output + rest

    output += rest.slice(0, start)
    const segmentStart = start + open.length
    const end = rest.indexOf(close, segmentStart)
    if (end < 0) return output + rest.slice(start)

    const segment = rest.slice(segmentStart, end)
    if (isPlaceholderSegment(segment)) {
      const replacement = placeholderReplacement(segment)
      if (replacement) output += replacement
    } else {
      output += `${open}${segment}${close}`
    }
    rest = rest.slice(end + close.length)
  }
}

function isPlaceholderSegment(segment: string) {
  const lower = segment.toLowerCase()
  return (
    segment.includes(PLACEHOLDER) ||
    segment.includes(NEEDS_REPLACE) ||
    segment.includes(REPLACE_WITH_REAL) ||
    segment.includes(REAL_CATEGORY) ||
    segment.includes(REAL_SPEC) ||
    segment.includes(REAL_PRICE) ||
    segment.includes(REAL_SALE_PRICE) ||
    segment.includes(ACTUAL_PRICE) ||
    segment.includes(PENDING_FILL) ||
    segment.includes(PENDING_SUPPLEMENT) ||
    lower.includes("placeholder") ||
    lower.includes("replace")
  )
}

function placeholderReplacement(segment: string) {
  let text = removePlaceholderPhrases(segment)
  for (const delimiter of [cjk(65288), "(", cjk(65292), ",", cjk(22914)]) {
    const index = text.indexOf(delimiter)
    if (index >= 0) text = text.slice(0, index)
  }
  text = text
    .replace(/\//g, "、")
    .trim()
    .replace(new RegExp(`[${CJK_COLON}${CJK_PERIOD}${CJK_PAUSE}:\\.\\s]+$`, "g"), "")

  if (
    !text ||
    text.includes("...") ||
    text.includes("…") ||
    text.includes(PLACEHOLDER) ||
    !isPlaceholderFieldLabel(text) ||
    Array.from(text).length > 24
  ) {
    return ""
  }
  return text
}

function isPlaceholderFieldLabel(text: string) {
  return [
    cjk(30495, 23454),
    cjk(35268, 26684),
    ACTUAL_PRICE,
    cjk(21806, 20215),
    cjk(20215, 26684),
    cjk(23383, 27573),
  ].some((term) => text.includes(term))
}

function removePlaceholderPhrases(input: string) {
  let text = input
  for (const phrase of PLACEHOLDER_PHRASES) {
    text = text.replaceAll(phrase, "")
  }
  return text
}

function compactLineSpacing(line: string) {
  return line
    .split(/\s+/)
    .join(" ")
    .trim()
    .replace(
      new RegExp(
        `\\s+([${CJK_COMMA}${CJK_PERIOD}${CJK_COLON}${CJK_SEMICOLON}${CJK_PAUSE}${CJK_EXCLAMATION}${CJK_QUESTION}])`,
        "g"
      ),
      "$1"
    )
}

function isInternalNoiseLine(line: string) {
  const lower = line.toLowerCase()
  return (
    [
      "task graph completed",
      "passed worker reports",
      "incomplete nodes",
      "task graph",
      "role_output",
      "final_report",
      "cognition failure",
      "json template validation failed",
      "missing key",
      "worker reported failed status",
      "worker_unavailable",
      "llm_provider_error",
      "failure_kind",
      "support role",
      "support worker",
      "support_workers",
      "task_id",
      "raw=",
      "fast-route",
    ].some((marker) => lower.includes(marker)) ||
    line.includes(INTERNAL_GRAPH) ||
    line.includes(INTERNAL_REPORT) ||
    line.includes(LEGACY_INCOMPLETE_ANSWER) ||
    line.includes(PARTIAL_ROLE_FAILURE) ||
    line.includes(ROLE_EXECUTOR_UNAVAILABLE) ||
    line.includes(UPSTREAM_ROLE_INCOMPLETE) ||
    line.includes(CHECK_SERVER_ROLE_CONFIG) ||
    line.includes(ROLE_EXECUTION_FAILED) ||
    line.includes(INTERNAL_FAILURE_DETAIL) ||
    line.includes(INTERNAL_FAILURE_EXPOSURE_AVOIDED) ||
    line.startsWith(TASK_DONE) ||
    line.startsWith(SUPPORT_DONE) ||
    line.includes(THINKING) ||
    line.includes(MISSING_USER_QUESTION) ||
    line.includes(PLACEHOLDER) ||
    lower.includes(" worker") ||
    lower.startsWith("worker") ||
    isEmptyListArtifact(line)
  )
}

function isEmptyListArtifact(line: string) {
  return /^[-*•]\s*[:：]?$/.test(line.trim())
}

function cjk(...codes: number[]) {
  return String.fromCodePoint(...codes)
}
