import { createVirtualizer } from '@tanstack/solid-virtual';
import IconMdiArrowDown from '~icons/mdi/arrow-down';
import {
  createEffect,
  createMemo,
  For,
  on,
  Show,
  type Component,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { LogLevel, LogLine } from '../model/logLines';

import { createLogScroll } from '../lib/createLogScroll';

/** Without wrapping a line is one row of this height; wrapped rows are measured. */
const ROW_HEIGHT = 22;
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
  onLoadOlder: () => Promise<void>;
  onLoadNewer: () => Promise<void>;
  onReloadTail: () => Promise<void>;
  class?: string;
};

export const LogViewer: Component<LogViewerProps> = (props) => {
  const [{ t }] = useTranslation();

  // oxlint-disable-next-line no-unassigned-vars
  let scrollRef: HTMLDivElement | undefined;

  const virtualizer = createVirtualizer({
    get count() {
      return props.lines.length;
    },
    overscan: 30,
    getScrollElement: () => scrollRef ?? null,
    estimateSize: () => ROW_HEIGHT,
  });

  const { stick, handleScroll, jumpToEnd } = createLogScroll({
    scrollElement: () => scrollRef,
    virtualizer,
    lines: () => props.lines,
    hasMoreBefore: () => props.hasMoreBefore,
    tailLoaded: () => props.tailLoaded,
    onLoadOlder: () => props.onLoadOlder(),
    onLoadNewer: () => props.onLoadNewer(),
    onReloadTail: () => props.onReloadTail(),
  });

  // Rows are keyed by their index, so a row that stays in view keeps its element when the
  // list around it changes; keyed by the virtual items themselves they would all be rebuilt
  // each time and flicker.
  const rowIndexes = createMemo(() =>
    virtualizer.getVirtualItems().map((item) => item.index),
  );
  // Where each row in view starts; the virtualizer tells when a measured height moved them.
  const rowStarts = createMemo(
    () =>
      new Map(
        virtualizer.getVirtualItems().map((item) => [item.index, item.start]),
      ),
  );
  const rowStart = (index: number) =>
    rowStarts().get(index) ?? index * ROW_HEIGHT;

  // Rows change height when lines start or stop wrapping.
  createEffect(
    on(
      () => props.wrap,
      () => virtualizer.measure(),
      { defer: true },
    ),
  );

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
