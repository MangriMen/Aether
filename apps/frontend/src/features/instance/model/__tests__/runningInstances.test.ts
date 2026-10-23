import { describe, expect, it } from 'vitest';

import {
  applyLoadingChange,
  applyRunningProcessUpdate,
  createRunningInstancesStore,
} from '../runningInstances';

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

describe('applyLoadingChange', () => {
  it('creates a loading entry', () => {
    expect(applyLoadingChange({}, 'a', true)).toEqual({
      a: { processId: undefined, isLoading: true, isRunning: false },
    });
  });

  it('drops the entry of a launch that failed', () => {
    const instances = applyLoadingChange(
      { a: { processId: undefined, isLoading: true, isRunning: false } },
      'a',
      false,
    );

    expect(instances).toEqual({});
  });

  it('keeps a running entry and only clears its loading flag', () => {
    const instances = applyLoadingChange(
      { a: { processId: 'p1', isLoading: true, isRunning: true } },
      'a',
      false,
    );

    expect(instances).toEqual({
      a: { processId: 'p1', isLoading: false, isRunning: true },
    });
  });

  it('ignores clearing the loading flag of an unknown instance', () => {
    const current = {};

    expect(applyLoadingChange(current, 'a', false)).toBe(current);
  });
});

describe('createRunningInstancesStore', () => {
  const launched = { instanceId: 'a', processId: 'p1', isRunning: true };
  const finished = { instanceId: 'a', processId: 'p1', isRunning: false };

  it('leaves no instance behind when the launch failed', () => {
    const store = createRunningInstancesStore();

    store.applyLoadingChange('a', true);
    expect(store.state.instances['a']?.isLoading).toBe(true);

    store.applyLoadingChange('a', false);
    expect(store.state.instances['a']).toBeUndefined();
  });

  it('stops showing an instance as running when its process finishes', () => {
    const store = createRunningInstancesStore();

    store.applyProcessUpdate(launched);
    expect(store.state.instances['a']?.isRunning).toBe(true);

    store.applyProcessUpdate(finished);
    expect(store.state.instances['a']).toBeUndefined();
  });

  it('stops showing an instance as running when it is killed while stopping', () => {
    const store = createRunningInstancesStore();

    store.applyProcessUpdate(launched);
    store.applyLoadingChange('a', true);
    store.applyProcessUpdate(finished);
    store.applyLoadingChange('a', false);

    expect(store.state.instances['a']).toBeUndefined();
  });

  it('shows an instance as running again after a relaunch', () => {
    const store = createRunningInstancesStore();

    store.applyProcessUpdate(launched);
    store.applyProcessUpdate(finished);
    store.applyProcessUpdate({ ...launched, processId: 'p2' });

    expect(store.state.instances['a']).toEqual({
      processId: 'p2',
      isLoading: false,
      isRunning: true,
    });
  });
});
