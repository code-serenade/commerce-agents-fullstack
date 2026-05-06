const messages = {
  web: {
    shell: {
      brand: {
        eyebrow: "Web App",
        title: "Web",
        description: "Member workspace",
      },
      navigation: {
        mobileLabel: "Primary navigation",
        workspace: "Workspace",
        home: "Home",
        agent: "Agent",
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
      "Hi, I am the project agent. Send a question and I will stream the reply back here.",
    streaming: "Thinking...",
    error: "Agent reply failed. Please try again later.",
    errorBadge: "Reply failed",
    sidebar: {
      newChat: "New chat",
      search: "Search history",
      recents: "Recents",
      chat: "Chat",
      more: "More",
      empty: "No conversations yet",
    },
    composer: {
      placeholder: "Message the agent...",
      loginPlaceholder: "Sign in before calling the agent",
      hint: "Enter to send, Shift + Enter for a new line",
      send: "Send",
      stop: "Stop",
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
