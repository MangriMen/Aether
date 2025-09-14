import type { LauncherEventPayload } from './event';

export enum ProcessPayloadType {
  Finished = 'finished',
  Launched = 'launched',
}

export interface ProcessPayload {
  event: ProcessPayloadType;
  instanceId: string;
  message: string;
  processId: string;
}

export const isProcessPayload = (
  payload: LauncherEventPayload,
): payload is ProcessPayload =>
  'event' in payload &&
  (payload.event === ProcessPayloadType.Launched ||
    payload.event === ProcessPayloadType.Finished);
