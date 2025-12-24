use super::{
    build_changelog_prompt, build_prompt, ChangelogContext, ChangelogSummary, CommitContext,
    CommitMessage,
};
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SSE streaming response chunk structure
#[derive(Deserialize, Debug)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize, Debug)]
struct StreamChoice {
    delta: StreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct StreamDelta {
    content: Option<String>,
}

/// Parse SSE streaming response and concatenate all content chunks
fn parse_streaming_response(response_text: &str) -> Option<String> {
    let mut content = String::new();
    let mut is_streaming = false;

    for line in response_text.lines() {
        let line = line.trim();

        // Check if this looks like SSE format
        if line.starts_with("data: ") {
            is_streaming = true;
            let data = line.strip_prefix("data: ").unwrap_or("");

            // Skip [DONE] marker
            if data == "[DONE]" {
                continue;
            }

            // Try to parse as streaming chunk
            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                for choice in chunk.choices {
                    if let Some(text) = choice.delta.content {
                        content.push_str(&text);
                    }
                }
            }
        }
    }

    if is_streaming && !content.is_empty() {
        Some(content)
    } else {
        None
    }
}

pub struct OpenAIClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
    initial_max_tokens: u32,
}

impl OpenAIClient {
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
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
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

        let mut max_tokens = self.initial_max_tokens;
        let max_attempts = 4;

        for attempt in 0..max_attempts {
            let mut messages = vec![Message {
                role: "system".to_string(),
                content: "You are a helpful assistant that generates git commit messages in JSON format. Reply with exactly one valid, minified JSON object.".to_string(),
            }];

            if attempt > 0 {
                messages.push(Message {
                    role: "system".to_string(),
                    content: "Your previous answer was truncated. Send the complete JSON object this time, keep it under 600 characters, and avoid any commentary or markdown fences.".to_string(),
                });
            }

            messages.push(Message {
                role: "user".to_string(),
                content: prompt.clone(),
            });

            let request = OpenAIRequest {
                model: self.model.clone(),
                messages,
                temperature: 0.7,
                max_tokens,
                response_format: Some(ResponseFormat {
                    type_field: "json_object".to_string(),
                }),
            };

            let response = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
                .context("Failed to send request to OpenAI")?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await?;

                // Sanitize error message to avoid exposing sensitive details
                let safe_error = match status.as_u16() {
                    401 => "Authentication failed. Please check your API key.",
                    403 => "Access forbidden. Please check your API permissions.",
                    429 => "Rate limit exceeded. Please try again later.",
                    500..=599 => "OpenAI service error. Please try again later.",
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

            // Try to detect and parse streaming response first
            let (content, finish_reason) =
                if let Some(streamed_content) = parse_streaming_response(&response_text) {
                    if debug {
                        println!(
                            "{}",
                            "=== DEBUG: Detected SSE streaming response ==="
                                .cyan()
                                .bold()
                        );
                    }
                    // For streaming responses, we assume completion when [DONE] is received
                    (streamed_content, Some("stop".to_string()))
                } else {
                    // Parse as standard OpenAI response
                    let api_response: OpenAIResponse = serde_json::from_str(&response_text)
                        .context("Failed to parse OpenAI response")?;

                    let choice = api_response
                        .choices
                        .first()
                        .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

                    let content = choice
                        .message
                        .content
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("Response content is null"))?;

                    (content, choice.finish_reason.clone())
                };

            if debug {
                println!("\n{}", "=== DEBUG: AI Message Content ===".cyan().bold());
                println!("{}", content);
                println!("{}", "==================================\n".cyan().bold());
            }

            match finish_reason.as_deref() {
                Some("length") => {
                    if content.trim().is_empty() && attempt + 1 == max_attempts {
                        anyhow::bail!("AI response was truncated repeatedly, resulting in empty content. Try reducing the diff size or switching models.");
                    }

                    if attempt + 1 < max_attempts {
                        max_tokens = (max_tokens.saturating_mul(2)).min(4000);
                        if debug {
                            println!(
                                "{}",
                                format!(
                                    "=== DEBUG: finish_reason=length, retrying with max_tokens={} ===",
                                    max_tokens
                                )
                                .cyan()
                                .bold()
                            );
                        }
                        continue;
                    } else {
                        anyhow::bail!("AI response was truncated before completing the JSON (finish_reason=length). Try reducing the diff size or switching models.");
                    }
                }
                Some("content_filter") => {
                    anyhow::bail!("The response was blocked by the provider's content filter.");
                }
                Some("stop") | Some("stop_sequence") | None => {}
                Some(other) => {
                    anyhow::bail!("Unexpected finish_reason '{}' from AI response.", other);
                }
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

            let commit_message = match serde_json::from_str::<CommitMessage>(clean_content) {
                Ok(msg) => msg,
                Err(primary_err) => {
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
                                if depth > 0 {
                                    depth -= 1;
                                }
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
                        serde_json::from_str::<CommitMessage>(json_str).with_context(|| {
                            format!(
                                "Failed to parse extracted JSON from OpenAI response: {}",
                                primary_err
                            )
                        })?
                    } else {
                        return Err(anyhow::anyhow!(
                            "Failed to parse commit message from OpenAI response: {}",
                            primary_err
                        ));
                    }
                }
            };

            return Ok(commit_message);
        }

        anyhow::bail!(
            "Failed to obtain a valid commit message from OpenAI after multiple attempts"
        );
    }

    pub async fn generate_changelog(
        &self,
        commits: &[crate::git::CommitInfo],
        context: &ChangelogContext,
        debug: bool,
    ) -> Result<ChangelogSummary> {
        let prompt = build_changelog_prompt(commits, context);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant that generates changelog summaries in JSON format. Reply with exactly one valid, minified JSON object.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
            max_tokens: self.initial_max_tokens,
            response_format: Some(ResponseFormat {
                type_field: "json_object".to_string(),
            }),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            let safe_error = match status.as_u16() {
                401 => "Authentication failed. Please check your API key.",
                403 => "Access forbidden. Please check your API permissions.",
                429 => "Rate limit exceeded. Please try again later.",
                500..=599 => "OpenAI service error. Please try again later.",
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

        // Try to detect and parse streaming response first
        let content = if let Some(streamed_content) = parse_streaming_response(&response_text) {
            if debug {
                println!(
                    "{}",
                    "=== DEBUG: Detected SSE streaming response ==="
                        .cyan()
                        .bold()
                );
            }
            streamed_content
        } else {
            // Parse as standard OpenAI response
            let api_response: OpenAIResponse =
                serde_json::from_str(&response_text).context("Failed to parse OpenAI response")?;

            let choice = api_response
                .choices
                .first()
                .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

            choice
                .message
                .content
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Response content is null"))?
        };

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
            Err(primary_err) => {
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
                            if depth > 0 {
                                depth -= 1;
                            }
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
                    serde_json::from_str::<ChangelogSummary>(json_str).with_context(|| {
                        format!(
                            "Failed to parse extracted JSON from OpenAI response: {}",
                            primary_err
                        )
                    })?
                } else {
                    return Err(anyhow::anyhow!(
                        "Failed to parse changelog from OpenAI response: {}",
                        primary_err
                    ));
                }
            }
        };

        Ok(changelog)
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    type_field: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_streaming_response() {
        let sse_response = r#"data: {"id":"chatcmpl-1766389658588","object":"chat.completion.chunk","created":1766389658,"model":"gemini-3-flash","choices":[{"index":0,"delta":{"content":"{\"type\":\"feat\",\"scope\":\"excel\",\"description\":\"新增资金调节表特殊"},"finish_reason":null}]}

data: {"id":"chatcmpl-1766389658588","object":"chat.completion.chunk","created":1766389658,"model":"gemini-3-flash","choices":[{"index":0,"delta":{"content":"识别逻辑\",\"description_en\":\"Add special identification logic\"}"},"finish_reason":null}]}

data: {"id":"chatcmpl-1766389658588","object":"chat.completion.chunk","created":1766389658,"model":"gemini-3-flash","choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":1404,"completion_tokens":107,"total_tokens":1511}}

data: [DONE]"#;

        let result = parse_streaming_response(sse_response);
        assert!(result.is_some());
        let content = result.unwrap();
        assert!(content.contains(r#""type":"feat""#));
        assert!(content.contains(r#""scope":"excel""#));
        assert!(content.contains("新增资金调节表特殊"));
        assert!(content.contains("识别逻辑"));
    }

    #[test]
    fn test_parse_streaming_response_returns_none_for_non_sse() {
        let normal_response =
            r#"{"choices":[{"message":{"content":"hello"},"finish_reason":"stop"}]}"#;
        let result = parse_streaming_response(normal_response);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_streaming_response_handles_empty_deltas() {
        let sse_response = r#"data: {"id":"test","choices":[{"index":0,"delta":{"content":"hello"},"finish_reason":null}]}

data: {"id":"test","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]"#;

        let result = parse_streaming_response(sse_response);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "hello");
    }
}
