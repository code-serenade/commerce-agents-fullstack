import { Button } from "@workspace/ui-components"
import type {
  CompetitorChangeResponse,
  ProductAssetResponse,
  ProductCompetitorResponse,
  ProductKnowledgeResponse,
  ProductMaterialResponse,
} from "@/api"
import {
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  getCommerceEnumLabel,
  getCommerceStatusLabel,
  knowledgeConfidenceLabel,
  knowledgeContentTypeLabel,
  knowledgeSourceTypeLabel,
} from "../../domain/enum-mappings"
import type { ProductDialogName, ProductsPageState } from "../use-products-page"
export function createKnowledgeColumns(
  page: ProductsPageState,
  t: CommerceTranslate
): CommerceColumn<ProductKnowledgeResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.products.columns.knowledge"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">
            {item.title || knowledgeContentTypeLabel(t, item.contentType)}
          </div>
          <MutedText>{item.content.slice(0, 64)}</MutedText>
        </div>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => knowledgeContentTypeLabel(t, item.contentType),
    },
    {
      key: "confidence",
      header: t("commerce.products.columns.confidence"),
      render: (item) => knowledgeConfidenceLabel(t, item.confidence),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          {editButton(t, "edit-knowledge", page, () =>
            page.setSelectedKnowledgeId(item.id)
          )}
          {archiveButton(t, () => page.archiveKnowledge(item.id))}
        </div>
      ),
    },
  ]
}

export function createAssetColumns(
  page: ProductsPageState,
  t: CommerceTranslate
): CommerceColumn<ProductAssetResponse>[] {
  return [
    {
      key: "file",
      header: t("commerce.products.columns.asset"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">{item.fileName}</div>
          <MutedText>{item.description || item.objectKey || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "type",
      header: t("commerce.common.type"),
      render: (item) => getCommerceEnumLabel(t, "assetType", item.assetType),
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          {editButton(t, "edit-asset", page, () =>
            page.setSelectedAssetId(item.id)
          )}
          <Button
            size="xs"
            variant="outline"
            onClick={() => page.accessAsset(item.id)}
          >
            {t("commerce.common.download")}
          </Button>
          {archiveButton(t, () => page.archiveAsset(item.id))}
        </div>
      ),
    },
  ]
}

export function createMaterialColumns(
  page: ProductsPageState,
  t: CommerceTranslate
): CommerceColumn<ProductMaterialResponse>[] {
  return [
    {
      key: "title",
      header: t("commerce.products.columns.material"),
      render: (item) => (
        <div className="min-w-0">
          <span className="block truncate font-medium text-foreground">
            {item.title ||
              getCommerceEnumLabel(t, "materialType", item.materialType)}
          </span>
          <MutedText>
            {item.content?.slice(0, 64) || t("commerce.common.noContent")}
          </MutedText>
        </div>
      ),
      minWidth: "9rem",
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>
          {getCommerceStatusLabel(t, item.status)}
        </StatusBadge>
      ),
      width: "7rem",
    },
    {
      key: "version",
      header: t("commerce.common.version"),
      render: (item) => item.version ?? "-",
      width: "5rem",
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) =>
        item.status === "archived" ? null : (
          <div className="flex flex-wrap gap-2">
            {editButton(t, "edit-material", page, () =>
              page.setSelectedMaterialId(item.id)
            )}
            {archiveButton(t, () => page.archiveMaterial(item.id))}
          </div>
        ),
      width: "9rem",
    },
  ]
}

export function createCompetitorColumns(
  page: ProductsPageState,
  t: CommerceTranslate
): CommerceColumn<ProductCompetitorResponse>[] {
  return [
    {
      key: "name",
      header: t("commerce.products.columns.competitor"),
      render: (item) => (
        <button
          type="button"
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedCompetitorId(item.id)}
        >
          <span className="block truncate font-medium text-foreground">
            {item.name}
          </span>
          <MutedText>{item.platform || item.url || "-"}</MutedText>
        </button>
      ),
    },
    {
      key: "price",
      header: t("commerce.fields.price"),
      render: (item) => item.price ?? "-",
    },
    {
      key: "rating",
      header: t("commerce.fields.rating"),
      render: (item) => item.rating ?? "-",
    },
    {
      key: "actions",
      header: t("commerce.common.actions"),
      render: (item) => (
        <div className="flex flex-wrap gap-2">
          {editButton(t, "edit-competitor", page, () =>
            page.setSelectedCompetitorId(item.id)
          )}
          {archiveButton(t, () => page.archiveCompetitor(item.id))}
        </div>
      ),
    },
  ]
}
function editButton(
  t: CommerceTranslate,
  dialog: ProductDialogName,
  page: ProductsPageState,
  selectTarget: () => void
) {
  return (
    <Button
      size="xs"
      variant="outline"
      onClick={() => {
        selectTarget()
        page.setDialog(dialog)
      }}
    >
      {t("commerce.common.edit")}
    </Button>
  )
}

function archiveButton(t: CommerceTranslate, onClick: () => void) {
  return (
    <Button size="xs" variant="outline" onClick={onClick}>
      {t("commerce.common.archive")}
    </Button>
  )
}

export function createChangeColumns(
  t: CommerceTranslate
): CommerceColumn<CompetitorChangeResponse>[] {
  return [
    {
      key: "change",
      header: t("commerce.products.columns.change"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">
            {getCommerceEnumLabel(t, "changeType", item.changeType)}
          </div>
          <MutedText>
            {item.summary || `${item.oldValue || "-"} -> ${item.newValue || "-"}`}
          </MutedText>
        </div>
      ),
    },
    {
      key: "source",
      header: t("commerce.common.source"),
      render: (item) => knowledgeSourceTypeLabel(t, item.sourceType),
    },
  ]
}
