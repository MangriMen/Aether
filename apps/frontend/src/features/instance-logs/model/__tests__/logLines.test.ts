import { describe, expect, it } from 'vitest';

import { createLineFilter, filterLogLines, parseLogText } from '../logLines';

describe('parseLogText', () => {
  it('should_read_the_level_when_a_line_has_one', () => {
    const lines = parseLogText(
      '[12:00:00] [main/WARN]: careful\n[12:00:01] [Render thread/FATAL]: boom\n',
    );

    expect(lines.map((line) => line.level)).toEqual(['warn', 'error']);
  });

  it('should_give_a_stack_trace_the_level_of_its_entry_when_it_has_none', () => {
    const lines = parseLogText(
      '[12:00:00] [main/ERROR]: crash\njava.lang.Error: x\n\tat Foo.bar(Foo.java:1)\n',
    );

    expect(lines.map((line) => line.level)).toEqual([
      'error',
      'error',
      'error',
    ]);
  });

  it('should_continue_the_previous_level_when_a_chunk_starts_with_a_continuation', () => {
    const lines = parseLogText('\tat Foo.bar(Foo.java:1)\n', 'error');

    expect(lines[0]?.level).toBe('error');
  });

  it('should_strip_carriage_returns_and_the_trailing_newline', () => {
    expect(parseLogText('a\r\nb\r\n').map((line) => line.text)).toEqual([
      'a',
      'b',
    ]);
  });

  it('should_keep_a_blank_line_between_entries', () => {
    expect(parseLogText('a\n\nb\n').map((line) => line.text)).toEqual([
      'a',
      '',
      'b',
    ]);
  });

  it('should_return_no_lines_when_the_text_is_empty', () => {
    expect(parseLogText('')).toEqual([]);
  });
});

describe('filterLogLines', () => {
  const lines = parseLogText(
    '[1] [main/INFO]: hello world\n[2] [main/WARN]: Hello again\n[3] [main/ERROR]: bad\n',
  );

  it('should_return_every_line_when_nothing_is_filtered', () => {
    expect(filterLogLines(lines, createLineFilter(new Set(), ''))).toBe(lines);
  });

  it('should_keep_only_the_chosen_levels', () => {
    const result = filterLogLines(
      lines,
      createLineFilter(new Set(['warn', 'error']), ''),
    );

    expect(result.map((line) => line.level)).toEqual(['warn', 'error']);
  });

  it('should_match_the_text_without_regard_to_case', () => {
    const result = filterLogLines(lines, createLineFilter(new Set(), 'HELLO'));

    expect(result).toHaveLength(2);
  });
});
