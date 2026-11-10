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
import { Button } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { LogLevel, LogLine } from '../model/logLines';

/** Rows have one fixed height (a line never wraps), so scroll positions are plain arithmetic. */
const ROW_HEIGHT = 20;
const LOAD_OLDER_THRESHOLD_PX = 400;
const STICK_THRESHOLD_PX = 40;
const MAX_ROW_CHARS = 4000;

const LEVEL_CLASS: Record<LogLevel, string> = {
  debug: 'text-muted-foreground',
  info: '',
  warn: 'text-warning',
  error: 'text-destructive',
};

export type LogViewerProps = {
  lines: readonly LogLine[];
  hasMoreBefore: boolean;
  /** The window ends where the file ends, so lines that arrive can be shown after it. */
  tailLoaded: boolean;
  onLoadOlder: () => Promise<void>;
  onReloadTail: () => Promise<void>;
  class?: string;
};

export const LogViewer: Component<LogViewerProps> = (props) => {
  const [{ t }] = useTranslation();

  // oxlint-disable-next-line no-unassigned-vars
  let scrollRef: HTMLDivElement | undefined;

  const [stick, setStick] = createSignal(true);
  let loadingOlder = false;

  const virtualizer = createVirtualizer({
    get count() {
      return props.lines.length;
    },
    overscan: 30,
    getScrollElement: () => scrollRef ?? null,
    estimateSize: () => ROW_HEIGHT,
  });

  const scrollToEnd = () => {
    if (props.lines.length > 0) {
      virtualizer.scrollToIndex(props.lines.length - 1, { align: 'end' });
    }
  };

  // The first row in view. Lines that come in or go out above it move it, and the view is
  // moved back by as much, so that what the reader looks at stays put.
  let anchor: LogLine | undefined;
  let anchorIndex = 0;

  const rememberAnchor = () => {
    if (!scrollRef) {
      return;
    }

    anchorIndex = Math.floor(scrollRef.scrollTop / ROW_HEIGHT);
    anchor = props.lines[anchorIndex];
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

  const loadOlderIfNearStart = () => {
    if (
      props.hasMoreBefore &&
      scrollRef &&
      scrollRef.scrollTop < LOAD_OLDER_THRESHOLD_PX
    ) {
      void loadOlder();
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
          if (scrollRef && index >= 0 && index !== anchorIndex) {
            scrollRef.scrollTop += (index - anchorIndex) * ROW_HEIGHT;
            rememberAnchor();
          }

          // A filter can leave too few lines to scroll, and then no scroll event would ever ask.
          loadOlderIfNearStart();
        });
      },
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
    loadOlderIfNearStart();
  };

  const jumpToEnd = async () => {
    setStick(true);

    if (!props.tailLoaded) {
      await props.onReloadTail();
    }

    scrollToEnd();
  };

  const widestLine = createMemo(() =>
    Math.min(
      props.lines.reduce((max, line) => Math.max(max, line.text.length), 0),
      MAX_ROW_CHARS,
    ),
  );

  return (
    <div class={cn('relative flex min-h-0 flex-col', props.class)}>
      <div
        ref={scrollRef}
        class='
          rounded-md bg-card/card text-xs min-h-0 flex-1 overflow-auto border
          font-mono
        '
        onScroll={handleScroll}
      >
        <div
          class='relative'
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: `calc(${widestLine()}ch + 1.5rem)`,
            'min-width': '100%',
          }}
        >
          <For each={virtualizer.getVirtualItems()}>
            {(row) => (
              <div
                class={cn(
                  'px-3 absolute left-0 w-full whitespace-pre',
                  LEVEL_CLASS[props.lines[row.index]?.level ?? 'info'],
                )}
                style={{
                  height: `${ROW_HEIGHT}px`,
                  'line-height': `${ROW_HEIGHT}px`,
                  transform: `translateY(${row.start}px)`,
                }}
              >
                {props.lines[row.index]?.text}
              </div>
            )}
          </For>
        </div>
      </div>
      <Show when={!stick() || !props.tailLoaded}>
        <Button
          class='right-4 bottom-4 absolute shadow-md'
          size='sm'
          variant='secondary'
          onClick={() => void jumpToEnd()}
        >
          <IconMdiArrowDown />
          {t('logs.jumpToEnd')}
        </Button>
      </Show>
    </div>
  );
};
