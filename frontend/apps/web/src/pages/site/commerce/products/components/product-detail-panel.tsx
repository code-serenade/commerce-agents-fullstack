import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Button, Tabs, TabsContent, TabsList, TabsTrigger } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  MutedText,
  StatusBadge,
  SubPanelAction,
  toneForStatus,
} from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"
import {
  createAssetColumns,
  createChangeColumns,
  createCompetitorColumns,
  createKnowledgeColumns,
  createMaterialColumns,
} from "./product-table-columns"

export function ProductDetailPanel({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("knowledge")

  if (!page.selectedProduct) {
    return (
      <CommercePanel
        title={t("commerce.products.panels.detail")}
        description={t("commerce.products.panels.detailDesc")}
      >
        <div className="rounded-lg border border-dashed border-(--app-border) p-8 text-center text-sm text-muted-foreground">
          {t("commerce.products.panels.noProduct")}
        </div>
      </CommercePanel>
    )
  }

  return (
    <CommercePanel
      title={page.selectedProduct.name}
      description={t("commerce.products.panels.selectedDesc")}
      actions={
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => page.setDialog("edit-product")}
          >
            {t("commerce.products.panels.edit")}
          </Button>
          <Button
            size="sm"
            variant="destructive"
            onClick={() => page.archiveProduct(page.selectedProduct!.id)}
          >
            {t("commerce.products.panels.archive")}
          </Button>
        </div>
      }
    >
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="knowledge">
            {t("commerce.products.panels.knowledgeTab")}
          </TabsTrigger>
          <TabsTrigger value="assets">
            {t("commerce.products.panels.assetsTab")}
          </TabsTrigger>
          <TabsTrigger value="materials">
            {t("commerce.products.panels.materialsTab")}
          </TabsTrigger>
          <TabsTrigger value="competitors">
            {t("commerce.products.panels.competitorsTab")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="knowledge">
          <SubPanelAction
            label={t("commerce.products.panels.createKnowledge")}
            onClick={() => page.setDialog("create-knowledge")}
          />
          <CommerceTable
            loading={page.loading}
            items={page.knowledge}
            emptyText={t("commerce.products.empty.knowledge")}
            getRowId={(item) => item.id}
            columns={createKnowledgeColumns(page, t)}
          />
        </TabsContent>

        <TabsContent value="assets">
          <SubPanelAction
            label={t("commerce.products.panels.registerAsset")}
            onClick={() => page.setDialog("create-asset")}
          />
          <CommerceTable
            loading={page.loading}
            items={page.assets}
            emptyText={t("commerce.products.empty.assets")}
            getRowId={(item) => item.id}
            columns={createAssetColumns(page, t)}
          />
        </TabsContent>

        <TabsContent value="materials">
          <ProductMaterialSection page={page} />
        </TabsContent>

        <TabsContent value="competitors">
          <ProductCompetitorSection page={page} />
        </TabsContent>
      </Tabs>
    </CommercePanel>
  )
}

function ProductMaterialSection({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(22rem,0.75fr)]">
      <div className="min-w-0">
        <SubPanelAction
          label={t("commerce.products.panels.createMaterial")}
          onClick={() => page.setDialog("create-material")}
        />
        <CommerceTable
          loading={page.loading}
          items={page.materials}
          emptyText={t("commerce.products.empty.materials")}
          getRowId={(item) => item.id}
          columns={createMaterialColumns(page, t)}
        />
      </div>
      <div className="min-w-0 rounded-lg border border-(--app-border) p-3">
        <div className="mb-3 min-w-0">
          <h3 className="truncate text-sm font-semibold">
            {page.selectedMaterial?.title ||
              page.selectedMaterial?.materialType ||
              t("commerce.products.panels.materialVersions")}
          </h3>
          <MutedText>{t("commerce.products.panels.materialVersionsDesc")}</MutedText>
        </div>
        <CommerceTable
          loading={page.materialVersionsLoading}
          items={page.materialVersions}
          emptyText={t("commerce.products.empty.materialVersions")}
          getRowId={(item) => item.id}
          columns={[
            {
              key: "version",
              header: t("commerce.common.version"),
              render: (item) => item.version ?? "-",
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
            {
              key: "title",
              header: t("commerce.products.columns.material"),
              render: (item) => item.title || item.materialType,
            },
          ]}
        />
      </div>
    </div>
  )
}

function ProductCompetitorSection({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(22rem,0.75fr)]">
      <div className="min-w-0">
        <SubPanelAction
          label={t("commerce.products.panels.createCompetitor")}
          onClick={() => page.setDialog("create-competitor")}
        />
        <CommerceTable
          loading={page.loading}
          items={page.competitors}
          emptyText={t("commerce.products.empty.competitors")}
          getRowId={(item) => item.id}
          columns={createCompetitorColumns(page, t)}
        />
      </div>
      <div className="min-w-0 rounded-lg border border-(--app-border) p-3">
        <div className="mb-3 flex items-center justify-between gap-2">
          <div className="min-w-0">
            <h3 className="truncate text-sm font-semibold">
              {page.selectedCompetitor?.name ??
                t("commerce.products.panels.competitorChanges")}
            </h3>
            <MutedText>
              {t("commerce.products.panels.competitorChangesDesc")}
            </MutedText>
          </div>
          <Button
            size="sm"
            variant="outline"
            disabled={!page.selectedCompetitor}
            onClick={() => page.setDialog("create-competitor-change")}
          >
            {t("commerce.products.panels.recordChange")}
          </Button>
        </div>
        <CommerceTable
          loading={page.loading}
          items={page.changes}
          emptyText={t("commerce.products.empty.changes")}
          getRowId={(item) => item.id}
          columns={createChangeColumns(t)}
        />
      </div>
    </div>
  )
}
