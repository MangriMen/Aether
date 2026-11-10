import type { LogLevel } from './logLines';

/** A page of a log is read in pieces this big, so a huge log never travels at once. */
export const PAGE_BYTES = 256 * 1024;
/** A log read whole comes in bigger pieces, to be done in fewer round trips. */
export const WHOLE_FILE_PAGE_BYTES = 1024 * 1024;

/** Most lines kept in memory; the far end of the window is let go past this. */
export const MAX_WINDOW_LINES = 20_000;
/**
 * A log up to this size is read whole, so that filters and search see all of it; a larger one is
 * read in a window that follows the reader. Rows are drawn by a virtual list either way.
 */
export const FULL_LOAD_BYTES = 32 * 1024 * 1024;
/** A chunk that grows with live lines is closed at this size, so whole chunks can be dropped. */
export const MAX_CHUNK_LINES = 2_000;

/** Typing must not start a new search of a large log at every key. */
export const SEARCH_DEBOUNCE_MS = 300;

/** More lines are asked for when the view is this close to an edge of what is loaded. */
export const LOAD_MORE_THRESHOLD_PX = 400;
/** The view follows the end of the log while it is this close to it. */
export const STICK_THRESHOLD_PX = 40;

/** Without wrapping a line is one row of this height; wrapped rows are measured. */
export const ROW_HEIGHT = 22;
/** The widest a row is made, in characters, so one huge line does not make the list scroll for miles. */
export const MAX_ROW_CHARS = 4000;
/** Rows drawn above and below the ones in view. */
export const ROW_OVERSCAN = 30;

/** The select option standing for the newest run's console log, whichever file that is. */
export const LIVE_SOURCE = 'live';

/** The colour of the lines of each level. */
export const LEVEL_CLASS: Record<LogLevel, string> = {
  debug: 'text-muted-foreground/60',
  info: 'text-muted-foreground',
  warn: 'text-warning',
  error: 'text-destructive',
};

/** A pressed level chip takes the colour its lines have in the log. */
export const LEVEL_CHIP_CLASS: Record<LogLevel, string> = {
  debug: '',
  info: 'data-pressed:bg-info/20 data-pressed:text-info',
  warn: 'data-pressed:bg-warning/25 data-pressed:text-warning',
  error: 'data-pressed:bg-destructive/40 data-pressed:text-foreground',
};
