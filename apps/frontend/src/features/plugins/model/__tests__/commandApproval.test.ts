import { describe, expect, it } from 'vitest';

import { byExpiry, secondsLeft } from '../commandApproval';

describe('secondsLeft', () => {
  it('should_round_up_and_stop_at_zero', () => {
    const expires = '2026-11-16T10:00:10.500Z';

    expect(secondsLeft(expires, Date.parse('2026-11-16T10:00:00Z'))).toBe(11);
    expect(secondsLeft(expires, Date.parse('2026-11-16T10:01:00Z'))).toBe(0);
    expect(secondsLeft('nonsense', 0)).toBe(0);
  });
});

describe('byExpiry', () => {
  it('should_put_the_oldest_question_first', () => {
    const sorted = byExpiry([
      { id: 'b', expiresAt: '2026-11-16T10:00:20Z' },
      { id: 'a', expiresAt: '2026-11-16T10:00:10Z' },
    ]);

    expect(sorted.map((x) => x.id)).toEqual(['a', 'b']);
  });
});
