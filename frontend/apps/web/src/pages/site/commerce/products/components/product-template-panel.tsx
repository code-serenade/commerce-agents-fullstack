import { useTranslation } from "react-i18next"
import { Button, Spinner } from "@workspace/ui-components"
import { PackagePlus } from "lucide-react"
import { CommercePanel } from "../../components/commerce-ui"
import type { ProductsPageState } from "../use-products-page"

export function ProductTemplatePanel({ page }: { page: ProductsPageState }) {
  const { t } = useTranslation()

  if (!page.loadingProductTemplates && !page.productTemplates.length) {
    return null
  }

  return (
    <CommercePanel
      title={t("commerce.products.templates.title")}
      description={t("commerce.products.templates.description")}
    >
      {page.loadingProductTemplates ? (
        <div className="flex min-h-24 items-center justify-center gap-2 text-sm text-muted-foreground">
          <Spinner />
          <span>{t("commerce.products.templates.loading")}</span>
        </div>
      ) : (
        <div className="grid min-w-0 gap-2 lg:grid-cols-3">
          {page.productTemplates.map((template) => (
            <div
              key={template.id}
              className="flex min-w-0 flex-col gap-3 rounded-md border border-(--app-border) bg-background p-3"
            >
              <div className="min-w-0 space-y-1">
                <h3 className="truncate text-sm font-semibold text-foreground">
                  {template.name}
                </h3>
                <p className="line-clamp-2 min-h-10 text-xs/5 text-muted-foreground">
                  {template.description ||
                    t("commerce.products.templates.noDescription")}
                </p>
              </div>
              <div className="grid min-w-0 gap-1 text-xs text-muted-foreground">
                <TemplateMeta
                  label={t("commerce.common.platform")}
                  value={template.platform}
                />
                <TemplateMeta
                  label={t("commerce.fields.category")}
                  value={template.category}
                />
              </div>
              <Button
                size="sm"
                variant="outline"
                disabled={page.importProductTemplatePending}
                onClick={() => page.importProductTemplate(template.id)}
              >
                <PackagePlus className="size-4" />
                {page.importProductTemplatePending
                  ? t("commerce.products.templates.importing")
                  : t("commerce.products.templates.import")}
              </Button>
            </div>
          ))}
        </div>
      )}
    </CommercePanel>
  )
}

function TemplateMeta({
  label,
  value,
}: {
  label: string
  value?: string | null
}) {
  return (
    <div className="flex min-w-0 items-center justify-between gap-2">
      <span className="shrink-0 text-muted-foreground/80">{label}</span>
      <span className="truncate text-foreground">{value || "-"}</span>
    </div>
  )
}
