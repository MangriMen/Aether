import type { Component } from 'solid-js';

import { cn } from '@/shared/utils';

import type { LogLine } from '../model/logLines';

import { LEVEL_CLASS, ROW_HEIGHT } from '../model/constants';

export type LogRowProps = {
  index: number;
  line: LogLine | undefined;
  /** Where the row starts in the list, in pixels. */
  start: number;
  /** Long lines continue on the next row instead of running off to the right. */
  wrap: boolean;
  /** Called with the row's element, for the list to measure it. */
  onElement: (element: HTMLDivElement) => void;
};

/** One line of the log, placed in the list at its place. */
export const LogRow: Component<LogRowProps> = (props) => (
  <div
    ref={(element) => props.onElement(element)}
    data-index={props.index}
    class={cn(
      'px-3 absolute left-0 w-full',
      props.wrap ? 'wrap-anywhere whitespace-pre-wrap' : 'whitespace-pre',
      LEVEL_CLASS[props.line?.level ?? 'info'],
    )}
    style={{
      'min-height': `${ROW_HEIGHT}px`,
      height: props.wrap ? undefined : `${ROW_HEIGHT}px`,
      'line-height': `${ROW_HEIGHT}px`,
      transform: `translateY(${props.start}px)`,
    }}
  >
    {props.line?.text}
  </div>
);
