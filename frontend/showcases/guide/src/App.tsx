import { RouterProvider, createBrowserRouter } from "react-router"
import { ToastProvider } from "@workspace/ui-components/stable/toast"
import { appRoutes } from "@/routes/appRoutes"

const router = createBrowserRouter([appRoutes])

export default function App() {
  return (
    <>
      <RouterProvider router={router} />
      <ToastProvider position="top-center" />
    </>
  )
}
