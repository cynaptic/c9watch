/**
 * API wrapper — automatically dispatches to Tauri IPC or WebSocket
 * depending on the runtime environment.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import type { Session, Conversation } from './types';
import { isDemoMode } from './demo';
import { getDemoSessions, demoConversations } from './demo/data';
import { wsClient, useWebSocket } from './ws';

/**
 * Get all active Claude Code sessions
 */
export async function getSessions(): Promise<Session[]> {
	if (get(isDemoMode)) return getDemoSessions();

	if (useWebSocket()) {
		return await wsClient.request<Session[]>('getSessions');
	}
	return await invoke<Session[]>('get_sessions');
}

/**
 * Get the full conversation history for a specific session
 */
export async function getConversation(sessionId: string): Promise<Conversation> {
	if (get(isDemoMode)) {
		return demoConversations[sessionId] ?? { sessionId, messages: [] };
	}

	if (useWebSocket()) {
		return await wsClient.request<Conversation>('getConversation', { sessionId });
	}
	return await invoke<Conversation>('get_conversation', { sessionId });
}

/**
 * Stop a running session by sending SIGTERM
 */
export async function stopSession(pid: number): Promise<void> {
	if (get(isDemoMode)) return;

	if (useWebSocket()) {
		await wsClient.request('stopSession', { pid });
		return;
	}
	await invoke<void>('stop_session', { pid });
}

/**
 * Open the terminal or IDE window for a session
 */
export async function openSession(pid: number, projectPath: string): Promise<void> {
	if (get(isDemoMode)) return;

	if (useWebSocket()) {
		await wsClient.request('openSession', { pid, projectPath });
		return;
	}
	await invoke<void>('open_session', { pid, projectPath });
}

/**
 * Rename a session title
 */
export async function renameSession(sessionId: string, newName: string): Promise<void> {
	if (get(isDemoMode)) return;

	if (useWebSocket()) {
		await wsClient.request('renameSession', { sessionId, newName });
		return;
	}
	await invoke<void>('rename_session', { sessionId, newName });
}

/**
 * Take over a session: kill existing process, mark as managed
 */
export async function takeoverSession(pid: number, sessionId: string, projectPath: string): Promise<void> {
	if (get(isDemoMode)) return;

	if (useWebSocket()) {
		await wsClient.request('takeoverSession', { pid, sessionId, projectPath });
		return;
	}
	await invoke<void>('takeover_session', { pid, sessionId, projectPath });
}

/**
 * Check if a session is managed (taken over) by c9watch
 */
export async function isSessionManaged(sessionId: string): Promise<boolean> {
	if (get(isDemoMode)) return false;

	if (useWebSocket()) {
		const result = await wsClient.request<{ managed: boolean } | boolean>('isSessionManaged', { sessionId });
		if (typeof result === 'boolean') return result;
		return result.managed;
	}
	return await invoke<boolean>('is_session_managed', { sessionId });
}

/**
 * Send input to a session via the SDK bridge.
 * On first send, kills the original claude process to avoid branch conflicts.
 */
export async function sendInput(sessionId: string, input: string, projectPath: string, pid: number = 0): Promise<void> {
	if (get(isDemoMode)) return;

	if (useWebSocket()) {
		await wsClient.request('sendInput', { sessionId, input, projectPath, pid });
		return;
	}
	await invoke<void>('send_input', { sessionId, input, projectPath, pid });
}

/**
 * Server connection info (desktop/Tauri only)
 */
export interface ServerInfo {
	token: string;
	port: number;
	localIp: string;
	wsUrl: string;
}

export async function getServerInfo(): Promise<ServerInfo> {
	return await invoke<ServerInfo>('get_server_info');
}

// ── Stream event subscriptions ──────────────────────────────────────

export interface StreamEventPayload {
	sessionId: string;
	data: any;
}

export interface StreamEndPayload {
	sessionId: string;
	success: boolean;
	error?: string;
}

/**
 * Subscribe to streaming events from claude process output.
 * Returns an unsubscribe function.
 */
export function onStreamEvent(cb: (e: StreamEventPayload) => void): () => void {
	if (useWebSocket()) {
		const handler = (msg: any) => cb(msg as StreamEventPayload);
		wsClient.on('streamEvent', handler);
		return () => wsClient.off('streamEvent', handler);
	}

	// Tauri: listen for 'stream-event' emitted from Rust
	let unlisten: (() => void) | null = null;
	listen<string>('stream-event', (event) => {
		try {
			const parsed = typeof event.payload === 'string' ? JSON.parse(event.payload) : event.payload;
			cb(parsed as StreamEventPayload);
		} catch (e) {
			console.error('[api] Failed to parse stream event:', e);
		}
	}).then((fn) => {
		unlisten = fn;
	});

	return () => {
		unlisten?.();
	};
}

/**
 * Subscribe to stream completion events.
 * Returns an unsubscribe function.
 */
export function onStreamEnd(cb: (e: StreamEndPayload) => void): () => void {
	if (useWebSocket()) {
		const handler = (msg: any) => cb(msg as StreamEndPayload);
		wsClient.on('streamEnd', handler);
		return () => wsClient.off('streamEnd', handler);
	}

	// Tauri: streamEnd is also sent via 'stream-event' channel
	let unlisten: (() => void) | null = null;
	listen<string>('stream-event', (event) => {
		try {
			const parsed = typeof event.payload === 'string' ? JSON.parse(event.payload) : event.payload;
			if (parsed.type === 'streamEnd') {
				cb(parsed as StreamEndPayload);
			}
		} catch (e) {
			console.error('[api] Failed to parse stream end:', e);
		}
	}).then((fn) => {
		unlisten = fn;
	});

	return () => {
		unlisten?.();
	};
}
