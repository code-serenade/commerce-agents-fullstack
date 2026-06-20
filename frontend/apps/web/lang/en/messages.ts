const messages = {
  web: {
    shell: {
      brand: {
        eyebrow: "电商多智能体",
        title: "电商多智能体",
        description: "Ecommerce AI training workspace",
      },
      navigation: {
        mobileLabel: "Primary navigation",
        workspace: "Training navigation",
        governance: "Manage and review",
        agentSection: "Training outputs",
        home: "Data diagnosis guidance",
        products: "Product selection analysis",
        materials: "Upload file library",
        campaigns: "Campaign planning practice",
        workspaces: "Training tasks and deliverables",
        platformConnect: "Platform data",
        approvals: "Teacher review records",
        operations: "Business result tracking",
        agent: "AI ecommerce partner",
        agentOutput: "Review report",
        profile: "Profile",
      },
      topbar: {
        meta: "Personal training space",
      },
      account: {
        defaultName: "Member",
        defaultId: "member@example.com",
        guestName: "Guest",
        guestId: "Not signed in",
        guestFallback: "G",
        login: "Sign in",
        register: "Register",
        logout: "Log out",
        actions: {
          profile: "Profile",
        },
      },
    },
  },
  commerce: {
    common: {
      all: "All",
      archive: "Archive",
      approve: "Approve",
      reject: "Reject",
      edit: "Edit",
      save: "Save",
      cancel: "Cancel",
      download: "Download",
      disable: "Disable",
      validate: "Validate",
      activate: "Activate",
      deactivate: "Deactivate",
      rollback: "Rollback",
      close: "Close",
      process: "Process",
      snooze: "Snooze",
      markRead: "Mark read",
      search: "Search",
      fillSample: "Fill sample",
      smartFill: "Smart fill",
      templateSmartFill: "Template fill",
      agentSmartFill: "Agent fill",
      smartFillReviewNotice:
        "Smart fill only writes a draft into this form and does not save it. Review the fields, then save to create the record.",
      smartFillProcess: {
        agentRoute: "Draft generated through the system agent",
        templateRoute: "Draft generated through template rules",
        reviewRequired: "Human review is required before saving",
        invalidJsonRejected: "Invalid model JSON is rejected before form fill",
        streamStarted: "Form draft generation started",
        streamInputPreflighted: "Input fields prepared",
        streamAgentStarted: "The system agent is organizing fields",
        streamValidationStarted: "Model output is being validated",
        streamValidationPassed: "Output validation passed",
        streamReviewReady: "Draft is ready for human review",
        streamFailed: "Draft generation failed; the form was not filled",
        separator: "; ",
      },
      loading: "Loading",
      status: "Status",
      actions: "Actions",
      decision: "Decision",
      type: "Type",
      source: "Source",
      platform: "Platform",
      channel: "Channel",
      document: "Docs",
      value: "Value",
      version: "Version",
      time: "Time",
      reason: "Reason",
      none: "Not set",
      noContent: "No content",
      manualApprove: "Frontend manual approval",
      manualReject: "Frontend manual rejection",
      manualCancel: "Frontend manual cancellation",
      manualClose: "Frontend manual close",
      manualEnable: "Frontend enable",
      manualDisable: "Frontend disable",
      manualRollback: "Frontend rollback",
      dialogDescription:
        "Fill the training-task fields and save. Nothing is written before saving.",
      countPending: "{{count}} pending",
      countRunning: "{{count}} running",
      countEnabled: "{{count}} enabled",
      countUnread: "{{count}} unread",
      countActive: "{{count}} active",
      countDone: "{{count}} done",
      driftCount: "{{count}} drift items found",
    },
    fields: {
      productName: "Product name",
      platformProductId: "Platform product ID",
      category: "Category",
      brand: "Brand",
      price: "Price",
      currency: "Currency",
      description: "Description",
      title: "Title",
      contentType: "Content type",
      sourceType: "Source type",
      confidence: "Confidence",
      content: "Content",
      assetType: "Asset type",
      fileName: "File name",
      objectKey: "Object key required without upload",
      mimeType: "MIME type",
      fileSize: "File size",
      note: "Note",
      materialType: "Material type",
      competitorName: "Competitor name",
      rating: "Rating",
      monthlySales: "Monthly sales",
      notes: "Notes",
      changeType: "Change type",
      oldValue: "Old value",
      newValue: "New value",
      summary: "Summary",
      campaignName: "Campaign name",
      productId: "Product ID",
      campaignId: "Campaign ID",
      campaignType: "Campaign type",
      budget: "Budget",
      startDate: "Start date",
      endDate: "End date",
      workspaceTitle: "Workspace title",
      workspaceType: "Workspace type",
      ownerRole: "Owner role",
      priority: "Priority",
      acceptanceCriteria: "Acceptance criteria",
      goal: "Run goal",
      requirements: "Requirements, one per line",
      workspaceTaskId: "Workspace task ID",
      seq: "Sequence",
      role: "Role",
      attempts: "Attempts",
      acceptanceScore: "Acceptance score",
      acceptancePassed: "Accepted true/false",
      acceptanceNotes: "Acceptance notes",
      executionNotes: "Execution notes",
      actionType: "Action type",
      expectedImpact: "Expected impact",
      suggestionDescription: "Suggestion description",
      skillName: "Tool or capability name",
      riskLevel: "Risk level",
      params: "Params",
      workspaceId: "Workspace ID",
      runId: "Run ID",
      conversationId: "Conversation ID",
      requestRole: "Request role",
      toolCallId: "Tool call ID",
      actionKey: "Action key",
      action: "Action",
      targetType: "Target type",
      targetId: "Target ID",
      executionQueueId: "Execution suggestion ID",
      toolApprovalId: "Tool approval ID",
      requestId: "Request ID",
      taskType: "Task type",
      progress: "Progress",
      totalSteps: "Total steps",
      currentStep: "Current step",
      errorMessage: "Error message",
      briefingDate: "Briefing date",
      eventType: "Event type",
      consumed: "Consumed true/false",
      alertType: "Alert type",
      severity: "Severity",
      message: "Message",
      dedupKey: "Dedup key",
      snoozedUntil: "Snoozed until",
      packageType: "Package type",
      packageId: "Package ID",
      packageName: "Package name",
      manifestPath: "Manifest path",
      lockedVersion: "Locked version",
      lockedStatus: "Locked status",
      required: "Required true/false",
      lockSource: "Lock source",
      appKey: "App Key",
      appSecret: "App Secret",
      tokenExpiresAt: "Token expiry",
      shopId: "Shop ID",
      shopName: "Shop name",
      enabled: "Enabled true/false",
      syncStatus: "Sync status",
      syncMessage: "Sync message",
      webhookUrl: "Webhook URL",
      url: "URL",
      accessToken: "Access Token",
      refreshToken: "Refresh Token",
      botToken: "Bot Token",
      chatId: "Chat ID",
      secret: "Secret",
      traceId: "Trace ID",
      metricDate: "Metric date",
      metricKey: "Metric key",
      metricValue: "Metric value",
      dimensionKey: "Dimension key",
      dimensionValue: "Dimension value",
    },
    pages: {
      overview: {
        eyebrow: "Training Console",
        title: "Data diagnosis guidance",
        description:
          "Start with products, campaigns, operating data, and pending items to decide what this training task should solve first.",
        boundaryTitle: "What students can do now",
        boundaryDescription:
          "This page helps students understand available materials and abnormal signals. It does not change platform data or publish content automatically.",
      },
      products: {
        eyebrow: "Product Training",
        title: "Product selection analysis",
        description:
          "Review products, product knowledge, file assets, material versions, competitors, and changes as context for selection analysis and AI training questions.",
      },
      files: {
        eyebrow: "Upload File Library",
        title: "Upload file library",
        description:
          "Manage chat-uploaded files and files uploaded in the upload file library that the AI can read and that can be selected for the current turn before students ask a question.",
        boundaryTitle: "Upload file usage boundary",
        boundaryDescription:
          "The upload file library only manages uploaded files and current-turn selection. It does not publish content or modify platform data.",
      },
      campaigns: {
        eyebrow: "Campaign Practice",
        title: "Campaign planning practice",
        description:
          "Practice campaign planning around goals, budgets, periods, outputs, and action metadata.",
      },
      workspaces: {
        eyebrow: "Training Tasks",
        title: "Training tasks and deliverables",
        description:
          "Carry AI output into tasks, runs, snapshots, and execution suggestions so student work becomes checkable deliverables.",
        boundaryTitle: "Execution note",
        boundaryDescription:
          "This phase stores execution records and suggestion decisions. It does not directly call Agent runtime or replay platform tools.",
      },
      platform: {
        eyebrow: "Platform Data",
        title: "Platform data",
        description:
          "Check whether Douyin, Qianchuan, and other platform data are connected so clicks, conversion, ROI, and other metrics can support training analysis.",
        boundaryTitle: "Data note",
        boundaryDescription:
          "This area is mainly for reading and checking data sources. Real platform writes or publishing actions require review.",
      },
      approvals: {
        eyebrow: "Review Records",
        title: "Teacher review records",
        description:
          "Review student submissions, teacher decisions, returned revisions, and key operation records so the training process stays checkable.",
        boundaryTitle: "Review note",
        boundaryDescription:
          "This area records review results and comments. High-risk platform actions cannot bypass teacher or admin confirmation.",
      },
      operations: {
        eyebrow: "Operating Results",
        title: "Business result tracking",
        description:
          "Track long-task records, daily briefings, background notifications, and alerts so students can review operating process and anomalies.",
        boundaryTitle: "Result note",
        boundaryDescription:
          "Use this area to observe progress, anomalies, and results, then support the next review report.",
      },
    },
    overview: {
      metrics: {
        products: "Products",
        productsTail: "Start with product materials, selling points, and competitors",
        campaigns: "Campaigns",
        campaignsTail: "Then plan goals, budgets, and cadence",
        workspaces: "Training tasks",
        workspacesTail: "Store tasks, submissions, and review materials",
        pending: "Pending",
        pendingTail: "Anomalies, execution queue, and missing materials",
      },
      panels: {
        productContext: "Start product analysis",
        productContextDesc:
          "Review product materials, selling points, assets, and competitors to decide which product this training task should analyze.",
        campaigns: "Plan the campaign",
        campaignsDesc:
          "Fill campaign goals, budget, period, and outputs to form a submittable campaign plan.",
        workspaces: "Submit training task",
        workspacesDesc:
          "Store AI output, checklists, and revisions as deliverables teachers can inspect.",
        platform: "Check platform data",
        platformDesc:
          "Check Douyin, Qianchuan, and other metrics for diagnosis and review.",
        governance: "Check risk reminders",
        governanceDesc:
          "Check anomaly reminders, execution queue items, and missing training materials.",
        enterProducts: "Start product analysis",
        enterCampaigns: "Start campaign planning",
        enterWorkspaces: "Submit training task",
        enterPlatform: "Check platform data",
        enterOperations: "Check operating results",
      },
      columns: {
        product: "Product",
        campaign: "Campaign",
        workspace: "Training task",
        platform: "Platform",
        gmv30d: "30d GMV",
        roi: "ROI",
        item: "Item",
      },
      empty: {
        products: "No product materials yet. Add one product for training analysis first.",
        campaigns: "No campaign plan yet. Create a campaign planning exercise first.",
        workspaces: "No training task yet. Generate or submit one from the AI training assistant.",
        platforms: "No platform data yet. Connect data before diagnosis.",
        governance: "No risk reminder or pending item right now.",
      },
      itemTypes: {
        execution: "Training suggestion",
        approval: "Risk reminder",
      },
    },
      products: {
        samples: {
          product: {
            brand: "Galaxy Cup",
            category: "smart thermos",
            description:
              "A temperature-display thermos for students and office commuters, focused on leak prevention, temperature reminders, and carry use.",
            name: "Galaxy Cup temperature-display thermos",
          },
        },
        metrics: {
        active: "Active products",
        activeTail: "{{total}} total, {{archived}} archived",
        knowledge: "Knowledge items",
        knowledgeTail: "Product context and selling point storage",
        materials: "Materials and assets",
        materialsTail:
          "{{assets}} file assets, {{materials}} content materials",
        competitors: "Competitors",
        competitorsTail: "Manual competitors and change history",
      },
      panels: {
        list: "Products to analyze",
        listDesc:
          "Choose one product, then review selling points, assets, price band, and competitors to form a product-selection judgment.",
        create: "Add training product",
        search: "Search products, platform, category, or brand",
        empty: "No product materials yet. Add one product for training analysis first.",
        detail: "Product analysis materials",
        detailDesc:
          "Select a product to review selling points, assets, knowledge, and competitor clues.",
        noProduct: "No product to analyze",
        selectedDesc:
          "Organize knowledge, assets, competitors, and changes around this product.",
        edit: "Edit product",
        archive: "Archive product",
        knowledgeTab: "Knowledge",
        assetsTab: "Assets",
        materialsTab: "Materials",
        competitorsTab: "Competitors",
        createKnowledge: "Add product knowledge",
        registerAsset: "Upload file material",
        createMaterial: "Add marketing material",
        materialVersions: "Material versions",
        materialVersionsDesc:
          "Compare different versions of the same material to decide which one fits the campaign.",
        createCompetitor: "Add competitor",
        competitorChanges: "Competitor changes",
        competitorChangesDesc:
          "Manually record price, sales, rating, or strategy changes",
        recordChange: "Record change",
      },
      columns: {
        product: "Product",
        knowledge: "Knowledge",
        asset: "Asset",
        material: "Material",
        competitor: "Competitor",
        confidence: "Confidence",
        change: "Change",
      },
      empty: {
        knowledge: "No product knowledge",
        assets: "No file assets",
        materials: "No product materials",
        materialVersions: "Select a material to view same-type versions",
        competitors: "No competitor records",
        changes: "No competitor change records",
      },
      dialogs: {
        create: "Create product",
        edit: "Edit product",
        knowledge: "Create product knowledge",
        editKnowledge: "Edit product knowledge",
        asset: "Register product asset",
        editAsset: "Edit product asset",
        material: "Create product material",
        editMaterial: "Edit product material",
        competitor: "Add competitor",
        editCompetitor: "Edit competitor",
        change: "Record competitor change",
        uploadFile: "Upload file",
      },
      notify: {
        created: "Product created",
        updated: "Product updated",
        archived: "Product archived",
        knowledgeCreated: "Product knowledge created",
        knowledgeUpdated: "Product knowledge updated",
        knowledgeArchived: "Product knowledge archived",
        materialCreated: "Material created",
        materialUpdated: "Material updated",
        materialArchived: "Material archived",
        assetCreated: "Asset registered",
        assetUpdated: "Asset updated",
        assetArchived: "Asset archived",
        competitorCreated: "Competitor created",
        competitorUpdated: "Competitor updated",
        competitorArchived: "Competitor archived",
        changeCreated: "Competitor change recorded",
        draftFilled: "Product draft filled into the form. Review before saving.",
      },
    },
      campaigns: {
        samples: {
          campaign: {
            description:
              "Run a 7-day short-video validation around temperature display, leak prevention, and commuting scenes. Track click rate, saves, and add-to-cart rate.",
            name: "Temperature thermos 7-day validation campaign",
          },
        },
        metrics: {
        total: "Campaigns",
        totalTail: "Current query scope",
        active: "Active",
        activeTail: "{{draft}} drafts, {{paused}} paused",
        budget: "Total budget",
        budgetTail: "From campaign record fields",
        ended: "Ended",
        endedTail: "Outputs and action metadata retained",
      },
      panel: {
        title: "Campaign planning plan",
        description:
          "Fill the training goal, campaign budget, period, creative direction, and review metrics to form a submittable campaign plan.",
        create: "Create campaign plan",
        search: "Search campaign name, type, or description",
        empty: "No campaign plan yet. Create a campaign planning exercise first.",
      },
      columns: {
        campaign: "Campaign",
        budget: "Budget",
        period: "Period",
      },
      options: {
        draft: "Draft",
        active: "Active",
        paused: "Paused",
        ended: "Ended",
        archived: "Archived",
      },
      dialogs: {
        create: "Create campaign",
        edit: "Edit campaign",
      },
      notify: {
        created: "Campaign created",
        updated: "Campaign updated",
        archived: "Campaign archived",
        draftFilled: "Campaign draft filled into the form. Review before saving.",
      },
    },
    workspaces: {
      metrics: {
        workspaces: "Training tasks",
        workspacesTail: "Student submissions and revisions",
        tasks: "Task items",
        runs: "Submission records",
        runsTail: "Stores goals, requirements, and reports",
        artifacts: "Review materials",
        artifactsTail: "{{count}} accepted",
        executions: "Improvement suggestions",
      },
      panels: {
        master: "Training task list",
        masterDesc:
          "Each training task can bind products and campaigns, then store submission records, review materials, and improvement suggestions.",
        createWorkspace: "Create training task",
        search: "Search training task title or type",
        empty: "No training task yet. Generate or create one from the AI training assistant.",
        detail: "Training task detail",
        detailDesc:
          "Select a training task to maintain task items, submission records, and review materials.",
        noWorkspace: "No training task",
        selectedDesc:
          "This area stores task items, submission records, review materials, and improvement suggestions teachers can inspect.",
        archiveWorkspace: "Archive training task",
        tasksTab: "Task items",
        runsTab: "Submission records",
        artifactsTab: "Review materials",
        executionsTab: "Improvement suggestions",
        createTask: "Add task item",
        createRun: "Add submission record",
        runSnapshot: "Submission task snapshot",
        runSnapshotDesc: "Stores each task item's completion state at submission time.",
        createSnapshot: "Create snapshot",
        createExecution: "Add improvement suggestion",
        artifactFormat: "Format",
        artifactStorage: "Storage",
        artifactFile: "File",
        artifactRun: "Run",
        artifactExport: "Export format",
        artifactExportSize: "Export chars",
        skillRuns: "AI analysis records",
        skillRunsDesc:
          "Review which analyses AI performed in this training task for student review.",
        qualityLoop: "Quality and learning loop",
        qualityLoopDesc:
          "Review answer quality, learning records, and review notes to improve the next submission.",
      },
      actions: {
        copyExport: "Copy export",
        downloadExport: "Download export",
      },
      columns: {
        workspace: "Training task",
        task: "Task item",
        priority: "Priority",
        runGoal: "Submission goal",
        snapshot: "Snapshot",
        acceptance: "Acceptance",
        artifact: "Review material",
        validation: "Validation",
        suggestion: "Improvement suggestion",
        skill: "Capability",
        executionMode: "Execution mode",
        summary: "Summary",
      },
      empty: {
        tasks: "No task items yet",
        runs: "No submission records yet",
        runTasks: "No submission task snapshot yet",
        artifacts: "No review materials yet",
        artifactDetail: "Select a review material to view detail",
        artifactContent: "No content or preview saved yet",
        executions: "No improvement suggestions yet",
        skillRuns: "No AI analysis records for this submission yet",
        qualityChecks: "No quality checks for the current run",
        qualityIssues: "No quality issues recorded",
        learningRecords: "No learning records for this training task",
        trustScores: "No trust scores recorded yet",
      },
      quality: {
        latestCheck: "Latest quality check",
        trustScore: "Agent trust score",
        learningRecords: "Learning records",
        learningRecordsDesc:
          "From quality checks, skill output, and user feedback",
        issues: "Quality issues",
        learningSamples: "Learning samples",
        score: "{{score}}",
        statusAttention: "Attention",
        statusEmpty: "Empty",
        statusPassed: "Passed",
        statusSaved: "Saved",
      },
      dialogs: {
        workspace: "Create workspace",
        task: "Create task",
        run: "Create run record",
        runTask: "Create run task snapshot",
        execution: "Create execution suggestion",
      },
      notify: {
        workspaceCreated: "Workspace created",
        workspaceArchived: "Workspace archived",
        taskCreated: "Task created",
        taskUpdated: "Task status updated",
        taskArchived: "Task archived",
        runCreated: "Run record created",
        runUpdated: "Run status updated",
        runTaskCreated: "Run task snapshot created",
        runTaskUpdated: "Run task status updated",
        executionCreated: "Execution suggestion created",
        executionApproved: "Execution suggestion approved",
        executionRejected: "Execution suggestion rejected",
      },
    },
    platform: {
      metrics: {
        platforms: "Platform connections",
        channels: "Channels",
        metrics: "Metric records",
        metricsTail: "Metrics written locally or after external sync",
        gmv: "30d GMV",
        gmvTail: "From local metric summary",
      },
      panels: {
        supportedPlatforms: "Supported platforms",
        supportedPlatformsDesc:
          "Configurable platform list returned by backend",
        supportedChannels: "Supported channels",
        supportedChannelsDesc:
          "Used for notifications, bots, and message delivery",
        platformConnections: "Platform connections",
        platformConnectionsDesc:
          "This phase stores connection records and does not validate real platform credentials",
        savePlatform: "Save platform",
        channelConnections: "Channels",
        channelConnectionsDesc:
          "Stores channel connection data such as WeCom and Feishu",
        saveChannel: "Save channel",
        localMetrics: "Local metrics",
        localMetricsDesc:
          "Metrics power readonly platform summaries and business dashboards",
        saveMetric: "Write metric",
        summary: "Platform summary",
        summaryDesc: "Readonly summary from store_metrics",
        sync: "Sync status",
        syncDesc: "Connection state and latest metric date",
        executionAdapters: "Platform execution capabilities",
        executionAdaptersDesc:
          "Readonly backend contract for high-risk platform write adapters and approval readiness",
        externalSearch: "External search status",
        externalSearchDesc:
          "Reads the backend external-search adapter status. If it is not configured, the frontend shows it as unavailable instead of fabricating sources.",
      },
      columns: {
        metric: "Metric",
        gmv30: "GMV30",
        orders30: "Orders30",
        sync: "Sync",
        latestMetric: "Latest metric",
        connection: "Connection",
        adapter: "Adapter",
        readiness: "Readiness",
      },
      labels: {
        connected: "Connected, approval still required",
        notExecutable: "Not executable now",
        requiredParams: "Required params",
        idempotencyFields: "Idempotency fields",
        canExecuteNow: "Backend says executable now",
        dispatchPending:
          "Approval required; no fake success when real adapter is missing",
      },
      externalSearch: {
        status: "Status",
        provider: "Provider",
        enabled: "Enabled",
        configured: "Configured",
      },
      empty: {
        supportedPlatforms: "No supported platform config",
        supportedChannels: "No supported channels",
        executionAdapters: "No platform execution capabilities",
        externalSearch: "No external search status loaded",
        platformConnections: "No platform connections",
        channelConnections: "No channel connections",
        metrics: "No local metrics",
        summary: "No summary data",
        sync: "No sync status",
      },
      dialogs: {
        platform: "Save platform connection",
        channel: "Save channel",
        metric: "Write local metric",
      },
      notify: {
        platformSaved: "Platform connection saved",
        channelSaved: "Channel saved",
        metricSaved: "Metric saved",
        platformDisabled: "Platform connection disabled",
        channelDisabled: "Channel disabled",
      },
    },
    approvals: {
      metrics: {
        requests: "Approval requests",
        approved: "Approved",
        approvedTail: "Only records approval status; does not replay tools",
        rejected: "Rejected",
        rejectedTail: "Keeps rejection reason and audit record",
        audit: "Audit logs",
        auditTail: "Business traceability",
      },
      panels: {
        requests: "Tool approval requests",
        requestsDesc:
          "Stores approval status and manual decisions without Agent tool replay",
        createRequest: "Create approval",
        emptyRequests: "No approval requests",
        audit: "Audit logs",
        auditDesc:
          "Records approvals, execution suggestions, and manual review actions",
        createAudit: "Write audit",
        emptyAudit: "No audit logs",
      },
      columns: {
        skill: "Capability/tool",
        risk: "Risk",
        execution: "Backend execution result",
      },
      execution: {
        retryable: "Retryable",
        providerResult: "Provider result",
        yes: "Yes",
        no: "No",
        nullValue: "Null result",
        objectValue: "Structured result",
      },
      dialogs: {
        approval: "Create tool approval",
        audit: "Write audit log",
      },
      notify: {
        created: "Approval request created",
        approved: "Approval approved",
        rejected: "Approval rejected",
        auditCreated: "Audit log created",
      },
    },
    operations: {
      metrics: {
        longTasks: "Long tasks",
        briefings: "Briefings",
        briefingsTail: "Manual or derived briefing records",
        notifications: "Notifications",
        alerts: "Alerts",
      },
      panels: {
        longTasks: "Long tasks",
        longTasksDesc:
          "Stores long-task progress records without starting a real async executor",
        createLongTask: "Create long task",
        emptyLongTasks: "No long-task records",
        briefing: "Daily briefing",
        briefingDesc:
          "Briefing summary can be derived from alerts and long-task state",
        saveBriefing: "Save briefing",
        emptyBriefing: "No briefing records",
        notifications: "Background notifications",
        notificationsDesc:
          "Notifications can be marked read individually or in bulk",
        markAllRead: "Mark all read",
        createNotification: "Create notification",
        emptyNotifications: "No background notifications",
        alerts: "Alert center",
        activeAlerts: "Current active: {{count}}",
        dismissAll: "Dismiss all",
        createAlert: "Create alert",
        emptyAlerts: "No alerts",
      },
      columns: {
        task: "Task",
        progress: "Progress",
        briefing: "Briefing",
        notification: "Notification",
        alert: "Alert",
        severity: "Severity",
      },
      states: {
        read: "Read",
        unread: "Unread",
      },
      dialogs: {
        longTask: "Create long-task record",
        briefing: "Save daily briefing",
        notification: "Create background notification",
        alert: "Create alert",
      },
      notify: {
        longTaskCreated: "Long-task record created",
        longTaskUpdated: "Long-task status updated",
        longTaskCancelled: "Long task cancelled",
        briefingSaved: "Briefing saved",
        briefingArchived: "Briefing archived",
        notificationCreated: "Notification created",
        notificationRead: "Notification marked read",
        allRead: "All notifications marked read",
        alertCreated: "Alert created",
        alertClosed: "Alert closed",
        alertActed: "Alert processed",
        alertSnoozed: "Alert snoozed for 24 hours",
        alertsClosed: "Alerts dismissed in bulk",
      },
    },
    files: {
      errorTitle: "Failed to load upload file library",
      panelTitle: "Upload file library",
      panelDescription:
        "Check upload read status and usage state, then choose which files should support this AI turn.",
      searchPlaceholder: "Search file name, type, object key, or status",
      deleteSelected: "Delete {{count}} file(s)",
      clearSelection: "Clear selection",
      clearFilters: "Clear filters",
      uploadingAction: "Uploading",
      uploadAction: "Upload files",
      emptyTitle: "No material files yet",
      empty:
        "Upload files from chat attachments or the upload file library, then review read status and choose what the AI should use this turn.",
      emptyAction: "Upload files",
      noFilterResultTitle: "No materials match these filters",
      noFilterResult:
        "Adjust the keyword, file type, read status, or usage state and try again.",
      summary: {
        total: {
          label: "Total materials",
          value: "{{count}} file(s)",
        },
        readable: {
          label: "AI readable",
          value: "{{count}} file(s)",
        },
        attached: {
          label: "Attached to chat",
          value: "{{count}} file(s)",
        },
        selectedForTurn: {
          label: "Selected this turn",
          value: "{{count}} file(s)",
        },
      },
      turnSelection: {
        title: "AI will read {{count}} material file(s) this turn",
        files: "{{names}}{{hiddenText}}",
        hiddenSuffix: " and {{count}} more",
        empty: "No material files are selected for this turn.",
        action: "Show turn materials",
        clearAction: "Clear turn selection",
      },
      filters: {
        kind: {
          all: "All types",
          image: "Images",
          pdf: "PDF",
          word: "Word documents",
          sheet: "Spreadsheets",
          text: "Text",
          other: "Other",
        },
        readStatus: {
          all: "All read statuses",
          readable: "AI readable",
          pending: "Pending parse",
          failed: "Parse failed",
          unsupported: "Unsupported",
        },
        usage: {
          all: "All usage states",
          available: "Unused",
          attached: "Attached to chat",
          selectedForTurn: "Selected this turn",
        },
      },
      filterSummary: {
        all: "{{total}} material file(s)",
        filtered: "{{filtered}} / {{total}} material file(s) shown",
      },
      table: {
        name: "File",
        status: "Read status",
        useStateHeader: "Usage state",
        size: "Size",
        createdAt: "Uploaded at",
        actions: "Actions",
        detail: "Details",
        using: "Processing",
        useState: {
          available: "Add to turn",
          attached: "Add to turn",
          selectedForTurn: "Remove from turn",
        },
      },
      status: {
        readable: "AI readable",
        pending: "Pending parse",
        failed: "Parse failed",
        unsupported: "Unsupported",
      },
      deleteDialog: {
        title: "Delete material files",
        description: "Delete the selected {{count}} material file(s)?",
        impactTitle: "Deletion impact",
        impactSummary:
          "This will delete {{total}} file(s), including {{readable}} AI-readable file(s), {{attached}} chat-attached file(s), and {{selectedForTurn}} file(s) selected for this turn.",
        targetNames: "Target files: {{names}}{{hiddenText}}",
        hiddenSuffix: " and {{count}} more",
        cancel: "Cancel",
        confirm: "Delete",
      },
      detailDialog: {
        title: "Material details",
        description:
          "Review file source, storage information, extraction summary, and current-turn usage state.",
        sections: {
          file: "File information",
          storage: "Storage information",
          reading: "Extraction result",
          usage: "Usage state",
          metadata: "Metadata",
        },
        fields: {
          displayName: "Display name",
          type: "File type",
          mimeType: "MIME type",
          size: "File size",
          recordStatus: "Record status",
          createdAt: "Uploaded at",
          fileId: "File ID",
          conversationId: "Conversation ID",
          messageId: "Message ID",
          bucket: "Bucket",
          objectKey: "Object key",
          objectRef: "Object reference",
          sha256: "SHA-256",
          readStatus: "Read status",
          summary: "Extraction summary",
          failureReason: "Failure reason",
          textChars: "Text characters",
          parser: "Parser",
          contentKind: "Content type",
          truncated: "Truncated",
          usageState: "Usage state",
          aiReadableThisTurn: "AI reads this turn",
        },
        yes: "Yes",
        no: "No",
        close: "Close",
        usageState: {
          available: "Unused",
          attached: "Attached to chat",
          selectedForTurn: "Selected this turn",
        },
        usageHint: {
          available:
            "This file is not included in the current turn. Select it before sending if the AI should use it.",
          attached:
            "This file is already available in chat materials and can be selected for this turn.",
          selectedForTurn:
            "This file will support the next AI answer and can be removed before sending.",
        },
        metadata: {
          description:
            "Raw metadata helps inspect extraction, source, and storage state.",
          copy: "Copy metadata",
        },
      },
      readSummary: {
        textChars: "{{count}} chars",
        parser: "Parser: {{parser}}",
        contentKind: "Content: {{kind}}",
        truncated: "Truncated",
        summary: "Summary: {{summary}}",
        failure: "Failed: {{reason}}",
      },
    },
  },
  home: {
    welcome: "Welcome back",
    publicWelcome: "Welcome",
    publicTitle: "Ecommerce AI training workspace",
    publicSubtitle:
      "Complete one training task through product selection, campaign planning, data diagnosis, submission, and review report.",
    fallbackName: "Member",
    actions: {
      assistant: "Ask AI assistant",
      profile: "Edit profile",
      login: "Sign in",
    },
    studentPath: {
      eyebrow: "Student training path",
      title: "Start here if you are not sure what to do",
      description:
        "Entries follow the way a student completes one ecommerce training task: ask the multi-agent assistant, then use selection analysis, planning, submission, and review.",
      steps: {
        assistant: {
          title: "Ask multi-agent AI",
          description:
            "Enter a product, campaign, or data question. The system detects the task and shows the collaboration process.",
          action: "Start asking",
        },
        products: {
          title: "Add product context",
          description:
            "Review products, materials, competitors, and knowledge so later analysis has real context.",
          action: "Open products",
        },
        campaigns: {
          title: "Plan campaign",
          description:
            "Turn budget, rhythm, scripts, and goals into an executable campaign plan.",
          action: "Plan campaign",
        },
        tasks: {
          title: "Submit task",
          description:
            "Save AI output as a task, run record, and checkable training deliverable.",
          action: "Open tasks",
        },
        reports: {
          title: "Build review report",
          description:
            "Organize conclusions, process, checklists, and improvements into final review materials.",
          action: "Open report",
        },
      },
    },
    account: {
      title: "Account ready",
      description:
        "Your training account is active. Conversations, task deliverables, and review materials stay with this account.",
    },
    publicAccount: {
      title: "Before training",
      description:
        "Sign in to continue training conversations, review product materials, submit tasks, and generate review reports.",
    },
    profile: {
      title: "Profile settings",
      description:
        "Update your display name, avatar, email, and password from your profile page.",
    },
    publicProfile: {
      title: "Personal training records",
      description:
        "Sign in to manage account details and keep your training process and deliverable records.",
    },
  },
  agentChat: {
    welcomeMessage:
      "Send the product, title, customer issue, or assignment you are stuck on. It does not need to be complete; I will first help you judge the direction, then ask for the one or two details that matter most.",
    answer: {
      sectionLabels: {
        direct: "Usable",
        final: "Final",
        reason: "Reason",
        team: "Team",
        action: "Action",
      },
    },
    streaming: "Thinking...",
    error: "The AI partner could not finish the reply. Please try again later.",
    errorBadge: "Reply failed",
    boundaryNotice:
      "Note: this turn is missing a selected product or live metrics in a few places. I still gave the useful judgment first and marked what can be added later.",
    missingStructuredOutput:
      "The previous turn did not produce a complete result. Send it again and I will rebuild the reasoning notes and sync this turn's thread to the side panel.",
    streamTimedOut:
      "This reply took longer than expected, so I collapsed the reasoning notes for now. Review the partial result or send it again to continue.",
    loginRequired: {
      title: "Sign in to keep using your AI commerce partner",
      description:
        "You can draft the ecommerce question and choose a strategy first. Sign in when you are ready to call the cloud model and save the conversation.",
      message:
        "Sign in to continue the analysis and save the conversation. You can still draft the question and choose a style here.",
      inline: "Sign in to continue and save history",
      action: "Sign in",
    },
    quota: {
      loading: "Loading account question quota...",
      availableBadge: "Quota available",
      exhaustedBadge: "No questions left",
      disabledBadge: "Paused",
      usage: "{{used}} / {{limit}} used, {{remaining}} remaining",
      account: "Account: {{account}}",
      profile: "Stats profile: {{school}} / {{team}}",
      exhaustedHint:
        "This account's AI question quota has been used up. Contact the event or platform administrator to add quota.",
      disabledHint:
        "This account's AI questions are paused. Contact an administrator.",
    },
    sidebar: {
      newChat: "New chat",
      search: "Search history",
      recents: "Recents",
      chat: "AI ecommerce partner",
      more: "More",
      less: "Collapse",
      moreHistory: "Show {{count}} more",
      lessHistory: "Collapse to latest 5",
      expandHistory: "Expand history",
      collapseHistory: "Collapse history",
      expandFunctionNav: "Expand training navigation",
      collapseFunctionNav: "Collapse training navigation",
      empty: "No conversations yet",
    },
    shell: {
      brandEyebrow: "AI ecommerce partner",
      brandTitle: "AI ecommerce partner",
      brandDescription: "Say the question directly and we will work through it",
      topbarTitle: "Say your ecommerce question",
      topbarMeta: "AI ecommerce partner · natural language entry",
      sections: {
        create: "",
        workspace: "Training navigation",
        governance: "Manage and review",
        recents: "History",
      },
      navigation: {
        newChat: "New chat",
        capabilities: "Capability and usage guide",
        smartOps: "AI ecommerce partner",
        productBoard: "Product selection analysis",
        materials: "Upload file library",
        campaigns: "Campaign planning practice",
        platform: "Platform data",
        marketResearch: "Data diagnosis guidance",
        operations: "Business result tracking",
        briefing: "Daily briefing",
        alerts: "Alerts",
        deliverables: "Review report",
        approvals: "Teacher review records",
        taskBoard: "Training tasks and deliverables",
        systemDashboard: "System dashboard",
        team: "AI team",
        history: "History",
      },
    },
    strategy: {
      current: "How to talk",
      options: {
        learning: {
          label: "Think together",
          hint: "Work through the issue like a partner, explain the useful judgment, then give the next move",
          shortHint: "Together",
        },
        quick: {
          label: "Be direct",
          hint: "Best for quick direction: judgment, key reason, and next step",
          shortHint: "Direct",
        },
        execution: {
          label: "Go deeper",
          hint: "Best for plans or assignments: path, risks, checklist, and review checks",
          shortHint: "Detailed",
        },
      },
    },
    composer: {
      placeholder:
        "Say what you want to handle, for example: can this product sell, improve this title, or how should I reply when a customer says it is expensive...",
      loginPlaceholder:
        "Draft the question here; after sign-in, I can keep helping and save the conversation.",
      hint: "Business context connected",
      send: "Send",
      stop: "Stop",
      status: {
        sending: "Working on this task. You can stop it anytime.",
        retry: "Retry",
      },
      attachTitle: "Upload attachment",
      attachShort: "Upload",
      attachPrompt:
        "Add the file or data constraints here, for example: product price, budget, inventory, target audience, current conversion rate, and platform account status.",
      agentTitle: "Reference angles for this turn",
      agentName: "AI ecommerce partner",
      contextPreview: {
        compactLabel: "Check first",
        compactTitle: "Check what this turn will reference",
        panelTitle: "Check what I will reference before sending",
        panelHint:
          "Reviews materials and memory summaries without generating an answer.",
        loading: "Preparing this turn's references",
        error: "Reference preview failed",
        empty:
          "Enter a question to see which materials and memories this turn will reference.",
        run: "Check references",
        refresh: "Check again",
        counts: {
          included: "Full text",
          metadataOnly: "Metadata",
          omitted: "Omitted",
          memory: "Memory",
        },
        refsTitle: "References for this turn",
        refsEmpty: "No extra materials for this turn.",
        moreRefs: "{{count}} more reference(s) hidden",
        refStatus: {
          included: "Full text",
          metadataOnly: "Metadata",
          omitted: "Omitted",
        },
        memoryTitle: "History memory",
        memoryEmpty: "No useful history summary found yet.",
        charCount: "{{count}} chars",
        truncated: " / truncated",
        warningTitle: "Warnings",
        audit: {
          title: "Context audit",
          history: {
            label: "Conversation history",
            value: "{{count}} / {{total}} messages",
            detail:
              "History {{chars}} chars, memory {{memory}} chars, older relevant {{older}}, {{truncated}}",
            truncated: "truncated",
            complete: "not truncated",
          },
          memory: {
            label: "Structured memory",
            value:
              "Facts {{facts}}, decisions {{decisions}}, constraints {{constraints}}, questions {{questions}}",
            detail: "{{summary}}, about {{chars}} chars",
            summaryReady: "summary available",
            summaryEmpty: "no summary",
          },
          workspace: {
            label: "Workspace materials",
            value: "{{count}} source(s), {{readiness}}",
            detail: "{{missing}} missing, {{warnings}} warning(s)",
          },
          pack: {
            label: "Readable pack",
            value: "{{count}} / {{total}} readable",
            detail:
              "{{readiness}}, about {{chars}} chars, {{omitted}} omitted, {{warnings}} warning(s)",
          },
          budget: {
            label: "Character budget",
            value: "About {{count}} context chars",
            detail: "{{input}} input chars",
          },
          readiness: {
            ready: "ready",
            partial: "partially ready",
            empty: "no usable material",
            unknown: "unknown status",
          },
        },
      },
      productContext: {
        label: "Product references",
        compactLabel: "Product",
        noneCompactLabel: "No product",
        compactTitle: "Product references for this turn",
        loading: "Loading products",
        empty: "No active products",
        noneSelected: "No product reference",
        choose: "Add product reference",
        collapse: "Collapse products",
        clear: "Remove product references",
        noneHint:
          "This turn will not read saved product data and will answer as a general ecommerce question.",
        manualHint:
          "I reference product materials only after you add a saved product.",
        searchPlaceholder:
          "Search product name, brand, category, platform, or status",
        selectedHint:
          "This turn will reference the added product materials.",
        selectedCount: "{{count}} product reference(s)",
        moreCount: "{{count}} more",
        showMore: "Show {{count}} more product(s)",
        noSearchResult: "No matching product",
        addProduct: "Add product",
        status: {
          active: "Active",
          draft: "Draft",
          listing: "Presale/listing",
          archived: "Archived",
          unknown: "Unknown",
        },
      },
      campaignContext: {
        label: "Campaign references",
        compactLabel: "Campaign",
        noneCompactLabel: "No campaign",
        compactTitle: "Campaign references for this turn",
        loading: "Loading campaigns",
        empty: "No available campaigns",
        noneSelected: "No campaign reference",
        choose: "Add campaign reference",
        collapse: "Collapse campaigns",
        clear: "Remove campaign references",
        noneHint:
          "This turn will not read saved campaign plans, budget, or execution data.",
        manualHint:
          "I reference campaign materials only after you add a campaign.",
        searchPlaceholder:
          "Search campaign name, type, status, budget, or date",
        selectedHint:
          "This turn will reference the added campaigns' goals, budget, dates, status, and saved outputs.",
        selectedCount: "{{count}} campaign reference(s)",
        budget: "Budget {{value}}",
        showMore: "Show {{count}} more campaign(s)",
        noSearchResult: "No matching campaign",
        status: {
          active: "Active",
          draft: "Draft",
          paused: "Paused",
          ended: "Ended",
          archived: "Archived",
          unknown: "Unknown",
        },
      },
      sourceContext: {
        compactLabel: "Files",
        noneCompactLabel: "No files",
        compactTitle: "File references for this turn",
        compactReadinessTitle: "{{status}}",
        noneSelected: "No file reference",
        selectedCount: "{{count}} file reference(s)",
        availableCount: "{{count}} available",
        readableCount: "{{count}} readable",
        clear: "Remove file references",
        deleteSelected: "Delete selected",
        deleteConfirm:
          "Delete {{count}} selected source(s): {{names}}{{hiddenText}}\n\nUse Remove file references if you only want to remove them from this turn.",
        deleteConfirmHidden: " and {{count}} more",
        empty:
          "No uploaded files are available in this conversation. Upload a file first, then add it as a reference.",
        manualHint:
          "I reference these files only after you add them to this turn.",
        selectedHint:
          "This turn will read only the added current-conversation files.",
        selectedPreview: "Will reference: {{names}}{{hiddenText}}",
        selectedPreviewHidden: " and {{count}} more",
        selectedReadinessLabel: "Read status",
        selectedReadiness: {
          readable: "{{count}} readable",
          pending: "{{count}} pending",
          failed: "{{count}} failed",
          unsupported: "{{count}} unsupported",
          metadataOnly: "{{count}} metadata only",
        },
        panelTitle: "File references for this turn",
        panelHint:
          "Add current files or library files for this answer to reference.",
        upload: "Upload",
        manageLibrary: "Browse file library",
        currentSection: "Current conversation files",
        currentSectionHint: "Only added files are read",
        currentSearch: "Search current files",
        currentNoSearchResult: "No current file matches the search.",
        showMoreCurrent: "Show {{count}} more current file(s)",
        librarySection: "File library",
        libraryHint: "Add existing files before using them as references.",
        librarySearch: "Search file library",
        libraryLoading: "Loading file library",
        libraryNoSearchResult: "No file library item matches the search.",
        showMoreLibrary: "Show {{count}} more library file(s)",
        libraryEmpty: "No saved files in the library yet.",
        libraryEmptyAction: "Open file library",
        libraryAttach: "Add",
        libraryAttached: "Added",
        libraryStatus: {
          failed: "Failed",
          pending: "Pending",
          readable: "Readable",
          unsupported: "Unsupported",
          uploaded: "Uploaded",
          visionReady: "Vision ready",
        },
        sourceStatus: {
          failed: "Failed",
          metadataOnly: "Metadata only",
          pending: "Pending",
          readable: "Readable",
          unsupported: "Unsupported",
          visionReady: "Vision ready",
        },
        readable: "Readable text",
        vision: "Vision",
        visionReady: "Vision ready",
        metadataOnly: "Metadata only",
        currentConversation: "Current conversation",
      },
      attachments: {
        uploading: "Uploading",
        uploaded: "Attached",
        pending: "Pending",
        failed: "Failed",
        unsupported: "Unsupported",
      },
      playbooks: {
        loading: "Loading scenarios",
        empty: "No scenarios",
      },
      settings: {
        compactLabel: "Answer style",
        compactTitle: "Adjust answer preferences",
        toggle: "Answer preferences",
        panelTitle: "How should we handle this turn",
        panelHint: "Current preference: {{mode}}; {{summary}}",
        rolesCount: "{{count}} angles",
        depthSummary: "Detail: {{depth}}",
        selectedPrefix: "Current",
        hideSelection: "Hide",
        showSelection: "Show current preferences",
        autoRolesSummary: "I will choose",
        multiAgentSummary: "{{count}} angles",
      },
      runtime: {
        label: "Reference angles",
        loading: "Loading answer preferences",
        model: "Model",
        modelSettings: "Model preference",
        modelHint:
          "The currently available Doubao model is preferred by default. If it is not available, only usable models are shown.",
        learning: "Detail level",
        collaboration: "Reference style",
        routingMode: "Handling style",
        roleMode: "Reference style",
        roles: "Reference angles",
        singleRole: "Selected angle",
        primary: "Main angle",
        leadRole: "Main angle",
        supportRoles: "Supporting angles",
        selectedRoles: "Reference angles for this turn",
        roleColumn: "Angle",
        primaryShort: "Main angle",
        supportShort: "Support",
        singleRoleHint:
          "Click one angle to focus this answer from that direction.",
        leadRoleHint:
          "Click an angle card to switch the main focus. The visible answer still reads like one natural recommendation.",
        supportRolesHint:
          "Select supporting angles to add useful evidence, risks, materials, and action suggestions.",
        answerRoleBadge: "Current angle",
        primaryBadge: "Main angle",
        supportBadge: "Support",
        currentLead: "Current main angle",
        currentSupport: "Supporting angles: {{roles}}",
        noSupportRoles: "No supporting angle selected",
        advanced: "More preferences",
        explanationDepth: "Explanation depth",
        autoRolesHint:
          "I will decide which angles are useful for this turn.",
        manualRolesHint:
          "Pick one main angle, and optionally add supporting angles. Up to {{max}}; more angles make the response slower.",
        roleModeHint:
          "For this turn, I can decide, focus on one angle, or add a few supporting angles.",
        learningLevels: {
          beginner: {
            label: "Basic",
            description: "Use fewer terms and clear steps",
          },
          higher_vocational: {
            label: "Applied",
            description: "Explain in ecommerce training terms",
          },
          professional: {
            label: "Professional",
            description: "Keep metrics, strategy, and risk reasoning",
          },
        },
        roleLabels: {
          chat: {
            name: "General angle",
          },
          ops: {
            name: "Operations angle",
          },
          data: {
            name: "Data angle",
          },
          creative: {
            name: "Creative angle",
          },
          service: {
            name: "Service angle",
          },
          douyin_ads: {
            name: "Ads angle",
          },
          accounting: {
            name: "Finance angle",
          },
          search: {
            name: "Source-check angle",
          },
          engineering: {
            name: "Technical angle",
          },
        },
        capabilityLabels: {
          "ops.strategy": "Operations strategy",
          "campaign.orchestration": "Campaign orchestration",
          "data.analysis": "Data analysis",
          "metrics.diagnosis": "Metric diagnosis",
          "creative.content": "Creative content",
          "creative.copywriting": "Copywriting",
          "service.workflow": "Service workflow",
          "service.sentiment": "Customer sentiment",
          "ads.optimization": "Ads optimization",
          "roi.control": "ROI control",
          "finance.profitability": "Profitability",
          "budget.planning": "Budget planning",
        },
        roleModes: {
          auto: {
            label: "I will choose",
            description: "Auto angles",
          },
          single: {
            label: "One focus",
            description: "One direction",
          },
          manual: {
            label: "Multiple angles",
            description: "Combined view",
          },
        },
        routingModes: {
          commander: {
            label: "Full planning",
            description: "Analyze the task before arranging steps",
          },
          fast: {
            label: "Fast handling",
            description: "Prefer an existing processing path",
          },
        },
        strategyNotes: {
          learning:
            "Think together: keep the substance while explaining useful judgment and next steps.",
          quick:
            "Best for fast judgment: conclusion, key reason, and only a few next actions.",
          execution:
            "Best for reports, reviews, and plans: use tables and checklists for steps, risks, and checks.",
        },
      },
      samples: {
        coffee: {
          label: "May Day coffee conversion",
          prompt:
            "I run coffee operations and mainly market on Douyin. May Day is coming soon. Please write an operations plan to improve conversion rate, including Qianchuan ads, budget, creative, and action steps.",
        },
        qianchuan: {
          label: "Qianchuan ads review",
          prompt:
            "Help me review a Douyin live room Qianchuan ads issue: click-through rate is fine but order conversion is low. Split it into data diagnosis, budget, creative, ads, and operations tasks.",
        },
      },
      chips: {
        products: {
          label: "Coffee products",
          title: "Add product context to the question.",
          prompt:
            "Product context: coffee SKU group, price band, selling points, inventory, gross margin, product knowledge, and competitor clues should be considered.",
        },
        campaign: {
          label: "May Day campaign",
          title: "Add campaign context to the question.",
          prompt:
            "Campaign context: May Day campaign goal, budget, schedule, creative direction, live room execution, and review metrics should be considered.",
        },
        platform: {
          label: "Douyin/Qianchuan data",
          title: "Add platform data context to the question.",
          prompt:
            "Platform data context: Douyin live room funnel, Qianchuan click-through rate, conversion rate, ROI, audience package, budget pace, and anomaly signals should be considered.",
        },
      },
    },
    workbench: {
      title: "Judgment notes",
      expand: "Expand judgment notes",
      collapse: "Collapse judgment notes",
      close: "Close judgment notes",
      closePanel: "Collapse notes panel",
      progress: "Progress",
      activity: "Run activity",
      adaptiveDispatch: "Adaptive dispatch",
      record: "Run records",
      recordCountSuffix: "items",
      businessInput: "Business input",
      preview: "Work preview",
      turns: "Attempts",
      handoff: "Business handoff",
      futureModules: "Connects to real modules later",
      idle: {
        planningTitle: "Organizing the reasoning path",
        historyTitle: "Historical chat, waiting for a new task",
        waitingTitle: "Waiting for business goal",
        planningDescription:
          "Input received. Matching products, campaigns, data, and delivery steps.",
        historyDescription:
          "Judgment notes only show this turn's reasoning path; historical messages are not counted in progress.",
        waitingDescription:
          "After you enter an ecommerce question, it will organize products, campaigns, platform data, alerts, and deliverables.",
        loopTitle: "01 | Operating loop",
        previewTitle: "Run preview",
        previewFile: "Operating loop preview",
        pending: "Pending",
        lines: [
          "Data diagnosis identifies metrics, attribution, and operating constraints",
          "Product selection analysis provides product knowledge, selling points, and competitor clues",
          "Campaigns carry planning, execution cadence, and review results",
          "Platform connections sync Douyin and Qianchuan status",
          "Budget, ads optimization, and creative work run in parallel",
          "Operations plan summarizes budget, creative, and launch cadence",
          "Deliverables store plans, checklists, and review language",
          "Alerts store budget, delivery, and conversion anomaly rules",
          "Daily briefing summarizes key progress and todos",
        ],
      },
    },
    feedback: {
      title: "Run feedback",
      description:
        "The rating is saved as a backend learning record for quality, trust, and review loops.",
      pending: "Pending",
      submitted: "Submitted",
      noTarget: "Waiting for run",
      target: "Target: {{target}}",
      ratingLabel: "Run answer rating",
      ratingTitle: "{{rating}} stars",
      placeholder: "Optional: what worked, what should improve",
      submit: "Submit feedback",
      disabled: {
        login: "Log in to submit feedback for this run.",
        noRun: "The backend has not returned a workspace run yet.",
        streaming: "Submit feedback after the answer finishes.",
        submitted: "Feedback for this run has been submitted.",
      },
      notify: {
        success: "Feedback saved to learning records",
        failed: "Failed to submit feedback",
      },
    },
    thread: {
      agentName: "AI ecommerce partner",
      welcomeCapabilities: ["Catch the issue", "Ask less", "Move forward"],
      startHere: "You can also ask like this",
      examplePrompts: [
        "Can this product sell? I only have a rough idea right now.",
        "Help make this title something customers are more likely to click.",
        "A customer says it is expensive. How can I reply naturally?",
      ],
      studentFlow: {
        title: "You can start naturally the first time",
        subtitle:
          "Say the issue in front of you, and I will help move it one step forward.",
        ask: {
          title: "1. Say the thing",
          detail:
            "Product, title, customer issue, or assignment requirements are all enough to start.",
        },
        watch: {
          title: "2. Get direction first",
          detail:
            "I will give a useful judgment first, then ask for the few details that matter.",
        },
        deliver: {
          title: "3. Keep moving",
          detail:
            "When needed, continue into a plan, reply, table, or review material.",
        },
      },
      processPromise:
        "If you want to see how the judgment was made, expand the process panel; by default, the answer stays first.",
      studentPromptRecipe: {
        title: "When unsure how to describe it, say it like this",
        subtitle:
          "You do not need to learn the menus or provide everything at once.",
        goal: {
          label: "Goal",
          value: "What to judge, create, improve, or review",
        },
        material: {
          label: "Materials",
          value: "Product, budget, platform, data, teacher requirements",
        },
        limit: {
          label: "Limits",
          value: "Time, budget, format, required rules",
        },
        output: {
          label: "Output",
          value: "Plan, checklist, table, report, or next step",
        },
      },
      studentTaskEntries: {
        title: "Start here",
        subtitle: "Choose one to put it into the input.",
        start: {
          title: "Check if this product can sell",
          detail: "Incomplete information is fine; start with margin, demand, and service risk.",
          prompt:
            "I have a product I want to sell, but the materials are incomplete. First help me judge whether it can work and what key detail is still missing.",
        },
        analyze: {
          title: "Improve title or selling points",
          detail: "Make the click reason and buying reason clearer.",
          prompt:
            "Help me see why this product title and selling points are not attractive enough, then rewrite one natural version.",
        },
        plan: {
          title: "Write a customer reply",
          detail: "Keep it natural without promising what the shop cannot prove.",
          prompt:
            "A customer says the price is expensive. Help me write a natural reply that does not overpromise.",
        },
        data: {
          title: "Review my data",
          detail: "You do not need to know the metrics first; I will help find what affects the result most.",
          prompt:
            "I have some browse, click, add-to-cart, purchase, or refund data. First help me judge where the issue may be stuck and what I should check next.",
        },
        review: {
          title: "Make the plan more convincing",
          detail: "Useful for assignments, presentations, or teacher feedback.",
          prompt:
            "My teacher said my plan is too empty. Help me make it more evidence-based and executable.",
        },
      },
      receivingOutput: "Preparing the usable answer",
      waitingEvents: "Organizing judgment notes",
      stopped: "This turn stopped. Send again or try a shorter question.",
      errorActions: {
        title: "Turn not completed",
        retry: "Refill",
        simplify: "Shorter ask",
        retryPrompt:
          "Please redo the previous task. Start with the conclusion, then give directly usable content.",
        shorterPrompt:
          "Please answer with a shorter task: conclusion, directly usable content, and next step.",
      },
      completedTasks: "{{completed}}/{{total}} steps completed",
      planning: "Organizing the reasoning path",
      historyNotExecuted: "Historical message has no reasoning thread",
      executionTitle: "Judgment notes for this turn",
      processKinds: {
        waiting: {
          title: "Preparing task",
          badge: "Preparing",
          progress: "Building steps",
          summary:
            "Checking the question, materials, and answer route.",
        },
        quick_dispatch: {
          title: "Quick judgment",
          badge: "Light",
          progress: "{{completed}}/{{total}} steps",
          summary: "{{role}} is taking a direct look at: {{task}}",
          stageRunning: "Reading materials and preparing the answer",
          stageFinished: "Quick answer completed",
          stageSynced: "Quick answer completed",
        },
        single_agent: {
          title: "Main angle",
          badge: "Focus",
          progress: "{{completed}}/{{total}} steps",
          summary: "{{role}} is focusing on: {{task}}",
          stageRunning: "Organizing this turn's judgment",
          stageFinished: "Judgment completed",
          stageSynced: "Judgment completed",
        },
        multi_agent: {
          title: "Multiple reference angles",
          badge: "Combining",
          progress: "{{completed}}/{{total}} steps",
          summary: "Combining useful angles for: {{task}}",
          stageRunning: "Preparing a usable answer",
          stageFinished: "Judgment summarized",
          stageSynced: "Judgment summarized",
        },
      },
      statuses: {
        executing: "Task running",
        synced: "",
        planning: "Arranging steps",
        waiting: "Waiting for a new task",
      },
      artifactsTitle: "Stage files",
      artifacts: {
        updating: "Updates with handling",
        preparing: "Preparing",
        pending: "Pending",
        backendCreated: "System created {{type}}, status: {{status}}",
        empty:
          "No review materials yet. Demo files are not shown for this run.",
        searchEvidence: "System saved real external search evidence.",
        waitingBackend: "Waiting for review materials or search evidence.",
      },
      currentProgress: "Task progress {{completed}}/{{total}}",
      collapseProcess: "Collapse judgment notes",
      expandProcess: "Expand judgment notes",
      openWorkbench: "View judgment notes",
      openWorkbenchForTurn: "View judgment notes",
      closeWorkbench: "Hide judgment notes",
      currentAnswerProcess: "Current judgment notes",
      processLinkedToAnswer: "Linked to this answer",
      processPanelActive: "Right panel open",
      contextSummary: {
        label: "Training materials for this turn",
        productNone: "No product",
        productCount: "{{count}} products",
        campaignCount: "{{count}} campaigns",
        sourceNone: "No files",
        sourceCount: "{{count}} files",
        strategy: "Mode: {{strategy}}",
        autoRoles: "Angles: I will choose",
        roles: "Angles: focus {{role}} / {{count}} total",
        model: "Model: {{model}}",
        details: "Context details",
        productDetails: "Referenced products",
        campaignDetails: "Referenced campaigns",
        sourceDetails: "Referenced files",
      },
      actions: {
        copy: "Copy",
        useful: "Useful",
        useless: "Not useful",
      },
      agents: {
        standby: "Standby",
        data: {
          name: "Data diagnosis",
          summary:
            "Identify metrics, attribution, and operating constraints first",
        },
        accounting: {
          name: "Budget review",
          summary: "Check budget, SKU, and risk boundaries",
        },
        douyinAds: {
          name: "Qianchuan ads",
          summary: "Break down plan structure, audiences, and ROI stop-loss",
        },
        creative: {
          name: "Creative output",
          summary:
            "Generate short-video, live, and campaign content directions",
        },
        ops: {
          name: "Ops plan",
          summary: "Combine upstream findings into an execution cadence",
        },
        delivery: {
          name: "Deliverable doc",
          summary: "Store the plan, checklist, and follow-up artifacts",
        },
      },
      steps: {
        readTarget: "Identify training goal",
        planReady: "Reasoning path generated",
        readingRules: "Check question and constraints",
        targetRules: "Question and constraints",
        assignTask: "Arrange reasoning steps",
        dispatchJudge: "Judge task complexity",
        dispatchRoute: "{{mode}}, focus {{role}}",
        dispatchWaiting: "Waiting for handling style",
        runtimeAction: "Organize required evidence",
        answerReady: "Prepare conclusion",
        answerFinished: "Answer completed",
        answerStreaming: "Generating answer",
        answerSynced: "Answer generated",
        workspaceContext: "Read {{readable}}/{{total}} usable materials",
        contextLoaded: "{{products}} products, {{knowledge}} knowledge records",
        memoryLoaded: "Referenced {{count}} history records",
        skillRequested: "Checking {{skill}} as a reference",
        skillResult: "{{skill}} reference result: {{status}}",
        modelRunning: "Organizing output",
        noExtraSkill: "No extra tool needed for this turn",
        singleAgentRoute: "Focus on {{role}}",
        taskRunning: "Working",
        waitingSync: "Waiting for status sync",
        checkingConstraints: "Checking model, budget, and ads constraints",
        enrichMaterial: "Retrieve sources and evidence",
        enrichByTask: "Enrich platform materials by task",
        enrichFallback: "Enrich industry and platform context",
        directAnswer: "Simple questions get a direct answer; complex questions combine reference angles",
        callModel: "Organize answer",
        singleAgentAnswer: "Organize the answer around the main angle",
        savedToReport: "Save the answer as reviewable material",
        graphNodeReady: "Reference step prepared for this question",
      },
      roleFallbackTitles: {
        chat: "Direct answer",
        data: "Data diagnosis",
        creative: "Creative output",
        service: "Service plan",
        design: "Experience design",
        accounting: "Budget review",
        engineering: "Engineering judgment",
        web: "Web/SEO plan",
        douyin_ads: "Qianchuan ads optimization",
        ops: "Operations plan",
      },
      artifactSpecs: {
        dataStatus: "Data diagnosis, metric attribution, platform context",
        adsStatus:
          "Qianchuan plan structure, audience packages, budget cadence",
        deliveryFile: "operation.execution.plan.docx",
        deliveryStatus: "Final plan, checklist, and review language",
      },
      handoff: {
        context: {
          label: "Context",
          detail: "Product selection analysis, platform connections, diagnosis guidance",
        },
        execution: {
          label: "Execution",
          detail: "Campaigns, task board, training tasks",
        },
        delivery: {
          label: "Delivery",
          detail: "Deliverables, daily briefing, review history",
        },
        review: {
          label: "Review",
          detail: "Process checks, risk reminders, follow-up questions",
        },
      },
      processStage: {
        failed:
          "Handling is blocked. Expand judgment notes to inspect action items.",
        finished: "Judgment summarized. Review notes and business links.",
        saved: "Judgment summarized. Review notes and business links.",
        running: "Combining reference angles",
        planning: "Organizing the reasoning path",
        synced: "Expand to review this turn's judgment notes",
        history: "Historical chat. Expand to review this turn's judgment notes.",
        direct: "Handled as direct answer. Expand to inspect the need decision.",
      },
      artifactState: {
        done: "Generated",
        boundary: "Needs data",
        working: "Generating",
        failed: "Blocked",
        queued: "Queued",
        idle: "Pending",
      },
      inlineState: {
        done: "Done",
        boundary: "Needs data",
        working: "Processing",
        reworking: "Retrying",
        failed: "Failed",
        queued: "Queued",
        idle: "Standby",
      },
      dispatchFacts: {
        plan: {
          label: "Task plan",
          detail: "{{difficulty}} difficulty, route {{mode}}",
        },
        adaptive: {
          label: "Task match",
          detail:
            "{{level}} / {{pressure}}, {{roles}} reference angle(s), {{steps}} handling step(s), {{skills}} reference(s)",
        },
        reasoning: {
          label: "Reason",
        },
        plannedSkills: {
          label: "References for this turn",
        },
        contextBudget: {
          label: "Context budget",
          detail: "{{history}} history messages, {{materials}} selected material signals",
        },
        historyWindow: {
          label: "History window",
          full: "This turn read all {{total}} same-conversation messages.",
          truncated:
            "This turn read the latest {{recent}}/{{total}} history messages and included an older compressed summary.",
        },
          memory: {
            label: "History memory",
            detail: "Loaded {{count}} records, kinds {{kinds}}",
            detailWithRecords: "Loaded {{count}} records, kinds {{kinds}}; summaries {{records}}",
            recordChars: "{{count}} chars",
            recordTruncated: "truncated",
          },
          conversationContext: {
            label: "Conversation context",
            detail:
              "Read latest {{recent}}/{{total}} history messages and recovered {{relevantOlder}} older relevant message(s)",
          },
          context: {
            label: "Context",
            detail: "{{products}} products, {{knowledge}} knowledge records, {{sources}} sources",
        },
        rag: {
          label: "Knowledge retrieval",
          detail: "Matched {{sources}} local sources",
          degraded: "Retrieval degraded: {{reason}}",
        },
        workspace: {
          label: "Workspace materials",
          detail: "{{readiness}}, readable {{readable}}/{{total}}",
        },
        workspaceWarning: {
          label: "Material note",
          detail: "{{warning}}",
        },
        workspaceOmitted: {
          label: "Not in context",
          detail: "{{count}} source(s) omitted: {{refs}}",
        },
        route: {
          label: "Task decision",
          detail: "{{mode}}, lead {{role}}, strategy {{strategy}}",
        },
        skill: {
          label: "Reference tools",
          detail: "{{count}} tool reference event(s) in this turn",
        },
        output: {
          label: "Output complete",
          detail: "Answer and workspace persistence events returned",
        },
        empty: {
          label: "Waiting for events",
          detail: "This turn's process events will appear here after sync",
        },
      },
      dispatchPanel: {
        title: "Judgment notes for this turn",
        workbenchTitle: "Judgment notes",
        steps: "Reasoning path",
        routeType: "Method",
        studentReadTitle: "Judgment note",
        emptyTask: "Waiting for this turn's task",
        waitingStepTitle: "Preparing reasoning steps",
        waitingStepDetail: "The question, materials, and steps will appear here.",
        compactedStepDetail:
          "Read the question, history, and available materials, then organized the answer.",
        routeModes: {
          chat_bypass: "quick answer route",
          task_graph: "multi-step judgment route",
          single_agent: "main-angle route",
        },
        previewFiles: {
          quick_dispatch: "quick.dispatch.process.md",
          single_agent: "single.agent.process.md",
          multi_agent: "multi.agent.process.md",
        },
        preview: {
          heading: "# {{mode}}",
          request: "Business input: {{request}}",
          summary: "Task note: {{summary}}",
          adaptive: "Task match: {{summary}}",
          progress: "Progress: {{completed}}/{{total}} | {{stage}}",
          step: "{{index}} {{title}}: {{detail}}",
          fact: "{{label}}: {{detail}}",
          fallbackMode: "Judgment notes",
          fallbackFactLabel: "Turn materials",
          fallbackFactDetail: "Turn material status synced",
          fallbackStage: "Working",
          fallbackStepTitle: "Reasoning step",
          fallbackStepDetail: "Step status syncing",
          fallbackSummary: "Preparing the answer for this task",
        },
        stableDetail:
          "Simple or complex, I organize the reasoning first; only the number of steps, reference angles, and tool references changes.",
        studentUseTitle: "How to read it",
        studentUseDetail:
          "Check the method and materials first, then review step progress.",
        difficultyTitle: "Why some flows are short and others are long",
        difficultyDetail:
          "Simple questions use fewer steps. Complex questions add material checks, reference angles, and summary.",
        realnessTitle: "Why not every angle is shown",
        realnessDetail:
          "More handling details appear only when the question actually needs multiple reference angles.",
      },
      nextActions: {
        title: "Useful next steps",
        meta: {
          default: {
            label: "Current answer",
            detail: "These suggestions change with the current answer instead of staying as a fixed template.",
          },
          quick: {
            label: "Quick answer",
            detail: "This answer is concise, so you can continue with steps, evidence, or risks.",
          },
          product: {
            label: "Product context",
            detail: "This answer uses selected product context, so you can continue with selling points, audience, risks, or campaign direction.",
          },
          productNoMaterial: {
            label: "Needs product info",
            detail: "This question needs more product information; add materials first, then review the judgment.",
          },
          campaign: {
            label: "Campaign context",
            detail: "This answer is about campaign planning, so you can continue with goals, budget, rhythm, and an execution checklist.",
          },
          data: {
            label: "Data context",
            detail: "This answer is about metrics or anomalies, so you can add data, check causes, and list validation steps.",
          },
          task: {
            label: "Task context",
            detail: "This answer can become a task, so you can organize steps, checklists, and submission requirements.",
          },
          report: {
            label: "Report context",
            detail: "This answer can become review material, so you can add process, evidence, and improvement suggestions.",
          },
        },
        prompts: [
          "I will add product materials; judge product fit and risks again",
          "Turn the conclusion above into training task steps",
          "Continue with a campaign plan, budget, and execution checklist",
          "Turn this answer into a review report",
        ],
        groups: {
          default: [
            "Turn this conclusion into the 3 things I should do next",
            "List the missing materials I should add",
            "Format this as a coursework task brief",
            "Separate the risks, evidence, and suggestions",
          ],
          quick: [
            "Expand the short answer into actionable steps",
            "Keep only the conclusion and evidence for submission",
            "Ask about the key risk behind this conclusion",
            "Rewrite it in the order a student can execute",
          ],
          product: [
            "Recheck selling points, audience, and risks for the selected product",
            "Turn the product conclusion into a selection analysis table",
            "Continue with campaign directions for this product",
            "List product materials still needed",
          ],
          productNoMaterial: [
            "Tell me what product materials are needed before judging",
            "Make an initial selection judgment from current information",
            "Give me a product material checklist",
            "List possible risks now, then recheck after I add materials",
          ],
          campaign: [
            "Split the campaign into goal, budget, timing, and actions",
            "Continue with an execution checklist",
            "Check the campaign risks",
            "Format the campaign plan for submission",
          ],
          data: [
            "Break the data issue into metrics, causes, and validation",
            "List the operating data I still need to add",
            "Turn the diagnosis into an improvement checklist",
            "Follow up on the key metric most likely to affect results",
          ],
          task: [
            "Turn this into task steps and submission requirements",
            "Generate a checklist I can follow",
            "Split the task into today and later",
            "Check what materials this task still lacks",
          ],
          report: [
            "Turn this answer into a review report draft",
            "Add process, evidence, and improvement suggestions",
            "List report gaps from a teacher review perspective",
            "Compress the report into a class presentation version",
          ],
        },
      },
    },
    swarm: {
      lanes: {
        context: { label: "Context", value: "Products/platform/insights" },
        execution: { label: "Execution", value: "Campaigns/tasks/budget" },
        delivery: { label: "Delivery", value: "Files/briefing/review" },
        review: { label: "Review", value: "Checks/risks/follow-up" },
      },
      agents: {
        data: {
          name: "Data diagnosis",
          summary:
            "Analyze data, metrics, attribution, and business constraints",
          objective:
            "Analyze the task data, metrics, attribution, and constraints",
        },
        accounting: {
          name: "Budget review",
          summary: "Review cost, budget, SKU, and operating risk",
          objective:
            "Review campaign cost, traffic SKU, ads budget, and risk from the data diagnosis",
        },
        douyinAds: {
          name: "Qianchuan optimization",
          summary:
            "Plan structure, audiences, budget cadence, and ROI stop-loss",
          objective:
            "Break down Qianchuan plan structure, audience packages, bids, budget, ROI stop-loss, and live or short-video handoff",
        },
        creative: {
          name: "Creative output",
          summary: "Produce short-video, live, or campaign creative ideas",
          objective:
            "Create Douyin short-video, live, or campaign content ideas from the data diagnosis",
        },
        ops: {
          name: "Ops plan",
          summary:
            "Combine finance, Qianchuan, and creative output into an executable plan",
          objective:
            "Combine finance, Qianchuan, and creative output into an executable operations plan",
        },
        delivery: {
          name: "Deliverable doc",
          summary:
            "Store plan structure, checklist, and downloadable artifacts",
          objective:
            "Turn the operations plan into a reusable customer document",
        },
      },
      phases: {
        data: {
          title: "Phase 1: Data diagnosis",
          detail:
            "Identify metrics, attribution, platform context, and known constraints",
        },
        parallel: {
          title: "Phase 2: Parallel work",
          detail: "Finance, Qianchuan ads, and creative tasks run in parallel",
        },
        ops: {
          title: "Phase 3: Cross-check",
          detail:
            "Merge upstream conclusions and cross-check budget, creative, and launch cadence",
        },
        delivery: {
          title: "Phase 4: Delivery summary",
          detail: "Compose final strategy, checklist, and deliverable files",
        },
        dynamicTitle: "Phase {{index}}: {{title}}",
        dynamicDeliveryTitle: "Phase {{index}}: Delivery summary",
        dynamicDeliveryDetail:
          "Summarize task results into a deliverable plan, checklist, and document.",
      },
      handoffs: {
        context: {
          title: "Context",
          detail:
            "Product selection analysis, platform connections, and diagnosis guidance become task input",
        },
        execution: {
          title: "Execution",
          detail:
            "Campaigns, task board, and training tasks carry the actions",
        },
        delivery: {
          title: "Delivery",
          detail:
            "Deliverables, daily briefing, and review history store results",
        },
        review: {
          title: "Review",
          detail:
            "Chat history, execution records, and team capability notes stay reusable",
        },
      },
      statePill: {
        done: "Done",
        boundary: "Needs data",
        working: "Working",
        queued: "Queued",
        failed: "Failed",
        reworking: "Retry",
        skipped: "Skipped",
        idle: "Standby",
      },
      dockState: {
        done: "Done",
        boundary: "Needs data",
        working: "Reasoning",
        reworking: "Retrying",
        failed: "Failed",
        queued: "Waiting",
        skipped: "Skipped",
        idle: "Waiting for task",
      },
      stage: {
        waitingInput: "Waiting for task input",
        planning: "Organizing the reasoning path",
        historyWaiting: "Historical chat, waiting for a new task",
        failedAgent: "Handling blocked: {{agent}}",
        reworkingAgent: "Retrying: {{agent}}",
        workingAgent: "Working: {{agent}}",
        multiTasking: "Handling multiple items",
        finished: "Judgment summarized",
      },
      timeline: {
        planning: [
          "Question received. Organizing the reasoning path.",
          "Steps, material boundaries, and key checks will sync to the judgment notes.",
        ],
        history: [
          "This is a historical conversation and the current run has not started.",
          "Send a new task to rebuild the processing steps.",
        ],
        empty: [
          "Waiting for your question, then the steps will be organized.",
          "Data diagnosis runs first; budget, ads, and creative tasks run in parallel.",
          "The ops plan cross-checks and summarizes final delivery.",
        ],
        events: {
          commanderThinking:
            "Analyzing the question and preparing the reasoning path.",
          routeDecided:
            "Question type identified. Using {{strategy}} style, reference mode {{mode}}, focus angle {{role}}.",
          graphPlanned: "Prepared {{count}} handling steps.",
          nodeStarted: "{{agent}} started pass {{attempt}}.",
          nodeRework: "{{agent}} is reorganizing: {{instruction}}",
          nodePassed: "{{agent}} finished the check.",
          nodeBoundary:
            "{{agent}} completed a category-level note without blocking the final answer.",
          nodeFailed: "{{agent}} was blocked: {{reason}}",
          skillRequested: "{{role}} is checking {{skill}}.",
          skillResult: "{{skill}} reference result: {{status}}.",
          approvalRequired: "Approval created: {{skill}}, risk {{risk}}.",
          workerStarted: "{{agent}} started organizing.",
            workerFinished: "{{agent}} finished organizing.",
            contextLoaded:
              "Loaded {{products}} product(s) and {{knowledge}} product knowledge record(s).",
            conversationContextLoaded:
              "Same-conversation context loaded: latest {{recent}}/{{total}} history messages.",
            ragContextLoaded:
              "Knowledge retrieval complete: {{sources}} source(s).",
          ragContextDegraded: "Knowledge retrieval degraded: {{reason}}",
          workspaceContextLoaded:
            "Training materials loaded: {{sources}} source(s), {{readiness}}.",
          workspaceContextPackLoaded:
            "Readable context attached: {{readable}}/{{total}} source(s) can be used directly.",
          workspaceRunLinked: "Training task linked: {{title}}.",
          workspaceTasksLinked: "Saved {{count}} training task item(s).",
          workspaceTaskUpdated: "Training task status updated: {{status}}.",
          workspaceTaskBoundary:
            "Training task recorded as boundary review: {{status}}.",
          workspaceRunCompleted: "Final answer saved to the submission record.",
          workspaceArtifactCreated: "Review material created: {{title}}.",
          agentLearningRecordCreated: "Review note saved: {{category}}.",
          qualityChecked: "Quality check complete: {{score}}, {{status}}.",
          qualityPassed: "passed",
          qualityNeedsWork: "needs work",
          workspaceRunFailed: "Training submission marked failed.",
          workspacePersistenceError:
            "Training record persistence failed: {{action}}.",
          finished: "This answer finished.",
          failed: "This answer was blocked: {{reason}}",
          graphFinished: "Reasoning path finished and summary output started.",
        },
      },
      cards: {
        readTarget: {
          title: "Read target",
          planMeta: "Reasoning path",
          userMeta: "User task",
          plannedDescription: "Split the task into {{count}} processing steps.",
          inputDescription:
            "Reading the user goal and preparing the reasoning path.",
          waitingLine: "Waiting for customer business goal.",
        },
        assign: {
          title: "Arrange steps",
          meta: "Waiting arrangement",
          description:
            "Preparing processing steps, dependencies, and acceptance rules.",
          currentTask: "Current task: {{agent}}",
          stateLine: "Handling state: {{state}}",
          latestAction: "Latest action: {{action}}",
          latestPlanning: "Latest action: breaking down task",
        },
        output: {
          reportTitle: "Answer output",
          materialTitle: "Supplemental context",
          materialMeta: "Industry and platform context",
          description:
            "After output starts, this switches to real summary, evidence, and risks.",
          fallbackLines: [
            "Coffee conversion",
            "Qianchuan ads review",
            "Ops budget and launch cadence",
          ],
        },
        contextLoaded: {
          title: "Product materials",
          meta: "{{products}} products / {{knowledge}} knowledge",
          description:
            "User-owned product facts were loaded before the answer.",
          productLine: "Product id: {{id}}",
        },
        ragContext: {
          title: "Knowledge retrieval",
          meta: "{{sources}} sources / {{knowledge}} knowledge",
          description:
            "Citable sources were retrieved from local product knowledge.",
          degradedDescription:
            "No local knowledge matched and external search data is not connected.",
          sourceLine: "{{type}}: {{title}}",
          queryLine: "Query terms: {{terms}}",
          fallbackLine: "Degraded: {{reason}}",
        },
        workspaceContext: {
          title: "Training materials",
          meta: "{{total}} sources / {{readiness}}",
          packTitle: "Materials usable in the answer",
          packMeta: "{{readable}}/{{total}} source(s) readable",
          readyDescription:
            "Files, materials, and review materials were attached for this answer.",
          partialDescription:
            "Some files, materials, or review materials were loaded; some materials are still missing.",
          emptyDescription:
            "No files, materials, or review materials are ready for this training task.",
          packReadyDescription:
            "Stored material text, review material text, and attachment text are used in this answer.",
          packPartialDescription:
            "Only part of the sources have readable text; binary attachments are not treated as read.",
          compactLine: "Training material sources: {{total}}",
          readableLine: "Readable text: {{count}}",
          sourceLine: "{{type}}: {{title}}",
          packItemLine: "{{kind}}: {{title}}",
          warningLine: "Note: {{warning}}",
          omittedLine: "Not in context: {{ref}}",
        },
        skill: {
          title: "Reference tool",
          statusLine: "Status: {{status}}",
          requested:
            "The tool reference was received and is being handled or waiting for external data.",
          completed: "The reference tool returned a result.",
          deferred:
            "This tool needs external data, so it is deferred for now.",
          approvalRequired:
            "This tool is high-risk or writes to a platform, so approval is required before it continues.",
        },
        rework: {
          title: "Recheck",
          description:
            "Acceptance failed and the system triggered a rework instruction.",
        },
        boundary: {
          title: "Needs data",
          description:
            "This step lacks a selected product or connected external data, and does not block the final answer.",
          line: "Handled as a category-level boundary: keep missing data, risks, and next steps without showing it as an answer failure.",
        },
        failed: {
          title: "Handling blocked",
          description:
            "The current step is temporarily blocked. Check model service or upstream dependencies if needed.",
        },
        workspace: {
          linkedTitle: "Training task linked",
          linkedDescription:
            "This answer is now attached to a training task record.",
          createdMeta: "Created training task",
          reusedMeta: "Reused training task",
          tasksLinkedTitle: "Tasks persisted",
          tasksMeta: "{{count}} tasks",
          tasksLinkedDescription:
            "Processing steps were written as training task snapshots.",
          taskUpdatedTitle: "Task snapshot updated",
          taskUpdatedDescription:
            "AI training analysis status was saved.",
          taskBoundaryTitle: "Needs data",
          taskBoundaryDescription:
            "This node is a category-level data boundary without a selected product. It is saved but does not block answer synthesis.",
          completedTitle: "Report saved",
          completedDescription:
            "The final answer was written to the training submission record.",
          artifactCreatedTitle: "Review material created",
          artifactCreatedDescription:
            "The final answer is now a standalone review material that can be queried, copied, and submitted.",
          approvalRequiredTitle: "Approval required",
          approvalRequiredMeta: "{{status}} / {{risk}}",
          approvalRequiredDescription:
            "The high-risk platform action has entered review and will not execute automatically.",
          approvalLine: "Approval {{id}}: {{status}}",
          approvalSkillLine: "Capability: {{skill}}",
          approvalReasonLine: "Reason: {{reason}}",
          approvalAlertLine: "Alert: {{message}}",
          qualityCheckedTitle: "Quality checked",
          qualityPassedMeta: "Passed",
          qualityNeedsWorkMeta: "Needs work",
          qualityCheckedDescription:
            "Answer quality check, trust scoring, and learning persistence are complete.",
          qualityScoreLine: "Quality score: {{score}}",
          trustScoreLine: "Trust score: {{score}} / {{level}}",
          learningRecordLine: "Learning records: {{count}}",
          failedTitle: "Training task run failed",
          failedDescription: "The run failure reason was saved.",
          errorTitle: "Persistence issue",
          errorDescription:
            "The AI answer continues, but this training record needs inspection.",
          runLine: "Run {{id}}: {{status}}",
          runTaskLine: "Task {{id}}: {{status}}",
          artifactLine: "Review material {{id}}: {{title}}",
          savedReportLine:
            "Final report saved and available after refreshing the training task.",
        },
      },
      screen: {
        acceptanceLine: "Acceptance rule {{index}}: {{item}}",
        responsibilityLine: "Reference focus: {{summary}}",
        objectiveLine: "Turn goal: {{request}}",
        reviewLine: "Review note: {{reason}}",
        reportMode: "Report",
        taskMode: "Task",
        deliveryFilename: "operation.execution.plan.docx",
      },
      runtimeDescriptions: {
        started: "{{agent}} started pass {{attempt}}.",
        reported:
          "{{agent}} organized a stage result and is waiting for summary.",
        passed: "{{agent}} finished the check.",
        boundary:
          "{{agent}} produced a category-level boundary for summary.",
        failed: "{{agent}} was blocked.",
        reworking: "{{agent}} is reorganizing output from feedback.",
        skillRequested: "Checking {{skill}}.",
        skillResult: "{{skill}} reference result: {{status}}.",
      },
      runtimeActions: {
        commanderThinking: "Breaking down task",
        graphPlanned: "Judgment path ready",
        nodeReady: "Step queued",
        nodeStarted: "Started organizing",
        nodeReported: "Stage result ready",
        nodeEvaluated: "Result reviewed",
        nodeRework: "Recheck started",
        nodePassed: "Check complete",
        nodeBoundary: "Needs data",
        nodeFailed: "Handling blocked",
        skillRequested: "Reference checked",
        skillResult: "Reference result",
        graphFinished: "Judgment path finished",
        synced: "State synced",
      },
      reportLines: {
        risk: "Risk: {{risk}}",
        question: "Open question: {{question}}",
      },
      fallbackResponse: {
        failed:
          "This turn is temporarily blocked, usually because the model service, tools, or upstream dependencies were unavailable.",
        boundary:
          "This reference angle was limited by no selected product, missing live metrics, or unavailable external data, so it produced a stage conclusion and missing-data list.",
        reworking:
          "The result needs another pass and is being adjusted from feedback.",
        working:
          "{{agent}} is organizing the request. Results will update here.",
        done: "{{agent}} finished the stage and is waiting for final summary.",
        idle: "After the turn starts, this area shows the question, handling result, and material summary.",
      },
    },
  },
  agentOutput: {
    prompt:
      "I run coffee operations. My main marketing platform is Douyin, and May Day is coming soon. Can you help me write an operations plan to improve conversion rate?",
    sectionsNavigation: "Reading guide",
    backend: {
      eyebrow: "Review Materials",
      title: "Review report materials",
      description:
        "This page gathers plans, checklists, and review materials generated by the AI training assistant so students can inspect, copy, download, and submit them.",
      refresh: "Refresh",
      loading: "Loading review materials",
      listTitle: "Review material list",
      listDesc: "{{artifacts}} material(s) / {{workspaces}} training task(s)",
      emptyTitle: "No review materials yet",
      emptyDescription:
        "Return to the multi-agent training assistant and send a training question. Generated plans or reports will appear here.",
      errorTitle: "Failed to load review materials",
      noSelectionTitle: "No material selected",
      noSelectionDescription:
        "Select a material to inspect content, source training task, and export metadata.",
      noContent: "No content or preview saved.",
      copy: "Copy",
      download: "Download",
      meta: {
        workspace: "Training task",
        artifactId: "Material ID",
        runId: "Submission ID",
        format: "Format",
        storage: "Storage",
        size: "Chars/size",
      },
    },
    hero: {
      badges: {
        output: "Agent output page",
        category: "Coffee operations",
        platform: "Douyin May Day conversion",
      },
      title: "May Day Douyin coffee conversion plan",
      description:
        "The agent reply is organized as a business page: conclusions, metrics, execution path, content script, finance guardrails, and rollback rules.",
      promptLabel: "User input",
      dispatchTitle: "Agent routing",
      dispatchDescription:
        "Primary agent: operations expert. Supporting roles: content creator and finance analyst.",
      export: "Export plan",
    },
    metrics: {
      target: {
        label: "Target lift",
        value: "+35%",
        description: "Product-click to order conversion in live room",
      },
      budget: {
        label: "Budget",
        value: "18k",
        description: "Short-video boost plus Qianchuan test spend",
      },
      period: {
        label: "Timeline",
        value: "10 days",
        description: "April 24 to May 3",
      },
      guardrail: {
        label: "Stop-loss",
        value: "ROI 1.2",
        description: "Pause if below threshold for two hours",
      },
    },
    plan: {
      eyebrow: "Execution workflow",
      title: "Four-step path from traffic bait to review",
      actionsTitle: "Actions in the next 24 hours",
    },
    steps: {
      sku: {
        title: "Entry SKU",
        owner: "Operations",
        timing: "Before April 24",
        detail:
          "Launch a 9.9 trial pack and 19.9 cold-brew freeze-dried pack with one-order limit, mapped to May Day travel and stay-home coffee scenarios.",
      },
      content: {
        title: "Content warm-up",
        owner: "Content",
        timing: "April 25-28",
        detail:
          "Publish ten May Day short videos with 500 each in boost spend. Replace any creative whose five-second completion rate falls below 35%.",
      },
      live: {
        title: "Live conversion",
        owner: "Live",
        timing: "April 29-May 3",
        detail:
          "Rotate travel, stay-home, and holiday-shift scenarios every 20 minutes, paired with buy-three-save-20 and buy-five-get-a-cup offers.",
      },
      traffic: {
        title: "Paid scaling",
        owner: "Ads",
        timing: "Full campaign",
        detail:
          "Target 18-35 users in tier-one and tier-two cities interested in commuting, travel, and food, then adjust hourly by conversion and real ROI.",
      },
    },
    actions: {
      inventory:
        "Confirm coffee category, AOV, and inventory today so the entry SKU does not drag down gross margin.",
      benefit:
        "Unify the live-room first-screen offer into one sentence: stock May Day coffee today, buy three and save 20.",
      hook: "Use the first three seconds for a scene hook instead of the brand, such as a travel coffee price comparison.",
      comment:
        "Pin a comment asking users to share their May Day plan for a monthly coffee giveaway, while collecting scenario insights.",
    },
    sections: {
      summary: {
        title: "30-second conclusion",
        eyebrow: "Execution first",
        content:
          "Do not rely on broad discounts before May Day. Use a low-priced trial pack to capture holiday traffic, then lift order value through live-room bundles. Test ten scenario videos first, then concentrate spend on creatives that drive live clicks and orders.",
        bullets: [
          "Keep entry SKUs under 15% of total exposure so low-price demand does not eat margin.",
          "Use bundles as the live-room hero; the trial pack exists only for acquisition.",
          "Review conversion, ROI, and return rate every two hours, then roll back below threshold.",
        ],
      },
      diagnosis: {
        title: "Conversion bottleneck",
        eyebrow: "Current assumption",
        content:
          "Coffee has clear May Day use cases. Users are less moved by cheap coffee than by concrete scenes where coffee is useful during travel, staying home, or working through the holiday. The likely gap is inconsistency between content hooks and live-room conversion.",
        bullets: [
          "Holiday-scene videos paired with product-parameter live rooms create drop-off.",
          "Vague benefits give users no reason to order immediately.",
          "Without purchase limits and bundles, the entry SKU can become a loss SKU.",
        ],
      },
      script: {
        title: "Copy-ready short-video script",
        eyebrow: "Content creator support",
        content:
          "Opening: May Day coffee at scenic spots costs 30 a cup, so I keep this in my bag. Body: one sachet works with cold water in the car, tent, or hotel, at about three per cup. CTA: today the live room has a 9.9 trial pack and buy-three-save-20 offer.",
        bullets: [
          "Shot 1: compare scenic-spot coffee pricing with the coffee in a backpack.",
          "Shot 2: show cold-water brewing completed within ten seconds.",
          "Shot 3: show the live-room offer popup and emphasize limited timing.",
        ],
      },
      finance: {
        title: "Finance guardrails",
        eyebrow: "Finance analyst support",
        content:
          "The 9.9 entry SKU needs a per-customer loss cap and should not be judged by GMV ROI alone. Use real ROI after returns, commission, product cost, logistics, and gift cost.",
        bullets: [
          "Keep total paid traffic spend under 18% of projected revenue.",
          "Reduce entry-SKU exposure when return rate exceeds 42%.",
          "Reserve cup gifts for high-AOV bundles so gifts do not consume margin.",
        ],
      },
    },
  },
  agentCapabilities: {
    header: {
      eyebrow: "Training Guide",
      title: "AI training capability and usage guide",
      description:
        "This page explains how students use AI Q&A, product analysis, campaign planning, data diagnosis, and review reports around one ecommerce training task.",
      action: "Open multi-agent training assistant",
    },
    principle: {
      title: "How students use it",
      description:
        "Students do not just open a chat box. They ask around a training task, choose product and campaign context, then store AI output as tasks, review records, and review materials.",
    },
    capabilities: {
      title: "Core feature entries",
      description:
        "These entries reuse real existing capabilities, but rename and explain them around common student training actions.",
      items: [
        {
          title: "Multi-agent training assistant",
          description:
            "The main entry where students state the problem, the system identifies the task, calls agents, and shows the process.",
        },
        {
          title: "Product selection analysis",
          description:
            "Uses manually selected products, assets, selling points, and competitor materials; without selection, the answer stays as a general training Q&A.",
        },
        {
          title: "Campaign planning practice",
          description:
            "Organizes goals, budget, rhythm, scripts, ads, and execution lists into submit-ready campaign planning work.",
        },
        {
          title: "Data diagnosis guidance",
          description:
            "Analyzes products, platform data, anomaly metrics, competitors, and rule boundaries to help students decide the next move.",
        },
        {
          title: "Business result tracking",
          description:
            "Shows process records, notifications, alerts, and stage results so students know progress and review needs.",
        },
        {
          title: "Training tasks and deliverables",
          description:
            "Turns AI output into tasks, runs, execution suggestions, and teacher-checkable submissions.",
        },
        {
          title: "Review report",
          description:
            "Stores plans, checklists, reports, and documents as review materials for preview, copy, download, and archive.",
        },
        {
          title: "Teacher review records",
          description:
            "High-risk actions and key submissions enter approvals, execution queue, and audit logs for teacher review.",
        },
      ],
    },
    modes: {
      title: "How to choose answer modes",
      description:
        "The same question can use different answer styles based on the classroom goal, so students do not only receive generic conclusions.",
      items: [
        {
          title: "Teaching mode",
          description:
            "Good for learning while doing. The answer adds reasoning, operating steps, and review metrics.",
        },
        {
          title: "Brief answer",
          description:
            "Good for quickly confirming direction. The answer prioritizes conclusions, key actions, and missing inputs.",
        },
        {
          title: "Full planning",
          description:
            "Good for assignments or plans. The answer covers goals, path, risks, checklists, and next steps.",
        },
        {
          title: "Auto dispatch",
          description:
            "The system decides whether product, campaign, data-diagnosis, or other specialist collaboration is needed.",
        },
      ],
    },
    qa: {
      title: "Common usage questions",
      description:
        "These questions help students know when to add materials and when to keep asking follow-up questions.",
      items: [
        {
          question: "Can I ask without selecting a product?",
          answer:
            "Yes. The system answers as a general ecommerce training question, but it will not invent specific inventory, price, material, or historical conversion data.",
        },
        {
          question: "When should I reference a product or campaign?",
          answer:
            "Reference materials when the question involves a specific SKU, selling point, asset, campaign budget, rhythm, or review metric.",
        },
        {
          question: "Can the answer be submitted as the final assignment?",
          answer:
            "Not directly. Students should verify data sources, add classroom requirements, and organize the execution process and evidence.",
        },
        {
          question: "Will historical context keep being used?",
          answer:
            "The same conversation keeps the task context, but after changing products, campaigns, or modes, state the current goal again in the question.",
        },
      ],
    },
    flow: {
      title: "How one training task flows",
      description:
        "The student-visible main flow should stay clear, continuous, and traceable.",
      items: [
        "The student creates a new conversation or opens a historical one, then enters a training question.",
        "The student may choose answer mode, collaborating agents, and product or campaign-plan context.",
        "The system plans the task, calls models, data diagnosis, and analysis capabilities, and shows the process in real time.",
        "Process cards appear above the answer and collapse after completion, so they do not block reading.",
        "Final answers and deliverables are stored in training tasks, review reports, teacher review, or business result pages.",
      ],
    },
    lanes: {
      title: "Training navigation task areas",
      description:
        "The left navigation is not just a page menu. It exposes the most-used feature entries for students completing training tasks.",
      items: [
        {
          title: "Multi-agent training assistant",
          description:
            "The main entry for training questions, strategy, product references, and process display.",
        },
        {
          title: "Product selection analysis",
          description:
            "Manages products, materials, knowledge, competitors, and manually selected task context.",
        },
        {
          title: "Campaign planning practice, data diagnosis guidance, business result tracking",
          description:
            "Connects campaign planning, metric attribution, process records, briefings, alerts, and source boundaries.",
        },
        {
          title: "Training tasks, review, and review report",
          description:
            "Carries training tasks, teacher review records, and final review reports.",
        },
      ],
    },
    boundary: {
      title: "Current design boundary",
      description:
        "Scheme A only reorganizes frontend entries and wording so students can understand existing capabilities; it does not add core capability.",
      backend:
        "Real capability still comes from the existing system. The frontend does not show fake buttons, fake states, or fake search results.",
      context:
        "Product materials are injected only after the user manually selects them. Without a selection, the answer uses ordinary Q&A context.",
      governance:
        "Actions such as platform writes, budget changes, publishing, and deletion should enter approval and audit.",
    },
  },
  login: {
    brand: "电商多智能体",
    hero: {
      titleLine1: "Enter ecommerce AI training,",
      titleLine2: "finish selection, planning, and review.",
      desc1:
        "First open the multi-agent training assistant and state the ecommerce training problem you want to solve.",
      desc2:
        "Then use product selection materials, campaign planning, data diagnosis, training tasks, and review reports as needed.",
      desc3:
        "After signing in, you can save conversations, reference product materials, submit tasks, and review teacher feedback and review materials.",
    },
    welcome: "WELCOME BACK",
    title: "Sign in to your training account",
    subtitle:
      "Use your account and password to enter the ecommerce AI training workspace.",
    form: {
      password: {
        label: "Password",
        placeholder: "Enter password",
      },
      submit: "Sign in",
      submitting: "Signing in...",
      identifier: {
        label: "Account",
        placeholder: "Enter username or email",
      },
    },
    footer: {
      toRegisterPrefix: "Don't have an account?",
      toRegisterAction: "Register",
    },
    error: {
      title: "Sign-in did not finish",
      invalidCredentials:
        "Login failed, please check your account and password.",
      authUnavailable:
        "The account service is temporarily unavailable. Confirm the auth service network is reachable, then sign in again.",
      backendUnavailable:
        "Your account was verified, but the local business backend did not respond. Start the 19878 backend service and 15432 database, then sign in again.",
      workspaceUnavailable:
        "Your account was verified, but training workspace permissions were not restored. Confirm this account is initialized for this product, or try again later.",
    },
  },
  register: {
    brand: "电商多智能体",
    hero: {
      titleLine1: "Create your account,",
      titleLine2: "start ecommerce AI training.",
      desc1:
        "Use one account to keep training conversations, product selection materials, campaign tasks, and review reports.",
      desc2:
        "After registering, complete product selection, campaign planning, data diagnosis, and review submission around one training task.",
    },
    welcome: "CREATE ACCOUNT",
    title: "Register your training account",
    subtitle:
      "Create an account to enter 电商多智能体 and start your ecommerce AI training task.",
    form: {
      password: {
        label: "Password",
        placeholder: "Enter password",
      },
      confirmPassword: {
        label: "Confirm password",
        placeholder: "Enter password again",
      },
      submit: "Register",
      submitting: "Registering...",
      username: {
        label: "Username",
        placeholder: "Enter username",
      },
      displayName: {
        label: "Display name (optional)",
        placeholder: "Enter display name",
      },
      email: {
        label: "Email (optional)",
        placeholder: "Enter email",
      },
    },
    footer: {
      toLoginPrefix: "Already have an account?",
      toLoginAction: "Sign in",
    },
    error: {
      passwordMismatch: "Passwords do not match.",
    },
    profile: {
      schoolName: {
        label: "School",
        placeholder: "Enter school name",
      },
      teamName: {
        label: "Team name",
        placeholder: "Enter team name",
      },
    },
  },
} as const

export default messages
