use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

use agent::api::{
    AgentId, AgentLlmCfg, AgentManager, AgentRequest, AgentRuntimeCfg, SessionEvent, SessionId,
    UserId, build_agent_manager,
};
use serde::Deserialize;
use toolcraft_config::load_settings;

const DEFAULT_CONFIG_PATH: &str = "config/services.toml";
const DEFAULT_TASK: &str = "基于数据诊断做详情页设计和预算风险评估，并给运营调整方案";

#[derive(Debug, Deserialize)]
struct SmokeSettings {
    #[serde(default)]
    llm: Vec<AgentLlmCfg>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let matrix_mode = args.iter().any(|arg| arg == "--matrix");
    let config_path = option_value(&args, "--config")
        .or_else(|| env::var("CONFIG_PATH").ok())
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let default_llm = option_value(&args, "--llm").or_else(|| env::var("AGENT_LLM").ok());
    let task = task_args(&args).join(" ");
    let task = if task.trim().is_empty() {
        DEFAULT_TASK.to_string()
    } else {
        task
    };

    let settings = load_smoke_settings(&config_path)?;
    let llm_names = settings
        .llm
        .iter()
        .map(|llm| llm.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    println!("config_path={config_path}");
    println!("llm_names=[{llm_names}]");
    println!(
        "selected_llm={}",
        default_llm.as_deref().unwrap_or("<first [[llm]]>")
    );
    println!(
        "roles_path={}",
        env_or_default("CM_AGENT_ROLES", "config/json/agent/roles.json")
    );
    println!(
        "task_graph_rules_path={}",
        env_or_default(
            "CM_AGENT_TASK_GRAPH_RULES",
            "config/json/agent/task-graph-rules.json"
        )
    );
    println!("task={task}");
    println!();

    let manager = Arc::new(build_agent_manager(AgentRuntimeCfg {
        llm: settings.llm,
        default_llm,
    })?);
    if matrix_mode {
        run_matrix(manager).await?;
        return Ok(());
    }

    let mut events = manager.run_stream(AgentRequest {
        user_id: Some(UserId("config-smoke-user".to_string())),
        workspace_id: None,
        agent_id: AgentId("config-smoke-agent".to_string()),
        session_id: SessionId("config-task-graph-smoke".to_string()),
        input: task,
        metadata: Default::default(),
    })?;

    while let Some(event) = events.recv().await {
        print_event(&event);
        if matches!(
            event,
            SessionEvent::Finished { .. } | SessionEvent::Failed { .. }
        ) {
            break;
        }
    }

    println!("active_sessions={}", manager.active_session_count());
    Ok(())
}

fn load_smoke_settings(path: &str) -> Result<SmokeSettings, Box<dyn std::error::Error>> {
    let settings: SmokeSettings = load_settings(path)?;
    if settings.llm.is_empty() {
        return Err(format!("config '{path}' does not contain any [[llm]] entries").into());
    }
    Ok(settings)
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then(|| window[1].clone()))
        .filter(|value| !value.trim().is_empty())
}

fn task_args(args: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--matrix" => index += 1,
            "--config" | "--llm" => index += 2,
            _ => {
                output.push(args[index].clone());
                index += 1;
            }
        }
    }
    output
}

fn env_or_default(env_name: &str, default: &str) -> String {
    env::var(env_name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[derive(Debug, Clone)]
struct MatrixCase {
    name: &'static str,
    task: &'static str,
    expected_roles: &'static [&'static str],
}

#[derive(Debug, Default)]
struct MatrixResult {
    planned_roles: HashSet<String>,
    started_roles: HashSet<String>,
    passed_roles: HashSet<String>,
    failed_roles: HashMap<String, Vec<String>>,
}

async fn run_matrix(manager: Arc<AgentManager>) -> Result<(), Box<dyn std::error::Error>> {
    let cases = role_matrix_cases();
    let mut covered_roles = HashSet::new();
    let mut all_ok = true;

    println!("matrix_cases={}", cases.len());
    println!();

    for (index, case) in cases.iter().enumerate() {
        println!("=== matrix_case={} name={} ===", index + 1, case.name);
        println!("task={}", case.task);
        let result = run_matrix_case(Arc::clone(&manager), case, index).await?;
        covered_roles.extend(result.started_roles.iter().cloned());

        let missing = case
            .expected_roles
            .iter()
            .filter(|role| !result.started_roles.contains(**role))
            .copied()
            .collect::<Vec<_>>();
        let failed_expected = case
            .expected_roles
            .iter()
            .filter(|role| !result.passed_roles.contains(**role))
            .copied()
            .collect::<Vec<_>>();
        let ok = missing.is_empty() && failed_expected.is_empty();
        all_ok &= ok;

        println!(
            "planned={:?} started={:?} passed={:?}",
            sorted_set(&result.planned_roles),
            sorted_set(&result.started_roles),
            sorted_set(&result.passed_roles)
        );
        if !result.failed_roles.is_empty() {
            println!("failed={:?}", result.failed_roles);
        }
        if !missing.is_empty() {
            println!("missing_expected_started={missing:?}");
        }
        if !failed_expected.is_empty() {
            println!("missing_expected_passed={failed_expected:?}");
        }
        println!("case_ok={ok}");
        println!();
    }

    let expected_all = [
        "chat",
        "ops",
        "data",
        "service",
        "creative",
        "engineering",
        "accounting",
        "design",
        "web",
    ];
    let missing_all = expected_all
        .iter()
        .filter(|role| !covered_roles.contains(**role))
        .copied()
        .collect::<Vec<_>>();
    println!("covered_roles={:?}", sorted_set(&covered_roles));
    println!("missing_roles={missing_all:?}");
    println!("matrix_ok={}", all_ok && missing_all.is_empty());
    println!("active_sessions={}", manager.active_session_count());

    Ok(())
}

async fn run_matrix_case(
    manager: Arc<AgentManager>,
    case: &MatrixCase,
    index: usize,
) -> Result<MatrixResult, Box<dyn std::error::Error>> {
    let mut events = manager.run_stream(AgentRequest {
        user_id: Some(UserId("config-matrix-user".to_string())),
        workspace_id: None,
        agent_id: AgentId("config-matrix-agent".to_string()),
        session_id: SessionId(format!("config-task-graph-matrix-{index}")),
        input: case.task.to_string(),
        metadata: Default::default(),
    })?;

    let mut result = MatrixResult::default();
    let mut node_roles = HashMap::<String, String>::new();

    while let Some(event) = events.recv().await {
        match &event {
            SessionEvent::TaskGraphPlanned { graph, .. } => {
                for node in &graph.nodes {
                    node_roles.insert(node.id.0.clone(), node.role.clone());
                    result.planned_roles.insert(node.role.clone());
                }
            }
            SessionEvent::TaskNodeStarted { node_id, .. } => {
                if let Some(role) = node_roles.get(&node_id.0) {
                    result.started_roles.insert(role.clone());
                }
            }
            SessionEvent::TaskNodeEvaluated { evaluation, .. } => {
                if let Some(role) = node_roles.get(&evaluation.node_id.0) {
                    if evaluation.passed {
                        result.passed_roles.insert(role.clone());
                    } else {
                        result
                            .failed_roles
                            .entry(role.clone())
                            .or_default()
                            .extend(evaluation.reasons.clone());
                    }
                }
            }
            SessionEvent::Finished { .. } | SessionEvent::Failed { .. } => break,
            _ => {}
        }
    }

    Ok(result)
}

fn role_matrix_cases() -> Vec<MatrixCase> {
    vec![
        MatrixCase {
            name: "chat",
            task: "请用三句话解释 agent session 是什么，以及它和普通聊天上下文有什么区别。",
            expected_roles: &["chat"],
        },
        MatrixCase {
            name: "ops",
            task: "为一个新品冷启动设计一套7天运营增长策略，包含投放、转化、复盘和风险控制。",
            expected_roles: &["ops"],
        },
        MatrixCase {
            name: "data_design_accounting_ops",
            task: "店铺A本周详情页UV从10000降到9200，详情页转化率从3.2%降到2.4%，加购率从8.5%\
                   降到6.1%，付费渠道CPC从1.2元升到1.6元，日预算上限3000元，毛利率35%。\
                   请基于这些数据诊断详情页问题，做详情页设计建议和预算风险评估，\
                   并给一套运营调整方案。",
            expected_roles: &["data", "design", "accounting", "ops"],
        },
        MatrixCase {
            name: "service",
            task: "客服团队近期投诉和退款增加，请设计一套售后SOP，包含首响话术、升级规则、\
                   退款边界和满意度复盘。",
            expected_roles: &["service"],
        },
        MatrixCase {
            name: "creative",
            task: "帮一个保温杯新品写3条短视频脚本钩子、3个标题和一版直播开场文案，不能编造功效。",
            expected_roles: &["ops", "creative"],
        },
        MatrixCase {
            name: "engineering",
            task: "系统接口延迟升高并影响SLA，请给出技术架构排查、性能优化、\
                   发布回滚和稳定性风险方案。",
            expected_roles: &["engineering"],
        },
        MatrixCase {
            name: "web",
            task: "为一个跨境电商独立站制定SEO关键词集群、标题优化、收录策略和自然流量增长方案。",
            expected_roles: &["web"],
        },
    ]
}

fn sorted_set(values: &HashSet<String>) -> Vec<String> {
    let mut values = values.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values
}

fn print_event(event: &SessionEvent) {
    match event {
        SessionEvent::TaskGraphPlanned { graph, .. } => {
            println!("event={} graph_id={}", event.event_name(), graph.graph_id.0);
            for node in &graph.nodes {
                let inputs = node
                    .input_refs
                    .iter()
                    .map(|input| input.0.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "  node={} role={} worker={} inputs=[{}]",
                    node.id.0, node.role, node.worker_id.0, inputs
                );
            }
        }
        SessionEvent::TaskNodeStarted {
            node_id,
            worker_id,
            attempt,
            ..
        } => {
            println!(
                "event={} node={} worker={} attempt={}",
                event.event_name(),
                node_id.0,
                worker_id.0,
                attempt
            );
        }
        SessionEvent::TaskNodeEvaluated { evaluation, .. } => {
            println!(
                "event={} node={} passed={} score={} reasons={:?}",
                event.event_name(),
                evaluation.node_id.0,
                evaluation.passed,
                evaluation.score,
                evaluation.reasons
            );
        }
        SessionEvent::Output { content, .. } => {
            println!("event={}", event.event_name());
            println!("{content}");
        }
        SessionEvent::Failed { reason, .. } => {
            println!("event={} reason={reason}", event.event_name());
        }
        _ => println!("event={}", event.event_name()),
    }
}
