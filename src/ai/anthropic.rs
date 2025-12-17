use super::{
    build_changelog_prompt, build_prompt, ChangelogContext, ChangelogSummary, CommitContext,
    CommitMessage,
};
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct AnthropicClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
    initial_max_tokens: u32,
}

impl AnthropicClient {
    pub fn new(
        api_key: String,
        model: String,
        base_url: Option<String>,
        initial_max_tokens: u32,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            client,
            initial_max_tokens,
        }
    }

    pub async fn generate_commit_message(
        &self,
        diff: &str,
        context: &CommitContext,
        debug: bool,
    ) -> Result<CommitMessage> {
        let prompt = build_prompt(diff, context);

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: self.initial_max_tokens,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: format!(
                    "{}\n\nPlease respond with only the JSON object, no other text.",
                    prompt
                ),
            }],
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            // Sanitize error message to avoid exposing sensitive details
            let safe_error = match status.as_u16() {
                401 => "Authentication failed. Please check your API key.",
                403 => "Access forbidden. Please check your API permissions.",
                429 => "Rate limit exceeded. Please try again later.",
                500..=599 => "Anthropic service error. Please try again later.",
                _ => "Request failed. Please check your configuration.",
            };

            if debug {
                eprintln!("Debug: Full error response: {}", error_text);
            }

            anyhow::bail!("{} (Status: {})", safe_error, status);
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        if debug {
            println!("\n{}", "=== DEBUG: Raw HTTP Response ===".cyan().bold());
            println!("{}", response_text);
            println!("{}", "=================================\n".cyan().bold());
        }

        let api_response: AnthropicResponse =
            serde_json::from_str(&response_text).context("Failed to parse Anthropic response")?;

        let content = api_response
            .content
            .first()
            .ok_or_else(|| anyhow::anyhow!("No response from Anthropic"))?
            .text
            .clone();

        if debug {
            println!("\n{}", "=== DEBUG: AI Message Content ===".cyan().bold());
            println!("{}", content);
            println!("{}", "==================================\n".cyan().bold());
        }

        // Strip markdown code block wrapper if present
        let clean_content = if content.starts_with("```json") && content.ends_with("```") {
            content
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(&content)
        } else if content.starts_with("```") && content.ends_with("```") {
            content
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(&content)
        } else {
            &content
        };

        // Try to parse the content directly first
        let commit_message = match serde_json::from_str::<CommitMessage>(clean_content) {
            Ok(msg) => msg,
            Err(_) => {
                // If direct parsing fails, try to extract JSON object
                // This is more robust than simple string searching
                let mut depth = 0;
                let mut start_idx = None;
                let mut end_idx = None;

                for (idx, ch) in clean_content.char_indices() {
                    match ch {
                        '{' => {
                            if depth == 0 && start_idx.is_none() {
                                start_idx = Some(idx);
                            }
                            depth += 1;
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 && start_idx.is_some() {
                                end_idx = Some(idx + ch.len_utf8());
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(start), Some(end)) = (start_idx, end_idx) {
                    let json_str = &clean_content[start..end];
                    serde_json::from_str(json_str)
                        .context("Failed to parse extracted JSON from Anthropic response")?
                } else {
                    anyhow::bail!("No valid JSON object found in Anthropic response");
                }
            }
        };

        Ok(commit_message)
    }

    pub async fn generate_changelog(
        &self,
        commits: &[crate::git::CommitInfo],
        context: &ChangelogContext,
        debug: bool,
    ) -> Result<ChangelogSummary> {
        let prompt = build_changelog_prompt(commits, context);

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: self.initial_max_tokens,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: format!(
                    "{}\n\nPlease respond with only the JSON object, no other text.",
                    prompt
                ),
            }],
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            let safe_error = match status.as_u16() {
                401 => "Authentication failed. Please check your API key.",
                403 => "Access forbidden. Please check your API permissions.",
                429 => "Rate limit exceeded. Please try again later.",
                500..=599 => "Anthropic service error. Please try again later.",
                _ => "Request failed. Please check your configuration.",
            };

            if debug {
                eprintln!("Debug: Full error response: {}", error_text);
            }

            anyhow::bail!("{} (Status: {})", safe_error, status);
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        if debug {
            println!("\n{}", "=== DEBUG: Raw HTTP Response ===".cyan().bold());
            println!("{}", response_text);
            println!("{}", "=================================\n".cyan().bold());
        }

        let api_response: AnthropicResponse =
            serde_json::from_str(&response_text).context("Failed to parse Anthropic response")?;

        let content = api_response
            .content
            .first()
            .ok_or_else(|| anyhow::anyhow!("No response from Anthropic"))?
            .text
            .clone();

        if debug {
            println!("\n{}", "=== DEBUG: AI Message Content ===".cyan().bold());
            println!("{}", content);
            println!("{}", "==================================\n".cyan().bold());
        }

        // Strip markdown code block wrapper if present
        let clean_content = if content.starts_with("```json") && content.ends_with("```") {
            content
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(&content)
        } else if content.starts_with("```") && content.ends_with("```") {
            content
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim())
                .unwrap_or(&content)
        } else {
            &content
        };

        let changelog = match serde_json::from_str::<ChangelogSummary>(clean_content) {
            Ok(summary) => summary,
            Err(_) => {
                // Try to extract JSON object
                let mut depth = 0;
                let mut start_idx = None;
                let mut end_idx = None;

                for (idx, ch) in clean_content.char_indices() {
                    match ch {
                        '{' => {
                            if depth == 0 && start_idx.is_none() {
                                start_idx = Some(idx);
                            }
                            depth += 1;
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 && start_idx.is_some() {
                                end_idx = Some(idx + ch.len_utf8());
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if let (Some(start), Some(end)) = (start_idx, end_idx) {
                    let json_str = &clean_content[start..end];
                    serde_json::from_str(json_str)
                        .context("Failed to parse extracted JSON from Anthropic response")?
                } else {
                    anyhow::bail!("No valid JSON object found in Anthropic response");
                }
            }
        };

        Ok(changelog)
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}
