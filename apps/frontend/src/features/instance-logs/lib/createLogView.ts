import {
  createEffect,
  createMemo,
  createSignal,
  on,
  untrack,
  type Accessor,
} from 'solid-js';

import { logError } from '@/shared/log';

import type { GameLogEvent } from '../model/gameLog';

import {
  readGameLogAfter,
  readGameLogBefore,
  readGameLogTail,
} from '../api/gameLogApi';
import {
  EMPTY_WINDOW,
  appendText,
  decideLive,
  prependPage,
  windowEnd,
  windowFromTail,
  windowLines,
  windowStart,
  type LogWindow,
} from '../model/logWindow';

export type LogViewProps = {
  instanceId: Accessor<string>;
  sourceId: Accessor<string | undefined>;
};

/**
 * The lines of one log file, kept as a window over it: the last lines are read first, older
 * ones on request, and lines announced while the game runs are added at the end. Everything
 * that touches the window runs one after another, so a read and an announcement for the same
 * range cannot both add it.
 */
export const createLogView = ({ instanceId, sourceId }: LogViewProps) => {
  // The reads run in async tasks, outside any reactive scope, and want the value of the moment.
  const currentInstanceId = () => untrack(instanceId);
  const currentSourceId = () => untrack(sourceId);

  const [window, setWindow] = createSignal<LogWindow>(EMPTY_WINDOW);
  const [loaded, setLoaded] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<unknown>();

  let generation = 0;
  let chain: Promise<void> = Promise.resolve();

  const enqueue = (task: () => Promise<void>) => {
    chain = chain.then(task).catch((err: unknown) => {
      logError('[LOGS] failed to update the log view', err);
    });

    return chain;
  };

  const loadTail = async (current: number) => {
    const instanceId = currentInstanceId();
    const sourceId = currentSourceId();

    if (sourceId === undefined) {
      return;
    }

    setLoading(true);
    setError(undefined);

    try {
      const page = await readGameLogTail(instanceId, sourceId);

      if (current !== generation) {
        return;
      }

      setWindow(windowFromTail(page));
      setLoaded(true);
    } catch (err) {
      if (current === generation) {
        setError(err);
      }
    } finally {
      if (current === generation) {
        setLoading(false);
      }
    }
  };

  createEffect(
    on([instanceId, sourceId], () => {
      generation += 1;
      const current = generation;

      setWindow(EMPTY_WINDOW);
      setLoaded(false);
      setError(undefined);

      if (currentSourceId() !== undefined) {
        void enqueue(() => loadTail(current));
      }
    }),
  );

  const readUntilEnd = async (current: number, sourceId: string) => {
    for (;;) {
      const page = await readGameLogAfter(
        currentInstanceId(),
        sourceId,
        windowEnd(window()),
      );

      if (current !== generation) {
        return;
      }

      if (page.end > windowEnd(window())) {
        setWindow((w) => appendText(w, page.start, page.end, page.text));
      }

      if (!page.hasMoreAfter || page.end === page.start) {
        return;
      }
    }
  };

  const applyLive = async (current: number, event: GameLogEvent) => {
    const sourceId = currentSourceId();

    if (current !== generation || !loaded() || sourceId === undefined) {
      return;
    }

    const state = window();

    if (!state.tailLoaded) {
      return;
    }

    const action = decideLive(state, event);

    if (action === 'append') {
      setWindow(appendText(state, event.start, event.end, event.text));
    } else if (action === 'read') {
      await readUntilEnd(current, sourceId);
    }
  };

  /** Lines announced by the backend; the ones for another log or instance are not of interest. */
  const pushLive = (event: GameLogEvent) => {
    if (
      event.instanceId !== currentInstanceId() ||
      event.sourceId !== currentSourceId()
    ) {
      return;
    }

    const current = generation;

    void enqueue(() => untrack(() => applyLive(current, event)));
  };

  const loadOlder = () => {
    const current = generation;

    return enqueue(async () => {
      const sourceId = currentSourceId();
      const state = untrack(window);

      if (
        current !== generation ||
        sourceId === undefined ||
        !state.hasMoreBefore
      ) {
        return;
      }

      const page = await readGameLogBefore(
        currentInstanceId(),
        sourceId,
        windowStart(state),
      );

      if (current !== generation) {
        return;
      }

      setWindow((w) => prependPage(w, page));
    });
  };

  /** Back to the end of the file after the window was moved away from it. */
  const reloadTail = () => {
    generation += 1;
    const current = generation;

    return enqueue(() => loadTail(current));
  };

  const lines = createMemo(() => windowLines(window()));

  return {
    lines,
    loaded,
    loading,
    error,
    hasMoreBefore: () => window().hasMoreBefore,
    tailLoaded: () => window().tailLoaded,
    pushLive,
    loadOlder,
    reloadTail,
  };
};
