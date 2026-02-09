<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { fade, scale } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import type { Session, Conversation } from '$lib/types';
	import { SessionStatus } from '$lib/types';
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

	let messagesContainer: HTMLDivElement;
	let isInitialLoad = $state(true);
	let hasScrolledToBottom = $state(false);
	let showTools = $state(true);
	let showThinking = $state(true);

	onMount(() => {
		isInitialLoad = false;

		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				handleClose();
			}
		};
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

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
					</div>
				{/if}
			</div>

		</div>

		<div class="nav-map-side" in:scale={{ start: 0.95, duration: 300, easing: quintOut }}>
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

	.nav-map-side {
		flex-shrink: 0;
		height: 100%;
		display: flex;
		flex-direction: column;
		pointer-events: auto; /* Enable clicks on the nav map */
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

	.status-indicator {
		position: relative;
		width: 8px;
		height: 8px;
		flex-shrink: 0;
	}

	.status-dot {
		width: 100%;
		height: 100%;
		background: var(--status-color);
	}

	.status-ring {
		display: none;
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

</style>
