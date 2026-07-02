use std::path::PathBuf;

use crate::{
    agent_utils::prompt::Prompt,
    core::protocol::TaskId,
    roles::{RoleContribution, RoleProfile, RoleSkillCatalog},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolePromptInput {
    pub role: RoleProfile,
    pub task: String,
    pub facts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalSynthesisPromptInput {
    pub task_id: TaskId,
    pub task: String,
    pub support_workers: String,
    pub contributions: Vec<RoleContribution>,
}

pub struct RolePromptBuilder;

impl RolePromptBuilder {
    pub fn build_worker_system_prompt(role: &RoleProfile) -> String {
        Self::render_role_prompt(role)
    }

    pub fn build_worker_prompt(input: &RolePromptInput) -> String {
        Self::render_role_prompt(&input.role)
    }

    pub fn prompt_path_for(role: &RoleProfile) -> PathBuf {
        PathBuf::from("config")
            .join("prompts")
            .join("zh")
            .join("roles")
            .join(format!("{}.md", safe_runtime_role(&role.runtime_role)))
    }

    pub fn final_synthesis_prompt_path() -> PathBuf {
        PathBuf::from("config")
            .join("prompts")
            .join("zh")
            .join("cognition")
            .join("final_synthesis.md")
    }

    pub fn build_final_synthesis_prompt(input: &FinalSynthesisPromptInput) -> String {
        let prompt_path = Self::final_synthesis_prompt_path();
        let prompt = match Prompt::load_from_repo(&prompt_path) {
            Ok(prompt) => prompt,
            Err(err) => {
                return format!(
                    "final_synthesis_prompt_unavailable\npath: {}\nerror: {}",
                    prompt_path.display(),
                    err
                );
            }
        };

        prompt.render(&[
            ("task_id", input.task_id.0.clone()),
            (
                "task",
                if input.task.trim().is_empty() {
                    "none".to_string()
                } else {
                    input.task.clone()
                },
            ),
            ("support_workers", input.support_workers.clone()),
            ("contributions", format_contributions(&input.contributions)),
        ])
    }

    fn render_role_prompt(role: &RoleProfile) -> String {
        let prompt_path = Self::prompt_path_for(role);
        let prompt = match Prompt::load_from_repo(&prompt_path) {
            Ok(prompt) => prompt,
            Err(err) => {
                return format!(
                    "role_prompt_unavailable\npath: {}\nerror: {}",
                    prompt_path.display(),
                    err
                );
            }
        };

        let body = prompt.render(&[
            ("role_name", role.name.clone()),
            ("runtime_role", role.runtime_role.clone()),
            (
                "required_capabilities",
                join_or_none(&role.required_capabilities),
            ),
            (
                "optional_capabilities",
                join_or_none(&role.optional_capabilities),
            ),
            ("preferred_actions", role_actions(role)),
            (
                "available_skills",
                RoleSkillCatalog::staged().format_for_prompt(role),
            ),
        ]);

        format!("{}\n\n{}", role_prompt_header(role), body.trim())
    }
}

fn role_prompt_header(role: &RoleProfile) -> String {
    let capabilities = compact_capabilities(role);
    let actions = role_actions(role);
    let skills = RoleSkillCatalog::staged().format_for_prompt(role);
    format!(
        "角色: {} ({})\n能力: {}\n动作: {}\n技能:\n{}",
        role.name, role.runtime_role, capabilities, actions, skills
    )
}

fn compact_capabilities(role: &RoleProfile) -> String {
    let required = join_or_none(&role.required_capabilities);
    let optional = join_or_none(&role.optional_capabilities);
    if optional == "none" {
        required
    } else {
        format!("{required}; {optional}")
    }
}

fn safe_runtime_role(runtime_role: &str) -> String {
    runtime_role
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .collect()
}

fn role_actions(role: &RoleProfile) -> String {
    let actions = role
        .preferred_actions
        .iter()
        .map(|action| action.as_str().to_string())
        .collect::<Vec<_>>();
    join_or_none(&actions)
}

fn format_contributions(contributions: &[RoleContribution]) -> String {
    if contributions.is_empty() {
        return "none".to_string();
    }

    contributions
        .iter()
        .enumerate()
        .map(|(index, contribution)| {
            let label = if index == 0 {
                "主角色"
            } else {
                "协作角色"
            };
            format!(
                "- {} {}: {}",
                label, contribution.runtime_role, contribution.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{FinalSynthesisPromptInput, RolePromptBuilder, RolePromptInput};
    use crate::{
        core::protocol::{TaskId, WorkerId},
        roles::{RoleCatalog, RoleContribution, RoleId},
    };

    #[test]
    fn builds_prompt_with_role_identity() {
        let role = RoleCatalog::builtin()
            .roles()
            .iter()
            .find(|role| role.runtime_role == "data")
            .expect("data role exists")
            .clone();
        let role_name = role.name.clone();

        let prompt = RolePromptBuilder::build_worker_prompt(&RolePromptInput {
            role,
            task: "分析转化率下降".to_string(),
            facts: vec!["用户提供了转化率指标".to_string()],
        });

        assert!(prompt.contains(&role_name));
        assert!(prompt.contains("角色: 数据分析师 (data)"));
        assert!(!prompt.contains("prompt_source:"));
        assert!(prompt.contains("技能:"));
        assert!(prompt.contains("data_funnel_analysis"));
        assert!(!prompt.contains("分析转化率下降"));
    }

    #[test]
    fn builds_system_prompt_without_task_specific_text() {
        let role = RoleCatalog::builtin()
            .roles()
            .iter()
            .find(|role| role.runtime_role == "creative")
            .expect("creative role exists")
            .clone();

        let prompt = RolePromptBuilder::build_worker_system_prompt(&role);

        assert!(prompt.contains(&role.name));
        assert!(prompt.contains("角色: 内容创意师 (creative)"));
        assert!(!prompt.contains("prompt_source:"));
    }

    #[test]
    fn every_builtin_role_has_prompt_file() {
        for role in RoleCatalog::builtin().roles() {
            let prompt = RolePromptBuilder::build_worker_system_prompt(role);

            assert!(
                !prompt.contains("role_prompt_unavailable"),
                "role {} should have a prompt file, got: {}",
                role.runtime_role,
                prompt
            );
            assert!(
                prompt.contains(&format!("角色: {} ({})", role.name, role.runtime_role)),
                "role {} prompt should include role header",
                role.runtime_role
            );
        }
    }

    #[test]
    fn resolves_prompt_path_from_runtime_role() {
        let role = RoleCatalog::builtin()
            .roles()
            .iter()
            .find(|role| role.runtime_role == "chat")
            .expect("chat role exists")
            .clone();

        let path = RolePromptBuilder::prompt_path_for(&role);

        assert_eq!(
            path.components()
                .map(|component| component.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<_>>(),
            vec!["config", "prompts", "zh", "roles", "chat.md"]
        );
    }

    #[test]
    fn builds_final_synthesis_prompt_from_markdown() {
        let prompt = RolePromptBuilder::build_final_synthesis_prompt(&FinalSynthesisPromptInput {
            task_id: TaskId("task-1".to_string()),
            task: "分析数据并写文案".to_string(),
            support_workers: "worker.creative".to_string(),
            contributions: vec![RoleContribution {
                role_id: RoleId("role.data-analyst".to_string()),
                runtime_role: "data".to_string(),
                worker_id: WorkerId("worker.data".to_string()),
                task_id: TaskId("task-1".to_string()),
                content: "数据贡献".to_string(),
                confidence: None,
                needs_follow_up: false,
            }],
        });

        assert!(prompt.contains("## 汇总合同"));
        assert!(prompt.contains("不复述任务、岗位或内部过程"));
        assert!(prompt.contains("只有用户明确要求虚拟员工或岗位分工时"));
        assert!(prompt.contains("半成品"));
        assert!(prompt.contains("中文小标题"));
        assert!(prompt.contains("【核心业务结论与定价判断】"));
        assert!(prompt.contains("Markdown 管道表格"));
        assert!(prompt.contains("分析数据并写文案"));
        assert!(prompt.contains("数据贡献"));
        assert!(!prompt.contains("# 多智能体协作结论"));
        assert!(!prompt.contains("worker.creative"));
    }

    #[test]
    fn final_synthesis_prompt_guides_experiment_review_modes() {
        let prompt = RolePromptBuilder::build_final_synthesis_prompt(&FinalSynthesisPromptInput {
            task_id: TaskId("task-review".to_string()),
            task: "复盘低糖冷萃咖啡MVP实验，判断是否放量".to_string(),
            support_workers: "worker.ops".to_string(),
            contributions: vec![RoleContribution {
                role_id: RoleId("role.ops-strategist".to_string()),
                runtime_role: "ops".to_string(),
                worker_id: WorkerId("worker.ops".to_string()),
                task_id: TaskId("task-review".to_string()),
                content: "experiment_review_report: \
                          MVP实验复盘报告已生成；复盘结论：可小步放量；赢家组合：G1；放量/重测/\
                          停止依据：样本达标。"
                    .to_string(),
                confidence: None,
                needs_follow_up: false,
            }],
        });

        assert!(prompt.contains("实验复盘"));
        assert!(prompt.contains("放量 / 调整重测 / 暂停停止 / 样本不足"));
        assert!(prompt.contains("ROAS、获客成本、毛利扣消耗、转化率、退款差评率"));
        assert!(prompt.contains("不要把实验计划当成已发生结果"));
        assert!(prompt.contains("experiment_review_report"));
        assert!(prompt.contains("MVP实验复盘报告已生成"));
        assert!(!prompt.contains("worker.ops"));
    }

    #[test]
    fn final_synthesis_prompt_keeps_partner_value_and_role_depth_contract() {
        let prompt = RolePromptBuilder::build_final_synthesis_prompt(&FinalSynthesisPromptInput {
            task_id: TaskId("task-partner-tone".to_string()),
            task: "判断学生宿舍桌面收纳盒是否可以小测".to_string(),
            support_workers: "worker.ops".to_string(),
            contributions: vec![RoleContribution {
                role_id: RoleId("role.ops-strategist".to_string()),
                runtime_role: "ops".to_string(),
                worker_id: WorkerId("worker.ops".to_string()),
                task_id: TaskId("task-partner-tone".to_string()),
                content: "可以小测，但要先验证主图点击和客服咨询顾虑。".to_string(),
                confidence: None,
                needs_follow_up: false,
            }],
        });

        assert!(prompt.contains("情绪价值和专业价值要同时出现"));
        assert!(prompt.contains("先接住用户当前的处境、限制或不确定感"));
        assert!(prompt.contains("今天能做的一个动作"));
        assert!(prompt.contains("角色深度要体现在业务判断里"));
        assert!(prompt.contains("不要只把岗位名换成普通建议"));
        assert!(prompt.contains("不要像老师批改、专家训话或系统报告"));
        assert!(!prompt.contains("worker.ops"));
    }
}
