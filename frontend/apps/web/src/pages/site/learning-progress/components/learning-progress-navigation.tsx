import { useTranslation } from "react-i18next"
import { Tabs, TabsList, TabsTrigger } from "@workspace/ui-components"
import type { LearningProgressViewKey } from "../types"

export function LearningProgressNavigation({
  activeView,
  onViewChange,
}: {
  activeView: LearningProgressViewKey
  onViewChange: (view: LearningProgressViewKey) => void
}) {
  const { t } = useTranslation()

  return (
    <div className="-mx-1 overflow-x-auto px-1 pb-1">
      <Tabs
        value={activeView === "review" ? "portfolio" : activeView}
        onValueChange={(value) => {
          if (isLearningProgressView(value)) onViewChange(value)
        }}
      >
        <TabsList>
          <TabsTrigger
            className="px-1 text-xs sm:px-1.5 sm:text-sm"
            value="overview"
          >
            {t("learningProgress.views.overview")}
          </TabsTrigger>
          <TabsTrigger
            className="px-1 text-xs sm:px-1.5 sm:text-sm"
            value="path"
          >
            {t("learningProgress.views.path")}
          </TabsTrigger>
          <TabsTrigger
            className="px-1 text-xs sm:px-1.5 sm:text-sm"
            value="map"
          >
            {t("learningProgress.views.map")}
          </TabsTrigger>
          <TabsTrigger
            className="px-1 text-xs sm:px-1.5 sm:text-sm"
            value="portfolio"
          >
            {t("learningProgress.views.portfolio")}
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  )
}

function isLearningProgressView(value: string): value is LearningProgressViewKey {
  return (
    value === "overview" ||
    value === "review" ||
    value === "path" ||
    value === "map" ||
    value === "portfolio"
  )
}
