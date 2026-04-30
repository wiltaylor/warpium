use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{anyhow, Context as _};
use command::r#async::Command;
use futures_lite::io::{AsyncBufReadExt, BufReader};
use futures_lite::StreamExt;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum LocalClaudeStreamEvent {
    AssistantText(String),
    ToolUse {
        id: Option<String>,
        name: String,
        input: Value,
    },
    ToolResult {
        id: Option<String>,
        content: String,
        is_error: bool,
    },
    SessionId(String),
    Finished {
        session_id: Option<String>,
        result: Option<String>,
        is_error: bool,
        cost_usd: Option<f64>,
    },
    Error(String),
}

pub async fn run_claude_stream(
    prompt: String,
    working_directory: Option<String>,
    session_id: Option<String>,
    tx: async_channel::Sender<LocalClaudeStreamEvent>,
) {
    if let Err(error) = run_claude_stream_inner(prompt, working_directory, session_id, &tx).await {
        let _ = tx
            .send(LocalClaudeStreamEvent::Error(error.to_string()))
            .await;
    }
}

async fn run_claude_stream_inner(
    prompt: String,
    working_directory: Option<String>,
    session_id: Option<String>,
    tx: &async_channel::Sender<LocalClaudeStreamEvent>,
) -> anyhow::Result<()> {
    let mut command = Command::new("claude");
    command
        .arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--include-partial-messages")
        .arg("--dangerously-skip-permissions")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    if let Some(session_id) = session_id {
        command.arg("--resume").arg(session_id);
    }

    if let Some(working_directory) = working_directory {
        command.current_dir(PathBuf::from(working_directory));
    }

    let mut child = command
        .spawn()
        .context("Failed to start Claude Code. Ensure `claude` is installed and on PATH.")?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Claude stdout was not available"))?;
    let mut lines = BufReader::new(stdout).lines();
    let mut saw_finished_event = false;

    while let Some(line) = lines.next().await.transpose()? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match parse_claude_stream_line(trimmed) {
            Ok(events) => {
                for event in events {
                    saw_finished_event |= matches!(event, LocalClaudeStreamEvent::Finished { .. });
                    let _ = tx.send(event).await;
                }
            }
            Err(error) => {
                log::debug!("Ignoring malformed Claude stream-json line: {error}");
            }
        }
    }

    let status = child.status().await?;
    if !status.success() {
        return Err(anyhow!("Claude exited with status {status}"));
    }
    if !saw_finished_event {
        let _ = tx
            .send(LocalClaudeStreamEvent::Finished {
                session_id: None,
                result: None,
                is_error: false,
                cost_usd: None,
            })
            .await;
    }
    Ok(())
}

fn parse_claude_stream_line(line: &str) -> anyhow::Result<Vec<LocalClaudeStreamEvent>> {
    let value: Value = serde_json::from_str(line)?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut events = Vec::new();

    if let Some(session_id) = value.get("session_id").and_then(Value::as_str) {
        events.push(LocalClaudeStreamEvent::SessionId(session_id.to_owned()));
    }

    match event_type {
        "assistant" => {
            let content = message_content_items(&value);
            let mut text = String::new();
            for item in content {
                match item.get("type").and_then(Value::as_str).unwrap_or_default() {
                    "text" => {
                        if let Some(part) = item.get("text").and_then(Value::as_str) {
                            text.push_str(part);
                        }
                    }
                    "tool_use" => {
                        if let Some(name) = item.get("name").and_then(Value::as_str) {
                            events.push(LocalClaudeStreamEvent::ToolUse {
                                id: item.get("id").and_then(Value::as_str).map(str::to_owned),
                                name: name.to_owned(),
                                input: item.get("input").cloned().unwrap_or(Value::Null),
                            });
                        }
                    }
                    "tool_result" => {
                        events.push(tool_result_event(&item));
                    }
                    _ => {}
                }
            }
            if !text.is_empty() {
                events.push(LocalClaudeStreamEvent::AssistantText(text));
            }
        }
        "user" => {
            for item in message_content_items(&value) {
                if item.get("type").and_then(Value::as_str) == Some("tool_result") {
                    events.push(tool_result_event(&item));
                }
            }
        }
        "result" => {
            events.push(LocalClaudeStreamEvent::Finished {
                session_id: value
                    .get("session_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                result: value
                    .get("result")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                is_error: value
                    .get("is_error")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                cost_usd: value.get("total_cost_usd").and_then(Value::as_f64),
            });
        }
        _ => {}
    }

    Ok(events)
}

fn message_content_items(value: &Value) -> Vec<Value> {
    value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn tool_result_event(item: &Value) -> LocalClaudeStreamEvent {
    LocalClaudeStreamEvent::ToolResult {
        id: item
            .get("tool_use_id")
            .or_else(|| item.get("id"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        content: tool_result_content(item.get("content").unwrap_or(&Value::Null)),
        is_error: item
            .get("is_error")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn tool_result_content(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.get("text")
                    .and_then(Value::as_str)
                    .or_else(|| item.as_str())
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}
