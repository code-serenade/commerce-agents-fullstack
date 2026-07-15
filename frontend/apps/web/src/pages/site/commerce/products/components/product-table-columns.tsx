import type { ProductResponse } from "@/api"
import { Button } from "@workspace/ui-components"
import { IdText, StatusBadge, toneForStatus } from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import { getCommerceStatusLabel } from "../../domain/status-options"
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
        <div className="min-w-0">
          <span className="block truncate font-medium text-foreground">
            {item.name}
          </span>
          <IdText>{item.id}</IdText>
        </div>
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
        item.price ? (
          <span className="inline-flex items-baseline gap-1">
            <span className="text-xs text-muted-foreground">
              {item.currency || "CNY"}
            </span>
            <span>{item.price}</span>
          </span>
        ) : (
          "-"
        ),
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {getCommerceStatusLabel(t, item.status)}
        </StatusBadge>
      ),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      width: "16rem",
      render: (item) => (
        <div className="flex items-center gap-1 whitespace-nowrap">
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.openProductEditor(item.id)}
          >
            {t("commerce.common.edit")}
          </Button>
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.openProductDetail(item.id)}
          >
            {t("commerce.products.panels.moreDetails")}
          </Button>
          <Button
            size="xs"
            variant="destructive"
            onClick={() => page.archiveProduct(item.id)}
          >
            {t("commerce.products.panels.archive")}
          </Button>
        </div>
      ),
    },
  ]
}
