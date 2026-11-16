import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { RunningProcessUpdate } from '@/features/process';

import { useRunningInstancesContext } from '../../model/runningInstancesContext';
import { RunningInstancesProvider } from '../RunningInstancesProvider';

const listener = vi.hoisted(() => ({
  callback: undefined as ((update: RunningProcessUpdate) => void) | undefined,
}));

const launches = vi.hoisted(() => ({
  callback: undefined as
    | ((instanceId: string, isLaunching: boolean) => void)
    | undefined,
}));

vi.mock('../../api/useLaunchStateListener', () => ({
  useLaunchStateListener: (
    callback: (instanceId: string, isLaunching: boolean) => void,
  ) => {
    launches.callback = callback;
  },
}));

vi.mock('@/features/process', () => ({
  useRunningProcessesListener: (
    callback: (update: RunningProcessUpdate) => void,
  ) => {
    listener.callback = callback;
  },
}));

const INSTANCE_ID = 'instance-1';
const PROCESS_ID = 'process-1';

const Consumer = () => {
  const [context, { get }] = useRunningInstancesContext();
  const running = () => get(context, INSTANCE_ID);

  return (
    <div>
      <div data-testid='process-id'>{running()?.processId ?? 'none'}</div>
      <div data-testid='launching'>{String(running()?.isLaunching)}</div>
    </div>
  );
};

let dispose: (() => void) | undefined;

const mount = () => {
  const container = document.createElement('div');
  document.body.append(container);

  dispose = render(
    () => (
      <RunningInstancesProvider>
        <Consumer />
      </RunningInstancesProvider>
    ),
    container,
  );

  return container;
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  listener.callback = undefined;
  launches.callback = undefined;
  document.body.innerHTML = '';
});

describe('RunningInstancesProvider', () => {
  it('should_expose_a_process_update_to_consumers_when_the_listener_reports_it', async () => {
    const container = mount();

    expect(listener.callback).toBeDefined();

    listener.callback?.({
      instanceId: INSTANCE_ID,
      processId: PROCESS_ID,
      isRunning: true,
    });

    await expect.poll(() => container.textContent).toContain(PROCESS_ID);
  });

  it('should_show_a_launch_until_the_backend_tells_that_it_is_over', async () => {
    const container = mount();

    launches.callback?.(INSTANCE_ID, true);
    await expect.poll(() => container.textContent).toContain('true');

    launches.callback?.(INSTANCE_ID, false);
    await expect.poll(() => container.textContent).toContain('undefined');
  });
});
