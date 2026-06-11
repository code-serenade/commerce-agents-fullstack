use super::{sanitize_final_user_facing_text, sanitize_user_facing_text, user_facing_report_text};
use crate::core::protocol::{
    RoleWorkOutput, TaskGraphId, TaskId, TaskNodeId, WorkerId, WorkerReport, WorkerReportStatus,
};

#[test]
fn sanitizes_placeholder_and_note_language() {
    let raw = "内容创意(creative): \
               【可直接测试的方案（占位符需替换为真实信息）】建议突出【真实品类，如挂耳/速溶/\
               胶囊】和【真实规格/到手价】。\n备注：这是教学模式说明。";

    let output = sanitize_user_facing_text(raw).expect("output should remain readable");

    assert!(output.contains("建议突出真实品类和真实规格、到手价。"));
    assert!(output.contains("说明：这是教学模式说明。"));
    assert!(!output.contains("内容创意(creative)"));
    assert!(!output.contains("占位符"));
    assert!(!output.contains('【'));
    assert!(!output.contains('】'));
    assert!(!output.contains("备注："));
}

#[test]
fn removes_user_answer_placeholder_shell() {
    let raw = "给用户看的完整中文回复：\n\n## 结论\n\n这个商品先按学生开学场景做小范围验证。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("## 结论"));
    assert!(output.contains("学生开学场景"));
    assert!(!output.contains("给用户看的完整中文回复"));
}

#[test]
fn filters_internal_task_graph_lines() {
    let raw = "Task graph completed.\nPassed worker reports:\n- creative(worker.creative): \
               raw={\"role_output\":{}}\n真正可用的建议：先补商品信息。";

    let output = sanitize_user_facing_text(raw).expect("valid line should remain");
    let lower = output.to_ascii_lowercase();

    assert_eq!(output, "真正可用的建议：先补商品信息。");
    assert!(!lower.contains("task graph"));
    assert!(!lower.contains("worker"));
    assert!(!lower.contains("role_output"));
    assert!(!lower.contains("raw="));
}

#[test]
fn filters_internal_assignment_context_keys_from_final_answer() {
    let raw = "worker.assignment.input: {\"context\":{\"selected_summary\":\"已选商品\"},\
               \"sourceMap\":{},\"memoryBudget\":4096}\n\
               conversation.memorySummaryJson={\"rolling_summary\":\"上一轮历史\"}\n\
               真正建议：先补商品价格带，再做3个卖点小样本测试。";

    let output = sanitize_final_user_facing_text(raw).expect("business suggestion should remain");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("真正建议"));
    assert!(output.contains("先补商品价格带，再做3个卖点小样本测试。"));
    assert!(!lower.contains("worker.assignment.input"));
    assert!(!lower.contains("selected_summary"));
    assert!(!lower.contains("sourcemap"));
    assert!(!lower.contains("memorybudget"));
    assert!(!lower.contains("memorysummaryjson"));
}

#[test]
fn failed_report_surfaces_llm_access_denied_user_message() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-llm".to_string()),
        node_id: TaskNodeId("data".to_string()),
        task_id: TaskId("task-data".to_string()),
        worker_id: WorkerId("worker.data".to_string()),
        role: "data".to_string(),
        content: "llm_access_denied: 当前 AI 模型服务配置不可用".to_string(),
        role_output: None,
        evidence: vec!["failure_kind:llm_access_denied".to_string()],
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Failed,
    };

    let output = user_facing_report_text(&report).expect("diagnostic should be visible");

    assert!(output.contains("模型服务配置不可用"));
    assert!(!output.contains("services.toml"));
    assert!(!output.contains("[agent_llm_usage]"));
    assert!(!output.contains("API Key"));
    assert!(!output.contains("api_key"));
    assert!(!output.contains("Request id"));
}

#[test]
fn removes_empty_role_shell_and_worker_artifacts() {
    let raw = "Task graph completed. Task: 这个咖啡如何做营销\nPassed worker reports\n- \
               内容创意(creative): 【可直接测试的千乐咖啡营销创意（占位符需替换为真实信息）】\n1. \
               主推标题：千乐咖啡【真实品类（如挂耳/速溶/胶囊/咖啡豆）】日常口粮款\n备注：\
               替换真实品类后即可上线，测试核心指标参考点击率";

    let output = sanitize_user_facing_text(raw).expect("business content should remain");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("千乐咖啡真实品类日常口粮款"));
    assert!(output.contains("说明：替换真实品类后即可上线"));
    assert!(!lower.contains("task graph"));
    assert!(!lower.contains("passed worker"));
    assert!(!lower.contains("worker"));
    assert!(!output.contains("内容创意(creative)"));
    assert!(!output.contains("占位符"));
    assert!(!output.contains("备注："));
    assert!(!output.contains('【'));
    assert!(!output.contains('】'));
}

#[test]
fn includes_live_search_evidence_in_user_facing_text() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-search".to_string()),
        node_id: TaskNodeId("search".to_string()),
        task_id: TaskId("task-search".to_string()),
        worker_id: WorkerId("worker.search".to_string()),
        role: "search".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "本轮未拿到实时来源，不能直接给出对应规则明细。".to_string(),
            evidence: vec![
                "实时搜索来源（search_platform_policy）：1 个可采信来源\n\
                     1. 官方公告 | 2026-05-20 | https://example.com/policy | 风险点：价格合规"
                    .to_string(),
            ],
            ..RoleWorkOutput::default()
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("live evidence should be visible");

    assert!(output.contains("本轮已通过外部搜索取得实时来源"));
    assert!(output.contains("实时搜索来源"));
    assert!(output.contains("https://example.com/policy"));
    assert!(!output.contains("本轮未拿到"));
}

#[test]
fn filters_internal_search_evidence_trace_and_packet_from_user_answer() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-search".to_string()),
        node_id: TaskNodeId("search".to_string()),
        task_id: TaskId("task-search".to_string()),
        worker_id: WorkerId("worker.search".to_string()),
        role: "search".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "本轮已核对竞品价格，建议先用 12-19 元价格带做小样本测试。".to_string(),
            evidence: vec![
                "实时搜索来源（search_competitor）：1 个可采信来源\n\
                     1. 冷萃咖啡竞品价格参考 | 2026-05-20 | \
                     https://example.com/cold-brew-price | 竞品价格集中在 12-19 元。"
                    .to_string(),
                "search_evidence_packet:{\"sources\":[{\"url\":\"https://example.com/raw\"}]}"
                    .to_string(),
                "search_evidence_trace: skill=search_competitor status=success \
                     adapter_status=live intent=competitor_price_or_benchmark sources=1 \
                     usable_claims=1 gaps=0 confidence=0.72 query=\"冷萃咖啡 竞品价格\""
                    .to_string(),
            ],
            ..RoleWorkOutput::default()
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("answer should remain readable");

    assert!(output.contains("本轮已核对竞品价格"));
    assert!(output.contains("实时搜索来源"));
    assert!(output.contains("https://example.com/cold-brew-price"));
    assert!(!output.contains("search_evidence_packet"));
    assert!(!output.contains("search_evidence_trace"));
    assert!(!output.contains("https://example.com/raw"));
}

#[test]
fn degraded_search_packet_does_not_claim_live_sources() {
    let report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: String::new(),
            role_output: Some(RoleWorkOutput {
                summary: "本轮已通过外部搜索取得实时来源。\n继续按星河杯方案，标题强调温显、防漏和礼盒感。"
                    .to_string(),
                evidence: vec![
                    "search_evidence_packet:{\"adapter_status\":\"degraded\",\"source_state\":\"no_reliable_sources\",\"sources\":[],\"gaps\":[\"external search provider request failed: http://1.85.61.130:29003/api/external/search?q=test\"]}"
                        .to_string(),
                    "search_evidence_trace: skill=search_market_info status=deferred adapter_status=degraded sources=0 usable_claims=0 gaps=2 confidence=0.00"
                        .to_string(),
                ],
                ..RoleWorkOutput::default()
            }),
            evidence: Vec::new(),
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics: Vec::new(),
            status: WorkerReportStatus::Completed,
        };

    let output = user_facing_report_text(&report).expect("business answer should remain");

    assert!(output.contains("继续按星河杯方案"));
    assert!(!output.contains("本轮已通过外部搜索取得实时来源"));
    assert!(!output.contains("实时搜索来源"));
    assert!(!output.contains("search_evidence_packet"));
    assert!(!output.contains("search_evidence_trace"));
    assert!(!output.contains("adapter_status"));
    assert!(!output.contains("1.85.61.130"));
}

#[test]
fn drops_empty_evidence_heading_after_internal_trace_filtering() {
    let raw = "建议先检查竞品价格。\n依据:\n- search_evidence_trace: skill=search_competitor \
               status=success adapter_status=live";

    let output = sanitize_user_facing_text(raw).expect("business line should remain");

    assert_eq!(output, "建议先检查竞品价格。");
}

#[test]
fn non_search_role_keeps_business_answer_before_live_search_evidence() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-final".to_string()),
        node_id: TaskNodeId("ops_final".to_string()),
        task_id: TaskId("task-final".to_string()),
        worker_id: WorkerId("worker.ops".to_string()),
        role: "ops".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "建议先用校园内小范围试卖验证冷萃咖啡价格带，再按毛利和复购反馈调整。"
                .to_string(),
            evidence: vec![
                "实时搜索来源（search_competitor）：1 个可采信来源\n\
                     1. 竞品价格参考 | 2026-05-20 | https://example.com/price | 价格区间"
                    .to_string(),
            ],
            ..RoleWorkOutput::default()
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("business answer should be visible");

    assert!(output.starts_with("建议先用校园内小范围试卖"));
    assert!(output.contains("实时搜索来源"));
    assert!(output.contains("https://example.com/price"));
}

#[test]
fn non_search_role_keeps_material_evidence_when_summary_is_too_generic() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-material".to_string()),
        node_id: TaskNodeId("data_final".to_string()),
        task_id: TaskId("task-material".to_string()),
        worker_id: WorkerId("worker.data".to_string()),
        role: "data".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "根据选中文件的广告数据，当前最大问题是退款率偏高，并非点击率不足。"
                .to_string(),
            evidence: vec![
                "资料依据：广告数据里点击率 CTR=1.2%、转化率 \
                 CVR=2.1%、退款率=18%，最大问题是退款率偏高，不是点击率不足。"
                    .to_string(),
                "skill:material_file_understanding status=success summary=已形成资料理解。"
                    .to_string(),
            ],
            ..RoleWorkOutput::default()
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("answer should include evidence");

    assert!(output.contains("当前最大问题是退款率偏高"));
    assert!(output.contains("资料依据"));
    assert!(output.contains("退款率=18%"));
    assert!(output.contains("点击率 CTR=1.2%"));
    assert!(output.contains("点击率不是主要矛盾"));
    assert!(!output.contains("点击率不足"));
    assert!(!output.contains("skill:"));
    assert!(!output.contains("material_file_understanding"));
}

#[test]
fn role_output_keeps_findings_recommendations_and_risks_in_user_answer() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-rich-role-output".to_string()),
        node_id: TaskNodeId("ops_final".to_string()),
        task_id: TaskId("task-rich-role-output".to_string()),
        worker_id: WorkerId("worker.ops".to_string()),
        role: "ops".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "已经稳定：销量、转化率、用户评价这三个观察指标已经明确。".to_string(),
            findings: vec![
                "仍有风险：真实曝光、点击、成交和退款数据还不足，不能直接验证优化效果。"
                    .to_string(),
            ],
            recommendations: vec![
                "学生下一步：先补齐一周样本数据，再用两个版本主图做小测试。".to_string(),
            ],
            risks: vec!["注意不要把没有核验过的网络资料当成确定结论。".to_string()],
            open_questions: vec!["还需要确认活动期间的目标 ROAS 和可接受退款率。".to_string()],
            evidence: Vec::new(),
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("rich role output should be visible");

    assert!(output.contains("已经稳定：销量、转化率、用户评价这三个观察指标已经明确。"));
    assert!(output.contains("仍有风险：真实曝光、点击、成交和退款数据还不足"));
    assert!(output.contains("学生下一步：先补齐一周样本数据"));
    assert!(output.contains("注意不要把没有核验过的网络资料当成确定结论。"));
    assert!(output.contains("还需要确认活动期间的目标 ROAS 和可接受退款率。"));
    assert!(!output.contains("role_output"));
    assert!(!output.contains("worker."));
}

#[test]
fn material_boundary_filters_blocking_wait_and_unable_lines() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-material-boundary".to_string()),
        node_id: TaskNodeId("ops_final".to_string()),
        task_id: TaskId("task-material-boundary".to_string()),
        worker_id: WorkerId("worker.ops".to_string()),
        role: "ops".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "先给学生书包开学季满减的可执行分析：先列字段，再看点击、加购、付款和退款。"
                .to_string(),
            findings: vec![
                "未提供真实曝光、点击、加购、付款等指标时，不能编造转化率，\
                 但可以先给建表字段和判断方法。"
                    .to_string(),
            ],
            recommendations: vec![
                "等待用户提供附件或表格数据。".to_string(),
                "先建一张基础表：日期、商品/SKU、渠道、曝光、点击、加购、下单、付款、GMV、退款、\
                 库存、客服咨询。"
                    .to_string(),
            ],
            risks: vec![
                "无法进行字段、粒度、主键、指标口径、样本边界和缺失数据的定义。".to_string(),
                "资料边界：表格资料分析没有产出可引用资料；本轮没有收到可解析的表格行或表格文本。\
                 以下业务判断只能先按用户文字和已进入上下文的信息给出，等补齐资料后再复核。"
                    .to_string(),
            ],
            open_questions: vec!["是否提供了附件或表格数据？".to_string()],
            evidence: Vec::new(),
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("answer should remain");

    assert!(output.contains("学生书包开学季满减"));
    assert!(output.contains("先建一张基础表"));
    assert!(output.contains("资料边界"));
    assert!(!output.contains("等待用户提供"));
    assert!(!output.contains("无法进行字段"));
    assert!(!output.contains("是否提供了附件"));
}

#[test]
fn material_evidence_suppresses_irrelevant_no_live_source_fallback() {
    let report = WorkerReport {
        graph_id: TaskGraphId("graph-material".to_string()),
        node_id: TaskNodeId("data_final".to_string()),
        task_id: TaskId("task-material".to_string()),
        worker_id: WorkerId("worker.data".to_string()),
        role: "data".to_string(),
        content: String::new(),
        role_output: Some(RoleWorkOutput {
            summary: "本轮没有可引用的实时外部来源，所以先不给平均数、排名或确定性对标。"
                .to_string(),
            evidence: vec![
                "资料依据：广告数据里点击率 CTR=1.2%、转化率 \
                 CVR=2.1%、退款率=18%，最大问题是退款率偏高。"
                    .to_string(),
            ],
            ..RoleWorkOutput::default()
        }),
        evidence: Vec::new(),
        risks: Vec::new(),
        open_questions: Vec::new(),
        diagnostics: Vec::new(),
        status: WorkerReportStatus::Completed,
    };

    let output = user_facing_report_text(&report).expect("material evidence should answer");

    assert!(output.contains("资料依据"));
    assert!(output.contains("退款率=18%"));
    assert!(!output.contains("实时外部来源"));
    assert!(!output.contains("平均数、排名"));
}

#[test]
fn sanitizing_collapses_repeated_answer_blocks() {
    let raw = "先看毛利，再看销量。\n- 补齐成本字段。\n\n先看毛利，再看销量。\n- 补齐成本字段。";

    let output = sanitize_user_facing_text(raw).expect("deduped output should remain");

    assert_eq!(output, "先看毛利，再看销量。\n- 补齐成本字段。");
}

#[test]
fn final_sanitizing_collapses_non_adjacent_duplicate_blocks() {
    let raw = "先看毛利，再看销量。\n- \
               补齐成本字段。\n\n资料边界：附件成本字段仍需核验。\n\n先看毛利，再看销量。\n- \
               补齐成本字段。";

    let output = sanitize_final_user_facing_text(raw).expect("deduped output should remain");
    assert_eq!(
        output,
        "先看毛利，再看销量。\n- 补齐成本字段。\n\n资料边界：附件成本字段仍需核验。"
    );
}

#[test]
fn filters_internal_context_prompt_leakage_from_final_answer() {
    let raw = "针对千乐咖啡，当前只确认了名称、售价和在售状态，缺少品类、平台和成本数据。\\
                   nCommerce product context resolved from current user's backend data. Use these \
               facts when relevant, cite product or knowledge titles, and do not invent missing \
               product details. Product source product:ce3f729c-a8bd-45ca-9859-7f2591d5973f | \
               name=千乐咖啡 | platform=unknown | category=unknown | brand=unknown | price=100 \
               CNY | status=active | description=适合于白领 Requested context refs: \
               product:ce3f729c-a8bd-45ca-9859-7f2591d5973f\n建议先补真实成本、\
               目标平台和用户反馈，再判断是否继续做活动。";

    let output = sanitize_final_user_facing_text(raw).expect("business content should remain");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("针对千乐咖啡"));
    assert!(output.contains("建议先补真实成本"));
    assert!(!lower.contains("commerce product context"));
    assert!(!lower.contains("product source product"));
    assert!(!lower.contains("requested context refs"));
    assert!(!lower.contains("platform=unknown"));
    assert!(!lower.contains("product:"));
    assert!(!output.contains("ce3f729c-a8bd-45ca-9859-7f2591d5973f"));
}

#[test]
fn filters_inline_context_fields_without_destroying_answer_layout() {
    let raw = "本轮已通过外部搜索取得实时来源。\n当前岗位任务：整合已选商品 已选商品 \
               title=QA矩阵智能保温杯 | product_knowledge:已选商品 title=QA矩阵商品知识 | \
               confidence=0.96\n先给结论：这款商品应该主打温显、防漏和礼盒感，不要做学生低价定位。\
               \n\n建议动作：\n- 主图第一屏放温显和防漏场景。\n- 活动页强调三件套礼盒。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("先给结论："));
    assert!(output.contains("这款商品应该主打温显、防漏和礼盒感"));
    assert!(output.contains("建议动作：\n- 主图第一屏"));
    assert!(!output.contains("当前岗位任务"));
    assert!(!output.contains("title="));
    assert!(!lower.contains("product_knowledge"));
    assert!(!lower.contains("confidence"));
}

#[test]
fn filters_conversation_history_wrappers_from_final_answer() {
    let raw = "给新手学生的三条618返场短视频开头文案：\n\n1. \
               办公室姐妹看过来，温显防漏保温杯返场了。\n\nUser: 请分析这个商品。\nAssistant: \
               这个商品在618返场活动里最应该主打温显、防漏、礼盒。\n当前用户请求：\\\\
                   \
               nn继续同一个商品和活动，给我三条短视频开头文案。\n\n基于上游数据、设计、\
               预算或创意结果，形成可执行运营调整方案。\n已选商品 \
               已选商品\n落地注意：每天看退款率，超过8%就调整素材。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("办公室姐妹看过来"));
    assert!(output.contains("落地注意"));
    assert!(!lower.contains("user:"));
    assert!(!lower.contains("assistant:"));
    assert!(!output.contains("当前用户请求"));
    assert!(!output.contains("基于上游数据、设计、预算或创意结果"));
    assert!(!output.contains("已选商品 已选商品"));
}

#[test]
fn filters_generic_task_objective_shell_from_final_answer() {
    let raw = "给新手学生的三条618返场短视频开头文案：\n\n1. \
               办公室姐妹看过来，温显防漏保温杯返场了。\n\n整合财务、千川投放和创意结果，\
               形成可执行运营方案。\n\n落地注意：每天看退款率，超过8%就调整素材。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("办公室姐妹看过来"));
    assert!(output.contains("落地注意"));
    assert!(!output.contains("整合财务、千川投放和创意结果"));
}

#[test]
fn filters_history_questions_misclassified_as_material_evidence() {
    let raw = "学生开网店第一周先看访客数、点击率和咨询加购率。\n\n资料依据：- \
               请分析这个商品在618返场活动里最应该主打什么卖点？回答给新手学生看\n\n资料依据：\
               原因是商品资料明确要求主打这三个点，活动针对的是办公室女性客群。\n\n资料依据：\
               广告数据里点击率 CTR=1.2%、转化率 CVR=2.1%、退款率=18%，最大问题是退款率偏高。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("学生开网店第一周先看访客数"));
    assert!(output.contains("退款率=18%"));
    assert!(!output.contains("请分析这个商品"));
    assert!(!output.contains("618返场活动"));
    assert!(!output.contains("办公室女性客群"));
}

#[test]
fn filters_internal_memory_labels_from_final_answer() {
    let raw = "继续沿用上一轮的校园二手教材包和618返场活动，先把首屏标题压短。\n\\
                   n资料关键事实：Same conversation structured memory: product=校园二手教材包; \
               campaign=618返场教材包\n资料关键事实：Rolling summary: No older compressed \
               history. Recent user facts: asked for table and image \
               diagnosis.\n\n首屏建议：突出版本齐全、48小时发出和开学前可复核书况。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("校园二手教材包"));
    assert!(output.contains("首屏建议"));
    assert!(!lower.contains("same conversation structured memory"));
    assert!(!lower.contains("rolling summary"));
    assert!(!lower.contains("recent user facts"));
    assert!(!lower.contains("no older compressed history"));
}

#[test]
fn rewrites_internal_evidence_terms_from_final_answer() {
    let raw = "QA矩阵智能保温杯适合做学生电商实训，因为商品锚点：温显、防漏、礼盒都清楚。\\
                   n资料边界：低置信参考只能作为补充，置信度0.96 \
               的内容仍要用真实素材核对，confidence=0.72 \
               不能直接展示给学生。\n建议先围绕宿舍、社团礼品和冬季热饮三个场景做短视频测试。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("QA矩阵智能保温杯"));
    assert!(output.contains("温显、防漏、礼盒"));
    assert!(output.contains("仍需核验的参考"));
    assert!(output.contains("资料来源较明确"));
    assert!(output.contains("不能直接展示给学生"));
    assert!(output.contains("短视频测试"));
    assert!(!output.contains("商品锚点"));
    assert!(!output.contains("低置信"));
    assert!(!output.contains("置信度"));
    assert!(!lower.contains("confidence"));
}

#[test]
fn final_sanitizing_restores_readable_sections_without_internal_terms() {
    let raw = "同学们好，基于当前你手动引用的QA矩阵智能保温杯全部资料，\
               现将这款商品适合和不适合做学生电商实训的地方明确梳理如下：\
               【核心业务结论与定价判断】这款商品当前129元的售价完全符合不低于119元的定价安全线，\
               定价合规性明确，整体规则边界清晰。【100%确定、\
               可直接落地的适合实训环节】选品定位练习：商品有明确的客群边界，\
               严格禁止学生党低价定位。内容合规练习：首屏文案必须包含温显、防漏、\
               礼盒三个核心卖点。【当前资料缺失、暂不适合开展的实训环节】当前没有实际曝光、点击、\
               成交、成本、退款等真实运营数据。【依据说明】确定部分的支撑来源：\
               QA矩阵智能保温杯商品锚点、QA矩阵商品知识、QA矩阵卖点素材，置信度0.96。\
               【下一步核验动作】你可以优先核验抖音平台智能保温杯类目的公开运营规则。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("【核心业务结论与定价判断】\n\n这款商品"));
    assert!(output.contains("\n\n【100%确定、可直接落地的适合实训环节】\n\n选品定位练习"));
    assert!(output.contains("\n\n【当前资料缺失、暂不适合开展的实训环节】\n\n当前没有实际曝光"));
    assert!(output.contains("\n\n【依据说明】\n\n确定部分的支撑来源"));
    assert!(output.contains("商品资料、QA矩阵商品知识"));
    assert!(output.contains("资料来源较明确"));
    assert!(!output.contains("商品锚点"));
    assert!(!output.contains("置信度"));
    assert!(!lower.contains("confidence"));
}

#[test]
fn final_sanitizing_keeps_existing_paragraph_breaks() {
    let raw = "先给结论：这款商品可以作为学生实训样例，但不能直接当成低价引流品。\n\\
                   \
               n适合部分：商品卖点、价格边界和场景都比较清楚，可以练选品定位。\n\n依据说明：\
               支撑来自QA矩阵智能保温杯商品锚点，置信度0.96。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();

    assert!(output.contains("样例，但不能直接当成低价引流品。\n\n适合部分："));
    assert!(output.contains("选品定位。\n\n依据说明："));
    assert!(output.contains("QA矩阵智能保温杯商品资料"));
    assert!(output.contains("资料来源较明确"));
    assert!(!output.contains("商品锚点"));
    assert!(!output.contains("置信度"));
    assert!(!lower.contains("confidence"));
}

#[test]
fn final_sanitizing_restores_generic_inline_colon_headings() {
    let raw = "这款商品可以做实训，但要先限定任务范围。适合部分：围绕卖点、价格和场景做选品定位。\
               不适合部分：没有真实投放数据时，不要做确定性ROI判断。核验动作：补成本、\
               退款和平台规则资料。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("限定任务范围。\n\n适合部分：\n\n围绕卖点"));
    assert!(output.contains("选品定位。\n\n不适合部分：\n\n没有真实投放数据"));
    assert!(output.contains("ROI判断。\n\n核验动作：\n\n补成本"));
}

#[test]
fn final_sanitizing_restores_inline_practice_headings() {
    let raw = "当前暂不适合做进阶判断。进阶数据类实训：没有曝光、点击、成交数据，暂不做转化测算。\
               行业对比类实训：没有可采信公开来源，暂不做行业对标。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("进阶判断。\n\n进阶数据类实训：\n\n没有曝光"));
    assert!(output.contains("转化测算。\n\n行业对比类实训：\n\n没有可采信"));
}

#[test]
fn final_sanitizing_restores_inline_numbered_lists() {
    let raw = "以下是给电商学生的全品类通用选品复盘清单，包含核心复盘模块、字段及风险提示：1. \
               前置说明：本清单为通用运营规则；2. 选品启动前复盘：核验选品依据；3. \
               小范围试卖后复盘：统计曝光、点击、咨询。风险边界：无对应资质不得进入高风险品类。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("风险提示：\n\n1. 前置说明："));
    assert!(output.contains("通用运营规则；\n\n2. 选品启动前复盘："));
    assert!(output.contains("核验选品依据；\n\n3. 小范围试卖后复盘："));
    assert!(output.contains("咨询。\n\n风险边界："));
}

#[test]
fn final_sanitizing_promotes_plain_section_headings() {
    let raw = "当前判断\n校园二手教材包可以先做开学前低风险试跑。\n\n下一步动作\n先核对版本、\
               ISBN、成色和缺页情况。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("## 当前判断\n校园二手教材包"));
    assert!(output.contains("## 下一步动作\n先核对版本"));
}

#[test]
fn final_sanitizing_does_not_promote_risk_sentence_as_heading() {
    let raw = "未核验平台规则可能导致素材违规、投放受限、定价被误判\\
                   n提前囤积库存可能导致滞销现金流风险\n是否有指定的淘宝平台规则查询入口？";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("提前囤积库存可能导致滞销现金流风险"));
    assert!(!output.contains("## 提前囤积库存可能导致滞销现金流风险"));
}

#[test]
fn final_sanitizing_formats_business_key_value_blocks_as_bullets() {
    let raw = "教学模式分析：校园二手教材包是否适合开学季电商实训\n\n判断逻辑（通俗版）\\
                   \
               n需求匹配度：开学前学生确实需要按课程快速配齐教材。\n履约难度：版本、ISBN、\
               成色和缺页必须能逐本核对。\n风险边界：不能承诺全新、官方指定或固定升值。\n\\
                   n下一步实训动作\n核对每本书的版本、ISBN、成色和缺页\\
                   n建一张教材包字段表记录成本、售价和售后问题";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("## 判断逻辑"));
    assert!(output.contains("- 需求匹配度：开学前学生确实需要按课程快速配齐教材。"));
    assert!(output.contains("- 履约难度：版本、ISBN、成色和缺页必须能逐本核对。"));
    assert!(output.contains("- 风险边界：不能承诺全新、官方指定或固定升值。"));
    assert!(output.contains("## 下一步实训动作"));
    assert!(output.contains("- 核对每本书的版本、ISBN、成色和缺页"));
    assert!(output.contains("- 建一张教材包字段表记录成本、售价和售后问题"));
}

#[test]
fn final_sanitizing_formats_short_copy_lines_and_drops_empty_evidence_heading() {
    let raw = "给新手学生的三条短视频开头文案：\n开学季高数英语经济学教材一次配齐，\
               版本可核对48小时发\n成套二手教材包，版本对得上才发货，48小时就能收到\\
                   n开学教材不用挨个找，成套版本可查48小时发出\n\n资料依据：## 下一步实训动作 \
               准备3-5套真实教材包，补齐版本和书况照片。\n\n资料依据：";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("给新手学生的三条短视频开头文案："));
    assert!(output.contains("- 开学季高数英语经济学教材一次配齐，版本可核对48小时发"));
    assert!(output.contains("- 成套二手教材包，版本对得上才发货，48小时就能收到"));
    assert!(output.contains("- 开学教材不用挨个找，成套版本可查48小时发出"));
    assert!(output.contains("## 下一步实训动作"));
    assert!(output.contains("准备3-5套真实教材包"));
    assert!(!output.ends_with("资料依据："));
    assert!(!output.contains("资料依据：##"));
}

#[test]
fn final_sanitizing_truncates_tail_history_answer_quotes() {
    let raw = "校园二手教材包+618返场活动 优先级动作排布：\n- \
               今天：核对所有在售教材包的版本清单、书况标签和618返场活动公示内容。\n- \
               优先理由：这是交易链路最前端的避错动作，减少版本咨询混乱、退单和售后争议。\n- \
               明天：把之前产出的3条短视频开头文案对应素材全部上线，每条绑定验证指标投放。\n- \
               优先理由：信息对齐后立刻落地内容引流，承接618返场剩余流量。\n- \
               本周：整理真实交易数据，统计版本核对咨询量、下单转化率、售后反馈量和发货达标率。\\
               n- 优先理由：前面的信息校验和内容投放跑完后，再汇总全链路数据做复盘。\n- \
               所有动作过程中只记录真实产生的数据，不要编造任何指标数值。\n\n## 判断依据\n\n- \
               精准踩中刚性需求：大学生开学最急需配齐新学期教材，打包售卖免去逐本找货。\n- \
               卖点真实无需编造：59元定价亲民，成套配齐、版本可核对、48小时发出。\n- \
               合规风险低：所属淘宝图书教材/校园二手类目，新手不容易踩规则红线。\n\n资料依据：\n\\
               n2. \
               文案：怕买错教材版本？这款二手教材包支持版本核对，高数、\
               英语等开学必备教材成套打包，59元配齐，48小时发货";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("校园二手教材包+618返场活动"));
    assert!(output.contains("本周：整理真实交易数据"));
    assert!(!output.contains("## 判断依据"));
    assert!(!output.contains("精准踩中刚性需求"));
    assert!(!output.contains("资料依据"));
    assert!(!output.contains("2. 文案：怕买错教材版本"));
}

#[test]
fn final_sanitizing_filters_worker_output_policy_lines() {
    let raw = "动作优先级排序\n- 今天：完成商品首屏ISBN核对流程上线\n- \
               明天：完成校园社群、短视频两个核心渠道的ROAS分渠道复盘\n- \
               本周：全量上线下单前书况、退换规则明确告知的链路优化\n\n排序理由\\
                   n版本咨询是用户当前最集中的转化阻碍，今天上线就能直接减少客服压力。\\
                   \
               n先写清目标、周期、预算和成功指标；如果用户只要局部交付，先完成局部交付，\
               不把短任务展开成完整方案。\n把数据、创意、投放和客服输入合并成一张行动清单：谁做、\
               今天做什么、明天看什么数据。\n保留清晰的复盘口径，重点看点击、加购、付款、\
               退款和客服咨询；表现明显低于本轮其他方向或本店上一轮基准时再调整。\\
                   \
               n缺资料时先给当前问题能用的版本，等用户补齐商品、平台、价格、预算或表格后再精修。\\
               \
               n先把这类商品当成一个小项目来做：明确用户真正要解决的问题，再选择人群、场景、渠道、\
               素材或复盘指标中的关键动作，避免一开始就做大预算。\\
               \
               n同时改动过多变量会导致复盘无法归因；每轮只改最影响判断的少量变量。\n资料依据：\
               判断依据：大学生开学需要快速配齐教材，59元、版本可核对、48小时发出、书况清晰标注";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("动作优先级排序"));
    assert!(output.contains("版本咨询是用户当前最集中的转化阻碍"));
    assert!(!output.contains("先写清目标"));
    assert!(!output.contains("把数据、创意、投放和客服输入"));
    assert!(!output.contains("保留清晰的复盘口径"));
    assert!(!output.contains("缺资料时先给当前问题能用的版本"));
    assert!(!output.contains("先把这类商品当成一个小项目"));
    assert!(!output.contains("同时改动过多变量"));
    assert!(!output.contains("资料依据：判断依据"));
}

#[test]
fn final_sanitizing_filters_bracketed_history_evidence_prefix() {
    let raw = "优先级动作\n- 今天：补全618返场投放素材的版本核验、书况标签、成套配齐核心卖点元素。\\
             n- 明天：把版本核对流程更新到商品详情页首屏。\n\n资料依据：【判断依据】1. \
         天然场景：开学季大学生必买教材，不用教育用户需求 2. 低门槛：客单价59元，试错成本低。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("优先级动作"));
    assert!(output.contains("版本核验"));
    assert!(!output.contains("资料依据"));
    assert!(!output.contains("【判断依据】"));
}

#[test]
fn final_sanitizing_keeps_current_turn_copywriting_items() {
    let raw = "文案：开学季教材不用逐本找！成套高数、英语、经济学教材配齐，版本可核对，\
               48小时发出。画面：展示打包好的成套教材。转化意图：吸引学生点击。\n\n2. \
               文案：怕买错教材版本？这款二手教材包支持版本核对，59元配齐，48小时发货。画面：\
               手持教材展示ISBN码。转化意图：提升下单转化率。\n\n3. \
               文案：开学季环保又省钱！二手教材包59元配齐教材，书况清晰标注。画面：书况标签特写。\
               转化意图：激发购买意愿。";

    let output = sanitize_final_user_facing_text(raw).expect("copy answer should remain");

    assert!(output.contains("文案：开学季教材不用逐本找"));
    assert!(output.contains("2. 文案：怕买错教材版本"));
    assert!(output.contains("3. 文案：开学季环保又省钱"));
}

#[test]
fn final_sanitizing_filters_tail_material_evidence_history_replay() {
    let raw = "建议今天先做库存盘点，确认每个品类的可售套数是否能支撑初期活动订单。\n\n原因：\
               如果库存不足，「开学公共课教材一次配齐」这个核心卖点不成立，\
               还会导致缺货和退款纠纷。\n\n资料依据：运营对象&平台 - \
               服务对象：开学季需要购买高数、英语、经济学等公共课教材的大学生 - 运营平台：淘宝 - \
               商品属性：公共课二手教材包，售价59元\n\n资料依据：表格要包含动作、负责人、\
               判断指标、达标线、风险处理，重点看59元定价、库存、退款和ROAS";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain");

    assert!(output.contains("建议今天先做库存盘点"));
    assert!(output.contains("退款纠纷"));
    assert!(!output.contains("资料依据"));
    assert!(!output.contains("运营对象&平台"));
    assert!(!output.contains("表格要包含动作"));
}

#[test]
fn final_sanitizing_filters_material_evidence_heading_with_history_replay_body() {
    let raw = "三条文案均严格控制在18字以内，完全匹配主图素材的预设布局位置。\n\n资料依据：\n\\
                   n3. 复盘字段：活动期间统计咨询top问题、成交转化率、退款原因、库存剩余量";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain");

    assert!(output.contains("三条文案均严格控制在18字以内"));
    assert!(!output.contains("资料依据"));
    assert!(!output.contains("复盘字段"));
    assert!(!output.contains("咨询top问题"));
}

#[test]
fn final_sanitizing_preserves_markdown_tables_for_frontend_table_renderer() {
    let raw = "## 执行表\n\n| 指标 | 怎么看 | 可能问题 | 下一步动作 |\n| --- | --- | --- | --- \
               |\n| 触达/点击 | 用户是否愿意进入详情 | 入口表达不清 | 测试标题和主图 |\n| \
               转化/付款 | 用户是否完成购买 | 价格或信任不足 | 补清楚版本、书况和发货 \
               |\n\n风险边界：没有真实销量和退款数据时，只能做方向判断。";

    let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

    assert!(output.contains("| 指标 | 怎么看 | 可能问题 | 下一步动作 |"));
    assert!(output.contains("| --- | --- | --- | --- |"));
    assert!(
        output.contains("| 触达/点击 | 用户是否愿意进入详情 | 入口表达不清 | 测试标题和主图 |")
    );
    assert!(output.contains("\n\n风险边界："));
}

#[test]
fn final_sanitizing_repairs_broken_multiline_table_rows() {
    let raw = "618返场执行表\n| 时间 | 动作 | 负责人 | 风险处理 |\n| ---- | ---- | ---- | ---- \
               |\n| 今天 | 1. 完成主图上线；\n\n2. 同步客服规范；\n\n3. 盘点教材包库存 | 学生运营 \
               | 1. 若素材排版不符合要求，1小时内调整；\n\n2. 若库存不足，先临时下架对应链接 |\n| \
               明天 | 每2小时看咨询和付款 | 学生运营+客服 | 若ROAS低于2，暂停付费投放 |";

    let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

    assert!(output.contains("| 时间 | 动作 | 负责人 | 风险处理 |"));
    assert!(output.contains("| ---- | ---- | ---- | ---- |"));
    assert!(output.contains(
        "| 今天 | 1. 完成主图上线； 2. 同步客服规范； 3. 盘点教材包库存 | 学生运营 | 1. \
         若素材排版不符合要求，1小时内调整； 2. 若库存不足，先临时下架对应链接 |"
    ));
    assert!(
        output
            .contains("| 明天 | 每2小时看咨询和付款 | 学生运营+客服 | 若ROAS低于2，暂停付费投放 |")
    );
    assert!(!output.contains("| 今天 | 1. 完成主图上线；\n\n2."));
}

#[test]
fn final_sanitizing_splits_inline_markdown_tables() {
    let raw = "今日到后天618返场活动执行表如下，聚焦59元定价、库存、退款、ROAS核心指标：\
               |执行日期|动作|负责人|判断指标|达标线|风险处理| |---|---|---|---|---|---| \
               |今日|核对教材库存|库存岗|库存盘点准确率|账实一致|库存不足时关闭超库存下单通道| \
               |明日|监控ROAS和退款|数据岗|ROAS/退款率|ROAS不低于目标|低转化渠道暂停投放|\\
                   n活动预算8888元，目标ROAS3.2。";

    let output = sanitize_final_user_facing_text(raw).expect("table answer should remain");

    assert!(
        output
            .contains("今日到后天618返场活动执行表如下，聚焦59元定价、库存、退款、ROAS核心指标：")
    );
    assert!(output.contains("\n|执行日期|动作|负责人|判断指标|达标线|风险处理|"));
    assert!(output.contains("\n|---|---|---|---|---|---|"));
    assert!(output.contains(
        "\n|今日|核对教材库存|库存岗|库存盘点准确率|账实一致|库存不足时关闭超库存下单通道|"
    ));
    assert!(
        output.contains(
            "\n|明日|监控ROAS和退款|数据岗|ROAS/退款率|ROAS不低于目标|低转化渠道暂停投放|"
        )
    );
}

#[test]
fn final_sanitizing_bulletizes_table_followup_lines() {
    let raw = "校园二手教材包618返场活动复盘表\n| 指标 | 怎么看 | 可能问题 | 下一步动作 |\n| ---- \
               | ---- | ---- | ---- |\n| 48小时发货率 | 核对承诺是否达标 | 库存或打包效率不足 | \
               按套系统计库存 |\n618返场活动预算8888元，目标ROAS3.2，核心投放渠道为校园社群、\
               短视频\\
                   n校园二手教材包核心风险点集中在版本不匹配、书况争议、库存滞销、退换纠纷\\
                   n优先优化首屏版本核对提示，降低用户咨询量，减少后续版本不符的退款风险";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("## 校园二手教材包618返场活动复盘表"));
    assert!(output.contains("| 指标 | 怎么看 | 可能问题 | 下一步动作 |"));
    assert!(
        output.contains("- 618返场活动预算8888元，目标ROAS3.2，核心投放渠道为校园社群、短视频")
    );
    assert!(
        output.contains("- 校园二手教材包核心风险点集中在版本不匹配、书况争议、库存滞销、退换纠纷")
    );
    assert!(
        output.contains("- 优先优化首屏版本核对提示，降低用户咨询量，减少后续版本不符的退款风险")
    );
}

#[test]
fn removes_unverified_live_search_availability_claims() {
    let raw = "外部搜索虽可用但暂未完成，先按已选商品资料判断。\n本轮未拿到实时来源，\
               不能当作最新竞品价格。\n可先把二手教材包价格和书况字段整理出来。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("本轮未拿到实时来源"));
    assert!(output.contains("二手教材包价格"));
    assert!(!output.contains("外部搜索虽可用"));
    assert!(!output.contains("外部搜索可用"));
}

#[test]
fn rewrites_internal_skill_ids_for_student_answer() {
    let raw = "本地无匹配商品知识库，当前无其他工作区数据源可用，\
               material_table_profiler技能未收到可剖析的表格行数据，暂无法产出结果。";

    let output = sanitize_user_facing_text(raw).expect("answer should remain readable");

    assert!(output.contains("表格资料分析"));
    assert!(!output.contains("material_table_profiler"));
    assert!(!output.contains("skill"));
}

#[test]
fn filters_runtime_skill_event_lines_from_answer() {
    let raw = "当前只掌握商品名称和售价，先补成本、平台和用户反馈。\nskill:\
               learning_teaching_explain status=success \
               summary=教学解释已生成。\n用户本轮手动选择的预设商品资料：千乐咖啡（已选商品，来源 \
               context.selected_summary）。\n本轮未检测到 \
               context.selected_summary；不注入任何预设商品资料，只按用户文本、\
               会话历史和工作区上下文理解问题。\n后续再做毛利核算和转化分析。";

    let output = sanitize_final_user_facing_text(raw).expect("answer should remain readable");
    let lower = output.to_ascii_lowercase();
    assert!(output.contains("当前只掌握商品名称和售价"));
    assert!(output.contains("本轮已使用你选中的商品资料：千乐咖啡。"));
    assert!(output.contains("本轮没有手动选择具体商品"));
    assert!(output.contains("后续再做毛利核算"));
    assert!(!lower.contains("skill:"));
    assert!(!lower.contains("status="));
    assert!(!lower.contains("context.selected_summary"));
}

#[test]
fn final_sanitizing_structures_review_answers_by_configured_sections() {
    let raw = "优化建议：先把讲解脚本改成痛点、证据、权益三段，并把客服问题提前写进直播间贴片。\n\
               数据表现：GMV 18000 元，成交转化率 4.5%，退款率 8%，直播数据需要继续按同口径记录。\n\
               复盘结论：这场直播可以小步放量，但不能直接翻倍预算。\n\
               sourceMap={\"internal\":\"debug\"}\n\
               复盘字段：曝光、点击、停留、咨询、加购、成交、GMV、退款率、差评原因。\n\
               问题诊断：主要问题不是流量不足，而是讲解后半段成交承接弱。\n\
               优先级：P0 先降退款，P1 优化成交话术，P2 再测试新素材。";

    let output = sanitize_final_user_facing_text(raw).expect("review answer should remain");
    let lower = output.to_ascii_lowercase();

    let expected_order = [
        "## 结论摘要",
        "## 直播数据结论",
        "## 经营问题诊断",
        "## 整改优先级",
        "## 落地优化建议",
        "## 下次复盘字段",
    ];
    let mut cursor = 0;
    for heading in expected_order {
        let index = output[cursor..]
            .find(heading)
            .unwrap_or_else(|| panic!("missing heading {heading}: {output}"));
        cursor += index + heading.len();
    }

    assert!(output.contains("这场直播可以小步放量"));
    assert!(output.contains("GMV 18000 元"));
    assert!(output.contains("成交承接弱"));
    assert!(output.contains("P0 先降退款"));
    assert!(output.contains("讲解脚本"));
    assert!(output.contains("曝光、点击、停留"));
    assert!(!lower.contains("sourcemap"));
    assert!(!lower.contains("internal"));
}
