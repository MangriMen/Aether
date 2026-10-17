import type {
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/platform/bindings/events';

export type ProgressEventType = ProgressEventTypeDto;

export type ProgressEvent = ProgressEventDto;

export const PROGRESS_EVENT_REMOVE_DELAY = 5000;
