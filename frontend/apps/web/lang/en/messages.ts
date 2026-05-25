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
        workspace: "Training flow",
        governance: "Review and records",
        agentSection: "Training outputs",
        home: "Data diagnosis",
        products: "Product materials",
        campaigns: "Campaign plans",
        workspaces: "Tasks and submissions",
        platformConnect: "Platform data",
        approvals: "Teacher review records",
        operations: "Result records",
        agent: "AI training assistant",
        agentOutput: "Report materials",
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
        title: "Data diagnosis",
        description:
          "Start with products, campaigns, operating data, and pending items to decide what this training task should solve first.",
        boundaryTitle: "What students can do now",
        boundaryDescription:
          "This page helps students understand available materials and abnormal signals. It does not change platform data or publish content automatically.",
      },
      products: {
        eyebrow: "Product Training",
        title: "Product materials",
        description:
          "Review products, product knowledge, file assets, material versions, competitors, and changes as context for selection analysis and AI training questions.",
      },
      campaigns: {
        eyebrow: "Campaign Practice",
        title: "Campaign plans",
        description:
          "Organize campaign goals, budgets, periods, outputs, and action metadata into a submittable plan.",
      },
      workspaces: {
        eyebrow: "Training Tasks",
        title: "Tasks and submissions",
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
        title: "Result records",
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
      },
    },
    campaigns: {
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
        "Entries follow the way a student completes one ecommerce training task: ask the multi-agent assistant, then use materials, planning, submission, and review.",
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
      "Hi, I am the AI training assistant. Enter a training task such as product selection, campaign planning, data anomaly diagnosis, or review report. I will detect the need, dispatch collaborating agents, and sync the process, conclusion, and deliverables.",
    answer: {
      sectionLabels: {
        final: "Final",
        team: "Team",
        action: "Action",
      },
    },
    streaming: "Thinking...",
    error: "Agent reply failed. Please try again later.",
    errorBadge: "Reply failed",
    boundaryNotice:
      "Note: some roles hit a no-selected-product or missing-live-metrics boundary. The interface treats these internal checks as reviewable business boundaries, not user-facing failures.",
    missingStructuredOutput:
      "The previous run did not produce complete structured output. Send the task again and I will rebuild the execution plan and sync the process to the task process panel.",
    streamTimedOut:
      "This run timed out and the process has been collapsed. Review the partial result or send the task again to continue.",
    loginRequired: {
      title: "Sign in to run the cloud agent",
      description:
        "You can draft the ecommerce question and choose a strategy first. Sign in when you are ready to call the cloud model and save the conversation.",
      message:
        "Sign in to call the cloud agent and save the conversation. You can still draft the question and choose a strategy here.",
      inline: "Sign in to run the cloud agent and save history",
      action: "Sign in",
    },
    sidebar: {
      newChat: "New task",
      search: "Search history",
      recents: "Recents",
      chat: "AI training assistant",
      more: "More",
      less: "Collapse",
      moreHistory: "Show {{count}} more",
      lessHistory: "Collapse to latest 5",
      expandHistory: "Expand history",
      collapseHistory: "Collapse history",
      expandFunctionNav: "Expand function navigation",
      collapseFunctionNav: "Collapse function navigation",
      empty: "No conversations yet",
    },
    shell: {
      brandEyebrow: "电商多智能体",
      brandTitle: "电商多智能体",
      brandDescription: "Ecommerce AI training workspace",
      topbarTitle: "Enter a training question",
      topbarMeta: "电商多智能体 · AI training collaboration",
      sections: {
        create: "",
        workspace: "Training navigation",
        recents: "History",
      },
      navigation: {
        newChat: "New chat",
        capabilities: "Capability guide",
        smartOps: "Multi-agent training assistant",
        productBoard: "Product materials",
        campaigns: "Campaign plans",
        platform: "Platform data",
        marketResearch: "Data diagnosis",
        operations: "Result records",
        briefing: "Daily briefing",
        alerts: "Alerts",
        deliverables: "Report materials",
        approvals: "Teacher review records",
        taskBoard: "Tasks and submissions",
        systemDashboard: "System dashboard",
        team: "AI team",
        history: "History",
      },
    },
    strategy: {
      current: "Agent work mode",
      options: {
        learning: {
          label: "Teaching mode",
          hint: "For students and beginners: explain terms, reasoning, and actionable steps",
          shortHint: "Teaching",
        },
        quick: {
          label: "Quick answer",
          hint: "Best for fast judgment: direct conclusion, key reason, and next steps without a long process",
          shortHint: "Short",
        },
        execution: {
          label: "Rich execution",
          hint: "Best for reports and delivery: full plan, role split, tables, risks, and acceptance checks",
          shortHint: "Full plan",
        },
      },
    },
    composer: {
      placeholder:
        "Enter a training task. For example: judge whether this product can run a campaign, then give product, planning, data, and review steps...",
      loginPlaceholder:
        "Draft your training question here. Sign in when you are ready to use the AI training assistant...",
      hint: "Business context connected",
      send: "Send",
      stop: "Stop",
      status: {
        sending:
          "The AI training assistant is analyzing. You can stop this answer at any time.",
      },
      attachTitle: "Upload attachment",
      attachShort: "Upload",
      attachPrompt:
        "Add the file or data constraints here, for example: product price, budget, inventory, target audience, current conversion rate, and platform account status.",
      agentTitle: "Choose collaborating agents",
      agentName: "AI training assistant",
      productContext: {
        label: "Products for this turn",
        compactLabel: "Product",
        noneCompactLabel: "No product",
        compactTitle: "Choose product context for this turn",
        loading: "Loading products",
        empty: "No active products",
        noneSelected: "No product selected",
        choose: "Choose saved product",
        collapse: "Collapse products",
        clear: "Clear products",
        noneHint:
          "This turn will not read saved product data and will answer as a general ecommerce question.",
        manualHint:
          "This answer references product materials only after you manually select a saved product.",
        searchPlaceholder:
          "Search product name, brand, category, platform, or status",
        selectedHint:
          "This turn references only the selected product materials.",
        selectedCount: "{{count}} product selected",
        moreCount: "{{count}} more",
        noSearchResult: "No matching product",
        status: {
          active: "Active",
          draft: "Draft",
          listing: "Presale/listing",
          archived: "Archived",
          unknown: "Unknown",
        },
      },
      campaignContext: {
        label: "Campaign context",
        compactLabel: "Campaign",
        noneCompactLabel: "No campaign",
        compactTitle: "Choose campaign context for this turn",
        loading: "Loading campaigns",
        empty: "No available campaigns",
        noneSelected: "No campaign selected",
        choose: "Choose campaign",
        collapse: "Collapse campaigns",
        clear: "Clear campaigns",
        noneHint:
          "This turn will not read saved campaign plans, budget, or execution data.",
        manualHint:
          "This answer references campaign materials only after you manually select a campaign.",
        searchPlaceholder:
          "Search campaign name, type, status, budget, or date",
        selectedHint:
          "This turn will read the selected campaigns' goals, budget, dates, status, and saved outputs.",
        selectedCount: "{{count}} campaign selected",
        budget: "Budget {{value}}",
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
        compactLabel: "Sources",
        noneCompactLabel: "No sources",
        compactTitle: "Choose sources for this turn",
        noneSelected: "No source selected",
        selectedCount: "{{count}} source selected",
        availableCount: "{{count}} available",
        readableCount: "{{count}} readable",
        clear: "Clear sources",
        empty:
          "No uploaded files are available in this conversation. Upload a file first, then select it for this turn.",
        manualHint:
          "This answer references these files only after you manually select them for this turn.",
        selectedHint:
          "This turn will read only the selected current-conversation sources.",
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
        compactLabel: "Mode",
        compactTitle: "Adjust answer mode, agent collaboration, and model",
        toggle: "Task settings",
        rolesCount: "{{count}} agents",
        depthSummary: "Depth: {{depth}}",
        selectedPrefix: "Selected",
        hideSelection: "Hide",
        showSelection: "Show selected mode",
        autoRolesSummary: "System auto",
        multiAgentSummary: "{{count}} agents collaborating",
      },
      runtime: {
        label: "Collaborating agents",
        loading: "Loading runtime",
        model: "Model",
        modelSettings: "Model settings",
        modelHint:
          "The configured Doubao model is used by default. If it is not configured, only available models are shown.",
        learning: "Explanation level",
        collaboration: "Agent collaboration",
        routingMode: "Routing mode",
        roleMode: "Collaboration mode",
        roles: "Collaborating agents",
        singleRole: "Selected agent",
        primary: "Lead agent",
        leadRole: "Lead agent",
        supportRoles: "Supporting agents",
        selectedRoles: "Select collaborating agents",
        roleColumn: "Agent",
        primaryShort: "Main answer",
        supportShort: "Support",
        singleRoleHint:
          "Click one agent card to let that agent lead this answer.",
        leadRoleHint:
          "Click an agent card to switch the lead agent. The lead owns synthesis and the final plan.",
        supportRolesHint:
          "Select supporting agents to add evidence, risks, materials, and action suggestions.",
        answerRoleBadge: "Current agent",
        primaryBadge: "Lead answer",
        supportBadge: "Support",
        currentLead: "Current lead agent",
        currentSupport: "Supporting agents: {{roles}}",
        noSupportRoles: "No supporting agent selected",
        advanced: "More settings",
        explanationDepth: "Explanation depth",
        autoRolesHint:
          "The system decides whether this turn needs one or multiple agents.",
        manualRolesHint:
          "Choose one lead agent for synthesis, then select supporting agents.",
        roleModeHint:
          "Choose how this turn calls agents: automatic dispatch, one selected agent, or selected multi-agent collaboration.",
        roleModes: {
          auto: {
            label: "Auto dispatch",
            description: "Backend decides",
          },
          single: {
            label: "Single agent",
            description: "One agent",
          },
          manual: {
            label: "Multi-agent",
            description: "Collaborative output",
          },
        },
        routingModes: {
          commander: {
            label: "Commander",
            description: "Commander analyzes first",
          },
          fast: {
            label: "Fast route",
            description: "Match configured task graph first",
          },
        },
        learningLevels: {
          beginner: {
            label: "Low",
            description: "Use fewer terms and focus on operating steps.",
          },
          higher_vocational: {
            label: "Medium",
            description: "Explain with ecommerce job practice in mind.",
          },
          professional: {
            label: "High",
            description: "Keep fuller metrics, reasoning, assumptions, and risks.",
          },
        },
        roleLabels: {
          chat: { name: "Chat assistant" },
          ops: { name: "Operations strategist" },
          data: { name: "Data analyst" },
          service: { name: "Customer success specialist" },
          creative: { name: "Content creative" },
          engineering: { name: "Technical architect" },
          accounting: { name: "Finance analyst" },
          design: { name: "Designer" },
          web: { name: "SEO specialist" },
          douyin_ads: { name: "Qianchuan ads specialist" },
        },
        capabilityLabels: {
          "role.chat": "chat role",
          "role.ops": "ops role",
          "role.data": "data role",
          "role.service": "service role",
          "role.creative": "creative role",
          "role.engineering": "engineering role",
          "role.accounting": "finance role",
          "role.design": "design role",
          "role.web": "SEO role",
          "role.douyin_ads": "Douyin ads role",
          "chat.answer": "chat answer",
          "chat.clarification": "clarification",
          "knowledge.explanation": "knowledge explanation",
          "intent.classification": "intent classification",
          "ops.strategy": "ops strategy",
          "campaign.orchestration": "campaign orchestration",
          "data.analysis": "data analysis",
          "creative.content": "creative content",
          "service.workflow": "service workflow",
          "metrics.diagnosis": "metrics diagnosis",
          "forecast.demand": "demand forecast",
          "attribution.modeling": "attribution modeling",
          "service.sentiment": "service sentiment",
          "nps.analysis": "NPS analysis",
          "refund.process": "refund process",
          "creative.video": "creative video",
          "creative.copywriting": "creative copywriting",
          "seo.title": "SEO title",
          "engineering.architecture": "technical architecture",
          "engineering.reliability": "system reliability",
          "engineering.sla": "SLA",
          "data.pipeline": "data pipeline",
          "finance.profitability": "profitability",
          "finance.cashflow": "cash flow",
          "budget.planning": "budget planning",
          "roi.modeling": "ROI modeling",
          "design.visual": "visual design",
          "design.layout": "layout design",
          "conversion.optimization": "conversion optimization",
          "seo.optimization": "SEO optimization",
          "web.content": "web content",
          "ads.optimization": "ads optimization",
          "roi.control": "ROI control",
          "audience.segmentation": "audience segmentation",
          "campaign.pacing": "campaign pacing",
          "action.answer": "answer",
          "action.analyze": "analyze",
          "action.create": "create",
          "action.optimize": "optimize",
          "action.plan": "plan",
          "action.execute": "execute",
          "action.query": "query",
        },
        strategyNotes: {
          learning:
            "Teaching mode: keep the substance while explaining terms, reasoning, and next steps.",
          quick:
            "Best for fast judgment: conclusion, key reason, and only a few next actions.",
          execution:
            "Best for reports, reviews, and plans: use tables and checklists for agent split, steps, risks, and acceptance checks.",
        },
      },
      samples: {
        coffee: {
          label: "May Day coffee conversion",
          prompt:
            "I run coffee operations and mainly market on Douyin. May Day is coming soon. Please write an operations plan to improve conversion rate, including Qianchuan ads, budget, creative, and execution plan.",
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
      title: "Task process",
      expand: "Expand agent process",
      collapse: "Collapse agent process",
      close: "Close task process",
      closePanel: "Collapse process panel",
      progress: "Progress",
      activity: "Run activity",
      record: "Run records",
      recordCountSuffix: "items",
      businessInput: "Business input",
      preview: "Work preview",
      turns: "Attempts",
      handoff: "Business handoff",
      futureModules: "Connects to real modules later",
      idle: {
        planningTitle: "Planning execution graph",
        historyTitle: "Historical chat, waiting for a new task",
        waitingTitle: "Waiting for business goal",
        planningDescription:
          "Input received. Matching products, campaigns, data, and delivery steps.",
        historyDescription:
          "The task process only shows the current run; historical messages are not counted in progress.",
        waitingDescription:
          "After you enter an ecommerce question, it will organize products, campaigns, platform data, alerts, and deliverables.",
        loopTitle: "01 | Operating loop",
        previewTitle: "Run preview",
        previewFile: "Operating loop preview",
        pending: "Pending",
        lines: [
          "Data diagnosis identifies metrics, attribution, and operating constraints",
          "Product board provides product knowledge, selling points, and competitor clues",
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
      agentName: "AI training assistant",
      welcomeCapabilities: [
        "Ask or add materials",
        "System chooses the flow",
        "Follow up into deliverables",
      ],
      startHere: "Start from a training task",
      examplePrompts: [
        "I am using this for the first time. Tell me how to start an ecommerce training task",
        "Judge whether a product is suitable for training product selection and what materials I should add",
        "Create a campaign plan, execution checklist, and review metrics for a 3000 budget",
      ],
      studentFlow: {
        title: "Follow this path on first use",
        subtitle:
          "State the task, watch how the system dispatches it, then turn the result into training material.",
        ask: {
          title: "1. State the task",
          detail:
            "Start with the goal. If unsure, mention the product, budget, platform, and assignment you need.",
        },
        watch: {
          title: "2. Watch the process",
          detail:
            "Every answer shows a dispatch process. Simple tasks are short; complex work expands into roles and Skills.",
        },
        deliver: {
          title: "3. Use the result",
          detail:
            "Continue with more materials, or turn the answer into task steps, a campaign plan, or a review report.",
        },
      },
      processPromise:
        "The process panel reflects real backend dispatch: the dispatcher reads this question and same-conversation history, then chooses quick answer, single role, or multi-agent collaboration.",
      studentPromptRecipe: {
        title: "When unsure how to describe it, use these four parts",
        subtitle:
          "Students do not need to learn the menus first. State the task clearly, and the dispatcher chooses the right flow.",
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
        title: "If unsure what to ask, choose a task type",
        subtitle:
          "These starters fill the input directly, so students do not need to understand every feature menu first.",
        start: {
          title: "I am starting",
          detail:
            "Ask the system what materials to prepare and how to phrase the first turn clearly.",
          prompt:
            "I am using this for the first time. Guide me through an ecommerce training task and tell me what materials to prepare",
        },
        analyze: {
          title: "Judge product or data",
          detail:
            "Use this for product fit, budget, funnel, and risk analysis.",
          prompt:
            "Judge whether this product or data is suitable for the training task, and list the key missing materials",
        },
        plan: {
          title: "Create an execution plan",
          detail:
            "Use this for campaigns, listing execution, scripts, service handoff, and review metrics.",
          prompt:
            "Break my goal into a campaign plan, execution checklist, role split, and review metrics",
        },
        review: {
          title: "Prepare report or review",
          detail:
            "Use this to turn an answer into coursework, a training report, review material, or next steps.",
          prompt:
            "Turn the conclusion above into a training review report that a student can submit",
        },
      },
      receivingOutput: "Receiving multi-agent output",
      waitingEvents: "Waiting for agent process events",
      stopped:
        "This answer was stopped, and the backend model call did not continue.",
      completedTasks: "{{completed}}/{{total}} steps completed",
      planning: "Planning agent collaboration",
      historyNotExecuted: "Historical message has no live process",
      executionTitle: "Multi-agent collaboration process",
      processKinds: {
        waiting: {
          title: "Waiting for backend dispatch",
          badge: "Syncing",
          progress: "Dispatch events syncing",
          summary:
            "This turn has started. Waiting for the backend to return the real route, context, and step events.",
        },
        quick_dispatch: {
          title: "Quick dispatch process",
          badge: "Light flow",
          progress: "{{completed}}/{{total}} dispatch steps",
          summary:
            "Dispatcher chose {{role}} for this turn: {{task}}",
          stageRunning: "Reading context and organizing a quick answer",
          stageFinished: "Quick dispatch finished. Inspect the real decision chain.",
          stageSynced: "Quick dispatch chain synced for this turn",
        },
        single_agent: {
          title: "Single-agent process",
          badge: "Single role",
          progress: "{{completed}}/{{total}} roles completed",
          summary: "Dispatcher selected {{role}} as lead: {{task}}",
          stageRunning: "Lead agent is processing this task",
          stageFinished: "Lead agent finished. Inspect the processing chain.",
          stageSynced: "Single-agent processing chain synced",
        },
        multi_agent: {
          title: "Multi-agent collaboration process",
          badge: "Role collaboration",
          progress: "{{completed}}/{{total}} roles completed",
          summary: "Dispatcher split this into role collaboration: {{task}}",
          stageRunning: "Agents are collaborating",
          stageFinished: "Multi-agent output summarized. Review process and business links.",
          stageSynced: "Expand to review this agent process",
        },
      },
      statuses: {
        executing: "Agents collaborating",
        synced: "",
        planning: "Assigning agents",
        waiting: "Waiting for a new collaboration",
      },
      artifactsTitle: "Stage files",
      artifacts: {
        updating: "Updates with execution",
        preparing: "Preparing",
        pending: "Pending",
        backendCreated: "System created {{type}}, status: {{status}}",
        empty:
          "No review materials yet. Demo files are not shown for this run.",
        searchEvidence: "System saved real external search evidence.",
        waitingBackend: "Waiting for review materials or search evidence.",
      },
      currentProgress: "Agent progress {{completed}}/{{total}}",
      collapseProcess: "Collapse agent process",
      expandProcess: "Expand agent process",
      openWorkbench: "View task process",
      openWorkbenchForTurn: "View task process",
      closeWorkbench: "Hide task process",
      currentAnswerProcess: "Current task process",
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
        autoRoles: "Agents: assigned automatically",
        roles: "Agents: lead {{role}} / {{count}} total",
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
        planReady: "Execution plan generated",
        readingRules: "Reading ecommerce question and business rules",
        targetRules: "Ecommerce question and business rules",
        assignTask: "Assign agents",
        dispatchJudge: "Judge task difficulty",
        dispatchRoute: "{{mode}}, lead {{role}}",
        dispatchWaiting: "Waiting for dispatch result",
        runtimeAction: "Run required actions",
        answerReady: "Compose answer",
        answerFinished: "Answer and process completed",
        answerStreaming: "Generating answer",
        answerSynced: "Answer generated",
        workspaceContext: "Read {{readable}}/{{total}} usable materials",
        contextLoaded: "{{products}} products, {{knowledge}} knowledge records",
        memoryLoaded: "Loaded {{count}} runtime memories",
        skillRequested: "{{count}} Skill request(s), latest {{skill}}",
        skillResult: "{{count}} Skill result(s), latest {{skill}} / {{status}}",
        modelRunning: "Model is organizing output",
        noExtraSkill: "No extra Skill needed for this turn",
        singleAgentRoute: "{{role}} handles this as one role",
        taskRunning: "Task is running",
        waitingSync: "Waiting for execution status sync",
        checkingConstraints: "Checking model, budget, and ads constraints",
        enrichMaterial: "Retrieve sources and evidence",
        enrichByTask: "Enrich platform materials by task",
        enrichFallback: "Enrich industry and platform context",
        directAnswer: "Simple questions answer directly; complex tasks start multi-agent collaboration",
        callModel: "Call agent runtime",
        singleAgentAnswer: "Lead agent quickly organizes the answer",
        savedToReport: "Save the answer as reviewable material",
        graphNodeReady: "System generated this agent step for the current task",
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
          detail: "Product board, platform connections, insights",
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
          "Collaboration blocked. Expand agent process to inspect action items.",
        finished: "Multi-agent output summarized. Review process and business links.",
        saved: "Multi-agent output summarized. Review process and business links.",
        running: "Agents are collaborating",
        planning: "Planning agent collaboration",
        synced: "Expand to review this agent process",
        history: "Historical chat. Expand to review this agent process.",
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
          label: "Backend dispatch plan",
          detail: "{{difficulty}} difficulty, route {{mode}}",
        },
        reasoning: {
          label: "Plan reason",
        },
        plannedSkills: {
          label: "Planned Skills",
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
          label: "Dispatch decision",
          detail: "{{mode}}, lead {{role}}, strategy {{strategy}}",
        },
        skill: {
          label: "Skill calls",
          detail: "{{count}} real Skill events in this turn",
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
        title: "Real dispatch chain for this turn",
        workbenchTitle: "Task process",
        steps: "Processing steps",
        routeType: "Route",
        studentReadTitle: "Why this route was used",
        emptyTask: "Waiting for this turn's task",
        waitingStepTitle: "Waiting for dispatch events",
        waitingStepDetail: "The route, context, and steps will appear after backend sync.",
        compactedStepDetail:
          "The system read this question, same-conversation history, and available materials, then organized the answer through the selected route. Internal context is not expanded in the student view.",
        routeModes: {
          chat_bypass: "quick answer route",
          task_graph: "multi-step task route",
          single_agent: "single-role route",
        },
        previewFiles: {
          quick_dispatch: "quick.dispatch.process.md",
          single_agent: "single.agent.process.md",
          multi_agent: "multi.agent.process.md",
        },
        preview: {
          heading: "# {{mode}}",
          request: "Business input: {{request}}",
          summary: "Dispatch note: {{summary}}",
          progress: "Progress: {{completed}}/{{total}} | {{stage}}",
          step: "{{index}} {{title}}: {{detail}}",
          fact: "{{label}}: {{detail}}",
        },
        stableDetail:
          "Simple or complex, every question is planned first; only the number of steps, roles, and Skills changes.",
        studentUseTitle: "How students should read it",
        studentUseDetail:
          "Check the route and context budget first to see whether history, products, and materials were read. Then inspect steps and Skills to tell whether this was a quick answer, one-role analysis, or multi-agent collaboration.",
        difficultyTitle: "Why some flows are short and others are long",
        difficultyDetail:
          "The dispatcher judges difficulty from this question, selected mode, and same-conversation history. Simple questions usually only read context and answer quickly; complex tasks add roles, Skills, and synthesis steps.",
        realnessTitle: "Why a full role row is not shown",
        realnessDetail:
          "The backend did not plan a multi-agent task graph for this turn, so only real route/context/skill/output events are shown. Complex tasks show multi-role collaboration.",
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
            "Product board, platform connections, and insights become task input",
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
        planning: "Planning execution graph",
        historyWaiting: "Historical chat, waiting for a new task",
        failedAgent: "Execution blocked: {{agent}}",
        reworkingAgent: "Retrying: {{agent}}",
        workingAgent: "Executing: {{agent}}",
        multiTasking: "Multi-task processing",
        finished: "Plan summarized",
      },
      timeline: {
        planning: [
          "Task received. Planning the execution graph.",
          "Steps, dependencies, and acceptance rules will sync to the workspace.",
        ],
        history: [
          "This is a historical conversation and the current run has not started.",
          "Send a new task to rebuild the processing steps.",
        ],
        empty: [
          "Waiting for customer task input, then the steps will be planned.",
          "Data diagnosis runs first; budget, ads, and creative tasks run in parallel.",
          "The ops plan cross-checks and summarizes final delivery.",
        ],
        events: {
          commanderThinking:
            "Analyzing the task and preparing the execution plan.",
          routeDecided:
            "System identified the task type, applied {{strategy}} strategy, collaboration mode {{mode}}, lead agent {{role}}.",
          graphPlanned: "Planned {{count}} agent processing steps.",
          nodeStarted: "{{agent}} started attempt {{attempt}}.",
          nodeRework: "{{agent}} entered retry: {{instruction}}",
          nodePassed: "{{agent}} passed acceptance.",
          nodeBoundary:
            "{{agent}} completed category-level boundary output without blocking the final answer.",
          nodeFailed: "{{agent}} failed: {{reason}}",
          skillRequested: "{{role}} called capability {{skill}}.",
          skillResult: "Capability {{skill}} returned status: {{status}}.",
          approvalRequired: "Approval created: {{skill}}, risk {{risk}}.",
          workerStarted: "{{agent}} started processing.",
            workerFinished: "{{agent}} finished processing.",
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
          agentLearningRecordCreated: "Training memory saved: {{category}}.",
          qualityChecked: "Quality check complete: {{score}}, {{status}}.",
          qualityPassed: "passed",
          qualityNeedsWork: "needs work",
          workspaceRunFailed: "Training submission marked failed.",
          workspacePersistenceError:
            "Training record persistence failed: {{action}}.",
          finished: "This answer finished.",
          failed: "This answer failed: {{reason}}",
          graphFinished: "Execution graph finished and summary output started.",
        },
      },
      cards: {
        readTarget: {
          title: "Read target",
          planMeta: "Execution plan",
          userMeta: "User task",
          plannedDescription: "Split the task into {{count}} processing steps.",
          inputDescription:
            "Reading the user goal and preparing the execution plan.",
          waitingLine: "Waiting for customer business goal.",
        },
        assign: {
          title: "Assign tasks",
          meta: "Waiting assignment",
          description:
            "Preparing processing steps, dependencies, and acceptance rules.",
          currentTask: "Current task: {{agent}}",
          stateLine: "Execution state: {{state}}",
          latestAction: "Latest action: {{action}}",
          latestPlanning: "Latest action: breaking down task",
        },
        output: {
          reportTitle: "Task output",
          materialTitle: "Supplemental context",
          materialMeta: "Industry and platform context",
          description:
            "After task output starts, this switches to real report summary, evidence, and risks.",
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
          title: "Capability call",
          statusLine: "Status: {{status}}",
          requested:
            "The system received a capability request and is executing it or waiting for external data.",
          completed: "The capability call returned a result.",
          deferred:
            "This capability needs external data and has been explicitly deferred.",
          approvalRequired:
            "This capability is high-risk or writes to a platform, so approval is required before execution.",
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
          line: "Handled as a category-level boundary: keep missing data, risks, and next steps without showing it as execution failure.",
        },
        failed: {
          title: "Execution blocked",
          description:
            "The current node failed. Check model service or upstream dependencies.",
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
        responsibilityLine: "Responsibility: {{summary}}",
        objectiveLine: "Task objective: {{request}}",
        reviewLine: "Review note: {{reason}}",
        reportMode: "Report",
        taskMode: "Task",
        deliveryFilename: "operation.execution.plan.docx",
      },
      runtimeDescriptions: {
        started: "{{agent}} started attempt {{attempt}}.",
        reported:
          "{{agent}} submitted a report and is waiting for acceptance or summary.",
        passed: "{{agent}} passed acceptance.",
        boundary:
          "{{agent}} produced a category-level boundary for the primary role to summarize.",
        failed: "{{agent}} failed.",
        reworking: "{{agent}} is reorganizing output from feedback.",
        skillRequested: "Requested capability {{skill}}.",
        skillResult: "Capability {{skill}} returned {{status}}.",
      },
      runtimeActions: {
        commanderThinking: "Breaking down task",
        graphPlanned: "Execution plan generated",
        nodeReady: "Task queued",
        nodeStarted: "Task started",
        nodeReported: "Task reported",
        nodeEvaluated: "Result reviewed",
        nodeRework: "Rework triggered",
        nodePassed: "Accepted",
        nodeBoundary: "Needs data",
        nodeFailed: "Execution blocked",
        skillRequested: "Capability requested",
        skillResult: "Capability returned",
        graphFinished: "Execution plan finished",
        synced: "State synced",
      },
      reportLines: {
        risk: "Risk: {{risk}}",
        question: "Open question: {{question}}",
      },
      fallbackResponse: {
        failed:
          "This task failed, usually because the model service, tools, or upstream dependencies were unavailable.",
        boundary:
          "This agent was limited by no selected product, missing live metrics, or unavailable external data, so it produced a stage conclusion and missing-data list.",
        reworking:
          "Acceptance failed. The task output is being reorganized from feedback.",
        working:
          "{{agent}} is processing the request. Results will update here.",
        done: "{{agent}} is complete and waiting for final summary.",
        idle: "After the task starts, this area shows the request, processing result, and review material summary.",
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
        "This page gathers plans, checklists, and reports generated by the AI training assistant so students can inspect, copy, download, and submit them.",
      refresh: "Refresh",
      loading: "Loading review materials",
      listTitle: "Review material list",
      listDesc: "{{artifacts}} material(s) / {{workspaces}} training task(s)",
      emptyTitle: "No review materials yet",
      emptyDescription:
        "Return to the AI training assistant and send a training question. Generated plans or reports will appear here.",
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
      title: "AI training capability guide",
      description:
        "This page explains how students use AI Q&A, product analysis, campaign planning, data diagnosis, and review reports around one ecommerce training task.",
      action: "Open AI training assistant",
    },
    principle: {
      title: "How students use it",
      description:
        "Students do more than open a chat box. They ask around a training task, choose product and campaign context, then store AI output as tasks, review records, and report materials.",
    },
    capabilities: {
      title: "Core training capabilities",
      description:
        "These entries reuse existing real capabilities while the frontend organizes them into a student-facing learning and task flow.",
      items: [
        {
          title: "Product materials",
          description:
            "Uses products, materials, selling points, and competitors manually selected by the student. Without selection, the flow stays as ordinary training Q&A.",
        },
        {
          title: "Selection and competitor research",
          description:
            "Analyzes categories, price bands, consumer demand, platform trends, and competitor actions.",
        },
        {
          title: "External search and real-time data",
          description:
            "Returns sourced data. When an integration is not configured, the UI states it clearly instead of fabricating results.",
        },
        {
          title: "Campaign plans",
          description:
            "Generates live scripts, short-video ideas, campaign rhythm, ads review, and operations plans for planning and execution.",
        },
        {
          title: "Data diagnosis and service suggestions",
          description:
            "Organizes replies, attribution, and follow-up actions from product materials, service rules, and operating data.",
        },
        {
          title: "Tasks and submissions",
          description:
            "Stores AI output as tasks, runs, execution suggestions, and traceable training deliverable states.",
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
    flow: {
      title: "How one training task flows",
      description:
        "The student-visible main flow should stay clear, continuous, and traceable.",
      items: [
        "The student creates a new conversation or opens a historical one, then enters a training question.",
        "The student may choose answer mode, collaborating agents, and manually selected product materials.",
        "The system plans the task, calls models and analysis capabilities, and shows the process in real time.",
        "Process cards appear above the answer and collapse after completion, so they do not block reading.",
        "Final answers and deliverables are stored in training tasks, review reports, teacher review, or operating results pages.",
      ],
    },
    lanes: {
      title: "Training task areas in navigation",
      description:
        "The left navigation is not just a page menu. It exposes the most-used capability entries for students completing training tasks.",
      items: [
        {
          title: "AI training assistant",
          description:
            "The main entry for training questions, strategy, product references, and process display.",
        },
        {
          title: "Product materials",
          description:
            "Manages products, materials, knowledge, competitors, and manually selected task context.",
        },
        {
          title: "Campaigns, data, and results",
          description:
            "Connects platforms, campaigns, metrics, briefings, alerts, and external data.",
        },
        {
          title: "Tasks, review, and reports",
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
        "First ask the multi-agent training assistant. It detects your task and shows the analysis process.",
      desc2:
        "Then move into product materials, campaign planning, training tasks, and review reports as needed.",
      desc3:
        "After signing in, you can save conversations, reference product materials, submit tasks, and review teacher feedback.",
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
        "Use one account to keep training conversations, product materials, campaign tasks, and review reports.",
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
  },
} as const

export default messages
