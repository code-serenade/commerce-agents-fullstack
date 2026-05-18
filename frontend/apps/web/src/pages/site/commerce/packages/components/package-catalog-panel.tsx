import { useTranslation } from "react-i18next"
import { Button, NativeSelect } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  CommerceToolbar,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import { packageKey, type PackagesPageState } from "../use-packages-page"
import { packageColumns } from "./package-table-columns"

export function PackageCatalogPanel({ page }: { page: PackagesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.packages.panels.catalog")}
      description={t("commerce.packages.panels.catalogDesc")}
      actions={
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={page.syncingLegacyBuiltins}
            onClick={() => page.syncLegacyBuiltins()}
          >
            {t("commerce.packages.panels.syncLegacy")}
          </Button>
          <Button size="sm" onClick={() => page.setDialog("package")}>
            {t("commerce.packages.panels.savePackage")}
          </Button>
        </div>
      }
    >
      <div className="space-y-3">
        <CommerceToolbar
          keyword={page.keyword}
          keywordPlaceholder={t("commerce.packages.panels.search")}
          onKeywordChange={page.setKeyword}
          filters={
            <NativeSelect
              value={page.packageType}
              placeholder={t("commerce.packages.panels.packageType")}
              options={packageTypeOptions(t)}
              onValueChange={page.setPackageType}
            />
          }
        />
        <CommerceTable
          loading={page.loading}
          items={page.packages}
          emptyText={t("commerce.packages.panels.emptyPackages")}
          getRowId={(item) => packageKey(item)}
          columns={packageColumns(page, t)}
        />
      </div>
    </CommercePanel>
  )
}

function packageTypeOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.common.all"), value: "" },
    { label: String("domain"), value: "domain" },
    { label: String("role"), value: "role" },
    { label: String("skill"), value: "skill" },
    { label: String("capability"), value: "capability" },
  ]
}
