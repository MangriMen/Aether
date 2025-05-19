export type InstancePayloadType = 'created' | 'synced' | 'edited' | 'removed';

export interface InstanceEvent {
  event: InstancePayloadType;
  instanceId: string;
}
