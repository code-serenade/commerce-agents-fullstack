import { useTranslation } from "react-i18next"
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
    <CommercePanel>
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
