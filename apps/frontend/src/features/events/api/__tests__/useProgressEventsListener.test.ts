import { createRoot } from 'solid-js';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ProgressEvent } from '../../model/progressEvent';

const ipc = vi.hoisted(() => ({
  deliver: undefined as ((e: { payload: ProgressEvent }) => void) | undefined,
}));

vi.mock('@/platform/ipc', () => ({
  eventsCommands: { listProgressBars: vi.fn().mockResolvedValue([]) },
  eventsEvents: { progressEventDto: {} },
  useEventListener: (
    _: unknown,
    cb: (e: { payload: ProgressEvent }) => void,
  ) => {
    ipc.deliver = cb;
  },
}));
vi.mock('@/shared/i18n', () => ({
  useTranslation: () => [{ t: (key: string) => key }],
}));
vi.mock('@/shared/log', () => ({ logDebug: vi.fn(), logError: vi.fn() }));

import { useProgressStore } from '../../model/progressStore';
import { useProgressEventsListener } from '../useProgressEventsListener';

const event = (
  id: string,
  status: ProgressEvent['status'],
  mirrorOf: string | null,
): ProgressEvent =>
  ({
    event: { type: 'minecraft_download' },
    progressBarId: id,
    message: '',
    fraction: 0,
    stage: null,
    status,
    parentId: null,
    mirrorOf,
    bytesDone: null,
    bytesTotal: null,
    speed: null,
    cancel: null,
  }) as unknown as ProgressEvent;

const flushed = () => new Promise((r) => setTimeout(r, 50));

describe('useProgressEventsListener', () => {
  beforeEach(() => {
    const [, set] = useProgressStore();
    set('payloads', {});
  });

  it('should_leave_a_cancelled_mirror_out_when_the_shared_download_is_called_off', async () => {
    const [store] = useProgressStore();
    createRoot(() => useProgressEventsListener());

    ipc.deliver?.({ payload: event('m', { type: 'running' }, 's') });
    await flushed();
    expect(store.payloads['m']).toBeDefined();

    ipc.deliver?.({ payload: event('m', { type: 'cancelled' }, 's') });
    await flushed();

    expect(store.payloads['m']).toBeUndefined();
  });

  it('should_keep_a_cancelled_bar_that_is_no_mirror_when_it_is_called_off', async () => {
    const [store] = useProgressStore();
    createRoot(() => useProgressEventsListener());

    ipc.deliver?.({ payload: event('b', { type: 'cancelled' }, null) });
    await flushed();

    expect(store.payloads['b']?.status.type).toBe('cancelled');
  });
});
