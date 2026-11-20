import { describe, expect, it } from 'vitest';

import {
  applyLaunchChange,
  applyLoadingChange,
  applyProcessSnapshot,
  applyRunningProcessUpdate,
  createRunningInstancesStore,
  getRemovalBlock,
  isStoppable,
} from '../runningInstances';

describe('getRemovalBlock', () => {
  it('blocks removal while the game runs, even when the install is incomplete', () => {
    const running = { isLoading: false, isRunning: true };

    expect(getRemovalBlock('installed', running)).toBe('running');
    expect(getRemovalBlock('not_installed', running)).toBe('running');
  });

  it('blocks removal while the game launches', () => {
    expect(
      getRemovalBlock('installed', { isLoading: true, isRunning: false }),
    ).toBe('launching');
  });

  it('allows removal of an installed instance that is not running', () => {
    expect(getRemovalBlock('installed', undefined)).toBeUndefined();
  });

  it('allows removal again after the process finishes', () => {
    const running = applyRunningProcessUpdate(
      {},
      { instanceId: 'a', processId: 'p1', isRunning: true },
    );

    expect(getRemovalBlock('installed', running.a)).toBe('running');

    const finished = applyRunningProcessUpdate(running, {
      instanceId: 'a',
      processId: 'p1',
      isRunning: false,
    });

    expect(getRemovalBlock('installed', finished.a)).toBeUndefined();
  });

  it('offers force only for an instance that is not installed', () => {
    expect(getRemovalBlock('not_installed', undefined)).toBe('notInstalled');
  });
});

describe('isStoppable', () => {
  it('stops a game that runs and a launch that has no game yet', () => {
    expect(isStoppable({ isRunning: true })).toBe(true);
    expect(isStoppable({ isRunning: false, isLaunching: true })).toBe(true);
  });

  it('has nothing to stop otherwise', () => {
    expect(isStoppable({ isRunning: false })).toBe(false);
    expect(isStoppable(undefined)).toBe(false);
  });
});

describe('applyProcessSnapshot', () => {
  it('drops a game that the fresh list does not have', () => {
    const instances = applyProcessSnapshot(
      { a: { processId: 'p1', isLoading: false, isRunning: true } },
      [],
    );

    expect(instances).toEqual({});
  });

  it('adds a game that the events missed and keeps a launch without a process', () => {
    const instances = applyProcessSnapshot(
      { b: { processId: undefined, isLoading: true, isRunning: false } },
      [{ instanceId: 'a', processId: 'p1' }],
    );

    expect(instances).toEqual({
      a: { processId: 'p1', isLoading: false, isRunning: true },
      b: { processId: undefined, isLoading: true, isRunning: false },
    });
  });
});

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

describe('applyLaunchChange', () => {
  it('marks a launch and leaves nothing behind when it fails', () => {
    const launching = applyLaunchChange({}, 'a', true);

    expect(launching.a).toEqual({
      processId: undefined,
      isLoading: true,
      isRunning: false,
      isLaunching: true,
    });
    expect(applyLaunchChange(launching, 'a', false)).toEqual({});
  });

  it('keeps the game of a launch that ends, without the launch', () => {
    const running = applyRunningProcessUpdate(
      applyLaunchChange({}, 'a', true),
      {
        instanceId: 'a',
        processId: 'p1',
        isRunning: true,
      },
    );

    expect(running.a).toEqual({
      processId: 'p1',
      isLoading: false,
      isRunning: true,
    });
    expect(applyLaunchChange(running, 'a', false).a).toEqual({
      processId: 'p1',
      isLoading: false,
      isRunning: true,
    });
  });

  it('ignores the end of a launch whose game has finished already', () => {
    const running = applyRunningProcessUpdate(
      applyLaunchChange({}, 'a', true),
      {
        instanceId: 'a',
        processId: 'p1',
        isRunning: true,
      },
    );
    const finished = applyRunningProcessUpdate(running, {
      instanceId: 'a',
      processId: 'p1',
      isRunning: false,
    });

    expect(applyLaunchChange(finished, 'a', false)).toEqual({});
  });

  it('does not touch the launch of another instance', () => {
    const instances = applyLaunchChange(
      applyLaunchChange({}, 'a', true),
      'b',
      true,
    );

    expect(applyLaunchChange(instances, 'b', false).a?.isLaunching).toBe(true);
  });
});

describe('adopted games', () => {
  it('should_mark_running_and_adopted_when_an_adopted_update_arrives', () => {
    const next = applyRunningProcessUpdate(
      {},
      { instanceId: 'a', processId: 'p1', isRunning: true, adopted: true },
    );

    expect(next.a).toEqual({
      processId: 'p1',
      isRunning: true,
      isLoading: false,
      adopted: true,
    });
  });

  it('should_mark_running_and_adopted_when_the_snapshot_has_an_adopted_game', () => {
    const next = applyProcessSnapshot({}, [
      { instanceId: 'a', processId: 'p1', adopted: true },
      { instanceId: 'b', processId: 'p2', adopted: false },
    ]);

    expect(next.a).toMatchObject({ isRunning: true, adopted: true });
    expect(next.b).toMatchObject({ isRunning: true });
    expect(next.b?.adopted).toBeUndefined();
  });
});
