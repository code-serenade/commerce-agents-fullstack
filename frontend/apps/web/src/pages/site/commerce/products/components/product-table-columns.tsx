import type { ProductResponse } from "@/api"
import {
  IdText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { ProductsPageState } from "../use-products-page"
export {
  createAssetColumns,
  createChangeColumns,
  createCompetitorColumns,
  createKnowledgeColumns,
  createMaterialColumns,
} from "./product-detail-columns"

export function createProductColumns(
  page: ProductsPageState,
  t: CommerceTranslate
): CommerceColumn<ProductResponse>[] {
  return [
    {
      key: "name",
      header: t("commerce.products.columns.product"),
      render: (item) => (
        <button
          type="button"
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedProductId(item.id)}
        >
          <span className="block truncate font-medium text-foreground">
            {item.name}
          </span>
          <IdText>{item.id}</IdText>
        </button>
      ),
    },
    {
      key: "platform",
      header: t("commerce.common.platform"),
      render: (item) => item.platform || "-",
    },
    {
      key: "price",
      header: t("commerce.fields.price"),
      render: (item) =>
        item.price ? `${item.currency || "CNY"} ${item.price}` : "-",
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {item.status}
        </StatusBadge>
      ),
    },
  ]
}
