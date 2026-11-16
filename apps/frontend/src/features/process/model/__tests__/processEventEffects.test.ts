import { describe, expect, it } from 'vitest';

import type { ProcessEvent } from '../processEvent';

import { getRunningState } from '../processEventEffects';

const event = (kind: ProcessEvent['event']): ProcessEvent =>
  ({ event: kind }) as ProcessEvent;

describe('getRunningState', () => {
  it('should_mark_running_when_the_game_launched', () => {
    expect(getRunningState(event('launched'))).toBe(true);
  });

  it('should_mark_stopped_when_the_game_finished', () => {
    expect(getRunningState(event('finished'))).toBe(false);
  });

  it('should_not_change_running_state_when_the_post_exit_hook_failed', () => {
    expect(getRunningState(event('post_exit_hook_failed'))).toBeUndefined();
  });
});
