import type {
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/platform/bindings/events';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ProgressEventType = ProgressEventTypeDto;

/** What a bar shows; the order number of the event is for the listener only. */
export type ProgressEvent = Omit<ProgressEventDto, 'seq'>;

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;
