export type InstancePayloadType = 'created' | 'synced' | 'edited' | 'removed';

export interface InstancePayload {
  instancePathId: string;
  event: InstancePayloadType;
}
