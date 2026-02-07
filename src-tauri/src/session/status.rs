use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::OnceLock;
use super::parser::{SessionEntry, MessageContent, AssistantMessage};
use super::permissions::PermissionChecker;

/// Global permission checker (loaded once from settings)
static PERMISSION_CHECKER: OnceLock<PermissionChecker> = OnceLock::new();

fn get_permission_checker() -> &'static PermissionChecker {
    PERMISSION_CHECKER.get_or_init(PermissionChecker::from_settings_file)
}

/// Represents the current status of a Claude Code session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SessionStatus {
    /// Claude is actively executing tools or thinking
    Working,

    /// Waiting for user approval to execute tools
    NeedsPermission,

    /// Idle, ready for next prompt
    WaitingForInput,

    /// Session starting up or no recent activity
    Connecting,
}

/// Analyzes session entries to determine the current status
///
/// # Arguments
/// * `entries` - Recent session entries (typically last 10-20 entries)
///
/// # Returns
/// The determined session status
pub fn determine_status(entries: &[SessionEntry]) -> SessionStatus {
    // If no entries or very few entries, session is likely starting up
    if entries.is_empty() {
        return SessionStatus::Connecting;
    }

    // Get the last entry
    let last_entry = &entries[entries.len() - 1];

    match last_entry {
        SessionEntry::User { .. } => {
            // User just sent a message, Claude should be responding
            SessionStatus::Working
        }
        SessionEntry::Assistant { base, message } => {
            // Get the raw status from analyzing the message
            // This now uses PermissionChecker to determine if tools need permission
            let raw_status = analyze_assistant_message(message);

            // If status is Working (auto-approved tools running), apply a brief
            // grace period in case the tool completes very quickly
            if raw_status == SessionStatus::Working {
                // Check if there are pending tool uses
                let has_pending_tools = has_pending_tool_uses(&message.content);
                if has_pending_tools && !is_entry_recent(&base.timestamp, 2) {
                    // Auto-approved tool running for > 2 seconds - still Working
                    // (this is normal for slower operations like file reads)
                    SessionStatus::Working
                } else {
                    raw_status
                }
            } else {
                // NeedsPermission, WaitingForInput, etc. - return immediately
                // No delay for permission prompts!
                raw_status
            }
        }
        _ => {
            // For file history snapshots, summaries, or unknown entries,
            // default to waiting for input
            SessionStatus::WaitingForInput
        }
    }
}

/// Checks if a timestamp is within the last N seconds
fn is_entry_recent(timestamp: &str, seconds: i64) -> bool {
    if let Ok(entry_time) = DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        let age = now.signed_duration_since(entry_time.with_timezone(&Utc));
        age.num_seconds() < seconds
    } else {
        // If we can't parse the timestamp, assume it's not recent
        false
    }
}

/// Analyzes an assistant message to determine status
fn analyze_assistant_message(message: &AssistantMessage) -> SessionStatus {
    // Check if the message contains any tool uses
    let has_tool_use = message.content.iter().any(|content| {
        matches!(content, MessageContent::ToolUse { .. })
    });

    if has_tool_use {
        // Check if all tool uses have corresponding results
        let all_tools_completed = check_all_tools_completed(&message.content);

        if all_tools_completed {
            // All tools completed - check stop reason to determine next state
            match message.stop_reason.as_deref() {
                Some("end_turn") => SessionStatus::WaitingForInput,
                Some("tool_use") => {
                    // This shouldn't happen if tools are completed, but if it does,
                    // the message is likely still being processed
                    SessionStatus::Working
                }
                Some("max_tokens") | Some("stop_sequence") => SessionStatus::WaitingForInput,
                _ => SessionStatus::WaitingForInput,
            }
        } else {
            // Tool use present but not all completed
            // Check if pending tools are auto-approved
            if are_pending_tools_auto_approved(&message.content) {
                // All pending tools will be auto-approved, so status is Working
                SessionStatus::Working
            } else {
                // At least one pending tool needs user permission
                SessionStatus::NeedsPermission
            }
        }
    } else {
        // No tool use, just text/thinking content
        match message.stop_reason.as_deref() {
            Some("end_turn") => SessionStatus::WaitingForInput,
            Some("max_tokens") | Some("stop_sequence") => SessionStatus::WaitingForInput,
            None => {
                // Still generating if no stop reason
                SessionStatus::Working
            }
            _ => SessionStatus::WaitingForInput,
        }
    }
}

/// Checks if all pending (incomplete) tool uses are auto-approved
fn are_pending_tools_auto_approved(content: &[MessageContent]) -> bool {
    let checker = get_permission_checker();

    // Get IDs of tools that have results
    let completed_ids: Vec<&str> = content
        .iter()
        .filter_map(|c| {
            if let MessageContent::ToolResult { tool_use_id, .. } = c {
                Some(tool_use_id.as_str())
            } else {
                None
            }
        })
        .collect();

    // Check each tool use - if it's pending (no result), check if auto-approved
    for item in content {
        if let MessageContent::ToolUse { id, name, input } = item {
            // Skip if already completed
            if completed_ids.contains(&id.as_str()) {
                continue;
            }

            // This tool is pending - check if it's auto-approved
            if !checker.is_auto_approved(name, input) {
                // Found a tool that needs permission
                return false;
            }
        }
    }

    // All pending tools are auto-approved
    true
}

/// Checks if there are any pending (incomplete) tool uses
fn has_pending_tool_uses(content: &[MessageContent]) -> bool {
    !check_all_tools_completed(content)
}

/// Checks if all tool uses in the content have corresponding tool results
fn check_all_tools_completed(content: &[MessageContent]) -> bool {
    // Collect all tool use IDs
    let mut tool_use_ids = Vec::new();
    for item in content {
        if let MessageContent::ToolUse { id, .. } = item {
            tool_use_ids.push(id.clone());
        }
    }

    // If no tool uses, return true (nothing to check)
    if tool_use_ids.is_empty() {
        return true;
    }

    // Collect all tool result IDs
    let mut tool_result_ids = Vec::new();
    for item in content {
        if let MessageContent::ToolResult { tool_use_id, .. } = item {
            tool_result_ids.push(tool_use_id.clone());
        }
    }

    // Check if all tool use IDs have corresponding results
    for tool_id in &tool_use_ids {
        if !tool_result_ids.contains(tool_id) {
            return false;
        }
    }

    true
}

/// Determines status with additional context from multiple entries
///
/// This function looks at the last few entries to get more context about
/// the session state, which can be more accurate than just looking at the
/// last entry alone.
pub fn determine_status_with_context(entries: &[SessionEntry]) -> SessionStatus {
    if entries.is_empty() {
        return SessionStatus::Connecting;
    }

    // For minimal context, treat as connecting
    if entries.len() <= 2 {
        return SessionStatus::Connecting;
    }

    // Get the basic status from the last entry
    let basic_status = determine_status(entries);

    // If we detect Working status, but the previous entry was also an assistant
    // message with completed tools, we might actually be waiting for input
    if basic_status == SessionStatus::Working && entries.len() >= 2 {
        let last_entry = &entries[entries.len() - 1];
        let prev_entry = &entries[entries.len() - 2];

        if let (
            SessionEntry::Assistant { message: last_msg, .. },
            SessionEntry::Assistant { .. },
        ) = (last_entry, prev_entry) {
            // If both are assistant messages and last one has no tool use,
            // might be waiting for input
            let last_has_tools = last_msg.content.iter().any(|c| {
                matches!(c, MessageContent::ToolUse { .. })
            });

            if !last_has_tools && last_msg.stop_reason.is_some() {
                return SessionStatus::WaitingForInput;
            }
        }
    }

    basic_status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::parser::{SessionEntryBase, UserMessage};

    fn create_base() -> SessionEntryBase {
        SessionEntryBase {
            uuid: "test-uuid".to_string(),
            timestamp: "2026-02-06T12:00:00Z".to_string(),
            session_id: Some("test-session".to_string()),
            cwd: None,
            version: None,
            git_branch: None,
            parent_uuid: None,
            is_sidechain: None,
            slug: None,
        }
    }

    #[test]
    fn test_empty_entries() {
        let entries: Vec<SessionEntry> = vec![];
        assert_eq!(determine_status(&entries), SessionStatus::Connecting);
    }

    #[test]
    fn test_user_message_means_working() {
        let entries = vec![
            SessionEntry::User {
                base: create_base(),
                message: UserMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::Working);
    }

    #[test]
    fn test_assistant_text_completed() {
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::Text {
                            text: "Hello there!".to_string(),
                        }
                    ],
                    stop_reason: Some("end_turn".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::WaitingForInput);
    }

    #[test]
    fn test_assistant_generating() {
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::Text {
                            text: "Thinking...".to_string(),
                        }
                    ],
                    stop_reason: None,
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::Working);
    }

    #[test]
    fn test_tool_use_pending_auto_approved() {
        // Read is auto-approved, so pending Read should be Working
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file.txt"}),
                        }
                    ],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::Working);
    }

    #[test]
    fn test_tool_use_pending_needs_permission() {
        // Bash with unknown command needs permission
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Bash".to_string(),
                            input: serde_json::json!({"command": "rm -rf /some/path"}),
                        }
                    ],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::NeedsPermission);
    }

    #[test]
    fn test_tool_use_completed() {
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file.txt"}),
                        },
                        MessageContent::ToolResult {
                            tool_use_id: "toolu_123".to_string(),
                            content: "File content here".to_string(),
                            is_error: Some(false),
                        }
                    ],
                    stop_reason: Some("end_turn".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::WaitingForInput);
    }

    #[test]
    fn test_multiple_tools_partially_completed_auto_approved() {
        // Both tools are Read (auto-approved), so even partial = Working
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file1.txt"}),
                        },
                        MessageContent::ToolUse {
                            id: "toolu_456".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file2.txt"}),
                        },
                        MessageContent::ToolResult {
                            tool_use_id: "toolu_123".to_string(),
                            content: "File 1 content".to_string(),
                            is_error: Some(false),
                        }
                    ],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::Working);
    }

    #[test]
    fn test_multiple_tools_partially_completed_needs_permission() {
        // One Read (auto) completed, one Bash (needs permission) pending
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file1.txt"}),
                        },
                        MessageContent::ToolUse {
                            id: "toolu_456".to_string(),
                            name: "Bash".to_string(),
                            input: serde_json::json!({"command": "make build"}),
                        },
                        MessageContent::ToolResult {
                            tool_use_id: "toolu_123".to_string(),
                            content: "File 1 content".to_string(),
                            is_error: Some(false),
                        }
                    ],
                    stop_reason: Some("tool_use".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::NeedsPermission);
    }

    #[test]
    fn test_multiple_tools_all_completed() {
        let entries = vec![
            SessionEntry::Assistant {
                base: create_base(),
                message: AssistantMessage {
                    model: "claude-opus-4-5-20251101".to_string(),
                    id: "msg_test".to_string(),
                    role: "assistant".to_string(),
                    content: vec![
                        MessageContent::ToolUse {
                            id: "toolu_123".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file1.txt"}),
                        },
                        MessageContent::ToolUse {
                            id: "toolu_456".to_string(),
                            name: "Read".to_string(),
                            input: serde_json::json!({"file_path": "/test/file2.txt"}),
                        },
                        MessageContent::ToolResult {
                            tool_use_id: "toolu_123".to_string(),
                            content: "File 1 content".to_string(),
                            is_error: Some(false),
                        },
                        MessageContent::ToolResult {
                            tool_use_id: "toolu_456".to_string(),
                            content: "File 2 content".to_string(),
                            is_error: Some(false),
                        }
                    ],
                    stop_reason: Some("end_turn".to_string()),
                    stop_sequence: None,
                    usage: None,
                },
            }
        ];
        assert_eq!(determine_status(&entries), SessionStatus::WaitingForInput);
    }

    #[test]
    fn test_check_all_tools_completed() {
        let content = vec![
            MessageContent::ToolUse {
                id: "toolu_1".to_string(),
                name: "Read".to_string(),
                input: serde_json::json!({}),
            },
            MessageContent::ToolResult {
                tool_use_id: "toolu_1".to_string(),
                content: "result".to_string(),
                is_error: None,
            }
        ];
        assert!(check_all_tools_completed(&content));

        let incomplete_content = vec![
            MessageContent::ToolUse {
                id: "toolu_1".to_string(),
                name: "Read".to_string(),
                input: serde_json::json!({}),
            }
        ];
        assert!(!check_all_tools_completed(&incomplete_content));
    }
}
