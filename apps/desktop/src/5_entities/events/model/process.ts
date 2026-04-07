import type { LauncherEventPayload } from './event';

export enum ProcessPayloadType {
  Launched = 'launched',
  Finished = 'finished',
}

export interface ProcessPayload {
  instanceId: string;
  processId: string;
  event: ProcessPayloadType;
  message: string;
}

export const isProcessPayload = (
  payload: LauncherEventPayload,
): payload is ProcessPayload =>
  'event' in payload &&
  (payload.event === ProcessPayloadType.Launched ||
    payload.event === ProcessPayloadType.Finished);
