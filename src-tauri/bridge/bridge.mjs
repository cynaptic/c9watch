#!/usr/bin/env node
/**
 * c9watch SDK Bridge
 *
 * Long-lived Node.js process that manages Claude Agent SDK V2 sessions.
 * Uses V1 query() API with resume option — one process per message turn,
 * but managed cleanly by the SDK with proper streaming.
 */

import { createInterface } from "node:readline";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { query } from "@anthropic-ai/claude-agent-sdk";

/**
 * Custom spawn function for the SDK.
 *
 * Fixes two common issues:
 * 1. "node" not in PATH — replaced with process.execPath (absolute path)
 * 2. cwd doesn't exist — Node.js confusingly reports ENOENT for the
 *    *command* instead of the cwd, so we fall back to home dir
 * 3. SDK passes a custom `env` that replaces process.env entirely —
 *    we merge them so PATH and other system vars are preserved
 */
function spawnClaude({ command, args, cwd, env, signal }) {
  const cmd = command === "node" ? process.execPath : command;

  // Validate cwd — non-existent cwd causes misleading ENOENT on the command
  if (cwd && !existsSync(cwd)) {
    log(`WARNING: cwd does not exist: ${cwd}, falling back to ${homedir()}`);
    cwd = homedir();
  }

  // Merge process.env with SDK's env so PATH and system vars are preserved
  const mergedEnv = { ...process.env, ...env };

  log(`Spawning: ${cmd} ${args[0] || ""} (cwd: ${cwd || "inherit"})`);
  return spawn(cmd, args, { cwd, env: mergedEnv, signal, stdio: ["pipe", "pipe", "pipe"] });
}

/** @type {Map<string, { cwd: string }>} */
const sessions = new Map();

function emit(obj) {
  process.stdout.write(JSON.stringify(obj) + "\n");
}

function ack(id, success, error) {
  emit({ type: "ack", id, success, ...(error ? { error } : {}) });
}

function log(msg) {
  process.stderr.write(`[bridge] ${msg}\n`);
}

// ── Prevent unhandled errors from killing the process ─────────────

process.on("uncaughtException", (err) => {
  log(`Uncaught exception: ${err.stack || err}`);
});

process.on("unhandledRejection", (err) => {
  log(`Unhandled rejection: ${err?.stack || err}`);
});

// ── Command handlers ──────────────────────────────────────────────

async function handleResume(id, sessionId, cwd) {
  // Just store the session info — no process spawned yet.
  // The actual claude process is spawned on each send().
  sessions.set(sessionId, { cwd });
  ack(id, true);
  log(`Session ${sessionId} registered (cwd: ${cwd})`);
}

async function handleSend(id, sessionId, message, cwd) {
  // Auto-register session if not seen before
  if (!sessions.has(sessionId) && cwd) {
    sessions.set(sessionId, { cwd });
    log(`Session ${sessionId} auto-registered (cwd: ${cwd})`);
  }
  const info = sessions.get(sessionId);
  if (!info) {
    ack(id, false, `Session ${sessionId} not found — provide cwd on first send`);
    return;
  }

  log(`Sending to ${sessionId}: ${message.substring(0, 80)}`);

  try {
    const q = query({
      prompt: message,
      options: {
        resume: sessionId,
        cwd: info.cwd || undefined,
        permissionMode: "bypassPermissions",
        allowDangerouslySkipPermissions: true,
        includePartialMessages: true,
        spawnClaudeCodeProcess: spawnClaude,
        stderr: (data) => {
          for (const line of data.split("\n")) {
            if (line.trim()) log(`[claude] ${line}`);
          }
        },
      },
    });

    // Ack immediately — streaming happens in background
    ack(id, true);

    for await (const msg of q) {
      if (msg.type === "stream_event") {
        emit({ type: "streamEvent", sessionId, data: msg.event });
      }
      if (msg.type === "result") {
        log(`Turn complete for ${sessionId} (${msg.subtype})`);
        break;
      }
    }

    emit({ type: "streamEnd", sessionId, success: true });
  } catch (err) {
    log(`Send error for ${sessionId}: ${err.stack || err}`);
    // If we haven't acked yet, ack with error
    // Otherwise emit streamEnd with error
    emit({ type: "streamEnd", sessionId, success: false, error: String(err) });
  }
}

function handleClose(id, sessionId) {
  sessions.delete(sessionId);
  ack(id, true);
}

// ── Main loop ─────────────────────────────────────────────────────

const rl = createInterface({ input: process.stdin });

rl.on("line", (line) => {
  let cmd;
  try {
    cmd = JSON.parse(line);
  } catch (err) {
    log(`Invalid JSON: ${line}`);
    return;
  }

  const { cmd: action, id, sessionId } = cmd;

  switch (action) {
    case "resume":  handleResume(id, sessionId, cmd.cwd); break;
    case "send":    handleSend(id, sessionId, cmd.message, cmd.cwd); break;
    case "close":   handleClose(id, sessionId); break;
    default:        ack(id, false, `Unknown command: ${action}`);
  }
});

rl.on("close", () => {
  process.exit(0);
});

log(`c9watch SDK bridge started (node: ${process.execPath})`);
