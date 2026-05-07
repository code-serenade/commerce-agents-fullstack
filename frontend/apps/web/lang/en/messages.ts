const messages = {
  web: {
    shell: {
      brand: {
        eyebrow: "Commerce AI",
        title: "Ops Center",
        description: "Commerce agent workspace",
      },
      navigation: {
        mobileLabel: "Primary navigation",
        workspace: "Workspace",
        home: "Home",
        agent: "Ops Agent",
        agentOutput: "Plan view",
        profile: "Profile",
      },
      topbar: {
        meta: "Personal workspace",
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
  home: {
    welcome: "Welcome back",
    publicWelcome: "Welcome",
    publicTitle: "Web workspace",
    publicSubtitle: "Public app pages use the same layout before and after sign-in.",
    fallbackName: "Member",
    actions: {
      profile: "Edit profile",
      login: "Sign in",
    },
    account: {
      title: "Account ready",
      description:
        "Your signed-in workspace is active. Private app data stays behind authentication.",
    },
    publicAccount: {
      title: "Public access",
      description:
        "You can browse shared pages here. Private account data is only available after sign-in.",
    },
    profile: {
      title: "Profile settings",
      description:
        "Update your display name, avatar, email, and password from your profile page.",
    },
    publicProfile: {
      title: "Personal features",
      description:
        "Sign in to manage your profile, avatar, email, and password.",
    },
  },
  agentChat: {
    welcomeMessage:
      "Hi, I am the Commerce Ops Agent. Send an ecommerce operations goal and I will split it across data, budget, Douyin ads, creative, and operations roles, with the task graph, progress, activity trail, and deliverables on the right.",
    streaming: "Thinking...",
    error: "Agent reply failed. Please try again later.",
    errorBadge: "Reply failed",
    missingStructuredOutput:
      "The previous run did not produce complete structured output. Send the task again and I will rebuild the execution plan and sync the process to the workspace.",
    sidebar: {
      newChat: "New task",
      search: "Search history",
      recents: "Recents",
      chat: "Ops Agent",
      more: "More",
      empty: "No conversations yet",
    },
    shell: {
      brandTitle: "Ops Center",
      brandDescription: "Commerce agent workspace",
      topbarTitle: "Enter an operations goal",
      topbarMeta: "Commerce Ops Agent · operating loop",
      sections: {
        create: "",
        workspace: "Ops Center",
        governance: "Management and Review",
        recents: "History",
      },
      navigation: {
        newChat: "New chat",
        smartOps: "Smart Ops",
        productBoard: "Product board",
        campaigns: "Campaigns",
        platform: "Platform connections",
        marketResearch: "Market insights",
        briefing: "Daily briefing",
        alerts: "Alerts",
        deliverables: "Deliverables",
        taskBoard: "Task board",
        packages: "Reusable packages",
        systemDashboard: "System dashboard",
        team: "AI team",
        history: "History",
      },
    },
    composer: {
      placeholder:
        "Enter an operations goal, such as a May Day coffee conversion plan, Qianchuan ads review, campaign budget, or creative breakdown...",
      loginPlaceholder: "Sign in before calling the agent",
      hint: "Enter to send, Shift + Enter for a new line",
      send: "Send",
      stop: "Stop",
      attachTitle: "Attach file",
      agentTitle: "Switch ops agent",
      agentName: "Commerce Ops Agent",
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
          title: "Later connects to product board and product knowledge.",
        },
        campaign: {
          label: "May Day campaign",
          title: "Later connects to campaign planning and execution tracking.",
        },
        platform: {
          label: "Douyin/Qianchuan data",
          title: "Later connects to platform integrations and imported ads data.",
        },
      },
    },
    workbench: {
      title: "Ops workspace",
      expand: "Expand ops workspace",
      collapse: "Collapse ops workspace",
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
          "The workspace only shows the current run; historical messages are not counted in progress.",
        waitingDescription:
          "After you enter an operations goal, it will organize products, campaigns, platform data, alerts, and deliverables.",
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
          "Reusable packages support follow-up start, stop, and rollback flows",
        ],
      },
    },
    thread: {
      agentName: "Commerce Ops Agent",
      welcomeCapabilities: ["Break down goals", "Organize execution", "Store deliverables"],
      receivingOutput: "Receiving business execution output",
      waitingEvents: "Waiting for workspace events",
      completedTasks: "{{completed}}/{{total}} tasks completed",
      planning: "Generating execution plan",
      historyNotExecuted: "Historical message not executed",
      executionTitle: "Business execution",
      statuses: {
        executing: "Executing",
        synced: "Run synced",
        planning: "Planning",
        waiting: "Waiting for a new run",
      },
      artifactsTitle: "Stage files",
      artifacts: {
        updating: "Updates with execution",
        preparing: "Preparing",
        pending: "Pending",
      },
      currentProgress: "Current progress {{completed}}/{{total}}",
      actions: {
        copy: "Copy",
        useful: "Useful",
        useless: "Not useful",
      },
      agents: {
        standby: "Standby",
        data: {
          name: "Data diagnosis",
          summary: "Identify metrics, attribution, and operating constraints first",
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
          summary: "Generate short-video, live, and campaign content directions",
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
        readTarget: "Read target",
        planReady: "Execution plan generated",
        readingRules: "Reading operating goal and business rules",
        targetRules: "Operating goal and business rules",
        assignTask: "Assign task",
        taskRunning: "Task is running",
        waitingSync: "Waiting for execution status sync",
        checkingConstraints: "Checking model, budget, and ads constraints",
        enrichMaterial: "Enrich materials",
        enrichByTask: "Enrich platform materials by task",
        enrichFallback: "Enrich industry and platform context",
      },
      artifactSpecs: {
        dataStatus: "Data diagnosis, metric attribution, platform context",
        adsStatus: "Qianchuan plan structure, audience packages, budget cadence",
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
          detail: "Campaigns, task board, workspace tasks",
        },
        delivery: {
          label: "Delivery",
          detail: "Deliverables, daily briefing, review history",
        },
        governance: {
          label: "Governance",
          detail: "Alerts, packages, approval rollback",
        },
      },
      processStage: {
        failed: "Execution blocked. Expand workspace to inspect action items.",
        finished: "Plan summarized. Review process and business links.",
        running: "Execution plan is progressing",
        planning: "Planning execution graph",
        synced: "Run process synced",
        history: "Historical chat. Expand workspace to review this run.",
      },
      artifactState: {
        done: "Generated",
        working: "Generating",
        failed: "Blocked",
        queued: "Queued",
        idle: "Pending",
      },
      inlineState: {
        done: "Done",
        working: "Processing",
        reworking: "Retrying",
        failed: "Failed",
        queued: "Queued",
        idle: "Standby",
      },
    },
    swarm: {
      lanes: {
        context: { label: "Context", value: "Products/platform/insights" },
        execution: { label: "Execution", value: "Campaigns/tasks/budget" },
        delivery: { label: "Delivery", value: "Files/briefing/review" },
        governance: { label: "Governance", value: "Alerts/approval/rollback" },
      },
      agents: {
        data: {
          name: "Data diagnosis",
          summary: "Analyze data, metrics, attribution, and business constraints",
          objective: "Analyze the task data, metrics, attribution, and constraints",
        },
        accounting: {
          name: "Budget review",
          summary: "Review cost, budget, SKU, and operating risk",
          objective: "Review campaign cost, traffic SKU, ads budget, and risk from the data diagnosis",
        },
        douyinAds: {
          name: "Qianchuan optimization",
          summary: "Plan structure, audiences, budget cadence, and ROI stop-loss",
          objective:
            "Break down Qianchuan plan structure, audience packages, bids, budget, ROI stop-loss, and live or short-video handoff",
        },
        creative: {
          name: "Creative output",
          summary: "Produce short-video, live, or campaign creative ideas",
          objective: "Create Douyin short-video, live, or campaign content ideas from the data diagnosis",
        },
        ops: {
          name: "Ops plan",
          summary: "Combine finance, Qianchuan, and creative output into an executable plan",
          objective: "Combine finance, Qianchuan, and creative output into an executable operations plan",
        },
        delivery: {
          name: "Deliverable doc",
          summary: "Store plan structure, checklist, and downloadable artifacts",
          objective: "Turn the operations plan into a reusable customer document",
        },
      },
      phases: {
        data: {
          title: "Phase 1: Data diagnosis",
          detail: "Identify metrics, attribution, platform context, and known constraints",
        },
        parallel: {
          title: "Phase 2: Parallel work",
          detail: "Finance, Qianchuan ads, and creative tasks run in parallel",
        },
        ops: {
          title: "Phase 3: Cross-check",
          detail: "Merge upstream conclusions and cross-check budget, creative, and launch cadence",
        },
        delivery: {
          title: "Phase 4: Delivery summary",
          detail: "Compose final strategy, checklist, and deliverable files",
        },
        dynamicTitle: "Phase {{index}}: {{title}}",
        dynamicDeliveryTitle: "Phase {{index}}: Delivery summary",
        dynamicDeliveryDetail: "Summarize task results into a deliverable plan, checklist, and document.",
      },
      handoffs: {
        context: {
          title: "Context",
          detail: "Product board, platform connections, and insights become task input",
        },
        execution: {
          title: "Execution",
          detail: "Campaigns, task board, and workspace tasks carry the actions",
        },
        delivery: {
          title: "Delivery",
          detail: "Deliverables, daily briefing, and review history store results",
        },
        governance: {
          title: "Governance",
          detail: "Alerts, package management, and approval rollback handle risk",
        },
        review: {
          title: "Review",
          detail: "Chat history, execution records, and team capability notes stay reusable",
        },
      },
      statePill: {
        done: "Done",
        working: "Working",
        queued: "Queued",
        failed: "Failed",
        reworking: "Retry",
        skipped: "Skipped",
        idle: "Standby",
      },
      dockState: {
        done: "Done",
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
          commanderThinking: "Analyzing the task and preparing the execution plan.",
          graphPlanned: "Planned {{count}} processing steps.",
          nodeStarted: "{{agent}} started attempt {{attempt}}.",
          nodeRework: "{{agent}} entered retry: {{instruction}}",
          nodePassed: "{{agent}} passed acceptance.",
          nodeFailed: "{{agent}} failed: {{reason}}",
          graphFinished: "Execution graph finished and summary output started.",
        },
      },
      cards: {
        readTarget: {
          title: "Read target",
          planMeta: "Execution plan",
          userMeta: "User task",
          plannedDescription: "Split the task into {{count}} processing steps.",
          inputDescription: "Reading the user goal and preparing the execution plan.",
          waitingLine: "Waiting for customer business goal.",
        },
        assign: {
          title: "Assign tasks",
          meta: "Waiting assignment",
          description: "Preparing processing steps, dependencies, and acceptance rules.",
          currentTask: "Current task: {{agent}}",
          stateLine: "Execution state: {{state}}",
          latestAction: "Latest action: {{action}}",
          latestPlanning: "Latest action: breaking down task",
        },
        output: {
          reportTitle: "Task output",
          materialTitle: "Supplemental context",
          materialMeta: "Industry and platform context",
          description: "After task output starts, this switches to real report summary, evidence, and risks.",
          fallbackLines: ["Coffee conversion", "Qianchuan ads review", "Ops budget and launch cadence"],
        },
        rework: {
          title: "Recheck",
          description: "Acceptance failed and the system triggered a rework instruction.",
        },
        failed: {
          title: "Execution blocked",
          description: "The current node failed. Check model service or upstream dependencies.",
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
        reported: "{{agent}} submitted a report and is waiting for acceptance or summary.",
        passed: "{{agent}} passed acceptance.",
        failed: "{{agent}} failed.",
        reworking: "{{agent}} is reorganizing output from feedback.",
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
        nodeFailed: "Execution blocked",
        graphFinished: "Execution plan finished",
        synced: "State synced",
      },
      reportLines: {
        risk: "Risk: {{risk}}",
        question: "Open question: {{question}}",
      },
      fallbackResponse: {
        failed: "This task failed, usually because the model service, tools, or upstream dependencies were unavailable.",
        reworking: "Acceptance failed. The task output is being reorganized from feedback.",
        working: "{{agent}} is processing the request. Results will update here.",
        done: "{{agent}} is complete and waiting for final summary.",
        idle: "After the task starts, this area shows the request, processing result, and artifact summary.",
      },
    },
  },
  agentOutput: {
    prompt:
      "I run coffee operations. My main marketing platform is Douyin, and May Day is coming soon. Can you help me write an operations plan to improve conversion rate?",
    sectionsNavigation: "Reading guide",
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
      hook:
        "Use the first three seconds for a scene hook instead of the brand, such as a travel coffee price comparison.",
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
  login: {
    brand: "AI STANDARDIZED FRAMEWORK",
    hero: {
      titleLine1: "Describe your needs,",
      titleLine2: "let AI code by the framework.",
      desc1:
        "This is an AI-standardized development framework across pages, state, routes, APIs, and themes.",
      desc2:
        "You provide requirements, and AI implements features while following the project conventions.",
    },
    welcome: "WELCOME BACK",
    title: "Sign in to your account",
    subtitle:
      "Use your identifier and password to continue into the AI development workflow.",
    form: {
      password: {
        label: "Password",
        placeholder: "Enter password",
      },
      submit: "Sign in",
      submitting: "Signing in...",
      identifier: {
        label: "Identifier",
        placeholder: "Enter identifier",
      },
    },
    footer: {
      toRegisterPrefix: "Don't have an account?",
      toRegisterAction: "Register",
    },
    error: {
      invalidCredentials:
        "Login failed, please check your identifier and password.",
    },
  },
  register: {
    brand: "AI STANDARDIZED FRAMEWORK",
    hero: {
      titleLine1: "Create your account,",
      titleLine2: "start AI development.",
      desc1:
        "Register once for the shared account system, then join this app when you sign in.",
      desc2:
        "Theme mode, i18n, and modular structure are ready for scalable iteration.",
    },
    welcome: "CREATE ACCOUNT",
    title: "Register your account",
    subtitle: "Create the shared account. This app will initialize its member record when you sign in.",
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
