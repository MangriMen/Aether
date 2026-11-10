import { createVirtualizer } from '@tanstack/solid-virtual';
import IconMdiArrowDown from '~icons/mdi/arrow-down';
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { LogLevel, LogLine } from '../model/logLines';

/** Without wrapping a line is one row of this height; wrapped rows are measured. */
const ROW_HEIGHT = 22;
const LOAD_MORE_THRESHOLD_PX = 400;
const STICK_THRESHOLD_PX = 40;
const MAX_ROW_CHARS = 4000;

const LEVEL_CLASS: Record<LogLevel, string> = {
  debug: 'text-muted-foreground/60',
  info: 'text-muted-foreground',
  warn: 'text-warning',
  error: 'text-destructive',
};

export type LogViewerProps = {
  lines: readonly LogLine[];
  hasMoreBefore: boolean;
  /** The window ends where the file ends, so lines that arrive can be shown after it. */
  tailLoaded: boolean;
  /** Long lines continue on the next row instead of running off to the right. */
  wrap: boolean;
  /** Shown in the middle of the box while there is nothing to list. */
  emptyText?: string;
  /** A short note on what the viewer is doing or leaving out, over the corner of the box. */
  statusText?: string;
  onLoadOlder: () => Promise<void>;
  onLoadNewer: () => Promise<void>;
  onReloadTail: () => Promise<void>;
  class?: string;
};

export const LogViewer: Component<LogViewerProps> = (props) => {
  const [{ t }] = useTranslation();

  // oxlint-disable-next-line no-unassigned-vars
  let scrollRef: HTMLDivElement | undefined;

  const [stick, setStick] = createSignal(true);
  let loadingOlder = false;
  let loadingNewer = false;

  const virtualizer = createVirtualizer({
    get count() {
      return props.lines.length;
    },
    overscan: 30,
    getScrollElement: () => scrollRef ?? null,
    estimateSize: () => ROW_HEIGHT,
  });

  // Rows are keyed by their index, so a row that stays in view keeps its element when the
  // list around it changes; keyed by the virtual items themselves they would all be rebuilt
  // each time and flicker.
  const rowIndexes = createMemo(() =>
    virtualizer.getVirtualItems().map((item) => item.index),
  );
  const rowStart = (index: number) =>
    virtualizer.getVirtualItems().find((item) => item.index === index)?.start ??
    index * ROW_HEIGHT;

  const scrollToEnd = () => {
    if (props.lines.length > 0) {
      virtualizer.scrollToIndex(props.lines.length - 1, { align: 'end' });
    }
  };

  // The first row in view and how far into it the view starts. Lines that come in or go out
  // above it move it, and the view is moved back by as much, so what is read stays put.
  let anchor: LogLine | undefined;
  let anchorShift = 0;

  const rememberAnchor = () => {
    if (!scrollRef) {
      return;
    }

    const item = virtualizer.getVirtualItemForOffset(scrollRef.scrollTop);

    anchor = item ? props.lines[item.index] : undefined;
    anchorShift = item ? scrollRef.scrollTop - item.start : 0;
  };

  const restoreAnchor = (index: number) => {
    const start = virtualizer.measurementsCache[index]?.start;

    if (scrollRef && start !== undefined) {
      scrollRef.scrollTop = start + anchorShift;
    }

    rememberAnchor();
  };

  const loadOlder = async () => {
    if (loadingOlder) {
      return;
    }

    loadingOlder = true;

    try {
      await props.onLoadOlder();
    } finally {
      loadingOlder = false;
    }
  };

  const loadNewer = async () => {
    if (loadingNewer) {
      return;
    }

    loadingNewer = true;

    try {
      await props.onLoadNewer();
    } finally {
      loadingNewer = false;
    }
  };

  const loadMoreIfNearEdge = () => {
    if (!scrollRef) {
      return;
    }

    if (props.hasMoreBefore && scrollRef.scrollTop < LOAD_MORE_THRESHOLD_PX) {
      void loadOlder();
    }

    const distanceToEnd =
      scrollRef.scrollHeight - scrollRef.scrollTop - scrollRef.clientHeight;

    if (!props.tailLoaded && distanceToEnd < LOAD_MORE_THRESHOLD_PX) {
      void loadNewer();
    }
  };

  createEffect(
    on(
      () => props.lines,
      (lines) => {
        const following = stick() && props.tailLoaded;

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

  // Rows change height when lines start or stop wrapping.
  createEffect(
    on(
      () => props.wrap,
      () => virtualizer.measure(),
      { defer: true },
    ),
  );

  const handleScroll = () => {
    if (!scrollRef) {
      return;
    }

    const distanceToEnd =
      scrollRef.scrollHeight - scrollRef.scrollTop - scrollRef.clientHeight;

    setStick(distanceToEnd < STICK_THRESHOLD_PX);
    rememberAnchor();
    loadMoreIfNearEdge();
  };

  const jumpToEnd = async () => {
    setStick(true);

    if (!props.tailLoaded) {
      await props.onReloadTail();
    }

    scrollToEnd();
  };

  const widestLine = createMemo(() =>
    props.wrap
      ? 0
      : Math.min(
          props.lines.reduce((max, line) => Math.max(max, line.text.length), 0),
          MAX_ROW_CHARS,
        ),
  );

  return (
    <div class={cn('relative flex min-h-0 flex-col', props.class)}>
      <div
        ref={scrollRef}
        class='
          rounded-md bg-card/card text-sm min-h-0 flex-1 overflow-auto border
          font-mono
        '
        onScroll={handleScroll}
      >
        <div
          class='relative'
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: props.wrap ? '100%' : `calc(${widestLine()}ch + 1.5rem)`,
            'min-width': '100%',
          }}
        >
          <For each={rowIndexes()}>
            {(index) => (
              <div
                ref={(node) => {
                  // The row's index attribute is set after the ref is handed over.
                  queueMicrotask(() => virtualizer.measureElement(node));
                }}
                data-index={index}
                class={cn(
                  'px-3 absolute left-0 w-full',
                  props.wrap
                    ? 'wrap-anywhere whitespace-pre-wrap'
                    : 'whitespace-pre',
                  LEVEL_CLASS[props.lines[index]?.level ?? 'info'],
                )}
                style={{
                  'min-height': `${ROW_HEIGHT}px`,
                  height: props.wrap ? undefined : `${ROW_HEIGHT}px`,
                  'line-height': `${ROW_HEIGHT}px`,
                  transform: `translateY(${rowStart(index)}px)`,
                }}
              >
                {props.lines[index]?.text}
              </div>
            )}
          </For>
        </div>
      </div>
      <Show when={props.lines.length === 0 && props.emptyText}>
        <div
          class='
            p-4 text-muted-foreground inset-0 absolute flex items-center
            justify-center text-center text-sm whitespace-pre-line
          '
        >
          {props.emptyText}
        </div>
      </Show>
      <Show when={props.statusText}>
        <div
          class='
            left-3 bottom-3 rounded-md bg-card/card px-2 py-1 text-muted-foreground
            absolute border text-sm shadow-md
          '
        >
          {props.statusText}
        </div>
      </Show>
      <Show when={!stick() || !props.tailLoaded}>
        <CombinedTooltip
          label={t('logs.jumpToEnd')}
          as={IconButton}
          class='right-4 bottom-4 absolute shadow-md'
          variant='secondary'
          icon={IconMdiArrowDown}
          onClick={() => void jumpToEnd()}
        />
      </Show>
    </div>
  );
};
