import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@workspace/ui-components/stable/tabs"
import type { RoleResponse } from "@/api"

export function RolePermissionTabs({
  activeRoleId,
  children,
  onRoleChange,
  roles,
}: {
  activeRoleId: string
  children: React.ReactNode
  onRoleChange: (roleId: string) => void
  roles: readonly RoleResponse[]
}) {
  return (
    <Tabs value={activeRoleId} onValueChange={onRoleChange}>
      <div className="flex min-w-0 flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="max-w-full min-w-0 overflow-x-auto pb-1">
          <TabsList>
            {roles.map((role) => (
              <TabsTrigger key={role.id} value={String(role.id)}>
                {role.name}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>
      </div>
      {children}
    </Tabs>
  )
}

export { TabsContent as RolePermissionTabsContent }
