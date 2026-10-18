import type {
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/platform/bindings/events';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ProgressEventType = ProgressEventTypeDto;

export type ProgressEvent = ProgressEventDto;

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;
