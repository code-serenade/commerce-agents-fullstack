export interface AgentOutputMetric {
  label: string
  value: string
  description: string
}

export interface AgentOutputStep {
  title: string
  owner: string
  timing: string
  detail: string
}

export type AgentOutputAction = string

export interface AgentOutputSection {
  id: string
  title: string
  eyebrow: string
  content: string
  bullets: string[]
}
