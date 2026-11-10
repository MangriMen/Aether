import {
  createEffect,
  createMemo,
  createSignal,
  on,
  untrack,
  type Accessor,
} from 'solid-js';

import type { GameLogEvent } from '../model/gameLog';
import type { LineFilter } from '../model/logLines';

import { readGameLogTail } from '../api/gameLogApi';
import { FULL_LOAD_BYTES } from '../model/constants';
import {
  EMPTY_WINDOW,
  appendText,
  decideLive,
  holdsWholeFile,
  windowFromTail,
  windowLines,
  type LogWindow,
} from '../model/logWindow';
import { createLogPager } from './createLogPager';
import { createLogTasks } from './createLogTasks';

export type LogViewProps = {
  instanceId: Accessor<string>;
  sourceId: Accessor<string | undefined>;
  /**
   * What the shown lines must match. A log that is read whole is filtered by the caller; for a
   * larger one it is the view that reads through the file and keeps only the matching lines.
   */
  lineFilter: Accessor<LineFilter | undefined>;
};

/**
 * The lines of one log file, kept as a window over it: the last lines are read first, older
 * ones on request, and lines announced while the game runs are added at the end. Everything
 * that touches the window runs one after another, so a read and an announcement for the same
 * range cannot both add it.
 */
export const createLogView = ({
  instanceId,
  sourceId,
  lineFilter,
}: LogViewProps) => {
  // The reads run in async tasks, outside any reactive scope, and want the value of the moment.
  const currentInstanceId = () => untrack(instanceId);
  const currentSourceId = () => untrack(sourceId);
  const currentLineFilter = () => untrack(lineFilter);

  const [window, setWindow] = createSignal<LogWindow>(EMPTY_WINDOW);
  const [loaded, setLoaded] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [loadingOlder, setLoadingOlder] = createSignal(false);
  // Set while a large log is read through for the lines that match the filter.
  const [searching, setSearching] = createSignal(false);
  let windowFilter: LineFilter | undefined;
  const [error, setError] = createSignal<unknown>();

  const tasks = createLogTasks();
  const pager = createLogPager({
    instanceId: currentInstanceId,
    sourceId: currentSourceId,
    window,
    setWindow,
    windowFilter: () => windowFilter,
    tasks,
    setLoadingOlder,
  });

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

      if (current !== tasks.current()) {
        return;
      }

      windowFilter =
        page.size > FULL_LOAD_BYTES ? currentLineFilter() : undefined;
      setSearching(windowFilter !== undefined);
      setWindow(windowFromTail(page, windowFilter));
      setLoaded(true);
      void tasks.enqueue(() => pager.loadRestOfFile(current));
    } catch (err) {
      if (current === tasks.current()) {
        setError(err);
      }
    } finally {
      if (current === tasks.current()) {
        setLoading(false);
      }
    }
  };

  createEffect(
    on([instanceId, sourceId], () => {
      const current = tasks.next();

      setWindow(EMPTY_WINDOW);
      setLoaded(false);
      setError(undefined);

      if (currentSourceId() !== undefined) {
        void tasks.enqueue(() => loadTail(current));
      }
    }),
  );

  const applyLive = async (current: number, event: GameLogEvent) => {
    const sourceId = currentSourceId();

    if (current !== tasks.current() || !loaded() || sourceId === undefined) {
      return;
    }

    const state = window();

    if (!state.tailLoaded) {
      return;
    }

    const action = decideLive(state, event);

    if (action === 'append') {
      setWindow(
        appendText(state, event.start, event.end, event.text, windowFilter),
      );
    } else if (action === 'read') {
      await pager.readUntilEnd(current, sourceId);
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

    const current = tasks.current();

    void tasks.enqueue(() => untrack(() => applyLive(current, event)));
  };

  /** Back to the end of the file after the window was moved away from it. */
  const reloadTail = () => {
    const current = tasks.next();

    return tasks.enqueue(() => loadTail(current));
  };

  // The filter changed under a log that is read in a window: what it keeps is another set of lines.
  createEffect(
    on(
      lineFilter,
      () => {
        if (loaded() && !holdsWholeFile(untrack(window))) {
          void reloadTail();
        }
      },
      { defer: true },
    ),
  );

  const lines = createMemo(() => windowLines(window()));

  return {
    lines,
    loading,
    loadingOlder,
    /** A search that stopped at the limit: more lines match than are shown. */
    searchCut: () => searching() && window().hasMoreBefore && !loadingOlder(),
    error,
    hasMoreBefore: () => window().hasMoreBefore,
    tailLoaded: () => window().tailLoaded,
    pushLive,
    loadOlder: pager.loadOlder,
    loadNewer: pager.loadNewer,
    reloadTail,
  };
};
