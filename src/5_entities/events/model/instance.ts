export interface InstanceEvent {
  event: InstancePayloadType;
  instanceId: string;
}

export type InstancePayloadType = 'created' | 'edited' | 'removed' | 'synced';
