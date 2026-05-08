const baseUrl = process.env.COMMERCE_SMOKE_BASE_URL || "http://127.0.0.1:5173"
const username = process.env.COMMERCE_SMOKE_USERNAME || "codex_ybl_20260508_2125"
const password = process.env.COMMERCE_SMOKE_PASSWORD || "CodexTest_20260508!"
const stamp = new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14)

let accessToken = ""
const results = []

function summarize(value) {
  if (value == null) return value
  if (typeof value !== "object") return value
  if (Array.isArray(value)) return { length: value.length, sample: value[0] ?? null }
  const summary = {}
  for (const [key, entry] of Object.entries(value)) {
    if (Array.isArray(entry)) summary[key] = { length: entry.length, sample: entry[0] ?? null }
    else if (entry && typeof entry === "object") summary[key] = "[object]"
    else summary[key] = entry
  }
  return summary
}

async function call(name, method, path, body, options = {}) {
  const headers = { Accept: "application/json" }
  if (accessToken) headers.Authorization = `Bearer ${accessToken}`
  if (body !== undefined) headers["Content-Type"] = "application/json"

  const response = await fetch(`${baseUrl}${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  const text = await response.text()
  let json = null
  try {
    json = text ? JSON.parse(text) : null
  } catch {
    json = { raw: text.slice(0, 500) }
  }
  const ok =
    response.ok &&
    (json == null || typeof json.code !== "number" || json.code === 0 || options.allowNonZeroCode)
  const data = json && Object.prototype.hasOwnProperty.call(json, "data") ? json.data : json
  results.push({
    name,
    ok,
    status: response.status,
    code: json?.code,
    message: json?.message,
    summary: summarize(data),
  })
  if (!ok && !options.allowFailure) {
    const detail = JSON.stringify(json ?? text, null, 2)
    throw new Error(`${name} failed: HTTP ${response.status}\n${detail}`)
  }
  return data
}

async function tryCall(name, method, path, body) {
  try {
    return await call(name, method, path, body, { allowFailure: true })
  } catch (error) {
    results.push({
      name,
      ok: false,
      status: "exception",
      message: error instanceof Error ? error.message : String(error),
    })
    return null
  }
}

async function streamAgentSession(name, body, timeoutMs = 90000) {
  const headers = {
    Accept: "text/event-stream",
    "Content-Type": "application/json",
  }
  if (accessToken) headers.Authorization = `Bearer ${accessToken}`

  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeoutMs)
  let text = ""
  const events = []

  try {
    const response = await fetch(`${baseUrl}/api/sse/agent/session/stream`, {
      method: "POST",
      headers,
      body: JSON.stringify(body),
      signal: controller.signal,
    })

    if (!response.ok || !response.body) {
      const detail = await response.text()
      throw new Error(`SSE HTTP ${response.status}: ${detail.slice(0, 500)}`)
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder("utf-8")
    let buffer = ""
    let doneEvent = false

    while (!doneEvent) {
      const { value, done } = await reader.read()
      if (done) break
      buffer += decoder.decode(value, { stream: true })

      let boundary = buffer.indexOf("\n\n")
      while (boundary !== -1) {
        const block = buffer.slice(0, boundary)
        buffer = buffer.slice(boundary + 2)
        const parsed = parseSseBlock(block)
        if (parsed.event) events.push(parsed.event)
        if (parsed.event === "delta") {
          const data = safeParseJson(parsed.data)
          if (typeof data?.content === "string") text += data.content
        }
        if (parsed.event === "error") {
          const data = safeParseJson(parsed.data)
          throw new Error(data?.message || parsed.data || "agent stream error")
        }
        if (parsed.event === "done" || parsed.event === "finished") {
          doneEvent = true
          break
        }
        boundary = buffer.indexOf("\n\n")
      }
    }

    const ok = text.trim().length > 0
    results.push({
      name,
      ok,
      status: response.status,
      message: ok ? "stream returned model text" : "stream ended without model text",
      summary: {
        events,
        textPreview: text.trim().slice(0, 240),
      },
    })
    if (!ok) throw new Error(`${name} failed: stream ended without model text`)
    return text
  } catch (error) {
    results.push({
      name,
      ok: false,
      status: "exception",
      message: error instanceof Error ? error.message : String(error),
      summary: {
        events,
        textPreview: text.trim().slice(0, 240),
      },
    })
    throw error
  } finally {
    clearTimeout(timer)
  }
}

function parseSseBlock(block) {
  let event = ""
  const data = []
  for (const line of block.split("\n")) {
    const trimmed = line.trimEnd()
    if (trimmed.startsWith("event:")) event = trimmed.slice("event:".length).trim()
    if (trimmed.startsWith("data:")) data.push(trimmed.slice("data:".length).trimStart())
  }
  return { event, data: data.join("\n") }
}

function safeParseJson(raw) {
  try {
    return JSON.parse(raw)
  } catch {
    return null
  }
}

async function login() {
  const data = await call("auth.login", "POST", "/auth/session/login", {
    identifier: username,
    password,
  })
  accessToken = data.accessToken
  const me = await call("auth.me", "GET", "/auth/user/me")
  const permissions = await tryCall("app.permissions.beforeEnsure", "GET", "/api/app/me/permissions")
  if (!permissions) {
    await call("app.register", "POST", "/api/app/register", {
      displayId: me.display_user_id || me.id,
      displayName: me.display_name || me.username || username,
      remark: "smoke connectivity account",
    })
  }
  await call("app.permissions", "GET", "/api/app/me/permissions")
}

async function run() {
  await login()

  await call("products.dashboard", "GET", "/api/app/products/dashboard")
  await call("products.list", "GET", "/api/app/products?page=1&pageSize=20")
  const product = await call("products.create", "POST", "/api/app/products", {
    name: `Smoke Product ${stamp}`,
    platform: "tmall",
    externalId: `smoke-${stamp}`,
    category: "skincare",
    brand: "Codex",
    description: "End-to-end connectivity smoke product.",
    price: "199.00",
    currency: "CNY",
    metadata: { smoke: true, stamp },
  })
  await call("products.get", "GET", `/api/app/products/${product.id}`)
  await call("products.update", "PATCH", `/api/app/products/${product.id}`, {
    description: "Updated by smoke test.",
    metadata: { smoke: true, updated: true, stamp },
  })
  const knowledge = await call(
    "products.knowledge.create",
    "POST",
    `/api/app/products/${product.id}/knowledge`,
    {
      title: "Core selling points",
      contentType: "text",
      content: "Main user benefit, audience, scenario, objections.",
      sourceType: "manual",
      confidence: 0.88,
      metadata: { smoke: true },
    }
  )
  await call("products.knowledge.list", "GET", `/api/app/products/${product.id}/knowledge`)
  const assetSign = await call(
    "products.asset.presign",
    "POST",
    `/api/app/products/${product.id}/assets/presign-upload`,
    {
      extension: "txt",
      contentType: "text/plain",
    }
  )
  await call("products.asset.create", "POST", `/api/app/products/${product.id}/assets`, {
    bucket: assetSign.bucket,
    objectKey: assetSign.objectKey,
    assetType: "document",
    fileName: "hero.txt",
    mimeType: "text/plain",
    fileSize: 12,
    description: "Registered smoke asset.",
    metadata: { smoke: true },
  })
  const material = await call(
    "products.material.create",
    "POST",
    `/api/app/products/${product.id}/materials`,
    {
      materialType: "listing_copy",
      title: "Listing copy",
      content: "Smoke test product copy.",
      metadata: { smoke: true },
    }
  )
  await call(
    "products.material.update",
    "PATCH",
    `/api/app/products/${product.id}/materials/${material.id}`,
    {
      title: "Updated listing copy",
      content: "Updated smoke copy.",
      metadata: { smoke: true, updated: true },
    }
  )
  const competitor = await call(
    "products.competitor.create",
    "POST",
    `/api/app/products/${product.id}/competitors`,
    {
      name: `Smoke Competitor ${stamp}`,
      platform: "tmall",
      url: "https://example.com/competitor",
      price: 188,
      rating: 4.7,
      monthlySales: "1000+",
      notes: "Smoke competitor.",
      searchResults: { smoke: true },
      metadata: { smoke: true },
    }
  )
  await call(
    "products.competitor.change.create",
    "POST",
    `/api/app/products/${product.id}/competitors/${competitor.id}/changes`,
    {
      changeType: "price",
      oldValue: "199",
      newValue: "188",
      summary: "Smoke price change.",
      sourceType: "manual",
      metadata: { smoke: true },
    }
  )

  await call("campaigns.list", "GET", "/api/app/campaigns?page=1&pageSize=20")
  const campaign = await call("campaigns.create", "POST", "/api/app/campaigns", {
    productId: product.id,
    name: `Smoke Campaign ${stamp}`,
    campaignType: "promotion",
    description: "Smoke campaign.",
    budget: 1000,
    startDate: "2026-05-08",
    endDate: "2026-06-08",
    status: "active",
    metadata: { smoke: true },
    output: {},
    actionMetadata: {},
  })
  await call("campaigns.get", "GET", `/api/app/campaigns/${campaign.id}`)
  await call("campaigns.update", "PATCH", `/api/app/campaigns/${campaign.id}`, {
    status: "active",
    metadata: { smoke: true, updated: true },
    output: {},
    actionMetadata: {},
  })

  await call("workspaces.list", "GET", "/api/app/workspaces?page=1&pageSize=20")
  const workspace = await call("workspaces.create", "POST", "/api/app/workspaces", {
    productId: product.id,
    campaignId: campaign.id,
    title: `Smoke Workspace ${stamp}`,
    workspaceType: "launch",
    config: { smoke: true },
  })
  const task = await call("workspaces.task.create", "POST", `/api/app/workspaces/${workspace.id}/tasks`, {
    title: "Write listing optimization plan",
    description: "Smoke workspace task.",
    ownerRole: "operator",
    status: "pending",
    priority: 5,
    acceptanceCriteria: "Plan includes audience and action list.",
    metadata: { smoke: true },
  })
  const run = await call("workspaces.run.create", "POST", `/api/app/workspaces/${workspace.id}/runs`, {
    goal: "Generate a launch operation plan.",
    requirements: ["Use product knowledge", "Respect platform constraints"],
    status: "planned",
    source: "manual",
    summary: "Smoke run.",
    report: {},
    metadata: { smoke: true },
  })
  await call("workspaces.runTask.create", "POST", `/api/app/workspaces/${workspace.id}/runs/${run.id}/tasks`, {
    workspaceTaskId: task.id,
    seq: 1,
    ownerRole: "operator",
    status: "pending",
    attempts: 0,
    metadata: { smoke: true },
  })
  const execution = await call("execution.create", "POST", "/api/app/execution-queue", {
    workspaceId: workspace.id,
    runId: run.id,
    workspaceTaskId: task.id,
    actionType: "publish_draft",
    description: "Smoke execution request.",
    platform: "tmall",
    payload: { smoke: true },
    expectedImpact: "Verify approval queue connectivity.",
  })
  await call("execution.approve", "POST", `/api/app/execution-queue/${execution.id}/approve`, {
    reason: "Smoke approve.",
    result: { smoke: true },
  })

  await call("platforms.catalog", "GET", "/api/app/platform-connect/platforms")
  const platformConnection = await call("platforms.connection.save", "POST", "/api/app/platform-connect/connections", {
    platform: "taobao",
    appKey: `smoke-key-${stamp}`,
    appSecret: "smoke-secret",
    accessToken: "smoke-token",
    refreshToken: "smoke-refresh",
    shopId: `shop-${stamp}`,
    shopName: "Smoke Shop",
    enabled: true,
    syncStatus: "idle",
    syncMessage: "Smoke connection.",
    extra: { smoke: true },
  })
  await call("platforms.connection.get", "GET", `/api/app/platform-connect/connections/${platformConnection.id}`)
  await call("platforms.metric.save", "POST", "/api/app/platform-connect/metrics", {
    platform: "taobao",
    metricDate: "2026-05-08",
    metricKey: "gmv",
    metricValue: 1234.5,
    dimensionKey: "shop",
    dimensionValue: "Smoke Shop",
    source: "smoke",
  })
  await call("platforms.summary", "GET", "/api/app/platform-connect/summary")
  await call("platforms.syncStatus", "GET", "/api/app/platform-connect/sync-status")
  await call("channels.catalog", "GET", "/api/app/platform-connect/channels")
  const channel = await call("channels.connection.save", "POST", "/api/app/platform-connect/channels/connections", {
    channel: "wecom",
    webhookUrl: "https://example.com/webhook",
    botToken: "smoke-bot",
    chatId: "smoke-chat",
    secret: "smoke-secret",
    extraCredentials: { smoke: true },
    enabled: true,
  })
  await call("channels.connection.get", "GET", `/api/app/platform-connect/channels/connections/${channel.id}`)

  const approval = await call("approval.create", "POST", "/api/app/approvals/tool-requests", {
    workspaceId: workspace.id,
    runId: run.id,
    requestRole: "operator",
    skillName: "publish_draft",
    toolCallId: `tool-${stamp}`,
    riskLevel: "low",
    args: { smoke: true },
    actionKey: `publish_draft_${stamp}`,
    reason: "Smoke approval.",
  })
  await call("approval.reject", "POST", `/api/app/approvals/tool-requests/${approval.id}/reject`, {
    reason: "Smoke reject after creation.",
    result: { smoke: true },
  })
  await call("audit.create", "POST", "/api/app/approvals/audit-logs", {
    workspaceId: workspace.id,
    runId: run.id,
    action: "smoke.audit",
    targetType: "workspace",
    targetId: workspace.id,
    status: "ok",
    reason: "Smoke audit log.",
    metadata: { smoke: true },
  })

  const longTask = await call("operations.longTask.create", "POST", "/api/app/long-tasks", {
    workspaceId: workspace.id,
    runId: run.id,
    taskType: "analysis",
    title: "Smoke long task",
    status: "running",
    progress: 20,
    totalSteps: 5,
    currentStep: "Smoke step",
    role: "analyst",
    inputParams: { smoke: true },
  })
  await call("operations.longTask.update", "PATCH", `/api/app/long-tasks/${longTask.id}`, {
    status: "completed",
    progress: 100,
    currentStep: "Done",
    result: { smoke: true },
  })
  const briefing = await call("briefing.save", "POST", "/api/app/briefing/daily-records", {
    briefingDate: "2026-05-08",
    title: `Smoke Briefing ${stamp}`,
    summary: "Smoke daily briefing.",
    sections: { smoke: true },
    source: "manual",
    status: "published",
    archived: false,
  })
  await call("briefing.get", "GET", `/api/app/briefing/daily-records/${briefing.id}`)
  const event = await call("notifications.create", "POST", "/api/app/briefing/notifications", {
    eventType: "smoke",
    title: "Smoke notification",
    data: { smoke: true },
    consumed: false,
  })
  await call("notifications.consume", "POST", `/api/app/briefing/notifications/${event.id}/consume`)
  const alert = await call("alerts.create", "POST", "/api/app/briefing/alerts", {
    workspaceId: workspace.id,
    runId: run.id,
    alertType: "smoke",
    severity: "low",
    message: "Smoke alert.",
    role: "operator",
    dedupKey: `smoke-${stamp}`,
    payload: { smoke: true },
  })
  await call("alerts.summary", "GET", "/api/app/briefing/alerts/summary")
  await call("alerts.dismiss", "POST", `/api/app/briefing/alerts/${alert.id}/dismiss`, {
    reason: "Smoke dismiss.",
  })

  await call("packages.catalog", "GET", "/api/app/packages/catalog?page=1&pageSize=20")
  const pkg = await call("packages.save", "POST", "/api/app/packages", {
    packageType: "skill",
    packageId: `smoke-skill-${stamp}`,
    packageName: "Smoke Skill Package",
    version: "0.0.1",
    status: "active",
    manifestPath: `packages/smoke-skill-${stamp}.json`,
    dependencies: {},
    compatibility: {},
    metadata: { smoke: true },
    reason: "Smoke package.",
  })
  await call("packages.get", "GET", `/api/app/packages/${pkg.packageType}/${pkg.packageId}`)
  await call("packages.validate", "GET", `/api/app/packages/${pkg.packageType}/${pkg.packageId}/validate`)
  await call("packages.history", "GET", `/api/app/packages/${pkg.packageType}/${pkg.packageId}/history?limit=20`)
  await call("packages.lock.save", "POST", `/api/app/workspaces/${workspace.id}/runs/${run.id}/package-locks`, {
    entries: [
      {
        packageType: pkg.packageType,
        packageId: pkg.packageId,
        packageName: pkg.packageName,
        lockedVersion: pkg.version,
        lockedStatus: pkg.status,
        required: true,
        lockSource: "smoke",
        manifestPath: pkg.manifestPath,
        notes: "Smoke package lock.",
        metadata: { smoke: true },
      },
    ],
  })
  await call("packages.lock.drift", "GET", `/api/app/workspaces/${workspace.id}/runs/${run.id}/package-locks/drift`)

  const conversation = await call("conversations.create", "POST", "/api/app/conversations", {
    title: `Smoke Conversation ${stamp}`,
    firstMessage: "请用一句话说明 2+2 等于几。",
    agentId: "web-agent",
  })
  await streamAgentSession("agent.stream.answer", {
    input: "请用一句中文回答：2+2 等于几？",
    agentId: "web-agent",
    sessionId: conversation.conversation.id,
    workspaceId: workspace.id,
  })

  const failed = results.filter((item) => !item.ok)
  console.log(JSON.stringify({ baseUrl, total: results.length, failed: failed.length, results }, null, 2))
  if (failed.length) process.exitCode = 1
}

run().catch((error) => {
  console.error(error)
  console.log(JSON.stringify({ baseUrl, total: results.length, failed: results.filter((item) => !item.ok).length, results }, null, 2))
  process.exitCode = 1
})
