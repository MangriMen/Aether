import { createRoot } from 'solid-js';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ProcessEvent } from '../../model/processEvent';

const ipc = vi.hoisted(() => ({
  deliver: undefined as ((e: { payload: ProcessEvent }) => void) | undefined,
  registered: undefined as (() => void) | undefined,
  list: vi.fn(),
}));

vi.mock('@/platform/ipc', () => ({
  processCommands: { list: ipc.list },
  processEvents: { processEventDto: {} },
  useReadyEventListener: (
    _: unknown,
    cb: (e: { payload: ProcessEvent }) => void,
  ) => {
    ipc.deliver = cb;

    return new Promise<void>((resolve) => {
      ipc.registered = resolve;
    });
  },
}));
vi.mock('@/shared/log', () => ({ logDebug: vi.fn(), logError: vi.fn() }));

import { useRunningProcessesListener } from '../useRunningProcessesListener';

const snapshotEntry = (
  instanceId: string,
  processId: string,
  adopted = false,
) => ({
  uuid: processId,
  instanceId,
  startTime: '',
  adopted,
});

const processEvent = (
  kind: ProcessEvent['event'],
  instanceId: string,
  processId: string,
): { payload: ProcessEvent } => ({
  payload: {
    seq: 1,
    event: kind,
    instanceId,
    processId,
  } as unknown as ProcessEvent,
});

const flushed = () => new Promise((r) => setTimeout(r, 0));

describe('useRunningProcessesListener', () => {
  beforeEach(() => {
    ipc.list.mockReset();
    ipc.deliver = undefined;
    ipc.registered = undefined;
  });

  const mount = () => {
    const updates: unknown[] = [];

    createRoot(() => {
      useRunningProcessesListener((u) => updates.push(u));
    });

    return updates;
  };

  it('should_report_running_when_an_adopted_event_arrives', async () => {
    ipc.list.mockResolvedValue([]);
    const updates = mount();
    ipc.registered?.();
    await flushed();

    ipc.deliver?.(processEvent('adopted', 'a', 'p1'));

    expect(updates).toEqual([
      { instanceId: 'a', processId: 'p1', isRunning: true, adopted: true },
    ]);
  });

  it('should_report_running_and_adopted_when_the_snapshot_has_an_adopted_game', async () => {
    ipc.list.mockResolvedValue([snapshotEntry('a', 'p1', true)]);
    const updates = mount();
    ipc.registered?.();
    await flushed();

    expect(updates).toEqual([
      { instanceId: 'a', processId: 'p1', isRunning: true, adopted: true },
    ]);
  });

  it('should_not_take_the_snapshot_before_the_listener_is_registered', async () => {
    ipc.list.mockResolvedValue([]);
    mount();
    await flushed();

    expect(ipc.list).not.toHaveBeenCalled();

    ipc.registered?.();
    await flushed();

    expect(ipc.list).toHaveBeenCalledOnce();
  });

  it('should_leave_no_ghost_when_the_game_finished_before_the_snapshot_was_applied', async () => {
    let resolveList: (v: unknown[]) => void = () => {};
    ipc.list.mockReturnValue(
      new Promise((r) => {
        resolveList = r;
      }),
    );
    const updates = mount();
    ipc.registered?.();
    await flushed();

    // The snapshot was taken while the game ran, then the game finished.
    ipc.deliver?.(processEvent('finished', 'a', 'p1'));
    resolveList([snapshotEntry('a', 'p1')]);
    await flushed();

    expect(updates).toEqual([
      { instanceId: 'a', processId: 'p1', isRunning: false, adopted: false },
    ]);
  });
});
