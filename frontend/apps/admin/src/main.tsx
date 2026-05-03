import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { Provider } from "react-redux"
import { QueryClientProvider } from "@tanstack/react-query"
import { store } from "./store"
import "./index.css"
import "@workspace/locales/i18n"
import { getRequestErrorMessage } from "@workspace/services/errors/error-message"
import { toRequestError } from "@workspace/services/errors/request-error"
import { queryClient } from "@workspace/services/query/client"
import { setHttpClientErrorHandler } from "@workspace/services/http-client"
import { initTheme } from "@workspace/ui-theme"
import { registerAdminLocaleMessages } from "../lang"
import { setRequestErrorMessage } from "./store/requestErrorSlice"
import App from "./App.tsx"

async function bootstrap() {
  const rootElement = document.getElementById("root")
  if (!rootElement) throw new Error("Root element #root not found")

  registerAdminLocaleMessages()
  initTheme()
  setHttpClientErrorHandler((error) => {
    const message = getRequestErrorMessage(toRequestError(error))
    store.dispatch(setRequestErrorMessage(message))
  })

  createRoot(rootElement).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <Provider store={store}>
          <App />
        </Provider>
      </QueryClientProvider>
    </StrictMode>
  )
}

void bootstrap()
