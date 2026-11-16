import { render } from 'solid-js/web';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useLaunchStateListener } from '../useLaunchStateListener';

const ipc = vi.hoisted(() => ({
  handler: undefined as
    | ((e: { payload: { event: string; instanceId: string } }) => void)
    | undefined,
  listLaunching: vi.fn<() => Promise<string[]>>(),
}));

vi.mock('@/platform/ipc', () => ({
  instanceEvents: { instanceEventDto: 'instance-event' },
  instanceCommands: { listLaunching: ipc.listLaunching },
  useEventListener: (_event: string, handler: typeof ipc.handler) => {
    ipc.handler = handler;
  },
}));

let dispose: (() => void) | undefined;

const mount = (callback: (id: string, isLaunching: boolean) => void) => {
  dispose = render(() => {
    useLaunchStateListener(callback);

    return <div />;
  }, document.body);
};

afterEach(() => {
  dispose?.();
  dispose = undefined;
  ipc.handler = undefined;
  ipc.listLaunching.mockReset();
});

describe('useLaunchStateListener', () => {
  it('reports the launches that began before the page was opened', async () => {
    ipc.listLaunching.mockResolvedValue(['a', 'b']);
    const callback = vi.fn();

    mount(callback);

    await vi.waitFor(() => expect(callback).toHaveBeenCalledTimes(2));
    expect(callback).toHaveBeenCalledWith('a', true);
    expect(callback).toHaveBeenCalledWith('b', true);
  });

  it('reports the start and the end of a launch from events, and nothing else', () => {
    ipc.listLaunching.mockResolvedValue([]);
    const callback = vi.fn();
    mount(callback);

    ipc.handler?.({ payload: { event: 'edited', instanceId: 'a' } });
    ipc.handler?.({ payload: { event: 'launch_started', instanceId: 'a' } });
    ipc.handler?.({ payload: { event: 'launch_ended', instanceId: 'a' } });

    expect(callback.mock.calls).toEqual([
      ['a', true],
      ['a', false],
    ]);
  });

  it('does not bring back a launch that ended while the list was loading', async () => {
    let resolve: ((ids: string[]) => void) | undefined;
    ipc.listLaunching.mockReturnValue(
      new Promise((done) => {
        resolve = done;
      }),
    );
    const callback = vi.fn();
    mount(callback);

    ipc.handler?.({ payload: { event: 'launch_ended', instanceId: 'a' } });
    resolve?.(['a', 'b']);

    await vi.waitFor(() => expect(callback).toHaveBeenCalledWith('b', true));
    expect(callback).not.toHaveBeenCalledWith('a', true);
  });
});
