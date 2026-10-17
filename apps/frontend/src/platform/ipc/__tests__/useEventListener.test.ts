import type { EventCallback, UnlistenFn } from '@tauri-apps/api/event';

import { createRoot } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';

import { useEventListener } from '../useEventListener';

const createEvent = () => {
  const unlisten = vi.fn<UnlistenFn>();
  let resolve: (fn: UnlistenFn) => void = () => {};
  const listen = vi.fn(
    (_callback: EventCallback<string>) =>
      new Promise<UnlistenFn>((r) => {
        resolve = r;
      }),
  );

  return { event: { listen }, unlisten, resolve: () => resolve(unlisten) };
};

describe('useEventListener', () => {
  it('removes the listener on cleanup', async () => {
    const { event, unlisten, resolve } = createEvent();
    const dispose = createRoot((dispose) => {
      useEventListener(event, () => {});
      return dispose;
    });

    expect(event.listen).toHaveBeenCalledOnce();
    resolve();
    await Promise.resolve();
    dispose();

    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('removes the listener when cleanup comes before listen resolves', async () => {
    const { event, unlisten, resolve } = createEvent();
    const dispose = createRoot((dispose) => {
      useEventListener(event, () => {});
      return dispose;
    });

    dispose();
    expect(unlisten).not.toHaveBeenCalled();
    resolve();
    await Promise.resolve();

    expect(unlisten).toHaveBeenCalledOnce();
  });
});
