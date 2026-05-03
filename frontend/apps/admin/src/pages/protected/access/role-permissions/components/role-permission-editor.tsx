import { useTranslation } from "react-i18next"
import { SearchInput, TreeView, type TreeNode } from "@workspace/ui-components"
import { Button } from "@workspace/ui-components/stable/button"
import type { RoleResponse } from "@/api"
import { PermissionSummary } from "./permission-summary"
import type { PermissionSummaryType } from "../types"

export function RolePermissionEditor({
  activeRole,
  defaultExpandedIds,
  isSaving,
  isTreeLoading,
  onReset,
  onSave,
  onSearchChange,
  onSelectedPermissionIdsChange,
  resourceTree,
  searchValue,
  selectedCount,
  selectedPermissionIds,
  summary,
}: {
  activeRole: RoleResponse
  defaultExpandedIds: string[]
  isSaving: boolean
  isTreeLoading: boolean
  onReset: () => void
  onSave: () => void
  onSearchChange: (value: string) => void
  onSelectedPermissionIdsChange: (value: string[]) => void
  resourceTree: TreeNode[]
  searchValue: string
  selectedCount: number
  selectedPermissionIds: string[]
  summary: Record<PermissionSummaryType, number>
}) {
  const { t } = useTranslation()

  return (
    <section className="space-y-4">
      <div>
        <div className="grid min-w-0 gap-3">
          <div className="min-w-0">
            <h2 className="text-lg font-semibold">{activeRole.name}</h2>
            <p className="mt-1 text-sm text-(--app-muted-text)">
              {t("admin.access.rolePermissions.editor.codePrefix")}
              {activeRole.code}
              {t("admin.access.rolePermissions.editor.descriptionSuffix")}
            </p>
          </div>
          <div className="flex flex-wrap justify-start gap-2">
            <Button variant="outline" onClick={onReset} disabled={isTreeLoading}>
              {t("admin.access.rolePermissions.editor.reset")}
            </Button>
            <Button
              variant="primary"
              onClick={onSave}
              disabled={isTreeLoading || isSaving}
            >
              {t("admin.access.rolePermissions.editor.save")}
            </Button>
          </div>
        </div>
      </div>

      <div className="space-y-4">
        <div className="w-full max-w-80">
          <SearchInput
            value={searchValue}
            onValueChange={onSearchChange}
            placeholder={t(
              "admin.access.rolePermissions.editor.searchPlaceholder"
            )}
          />
        </div>

        <PermissionSummary selectedCount={selectedCount} summary={summary} />

        <div className="rounded-(--ui-radius-md) bg-(--app-panel) p-3">
          <TreeView
            data={resourceTree}
            value={selectedPermissionIds}
            onValueChange={onSelectedPermissionIdsChange}
            defaultExpandedIds={defaultExpandedIds}
            searchValue={searchValue}
            maxHeight={520}
            emptyLabel={
              isTreeLoading
                ? t("admin.access.rolePermissions.editor.loading")
                : t("admin.access.rolePermissions.editor.empty")
            }
          />
        </div>
      </div>
    </section>
  )
}
