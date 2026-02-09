# Notification Click-to-Focus via URL Scheme

## Overview

Enable clicking on macOS notifications to focus the corresponding Claude Code session's terminal/IDE window. Uses a custom URL scheme (`c9watch://`) embedded in notification text that macOS automatically makes clickable.

## Problem

Tauri v2's notification plugin doesn't support click handlers on macOS desktop. The `onAction()` API is mobile-only. We need a way to make notifications interactive without relying on unsupported APIs.

## Solution

Register a custom URL scheme (`c9watch://`) and embed deep links in notification bodies. When clicked, macOS routes the URL to our app's handler, which calls the existing `open_session(pid, project_path)` function.

## Design

### URL Scheme Registration (tauri.conf.json)

Register `c9watch://` as a custom protocol:

```json
{
  "bundle": {
    "macOS": {
      "deeplink": {
        "protocol": "c9watch"
      }
    }
  }
}
```

This tells macOS to route any `c9watch://` URLs to our app.

### Deep Link Handler (lib.rs)

Use `tauri-plugin-deep-link` to listen for incoming URLs:

```rust
.plugin(tauri_plugin_deep-link::init())
.setup(|app| {
    let app_handle = app.handle().clone();
    tauri_plugin_deep_link::register("c9watch", move |request| {
        // Parse URL: c9watch://open-session?pid=12345&path=/foo/bar
        // Extract pid and path from query parameters
        // Call open_session_action(pid, path)
    })?;

    // ... existing setup code
})
```

The handler:
1. Parses the incoming URL
2. Extracts `pid` and `path` query parameters
3. Calls `open_session_action(pid, path)` to focus the window

### Notification Format (polling.rs)

Modify `fire_notification()` to embed deep links:

**Format (Option B - chosen):**
```
Title: <truncated_first_prompt> - <project_name>
Body:  <status_message>
       c9watch://open-session?pid=<pid>&path=<url_encoded_path>
```

**Example:**
```
Title: Add user authentication - c9watch
Body:  Needs permission for Read
       c9watch://open-session?pid=12345&path=%2FUsers%2Fuser%2Fproject
```

**Implementation:**
```rust
fn fire_notification(...) {
    // Title: prompt + project name
    let title = format!("{} - {}",
        truncate_string(first_prompt, 50),
        project_name
    );

    // Deep link URL with encoded path
    let deep_link = format!(
        "c9watch://open-session?pid={}&path={}",
        pid,
        urlencoding::encode(project_path)
    );

    // Body: status message + clickable link
    let body = match status {
        SessionStatus::NeedsPermission => {
            let tool_name = pending_tool_name.unwrap_or("unknown tool");
            format!("Needs permission for {}\n{}", tool_name, deep_link)
        }
        SessionStatus::WaitingForInput => {
            format!("Finished working\n{}", deep_link)
        }
        _ => return,
    };

    app_handle.notification().builder()
        .title(&title)
        .body(&body)
        .show()
}
```

## Changes

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add `tauri-plugin-deep-link = "2"` and `urlencoding = "2"` |
| `src-tauri/tauri.conf.json` | Add `bundle.macOS.deeplink.protocol = "c9watch"` |
| `src-tauri/src/lib.rs` | Register deep link plugin, add handler to parse URL and call `open_session()` |
| `src-tauri/src/polling.rs` | Update `fire_notification()` to embed deep link URL in body, update title format |

## Trade-offs

**Pros:**
- Works reliably with native macOS notifications
- Reuses existing `open_session()` logic
- No polling or file watching needed
- Minimal code changes

**Cons:**
- URL is visible in notification (not the cleanest UX)
- Requires URL encoding for file paths with spaces/special chars
- Adds ~2 dependencies

## Out of Scope

- Hiding the URL (not possible with macOS native notifications)
- Windows/Linux support (deep link registration differs per platform)
- Action buttons in notifications (mobile-only API)
