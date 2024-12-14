export enum ProcessPayloadType {
  Launched = 'launched',
  Finished = 'finished',
}

export interface ProcessPayload {
  instanceNameId: string;
  uuid: string;
  event: ProcessPayloadType;
  message: string;
}

export interface MinecraftProcessMetadata {
  uuid: string;
  nameId: string;
  startTime: Date;
}
