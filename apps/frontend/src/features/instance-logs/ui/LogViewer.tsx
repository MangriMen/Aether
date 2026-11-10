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

import type { LogLine } from '../model/logLines';

import { createLogScroll } from '../lib/createLogScroll';
import { MAX_ROW_CHARS, ROW_HEIGHT, ROW_OVERSCAN } from '../model/constants';
import { LogRow } from './LogRow';

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
    overscan: ROW_OVERSCAN,
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

  // Without wrapping the list is as wide as its widest line, so that it scrolls sideways.
  const contentStyle = createMemo(() => ({
    height: `${virtualizer.getTotalSize()}px`,
    width: props.wrap ? '100%' : `calc(${widestLine()}ch + 1.5rem)`,
    'min-width': '100%',
  }));

  // The row's index attribute is set after the ref is handed over.
  const measureRow = (element: HTMLDivElement) => {
    queueMicrotask(() => virtualizer.measureElement(element));
  };

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
        <div class='relative' style={contentStyle()}>
          <For each={rowIndexes()}>
            {(index) => (
              <LogRow
                index={index}
                line={props.lines[index]}
                start={rowStart(index)}
                wrap={props.wrap}
                onElement={measureRow}
              />
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
