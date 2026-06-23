import { includesAll, normalizeSectionHeading, stripListPrefix } from './agent-chat-answer-dedupe-helpers'
import { shouldDedupeSunscreenTitleSections } from './agent-chat-answer-dedupe-sunscreen-sections'
import {
  AT_LEAST_1000_IMPRESSIONS,
  CANNOT_FABRICATE_CONVERSION_OR_RANKING,
  COMPLIANCE_RISK,
  CTR_FACTOR_PRIORITY,
  CTR_LIFT_CANNOT_GUARANTEE,
  CURRENT_LIMIT_AND_NEXT_PRACTICE,
  CURRENT_MISSING_TITLE_IMAGE_TRAFFIC_PRICE_PLATFORM,
  CURRENT_NO_PRODUCT_TITLE_IMAGE_PRICE,
  CURRENTLY_NO_VALID_PRODUCT_DATA,
  DEDICATED_DATA_VERIFY,
  DIRECT_GENERAL_NATURAL_TITLE,
  FIRST_PRIORITY_CHANGE,
  FOLLOWUP_SUPPLEMENT_FIELDS_HEADING,
  GENERAL_TITLE_COVERS_CORE_NEEDS,
  GENERAL_TITLE_FIT_HAS_BIAS,
  GENERAL_SUNSCREEN_OPS_DIRECTION,
  GENERAL_SUNSCREEN_TITLE_CTR_LOGIC,
  HAS_PLATFORM_SEARCH_KEYWORDS,
  IMAGE_SELLING_POINT_CLARITY,
  METRIC_CALCULATION,
  MISSING_DATA,
  NO_IMPRESSION_CLICK_TRAFFIC_DATA,
  PRECISE_METRIC_CALCULATION_UNAVAILABLE,
  PRICE_COMPETITIVENESS,
  PRIORITIZE_TITLE_OPTIMIZATION,
  REAL_FUNNEL_CHANNEL_DATA,
  RECORD_FIELDS_AND_CALCULATION,
  RECORD_SCOPE,
  REFERENCE_NATURAL_TITLE,
  RISK_BOUNDARY_HEADING,
  SAMPLE_BOUNDARY,
  SUPPLEMENT_CORE_INFO_PERSONALIZE,
  SUPPLEMENT_ORIGINAL_TITLE_IMAGE_PRICE_TRAFFIC,
  SUPPLEMENT_SUNSCREEN_CORE_SEARCH_KEYWORDS,
  SUNSCREEN_USER_CORE_SEARCH_NEEDS,
  TEST_GENERAL_TITLE_CTR_CHANGE,
  TITLE_IMAGE_MATCH_FIRST,
  TITLE_KEYWORD_MATCH,
  TITLE_MISSING_SEARCH_KEYWORDS,
  TITLE_SUNSCREEN_INDEX_MISMATCH,
  NEXT_OPTION_MORE_ACCURATE,
} from './agent-chat-answer-dedupe-terms'

export function truncateRawRoleOutputTail(content: string) {
  const lines = content.split("\n")
  const keepUntil = findRawRoleOutputTailStart(lines)
  if (keepUntil === null) return content
  return lines
    .slice(0, keepUntil)
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim()
}

export function truncateRepeatedSunscreenSummaryTail(content: string) {
  if (!shouldDedupeSunscreenTitleSections(content)) return content

  const text = content.trim()
  if (!hasCompleteSunscreenAnswerBeforeTail(text)) return content

  const tailStart = findRepeatedSunscreenSummaryTailStart(text)
  if (tailStart === null) return content
  return text.slice(0, tailStart).replace(/\n{3,}/g, "\n\n").trim()
}

function hasCompleteSunscreenAnswerBeforeTail(content: string) {
  const legacyStructureSignals = [
    FIRST_PRIORITY_CHANGE,
    REFERENCE_NATURAL_TITLE,
    RECORD_FIELDS_AND_CALCULATION,
    SAMPLE_BOUNDARY,
    CURRENT_LIMIT_AND_NEXT_PRACTICE,
  ].filter((term) => content.includes(term)).length

  const edgeVisibleStructureSignals = [
    DIRECT_GENERAL_NATURAL_TITLE,
    RISK_BOUNDARY_HEADING,
    NEXT_OPTION_MORE_ACCURATE,
  ].filter((term) => content.includes(term)).length

  return legacyStructureSignals >= 3 || edgeVisibleStructureSignals >= 2
}

function findRepeatedSunscreenSummaryTailStart(content: string) {
  const starts = [
    CURRENT_NO_PRODUCT_TITLE_IMAGE_PRICE,
    CURRENT_MISSING_TITLE_IMAGE_TRAFFIC_PRICE_PLATFORM,
    CTR_FACTOR_PRIORITY,
    PRIORITIZE_TITLE_OPTIMIZATION,
  ]
    .map((term) => content.indexOf(term))
    .filter((index) => index >= 0)
    .sort((a, b) => a - b)

  for (const index of starts) {
    if (!looksLikeRepeatedSunscreenSummaryTail(content.slice(index))) continue
    return index
  }
  return null
}

function looksLikeRepeatedSunscreenSummaryTail(value: string) {
  const signals = [
    CURRENT_NO_PRODUCT_TITLE_IMAGE_PRICE,
    NO_IMPRESSION_CLICK_TRAFFIC_DATA,
    GENERAL_SUNSCREEN_OPS_DIRECTION,
    CTR_FACTOR_PRIORITY,
    TITLE_KEYWORD_MATCH,
    IMAGE_SELLING_POINT_CLARITY,
    PRICE_COMPETITIVENESS,
    REAL_FUNNEL_CHANNEL_DATA,
    CANNOT_FABRICATE_CONVERSION_OR_RANKING,
    PRIORITIZE_TITLE_OPTIMIZATION,
    SUPPLEMENT_SUNSCREEN_CORE_SEARCH_KEYWORDS,
    AT_LEAST_1000_IMPRESSIONS,
    SUPPLEMENT_ORIGINAL_TITLE_IMAGE_PRICE_TRAFFIC,
    CURRENT_MISSING_TITLE_IMAGE_TRAFFIC_PRICE_PLATFORM,
    SUNSCREEN_USER_CORE_SEARCH_NEEDS,
    GENERAL_TITLE_COVERS_CORE_NEEDS,
    TEST_GENERAL_TITLE_CTR_CHANGE,
    SUPPLEMENT_CORE_INFO_PERSONALIZE,
    TITLE_IMAGE_MATCH_FIRST,
    GENERAL_TITLE_FIT_HAS_BIAS,
    CTR_LIFT_CANNOT_GUARANTEE,
    TITLE_SUNSCREEN_INDEX_MISMATCH,
    COMPLIANCE_RISK,
  ].filter((term) => value.includes(term)).length

  return (
    signals >= 4 ||
    includesAll(value, [
      CURRENT_NO_PRODUCT_TITLE_IMAGE_PRICE,
      NO_IMPRESSION_CLICK_TRAFFIC_DATA,
      CTR_FACTOR_PRIORITY,
    ]) ||
    includesAll(value, [
      REAL_FUNNEL_CHANNEL_DATA,
      CANNOT_FABRICATE_CONVERSION_OR_RANKING,
      PRIORITIZE_TITLE_OPTIMIZATION,
    ]) ||
    includesAll(value, [
      CURRENT_MISSING_TITLE_IMAGE_TRAFFIC_PRICE_PLATFORM,
      SUNSCREEN_USER_CORE_SEARCH_NEEDS,
      GENERAL_TITLE_COVERS_CORE_NEEDS,
    ]) ||
    includesAll(value, [
      GENERAL_TITLE_FIT_HAS_BIAS,
      CTR_LIFT_CANNOT_GUARANTEE,
      TITLE_SUNSCREEN_INDEX_MISMATCH,
    ])
  )
}

function findRawRoleOutputTailStart(lines: string[]) {
  const followupIndex = lines.findIndex((line) =>
    normalizeSectionHeading(line).includes(FOLLOWUP_SUPPLEMENT_FIELDS_HEADING)
  )
  if (followupIndex < 0) return null

  const tailStart = lines.findIndex(
    (line, index) =>
      index > followupIndex &&
      looksLikeRawRoleOutputTailItem(stripListPrefix(line))
  )
  if (tailStart < 0) return null
  if (!hasStructuredFollowupFields(lines.slice(followupIndex, tailStart))) {
    return null
  }

  const tailSignals = lines
    .slice(tailStart)
    .filter((line) => looksLikeRawRoleOutputTailItem(stripListPrefix(line)))
    .length
  return tailSignals >= 3 ? tailStart : null
}

function hasStructuredFollowupFields(lines: string[]) {
  const text = lines.map(stripListPrefix).join("\n")
  return [
    RECORD_SCOPE,
    METRIC_CALCULATION,
    SAMPLE_BOUNDARY,
    MISSING_DATA,
  ].filter((term) => text.includes(term)).length >= 2
}

function looksLikeRawRoleOutputTailItem(line: string) {
  const text = line.trim()
  if (!text) return false
  return [
    CURRENTLY_NO_VALID_PRODUCT_DATA,
    TITLE_MISSING_SEARCH_KEYWORDS,
    PRECISE_METRIC_CALCULATION_UNAVAILABLE,
    HAS_PLATFORM_SEARCH_KEYWORDS,
    GENERAL_SUNSCREEN_TITLE_CTR_LOGIC,
    DEDICATED_DATA_VERIFY,
  ].some((term) => text.includes(term))
}
