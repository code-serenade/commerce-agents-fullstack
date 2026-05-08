import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  CommerceToolbar,
} from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"
import { createProductColumns } from "./product-table-columns"

export function ProductsMasterPanel({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.products.panels.list")}
      description={t("commerce.products.panels.listDesc")}
      actions={
        <Button size="sm" onClick={() => page.setDialog("create-product")}>
          {t("commerce.products.panels.create")}
        </Button>
      }
    >
      <div className="space-y-3">
        <CommerceToolbar
          keyword={page.keyword}
          keywordPlaceholder={t("commerce.products.panels.search")}
          onKeywordChange={page.setKeyword}
        />
        <CommerceTable
          loading={page.loading}
          items={page.products}
          emptyText={t("commerce.products.panels.empty")}
          getRowId={(item) => item.id}
          columns={createProductColumns(page, t)}
        />
      </div>
    </CommercePanel>
  )
}
