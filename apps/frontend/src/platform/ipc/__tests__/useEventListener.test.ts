import type { Event, EventCallback, UnlistenFn } from '@tauri-apps/api/event';

import { createRoot } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';

import { useEventListener } from '../useEventListener';

type Payload = { seq: number };

const createEvent = () => {
  const unlisten = vi.fn<UnlistenFn>();
  let resolve: (fn: UnlistenFn) => void = () => {};
  let deliver: EventCallback<Payload> = () => {};
  const listen = vi.fn(
    (callback: EventCallback<Payload>) =>
      new Promise<UnlistenFn>((r) => {
        deliver = callback;
        resolve = r;
      }),
  );

  return {
    event: { listen },
    unlisten,
    resolve: () => resolve(unlisten),
    send: (seq: number) =>
      deliver({ event: 'test', id: seq, payload: { seq } } as Event<Payload>),
  };
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

  it('drops an event it has heard already, and one that came before the last', () => {
    const { event, send } = createEvent();
    const heard: number[] = [];
    createRoot(() => {
      useEventListener(event, (e) => heard.push(e.payload.seq));
    });

    // Numbers may skip: other events of the backend have their own.
    for (const seq of [1, 2, 5, 3, 5, 6, 2]) {
      send(seq);
    }

    expect(heard).toEqual([1, 2, 5, 6]);
  });

  it('keeps the count of each listener to itself', () => {
    const first = createEvent();
    const heardFirst: number[] = [];
    createRoot(() => {
      useEventListener(first.event, (e) => heardFirst.push(e.payload.seq));
    });
    first.send(7);

    // A listener that comes later hears what follows, not what it missed.
    const second = createEvent();
    const heardSecond: number[] = [];
    createRoot(() => {
      useEventListener(second.event, (e) => heardSecond.push(e.payload.seq));
    });
    second.send(3);
    second.send(8);

    expect(heardFirst).toEqual([7]);
    expect(heardSecond).toEqual([3, 8]);
  });
});
