import {
  CommerceBody,
  CommerceSurface,
} from "../../commerce/components/commerce-ui"
import { UsageGuideJourney } from "../components/usage-guide-journey"
import { UsageGuideOverview } from "../components/usage-guide-overview"
import { UsageGuideReference } from "../components/usage-guide-reference"

export default function UsageGuidePage() {
  return (
    <CommerceSurface>
      <CommerceBody className="min-[1200px]:px-8">
        <div className="space-y-6 [&>section+section]:border-t [&>section+section]:border-(--app-border)/60 [&>section+section]:pt-6">
          <UsageGuideOverview />
          <UsageGuideJourney />
          <UsageGuideReference />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
