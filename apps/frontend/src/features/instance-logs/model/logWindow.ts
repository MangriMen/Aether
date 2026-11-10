import type { LogPage } from './gameLog';
import type { LogLevel, LogLine } from './logLines';

import { parseLogText } from './logLines';

/** Most lines kept in memory; the far end of the window is let go past this. */
export const MAX_WINDOW_LINES = 20_000;
/** A chunk that grows with live lines is closed at this size, so whole chunks can be dropped. */
const MAX_CHUNK_LINES = 2_000;

/** Lines of one byte range of the log file. */
type LogChunk = {
  start: number;
  end: number;
  lines: LogLine[];
};

export type LogWindow = {
  chunks: LogChunk[];
  /** The file has lines before the first chunk. */
  hasMoreBefore: boolean;
  /** The last chunk reaches the end of the file as far as is known, so live lines can follow it. */
  tailLoaded: boolean;
};

export const EMPTY_WINDOW: LogWindow = {
  chunks: [],
  hasMoreBefore: false,
  tailLoaded: false,
};

export const windowStart = (window: LogWindow): number =>
  window.chunks[0]?.start ?? 0;

export const windowEnd = (window: LogWindow): number =>
  window.chunks[window.chunks.length - 1]?.end ?? 0;

export const windowLines = (window: LogWindow): LogLine[] =>
  window.chunks.flatMap((chunk) => chunk.lines);

const lineCount = (window: LogWindow): number =>
  window.chunks.reduce((sum, chunk) => sum + chunk.lines.length, 0);

const lastLevel = (window: LogWindow): LogLevel | undefined => {
  const lastChunk = window.chunks[window.chunks.length - 1];

  return lastChunk?.lines[lastChunk.lines.length - 1]?.level;
};

export const windowFromTail = (page: LogPage): LogWindow => ({
  chunks: [
    { start: page.start, end: page.end, lines: parseLogText(page.text) },
  ],
  hasMoreBefore: page.hasMoreBefore,
  tailLoaded: !page.hasMoreAfter,
});

/** Adds the lines that follow the window; the window then drops its oldest chunks if too long. */
export const appendText = (
  window: LogWindow,
  start: number,
  end: number,
  text: string,
): LogWindow => {
  const lines = parseLogText(text, lastLevel(window));
  const last = window.chunks[window.chunks.length - 1];

  let chunks: LogChunk[];

  if (last && last.lines.length < MAX_CHUNK_LINES) {
    chunks = [
      ...window.chunks.slice(0, -1),
      { start: last.start, end, lines: [...last.lines, ...lines] },
    ];
  } else {
    chunks = [...window.chunks, { start, end, lines }];
  }

  return dropFromTop({ ...window, chunks });
};

/** Adds the lines that precede the window; the window then drops its newest chunks if too long. */
export const prependPage = (window: LogWindow, page: LogPage): LogWindow => {
  const chunk: LogChunk = {
    start: page.start,
    end: page.end,
    lines: parseLogText(page.text),
  };

  return dropFromBottom({
    chunks: [chunk, ...window.chunks],
    hasMoreBefore: page.hasMoreBefore,
    tailLoaded: window.tailLoaded,
  });
};

const dropFromTop = (window: LogWindow): LogWindow => {
  let { chunks } = window;
  let hasMoreBefore = window.hasMoreBefore;
  let count = lineCount(window);

  while (count > MAX_WINDOW_LINES && chunks.length > 1) {
    count -= chunks[0]?.lines.length ?? 0;
    chunks = chunks.slice(1);
    hasMoreBefore = true;
  }

  return { ...window, chunks, hasMoreBefore };
};

const dropFromBottom = (window: LogWindow): LogWindow => {
  let { chunks } = window;
  let tailLoaded = window.tailLoaded;
  let count = lineCount(window);

  while (count > MAX_WINDOW_LINES && chunks.length > 1) {
    count -= chunks[chunks.length - 1]?.lines.length ?? 0;
    chunks = chunks.slice(0, -1);
    tailLoaded = false;
  }

  return { ...window, chunks, tailLoaded };
};

export type LiveAction = 'append' | 'ignore' | 'read';

/**
 * What to do with lines announced for a byte range. Their text is used only when it continues
 * the window exactly; a range seen already is dropped, and anything else (a gap, or text too
 * big to send) is read from the file.
 */
export const decideLive = (
  window: LogWindow,
  event: { start: number; end: number; text: string },
): LiveAction => {
  const end = windowEnd(window);

  if (event.end <= end) {
    return 'ignore';
  }

  return event.start === end && event.text !== '' ? 'append' : 'read';
};
