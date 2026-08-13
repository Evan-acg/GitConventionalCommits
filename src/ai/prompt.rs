use super::{Request, ScopeRequest};

/// 组装发送给 LLM 的提示词，与具体 provider 的传输层解耦
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_system(types: &[String], scopes: &[String]) -> String {
        let type_list: String = types
            .iter()
            .map(|t| format!("- {t}\n"))
            .collect();
        let scope_list: String = scopes
            .iter()
            .map(|s| format!("- {s}\n"))
            .collect();
        let scope_list = if scope_list.is_empty() {
            "- General (未配置自定义 Scope，可根据需要自行推断)\n".to_string()
        } else {
            scope_list
        };

        format!(
            r#"你是一个 git commit 消息生成助手。根据以下 git 信息和可用的 type/scope 分类，生成 conventional commit 消息。

可用的 Type:
{type_list}

可用的 Scope:
{scope_list}

输出格式: 始终返回纯 JSON 对象（不要 markdown 代码块，不要额外说明），包含以下字段:
- reason: 分析说明 (中文，说明为何选择单条或多条提交)
- data: commit 消息数组，每个元素包含以下字段:
  - type: 变更类型 (必填，从可用 Type 中选择)
  - scope: 变更范围 (必填，从可用 Scope 中选择；若未列出合适项，可自行推断)
  - message: 中文描述 (一句话概括变更内容)
  - files: 该 commit 涉及的文件路径数组 (需要 git add 的文件)
  - detail: 变更的详细描述 (markdown 列表格式，以 - 开头列出每个具体变更)

规则:
- 根据 diff 内容选择最匹配的 Type 和 Scope
- type 和 scope 为必填字段，不得为空
- 消息用中文描述变更内容
- 分析 diff 内容判断是否需要分多条 commit
- 如果 diff 包含多个独立不相关的变更，为每组独立变更输出一条 commit
- 如果所有变更是相关的、完成单一目标，只输出一条
- 只返回 JSON 对象，不要 markdown 代码块、不要额外说明

示例输出（这是唯一合法格式）:
{{"reason": "本次变更中的修改紧密相关，适合作为单条提交", "data": [{{"type": "Feat", "scope": "Git", "message": "添加新的 git 函数", "files": ["internal/git/git.go"], "detail": "- 新增 StatusShort 函数\n- 新增 DiffStat 函数"}}]}}"#
        )
    }

    pub fn build_user(req: &Request) -> String {
        let mut user_content = "请根据以下 git 信息生成 commit 消息:\n\n".to_string();
        if !req.git_info.is_empty() {
            user_content.push_str(&format!("--- 工作区状态 ---\n{}\n\n", req.git_info));
        }
        user_content.push_str(&format!("--- 完整 diff ---\n{}", req.diff));
        if !req.extra_context.is_empty() {
            user_content.push_str(&format!("\n\n--- 变更上下文 ---\n{}", req.extra_context));
        }
        user_content
    }

    /// 生成 scope 列表的 system 提示词
    pub fn build_scope_system() -> String {
        r#"你是一个项目结构分析助手。根据给定的项目目录结构、git 历史中已使用的 scope 以及现有配置，生成用于 conventional commit 的 scope 分类列表。

输出格式: 始终返回纯 JSON 对象（不要 markdown 代码块，不要额外说明），包含以下字段:
- scopes: scope 列表，每个元素包含:
  - name: scope 名称（优先使用目录名/模块名，保留原大小写，去重）
  - docs: 中文说明（一句话描述该 scope 覆盖的范围）

规则:
- 从项目目录结构中提炼有意义的模块/组件名作为 scope，粒度以顶层或 src 下一层为准
- 合并 git 历史中已使用的 scope（名称相同则保留）
- 保留现有配置中的所有 scope，可补充 docs 但不要改名
- 每个 scope 只出现一次，名称去重（忽略大小写差异时保留一个）
- 通常 5-15 个 scope，避免过细或过粗
- 只返回 JSON 对象，不要 markdown 代码块、不要额外说明"#
            .to_string()
    }

    /// 生成 scope 列表的 user 提示词
    pub fn build_scope_user(req: &ScopeRequest) -> String {
        let mut content = String::new();
        if !req.dirs.is_empty() {
            content.push_str("--- 项目目录结构 ---\n");
            content.push_str(&req.dirs.join("\n"));
            content.push('\n');
        }
        if !req.history_scopes.is_empty() {
            content.push_str("\n--- git 历史中已使用的 scope ---\n");
            content.push_str(&req.history_scopes.join("\n"));
            content.push('\n');
        }
        if !req.existing_yaml.is_empty() {
            content.push_str("\n--- 现有 .git-style-scope.yaml 内容（保留其中条目，不要改名） ---\n");
            content.push_str(&req.existing_yaml);
        }
        if content.is_empty() {
            content.push_str("（无输入，请根据常见项目结构生成合理的 scope 列表）");
        }
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_system_contains_types_and_scopes() {
        let prompt = PromptBuilder::build_system(&["Feat".into()], &["Git".into()]);
        assert!(prompt.contains("- Feat\n"));
        assert!(prompt.contains("- Git\n"));
    }

    #[test]
    fn build_system_empty_scopes_uses_general_hint() {
        let prompt = PromptBuilder::build_system(&["Feat".into()], &[]);
        assert!(prompt.contains("- General"));
    }

    #[test]
    fn build_user_contains_git_info_diff_and_context() {
        let req = Request {
            types: vec![],
            scopes: vec![],
            diff: "diff-content".into(),
            git_info: "status-content".into(),
            extra_context: "context-content".into(),
        };
        let prompt = PromptBuilder::build_user(&req);
        assert!(prompt.contains("diff-content"));
        assert!(prompt.contains("status-content"));
        assert!(prompt.contains("context-content"));
    }
}
