import { useCallback, useMemo, useState } from "react"
import {
  BookMarked,
  ClipboardList,
  Filter,
  PencilLine,
  Settings2,
  TableProperties,
  Trash2,
} from "lucide-react"
import {
  DataTable,
  type DataTableBuiltInQueryField,
  type DateRangeValue,
  type DataTableColumn,
  type DataTableFetchResult,
  type DataTableQueryField,
  type DataTableSortState,
} from "@workspace/app-kit"
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Separator,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@workspace/ui-components"

type ScenarioPreset = "minimal" | "operations" | "audit"

interface CustomerRow {
  id: string
  name: string
  tier: "Enterprise" | "Growth" | "Starter"
  status: "Active" | "Paused" | "Review"
  region: string
  owner: string
  channel: string
  industry: string
  createdAt: Date
  updatedAt: Date
  renewalAt: Date
  contractValue: number
  usageScore: number
  lastActiveAt: Date
}

interface TableQuery {
  keyword: string
  status: "" | CustomerRow["status"]
  region: "" | CustomerRow["region"]
  owner: "" | CustomerRow["owner"]
  createdAt?: DateRangeValue
  updatedAt?: DateRangeValue
  auditField: "createdAt" | "updatedAt"
  auditRange?: DateRangeValue
}

interface FeatureState {
  insert: boolean
  bulkUpdate: boolean
  bulkDelete: boolean
  rowActions: boolean
  keywordSearch: boolean
  advancedFilters: boolean
  createdAtQuery: boolean
  updatedAtQuery: boolean
  auditQuery: boolean
  stickyLeft: boolean
  stickyRight: boolean
  denseColumns: boolean
  zebraRows: boolean
  compactColumns: boolean
  compactRows: boolean
  showEmptyState: boolean
  customEmptyCopy: boolean
}

const REGIONS = ["North China", "East China", "South China", "West China"] as const
const OWNERS = ["Alice", "Bob", "Cathy", "David"] as const
const CHANNELS = ["Direct", "Partner", "Online", "Field"] as const
const INDUSTRIES = ["Retail", "Finance", "Education", "Manufacturing"] as const
const TIERS = ["Enterprise", "Growth", "Starter"] as const
const STATUSES = ["Active", "Paused", "Review"] as const

const PRESET_FEATURES: Record<ScenarioPreset, FeatureState> = {
  minimal: {
    insert: false,
    bulkUpdate: false,
    bulkDelete: false,
    rowActions: false,
    keywordSearch: true,
    advancedFilters: false,
    createdAtQuery: false,
    updatedAtQuery: false,
    auditQuery: false,
    stickyLeft: false,
    stickyRight: false,
    denseColumns: false,
    zebraRows: false,
    compactColumns: false,
    compactRows: false,
    showEmptyState: false,
    customEmptyCopy: false,
  },
  operations: {
    insert: true,
    bulkUpdate: true,
    bulkDelete: false,
    rowActions: true,
    keywordSearch: true,
    advancedFilters: true,
    createdAtQuery: false,
    updatedAtQuery: false,
    auditQuery: false,
    stickyLeft: true,
    stickyRight: true,
    denseColumns: false,
    zebraRows: true,
    compactColumns: false,
    compactRows: false,
    showEmptyState: false,
    customEmptyCopy: false,
  },
  audit: {
    insert: false,
    bulkUpdate: false,
    bulkDelete: true,
    rowActions: true,
    keywordSearch: true,
    advancedFilters: true,
    createdAtQuery: true,
    updatedAtQuery: true,
    auditQuery: true,
    stickyLeft: true,
    stickyRight: true,
    denseColumns: true,
    zebraRows: true,
    compactColumns: true,
    compactRows: true,
    showEmptyState: false,
    customEmptyCopy: false,
  },
}

const PRESET_COPY: Record<
  ScenarioPreset,
  { title: string; summary: string; badge: string }
> = {
  minimal: {
    title: "基础列表",
    summary: "只保留搜索、分页和基础列，适合后台首版或轻量组件页。",
    badge: "Starter",
  },
  operations: {
    title: "运营后台",
    summary: "开启筛选、批量修改、插入和行操作，适合日常业务操作台。",
    badge: "Recommended",
  },
  audit: {
    title: "审计宽表",
    summary: "强调宽表、冻结列和删除能力，适合稽核、报表和历史追踪页面。",
    badge: "Wide Table",
  },
}

const FIELD_PLAYBOOK = [
  {
    title: "基础必填",
    description:
      "`columns`、`fetchData`、`getRowId` 是最小闭环。只要这三个完整，表格就能加载、分页和排序。",
  },
  {
    title: "查询区域",
    description:
      "`query.builtInFields` 只放主搜索等非时间内建查询；状态、归属人、区域等业务筛选放 `query.fields`。",
  },
  {
    title: "操作能力",
    description:
      "`insert`、`bulkUpdate`、`bulkDelete`、`rowActions` 分别覆盖新增、批量处理、批量删除和单行操作。",
  },
  {
    title: "时间字段",
    description:
      "`auditColumns` 决定标准 `createdAt` / `updatedAt` 字段集合和列展示；`query.audit` 只控制是否显示时间筛选组件。",
  },
]

const INITIAL_QUERY: TableQuery = {
  keyword: "",
  status: "",
  region: "",
  owner: "",
  createdAt: undefined,
  updatedAt: undefined,
  auditField: "createdAt",
  auditRange: undefined,
}

export default function DataTableGuidePage() {
  const [preset, setPreset] = useState<ScenarioPreset>("operations")
  const [features, setFeatures] = useState<FeatureState>(
    PRESET_FEATURES.operations
  )
  const [rows, setRows] = useState<CustomerRow[]>(() => createCustomerRows())
  const [insertDraft, setInsertDraft] = useState({
    name: "New Workspace Customer",
    region: "East China",
  })
  const [activeTab, setActiveTab] = useState("live")
  const [auditDialogRow, setAuditDialogRow] = useState<CustomerRow | null>(null)

  const applyPreset = (nextPreset: ScenarioPreset) => {
    setPreset(nextPreset)
    setFeatures(PRESET_FEATURES[nextPreset])
  }

  const setFeature = (key: keyof FeatureState, value: boolean) => {
    setFeatures((current) => {
      const next = {
        ...current,
        [key]: value,
      }

      if (key === "customEmptyCopy" && value) {
        next.showEmptyState = true
      }

      return next
    })
  }

  const columns = useMemo<readonly DataTableColumn<CustomerRow>[]>(() => {
    const baseColumns: DataTableColumn<CustomerRow>[] = [
      {
        key: "id",
        header: "客户 ID",
        width: 120,
        sortable: true,
        renderCell: (row) => row.id,
      },
      {
        key: "name",
        header: "客户名称",
        width: 220,
        sortable: true,
        renderCell: (row) => (
          <div className="space-y-1">
            <div className="font-medium">{row.name}</div>
            <div className="text-xs text-(--app-muted-text)">{row.owner}</div>
          </div>
        ),
      },
      {
        key: "status",
        header: "状态",
        width: 120,
        sortable: true,
        renderCell: (row) => (
          <Badge variant={resolveStatusVariant(row.status)}>{row.status}</Badge>
        ),
      },
      {
        key: "tier",
        header: "层级",
        width: 120,
        sortable: true,
        renderCell: (row) => row.tier,
      },
      {
        key: "region",
        header: "区域",
        width: 140,
        sortable: true,
        renderCell: (row) => row.region,
      },
      {
        key: "contractValue",
        header: "合同金额",
        width: 140,
        sortable: true,
        renderCell: (row) => `¥${row.contractValue.toLocaleString()}`,
      },
      {
        key: "usageScore",
        header: "健康分",
        width: 120,
        sortable: true,
        renderCell: (row) => `${row.usageScore} / 100`,
      },
    ]

    if (!features.denseColumns) return baseColumns

    return [
      ...baseColumns,
      {
        key: "channel",
        header: "获客渠道",
        width: 140,
        renderCell: (row) => row.channel,
      },
      {
        key: "industry",
        header: "行业",
        width: 140,
        renderCell: (row) => row.industry,
      },
      {
        key: "owner",
        header: "负责人",
        width: 120,
        sortable: true,
        renderCell: (row) => row.owner,
      },
      {
        key: "renewalAt",
        header: "续约日期",
        width: 150,
        sortable: true,
        renderCell: (row) => formatDate(row.renewalAt),
      },
      {
        key: "lastActiveAt",
        header: "最近活跃",
        width: 150,
        sortable: true,
        renderCell: (row) => formatDate(row.lastActiveAt),
      },
    ]
  }, [features.denseColumns])

  const fetchData = useCallback(
    async ({
      page,
      pageSize,
      query,
      sort,
      signal,
    }: {
      page: number
      pageSize: number
      query: TableQuery
      sort: DataTableSortState | null
      signal: AbortSignal
    }): Promise<DataTableFetchResult<CustomerRow>> => {
      await sleep(120)
      if (signal.aborted) {
        return {
          items: [],
          total: 0,
        }
      }

      const filteredRows = rows.filter((row) => {
        const keyword = query.keyword.trim().toLowerCase()
        const candidates = [row.id, row.name, row.owner, row.region]
        const keywordMatched =
          keyword.length === 0 ||
          candidates.some((value) => value.toLowerCase().includes(keyword))

        const statusMatched =
          query.status.length === 0 || row.status === query.status
        const regionMatched =
          query.region.length === 0 || row.region === query.region
        const ownerMatched =
          query.owner.length === 0 || row.owner === query.owner
        const auditField =
          features.updatedAtQuery && !features.createdAtQuery
            ? "updatedAt"
            : query.auditField
        const activeAuditRange =
          features.createdAtQuery && features.updatedAtQuery
            ? query.auditRange
            : features.updatedAtQuery
              ? query.updatedAt
              : query.createdAt
        const auditDate = resolveAuditDate(row, auditField)
        const auditFrom = activeAuditRange?.from
        const auditTo = activeAuditRange?.to
        const auditDateMatched =
          (!auditFrom || auditDate >= startOfDay(auditFrom)) &&
          (!auditTo || auditDate <= endOfDay(auditTo))

        return (
          keywordMatched &&
          statusMatched &&
          regionMatched &&
          ownerMatched &&
          auditDateMatched
        )
      })

      const sortedRows = [...filteredRows].sort((left, right) => {
        if (!sort) return 0

        const leftValue = resolveSortValue(left, sort.columnKey)
        const rightValue = resolveSortValue(right, sort.columnKey)
        const compared = compareValues(leftValue, rightValue)

        return sort.direction === "asc" ? compared : -compared
      })

      const start = (page - 1) * pageSize
      if (features.showEmptyState) {
        return {
          items: [],
          total: 0,
        }
      }

      return {
        items: sortedRows.slice(start, start + pageSize),
        total: sortedRows.length,
      }
    },
    [
      features.createdAtQuery,
      features.showEmptyState,
      features.updatedAtQuery,
      rows,
    ]
  )

  const builtInQueryFields = useMemo<
    readonly DataTableBuiltInQueryField<TableQuery>[]
  >(() => {
    const fields: DataTableBuiltInQueryField<TableQuery>[] = []

    if (features.keywordSearch) {
      fields.push({
        key: "keyword",
        type: "search" as const,
        label: "关键词",
        placeholder: "搜客户名、客户 ID、负责人",
      })
    }

    return fields
  }, [features.keywordSearch])

  const queryFields = useMemo<readonly DataTableQueryField<TableQuery>[]>(() => {
    const fields: DataTableQueryField<TableQuery>[] = []

    if (!features.advancedFilters) return fields

    fields.push(
      {
        key: "status",
        type: "select" as const,
        label: "状态",
        placeholder: "按状态过滤",
        options: STATUSES.map((status) => ({ label: status, value: status })),
      },
      {
        key: "region",
        type: "select" as const,
        label: "区域",
        placeholder: "按区域过滤",
        options: REGIONS.map((region) => ({ label: region, value: region })),
      },
      {
        key: "owner",
        type: "select" as const,
        label: "负责人",
        placeholder: "按负责人过滤",
        options: OWNERS.map((owner) => ({ label: owner, value: owner })),
      },
    )

    return fields
  }, [
    features.advancedFilters,
  ])

  const auditQueryColumns = useMemo(() => {
    const columns: Array<"createdAt" | "updatedAt"> = []

    if (features.createdAtQuery) columns.push("createdAt")
    if (features.updatedAtQuery) columns.push("updatedAt")

    return columns
  }, [features.createdAtQuery, features.updatedAtQuery])

  const generatedSnippet = useMemo(
    () => buildSnippet({ features, preset }),
    [features, preset]
  )
  const dialogSnippet = useMemo(() => buildDialogSnippet(), [])

  const currentPreset = PRESET_COPY[preset]

  return (
    <div className="min-h-0 flex-1 overflow-auto">
      <div className="mx-auto flex w-full max-w-[1600px] flex-col gap-6 pb-10">
        <section
          id="presets"
          className="grid gap-4 lg:grid-cols-[1.1fr_0.9fr_0.9fr]"
        >
          <div className="lg:col-span-2">
            <Card>
              <CardHeader>
                <CardTitle>
                  <span className="flex items-center gap-2">
                    <Settings2 className="size-4" />
                    快速预设
                  </span>
                </CardTitle>
                <CardDescription>
                  先给用户 3 个典型场景按钮，再允许他们继续微调每个配置开关。
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex flex-wrap gap-3">
                    {(
                      Object.keys(PRESET_COPY) as Array<keyof typeof PRESET_COPY>
                    ).map((key) => (
                      <Button
                        key={key}
                        type="button"
                        variant={preset === key ? "primary" : "outline"}
                        onClick={() => applyPreset(key)}
                      >
                        {PRESET_COPY[key].title}
                      </Button>
                    ))}
                  </div>
                  <div className="rounded-2xl border border-dashed border-(--app-border) bg-(--app-surface) p-4 text-sm/6  text-(--app-muted-text)">
                    {currentPreset.summary}
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>
                <span className="flex items-center gap-2">
                  <BookMarked className="size-4" />
                  配置原则
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3 text-sm/6  text-(--app-muted-text)">
                <p>业务用户最关心的通常不是实现细节，而是“我该开哪些能力”。</p>
                <p>所以这页先提供按钮，再展示 props，对业务沟通会更顺。</p>
              </div>
            </CardContent>
          </Card>
        </section>

        <section className="grid items-start gap-4 lg:grid-cols-[260px_minmax(0,1fr)]">
          <div className="lg:sticky lg:top-4">
            <Card>
              <CardHeader>
                <CardTitle>
                  <span className="flex items-center gap-2">
                    <Filter className="size-4" />
                    配置工具条
                  </span>
                </CardTitle>
                <CardDescription>
                  开关放在左侧，收窄控制区，主区域尽量留给表格预览。
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2">
                <div className="px-1 pt-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-(--app-muted-text)">
                  业务能力
                </div>
                {renderFeatureChip(
                  "新增弹窗",
                  "insert",
                  features.insert,
                  (value) => setFeature("insert", value)
                )}
                {renderFeatureChip(
                  "批量修改",
                  "bulkUpdate",
                  features.bulkUpdate,
                  (value) => setFeature("bulkUpdate", value)
                )}
                {renderFeatureChip(
                  "批量删除",
                  "bulkDelete",
                  features.bulkDelete,
                  (value) => setFeature("bulkDelete", value)
                )}
                {renderFeatureChip(
                  "行操作",
                  "rowActions",
                  features.rowActions,
                  (value) => setFeature("rowActions", value)
                )}
                {renderFeatureChip(
                  "搜索框",
                  "query.builtInFields.search",
                  features.keywordSearch,
                  (value) => setFeature("keywordSearch", value)
                )}
                {renderFeatureChip(
                  "高级筛选",
                  "query.fields",
                  features.advancedFilters,
                  (value) => setFeature("advancedFilters", value)
                )}
                {renderFeatureChip(
                  "创建时间",
                  "auditColumns.createdAt",
                  features.createdAtQuery,
                  (value) => setFeature("createdAtQuery", value)
                )}
                {renderFeatureChip(
                  "更新时间",
                  "auditColumns.updatedAt",
                  features.updatedAtQuery,
                  (value) => setFeature("updatedAtQuery", value)
                )}
                {renderFeatureChip(
                  "时间筛选",
                  "query.audit",
                  features.auditQuery,
                  (value) => setFeature("auditQuery", value)
                )}
                {renderFeatureChip(
                  "固定左列",
                  "stickyColumns.left",
                  features.stickyLeft,
                  (value) => setFeature("stickyLeft", value)
                )}
                {renderFeatureChip(
                  "固定右侧操作列",
                  "stickyColumns.right",
                  features.stickyRight,
                  (value) => setFeature("stickyRight", value)
                )}
                {renderFeatureChip(
                  "宽表模式",
                  "extra columns",
                  features.denseColumns,
                  (value) => setFeature("denseColumns", value)
                )}
                <div className="px-1 pt-3 text-[10px] font-semibold uppercase tracking-[0.18em] text-(--app-muted-text)">
                  展示效果
                </div>
                {renderFeatureChip(
                  "斑马纹",
                  "striped rows",
                  features.zebraRows,
                  (value) => setFeature("zebraRows", value)
                )}
                {renderFeatureChip(
                  "紧凑列距",
                  "compact columns",
                  features.compactColumns,
                  (value) => setFeature("compactColumns", value)
                )}
                {renderFeatureChip(
                  "紧凑行高",
                  "compact rows",
                  features.compactRows,
                  (value) => setFeature("compactRows", value)
                )}
                {renderFeatureChip(
                  "空态演示",
                  "empty rows",
                  features.showEmptyState,
                  (value) => setFeature("showEmptyState", value)
                )}
                {renderFeatureChip(
                  "自定义空文案",
                  "renderEmpty",
                  features.customEmptyCopy,
                  (value) => setFeature("customEmptyCopy", value)
                )}
              </div>
                <div className="mt-4 rounded-xl border border-dashed border-(--app-border) p-3">
                  <div className="text-[10px] font-semibold uppercase tracking-[0.18em] text-(--app-muted-text)">
                    弹窗示例
                  </div>
                  <div className="mt-3 grid gap-2">
                    {renderDemoHint(
                      "新增弹窗",
                      "打开表格右上角“新增客户”，看 insert.renderContent 自定义内容。"
                    )}
                    {renderDemoHint(
                      "编辑弹窗",
                      "点任意行的编辑图标，看 rowActions.edit.title / description / renderContent。"
                    )}
                    {renderDemoHint(
                      "删除弹窗",
                      "点任意行的删除图标，看 rowActions.delete 的自定义标题和说明。"
                    )}
                    {renderDemoHint(
                      "更多菜单",
                      "点操作列里的更多菜单，查看 moreItems；其中“打开审计弹窗”会触发业务自定义 Dialog。"
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          <div id="live-demo" className="min-h-[820px] overflow-hidden rounded-[24px] border border-(--app-border) bg-(--app-surface)">
            <Card>
              <CardHeader>
                <div className="border-b border-(--app-border) pb-4">
                  <CardTitle>
                    <span className="flex items-center gap-2">
                      <TableProperties className="size-4" />
                      实时示例
                    </span>
                  </CardTitle>
                  <CardDescription>
                    用户点击左侧按钮后，这个表格会立刻反映配置变化。
                  </CardDescription>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex min-h-0 flex-1 flex-col gap-4 p-1">
              <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                  <TabsTrigger value="live">Live Demo</TabsTrigger>
                  <TabsTrigger value="config">配置片段</TabsTrigger>
                  <TabsTrigger value="dialogs">弹窗范例</TabsTrigger>
                  <TabsTrigger value="playbook">接入说明</TabsTrigger>
                </TabsList>

                <TabsContent value="live">
                  <div className="space-y-4">
                    <div className="rounded-xl border border-dashed border-(--app-border) px-4 py-3 text-sm/6  text-(--app-muted-text)">
                      例子入口：
                      点表格右上角“新增客户”；点任意行的编辑、删除、更多菜单；更多菜单里的“打开审计弹窗”演示的是业务层自定义 Dialog。
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="outline">preset: {preset}</Badge>
                      <Badge variant="secondary">columns: {columns.length}</Badge>
                      <Badge variant="outline">
                        filters: {builtInQueryFields.length + queryFields.length}
                      </Badge>
                      {features.zebraRows ? <Badge variant="outline">striped</Badge> : null}
                      {features.compactColumns ? (
                        <Badge variant="outline">compact columns</Badge>
                      ) : null}
                      {features.compactRows ? (
                        <Badge variant="outline">compact rows</Badge>
                      ) : null}
                      {features.showEmptyState ? <Badge variant="outline">empty state</Badge> : null}
                      {features.rowActions ? <Badge>rowActions</Badge> : null}
                      {features.bulkUpdate ? <Badge>bulkUpdate</Badge> : null}
                      {features.bulkDelete ? <Badge>bulkDelete</Badge> : null}
                      {features.denseColumns ? <Badge>wide table</Badge> : null}
                      {features.createdAtQuery ? <Badge>createdAt query</Badge> : null}
                      {features.updatedAtQuery ? <Badge>updatedAt query</Badge> : null}
                      {features.auditQuery && auditQueryColumns.length > 0 ? (
                        <Badge>audit query</Badge>
                      ) : null}
                      {auditQueryColumns.length > 0 ? <Badge>audit columns</Badge> : null}
                    </div>

                    <div className="h-[620px] min-h-[620px] overflow-hidden rounded-2xl border border-(--app-border)">
                      <DataTable<CustomerRow, TableQuery>
                        columns={columns}
                        fetchData={fetchData}
                        getRowId={(row) => row.id}
                        caption={
                          features.customEmptyCopy
                            ? "Guide demo with custom empty experience"
                            : "Guide demo for DataTable configuration"
                        }
                        localeText={
                          features.customEmptyCopy
                            ? {
                                emptyText:
                                  "当前筛选下没有客户数据。你可以修改筛选条件后重试。",
                              }
                            : undefined
                        }
                        renderEmpty={
                          features.customEmptyCopy
                            ? () => (
                                <div className="flex min-h-[220px] flex-col items-center justify-center gap-3 text-center">
                                  <div className="text-sm font-medium">
                                    当前没有可展示的数据
                                  </div>
                                  <div className="max-w-[320px] text-sm text-(--app-muted-text)">
                                    这块可以替换成业务空态，比如引导创建首条记录，或者提示先完成筛选条件。
                                  </div>
                                </div>
                              )
                            : undefined
                        }
                        stripedRows={features.zebraRows}
                        compactColumns={features.compactColumns}
                        compactRows={features.compactRows}
                        auditColumns={auditQueryColumns}
                        height="100%"
                        stickyColumns={{
                          left: features.stickyLeft ? 2 : 0,
                          right: features.stickyRight ? 1 : 0,
                        }}
                        initialPageSize={10}
                        initialQuery={INITIAL_QUERY}
                        pageSizeOptions={[10, 20, 50]}
                        initialSort={{ columnKey: "createdAt", direction: "desc" }}
                        query={{
                          builtInFields: builtInQueryFields,
                          fields: queryFields,
                          audit: features.auditQuery,
                        }}
                        insert={
                          features.insert
                            ? {
                                label: "新增客户",
                                title: "新增客户示例",
                                description:
                                  "这块内容会放在 DataTable 自带的弹窗里，你可以接业务表单。",
                                renderContent: () => (
                                  <div className="grid gap-4 py-2">
                                    <Input
                                      value={insertDraft.name}
                                      onValueChange={(value) =>
                                        setInsertDraft((current) => ({
                                          ...current,
                                          name: value,
                                        }))
                                      }
                                      placeholder="客户名称"
                                    />
                                    <Input
                                      value={insertDraft.region}
                                      onValueChange={(value) =>
                                        setInsertDraft((current) => ({
                                          ...current,
                                          region: value,
                                        }))
                                      }
                                      placeholder="区域"
                                    />
                                  </div>
                                ),
                                onConfirm: async () => {
                                  await sleep(150)
                                  setRows((current) => [
                                    createInsertedCustomer(insertDraft),
                                    ...current,
                                  ])
                                  setInsertDraft({
                                    name: "New Workspace Customer",
                                    region: "East China",
                                  })
                                },
                              }
                            : false
                        }
                        selection={features.selection ? {} : false}
                        bulkUpdate={
                          features.bulkUpdate
                            ? {
                                title: "批量修改示例",
                                description:
                                  "常见场景是批量改状态、负责人、区域。",
                                fields: [
                                  {
                                    key: "status",
                                    label: "状态",
                                    type: "select",
                                    options: STATUSES.map((status) => ({
                                      label: status,
                                      value: status,
                                    })),
                                  },
                                  {
                                    key: "owner",
                                    label: "负责人",
                                    type: "select",
                                    options: OWNERS.map((owner) => ({
                                      label: owner,
                                      value: owner,
                                    })),
                                  },
                                ],
                                onSubmit: async ({
                                  fieldKey,
                                  selectedRowKeys,
                                  value,
                                }) => {
                                  await sleep(150)
                                  setRows((current) =>
                                    current.map((row) =>
                                      selectedRowKeys.includes(row.id)
                                        ? ({
                                            ...row,
                                            [fieldKey]: value,
                                          } as CustomerRow)
                                        : row
                                    )
                                  )
                                },
                              }
                            : false
                        }
                        bulkDelete={
                          features.bulkDelete
                            ? {
                                title: "批量删除示例",
                                description: (count) =>
                                  `当前会删除 ${count} 条数据。实际接入时这里通常连 API。`,
                                onDelete: async ({ selectedRowKeys }) => {
                                  await sleep(150)
                                  setRows((current) =>
                                    current.filter(
                                      (row) => !selectedRowKeys.includes(row.id)
                                    )
                                  )
                                },
                              }
                            : false
                        }
                        rowActions={
                          features.rowActions
                            ? {
                                sticky: features.stickyRight ? "right" : false,
                                edit: {
                                  title: (row) => `编辑 ${row.name}`,
                                  description: (row) =>
                                    `这里可以接你的编辑表单。当前示例会把 ${row.id} 标记成 Review。`,
                                  renderContent: ({ row }) => (
                                    <div className="grid gap-3 py-2">
                                      <Input
                                        value={row.name}
                                        onValueChange={() => {}}
                                        disabled
                                      />
                                      <Input
                                        value={row.region}
                                        onValueChange={() => {}}
                                        disabled
                                      />
                                    </div>
                                  ),
                                  onConfirm: async (row) => {
                                    await sleep(150)
                                    setRows((current) =>
                                      current.map((item) =>
                                        item.id === row.id
                                          ? { ...item, status: "Review" }
                                          : item
                                      )
                                    )
                                  },
                                },
                                delete: {
                                  title: (row) => `删除 ${row.name}？`,
                                  description: (row) =>
                                    `这行演示了单行删除文案。客户 ID：${row.id}`,
                                  onConfirm: async (row) => {
                                    await sleep(150)
                                    setRows((current) =>
                                      current.filter((item) => item.id !== row.id)
                                    )
                                  },
                                },
                                moreItems: [
                                  {
                                    key: "copy-id",
                                    label: "复制客户 ID",
                                    onClick: (row) => {
                                      void navigator.clipboard?.writeText(row.id)
                                    },
                                  },
                                  {
                                    key: "open-audit-dialog",
                                    label: "打开审计弹窗",
                                    onClick: (row) => {
                                      setAuditDialogRow(row)
                                    },
                                  },
                                ],
                              }
                            : false
                        }
                      />
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="config">
                  <div className="space-y-4">
                    <div className="rounded-2xl border border-(--app-border) bg-[#101828] p-4 text-sm text-slate-100">
                      <pre className="overflow-x-auto whitespace-pre-wrap font-mono leading-6">
                        {generatedSnippet}
                      </pre>
                    </div>
                    <p className="text-sm/6  text-(--app-muted-text)">
                      这个片段不是完整页面，而是告诉用户当前这组按钮会映射成哪些关键 props。
                    </p>
                  </div>
                </TabsContent>

                <TabsContent value="dialogs">
                  <div className="space-y-4">
                    <div className="rounded-2xl border border-(--app-border) bg-[#101828] p-4 text-sm text-slate-100">
                      <pre className="overflow-x-auto whitespace-pre-wrap font-mono leading-6">
                        {dialogSnippet}
                      </pre>
                    </div>
                    <div className="grid gap-3 lg:grid-cols-3">
                      <div className="rounded-2xl border border-(--app-border) p-4">
                        <div className="text-sm font-medium">新增弹窗</div>
                        <div className="mt-2 text-sm/6  text-(--app-muted-text)">
                          用 `insert.title`、`insert.description` 和
                          `insert.renderContent` 自定义表单内容。
                        </div>
                      </div>
                      <div className="rounded-2xl border border-(--app-border) p-4">
                        <div className="text-sm font-medium">编辑弹窗</div>
                        <div className="mt-2 text-sm/6  text-(--app-muted-text)">
                          `rowActions.edit.renderContent` 可以按当前行内容渲染不同表单。
                        </div>
                      </div>
                      <div className="rounded-2xl border border-(--app-border) p-4">
                        <div className="text-sm font-medium">更多菜单</div>
                        <div className="mt-2 text-sm/6  text-(--app-muted-text)">
                          `moreItems` 适合挂复制、跳转、打开二次确认弹窗这类扩展动作。
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent value="playbook">
                  <div id="playbook" className="grid gap-4">
                    {FIELD_PLAYBOOK.map((item) => (
                      <div
                        key={item.title}
                        className="rounded-2xl border border-(--app-border)"
                      >
                        <Card>
                          <CardHeader>
                            <CardTitle>
                              <span className="text-base">{item.title}</span>
                            </CardTitle>
                          </CardHeader>
                          <CardContent>
                            <div className="text-sm/6  text-(--app-muted-text)">
                              {item.description}
                            </div>
                          </CardContent>
                        </Card>
                      </div>
                    ))}
                  </div>
                </TabsContent>
              </Tabs>
                </div>
              </CardContent>
            </Card>
          </div>
        </section>

        <section id="playbook" className="grid gap-4 lg:grid-cols-3">
          <Card>
            <CardHeader>
              <CardTitle>
                <span className="flex items-center gap-2 text-base">
                  <ClipboardList className="size-4" />
                  第一步
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-sm/6  text-(--app-muted-text)">
                先定义 `Row` 和 `Query` 类型。DataTable 的复杂度主要来自这两个对象，不要先写 JSX。
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>
                <span className="flex items-center gap-2 text-base">
                  <PencilLine className="size-4" />
                  第二步
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-sm/6  text-(--app-muted-text)">
                再把 `columns`、`query.fields`、`rowActions` 按功能块拆开，业务页面会更容易维护。
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>
                <span className="flex items-center gap-2 text-base">
                  <Trash2 className="size-4" />
                  第三步
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-sm/6  text-(--app-muted-text)">
                最后才接 `insert`、`bulkUpdate`、`bulkDelete` 这些变更型操作，这样不会把表格页写成一团。
              </div>
            </CardContent>
          </Card>
        </section>

        <section className="rounded-[28px] border border-(--app-border) bg-(--app-surface) p-6">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">推荐对外展示方式</h2>
              <p className="text-sm/6  text-(--app-muted-text)">
                给用户看组件时，不要只放一个表格。最有效的是“场景按钮 + 实时示例 + props 片段 + 接入说明”四件套。
              </p>
            </div>
            <div className="hidden h-12 lg:block">
              <Separator orientation="vertical" />
            </div>
            <div className="grid gap-2 text-sm text-(--app-muted-text)">
              <div>1. 先选场景</div>
              <div>2. 再看表格效果</div>
              <div>3. 最后抄配置代码</div>
            </div>
          </div>
        </section>
      </div>

      <Dialog
        open={auditDialogRow !== null}
        onOpenChange={(open) => {
          if (!open) {
            setAuditDialogRow(null)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>自定义审计弹窗示例</DialogTitle>
            <DialogDescription>
              这个弹窗不是 DataTable 内建动作，而是通过 `moreItems[].onClick`
              从业务层自己打开的。
            </DialogDescription>
          </DialogHeader>
          {auditDialogRow ? (
            <div className="grid gap-3 rounded-xl border border-(--app-border) p-4 text-sm">
              <div>客户名称：{auditDialogRow.name}</div>
              <div>客户 ID：{auditDialogRow.id}</div>
              <div>当前状态：{auditDialogRow.status}</div>
              <div>负责人：{auditDialogRow.owner}</div>
              <div>创建时间：{formatDate(auditDialogRow.createdAt)}</div>
              <div>更新时间：{formatDate(auditDialogRow.updatedAt)}</div>
            </div>
          ) : null}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setAuditDialogRow(null)}
            >
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function renderFeatureChip(
  title: string,
  hint: string,
  checked: boolean,
  onCheckedChange: (value: boolean) => void
) {
  return (
    <div className="flex w-full items-center gap-1.5 rounded-lg border border-(--app-border) bg-(--app-surface) px-2 py-1.5">
      <div className="min-w-0 flex-1">
        <div className="truncate text-[13px]/4 font-medium ">{title}</div>
        <div className="truncate text-[9px]/3  text-(--app-muted-text)">
          {hint}
        </div>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  )
}

function renderDemoHint(title: string, description: string) {
  return (
    <div className="rounded-lg border border-(--app-border) px-2.5 py-2">
      <div className="text-[12px]/4 font-medium ">{title}</div>
      <div className="mt-1 text-[11px]/4  text-(--app-muted-text)">
        {description}
      </div>
    </div>
  )
}

function buildSnippet({
  features,
  preset,
}: {
  features: FeatureState
  preset: ScenarioPreset
}) {
  const lines = [
    `// preset: ${preset}`,
    `<DataTable<CustomerRow, TableQuery>`,
    `  columns={columns}`,
    `  fetchData={fetchData}`,
    `  getRowId={(row) => row.id}`,
    `  initialQuery={INITIAL_QUERY}`,
    `  initialPageSize={10}`,
    `  stickyColumns={{ left: ${features.stickyLeft ? 2 : 0}, right: ${features.stickyRight ? 1 : 0} }}`,
  ]

  const queryConfigLines: string[] = []

  if (features.keywordSearch) {
    queryConfigLines.push(`builtInFields: builtInQueryFields`)
  }

  if (features.advancedFilters) {
    queryConfigLines.push(`fields: queryFields`)
  }

  if (features.createdAtQuery || features.updatedAtQuery) {
    lines.push(`  auditColumns={auditColumns}`)
  }

  if (features.auditQuery) {
    queryConfigLines.push(`audit: true`)
  }

  if (queryConfigLines.length > 0) {
    lines.push(`  query={{ ${queryConfigLines.join(", ")} }}`)
  }

  if (features.insert) {
    lines.push(`  insert={{ label: "新增客户", onConfirm: handleInsert }}`)
  }

  if (features.selection) {
    lines.push(`  selection={{}}`)
  }

  if (features.bulkUpdate) {
    lines.push(`  bulkUpdate={{ fields: bulkUpdateFields, onSubmit: handleBulkUpdate }}`)
  }

  if (features.bulkDelete) {
    lines.push(`  bulkDelete={{ onDelete: handleBulkDelete }}`)
  }

  if (features.rowActions) {
    lines.push(`  rowActions={{ edit, delete, moreItems }}`)
  }

  if (!features.zebraRows) {
    lines.push(`  stripedRows={false}`)
  }

  if (features.compactColumns) {
    lines.push(`  compactColumns`)
  }

  if (features.compactRows) {
    lines.push(`  compactRows`)
  }

  if (features.showEmptyState) {
    lines.push(`  // empty state demo: fetchData returns { items: [], total: 0 }`)
  }

  if (features.customEmptyCopy) {
    lines.push(`  renderEmpty={() => <CustomEmptyState />}`)
  }

  lines.push(`/>`)

  return lines.join("\n")
}

function buildDialogSnippet() {
  return [
    `<DataTable<CustomerRow, TableQuery>`,
    `  insert={{`,
    `    title: "新增客户示例",`,
    `    description: "这里可以放自定义说明文案。",`,
    `    renderContent: ({ close }) => (`,
    `      <CustomInsertForm onCancel={close} />`,
    `    ),`,
    `    onConfirm: handleInsert,`,
    `  }}`,
    ``,
    `  rowActions={{`,
    `    edit: {`,
    `      title: (row) => \`编辑 \${row.name}\`,`,
    `      description: (row) => \`客户 ID: \${row.id}\`,`,
    `      renderContent: ({ row, close }) => (`,
    `        <CustomerEditor row={row} onCancel={close} />`,
    `      ),`,
    `      onConfirm: handleRowEdit,`,
    `    },`,
    `    delete: {`,
    `      title: (row) => \`删除 \${row.name}？\`,`,
    `      description: (row) => \`删除前请确认客户 ID: \${row.id}\`,`,
    `      onConfirm: handleRowDelete,`,
    `    },`,
    `    moreItems: [`,
    `      {`,
    `        key: "copy-id",`,
    `        label: "复制客户 ID",`,
    `        onClick: (row) => navigator.clipboard.writeText(row.id),`,
    `      },`,
    `      {`,
    `        key: "open-audit-dialog",`,
    `        label: "打开审计弹窗",`,
    `        onClick: (row) => openAuditDialog(row),`,
    `      },`,
    `    ],`,
    `  }}`,
    `/>`,
  ].join("\n")
}

function createCustomerRows() {
  return Array.from({ length: 56 }, (_, index) => {
    const idSeed = index + 1001
    const createdAt = new Date(2026, 0, 1 + index, 9 + (index % 8), 0, 0)
    const updatedAt = addDays(createdAt, 2 + (index % 9))

    return {
      id: `C-${idSeed}`,
      name: `Workspace Customer ${index + 1}`,
      tier: TIERS[index % TIERS.length],
      status: STATUSES[index % STATUSES.length],
      region: REGIONS[index % REGIONS.length],
      owner: OWNERS[index % OWNERS.length],
      channel: CHANNELS[index % CHANNELS.length],
      industry: INDUSTRIES[index % INDUSTRIES.length],
      createdAt,
      updatedAt,
      renewalAt: addDays(createdAt, 90),
      contractValue: 18000 + index * 1600,
      usageScore: 62 + (index % 31),
      lastActiveAt: addDays(createdAt, 7 + (index % 4)),
    } satisfies CustomerRow
  })
}

function createInsertedCustomer(draft: { name: string; region: string }): CustomerRow {
  const baseDate = new Date()

  return {
    id: `C-${Math.floor(baseDate.getTime() / 1000)}`,
    name: draft.name || "New Customer",
    tier: "Growth",
    status: "Active",
    region: draft.region || "East China",
    owner: "Alice",
    channel: "Direct",
    industry: "Retail",
    createdAt: baseDate,
    updatedAt: baseDate,
    renewalAt: addDays(baseDate, 90),
    contractValue: 28000,
    usageScore: 88,
    lastActiveAt: baseDate,
  }
}

function resolveStatusVariant(status: CustomerRow["status"]) {
  if (status === "Active") return "default"
  if (status === "Paused") return "secondary"
  return "outline"
}

function resolveSortValue(row: CustomerRow, columnKey: string) {
  switch (columnKey) {
    case "id":
      return row.id
    case "name":
      return row.name
    case "status":
      return row.status
    case "tier":
      return row.tier
    case "region":
      return row.region
    case "owner":
      return row.owner
    case "contractValue":
      return row.contractValue
    case "usageScore":
      return row.usageScore
    case "createdAt":
      return row.createdAt.getTime()
    case "updatedAt":
      return row.updatedAt.getTime()
    case "renewalAt":
      return row.renewalAt.getTime()
    case "lastActiveAt":
      return row.lastActiveAt.getTime()
    default:
      return row.id
  }
}

function resolveAuditDate(
  row: CustomerRow,
  field: TableQuery["auditField"] = "createdAt"
) {
  return field === "updatedAt" ? row.updatedAt : row.createdAt
}

function compareValues(left: string | number, right: string | number) {
  if (typeof left === "number" && typeof right === "number") {
    return left - right
  }

  return String(left).localeCompare(String(right), "zh-CN")
}

function formatDate(date: Date) {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date)
}

function sleep(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms))
}

function addDays(date: Date, days: number) {
  const value = new Date(date)
  value.setDate(value.getDate() + days)
  return value
}

function startOfDay(date: Date) {
  const value = new Date(date)
  value.setHours(0, 0, 0, 0)
  return value
}

function endOfDay(date: Date) {
  const value = new Date(date)
  value.setHours(23, 59, 59, 999)
  return value
}
