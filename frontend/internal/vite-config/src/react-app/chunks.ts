export function createManualChunkName(id: string) {
  if (!id.includes("node_modules")) return undefined

  if (
    id.includes("/node_modules/react/") ||
    id.includes("/node_modules/react-dom/") ||
    id.includes("/node_modules/scheduler/")
  ) {
    return "vendor-react-core"
  }

  if (id.includes("sonner")) return "vendor-toast"
  if (id.includes("@tanstack/react-query")) return "vendor-query"
  if (
    id.includes("i18next") ||
    id.includes("react-i18next") ||
    id.includes("i18next-browser-languagedetector")
  ) {
    return "vendor-i18n"
  }
  if (
    id.includes("@reduxjs/toolkit") ||
    id.includes("react-redux") ||
    id.includes("redux")
  ) {
    return "vendor-redux"
  }
  if (id.includes("react-router")) return "vendor-router"
  if (
    id.includes("@radix-ui/") ||
    id.includes("@floating-ui/") ||
    id.includes("@base-ui/") ||
    id.includes("input-otp") ||
    id.includes("cmdk") ||
    id.includes("embla-carousel") ||
    id.includes("vaul") ||
    id.includes("lucide-react")
  ) {
    return "vendor-ui"
  }
  if (
    id.includes("react-hook-form") ||
    id.includes("@hookform/resolvers") ||
    id.includes("zod")
  ) {
    return "vendor-forms"
  }
  if (id.includes("react")) return "vendor-react-ecosystem"

  return "vendor-misc"
}
