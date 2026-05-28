use serde_json::json;

use crate::skills::{SkillCategory, SkillInputField, SkillPriority, SkillSpec};

fn category() -> SkillCategory {
    SkillCategory::Custom("material".to_owned())
}

pub fn specs() -> Vec<SkillSpec> {
    vec![
        material_file_understanding_spec(),
        material_table_profiler_spec(),
        material_image_diagnosis_spec(),
        material_evidence_extractor_spec(),
        material_multi_file_compare_spec(),
    ]
}

pub fn material_file_understanding_spec() -> SkillSpec {
    SkillSpec::new(
        "material_file_understanding",
        "资料理解",
        "理解本轮上传或选中的 PDF、Word、Excel、图片 \
         OCR、文本资料，输出摘要、关键事实、可引用证据、适用场景、缺失信息和风险边界；\
         没有正文时必须明确只看到了元数据。",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "material_name",
        "资料名称、文件名或用户给出的资料标题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "material_type",
        "资料类型：pdf / word / excel / image / text / unknown",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "content_text",
        "文档或文本资料正文；可来自文件抽取、用户粘贴或上下文打包",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "ocr_text",
        "图片 OCR 或视觉预处理得到的文字",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "summary",
        "上游文件抽取、资料库或上下文打包生成的摘要",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "user_question",
        "用户围绕资料提出的问题或本轮任务文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context_refs",
        "本轮显式选中的资料引用，如 conversation_file:{id}、workspace_source:{id}",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "file_count",
        "本轮资料或附件数量",
        json!({"type": "integer", "minimum": 0}),
    ))
    .with_tag("deterministic")
    .with_tag("material_context")
    .with_tag("evidence")
    .with_tag("p0")
}

pub fn material_table_profiler_spec() -> SkillSpec {
    SkillSpec::new(
        "material_table_profiler",
        "表格剖析",
        "剖析 Excel/CSV/表格文本的字段、类型、样例、缺失、异常值、可计算电商指标和后续测算建议，\
         为财务、运营、数据岗位提供稳定结构化输入。",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "table_name",
        "表格名称、文件名或 sheet 名称",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "table_text",
        "CSV、TSV、Markdown 表格或文件抽取后的表格文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "rows",
        "结构化行数据数组；每行可以是对象或数组",
        json!({"type": "array"}),
    ))
    .with_input(SkillInputField::optional(
        "user_question",
        "用户围绕表格提出的问题或本轮任务文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context_refs",
        "本轮显式选中的资料引用，如 conversation_file:{id}",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "max_rows",
        "最多剖析多少行样本，默认 80",
        json!({"type": "integer", "minimum": 1, "maximum": 500}),
    ))
    .with_tag("deterministic")
    .with_tag("material_context")
    .with_tag("table")
    .with_tag("evidence")
    .with_tag("p0")
}

pub fn material_image_diagnosis_spec() -> SkillSpec {
    SkillSpec::new(
        "material_image_diagnosis",
        "图片诊断",
        "基于本轮图片的视觉摘要、OCR、图片说明或上下文元数据，诊断主图、详情页、海报、截图的主体、\
         卖点、文案可读性、价格促销表达、合规风险和转化阻碍；没有视觉摘要或 OCR \
         时必须明确只看到了元数据。",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "image_name",
        "图片名称、文件名或用户给出的图片标题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "image_type",
        "图片类型：main_image / detail_page / poster / screenshot / unknown",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "vision_summary",
        "上游视觉模型或图片预处理得到的画面摘要",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "ocr_text",
        "图片 OCR 识别出的文字",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "image_notes",
        "用户或上游流程补充的图片说明、用途、场景或问题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "user_question",
        "用户围绕图片提出的问题或本轮任务文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context_refs",
        "本轮显式选中的图片或资料引用，如 conversation_file:{id}",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "image_count",
        "本轮图片或图片附件数量",
        json!({"type": "integer", "minimum": 0}),
    ))
    .with_tag("deterministic")
    .with_tag("material_context")
    .with_tag("image")
    .with_tag("evidence")
    .with_tag("p0")
}

pub fn material_evidence_extractor_spec() -> SkillSpec {
    SkillSpec::new(
        "material_evidence_extractor",
        "证据提取",
        "从本轮上传或选中的文档正文、OCR、图片摘要、表格摘要或多资料上下文中抽取可引用证据包，\
         标注证据类型、来源引用、相关度、可信使用方式和缺失风险；没有可读内容时必须返回待补资料。",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "material_name",
        "资料名称、文件名或用户给出的资料标题",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "content_text",
        "文档正文、OCR、上游摘要、表格摘要或多资料合并文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "user_question",
        "用户围绕资料提出的问题或本轮任务文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context_refs",
        "本轮显式选中的资料引用，如 conversation_file:{id}、workspace_source:{id}",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "source_summaries",
        "多资料摘要数组；每项可包含 ref/title/content/summary/text",
        json!({"type": "array"}),
    ))
    .with_input(SkillInputField::optional(
        "max_evidence",
        "最多输出多少条证据，默认 8",
        json!({"type": "integer", "minimum": 1, "maximum": 30}),
    ))
    .with_tag("deterministic")
    .with_tag("material_context")
    .with_tag("evidence")
    .with_tag("citation")
    .with_tag("p0")
}

pub fn material_multi_file_compare_spec() -> SkillSpec {
    SkillSpec::new(
        "material_multi_file_compare",
        "多资料对比",
        "比较本轮多份上传或选中的文档、表格摘要、图片摘要或资料上下文，输出共同点、差异点、\
         冲突点、版本变化、可合并结论和缺失资料；资料少于两份或没有可读内容时必须明确待补充。",
    )
    .with_category(category())
    .with_priority(SkillPriority::Critical)
    .with_input(SkillInputField::optional(
        "materials",
        "多资料数组；每项可包含 ref/title/type/content/summary/text/updated_at/version",
        json!({"type": "array"}),
    ))
    .with_input(SkillInputField::optional(
        "user_question",
        "用户围绕多份资料提出的问题或本轮任务文本",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "context_refs",
        "本轮显式选中的资料引用，如 conversation_file:{id}、workspace_source:{id}",
        json!({"type": "array", "items": {"type": "string"}}),
    ))
    .with_input(SkillInputField::optional(
        "compare_focus",
        "对比重点：conflict / version / metrics / product / risk / general",
        json!({"type": "string"}),
    ))
    .with_input(SkillInputField::optional(
        "max_points",
        "每类最多输出多少条对比点，默认 6",
        json!({"type": "integer", "minimum": 1, "maximum": 20}),
    ))
    .with_tag("deterministic")
    .with_tag("material_context")
    .with_tag("compare")
    .with_tag("evidence")
    .with_tag("p0")
}
