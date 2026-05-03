import { useTranslation } from "react-i18next"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@workspace/ui-components/stable/card"

import { RolePermissionEditor } from "./components/role-permission-editor"
import {
  RolePermissionTabs,
  RolePermissionTabsContent,
} from "./components/role-tabs"
import { useRolePermissionsPage } from "./use-role-permissions-page"

export default function RolePermissionsPage() {
  const { t } = useTranslation()
  const page = useRolePermissionsPage()

  return (
    <div className="w-full min-w-0 space-y-4 px-4 pb-4">
      {page.roles.length > 0 && page.activeRole ? (
        <RolePermissionTabs
          activeRoleId={page.resolvedRoleId}
          roles={page.roles}
          onRoleChange={page.handleRoleChange}
        >
          {page.roles.map((role) => (
            <RolePermissionTabsContent key={role.id} value={String(role.id)}>
              <RolePermissionEditor
                activeRole={role}
                defaultExpandedIds={page.defaultExpandedIds}
                isSaving={page.isSaving}
                isTreeLoading={page.isTreeLoading}
                resourceTree={page.resourceTree}
                searchValue={page.searchValue}
                selectedCount={page.selectedPermissionIds.length}
                selectedPermissionIds={page.selectedPermissionIds}
                summary={page.summary}
                onReset={page.handleResetSelection}
                onSave={page.handleSave}
                onSearchChange={page.handleSearchChange}
                onSelectedPermissionIdsChange={
                  page.handleSelectedPermissionIdsChange
                }
              />
            </RolePermissionTabsContent>
          ))}
        </RolePermissionTabs>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>
              {t("admin.access.rolePermissions.empty.title")}
            </CardTitle>
            <CardDescription>
              {t("admin.access.rolePermissions.empty.description")}
            </CardDescription>
          </CardHeader>
        </Card>
      )}
    </div>
  )
}
