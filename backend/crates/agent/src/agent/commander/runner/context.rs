use std::collections::HashMap;

use crate::cognition::Fact;

pub(super) fn extend_runtime_strategy_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let mode = runtime_response_mode(metadata);
    let answer_style = match mode {
        "quick" => {
            "回答模式：快速简答。优先给直接结论、关键理由和最多 3 \
             个下一步动作；减少铺垫、过程展开和长篇教学，除非用户明确要求详细推导。"
        }
        "execution" => {
            "回答模式：丰富执行。输出完整业务方案，优先使用 Markdown \
             表格呈现岗位分工、行动清单、负责人、验收指标、风险边界和复盘节奏。"
        }
        _ => {
            "回答模式：教学模式。默认面向电商学生和新手，先给可用结论，再用通俗语言解释概念、\
             判断过程、为什么这么做和下一步怎么练。"
        }
    };
    facts.push(Fact {
        source: "runtime.answer_style".to_string(),
        content: answer_style.to_string(),
        reliability: 1.0,
    });

    let explanation_depth = match runtime_learning_level(metadata) {
        "beginner" => {
            "讲解深度：低。尽量少用术语，遇到 \
             ROI、转化率、漏斗、客单价等专业词要用白话解释，并给简单例子。"
        }
        "professional" => {
            "讲解深度：高。保留关键指标、推导路径、约束假设、风险边界和可复核口径，\
             适合汇报和专业复盘。"
        }
        _ => "讲解深度：中。按电商岗位应用口径解释，术语不过度展开，但关键概念和动作口径要说清楚。",
    };
    facts.push(Fact {
        source: "runtime.explanation_depth".to_string(),
        content: explanation_depth.to_string(),
        reliability: 1.0,
    });

    let role_dispatch = match metadata
        .get("strategy.collaboration_mode")
        .map(String::as_str)
        .unwrap_or("auto")
    {
        "single" => "岗位分配：指定单岗。只让用户指定的一个角色负责本轮回答，不额外扩展协作角色。",
        "manual" => "岗位分配：指定多岗。主角色负责综合回答，协作角色分别产出独立判断和交接内容。",
        _ if mode == "quick" => {
            "岗位分配：自动分配。快速简答默认倾向单岗短链路，除非用户明确要求多角色协作。"
        }
        _ => {
            "岗位分配：自动分配。后端根据问题判断一个或多个角色，\
             复杂任务应体现角色分工和主角色汇总。"
        }
    };
    facts.push(Fact {
        source: "runtime.role_dispatch".to_string(),
        content: role_dispatch.to_string(),
        reliability: 1.0,
    });
}

pub(super) fn runtime_response_mode(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    if let Some(response_mode) = metadata
        .get("strategy.response_mode")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return response_mode;
    }
    if let Some(strategy_id) = metadata
        .get("strategy.id")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return strategy_id;
    }
    if metadata
        .get("strategy.id")
        .is_some_and(|value| value == "collab")
    {
        return "execution";
    }
    "learning"
}

fn runtime_learning_level(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    metadata
        .get("strategy.learning_level")
        .map(String::as_str)
        .filter(|value| matches!(*value, "beginner" | "higher_vocational" | "professional"))
        .unwrap_or("higher_vocational")
}

pub(super) fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    if let Some(history) = metadata
        .get("conversation.history_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        facts.push(Fact {
            source: "conversation.history".to_string(),
            content: format!(
                "同一会话历史上下文如下。回答追问、补充、继续、按前面数据等问题时，\
                 必须优先结合这段历史；不要要求用户重复已经在这里提供的信息。\n{history}"
            ),
            reliability: 1.0,
        });
    }
    extend_named_context_fact(
        metadata,
        facts,
        "context.selected_summary",
        "commerce.selected_product_context",
        "用户本轮手动选择的商品/知识上下文如下；只有存在这段事实时，才可按指定商品回答。\n",
    );
    if metadata
        .get("context.selected_summary")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        facts.push(Fact {
            source: "commerce.unselected_product_context".to_string(),
            content: "本轮没有用户手动选择的商品/知识上下文。可以结合用户文本、\
                      同一会话历史和工作区资料理解问题，但不得把历史商品、\
                      测试商品或默认商品当成本轮指定商品；\
                      需要单品级分析时应要求用户显式选择或提供商品。"
                .to_string(),
            reliability: 1.0,
        });
    }
    extend_named_context_fact(
        metadata,
        facts,
        "context.sources_json",
        "commerce.selected_product_context.sources",
        "",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "context.combined_summary",
        "request.combined_context",
        "本轮合并上下文摘要如下。它可能包含会话历史、工作区资料或 RAG 降级信息；只有 source 为 \
         commerce.selected_product_context 时才代表用户手动指定商品。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContext.summary",
        "workspace.context_summary",
        "本轮工作区上下文可用性摘要如下；它不是用户手动选择的商品资料。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.summary",
        "workspace.context_pack_summary",
        "本轮工作区可读资料摘要如下；它不是用户手动选择的商品资料。\n",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.items_json",
        "workspace.context_pack.items",
        "",
    );
}

fn extend_named_context_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
    prefix: &str,
) {
    let Some(value) = metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    facts.push(Fact {
        source: source.to_string(),
        content: format!("{prefix}{value}"),
        reliability: 1.0,
    });
}
