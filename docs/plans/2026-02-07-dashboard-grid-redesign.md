# Dashboard Grid UI Redesign

## Overview

Redesign the Claude Session Monitor UI from a master-detail layout to a dashboard grid view optimized for parallel task monitoring and context preservation.

## Goals

- See all session statuses at a glance without clicking
- Take quick actions (approve permissions, send input) without losing context
- Preserve scroll position and draft prompts per session
- Handle 5-8 concurrent sessions comfortably

## Design

### Grid Layout and Cards

The main view is a grid of session cards, 2-3 columns depending on window width.

**Card contents:**
- Status indicator - Large colored dot in the corner
  - Blue = working
  - Orange = needs permission
  - Green = waiting for input
  - Gray = connecting
- Project name - Bold, primary text
- Git branch - Secondary text next to project name
- First prompt / current task - 2-line truncated preview
- Last activity - "2m ago" timestamp
- Message count - Small badge

**Visual hierarchy:**
- Cards needing attention (orange/green) have a subtle pulsing glow or highlighted border
- Working sessions (blue) are visually calm
- Grid auto-sorts: attention-needed cards float to top-left

**Spacing:**
- Comfortable padding with 12-16px gaps between cards
- Grid scrolls vertically if more than ~8 sessions

### Quick Actions Without Expansion

Act directly from cards without expanding them.

**Needs Permission (orange):**
- Card shows a prominent "Approve" button
- Clicking sends approval; card returns to "Working" state

**Waiting for Input (green):**
- Card shows a compact text input with "Send" button
- Type quick response and send without leaving grid
- For longer responses, expand the card

**Working (blue):**
- No action buttons; session is autonomous

**Action feedback:**
- Cards briefly flash to confirm actions
- Status updates in real-time

**Stop action:**
- Subtle "⋮" menu icon in card corner
- Menu contains: "Stop Session", "Open in Terminal/IDE"

### Expanded Card Overlay

Click a card (not on action buttons) to expand into a floating overlay.

**Expansion behavior:**
- Card animates upward and scales to ~70% window width, ~80% height
- Floats above grid with subtle shadow
- Grid dims but remains visible (status indicators still show)

**Overlay contents:**
- Header: Project name, git branch, status badge, "✕" close button
- Conversation area: Scrollable message history
- Prompt input: Full-width text input with Send button
- Action buttons: "Stop" and "Open in IDE"

**Context preservation:**
- Scroll position remembered per session
- Draft text in prompt input saved per session
- Switching away and back preserves both

**Closing:**
- Click "✕", press Escape, or click dimmed background
- Card animates back to grid position

### Attention Visibility

**While viewing expanded card:**
- Dimmed grid still shows status indicators
- Attention-needed cards have visible glow through dim overlay
- Floating badge: "2 sessions need attention" - clickable to return to grid

**Auto-sorting priority:**
1. Needs Permission (orange)
2. Waiting for Input (green)
3. Working (blue)
4. Connecting (gray)

Cards animate smoothly when status changes.

**System notifications (optional):**
- macOS notification when session needs attention and window not focused
- Clicking notification brings app to front with session highlighted

### Header and Global Controls

**Top header bar:**
- Session count: "6 Active Sessions"
- Status summary badges: "4 working · 1 needs permission · 1 waiting"
- Refresh indicator: "Updated 2s ago"

**No sidebar** - all navigation through grid.

**Window behavior:**
- Menu bar icon click toggles window
- Window remembers size and position
- Starts hidden

**Keyboard shortcuts:**
- `1-9` to quick-select session by grid position
- `Escape` to close expanded card
- `Tab` to cycle through attention-needed sessions

## Visual Layout

```
┌─────────────────────────────────────────────────┐
│  6 Active Sessions    4 working · 1 perm · 1 in │
├─────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ 🟠 Proj │  │ 🔵 Proj │  │ 🟢 Proj │         │
│  │ [Approve]│  │ main    │  │ [____] →│         │
│  └─────────┘  └─────────┘  └─────────┘         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │ 🔵 Proj │  │ 🔵 Proj │  │ 🔵 Proj │         │
│  │ feature │  │ develop │  │ main    │         │
│  └─────────┘  └─────────┘  └─────────┘         │
└─────────────────────────────────────────────────┘
```

## Implementation Changes

**Remove:**
- Left sidebar
- Master-detail layout
- SessionList.svelte (replace with grid)

**Add:**
- Grid layout container
- New SessionCard.svelte with inline actions
- ExpandedCardOverlay.svelte for conversation view
- Per-session state tracking (scroll position, drafts)

**Modify:**
- +page.svelte - new grid-based layout
- sessions store - add scroll/draft state per session
- ConversationView.svelte - adapt for overlay context

## State Management

```typescript
interface SessionUIState {
  scrollPosition: number;
  draftPrompt: string;
}

// Store: Map<sessionId, SessionUIState>
```

Grid sort order computed reactively from session statuses.
