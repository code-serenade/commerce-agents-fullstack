# admin

后台管理台应用。

## 运行

```bash
pnpm -C apps/admin dev
```

## 说明

- `apps/admin` 是标准后台基础能力，不是演示 app。
- 每个商用 admin 默认需要账号、角色、权限和菜单可见性管理。
- UI 壳层使用 workspace 自有组件实现。
- 服务、鉴权恢复和权限恢复复用 workspace 现有实现，并通过本地服务端联调。
