const PRODUCT_STATUS_KEYS = new Set(["active", "draft", "listing", "archived"])

export function productStatusTranslationKey(status: string) {
  const normalized = status.trim().toLowerCase()
  return `agentChat.composer.productContext.status.${
    PRODUCT_STATUS_KEYS.has(normalized) ? normalized : "unknown"
  }`
}
