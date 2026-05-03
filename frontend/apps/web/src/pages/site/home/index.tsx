import { HomeOverview } from "./components/home-overview"
import { useHomePage } from "./use-home-page"

export default function HomePage() {
  const home = useHomePage()

  return (
    <HomeOverview
      user={home.user}
      displayName={home.displayName}
      displayId={home.displayId}
      avatarFallback={home.avatarFallback}
    />
  )
}
