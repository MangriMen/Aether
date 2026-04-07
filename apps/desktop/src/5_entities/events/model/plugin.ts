export enum PluginEventTypeEnum {
  Add = 'add',
  Edit = 'edit',
  Remove = 'remove',
  Sync = 'sync',
}

export interface PluginAdd {
  type: PluginEventTypeEnum.Add;
  plugin_id: string;
}

export interface PluginEdit {
  type: PluginEventTypeEnum.Edit;
  plugin_id: string;
}

export interface PluginRemove {
  type: PluginEventTypeEnum.Remove;
  plugin_id: string;
}

export interface PluginSync {
  type: PluginEventTypeEnum.Sync;
}

export type PluginEventType =
  | PluginAdd
  | PluginEdit
  | PluginRemove
  | PluginSync;

export interface PluginPayload {
  event: PluginEventType;
}
