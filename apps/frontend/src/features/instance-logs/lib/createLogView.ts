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
import type { LineFilter } from '../model/logLines';

import {
  readGameLogAfter,
  readGameLogBefore,
  readGameLogTail,
  WHOLE_FILE_PAGE_BYTES,
} from '../api/gameLogApi';
import {
  EMPTY_WINDOW,
  appendText,
  decideLive,
  FULL_LOAD_BYTES,
  holdsWholeFile,
  windowLineCount,
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

      windowFilter =
        page.size > FULL_LOAD_BYTES ? currentLineFilter() : undefined;
      setSearching(windowFilter !== undefined);
      setWindow(windowFromTail(page, windowFilter));
      setLoaded(true);
      void enqueue(() => loadRestOfFile(current));
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
        setWindow((w) =>
          appendText(w, page.start, page.end, page.text, windowFilter),
        );
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
      setWindow(
        appendText(state, event.start, event.end, event.text, windowFilter),
      );
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

  /** One page above the window; false when there is nothing above it or the view moved on. */
  const readOlderPage = async (current: number, maxBytes?: number) => {
    const sourceId = currentSourceId();
    const state = untrack(window);

    if (
      current !== generation ||
      sourceId === undefined ||
      !state.hasMoreBefore
    ) {
      return false;
    }

    const page = await readGameLogBefore(
      currentInstanceId(),
      sourceId,
      windowStart(state),
      maxBytes,
    );

    if (current !== generation) {
      return false;
    }

    setWindow((w) => prependPage(w, page, windowFilter));

    return true;
  };

  const loadOlder = () => {
    const current = generation;

    // A searched log is read through by itself, from the end.
    if (windowFilter !== undefined) {
      return Promise.resolve();
    }

    return enqueue(async () => {
      setLoadingOlder(true);

      try {
        await readOlderPage(current);
      } finally {
        setLoadingOlder(false);
      }
    });
  };

  // A log held whole is read to its start; one that is searched, until its matches fill the window.
  const wantsRestOfFile = (state: LogWindow) =>
    holdsWholeFile(state) ||
    (windowFilter !== undefined && windowLineCount(state) < state.maxLines);

  /** A log small enough to hold whole is read to its start right after its end, in the background. */
  const loadRestOfFile = async (current: number) => {
    setLoadingOlder(true);

    try {
      while (
        current === generation &&
        wantsRestOfFile(untrack(window)) &&
        (await readOlderPage(current, WHOLE_FILE_PAGE_BYTES))
      ) {
        // Each page is added above the reader's place; the next one is asked for at once.
      }
    } finally {
      setLoadingOlder(false);
    }
  };

  /** One page below the window, when it was moved up from the end of a large file. */
  const loadNewer = () => {
    const current = generation;

    return enqueue(async () => {
      const sourceId = currentSourceId();
      const state = untrack(window);

      if (
        current !== generation ||
        sourceId === undefined ||
        state.tailLoaded
      ) {
        return;
      }

      const page = await readGameLogAfter(
        currentInstanceId(),
        sourceId,
        windowEnd(state),
      );

      if (current !== generation) {
        return;
      }

      setWindow((w) => ({
        ...appendText(w, page.start, page.end, page.text, windowFilter),
        tailLoaded: !page.hasMoreAfter,
      }));
    });
  };

  /** Back to the end of the file after the window was moved away from it. */
  const reloadTail = () => {
    generation += 1;
    const current = generation;

    return enqueue(() => loadTail(current));
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
    loaded,
    loading,
    loadingOlder,
    /** A search that stopped at the limit: more lines match than are shown. */
    searchCut: () => searching() && window().hasMoreBefore && !loadingOlder(),
    holdsWholeFile: () => holdsWholeFile(window()),
    error,
    hasMoreBefore: () => window().hasMoreBefore,
    tailLoaded: () => window().tailLoaded,
    pushLive,
    loadOlder,
    loadNewer,
    reloadTail,
  };
};
