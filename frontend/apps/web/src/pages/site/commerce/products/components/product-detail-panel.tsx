import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  Button,
  NativeSelect,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@workspace/ui-components"
import {
  CommerceDetailDrawer,
  CommerceTable,
  MutedText,
  SubPanelAction,
} from "../../components/commerce-ui"
import { commerceStatusOptions } from "../../domain/status-options"
import type { ProductsPageState } from "../use-products-page"
import {
  createAssetColumns,
  createChangeColumns,
  createCompetitorColumns,
  createKnowledgeColumns,
  createMaterialColumns,
} from "./product-table-columns"

export function ProductDetailDrawer({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("knowledge")

  if (!page.selectedProduct) return null

  return (
    <CommerceDetailDrawer
      open={page.detailOpen}
      onClose={() => page.setDetailOpen(false)}
      closeLabel={t("commerce.common.close")}
      title={page.selectedProduct.name}
      description={t("commerce.products.panels.selectedDesc")}
    >
      <div className="space-y-4">
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="h-auto max-w-full flex-wrap justify-start">
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
              buttonSize="default"
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
              buttonSize="default"
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
      </div>
    </CommerceDetailDrawer>
  )
}

function ProductMaterialSection({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()
  const statusOptions = [
    { label: t("commerce.common.all"), value: "" },
    ...commerceStatusOptions(t, ["active", "archived"]),
  ]

  return (
    <div className="min-w-0">
      <div className="mb-3 flex flex-wrap items-end justify-between gap-2">
        <label className="w-40 max-w-full">
          <span className="sr-only">{t("commerce.common.status")}</span>
          <NativeSelect
            value={page.materialStatus}
            options={statusOptions}
            onValueChange={page.setMaterialStatus}
          />
        </label>
        <Button
          size="default"
          variant="outline"
          onClick={() => page.setDialog("create-material")}
        >
          {t("commerce.products.panels.createMaterial")}
        </Button>
      </div>
      <CommerceTable
        loading={page.loading}
        items={page.materials}
        emptyText={t("commerce.products.empty.materials")}
        getRowId={(item) => item.id}
        columns={createMaterialColumns(page, t)}
      />
    </div>
  )
}

function ProductCompetitorSection({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-5">
      <div className="min-w-0">
        <SubPanelAction
          buttonSize="default"
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
      {page.selectedCompetitor ? (
        <section className="min-w-0 border-t border-(--app-border) pt-4">
          <div className="mb-3 flex items-center justify-between gap-3">
            <div className="min-w-0">
              <h3 className="text-sm font-semibold">
                {t("commerce.products.panels.competitorChanges")}
              </h3>
              <MutedText>{page.selectedCompetitor.name}</MutedText>
            </div>
            <Button
              className="shrink-0"
              size="default"
              variant="outline"
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
        </section>
      ) : null}
    </div>
  )
}
