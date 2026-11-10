import type { LogPage } from './gameLog';
import type { LineFilter, LogLevel, LogLine } from './logLines';

import { parseLogText } from './logLines';

/** Most lines kept in memory; the far end of the window is let go past this. */
export const MAX_WINDOW_LINES = 20_000;
/**
 * A log up to this size is read whole, so that filters and search see all of it; a larger one is
 * read in a window that follows the reader. Rows are drawn by a virtual list either way.
 */
export const FULL_LOAD_BYTES = 32 * 1024 * 1024;
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
  /** Most lines kept; the whole file is kept when it is small enough to read at once. */
  maxLines: number;
  /** The level of the last line read, which a continuation line at the next read belongs to. */
  tailLevel: LogLevel;
};

export const EMPTY_WINDOW: LogWindow = {
  chunks: [],
  hasMoreBefore: false,
  tailLoaded: false,
  maxLines: MAX_WINDOW_LINES,
  tailLevel: 'info',
};

export const windowStart = (window: LogWindow): number =>
  window.chunks[0]?.start ?? 0;

export const windowEnd = (window: LogWindow): number =>
  window.chunks[window.chunks.length - 1]?.end ?? 0;

export const windowLines = (window: LogWindow): LogLine[] =>
  window.chunks.flatMap((chunk) => chunk.lines);

export const windowLineCount = (window: LogWindow): number =>
  window.chunks.reduce((sum, chunk) => sum + chunk.lines.length, 0);

export const windowFromTail = (page: LogPage, keep?: LineFilter): LogWindow => {
  const read = parseLogText(page.text);

  return {
    chunks: [
      {
        start: page.start,
        end: page.end,
        lines: keep ? read.filter(keep) : read,
      },
    ],
    hasMoreBefore: page.hasMoreBefore,
    tailLoaded: !page.hasMoreAfter,
    maxLines:
      page.size <= FULL_LOAD_BYTES
        ? Number.POSITIVE_INFINITY
        : MAX_WINDOW_LINES,
    tailLevel: read[read.length - 1]?.level ?? 'info',
  };
};

/** Adds the lines that follow the window; the window then drops its oldest chunks if too long. */
export const appendText = (
  window: LogWindow,
  start: number,
  end: number,
  text: string,
  keep?: LineFilter,
): LogWindow => {
  const read = parseLogText(text, window.tailLevel);
  const lines = keep ? read.filter(keep) : read;
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

  return dropFromTop({
    ...window,
    chunks,
    tailLevel: read[read.length - 1]?.level ?? window.tailLevel,
  });
};

/** Adds the lines that precede the window; the window then drops its newest chunks if too long. */
export const prependPage = (
  window: LogWindow,
  page: LogPage,
  keep?: LineFilter,
): LogWindow => {
  const read = parseLogText(page.text);
  const chunk: LogChunk = {
    start: page.start,
    end: page.end,
    lines: keep ? read.filter(keep) : read,
  };

  return dropFromBottom({
    ...window,
    chunks: [chunk, ...window.chunks],
    hasMoreBefore: page.hasMoreBefore,
  });
};

const dropFromTop = (window: LogWindow): LogWindow => {
  let { chunks } = window;
  let hasMoreBefore = window.hasMoreBefore;
  let count = windowLineCount(window);

  while (count > window.maxLines && chunks.length > 1) {
    count -= chunks[0]?.lines.length ?? 0;
    chunks = chunks.slice(1);
    hasMoreBefore = true;
  }

  return { ...window, chunks, hasMoreBefore };
};

const dropFromBottom = (window: LogWindow): LogWindow => {
  let { chunks } = window;
  let tailLoaded = window.tailLoaded;
  let count = windowLineCount(window);

  while (count > window.maxLines && chunks.length > 1) {
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

/** Every line of the file is (or will be) kept, so a filter over the window is a filter over the file. */
export const holdsWholeFile = (window: LogWindow): boolean =>
  window.maxLines === Number.POSITIVE_INFINITY;
