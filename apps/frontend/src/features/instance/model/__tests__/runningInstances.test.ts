import { describe, expect, it } from 'vitest';

import { applyRunningProcessUpdate } from '../runningInstances';

describe('applyRunningProcessUpdate', () => {
  it('adds a running instance', () => {
    const instances = applyRunningProcessUpdate(
      {},
      { instanceId: 'a', processId: 'p1', isRunning: true },
    );

    expect(instances).toEqual({
      a: { processId: 'p1', isLoading: false, isRunning: true },
    });
  });

  it('drops the entry when its process finishes', () => {
    const instances = applyRunningProcessUpdate(
      { a: { processId: 'p1', isLoading: false, isRunning: true } },
      { instanceId: 'a', processId: 'p1', isRunning: false },
    );

    expect(instances).toEqual({});
  });

  it('keeps the entry of a newer process when an older one finishes', () => {
    const current = {
      a: { processId: 'p2', isLoading: false, isRunning: true },
    };
    const instances = applyRunningProcessUpdate(current, {
      instanceId: 'a',
      processId: 'p1',
      isRunning: false,
    });

    expect(instances).toBe(current);
  });

  it('keeps a launching entry without a process when an old process finishes', () => {
    const current = {
      a: { processId: undefined, isLoading: true, isRunning: false },
    };
    const instances = applyRunningProcessUpdate(current, {
      instanceId: 'a',
      processId: 'p1',
      isRunning: false,
    });

    expect(instances).toBe(current);
  });

  it('ignores the end of an unknown instance', () => {
    const current = {};
    const instances = applyRunningProcessUpdate(current, {
      instanceId: 'a',
      processId: 'p1',
      isRunning: false,
    });

    expect(instances).toBe(current);
  });
});
