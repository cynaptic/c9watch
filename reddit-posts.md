# Reddit Post Templates for c9watch

## Main Template (Detailed Version)

**Title:** Built c9watch - a dashboard to monitor Claude Code sessions across multiple projects [Open Source]

**Body:**

I run about 10 Claude Code sessions simultaneously across different projects. Two problems kept coming up:

1. I had to constantly switch between terminals to check if sessions were blocked on permissions or if they'd finished their tasks.

2. Other monitoring tools exist, but they require launching sessions from within their app. I didn't want to change my workflow - I just wanted to keep using Claude Code in Zed and Ghostty like I already do.

So I built c9watch. It's a lightweight, open source macOS dashboard that monitors all your sessions without touching your workflow.

**What it does:**

- Auto-discovers sessions by scanning running processes - works with any terminal or IDE
- Shows real-time status: Working, Needs Permission, or Idle
- Multiple views: Status, Project, or Compact
- Conversation viewer to inspect any session
- Permission requests surface to the top automatically
- Session controls: stop, open parent terminal, rename

**How it works:**

Polls every 2 seconds, scans for running claude processes, matches them to session files in ~/.claude/projects/, and parses JSONL to determine status. Built with Tauri, Rust, and Svelte.

**Demo:** [link]

**Download:** [link]
**Source:** [link]

MIT licensed and fully open source. Would appreciate feedback from anyone juggling multiple Claude Code sessions.

---

## Short Template

**Title:** c9watch - Monitor multiple Claude Code sessions without changing your workflow [Open Source]

**Body:**

Built this to solve two problems:

1. Constantly switching terminals to check if sessions were blocked or finished (I run ~10 sessions simultaneously)
2. Existing tools lock you into their app - I wanted to keep using Zed and Ghostty

c9watch monitors all Claude Code sessions by scanning processes. No plugins, no workflow changes.

- Real-time status for each session
- Auto-discovers sessions from any terminal/IDE
- Conversation viewer
- Permission requests surface first
- Built with Tauri + Rust + Svelte

MIT licensed, fully open source.

[Demo] [GitHub]

---

## r/ClaudeAI

**Title:** Built a dashboard to track Claude Code sessions across multiple projects [Open Source]

**Body:**

I run about 10 Claude Code sessions simultaneously across different projects and kept losing track of which ones needed permission or had finished their tasks.

Other monitoring tools require launching sessions from within their app. I didn't want to change my workflow - just wanted to keep using Claude Code in Zed and Ghostty like normal.

Built c9watch to solve this. It's a lightweight, open source macOS dashboard that:

- Auto-discovers all sessions by scanning processes
- Shows real-time status (Working, Needs Permission, Idle)
- Permission requests surface to the top
- Conversation viewer to inspect any session
- Works with any terminal or IDE - no plugins needed

Built with Tauri, Rust, and Svelte. MIT licensed and fully open source.

[Demo] [Download] [GitHub]

Would appreciate feedback from anyone running multiple sessions.

---

## r/ClaudeCode

**Title:** Built a dashboard to track Claude Code sessions across multiple projects [Open Source]

**Body:**

I run about 10 Claude Code sessions simultaneously across different projects and kept losing track of which ones needed permission or had finished their tasks.

Other monitoring tools require launching sessions from within their app. I didn't want to change my workflow - just wanted to keep using Claude Code in Zed and Ghostty like normal.

Built c9watch to solve this. It's a lightweight, open source macOS dashboard that:

- Auto-discovers all sessions by scanning processes
- Shows real-time status (Working, Needs Permission, Idle)
- Permission requests surface to the top
- Conversation viewer to inspect any session
- Works with any terminal or IDE - no plugins needed

Built with Tauri, Rust, and Svelte. MIT licensed and fully open source.

[Demo] [Download] [GitHub]

Would appreciate feedback from anyone running multiple sessions.

---

## r/MacApps

**Title:** c9watch - Dashboard for monitoring Claude Code sessions [Open Source, native macOS]

**Body:**

Built a native, open source macOS app to monitor Claude Code sessions across multiple projects.

The problem: running ~10 sessions simultaneously in Zed and Ghostty, constantly switching terminals to check if they're blocked on permissions or finished.

c9watch auto-discovers all sessions by scanning processes and shows their status in real-time. Works with any terminal or IDE - no plugins or workflow changes needed.

- Multiple views (Status, Project, Compact)
- Conversation viewer
- Session controls (stop, open, rename)
- Built with Tauri + Rust (not Electron)
- Menu bar integration

MIT licensed, fully open source.

[Demo] [Download] [GitHub]

---

## r/commandline

**Title:** c9watch - Monitor multiple claude processes without changing your terminal workflow [Open Source]

**Body:**

Built a dashboard to monitor Claude Code sessions across terminals.

Scans running claude processes every 2s, matches them to session files in ~/.claude/projects/, parses JSONL for status. Shows which sessions are working, blocked on permissions, or idle.

Works with any terminal (I use Ghostty). No plugins. Built with Tauri + Rust + Svelte.

MIT licensed, open source.

[Demo] [GitHub]

---

## r/SideProject ⭐

**Title:** Built c9watch to track my Claude Code sessions across projects [Open Source]

**Body:**

I run about 10 Claude Code sessions simultaneously across different projects in Zed and Ghostty. The problem: constantly switching between terminals to check if sessions were blocked on permissions or had finished.

Other monitoring tools exist but they lock you into launching sessions from their app. I wanted to keep using my own setup.

Built c9watch - a lightweight, open source macOS dashboard that monitors sessions by scanning processes. No plugins, no workflow changes.

Technical challenge was matching running processes to session files and parsing JSONL to determine status. Built with Tauri, Rust, and Svelte.

MIT licensed, fully open source.

[Demo] [Download] [GitHub]

Feedback welcome.

**First comment to post immediately after:**

```
Technical overview:

The app polls every 2 seconds and scans for running claude processes using sysinfo. Each process is matched to its session file in ~/.claude/projects/ through path encoding and timestamp correlation. It parses the last N entries of each session's JSONL file to determine status (Working, Needs Permission, or Idle). Status updates are pushed to the Svelte frontend via Tauri events, and the UI sorts sessions to surface permission requests first.

I built this because other solutions require you to launch sessions from their app. This just monitors - it works with whatever terminal or IDE you're already using.

Happy to answer questions about the implementation.
```

---

## r/programming or r/opensource

**Title:** c9watch - Process-level monitoring for Claude Code sessions (Rust + Tauri) [Open Source]

**Body:**

Built an open source macOS dashboard to monitor Claude Code sessions by scanning processes.

Polls every 2s using sysinfo, matches claude processes to session files in ~/.claude/projects/ via path encoding, parses JSONL to determine status (Working, Needs Permission, Idle). Pushes updates to Svelte frontend via Tauri events.

Works with any terminal or IDE - just scans at the OS level. No plugins required.

Built with Tauri, Rust, Svelte. MIT licensed.

[GitHub] [Demo]

---

## r/Zed

**Title:** Built c9watch to monitor multiple Claude Code sessions in Zed [Open Source]

**Body:**

I run about 10 Claude Code sessions simultaneously in Zed across different projects. Kept losing track of which sessions needed permission or had finished.

Built an open source macOS dashboard that auto-discovers all sessions by scanning processes and shows their real-time status. Works with Zed, Ghostty, or any other terminal/IDE - no plugins needed.

Built with Tauri, Rust, and Svelte. MIT licensed.

[Demo] [GitHub]

---

## General First Comment (Technical)

For technical subreddits (r/programming, r/opensource, r/rust, r/commandline):

```
Technical overview:

The app polls every 2 seconds and scans for running claude processes using sysinfo. Each process is matched to its session file in ~/.claude/projects/ through path encoding and timestamp correlation. It parses the last N entries of each session's JSONL file to determine status (Working, Needs Permission, or Idle). Status updates are pushed to the Svelte frontend via Tauri events, and the UI sorts sessions to surface permission requests first.

I built this because other solutions require you to launch sessions from their app. This just monitors - it works with whatever terminal or IDE you're already using.

Happy to answer questions about the implementation.
```

---

## General First Comment (Community)

For community subreddits (r/ClaudeAI, r/MacApps):

```
Context: I run about 10 Claude Code sessions simultaneously across different projects. I got tired of switching between Zed and Ghostty terminals to check statuses, missing permission requests because I forgot which terminal had what, and not knowing if sessions finished or got stuck.

This app sits in the background and shows everything in one place. No plugins, no workflow changes - just open it and it finds all your sessions automatically.

Let me know if you run into any issues or have feature requests.
```

---

## Posting Tips

- **Start with r/ClaudeAI** - Most targeted audience of actual Claude Code users
- Post during peak hours (9-11 AM or 6-8 PM EST on weekdays)
- Always check each subreddit's rules about self-promotion first
- Post your first comment within 1-2 minutes to boost visibility
- Respond quickly to early comments
- Have your demo video/GIF ready to paste in comments if needed
- r/SideProject is marked with ⭐ as it's particularly well-suited for your story

---

## Minimal Version (To Avoid Reddit Spam Filter)

Use this version if your post gets auto-removed. Post without links, then add links in first comment.

### r/SideProject (Zero Trigger Words - Maximum Filter Avoidance)

**Title:** I built a dashboard to monitor multiple Claude Code sessions at once

**Body:**

Running multiple Claude Code agents across different projects was driving me crazy.

I'd have 10+ sessions going. Constantly switching terminals to check if sessions were blocked waiting for permission or if they finished.

Other tools exist but they force you to launch everything from their app. I just wanted to keep my normal workflow.

So I built a dashboard that auto-discovers every session by scanning processes. Shows real-time status. Permission requests surface to the top. Click any session to view the full conversation.

No plugins. No workflow changes.

Built with Tauri, Rust, and Svelte.

Video in comments. Would love feedback.

**First comment (post immediately after):**

```
Video demo: https://youtu.be/9PdN7joYmUk

The app scans running processes every 2 seconds to find Claude Code sessions, matches them to session files, and parses status from the JSONL format.

Project name is c9watch. You can find it on my profile.

Happy to answer questions.
```

---

### r/SideProject (Ultra-Minimal - Maximum Filter Avoidance)

---

### r/SideProject (Minimal - No Links, Reddit Filter-Optimized)

**Title:** I built a dashboard to monitor 10+ Claude Code sessions at once. No more terminal switching to check if agents are stuck.

**Body:**

Running multiple Claude Code agents across different projects was driving me crazy.

I'd have 10+ sessions going in Zed and Ghostty. Constantly switching terminals to check:

- Is this one blocked waiting for permission?
- Did that one finish yet or is it stuck?
- Which terminal was that task in again?

Other tools exist but they force you to launch everything from their app. I just wanted to keep my normal workflow.

Built c9watch to solve this. It's a macOS dashboard that:

- Auto-discovers every session by scanning processes
- Shows real-time status (Working / Needs Permission / Idle)
- Permission requests automatically surface to the top
- Click any session to view the full conversation
- Works with any terminal or IDE you already use

No plugins. No workflow changes. Just open it and see everything.

Built with Tauri, Rust, and Svelte. Open source (MIT).

Demo and download in comments. Would love feedback from anyone juggling multiple sessions.

**First comment (post immediately after):**

```
It's live now:

Demo: https://youtu.be/9PdN7joYmUk

Download: https://github.com/minchenlee/c9watch/releases

Source: https://github.com/minchenlee/c9watch

---

The technical challenge was matching running processes to session files and parsing the JSONL format to determine status in real-time.

Happy to answer questions about the implementation.
```

---

### r/ClaudeAI or r/ClaudeCode (Minimal - No Links)

**Title:** Built a dashboard to track Claude Code sessions across multiple projects

**Body:**

I run about 10 Claude Code sessions simultaneously across different projects and kept losing track of which ones needed permission or had finished their tasks.

Other monitoring tools require launching sessions from within their app. I didn't want to change my workflow - just wanted to keep using Claude Code in Zed and Ghostty like normal.

Built c9watch to solve this. It's a lightweight macOS dashboard that:

- Auto-discovers all sessions by scanning processes
- Shows real-time status (Working, Needs Permission, Idle)
- Permission requests surface to the top
- Conversation viewer to inspect any session
- Works with any terminal or IDE - no plugins needed

Built with Tauri, Rust, and Svelte. Open source (MIT).

Links in comments. Would appreciate feedback from anyone running multiple sessions.

**First comment (with links):**

```
Links:

Demo: https://youtu.be/9PdN7joYmUk

GitHub: https://github.com/minchenlee/c9watch

Download: https://github.com/minchenlee/c9watch/releases

---

Context: I got tired of switching between Zed and Ghostty terminals to check statuses, missing permission requests because I forgot which terminal had what, and not knowing if sessions finished or got stuck.

This app sits in the background and shows everything in one place. No plugins, no workflow changes - just open it and it finds all your sessions automatically.

Let me know if you run into any issues or have feature requests.
```

---

### r/MacApps (Minimal - No Links)

**Title:** c9watch - Dashboard for monitoring Claude Code sessions (native macOS)

**Body:**

Built a native macOS app to monitor Claude Code sessions across multiple projects.

The problem: running about 10 sessions simultaneously in Zed and Ghostty, constantly switching terminals to check if they're blocked on permissions or finished.

c9watch auto-discovers all sessions by scanning processes and shows their status in real-time. Works with any terminal or IDE - no plugins or workflow changes needed.

- Multiple views (Status, Project, Compact)
- Conversation viewer
- Session controls (stop, open, rename)
- Built with Tauri + Rust (not Electron)
- Menu bar integration

Open source (MIT). Links in comments.

**First comment (with links):**

```
Links:

Demo: https://youtu.be/9PdN7joYmUk

GitHub: https://github.com/minchenlee/c9watch

Download: https://github.com/minchenlee/c9watch/releases
```

---

---

## Threads 繁體中文版本

### 版本 1 (詳細版)

同時跑 10 個 Claude Code session 真的會瘋掉。

一直在不同的 terminal 之間切換，檢查哪個 session 卡在等 permission，哪個已經完成了。

市面上有其他工具，但它們都要你從它們的 app 裡面啟動 session。我只是想繼續用 Zed 和 Ghostty，不想改變工作流程。

所以我做了 c9watch —— 一個輕量的 macOS dashboard，透過掃描 process 自動找到所有 session。

功能：
- 即時顯示每個 session 的狀態（工作中 / 等待許可 / 閒置）
- 需要 permission 的自動浮到最上面
- 點擊任何 session 可以看完整對話
- 支援任何 terminal 或 IDE
- 不用裝 plugin，不改變工作流程

用 Tauri + Rust + Svelte 做的。開源 (MIT)。

Demo: https://youtu.be/9PdN7joYmUk
下載: https://github.com/minchenlee/c9watch/releases
原始碼: https://github.com/minchenlee/c9watch

歡迎回饋！

---

### 版本 2 (簡潔版)

做了一個 dashboard 來監控 Claude Code sessions。

問題：同時跑 10 個 sessions，一直切換 terminal 檢查哪個卡住、哪個完成了。

解法：自動掃描所有 process，即時顯示狀態，需要 permission 的自動浮到最上面。

用 Tauri + Rust + Svelte 做的。開源。

Demo: https://youtu.be/9PdN7joYmUk
GitHub: https://github.com/minchenlee/c9watch

---

### 版本 3 (Threads 風格 - 短篇幅)

最近做了個 dashboard 來監控 Claude Code sessions，因為同時跑 10 個 sessions 真的會一直在 Zed 和 Ghostty 的 terminal 之間切來切去，檢查哪個卡在等 permission、哪個已經跑完了。

本來想用現有的工具，但它們都要從它們的 app 裡面啟動 session，我只是想繼續用自己習慣的 terminal 而已。

所以就自己做了，用 process scanning 自動找到所有 sessions，解析 JSONL 判斷狀態。用 Tauri + Rust + Svelte 寫的，開源。

Demo: https://youtu.be/9PdN7joYmUk
GitHub: https://github.com/minchenlee/c9watch

---

### 版本 4 (超簡短 - 適合快速滑動)

同時管理 10+ 個 Claude Code sessions 的痛點：一直切換 terminal 檢查狀態

做了 c9watch 來解決：自動掃描所有 sessions，即時顯示狀態

開源 | macOS | Tauri + Rust + Svelte

https://youtu.be/9PdN7joYmUk

---

## Threads 發文建議

1. **保持簡短** - Threads 用戶喜歡快速閱讀，不要太長
2. **使用表情符號** - 比 Reddit 更隨性，適度使用 emoji 沒問題
3. **分段清楚** - 每段之間空行，方便閱讀
4. **直接放連結** - 不需要像 Reddit 那樣擔心被過濾
5. **視覺化** - 如果有截圖或 demo GIF，直接上傳效果更好
6. **標籤** - 可以加 #開發 #macOS #開源 等標籤增加曝光

---

## Links to Replace

Before posting, replace these placeholders:

- `[Demo]` → https://youtu.be/9PdN7joYmUk
- `[Download]` → https://github.com/minchenlee/c9watch/releases
- `[GitHub]` → https://github.com/minchenlee/c9watch
- `[link]` → Appropriate link based on context
