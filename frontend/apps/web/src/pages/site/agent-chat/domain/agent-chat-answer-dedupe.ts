import { normalizeRepeatedThemeKey, removeRepeatedSupplementInfoBullets, removeRepeatedUserFacingSections } from './agent-chat-answer-dedupe-sunscreen-sections'
import { truncateRawRoleOutputTail, truncateRepeatedSunscreenSummaryTail } from './agent-chat-answer-dedupe-sunscreen-tail'

export function removeRepeatedUserFacingContent(content: string) {
  return removeRepeatedUserFacingBullets(
    truncateRepeatedSunscreenSummaryTail(
      truncateRawRoleOutputTail(removeRepeatedUserFacingSections(content))
    )
  )
}

function removeRepeatedUserFacingBullets(content: string) {
  const seenStatements = new Set<string>()
  const seenThemes = new Set<string>()
  return removeRepeatedSupplementInfoBullets(content)
    .split("\n")
    .filter((line) => {
      const trimmedLine = line.trim()
      const key = normalizeRepeatedLineKey(trimmedLine)
      const theme = normalizeRepeatedThemeKey(trimmedLine)
      if (theme && seenThemes.has(theme)) return false
      if (seenStatements.has(key)) return false
      if (theme) seenThemes.add(theme)
      if (!key) return true
      seenStatements.add(key)
      return true
    })
    .join("\n")
}

function normalizeRepeatedLineKey(line: string) {
  const text = line
    .replace(/^[-*•]\s*/, "")
    .replace(/^#+\s*/, "")
    .replace(/\s+/g, "")
    .trim()
  if (text.length < 10) return ""
  return text.replace(/[。；;,.，：:、]/g, "")
}
