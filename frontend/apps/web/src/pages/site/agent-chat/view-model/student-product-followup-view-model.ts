import type { TFunction } from "i18next"

export function getContextualProductFollowupPrompts(t: TFunction, text: string) {
  const context = getProductFollowupContext(text, t)
  if (!context.hasSpecificSignal) return []

  return [
    t("agentChat.thread.nextActions.contextualProductFollowup.mainImage", context),
    t("agentChat.thread.nextActions.contextualProductFollowup.title", context),
    t("agentChat.thread.nextActions.contextualProductFollowup.value", context),
    t("agentChat.thread.nextActions.contextualProductFollowup.validation", context),
  ]
}

export function matchesProductFollowupIntent(text: string) {
  return /product|sku|item|\u5546\u54c1|\u9009\u54c1|\u5356\u70b9|\u4eba\u7fa4|\u6807\u9898|\u4e3b\u56fe|\u5b9a\u4ef7|\u4ef7\u683c|\u52a0\u8d2d|\u70b9\u51fb|\u9632\u6652\u8863|\u5927\u5b66\u751f|\u6bdb\u5229|\u5229\u6da6|\u7ade\u54c1|\u5bf9\u6807|\u6d4b\u6b3e|\u8f6c\u5316|\u5c0f\u6d4b/i.test(text)
}

export function hasConcreteProductFollowupSignal(text: string) {
  return Boolean(
    extractProductSubject(text) ||
      extractFirstMatch(text, /(\d+(?:\.\d+)?\s*\u5143)/i) ||
      extractAudience(text) ||
      /\u4e3b\u56fe|\u9996\u56fe|\u56fe\u7247|\u89c6\u89c9|\u6807\u9898|\u5173\u952e\u8bcd|\u4ef7\u683c|\u5b9a\u4ef7|\u552e\u4ef7|\u5229\u6da6|\u6bdb\u5229|\u7ade\u54c1|\u5bf9\u6807|\u6d4b\u6b3e|\u8f6c\u5316|\u70b9\u51fb|\u5c0f\u6d4b/i.test(text)
  )
}

function getProductFollowupContext(text: string, t: TFunction) {
  const subject = extractProductSubject(text)
  const price = extractFirstMatch(text, /(\d+(?:\.\d+)?\s*\u5143)/i)
  const audience = extractAudience(text)
  const issue = getProductIssue(text, t)

  return {
    audience: audience || t("agentChat.thread.nextActions.contextFallback.audience"),
    hasSpecificSignal: Boolean(subject || price || audience || issue),
    issue: issue || t("agentChat.thread.nextActions.contextFallback.issue"),
    price: price || t("agentChat.thread.nextActions.contextFallback.price"),
    subject: subject || t("agentChat.thread.nextActions.contextFallback.subject"),
  }
}

function extractProductSubject(text: string) {
  return (
    extractSpecificSubject(
      text,
      /(\u5b66\u751f\u5bbf\u820d\u7528\u684c\u9762\u6536\u7eb3\u76d2|\u684c\u9762\u6536\u7eb3\u76d2|\u6536\u7eb3\u76d2|\u590f\u5b63\u9632\u6652\u8863|\u9632\u6652\u8863|\u5496\u5561|\u8fde\u8863\u88d9|\u53cc\u80a9\u5305|\u6c34\u676f)/i
    ) ||
    extractSpecificSubject(
      text,
      /(?:\u5546\u54c1|\u4ea7\u54c1)\s*[:：]\s*([^，。,.；;\n]{2,24})/i
    ) ||
    extractSpecificSubject(
      text,
      /(?:\u5356|\u505a|\u4e00\u6b3e|\u8fd9\u6b3e)\s*([^，。,.；;\n]{2,24}?)(?:[，。,.；;\s]*(?:\u4ef7\u683c|\u552e\u4ef7|\u5b9a\u4ef7|\u9762\u5411|\u4e3b\u56fe|\u6807\u9898)|$)/i
    )
  )
}

function extractAudience(text: string) {
  const explicitAudience = extractFirstMatch(
    text,
    /(?:\u9762\u5411|\u76ee\u6807\u4eba\u7fa4|\u4eba\u7fa4|\u9002\u5408)\s*[:：]?\s*([^，。,.；;\n]{2,18})/i
  )
  if (explicitAudience && !isGenericExtractedValue(explicitAudience)) {
    return explicitAudience
  }
  return extractFirstMatch(
    text,
    /(\u5927\u5b66\u751f|\u5b66\u751f\u515a|\u5b9d\u5988|\u767d\u9886|\u901a\u52e4\u4eba\u7fa4|\u5e74\u8f7b\u5973\u6027|\u65b0\u624b\u5546\u5bb6)/i
  )
}

function getProductIssue(text: string, t: TFunction) {
  if (/\u4e3b\u56fe|\u9996\u56fe|\u56fe\u7247|\u89c6\u89c9/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.mainImage")
  }
  if (/\u6807\u9898|\u5173\u952e\u8bcd/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.title")
  }
  if (/\u4ef7\u683c|\u5b9a\u4ef7|\u552e\u4ef7|\u5229\u6da6|\u6bdb\u5229/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.price")
  }
  if (/\u7ade\u54c1|\u5bf9\u6807|\u540c\u884c|\u7ade\u4e89/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.competition")
  }
  if (/\u6d4b\u6b3e|\u8f6c\u5316|\u70b9\u51fb|\u52a0\u8d2d|\u5c0f\u6d4b|\u9a8c\u8bc1/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.validation")
  }
  if (/\u4eba\u7fa4|\u5927\u5b66\u751f|\u5b66\u751f|\u7528\u6237/i.test(text)) {
    return t("agentChat.thread.nextActions.contextIssue.audience")
  }
  return ""
}

function extractFirstMatch(text: string, pattern: RegExp) {
  return normalizeExtractedValue(pattern.exec(text)?.[1])
}

function extractSpecificSubject(text: string, pattern: RegExp) {
  const value = extractFirstMatch(text, pattern)
  if (!value || isGenericExtractedValue(value)) return ""
  return value
}

function normalizeExtractedValue(value?: string) {
  return value
    ?.trim()
    .replace(/^[：:，,。.\s]+|[：:，,。.\s]+$/g, "")
    .slice(0, 24)
}

function isGenericExtractedValue(value: string) {
  return /\u5206\u6790\u5bf9\u8c61|\u76ee\u6807\u52a8\u4f5c|\u5f53\u524d\u53ef\u7528\u4fe1\u606f|\u5173\u952e\u5224\u65ad|\u8fd9\u4e2a\u7ed3\u8bba|\u5f53\u524d\u56de\u7b54|\u4e0b\u4e00\u6b65|\u6700\u503c\u5f97|\u91cd\u505a\u4e00\u7248|\u7684\u662f/.test(value)
}
