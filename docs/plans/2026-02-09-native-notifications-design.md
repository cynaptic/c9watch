# Native Notifications for Session Status Changes

## Overview

Add macOS native notifications when a Claude Code session needs attention. Two triggers:

1. **Needs Permission** — A session transitions from `Working` to `NeedsPermission`. Notification includes which tool needs approval.
2. **Work Complete** — A session transitions from `Working` to `WaitingForInput`. Notification tells the user Claude finished.

Clicking a notification focuses the terminal/IDE window where that session is running.

Notifications are on by default. No settings UI for now.

## Notification Content

**Title:** Truncated `first_prompt` (the session's task summary, already on the `Session` struct).

**Body:**
- Permission: `"{projectName}: Needs permission for {toolName}"`
- Complete: `"{projectName}: Finished working"`

## Design

### Status transition detection (polling.rs)

The polling loop runs every 2 seconds. Currently it has no memory between cycles.

Add a `HashMap<String, SessionStatus>` to track each session's previous status. After enriching sessions each cycle, compare new vs previous:

- `Working` → `NeedsPermission` → fire "needs permission" notification
- `Working` → `WaitingForInput` → fire "work complete" notification
- All other transitions → no notification

Edge cases:
- **First poll cycle:** Seed the map without sending notifications (avoids spam on app launch).
- **Session disappears:** Remove from map. No notification.
- **Deduplication:** Only fires on transitions, not repeated polls with same status.

### Pending tool name extraction (session/status.rs)

Add `get_pending_tool_name(entries: &[SessionEntry]) -> Option<String>` that finds the first unapproved `ToolUse` in the last assistant message. Reuses the same logic `are_pending_tools_auto_approved` already walks.

Add `pending_tool_name: Option<String>` to the `Session` struct in `polling.rs`, populated during enrichment.

### Native notifications (lib.rs + polling.rs)

Use `tauri-plugin-notification` crate. Register in `lib.rs` setup.

Fire notifications from the polling loop via `AppHandle`:

```rust
use tauri_plugin_notification::NotificationExt;

app_handle.notification()
    .builder()
    .title(&truncated_title)
    .body(&body_text)
    .show();
```

### Click-to-focus (lib.rs)

Attach session `pid` and `project_path` as the notification identifier. Register a notification event handler in `lib.rs` setup that parses the identifier and calls `open_session(pid, project_path)`.

## Changes

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add `tauri-plugin-notification = "2"` |
| `src-tauri/capabilities/default.json` | Add `"notification:default"` permission |
| `src-tauri/src/lib.rs` | Register notification plugin, register click handler |
| `src-tauri/src/polling.rs` | Track previous statuses, detect transitions, fire notifications, add `pending_tool_name` to `Session` |
| `src-tauri/src/session/status.rs` | Add `get_pending_tool_name()` helper |
| `src/lib/types.ts` | Add `pendingToolName: string \| null` to `Session` interface |

## Out of scope

- Settings UI / notification toggle
- Windows support
- Notification sound customization
- Frontend UI changes beyond the type update
