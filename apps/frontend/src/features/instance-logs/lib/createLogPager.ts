import { untrack, type Accessor, type Setter } from 'solid-js';

import type { LineFilter } from '../model/logLines';
import type { LogTasks } from './createLogTasks';

import { readGameLogAfter, readGameLogBefore } from '../api/gameLogApi';
import { WHOLE_FILE_PAGE_BYTES } from '../model/constants';
import {
  appendText,
  holdsWholeFile,
  prependPage,
  windowEnd,
  windowLineCount,
  windowStart,
  type LogWindow,
} from '../model/logWindow';

export type LogPagerProps = {
  /** The reads run outside any reactive scope and want the value of the moment. */
  instanceId: () => string;
  sourceId: () => string | undefined;
  window: Accessor<LogWindow>;
  setWindow: Setter<LogWindow>;
  /** What the lines read must match, when the log is searched through rather than held whole. */
  windowFilter: () => LineFilter | undefined;
  tasks: LogTasks;
  setLoadingOlder: (loading: boolean) => void;
};

/** Reads more of the log into the window: older lines above it, newer ones below it. */
export const createLogPager = ({
  instanceId,
  sourceId,
  window,
  setWindow,
  windowFilter,
  tasks,
  setLoadingOlder,
}: LogPagerProps) => {
  /** Reads on from the end of the window to the end of the file. */
  const readUntilEnd = async (current: number, source: string) => {
    for (;;) {
      const page = await readGameLogAfter(
        instanceId(),
        source,
        windowEnd(window()),
      );

      if (current !== tasks.current()) {
        return;
      }

      if (page.end > windowEnd(window())) {
        setWindow((w) =>
          appendText(w, page.start, page.end, page.text, windowFilter()),
        );
      }

      if (!page.hasMoreAfter || page.end === page.start) {
        return;
      }
    }
  };

  /** One page above the window; false when there is nothing above it or the view moved on. */
  const readOlderPage = async (current: number, maxBytes?: number) => {
    const source = sourceId();
    const state = untrack(window);

    if (
      current !== tasks.current() ||
      source === undefined ||
      !state.hasMoreBefore
    ) {
      return false;
    }

    const page = await readGameLogBefore(
      instanceId(),
      source,
      windowStart(state),
      maxBytes,
    );

    if (current !== tasks.current()) {
      return false;
    }

    setWindow((w) => prependPage(w, page, windowFilter()));

    return true;
  };

  const loadOlder = () => {
    const current = tasks.current();

    // A searched log is read through by itself, from the end.
    if (windowFilter() !== undefined) {
      return Promise.resolve();
    }

    return tasks.enqueue(async () => {
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
    (windowFilter() !== undefined && windowLineCount(state) < state.maxLines);

  /** A log small enough to hold whole is read to its start right after its end, in the background. */
  const loadRestOfFile = async (current: number) => {
    setLoadingOlder(true);

    try {
      while (
        current === tasks.current() &&
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
    const current = tasks.current();

    return tasks.enqueue(async () => {
      const source = sourceId();
      const state = untrack(window);

      if (
        current !== tasks.current() ||
        source === undefined ||
        state.tailLoaded
      ) {
        return;
      }

      const page = await readGameLogAfter(
        instanceId(),
        source,
        windowEnd(state),
      );

      if (current !== tasks.current()) {
        return;
      }

      setWindow((w) => ({
        ...appendText(w, page.start, page.end, page.text, windowFilter()),
        tailLoaded: !page.hasMoreAfter,
      }));
    });
  };

  return { readUntilEnd, loadOlder, loadRestOfFile, loadNewer };
};
