import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../components/commerce-ui"
import { Button } from "@workspace/ui-components"
import { PackageSearch, Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DocumentIntroCard } from "../components/document-section"
import { ProductDialogs } from "./components/product-dialogs"
import {
  ProductDetailDrawer,
  ProductsMasterPanel,
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
        <DocumentIntroCard
          title={t("commerce.products.panels.list")}
          description={t("commerce.products.panels.listDesc")}
          icon={<PackageSearch className="size-4" />}
          actions={
            <Button
              size="lg"
              variant="primary"
              onClick={() => page.setDialog("create-product")}
            >
              <Plus className="size-4" />
              {t("commerce.products.panels.create")}
            </Button>
          }
        />
        <ProductsMasterPanel page={page} />
      </CommerceBody>
      <ProductDetailDrawer page={page} />
      <ProductDialogs page={page} />
    </CommerceSurface>
  )
}
