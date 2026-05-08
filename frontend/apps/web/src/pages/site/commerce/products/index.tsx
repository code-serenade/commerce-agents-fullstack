import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { useTranslation } from "react-i18next"
import { ProductDialogs } from "./components/product-dialogs"
import {
  ProductDetailPanel,
  ProductsMasterPanel,
  ProductsMetrics,
} from "./components/product-panels"
import { useProductsPage } from "./use-products-page"

export default function ProductsPage() {
  const { t } = useTranslation()
  const page = useProductsPage()

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.products.eyebrow")}
        title={t("commerce.pages.products.title")}
        description={t("commerce.pages.products.description")}
      />
      <CommerceBody>
        <ProductsMetrics items={page.metricCards} />
        <div className="grid gap-4 xl:grid-cols-[minmax(22rem,0.85fr)_minmax(0,1.65fr)]">
          <ProductsMasterPanel page={page} />
          <ProductDetailPanel page={page} />
        </div>
      </CommerceBody>
      <ProductDialogs page={page} />
    </CommerceSurface>
  )
}
