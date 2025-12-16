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
