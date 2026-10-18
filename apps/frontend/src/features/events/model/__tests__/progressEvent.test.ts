import { describe, expect, it } from 'vitest';

import type { ProgressEvent } from '../progressEvent';

import {
  getProgressRemoveDelay,
  isProgressFailed,
  PROGRESS_EVENT_FAILED_REMOVE_DELAY,
  PROGRESS_EVENT_REMOVE_DELAY,
} from '../progressEvent';

const make = (status: ProgressEvent['status']): ProgressEvent =>
  ({ status, fraction: null }) as ProgressEvent;

describe('progress event helpers', () => {
  it('keeps failed bars longer than finished ones', () => {
    expect(getProgressRemoveDelay(make({ type: 'failed', message: 'x' }))).toBe(
      PROGRESS_EVENT_FAILED_REMOVE_DELAY,
    );
    expect(getProgressRemoveDelay(make({ type: 'done' }))).toBe(
      PROGRESS_EVENT_REMOVE_DELAY,
    );
  });

  it('detects failure by status only', () => {
    expect(isProgressFailed(make({ type: 'failed', message: 'x' }))).toBe(true);
    expect(isProgressFailed(make({ type: 'running' }))).toBe(false);
  });
});
