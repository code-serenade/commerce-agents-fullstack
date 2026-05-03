export const metricCards = [
  {
    labelKey: "admin.dashboard.metrics.revenue",
    value: "¥ 284.6k",
    delta: "+12.4%",
    marker: "GMV",
  },
  {
    labelKey: "admin.dashboard.metrics.newUsers",
    value: "1,842",
    delta: "+6.8%",
    marker: "USR",
  },
  {
    labelKey: "admin.dashboard.metrics.pendingTickets",
    value: "27",
    delta: "-8.1%",
    marker: "TKT",
  },
  {
    labelKey: "admin.dashboard.metrics.availability",
    value: "99.96%",
    delta: "+0.2%",
    marker: "SLA",
  },
]

export const revenueTrend = [
  { dayKey: "admin.dashboard.days.mon", revenue: 128, orders: 92 },
  { dayKey: "admin.dashboard.days.tue", revenue: 164, orders: 118 },
  { dayKey: "admin.dashboard.days.wed", revenue: 152, orders: 124 },
  { dayKey: "admin.dashboard.days.thu", revenue: 196, orders: 148 },
  { dayKey: "admin.dashboard.days.fri", revenue: 238, orders: 172 },
  { dayKey: "admin.dashboard.days.sat", revenue: 214, orders: 156 },
  { dayKey: "admin.dashboard.days.sun", revenue: 286, orders: 198 },
]

export const channelStats = [
  { channelKey: "admin.dashboard.channels.directStore", value: 42 },
  { channelKey: "admin.dashboard.channels.partner", value: 28 },
  { channelKey: "admin.dashboard.channels.miniProgram", value: 22 },
  { channelKey: "admin.dashboard.channels.offline", value: 17 },
]

export const userSegments = [
  {
    key: "new",
    labelKey: "admin.dashboard.userSegments.new",
    value: 38,
  },
  {
    key: "active",
    labelKey: "admin.dashboard.userSegments.active",
    value: 44,
  },
  {
    key: "silent",
    labelKey: "admin.dashboard.userSegments.silent",
    value: 18,
  },
]

export const serviceStatus = [
  {
    serviceKey: "admin.dashboard.services.apiGateway",
    statusKey: "admin.dashboard.serviceStatuses.normal",
    metric: "38ms",
  },
  {
    serviceKey: "admin.dashboard.services.jobQueue",
    statusKey: "admin.dashboard.serviceStatuses.normal",
    metric: "12 jobs",
  },
  {
    serviceKey: "admin.dashboard.services.notifications",
    statusKey: "admin.dashboard.serviceStatuses.watching",
    metric: "96.4%",
  },
]

export const orders = [
  {
    id: "SO-20260426-01",
    sourceKey: "admin.dashboard.orderSources.eastChina",
    statusKey: "admin.dashboard.orderStatuses.reviewing",
    amount: "¥ 128,000",
  },
  {
    id: "SO-20260426-02",
    sourceKey: "admin.dashboard.orderSources.directStore",
    statusKey: "admin.dashboard.orderStatuses.shipped",
    amount: "¥ 52,600",
  },
  {
    id: "SO-20260426-03",
    sourceKey: "admin.dashboard.orderSources.partner",
    statusKey: "admin.dashboard.orderStatuses.pendingPayment",
    amount: "¥ 310,500",
  },
  {
    id: "SO-20260426-04",
    sourceKey: "admin.dashboard.orderSources.miniProgram",
    statusKey: "admin.dashboard.orderStatuses.completed",
    amount: "¥ 86,200",
  },
]

export const riskItems = [
  { labelKey: "admin.dashboard.risks.refundBacklog", value: 62 },
  { labelKey: "admin.dashboard.risks.permissionReview", value: 38 },
  { labelKey: "admin.dashboard.risks.inventoryDelay", value: 14 },
]
