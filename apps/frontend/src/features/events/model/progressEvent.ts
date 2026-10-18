import type {
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/platform/bindings/events';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ProgressEventType = ProgressEventTypeDto;

export type ProgressEvent = ProgressEventDto;

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;

/**
 * An event that arrives after the bar's last one is out of order and must not bring the bar
 * back to life: a finished bar stays as it ended.
 */
export const isStaleEvent = (
  current: ProgressEvent | undefined,
  incoming: ProgressEvent,
) =>
  current !== undefined &&
  current.status.type !== 'running' &&
  incoming.status.type === 'running';
