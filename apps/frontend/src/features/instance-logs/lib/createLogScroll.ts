import type { Virtualizer } from '@tanstack/solid-virtual';

import { createEffect, createSignal, on } from 'solid-js';

import type { LogLine } from '../model/logLines';

import { LOAD_MORE_THRESHOLD_PX, STICK_THRESHOLD_PX } from '../model/constants';

export type LogScrollProps = {
  scrollElement: () => HTMLDivElement | undefined;
  virtualizer: Virtualizer<HTMLDivElement, Element>;
  lines: () => readonly LogLine[];
  hasMoreBefore: () => boolean;
  /** The lines end where the file ends, so lines that arrive can be shown after them. */
  tailLoaded: () => boolean;
  onLoadOlder: () => Promise<void>;
  onLoadNewer: () => Promise<void>;
  onReloadTail: () => Promise<void>;
};

/** A call that is let through only when the one before it is done. */
const singleFlight = (task: () => Promise<void>) => {
  let busy = false;

  return async () => {
    if (busy) {
      return;
    }

    busy = true;

    try {
      await task();
    } finally {
      busy = false;
    }
  };
};

/**
 * Where a log list is scrolled to: it follows the end while the reader is at the end, keeps
 * the lines being read in place when lines change above them, and asks for more lines when
 * the reader nears either edge of what is loaded.
 */
export const createLogScroll = ({
  onLoadOlder,
  onLoadNewer,
  ...props
}: LogScrollProps) => {
  const [stick, setStick] = createSignal(true);

  const distanceToEnd = (element: HTMLDivElement) =>
    element.scrollHeight - element.scrollTop - element.clientHeight;

  const scrollToEnd = () => {
    const count = props.lines().length;

    if (count > 0) {
      props.virtualizer.scrollToIndex(count - 1, { align: 'end' });
    }
  };

  // The first line in view and how far into it the view starts. Lines that come in or go out
  // above it move it, and the view is moved back by as much, so what is read stays put.
  let anchor: LogLine | undefined;
  let anchorShift = 0;

  const rememberAnchor = () => {
    const element = props.scrollElement();

    if (!element) {
      return;
    }

    const item = props.virtualizer.getVirtualItemForOffset(element.scrollTop);

    anchor = item ? props.lines()[item.index] : undefined;
    anchorShift = item ? element.scrollTop - item.start : 0;
  };

  const restoreAnchor = (index: number) => {
    const element = props.scrollElement();
    const start = props.virtualizer.measurementsCache[index]?.start;

    if (element && start !== undefined) {
      element.scrollTop = start + anchorShift;
    }

    rememberAnchor();
  };

  const loadOlder = singleFlight(onLoadOlder);
  const loadNewer = singleFlight(onLoadNewer);

  const loadMoreIfNearEdge = () => {
    const element = props.scrollElement();

    if (!element) {
      return;
    }

    if (props.hasMoreBefore() && element.scrollTop < LOAD_MORE_THRESHOLD_PX) {
      void loadOlder();
    }

    if (
      !props.tailLoaded() &&
      distanceToEnd(element) < LOAD_MORE_THRESHOLD_PX
    ) {
      void loadNewer();
    }
  };

  createEffect(
    on(
      () => props.lines(),
      (lines) => {
        const following = stick() && props.tailLoaded();

        if (following) {
          queueMicrotask(scrollToEnd);
        }

        const index = !following && anchor ? lines.indexOf(anchor) : -1;

        requestAnimationFrame(() => {
          if (index >= 0) {
            restoreAnchor(index);
          }

          // A filter can leave too few lines to scroll, and then no scroll event would ever ask.
          loadMoreIfNearEdge();
        });
      },
    ),
  );

  const handleScroll = () => {
    const element = props.scrollElement();

    if (!element) {
      return;
    }

    setStick(distanceToEnd(element) < STICK_THRESHOLD_PX);
    rememberAnchor();
    loadMoreIfNearEdge();
  };

  const jumpToEnd = async () => {
    setStick(true);

    if (!props.tailLoaded()) {
      await props.onReloadTail();
    }

    scrollToEnd();
  };

  return { stick, handleScroll, jumpToEnd };
};
