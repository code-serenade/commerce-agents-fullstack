import type { ComponentProps } from "react"
import type {
  AgentChatCampaignContextSelection,
  AgentChatProductContextSelection,
} from "../domain/agent-chat-context-selection"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import { AgentChatComposerPanels } from "./agent-chat-composer-panels"
import { AgentChatComposerToolbar } from "./agent-chat-composer-toolbar"

export const product: AgentChatProductContextSelection = {
  brand: "Galaxy",
  category: "Thermos",
  externalId: "sku-1",
  id: "product-1",
  name: "Temperature thermos",
  platform: "Douyin",
  price: "99",
  status: "active",
}

export const campaign: AgentChatCampaignContextSelection = {
  budget: 3000,
  campaignType: "validation",
  id: "campaign-1",
  name: "7-day validation campaign",
  status: "draft",
}

export const runtimeSelection: AgentChatRuntimeSelection = {
  collaborationMode: "auto",
  learningLevel: "higher_vocational",
  primaryRole: "ops",
  routingMode: "commander",
  selectedRoles: [],
}

export const runtimeOptions: AgentRuntimeOptions = {
  collaborationModes: [
    choice("auto", true),
    choice("single", false),
    choice("manual", false),
  ],
  learningLevels: [
    choice("beginner", false),
    choice("higher_vocational", true),
    choice("professional", false),
  ],
  responseModes: [],
  roles: [
    role("ops", true),
    role("data", false),
    role("creative", false),
  ],
  routingModes: [choice("commander", true), choice("fast", false)],
  strategies: [],
}

export function baseToolbarProps(): ComponentProps<
  typeof AgentChatComposerToolbar
> {
  return {
    availableCampaigns: [],
    availableProducts: [],
    availableSources: [],
    campaignContextOpen: false,
    canSend: true,
    isLoadingCampaigns: false,
    isLoadingPlaybooks: false,
    isLoadingProducts: false,
    isLoadingRuntimeOptions: false,
    isLoggedIn: true,
    isStreaming: false,
    onClearCampaignContext: noop,
    onClearProductContext: noop,
    onClearSourceContext: noop,
    onRuntimeCollaborationModeChange: noop,
    onRuntimeLearningLevelChange: noop,
    onRuntimePrimaryRoleChange: noop,
    onRuntimeRoutingModeChange: noop,
    onSelectPlaybook: noop,
    onSend: noop,
    onStop: noop,
    onStrategyChange: noop,
    onToggleCampaignContext: noop,
    onToggleCampaignPanel: noopBoolean,
    onToggleProductContext: noop,
    onToggleProductPanel: noopBoolean,
    onToggleRuntimeRole: noop,
    onToggleSettingsPanel: noop,
    onToggleSourceContext: noop,
    onToggleSourcePanel: noopBoolean,
    playbooks: [],
    productContextOpen: false,
    runtimeOptions,
    runtimeSelection,
    selectedCampaigns: [],
    selectedProducts: [],
    selectedSources: [],
    settingsOpen: false,
    sourceContextOpen: false,
    strategyId: "learning",
  }
}

export function basePanelProps(): ComponentProps<typeof AgentChatComposerPanels> {
  return {
    availableCampaigns: [],
    availableProducts: [],
    availableSources: [],
    campaignContextOpen: false,
    contextPreview: {
      canPreviewContext: false,
      clearContextPreview: noop,
      isPreviewingContext: false,
      previewContext: asyncNoop,
      previewContextError: null,
      previewContextResult: null,
    },
    contextPreviewOpen: false,
    isLoadingCampaigns: false,
    isLoadingPlaybooks: false,
    isLoadingProducts: false,
    isLoadingRuntimeOptions: false,
    isLoadingSourceLibrary: false,
    isStreaming: false,
    onAttachExistingSource: noop,
    onClearCampaignContext: noop,
    onClearProductContext: noop,
    onClearSourceContext: noop,
    onClosePanels: noop,
    onDeleteSelectedSources: noop,
    onManageSources: noop,
    onRuntimeCollaborationModeChange: noop,
    onRuntimeLearningLevelChange: noop,
    onRuntimePrimaryRoleChange: noop,
    onRuntimeRoutingModeChange: noop,
    onSelectPlaybook: noop,
    onStrategyChange: noop,
    onToggleCampaignContext: noop,
    onToggleProductContext: noop,
    onToggleRuntimeRole: noop,
    onToggleSourceContext: noop,
    onUploadClick: noop,
    panelMaxHeight: 420,
    playbooks: [],
    productContextOpen: false,
    runtimeOptions,
    runtimeSelection,
    selectedCampaigns: [],
    selectedProducts: [],
    selectedSources: [],
    settingsOpen: false,
    sourceContextOpen: false,
    sourceLibrary: [],
    sourceLibraryError: null,
    strategyId: "learning",
  }
}

function choice(id: string, isDefault: boolean) {
  return { description: id, id, isDefault, label: id }
}

function role(runtimeRole: string, isDefault: boolean) {
  return {
    capabilities: ["analysis"],
    id: runtimeRole,
    isDefault,
    name: runtimeRole,
    runtimeRole,
  }
}

function noop() {}

function noopBoolean() {}

async function asyncNoop() {}
