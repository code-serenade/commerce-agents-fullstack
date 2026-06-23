import { includesAll, normalizeSectionHeading, stripListPrefix } from './agent-chat-answer-dedupe-helpers'
import {
  ADJUST_ONE_VARIABLE,
  BEST_OPTIMIZATION_DIRECTION_HEADING,
  CATEGORY_TRAFFIC_LOGIC_DIRECTION,
  CHANGE_TITLE_IMAGE_PRICE_TOGETHER,
  CLICK,
  CORE_NEED_WORDS,
  COVER_CORE_SEARCH_AND_SELLING_POINTS,
  CTR_LOW,
  CTR_LOW_POSSIBILITY_PRIORITY,
  CURRENT_MISSING_TITLE_PRICE_IMAGE_HISTORY,
  DATA_RECORDING_HEADING,
  DATA_REVIEW,
  DEDICATED_DATA_VERIFY,
  DEDICATED_IMPRESSION,
  DIRECT_NATURAL_TITLE_HEADING,
  EFFECT_TEST,
  EXECUTION_PLAN,
  FAKE_SELLING_POINTS,
  FIRST_CHANGE_HEADING,
  GENERAL_CATEGORY_LOGIC,
  GENERAL_LOGIC,
  GENERAL_NATURAL_TITLE_HEADING,
  GENERAL_SUNSCREEN_TITLE_CTR_LOGIC,
  IMAGE_SELLING_POINT_NOT_CLEAR,
  IMPRESSION,
  MISSING_INFO_TO_SUPPLEMENT,
  NATURAL_TITLE,
  NORMAL_CTR_RANGE,
  OPTIMIZATION_DIRECTION_HEADING,
  PRICE_OUT_OF_CATEGORY_RANGE,
  PRIORITIZE_ADJUST_TITLE,
  REAL_IMPRESSION,
  RECORD_CHANGE_CONTENT_AND_TIME,
  REVIEW_TEST_HEADING,
  RISK_AND_MISSING_INFO_HEADING,
  RISK_BOUNDARY_HEADING,
  SUNSCREEN_CATEGORY_HIGH_FREQ_SEARCH_WORDS,
  SUNSCREEN_CLOTHING,
  SUNSCREEN_ITEM_MISSING_DATA,
  SUNSCREEN_SEARCH_WORD_DIMENSIONS,
  SUPPLEMENT,
  THIS_TURN_NO_PRODUCT_DATA,
  TITLE_FRONT,
  TITLE_NOT_COVER_CORE_SEARCH,
} from './agent-chat-answer-dedupe-terms'

type RepeatedSectionTheme = "direction" | "title" | "test" | "risk"

interface RepeatedSection {
  lines: string[]
  theme: RepeatedSectionTheme | null
}

export function removeRepeatedSupplementInfoBullets(content: string) {
  if (!shouldDedupeSunscreenTitleSections(content)) return content
  if (!hasStructuredSunscreenExecutionAnswer(content)) return content

  let insideSupplementInfo = false
  return content
    .split("\n")
    .filter((line) => {
      const text = stripListPrefix(line)
      if (text.includes(MISSING_INFO_TO_SUPPLEMENT)) {
        insideSupplementInfo = true
        return true
      }
      if (!insideSupplementInfo) return true
      if (isRepeatedSunscreenSupplementSummary(text)) return false
      return true
    })
    .join("\n")
}

function hasStructuredSunscreenExecutionAnswer(content: string) {
  return (
    content.includes(EXECUTION_PLAN) ||
    content.includes(EFFECT_TEST) ||
    content.includes(DATA_REVIEW)
  ) && content.includes(RISK_BOUNDARY_HEADING)
}

function isRepeatedSunscreenSupplementSummary(value: string) {
  const text = value.trim()
  if (!text || text.endsWith("?") || text.endsWith("？")) return false
  const signals = [
    CURRENT_MISSING_TITLE_PRICE_IMAGE_HISTORY,
    CATEGORY_TRAFFIC_LOGIC_DIRECTION,
    CTR_LOW_POSSIBILITY_PRIORITY,
    TITLE_NOT_COVER_CORE_SEARCH,
    IMAGE_SELLING_POINT_NOT_CLEAR,
    PRICE_OUT_OF_CATEGORY_RANGE,
    SUNSCREEN_CATEGORY_HIGH_FREQ_SEARCH_WORDS,
    SUNSCREEN_SEARCH_WORD_DIMENSIONS,
    NORMAL_CTR_RANGE,
    PRIORITIZE_ADJUST_TITLE,
    COVER_CORE_SEARCH_AND_SELLING_POINTS,
    ADJUST_ONE_VARIABLE,
    RECORD_CHANGE_CONTENT_AND_TIME,
    CHANGE_TITLE_IMAGE_PRICE_TOGETHER,
    FAKE_SELLING_POINTS,
    THIS_TURN_NO_PRODUCT_DATA,
    GENERAL_CATEGORY_LOGIC,
  ].filter((term) => text.includes(term)).length
  return signals > 0
}

export function removeRepeatedUserFacingSections(content: string) {
  if (!shouldDedupeSunscreenTitleSections(content)) return content

  const sections = splitUserFacingSections(content.split("\n"))
  const lastThemeIndexes = new Map<RepeatedSectionTheme, number>()
  sections.forEach((section, index) => {
    if (section.theme) lastThemeIndexes.set(section.theme, index)
  })

  const duplicatedThemes = new Set<RepeatedSectionTheme>()
  sections.forEach((section, index) => {
    if (section.theme && lastThemeIndexes.get(section.theme) !== index) {
      duplicatedThemes.add(section.theme)
    }
  })
  if (duplicatedThemes.size === 0) return content

  return sections
    .filter((section, index) => {
      if (!section.theme) return true
      return lastThemeIndexes.get(section.theme) === index
    })
    .flatMap((section) => section.lines)
    .join("\n")
}

function splitUserFacingSections(lines: string[]) {
  const sections: RepeatedSection[] = []
  let current: RepeatedSection = { lines: [], theme: null }

  const flush = () => {
    if (current.lines.length === 0) return
    sections.push(current)
  }

  for (const line of lines) {
    const theme = readRepeatedSectionTheme(line)
    if (theme && current.lines.length > 0) {
      flush()
      current = { lines: [line], theme }
      continue
    }
    current.lines.push(line)
    current.theme = current.theme ?? theme
  }
  flush()

  return sections
}

export function shouldDedupeSunscreenTitleSections(content: string) {
  return content.includes(SUNSCREEN_CLOTHING) && content.includes(NATURAL_TITLE)
}

function readRepeatedSectionTheme(line: string): RepeatedSectionTheme | null {
  const text = normalizeSectionHeading(line)
  if (!text) return null
  if (
    text.includes(BEST_OPTIMIZATION_DIRECTION_HEADING) ||
    text.includes(OPTIMIZATION_DIRECTION_HEADING) ||
    text.includes(FIRST_CHANGE_HEADING)
  ) {
    return "direction"
  }
  if (
    text.includes(DIRECT_NATURAL_TITLE_HEADING) ||
    text.includes(GENERAL_NATURAL_TITLE_HEADING) ||
    text.includes(NATURAL_TITLE)
  ) {
    return "title"
  }
  if (
    text.includes(DATA_RECORDING_HEADING) ||
    text.includes(REVIEW_TEST_HEADING)
  ) {
    return "test"
  }
  if (
    text.includes(RISK_BOUNDARY_HEADING) ||
    text.includes(RISK_AND_MISSING_INFO_HEADING)
  ) {
    return "risk"
  }
  return null
}

export function normalizeRepeatedThemeKey(line: string) {
  const text = line.replace(/^[-*•]\s*/, "").trim()
  if (!text) return ""

  if (
    includesAll(text, [SUNSCREEN_ITEM_MISSING_DATA]) ||
    includesAll(text, [GENERAL_SUNSCREEN_TITLE_CTR_LOGIC])
  ) {
    return "sunscreen-missing-data"
  }
  if (
    includesAll(text, [CORE_NEED_WORDS, TITLE_FRONT]) ||
    includesAll(text, [CTR_LOW, CORE_NEED_WORDS])
  ) {
    return "sunscreen-title-core-need"
  }
  if (
    includesAll(text, [SUPPLEMENT, IMPRESSION, CLICK]) ||
    includesAll(text, [REAL_IMPRESSION, CLICK]) ||
    includesAll(text, [DEDICATED_IMPRESSION, CLICK])
  ) {
    return "sunscreen-follow-up-data"
  }
  if (
    includesAll(text, [GENERAL_LOGIC, DEDICATED_DATA_VERIFY]) ||
    includesAll(text, [DEDICATED_DATA_VERIFY])
  ) {
    return "sunscreen-data-boundary"
  }

  return ""
}
