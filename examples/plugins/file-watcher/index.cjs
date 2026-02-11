#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const readline = require("readline");

// --- CLI args ---
const args = process.argv.slice(2);
function getArg(name) {
  const idx = args.indexOf(name);
  return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : null;
}

const watchDir = getArg("--dir");
const ignoreRaw = getArg("--ignore");
const debounceMs = parseInt(getArg("--debounce") || "300", 10);

if (!watchDir) {
  process.stderr.write("Usage: node index.js --dir <path> [--ignore <patterns>] [--debounce <ms>]\n");
  process.exit(1);
}

const resolvedDir = path.resolve(watchDir);
if (!fs.existsSync(resolvedDir)) {
  process.stderr.write(`Directory not found: ${resolvedDir}\n`);
  process.exit(1);
}

const ignorePatterns = ignoreRaw
  ? ignoreRaw.split(",").map((p) => p.trim())
  : [".git", "node_modules", ".DS_Store", "Thumbs.db"];

// --- JSON-RPC helpers ---
function sendJson(obj) {
  const line = JSON.stringify(obj);
  process.stdout.write(line + "\n");
}

function sendResponse(id, result) {
  sendJson({ jsonrpc: "2.0", id, result });
}

function sendEvent(eventType, data) {
  sendJson({
    jsonrpc: "2.0",
    method: "event",
    params: { eventType, data },
  });
}

// --- File tracking & debounce ---
const knownFiles = new Set();
const debounceTimers = new Map();

function scanExistingFiles(dir) {
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      if (ignorePatterns.includes(entry.name)) continue;
      const fullPath = path.join(dir, entry.name);
      if (entry.isFile()) {
        knownFiles.add(fullPath);
      } else if (entry.isDirectory()) {
        scanExistingFiles(fullPath);
      }
    }
  } catch {
    // skip inaccessible directories
  }
}

function shouldIgnore(filePath) {
  const parts = filePath.split(path.sep);
  return parts.some((part) => ignorePatterns.includes(part));
}

function handleFileEvent(filePath) {
  if (shouldIgnore(filePath)) return;

  // Debounce: reset timer for this file
  if (debounceTimers.has(filePath)) {
    clearTimeout(debounceTimers.get(filePath));
  }

  debounceTimers.set(
    filePath,
    setTimeout(() => {
      debounceTimers.delete(filePath);

      try {
        const stat = fs.statSync(filePath);
        if (!stat.isFile()) return;

        const isNew = !knownFiles.has(filePath);
        knownFiles.add(filePath);

        const eventType = isNew ? "file_created" : "file_changed";
        sendEvent(eventType, {
          file_path: filePath,
          event_type: eventType,
          timestamp: new Date().toISOString(),
        });
      } catch {
        // File may have been deleted between event and stat
        if (knownFiles.has(filePath)) {
          knownFiles.delete(filePath);
        }
      }
    }, debounceMs),
  );
}

// --- Watcher ---
let watcher = null;

function startWatching() {
  scanExistingFiles(resolvedDir);

  watcher = fs.watch(resolvedDir, { recursive: true }, (_eventType, filename) => {
    if (!filename) return;
    const fullPath = path.join(resolvedDir, filename);
    handleFileEvent(fullPath);
  });

  watcher.on("error", (err) => {
    process.stderr.write(`Watcher error: ${err.message}\n`);
  });

  process.stderr.write(`Watching: ${resolvedDir}\n`);
}

function stopWatching() {
  if (watcher) {
    watcher.close();
    watcher = null;
  }
  for (const timer of debounceTimers.values()) {
    clearTimeout(timer);
  }
  debounceTimers.clear();
}

// --- JSON-RPC stdin handler ---
const rl = readline.createInterface({ input: process.stdin });

rl.on("line", (line) => {
  let msg;
  try {
    msg = JSON.parse(line);
  } catch {
    return;
  }

  if (msg.method === "initialize" && msg.id != null) {
    sendResponse(msg.id, {
      name: "file-watcher",
      version: "1.0.0",
      description: "Watches a directory for file changes",
      watchDir: resolvedDir,
      ignorePatterns,
      debounceMs,
    });
    startWatching();
  } else if (msg.method === "shutdown" && msg.id != null) {
    stopWatching();
    sendResponse(msg.id, { status: "ok" });
    process.exit(0);
  }
});

rl.on("close", () => {
  stopWatching();
  process.exit(0);
});
