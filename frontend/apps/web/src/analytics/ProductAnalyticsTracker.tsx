import { useEffect } from "react"
import { useLocation } from "react-router"
import { trackProductPageView } from "./product-analytics"

export function ProductAnalyticsTracker() {
  const location = useLocation()

  useEffect(() => {
    trackProductPageView(location.pathname, location.search)
  }, [location.pathname, location.search])

  return null
}
