import { error } from '@tauri-apps/plugin-log';

import { isDebug } from './debug';

const MAX_MESSAGE_LENGTH = 2000;
const MAX_STACK_LINES = 5;
const DUPLICATE_WINDOW_MS = 10_000;
const RATE_WINDOW_MS = 60_000;
const MAX_PER_WINDOW = 20;

const lastSeen = new Map<string, number>();
let sentAt: number[] = [];

const isLauncherErrorLike = (
  value: unknown,
): value is { type: string; payload: { code: string } } => {
  if (typeof value !== 'object' || value === null) {
    return false;
  }

  const { type, payload } = value as { type?: unknown; payload?: unknown };
  return (
    typeof type === 'string' &&
    typeof payload === 'object' &&
    payload !== null &&
    typeof (payload as { code?: unknown }).code === 'string'
  );
};

const safeStringify = (value: unknown): string => {
  try {
    const seen = new WeakSet<object>();
    const json = JSON.stringify(value, (_key, v: unknown) => {
      if (typeof v === 'bigint') {
        return v.toString();
      }

      if (typeof v === 'object' && v !== null) {
        if (seen.has(v)) {
          return '[Circular]';
        }

        seen.add(v);
      }

      return v;
    });
    // JSON.stringify yields undefined for undefined, functions and symbols
    return json ?? String(value);
  } catch {
    return '[unserializable]';
  }
};

const formatOne = (value: unknown): string => {
  if (value instanceof Error) {
    const stack = (value.stack ?? '').split('\n');
    // V8 stacks start with the "name: message" line, which is already emitted below
    const frames = stack
      .filter((line) => /^\s+at\s/.test(line))
      .slice(0, MAX_STACK_LINES);
    return [`${value.name}: ${value.message}`, ...frames].join('\n');
  }

  if (isLauncherErrorLike(value)) {
    return `${value.type}.${value.payload.code}`;
  }

  if (typeof value === 'string') {
    return value;
  }

  return safeStringify(value);
};

export const formatLogData = (data: unknown[]): string => {
  const text = data.map(formatOne).join(' ');
  return text.length > MAX_MESSAGE_LENGTH
    ? text.slice(0, MAX_MESSAGE_LENGTH)
    : text;
};

const shouldSend = (message: string): boolean => {
  const now = Date.now();

  for (const [key, time] of lastSeen) {
    if (now - time >= DUPLICATE_WINDOW_MS) {
      lastSeen.delete(key);
    }
  }

  if (lastSeen.has(message)) {
    return false;
  }

  sentAt = sentAt.filter((time) => now - time < RATE_WINDOW_MS);

  if (sentAt.length >= MAX_PER_WINDOW) {
    return false;
  }

  lastSeen.set(message, now);
  sentAt.push(now);
  return true;
};

export const logError = (...data: unknown[]) => {
  if (isDebug()) {
    // oxlint-disable-next-line no-console
    console.error(...data);
  }

  try {
    const message = formatLogData(data);

    if (shouldSend(message)) {
      // Nothing may be logged on failure: the logger would recurse into itself
      void error(message).catch(() => {});
    }
  } catch {
    // Outside Tauri (tests, web mode) the plugin throws synchronously
  }
};

export const logDebug = (...data: unknown[]) => {
  if (isDebug()) {
    // oxlint-disable-next-line no-console
    console.debug(...data);
  }
};
