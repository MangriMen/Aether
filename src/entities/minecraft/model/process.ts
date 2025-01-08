export enum ProcessPayloadType {
  Launched = 'launched',
  Finished = 'finished',
}

export interface ProcessPayload {
  instanceId: string;
  uuid: string;
  event: ProcessPayloadType;
  message: string;
}

export interface MinecraftProcessMetadata {
  uuid: string;
  id: string;
  startTime: Date;
}
