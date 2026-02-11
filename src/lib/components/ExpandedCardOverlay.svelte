<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { fade, scale } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import type { Session, Conversation } from '$lib/types';
	import { SessionStatus } from '$lib/types';
	import { isSessionManaged, takeoverSession, sendInput, getConversation, onStreamEvent, onStreamEnd } from '$lib/api';
	import type { StreamEventPayload, StreamEndPayload } from '$lib/api';
	import { currentConversation } from '$lib/stores/sessions';
	import MessageBubble from './MessageBubble.svelte';
	import MessageNavMap from './MessageNavMap.svelte';

	interface Props {
		session: Session;
		conversation: Conversation | null;
		onclose?: () => void;
		onstop?: () => void;
		onopen?: () => void;
	}

	let { session, conversation, onclose, onstop, onopen }: Props = $props();

	let managed = $state(false);
	let inputText = $state('');
	let sending = $state(false);
	let takingOver = $state(false);

	// Streaming state
	let streamText = $state('');
	let streamTool = $state<string | null>(null);
	let streamActive = $state(false);
	let streamError = $state<string | null>(null);
	let sentMessage = $state<string | null>(null);
	let streamStartTime = $state<number>(0);
	let elapsedSeconds = $state(0);
	let elapsedTimer = $state<ReturnType<typeof setInterval> | null>(null);
	let hasFirstContent = $state(false);

	let messagesContainer = $state<HTMLDivElement>(undefined!);
	let isInitialLoad = $state(true);
	let hasScrolledToBottom = $state(false);
	let showTools = $state(true);
	let showThinking = $state(true);
	let navSheetOpen = $state(false);

	function handleNavItemClick() {
		// Close the bottom sheet on mobile after navigating
		navSheetOpen = false;
	}

	function handleStreamChunk(e: StreamEventPayload) {
		if (e.sessionId !== session.id) return;
		const data = e.data;
		if (!data) return;

		if (!hasFirstContent) {
			hasFirstContent = true;
			// Stop the elapsed timer once content starts flowing
			if (elapsedTimer) { clearInterval(elapsedTimer); elapsedTimer = null; }
		}

		// Claude stream-json format: each line is a JSON object with a "type" field
		if (data.type === 'content_block_start') {
			const block = data.content_block;
			if (block?.type === 'tool_use') {
				streamTool = block.name || 'tool';
			}
		} else if (data.type === 'content_block_delta') {
			const delta = data.delta;
			if (delta?.type === 'text_delta' && delta.text) {
				streamText += delta.text;
			}
		} else if (data.type === 'content_block_stop') {
			streamTool = null;
		} else if (data.type === 'message_start' || data.type === 'message_delta') {
			// Message-level events, no text to append
		}
	}

	function handleStreamEnd(e: StreamEndPayload) {
		if (e.sessionId !== session.id) return;
		streamActive = false;
		sending = false;
		streamTool = null;
		hasFirstContent = false;
		if (elapsedTimer) { clearInterval(elapsedTimer); elapsedTimer = null; }
		if (!e.success) {
			streamError = e.error || 'Stream ended with error';
		} else {
			sentMessage = null;
			streamText = '';
			// Re-fetch conversation from JSONL to show the new messages
			getConversation(session.id).then((conv) => {
				currentConversation.set(conv);
			}).catch((err) => {
				console.error('Failed to refresh conversation:', err);
			});
		}
	}

	onMount(() => {
		isInitialLoad = false;

		isSessionManaged(session.id).then((m) => {
			managed = m;
		}).catch(() => {});

		// Subscribe to stream events
		const unsubEvent = onStreamEvent(handleStreamChunk);
		const unsubEnd = onStreamEnd(handleStreamEnd);

		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				handleClose();
			}
		};
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
			unsubEvent();
			unsubEnd();
			if (elapsedTimer) clearInterval(elapsedTimer);
		};
	});

	async function handleTakeover() {
		takingOver = true;
		try {
			await takeoverSession(session.pid, session.id, session.projectPath);
			managed = true;
		} catch (err) {
			console.error('Takeover failed:', err);
		} finally {
			takingOver = false;
		}
	}

	async function handleSendInput() {
		const text = inputText.trim();
		if (!text || sending) return;
		sending = true;
		streamText = '';
		streamTool = null;
		streamError = null;
		streamActive = true;
		sentMessage = text;
		hasFirstContent = false;
		inputText = '';

		// Start elapsed timer
		streamStartTime = Date.now();
		elapsedSeconds = 0;
		if (elapsedTimer) clearInterval(elapsedTimer);
		elapsedTimer = setInterval(() => {
			elapsedSeconds = Math.floor((Date.now() - streamStartTime) / 1000);
		}, 1000);

		try {
			await sendInput(session.id, text, session.projectPath, session.pid);
		} catch (err) {
			console.error('Send input failed:', err);
			streamActive = false;
			streamError = String(err);
			sending = false;
			if (elapsedTimer) { clearInterval(elapsedTimer); elapsedTimer = null; }
		}
	}

	function handleInputKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			handleSendInput();
		}
	}

	function handleScroll() {
		// No longer persisting scroll position as we want to always show latest on open
	}

	$effect(() => {
		if (conversation && conversation.messages.length > 0 && messagesContainer) {
			if (!hasScrolledToBottom) {
				// Initial scroll to bottom when opening
				tick().then(() => {
					messagesContainer.scrollTop = messagesContainer.scrollHeight;
					hasScrolledToBottom = true;
				});
			} else {
				// Auto-scroll logic for new messages
				const isAtBottom =
					messagesContainer.scrollHeight - messagesContainer.scrollTop - messagesContainer.clientHeight < 150;
				if (isAtBottom) {
					tick().then(() => {
						messagesContainer.scrollTop = messagesContainer.scrollHeight;
					});
				}
			}
		}
	});

	// Auto-scroll when sent message appears or streaming text updates
	$effect(() => {
		if ((sentMessage || streamText) && messagesContainer) {
			tick().then(() => {
				messagesContainer.scrollTop = messagesContainer.scrollHeight;
			});
		}
	});

	let isPermission = $derived(session.status === SessionStatus.NeedsPermission);
	let isWaitingInput = $derived(session.status === SessionStatus.WaitingForInput);
	let isWorking = $derived(session.status === SessionStatus.Working);

	function getStatusColor(): string {
		switch (session.status) {
			case SessionStatus.Working:
				return 'var(--status-working)';
			case SessionStatus.NeedsPermission:
				return 'var(--status-permission)';
			case SessionStatus.WaitingForInput:
				return 'var(--status-input)';
			case SessionStatus.Connecting:
				return 'var(--status-working)';
			default:
				return 'var(--status-working)';
		}
	}

	function getStatusLabel(): string {
		switch (session.status) {
			case SessionStatus.Working:
				return 'Working';
			case SessionStatus.NeedsPermission:
				return 'Approval Required';
			case SessionStatus.WaitingForInput:
				return 'Ready';
			case SessionStatus.Connecting:
				return 'Connecting';
			default:
				return 'Unknown';
		}
	}

	function handleClose() {
		onclose?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class="overlay-backdrop"
	onclick={handleBackdropClick}
	role="dialog"
	aria-modal="true"
	aria-labelledby="overlay-title"
	tabindex="-1"
	transition:fade={{ duration: 200 }}
>
	<div class="overlay-layout">
		<div
			class="overlay-card"
			class:permission={isPermission}
			class:waiting={isWaitingInput}
			in:scale={{ start: 0.95, duration: 300, easing: quintOut }}
		>
			<!-- Header -->
			<header class="overlay-header" data-tauri-drag-region>
				<div class="header-left" data-tauri-drag-region>

					<div class="header-info">
						<div class="header-title">
							<h2 id="overlay-title" class="project-name">{session.summary || session.firstPrompt || 'New Session'}</h2>
						</div>
						<div class="header-meta">
							<span class="status-label" style="color: {getStatusColor()}">{getStatusLabel()}</span>
							<span class="separator">·</span>
							<span class="session-name-badge">{session.sessionName}</span>
							<span class="separator">·</span>
							<span class="message-count">{conversation?.messages.length ?? 0} messages</span>
							{#if session.gitBranch}
								<span class="separator">·</span>
								<div class="git-info">
									<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<line x1="6" y1="3" x2="6" y2="15" />
										<circle cx="18" cy="6" r="3" />
										<circle cx="6" cy="18" r="3" />
										<path d="M18 9a9 9 0 0 1-9 9" />
									</svg>
									<span class="branch-name" title={session.gitBranch}>{session.gitBranch}</span>
								</div>
							{/if}
						</div>
					</div>
				</div>
				<div class="header-actions">
					{#if !managed}
						<button
							type="button"
							class="header-button takeover-btn"
							onclick={handleTakeover}
							disabled={takingOver}
							title="Takeover Session"
						>
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
							</svg>
						</button>
					{/if}
					<button type="button" class="header-button" onclick={() => onstop?.()} title="Stop Session">
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<rect x="6" y="6" width="12" height="12" rx="1" />
						</svg>
					</button>
					<button type="button" class="header-button" onclick={() => onopen?.()} title="Open in IDE">
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<polyline points="12 6 18 6 18 12" />
							<line x1="7" y1="17" x2="18" y2="6" />
						</svg>
					</button>
					<div class="header-divider"></div>
					<button 
						type="button" 
						class="header-button toggle-thinking" 
						class:active={showThinking} 
						onclick={() => showThinking = !showThinking} 
						title={showThinking ? "Hide Thinking" : "Show Thinking"}
					>
						<span>◇</span>
					</button>
					<button 
						type="button" 
						class="header-button toggle-tools" 
						class:active={showTools} 
						onclick={() => showTools = !showTools} 
						title={showTools ? "Hide Tools" : "Show Tools"}
					>
						<span>⚙</span>
					</button>
					<div class="header-divider"></div>
					<button type="button" class="close-button" onclick={handleClose} aria-label="Close">
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<line x1="18" y1="6" x2="6" y2="18" />
							<line x1="6" y1="6" x2="18" y2="18" />
						</svg>
					</button>
				</div>
			</header>

			<!-- Conversation Area -->
			<div class="conversation-area" bind:this={messagesContainer} onscroll={handleScroll}>
				{#if !conversation}
					<div class="loading-state">

						<p>Loading conversation...</p>
					</div>
				{:else if conversation.messages.length === 0}
					<div class="empty-state">
						<div class="empty-icon">
							<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
								<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
							</svg>
						</div>
						<p>No messages yet</p>
						<p class="empty-hint">Send a message to start the conversation</p>
					</div>
				{:else}
					<div class="messages">
						{#each conversation.messages as message, index (index)}
							{#if (showTools || (message.messageType !== 'ToolUse' && message.messageType !== 'ToolResult')) && (showThinking || message.messageType !== 'Thinking')}
								<MessageBubble {message} />
							{/if}
						{/each}

						<!-- Inline streaming area (inside scroll) -->
						{#if sentMessage}
							<div class="stream-bubble user">
								<div class="stream-header">
									<span class="stream-icon">→</span>
									<span class="stream-role">You</span>
								</div>
								<div class="stream-content">{sentMessage}</div>
							</div>
						{/if}
						{#if streamActive || streamText || streamError}
							<div class="stream-bubble assistant">
								<div class="stream-header">
									<span class="stream-icon">◆</span>
									<span class="stream-role">Claude</span>
									{#if streamActive && !hasFirstContent}
										<span class="stream-elapsed">{elapsedSeconds}s</span>
									{/if}
								</div>
								{#if streamActive && !hasFirstContent}
									<div class="stream-waiting">
										<div class="waiting-bar"></div>
										<span class="waiting-label">
											{#if elapsedSeconds < 3}
												Starting session...
											{:else if elapsedSeconds < 8}
												Warming up...
											{:else}
												Thinking...
											{/if}
										</span>
									</div>
								{/if}
								{#if streamTool}
									<div class="stream-tool-indicator">
										<span class="tool-spinner"></span>
										Using {streamTool}...
									</div>
								{/if}
								{#if streamText}
									<div class="stream-content">{streamText}</div>
								{/if}
								{#if streamError}
									<div class="stream-error">{streamError}</div>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Input Bar (only after takeover) -->
			{#if managed}
				<div class="input-bar">
					<textarea
						class="input-textarea"
						bind:value={inputText}
						onkeydown={handleInputKeydown}
						placeholder="Send a message... (Ctrl+Enter)"
						rows="2"
						disabled={sending}
					></textarea>
					<button
						type="button"
						class="send-btn"
						onclick={handleSendInput}
						disabled={!inputText.trim() || sending}
						title="Send (Ctrl+Enter)"
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<line x1="22" y1="2" x2="11" y2="13" />
							<polygon points="22 2 15 22 11 13 2 9 22 2" />
						</svg>
					</button>
				</div>
			{/if}

			<!-- Mobile: FAB to open nav sheet -->
			<button
				type="button"
				class="nav-fab"
				class:open={navSheetOpen}
				onclick={() => navSheetOpen = !navSheetOpen}
				title="Navigation"
			>
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<line x1="3" y1="6" x2="21" y2="6" />
					<line x1="3" y1="12" x2="21" y2="12" />
					<line x1="3" y1="18" x2="21" y2="18" />
				</svg>
			</button>
		</div>

		<!-- Desktop: sidebar nav -->
		<div class="nav-map-side nav-desktop" in:scale={{ start: 0.95, duration: 300, easing: quintOut }}>
			<MessageNavMap {conversation} scrollContainer={messagesContainer} bind:showTools bind:showThinking />
		</div>

		<!-- Mobile: bottom sheet nav -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="nav-sheet-backdrop" class:open={navSheetOpen} onclick={() => navSheetOpen = false}></div>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="nav-sheet" class:open={navSheetOpen} onclick={handleNavItemClick}>
			<div class="nav-sheet-handle">
				<div class="handle-bar"></div>
			</div>
			<MessageNavMap {conversation} scrollContainer={messagesContainer} bind:showTools bind:showThinking />
		</div>
	</div>
</div>

<style>
	.overlay-backdrop {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: var(--space-2xl);
	}

	.overlay-layout {
		display: flex;
		align-items: flex-start;
		gap: var(--space-xl);
		width: 100%;
		max-width: 1100px;
		height: 85vh;
		max-height: 900px;
		pointer-events: none; /* Allow clicks through empty layout area */
	}

	.overlay-card {
		position: relative;
		flex: 1; /* Take up remaining space */
		height: 100%;
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		pointer-events: auto; /* Enable clicks on the card */
		box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
	}

	.nav-map-side.nav-desktop {
		flex-shrink: 0;
		height: 100%;
		display: flex;
		flex-direction: column;
		pointer-events: auto;
	}

	/* Mobile bottom sheet elements — hidden on desktop */
	.nav-fab,
	.nav-sheet-backdrop,
	.nav-sheet {
		display: none;
	}

	/* Header */
	.overlay-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-lg) var(--space-xl);
		border-bottom: 1px solid var(--border-default);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.header-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.header-title {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.project-name {
		font-family: var(--font-pixel);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 500px;
	}

	.session-name-badge {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 500;
		color: var(--text-muted);
		background: var(--bg-elevated);
		padding: 2px 6px;
		border: 1px solid var(--border-default);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.git-info {
		display: flex;
		align-items: center;
		gap: 6px;
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		min-width: 0;
	}

	.branch-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 200px;
	}

	.header-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.status-label {
		font-weight: 500;
	}

	.separator {
		color: var(--text-muted);
	}

	.message-count {
		color: var(--text-muted);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.header-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.header-button:hover {
		color: var(--text-primary);
	}

	.header-button span {
		font-family: var(--font-mono);
		font-size: 14px;
	}

	.header-button.active.toggle-thinking {
		color: var(--status-permission);
		opacity: 1;
	}

	.header-button.active.toggle-tools {
		color: var(--status-input);
		opacity: 1;
	}

	.header-divider {
		width: 1px;
		height: 16px;
		background: var(--border-default);
		margin: 0 var(--space-sm);
	}

	.close-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.close-button:hover {
		color: var(--accent-red);
	}

	.conversation-area {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.messages {
		display: flex;
		flex-direction: column;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: var(--space-md);
		color: var(--text-muted);
	}



	.empty-icon {
		opacity: 0.3;
		margin-bottom: var(--space-sm);
	}

	.empty-hint {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	/* ── Streaming Area ───────────────────────────────────────── */
	.stream-bubble {
		margin: var(--space-sm) 0;
		padding: var(--space-md) var(--space-lg);
		max-width: 100%;
		border-left: 1px solid var(--border-default);
	}

	.stream-bubble.user {
		border-left: 2px solid var(--text-primary);
		background: rgba(255, 255, 255, 0.01);
	}

	.stream-bubble.assistant {
		border-left-color: var(--text-muted);
	}

	.stream-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-sm);
	}

	.stream-icon {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.stream-role {
		font-family: var(--font-mono);
		font-weight: 500;
		font-size: 12px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.stream-bubble.user .stream-role {
		color: var(--text-primary);
	}

	.stream-content {
		color: var(--text-secondary);
		font-size: 15px;
		line-height: 1.6;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.stream-bubble.user .stream-content {
		color: var(--text-primary);
	}

	.stream-tool-indicator {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		color: var(--status-input);
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: var(--space-sm);
	}

	.tool-spinner {
		display: inline-block;
		width: 10px;
		height: 10px;
		border: 2px solid var(--status-input);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.stream-elapsed {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		letter-spacing: 0.05em;
		opacity: 0.6;
	}

	.stream-waiting {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.waiting-bar {
		height: 2px;
		width: 100%;
		background: var(--border-default);
		position: relative;
		overflow: hidden;
	}

	.waiting-bar::after {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		width: 40%;
		background: var(--text-muted);
		animation: waiting-slide 1.5s ease-in-out infinite;
	}

	@keyframes waiting-slide {
		0% { left: -40%; }
		100% { left: 100%; }
	}

	.waiting-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.stream-error {
		color: var(--accent-red);
		margin-top: var(--space-sm);
	}

	/* ── Takeover Button ───────────────────────────────────────── */
	.header-button.takeover-btn {
		color: var(--status-input);
	}

	.header-button.takeover-btn:hover {
		color: var(--text-primary);
	}

	.header-button.takeover-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	/* ── Input Bar ─────────────────────────────────────────────── */
	.input-bar {
		display: flex;
		align-items: flex-end;
		gap: var(--space-sm);
		padding: var(--space-md) var(--space-xl);
		border-top: 1px solid var(--border-default);
		background: var(--bg-card);
	}

	.input-textarea {
		flex: 1;
		resize: none;
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--text-primary);
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		padding: var(--space-sm) var(--space-md);
		outline: none;
		transition: border-color 0.2s ease;
		min-height: 40px;
		max-height: 120px;
	}

	.input-textarea:focus {
		border-color: var(--status-input);
	}

	.input-textarea:disabled {
		opacity: 0.5;
	}

	.send-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		background: var(--text-primary);
		border: 1px solid var(--text-primary);
		color: var(--bg-base);
		cursor: pointer;
		transition: all 0.2s ease;
		flex-shrink: 0;
	}

	.send-btn:hover:not(:disabled) {
		background: var(--text-secondary);
		border-color: var(--text-secondary);
	}

	.send-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	/* ── Mobile Responsive ─────────────────────────────────────── */
	@media (max-width: 768px) {
		.overlay-backdrop {
			padding: 0;
		}

		.overlay-layout {
			max-width: 100%;
			height: 100vh;
			max-height: 100vh;
		}

		/* Hide the desktop sidebar nav on mobile */
		.nav-map-side.nav-desktop {
			display: none;
		}

		.overlay-card {
			border: none;
			box-shadow: none;
		}

		.overlay-header {
			padding: var(--space-md) var(--space-md);
			gap: var(--space-sm);
		}

		.header-left {
			min-width: 0;
			flex: 1;
		}

		.project-name {
			font-size: 13px;
			max-width: none;
		}

		.header-meta {
			flex-wrap: wrap;
			font-size: 11px;
			gap: var(--space-xs);
		}

		.header-actions {
			flex-shrink: 0;
		}

		.header-divider {
			display: none;
		}

		.header-button {
			width: 28px;
			height: 28px;
		}

		.close-button {
			width: 28px;
			height: 28px;
		}

		.conversation-area {
			padding: var(--space-md);
			padding-bottom: 72px; /* Space for FAB */
		}

		/* ── FAB (Floating Action Button) ────────────── */
		.nav-fab {
			display: flex;
			align-items: center;
			justify-content: center;
			position: fixed;
			bottom: 20px;
			right: 20px;
			width: 48px;
			height: 48px;
			background: var(--bg-card);
			border: 1px solid var(--border-default);
			color: var(--text-secondary);
			z-index: 1010;
			pointer-events: auto;
			box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6);
			transition: all 0.2s ease;
		}

		.nav-fab:active {
			transform: scale(0.95);
		}

		.nav-fab.open {
			background: var(--text-primary);
			color: var(--bg-base);
			border-color: var(--text-primary);
		}

		/* ── Bottom Sheet Backdrop ────────────────────── */
		.nav-sheet-backdrop {
			display: block;
			position: fixed;
			inset: 0;
			background: rgba(0, 0, 0, 0.5);
			z-index: 1020;
			pointer-events: none;
			opacity: 0;
			transition: opacity 0.25s ease;
		}

		.nav-sheet-backdrop.open {
			pointer-events: auto;
			opacity: 1;
		}

		/* ── Bottom Sheet ─────────────────────────────── */
		.nav-sheet {
			display: flex;
			flex-direction: column;
			position: fixed;
			left: 0;
			right: 0;
			bottom: 0;
			height: 55vh;
			background: var(--bg-card);
			border-top: 1px solid var(--border-default);
			z-index: 1030;
			pointer-events: auto;
			transform: translateY(100%);
			transition: transform 0.3s cubic-bezier(0.32, 0.72, 0, 1);
			overflow: hidden;
		}

		.nav-sheet.open {
			transform: translateY(0);
		}

		.nav-sheet-handle {
			display: flex;
			justify-content: center;
			padding: var(--space-md) 0 var(--space-sm);
			flex-shrink: 0;
		}

		.handle-bar {
			width: 36px;
			height: 4px;
			background: var(--border-default);
			border-radius: 2px;
		}
	}

</style>
