import { describe, expect, it } from 'vitest';

import type { ProgressEvent } from '../progressEvent';

import { isStaleEvent } from '../progressEvent';

const make = (status: ProgressEvent['status']): ProgressEvent =>
  ({ status, fraction: null }) as ProgressEvent;

describe('isStaleEvent', () => {
  it('lets the first event of a bar and running updates through', () => {
    expect(isStaleEvent(undefined, make({ type: 'running' }))).toBe(false);
    expect(
      isStaleEvent(make({ type: 'running' }), make({ type: 'running' })),
    ).toBe(false);
  });

  it('does not let a running event bring a finished bar back', () => {
    for (const type of ['done', 'cancelled'] as const) {
      expect(isStaleEvent(make({ type }), make({ type: 'running' }))).toBe(
        true,
      );
    }

    expect(
      isStaleEvent(
        make({ type: 'failed', message: 'x' }),
        make({ type: 'running' }),
      ),
    ).toBe(true);
  });

  it('accepts the final event of a running bar', () => {
    expect(
      isStaleEvent(make({ type: 'running' }), make({ type: 'done' })),
    ).toBe(false);
  });
});
