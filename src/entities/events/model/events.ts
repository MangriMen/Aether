export enum LoadingBarTypeEnum {
  MinecraftDownload = 'minecraft_download',
  JavaDownload = 'java_download',
  PluginDownload = 'plugin_download',
  LauncherUpdate = 'launcher_update',
}

export interface MinecraftDownload {
  type: LoadingBarTypeEnum.MinecraftDownload;
  instance_name: string;
  instance_id: string;
}

export interface JavaDownload {
  type: LoadingBarTypeEnum.JavaDownload;
  version: number;
}

export interface PluginDownload {
  type: LoadingBarTypeEnum.PluginDownload;
  plugin_name: string;
}

export interface LauncherUpdate {
  type: LoadingBarTypeEnum.LauncherUpdate;
  version: string;
  current_version: string;
}

export type LoadingBarType =
  | MinecraftDownload
  | JavaDownload
  | PluginDownload
  | LauncherUpdate;

export interface LoadingBar {
  loadingBarUuid: string;
  message: string;
  total: number;
  current: number;
  barType: LoadingBarType;
}

export interface LoadingPayload {
  event: LoadingBarType;
  loaderUuid: string;
  fraction: number | null; // by convention, if optional, it means the loading is done
  message: string;
}

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

export enum MinecraftEvent {
  Loading = 'loading',
  Process = 'process',
}

export type MinecraftEventName = `${MinecraftEvent}`;
export type MinecraftEventPayload = LoadingPayload | ProcessPayload;

export const isProcessPayload = (
  payload: MinecraftEventPayload,
): payload is ProcessPayload =>
  'event' in payload &&
  (payload.event === ProcessPayloadType.Launched ||
    payload.event === ProcessPayloadType.Finished);

export const isLoadingPayload = (
  payload: MinecraftEventPayload,
): payload is LoadingPayload => 'event' in payload && 'fraction' in payload;
