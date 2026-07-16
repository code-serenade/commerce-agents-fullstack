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
      <CommerceBody className="bg-background min-[1200px]:px-8">
        <div className="mx-auto w-full max-w-[1200px] space-y-8 [&>section+section]:border-t [&>section+section]:border-(--app-border)/55 [&>section+section]:pt-8">
          <UsageGuideOverview />
          <UsageGuideJourney />
          <UsageGuideReference />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
