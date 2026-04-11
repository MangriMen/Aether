import type {
  ProgressBarDto,
  ProgressEventDto,
  ProgressEventTypeDto,
} from '@/shared/api';

import type { LauncherEventPayload } from './event';

export type ProgressEventType = ProgressEventTypeDto;

export type ProgressBar = ProgressBarDto;

export type ProgressEvent = ProgressEventDto;

export const isProgressEvent = (
  payload: LauncherEventPayload,
): payload is ProgressEvent => 'event' in payload && 'fraction' in payload;
