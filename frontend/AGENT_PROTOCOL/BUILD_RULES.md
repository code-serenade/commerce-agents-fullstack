# 构建与脚本规则

## 总规则

在本仓库执行 build、typecheck、lint、test、dev、preview、format、clean 等操作时：

- 必须使用最近相关 `package.json` 中定义的脚本。
- 在仓库根目录，优先使用根脚本，例如 `pnpm build`、`pnpm typecheck`、`pnpm lint`、`pnpm test`、`pnpm rules`、`pnpm dev`、`pnpm format`。
- 针对某个 app 或 package 时，优先使用 `pnpm -C <dir> <script>`，并确保该脚本存在于对应 package。
- 如果 package script 已存在，不要运行临时拼出来的编译或构建命令。
- 根目录 `package.json` 是 workspace 级验证脚本的事实来源。

## 代码修改后的必需验证

修改源码、package metadata、影响 AI/代码行为的协议文件、构建配置、测试或 rules 后，结束前必须运行验证。

最低必需门禁：

```bash
rtk pnpm rules
```

`pnpm rules` 是代码改动后的强制门禁，因为它检查仓库架构和 AI-facing 边界。除非用户明确限制范围并接受更窄验证，否则不能用临时 `vitest`、`tsc` 或 package-only 检查替代它。

还要根据实际改动运行最近相关脚本：

- TypeScript / React / package API 改动：`rtk pnpm typecheck` 或 `rtk pnpm -C <dir> typecheck`
- UI / Tailwind / TSX 改动：`rtk pnpm lint` 或 `rtk pnpm -C <dir> lint`
- 行为或可测试逻辑改动：`rtk pnpm test` 或 `rtk pnpm -C <dir> test`
- 构建或 app 运行时改动：`rtk pnpm build` 或 `rtk pnpm -C <dir> build`

如果必需脚本因为已有无关问题失败，要报告具体失败文件和命令，然后在可用时运行下一个最接近的非临时脚本。不要悄悄用 raw command 替代 package script。

## 未经用户明确要求禁止直接运行

除非用户明确要求运行这些具体命令，否则不要直接运行：

```bash
tsc
pnpm exec tsc
npx tsc
vite build
vitest
eslint
```

请使用对应 `package.json` script。

如果某个 package 没有暴露需要的脚本，优先使用根脚本。只有确认没有 package script 且说明原因后，才允许用 raw command 诊断。

## TypeScript 安全

- 不要运行根级 TypeScript emit 命令。
- 不要用可能把 `.js` 输出到 `src/` 的方式运行 `tsc`。
- 诊断构建行为时，先检查相关 `package.json` script，再使用该 script 复现。

## Tailwind class 安全

- Tailwind canonical class 检查通过 package `lint` 脚本中的 `eslint-plugin-better-tailwindcss` 执行。
- 修改 React/TSX UI、`className` 字符串、`cn(...)`、`cva(...)` 或 Tailwind utility class 后，必须运行最近相关 lint 脚本，让 canonical class 警告显示在终端里。
- admin UI 工作使用：

```bash
rtk pnpm -C apps/admin run lint
```

- workspace 级验证使用：

```bash
rtk pnpm lint
```

- 修复 canonical class 问题，例如 `rounded-[var(--ui-radius-lg)]` -> `rounded-(--ui-radius-lg)`，再认为任务完成。
- 不要只依赖 VS Code Tailwind IntelliSense；触碰 Tailwind class 时，命令行 lint 必须干净。

## 示例

```bash
rtk pnpm build
rtk pnpm -C apps/admin build
rtk pnpm -C apps/web test
rtk pnpm typecheck
rtk pnpm rules
```

## 构建前检查

运行构建相关命令前，先确认脚本来源：

```bash
rtk cat package.json
rtk cat apps/admin/package.json
```
