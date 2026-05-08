import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { PackageDialogs } from "./components/package-dialogs"
import {
  PackageCatalogPanel,
  PackageDetailPanel,
  PackageMetrics,
} from "./components/package-panels"
import { usePackagesPage } from "./use-packages-page"

export default function PackagesPage() {
  const { t } = useTranslation()
  const page = usePackagesPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.packages.eyebrow")}
        title={t("commerce.pages.packages.title")}
        description={t("commerce.pages.packages.description")}
      />
      <CommerceBody>
        <InlineAlert
          variant="warning"
          title={t("commerce.pages.packages.boundaryTitle")}
          description={t("commerce.pages.packages.boundaryDescription")}
        />
        <PackageMetrics items={page.metricCards} />
        <PackageCatalogPanel page={page} />
        <PackageDetailPanel page={page} />
      </CommerceBody>
      <PackageDialogs page={page} />
    </CommerceSurface>
  )
}
