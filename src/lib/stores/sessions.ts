/**
 * Svelte stores for session state management
 */

import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
import type { Session, Conversation } from '../types';
import { SessionStatus } from '../types';
import { isDemoMode } from '../demo';
import { openSession } from '../api';
import { wsClient, useWebSocket, getStoredWsUrl, isTauri } from '../ws';

/**
 * Store containing all active sessions
 */
export const sessions = writable<Session[]>([]);

/**
 * Store containing the currently expanded session ID (for overlay)
 */
export const expandedSessionId = writable<string | null>(null);

/**
 * Store containing the conversation for the currently expanded session
 */
export const currentConversation = writable<Conversation | null>(null);

/**
 * Store for notification permission status
 */
export const notificationPermission = writable<'granted' | 'denied' | 'default'>('default');

/**
 * Derived store: sessions sorted by attention priority
 * Priority: NeedsPermission > WaitingForInput > Working > Connecting
 */
export const sortedSessions = derived(sessions, ($sessions) => {
	const priorityOrder: Record<SessionStatus, number> = {
		[SessionStatus.NeedsPermission]: 0,
		[SessionStatus.WaitingForInput]: 1,
		[SessionStatus.Working]: 2,
		[SessionStatus.Connecting]: 3
	};

	return [...$sessions].sort((a, b) => {
		const priorityA = priorityOrder[a.status] ?? 4;
		const priorityB = priorityOrder[b.status] ?? 4;
		if (priorityA !== priorityB) {
			return priorityA - priorityB;
		}
		return new Date(a.modified).getTime() - new Date(b.modified).getTime();
	});
});

/**
 * Derived store: count of sessions needing attention
 */
export const attentionCount = derived(sessions, ($sessions) => {
	return $sessions.filter(
		(s) => s.status === SessionStatus.NeedsPermission || s.status === SessionStatus.WaitingForInput
	).length;
});

/**
 * Derived store: status summary for header
 */
export const statusSummary = derived(sessions, ($sessions) => {
	const working = $sessions.filter(
		(s) => s.status === SessionStatus.Working || s.status === SessionStatus.Connecting
	).length;
	const permission = $sessions.filter(
		(s) => s.status === SessionStatus.NeedsPermission
	).length;
	const input = $sessions.filter(
		(s) => s.status === SessionStatus.WaitingForInput
	).length;

	return { working, permission, input };
});

/**
 * Notification metadata for click-to-focus
 */
interface NotificationMetadata {
	notificationId: number;
	sessionId: string;
	pid: number;
	projectPath: string;
	title: string;
}

const notificationMetadataMap = new Map<number, NotificationMetadata>();
const MAX_NOTIFICATION_ENTRIES = 10;

/**
 * Initialize event listeners for backend updates.
 * Automatically chooses Tauri IPC events or WebSocket depending on environment.
 */
export async function initializeSessionListeners() {
	if (useWebSocket()) {
		await initWebSocketListeners();
	} else {
		await initTauriListeners();
	}
}

// ── WebSocket mode ──────────────────────────────────────────────────

async function initWebSocketListeners() {
	const wsUrl = getStoredWsUrl();
	if (!wsUrl) {
		console.warn('[ws] No server URL configured');
		return;
	}

	try {
		await wsClient.connect(wsUrl);
	} catch (e) {
		console.error('[ws] Failed to connect:', e);
		return;
	}

	wsClient.on('sessionsUpdated', (data: Session[]) => {
		if (!get(isDemoMode)) {
			sessions.set(data);
		}
	});
}

// ── Tauri IPC mode ──────────────────────────────────────────────────

async function initTauriListeners() {
	await listen<Session[]>('sessions-updated', (event) => {
		if (!get(isDemoMode)) {
			sessions.set(event.payload);
		}
	});

	await listen<Conversation>('conversation-updated', (event) => {
		currentConversation.set(event.payload);
	});

	await listen<NotificationMetadata>('notification-fired', (event) => {
		if (!get(isDemoMode)) {
			const metadata = event.payload;
			notificationMetadataMap.set(metadata.notificationId, metadata);

			while (notificationMetadataMap.size > MAX_NOTIFICATION_ENTRIES) {
				const firstKey = notificationMetadataMap.keys().next().value;
				if (firstKey !== undefined) {
					notificationMetadataMap.delete(firstKey);
				}
			}
		}
	});
}

/**
 * Check if notification permission is granted (Tauri only)
 */
export async function checkNotificationPermission() {
	if (!isTauri()) {
		notificationPermission.set('default');
		return false;
	}
	try {
		const granted = await isPermissionGranted();
		notificationPermission.set(granted ? 'granted' : 'default');
		return granted;
	} catch (error) {
		console.error('[notification] Failed to check permission:', error);
		notificationPermission.set('default');
		return false;
	}
}

/**
 * Request notification permission from the user (Tauri only)
 */
export async function requestNotificationPermission() {
	if (!isTauri()) {
		notificationPermission.set('denied');
		return false;
	}
	try {
		const permission = await requestPermission();
		notificationPermission.set(permission);
		return permission === 'granted';
	} catch (error) {
		console.error('[notification] Failed to request permission:', error);
		notificationPermission.set('denied');
		return false;
	}
}

// Legacy alias for backward compatibility
export const selectedSessionId = expandedSessionId;
