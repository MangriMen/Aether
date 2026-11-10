import { describe, expect, it } from 'vitest';

import type { LogPage } from '../gameLog';

import {
  EMPTY_WINDOW,
  MAX_WINDOW_LINES,
  appendText,
  decideLive,
  prependPage,
  windowEnd,
  windowFromTail,
  windowLines,
  windowStart,
} from '../logWindow';

const page = (overrides: Partial<LogPage> = {}): LogPage => ({
  text: 'a\nb\n',
  start: 10,
  end: 14,
  hasMoreBefore: true,
  hasMoreAfter: false,
  ...overrides,
});

const manyLines = (count: number, prefix = 'l') =>
  Array.from({ length: count }, (_, i) => `${prefix}${i}\n`).join('');

describe('decideLive', () => {
  const window = windowFromTail(page());

  it('should_append_when_the_event_continues_the_window', () => {
    expect(decideLive(window, { start: 14, end: 20, text: 'c\n' })).toBe(
      'append',
    );
  });

  it('should_ignore_when_the_range_was_read_already', () => {
    expect(decideLive(window, { start: 10, end: 14, text: 'a\nb\n' })).toBe(
      'ignore',
    );
    expect(decideLive(window, { start: 12, end: 14, text: 'b\n' })).toBe(
      'ignore',
    );
  });

  it('should_read_when_there_is_a_gap_before_the_event', () => {
    expect(decideLive(window, { start: 18, end: 24, text: 'x\n' })).toBe(
      'read',
    );
  });

  it('should_read_when_the_event_overlaps_but_reaches_past_the_end', () => {
    expect(decideLive(window, { start: 12, end: 20, text: 'b\nc\n' })).toBe(
      'read',
    );
  });

  it('should_read_when_the_text_was_too_big_to_send', () => {
    expect(decideLive(window, { start: 14, end: 9000, text: '' })).toBe('read');
  });
});

describe('windowFromTail', () => {
  it('should_mark_the_tail_loaded_when_the_page_reaches_the_end', () => {
    const window = windowFromTail(page());

    expect(window.tailLoaded).toBe(true);
    expect(windowEnd(window)).toBe(14);
    expect(windowStart(window)).toBe(10);
  });
});

describe('appendText', () => {
  it('should_extend_the_end_and_keep_the_level_of_the_last_entry', () => {
    const base = windowFromTail(
      page({ text: '[1] [main/ERROR]: crash\n', end: 34 }),
    );

    const next = appendText(base, 34, 60, '\tat Foo.bar\n');

    expect(windowEnd(next)).toBe(60);
    expect(windowLines(next).map((line) => line.level)).toEqual([
      'error',
      'error',
    ]);
  });

  it('should_drop_the_oldest_lines_and_allow_reading_them_back_when_the_window_is_full', () => {
    let window = windowFromTail(
      page({ text: manyLines(MAX_WINDOW_LINES), start: 0, end: 100 }),
    );
    let end = 100;

    for (let i = 0; i < 5; i += 1) {
      window = appendText(window, end, end + 10, manyLines(1_000, 'n'));
      end += 10;
    }

    expect(windowLines(window).length).toBeLessThanOrEqual(
      MAX_WINDOW_LINES + 2_000,
    );
    expect(windowStart(window)).toBeGreaterThan(0);
    expect(window.hasMoreBefore).toBe(true);
    expect(windowEnd(window)).toBe(end);
  });
});

describe('prependPage', () => {
  it('should_put_older_lines_first_and_move_the_start', () => {
    const base = windowFromTail(page());

    const next = prependPage(
      base,
      page({ text: 'x\n', start: 6, end: 10, hasMoreBefore: false }),
    );

    expect(windowLines(next).map((line) => line.text)).toEqual(['x', 'a', 'b']);
    expect(windowStart(next)).toBe(6);
    expect(next.hasMoreBefore).toBe(false);
  });

  it('should_leave_the_tail_when_the_window_is_full_of_older_lines', () => {
    let window = windowFromTail(
      page({ text: manyLines(MAX_WINDOW_LINES), start: 100_000, end: 200_000 }),
    );

    window = prependPage(
      window,
      page({ text: manyLines(5_000, 'o'), start: 50_000, end: 100_000 }),
    );

    expect(window.tailLoaded).toBe(false);
    expect(windowLines(window).length).toBeLessThanOrEqual(
      MAX_WINDOW_LINES + 5_000,
    );
  });
});

describe('EMPTY_WINDOW', () => {
  it('should_start_without_a_tail', () => {
    expect(EMPTY_WINDOW.tailLoaded).toBe(false);
    expect(windowEnd(EMPTY_WINDOW)).toBe(0);
  });
});
