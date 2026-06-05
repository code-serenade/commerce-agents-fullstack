export function hasDisplayableAnswer(content: string) {
  return content.trim().length > 0
}

export function isAbortLikeStreamError(error: unknown) {
  const message =
    error instanceof Error
      ? `${error.name} ${error.message}`
      : typeof error === "string"
        ? error
        : ""
  return (
    /\b(abort|aborted|aborterror|terminated)\b/i.test(message) ||
    /\bconnection\s+(closed|reset|lost|interrupted)\b/i.test(message) ||
    /\bnetwork\s+(error|failure|failed|interrupted)\b/i.test(message) ||
    /\bfailed to fetch\b/i.test(message)
  )
}
