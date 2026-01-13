use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};

pub mod anthropic;
pub mod openai;

#[derive(Debug, Clone)]
pub struct CommitContext {
    pub branch_name: Option<String>,
    pub file_count: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
}

#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub branch_name: Option<String>,
    pub file_count: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub diff_scope: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitMessage {
    #[serde(alias = "type", alias = "commit_type")]
    pub commit_type: String,
    pub scope: Option<String>,
    pub description: String,
    #[serde(default)]
    pub description_en: String, // 英文描述
    #[serde(deserialize_with = "deserialize_body", default)]
    pub body: Option<Vec<String>>, // 改为数组，每个元素是一条说明
    #[serde(default)]
    pub body_en: Option<Vec<String>>, // 英文说明
    #[serde(deserialize_with = "deserialize_breaking_change")]
    pub breaking_change: Option<String>,
}

fn deserialize_body<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Body {
        String(String),
        Array(Vec<String>),
        Null,
    }

    match Body::deserialize(deserializer) {
        Ok(Body::String(s)) => Ok(Some(vec![s])),
        Ok(Body::Array(arr)) => Ok(Some(arr)),
        Ok(Body::Null) => Ok(None),
        Err(e) => Err(e),
    }
}

fn deserialize_breaking_change<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BreakingChange {
        Bool(bool),
        String(String),
        Null,
    }

    match BreakingChange::deserialize(deserializer) {
        Ok(BreakingChange::Bool(false)) | Ok(BreakingChange::Null) => Ok(None),
        Ok(BreakingChange::Bool(true)) => Ok(Some("Breaking change".to_string())),
        Ok(BreakingChange::String(s)) => Ok(Some(s)),
        Err(e) => Err(e),
    }
}

impl CommitMessage {
    pub fn format_conventional(&self) -> String {
        let mut message = String::new();

        // Header: type(scope): 中文描述
        message.push_str(&self.commit_type);
        if let Some(scope) = &self.scope {
            message.push_str(&format!("({})", scope));
        }
        message.push_str(": ");
        message.push_str(&self.description);
        message.push('\n');
        message.push_str(&self.description_en);

        // Body - 双语格式
        if let (Some(body_zh), Some(body_en)) = (&self.body, &self.body_en) {
            if !body_zh.is_empty() || !body_en.is_empty() {
                message.push_str("\n\n");

                // 使用较长的数组长度，确保所有内容都被包含
                let max_len = body_zh.len().max(body_en.len());

                for i in 0..max_len {
                    if i > 0 {
                        message.push('\n');
                    }

                    // 安全获取中文内容
                    if let Some(zh) = body_zh.get(i) {
                        message.push_str(zh);
                        message.push('\n');
                    }

                    // 安全获取英文内容
                    if let Some(en) = body_en.get(i) {
                        message.push_str(en);
                    } else if body_zh.get(i).is_some() {
                        // 如果有中文但没有对应英文，添加占位符
                        message.push_str("[Translation needed]");
                    }
                }
            }
        }

        // Breaking change
        if let Some(breaking) = &self.breaking_change {
            message.push_str("\n\n");
            message.push_str("BREAKING CHANGE: ");
            message.push_str(breaking);
        }

        message
    }
}

pub enum AIClient {
    OpenAI(openai::OpenAIClient),
    Anthropic(anthropic::AnthropicClient),
}

impl AIClient {
    pub async fn generate_commit_message(
        &self,
        diff: &str,
        context: &CommitContext,
        debug: bool,
    ) -> Result<CommitMessage> {
        match self {
            AIClient::OpenAI(client) => client.generate_commit_message(diff, context, debug).await,
            AIClient::Anthropic(client) => {
                client.generate_commit_message(diff, context, debug).await
            }
        }
    }

    pub async fn generate_review_report(
        &self,
        diff: &str,
        context: &ReviewContext,
        debug: bool,
    ) -> Result<ReviewReport> {
        match self {
            AIClient::OpenAI(client) => client.generate_review_report(diff, context, debug).await,
            AIClient::Anthropic(client) => {
                client.generate_review_report(diff, context, debug).await
            }
        }
    }

    pub async fn generate_changelog(
        &self,
        commits: &[crate::git::CommitInfo],
        context: &ChangelogContext,
        debug: bool,
    ) -> Result<ChangelogSummary> {
        match self {
            AIClient::OpenAI(client) => client.generate_changelog(commits, context, debug).await,
            AIClient::Anthropic(client) => client.generate_changelog(commits, context, debug).await,
        }
    }
}

pub fn create_client(
    provider: &str,
    api_key: String,
    model: String,
    base_url: Option<String>,
    max_tokens: u32,
) -> Result<AIClient> {
    match provider.to_lowercase().as_str() {
        "openai" => Ok(AIClient::OpenAI(openai::OpenAIClient::new(
            api_key, model, base_url, max_tokens,
        ))),
        "anthropic" => Ok(AIClient::Anthropic(anthropic::AnthropicClient::new(
            api_key, model, base_url, max_tokens,
        ))),
        _ => anyhow::bail!("Unsupported AI provider: {}", provider),
    }
}

pub fn build_prompt(diff: &str, context: &CommitContext) -> String {
    format!(
        r#"You are a Git commit message generator. Based on the following git diff, generate a bilingual (Chinese and English) structured commit message.

Context:
- Branch: {}
- Files changed: {}
- Lines added: {}
- Lines removed: {}

Git Diff:
```
{}
```

Generate a commit message following the Conventional Commits specification with bilingual format:
- type: feat, fix, docs, style, refactor, test, chore, perf
- scope: optional, the component or area affected
- description: 中文简要描述（50字符以内）
- description_en: English brief description (50 chars or less)
- body: 中文详细说明数组，每个元素是一条说明（如："添加了用户认证功能"、"优化了数据库查询性能"）
- body_en: English detailed explanation array, each element corresponds to Chinese version
- breaking_change: optional, if there are breaking changes

Important requirements:
1. description should be in Chinese, description_en should be its English translation
2. body and body_en should be arrays of strings, each element is one point
3. Each Chinese point in body should have a corresponding English translation in body_en
4. Keep descriptions concise and clear

Respond with a JSON object containing these fields. Example:
{{
    "type": "feat",
    "scope": "auth",
    "description": "添加用户认证功能",
    "description_en": "Add user authentication feature",
    "body": ["实现了JWT令牌验证", "添加了用户登录接口", "集成了OAuth2.0支持"],
    "body_en": ["Implement JWT token validation", "Add user login endpoint", "Integrate OAuth2.0 support"],
    "breaking_change": null
}}
"#,
        context.branch_name.as_deref().unwrap_or("unknown"),
        context.file_count,
        context.added_lines,
        context.removed_lines,
        truncate_diff(diff, 3000)
    )
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewReport {
    pub verdict: String,
    pub verdict_en: String,
    pub summary: String,
    pub summary_en: String,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub risks_en: Vec<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub suggestions_en: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
    #[serde(default)]
    pub tests_en: Vec<String>,
}

impl ReviewReport {
    pub fn format_display(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "## 结论 / Verdict\n- {} / {}\n\n",
            self.verdict, self.verdict_en
        ));
        output.push_str(&format!(
            "## 摘要 / Summary\n{}\n{}\n\n",
            self.summary, self.summary_en
        ));

        format_bilingual_section(&mut output, "## 风险 / Risks", &self.risks, &self.risks_en);
        format_bilingual_section(
            &mut output,
            "## 建议 / Suggestions",
            &self.suggestions,
            &self.suggestions_en,
        );
        format_bilingual_section(&mut output, "## 测试 / Tests", &self.tests, &self.tests_en);

        output
    }
}

fn format_bilingual_section(output: &mut String, title: &str, zh: &[String], en: &[String]) {
    if zh.is_empty() && en.is_empty() {
        return;
    }
    output.push_str(title);
    output.push('\n');

    let max_len = zh.len().max(en.len());
    for i in 0..max_len {
        let zh_item = zh.get(i).map(|s| s.as_str()).unwrap_or("[Missing]");
        let en_item = en
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("[Missing translation]");
        output.push_str(&format!("- {} / {}\n", zh_item, en_item));
    }
    output.push('\n');
}

pub fn build_review_prompt(diff: &str, context: &ReviewContext) -> String {
    format!(
        r#"You are a senior software engineer doing a code review. Based on the following git diff, generate a bilingual (Chinese and English) review report.

Context:
- Branch: {}
- Diff scope: {}
- Files changed: {}
- Lines added: {}
- Lines removed: {}

Git Diff:
```
{}
```

Focus on:
1) Correctness and potential bugs
2) Edge cases and error handling
3) Security and privacy concerns
4) Performance pitfalls
5) Maintainability and readability
6) Missing or suggested tests

Output requirements:
- Respond with exactly one valid JSON object (no markdown, no commentary).
- Be specific: reference file paths and identifiers when possible.
- Keep each bullet concise.
- For bilingual arrays, keep the same order and meaning in both languages.

JSON schema:
{{
  "verdict": "approve|comment|request_changes",
  "verdict_en": "approve|comment|request_changes",
  "summary": "中文摘要（2-3句）",
  "summary_en": "English summary (2-3 sentences)",
  "risks": ["中文风险点1", "中文风险点2"],
  "risks_en": ["English risk 1", "English risk 2"],
  "suggestions": ["中文建议1", "中文建议2"],
  "suggestions_en": ["English suggestion 1", "English suggestion 2"],
  "tests": ["中文测试建议1", "中文测试建议2"],
  "tests_en": ["English test suggestion 1", "English test suggestion 2"]
}}
"#,
        context.branch_name.as_deref().unwrap_or("unknown"),
        context.diff_scope,
        context.file_count,
        context.added_lines,
        context.removed_lines,
        truncate_diff(diff, 6000)
    )
}

fn truncate_diff(diff: &str, max_chars: usize) -> &str {
    if diff.len() <= max_chars {
        diff
    } else {
        // Find the char boundary at or before max_chars
        let mut boundary = max_chars;
        while !diff.is_char_boundary(boundary) && boundary > 0 {
            boundary -= 1;
        }
        &diff[..boundary]
    }
}

// Changelog generation types and functions

#[derive(Debug, Clone)]
pub struct ChangelogContext {
    pub total_commits: usize,
    pub date_range: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangelogSummary {
    pub title: String,
    pub title_en: String,
    pub highlights: Vec<String>,
    pub highlights_en: Vec<String>,
    pub categories: ChangelogCategories,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChangelogCategories {
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub fixes: Vec<String>,
    #[serde(default)]
    pub improvements: Vec<String>,
    #[serde(default)]
    pub others: Vec<String>,
}

impl ChangelogSummary {
    pub fn format_display(&self) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("## {}\n", self.title));
        output.push_str(&format!("## {}\n\n", self.title_en));

        // Highlights
        if !self.highlights.is_empty() {
            output.push_str("### 亮点 / Highlights\n");
            for (zh, en) in self.highlights.iter().zip(self.highlights_en.iter()) {
                output.push_str(&format!("- {} / {}\n", zh, en));
            }
            output.push('\n');
        }

        // Categories
        if !self.categories.features.is_empty() {
            output.push_str("### ✨ 新功能 / Features\n");
            for item in &self.categories.features {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        if !self.categories.fixes.is_empty() {
            output.push_str("### 🐛 修复 / Fixes\n");
            for item in &self.categories.fixes {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        if !self.categories.improvements.is_empty() {
            output.push_str("### 🔧 改进 / Improvements\n");
            for item in &self.categories.improvements {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        if !self.categories.others.is_empty() {
            output.push_str("### 📝 其他 / Others\n");
            for item in &self.categories.others {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        output
    }
}

pub fn build_changelog_prompt(
    commits: &[crate::git::CommitInfo],
    context: &ChangelogContext,
) -> String {
    let commits_text: String = commits
        .iter()
        .map(|c| {
            format!(
                "- [{}] {} - {} ({})",
                c.short_id,
                c.time.format("%Y-%m-%d"),
                c.summary,
                c.author
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are a changelog summarizer. Based on the following git commits, generate a bilingual (Chinese and English) changelog summary.

Context:
- Total commits: {}
- Date range: {}

Git Commits:
```
{}
```

Generate a changelog summary with the following structure:
- title: 中文标题，简要概括这些提交的主题
- title_en: English title summarizing the theme
- highlights: 中文亮点列表，最重要的2-3个变更
- highlights_en: English highlights corresponding to Chinese
- categories: 按类型分类的变更列表（双语混合格式）
  - features: 新功能列表
  - fixes: 修复列表
  - improvements: 改进列表
  - others: 其他变更

Important:
1. Analyze commit messages to understand the changes
2. Group similar changes together
3. Use clear, concise language
4. Each item in categories should be bilingual format: "中文描述 / English description"

Respond with a JSON object. Example:
{{
    "title": "用户认证与性能优化",
    "title_en": "User Authentication and Performance Optimization",
    "highlights": ["添加了完整的用户认证系统", "优化了数据库查询性能"],
    "highlights_en": ["Added complete user authentication system", "Optimized database query performance"],
    "categories": {{
        "features": ["用户登录功能 / User login feature", "OAuth2.0 支持 / OAuth2.0 support"],
        "fixes": ["修复登录超时问题 / Fix login timeout issue"],
        "improvements": ["优化API响应速度 / Optimize API response speed"],
        "others": ["更新依赖版本 / Update dependencies"]
    }}
}}
"#,
        context.total_commits,
        context.date_range.as_deref().unwrap_or("N/A"),
        commits_text
    )
}
