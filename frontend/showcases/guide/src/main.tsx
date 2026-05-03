import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import "./index.css"
import "@workspace/locales/i18n"
import App from "./App.tsx"
import { initTheme } from "@workspace/ui-theme"
import { applyThemeOverrides, loadThemeOverrides } from "@/theme/themeOverrides"

async function bootstrap() {
  const rootElement = document.getElementById("root")
  if (!rootElement) throw new Error("Root element #root not found")

  initTheme()
  const themeOverrides = loadThemeOverrides()
  if (themeOverrides) {
    applyThemeOverrides(themeOverrides)
  }

  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>
  )
}

void bootstrap()
