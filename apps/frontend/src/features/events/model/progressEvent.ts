import type {
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/platform/bindings/events';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ProgressEventType = ProgressEventTypeDto;

export type ProgressEvent = ProgressEventDto;

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;

export const PROGRESS_EVENT_FAILED_REMOVE_DELAY = 15000;

export const isProgressFailed = (payload: ProgressEvent) =>
  payload.status.type === 'failed';

export const getProgressRemoveDelay = (payload: ProgressEvent) =>
  isProgressFailed(payload)
    ? PROGRESS_EVENT_FAILED_REMOVE_DELAY
    : PROGRESS_EVENT_REMOVE_DELAY;
