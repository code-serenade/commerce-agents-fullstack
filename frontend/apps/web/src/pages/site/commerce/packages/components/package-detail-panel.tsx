import { useTranslation } from "react-i18next"
import { Button, NativeSelect } from "@workspace/ui-components"
import {
  CommercePanel,
  CommerceTable,
  InlineAlert,
} from "../../components/commerce-ui"
import { type PackagesPageState } from "../use-packages-page"
import { historyColumns, lockColumns } from "./package-table-columns"

export function PackageDetailPanel({ page }: { page: PackagesPageState }) {
  return (
    <div className="grid gap-4 xl:grid-cols-2">
      <PackageHistoryPanel page={page} />
      <PackageLocksPanel page={page} />
    </div>
  )
}

function PackageHistoryPanel({ page }: { page: PackagesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={page.selectedPackage?.packageName ?? t("commerce.packages.panels.detail")}
      description={t("commerce.packages.panels.detailDesc")}
      actions={
        page.selectedPackage ? (
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => page.validatePackage(page.selectedPackage!)}
            >
              {t("commerce.common.validate")}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => page.activatePackage(page.selectedPackage!)}
            >
              {t("commerce.common.activate")}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => page.deactivatePackage(page.selectedPackage!)}
            >
              {t("commerce.common.deactivate")}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => page.rollbackPackage(page.selectedPackage!)}
            >
              {t("commerce.common.rollback")}
            </Button>
          </div>
        ) : null
      }
    >
      {page.validation ? (
        <div className="mb-3">
          <InlineAlert
            variant={page.validation.valid ? "success" : "warning"}
            title={
              page.validation.valid
                ? t("commerce.packages.panels.valid")
                : t("commerce.packages.panels.invalid")
            }
            description={
              [...page.validation.errors, ...page.validation.warnings].join("; ") ||
              t("commerce.packages.panels.noValidationIssues")
            }
          />
        </div>
      ) : null}
      <CommerceTable
        loading={page.loading}
        items={page.history}
        emptyText={t("commerce.packages.panels.emptyPackages")}
        getRowId={(item) => item.id}
        columns={historyColumns(t)}
      />
    </CommercePanel>
  )
}

function PackageLocksPanel({ page }: { page: PackagesPageState }) {
  const { t } = useTranslation()

  return (
    <CommercePanel
      title={t("commerce.packages.panels.locks")}
      description={t("commerce.packages.panels.locksDesc")}
      actions={
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={!page.workspaceId || !page.runId}
            onClick={() => page.checkDrift()}
          >
            {t("commerce.packages.panels.checkDrift")}
          </Button>
          <Button
            size="sm"
            disabled={!page.workspaceId || !page.runId}
            onClick={() => page.setDialog("lock")}
          >
            {t("commerce.packages.panels.saveLock")}
          </Button>
        </div>
      }
    >
      <div className="mb-3 grid gap-2 sm:grid-cols-2">
        <NativeSelect
          value={page.workspaceId}
          placeholder={t("commerce.packages.panels.selectWorkspace")}
          onValueChange={page.setWorkspaceId}
          options={page.workspaces.map((workspace) => ({
            label: workspace.title,
            value: workspace.id,
          }))}
        />
        <NativeSelect
          value={page.runId}
          placeholder={t("commerce.packages.panels.selectRun")}
          onValueChange={page.setRunId}
          options={page.runs.map((run) => ({
            label: `${run.goal || run.id.slice(0, 8)} · ${run.status}`,
            value: run.id,
          }))}
          disabled={!page.workspaceId || page.runs.length === 0}
        />
      </div>
      {page.workspaceId && page.runId ? (
        <div className="mb-3">
          <InlineAlert
            variant="success"
            title={t("commerce.packages.panels.lockContext")}
            description={t("commerce.packages.panels.lockContextDesc", {
              run:
                page.selectedRun?.goal ||
                page.selectedRun?.id.slice(0, 8) ||
                page.runId.slice(0, 8),
              workspace:
                page.selectedWorkspace?.title ||
                page.workspaceId.slice(0, 8),
            })}
          />
        </div>
      ) : null}
      {page.drift ? (
        <div className="mb-3">
          <InlineAlert
            variant={page.drift.drifts.length ? "warning" : "success"}
            title={t("commerce.packages.panels.drift")}
            description={t("commerce.common.driftCount", {
              count: page.drift.drifts.length,
            })}
          />
        </div>
      ) : null}
      <CommerceTable
        loading={page.loading}
        items={page.locks}
        emptyText={
          page.workspaceId && page.runId
            ? t("commerce.packages.panels.emptyLocksForRun")
            : t("commerce.packages.panels.emptyLocks")
        }
        getRowId={(item, index) =>
          item.id ?? `${item.packageType}:${item.packageId}:${index}`
        }
        columns={lockColumns(t)}
      />
    </CommercePanel>
  )
}
