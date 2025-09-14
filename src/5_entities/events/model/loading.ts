import type { LauncherEventPayload } from './event';

export enum LoadingBarTypeEnum {
  JavaDownload = 'java_download',
  LauncherUpdate = 'launcher_update',
  MinecraftDownload = 'minecraft_download',
  PluginDownload = 'plugin_download',
}

export interface JavaDownload {
  type: LoadingBarTypeEnum.JavaDownload;
  version: number;
}

export interface LauncherUpdate {
  current_version: string;
  type: LoadingBarTypeEnum.LauncherUpdate;
  version: string;
}

export interface LoadingBar {
  barType: LoadingBarType;
  current: number;
  loadingBarUuid: string;
  message: string;
  total: number;
}

export type LoadingBarType =
  | JavaDownload
  | LauncherUpdate
  | MinecraftDownload
  | PluginDownload;

export interface LoadingPayload {
  event: LoadingBarType;
  fraction: null | number; // by convention, if optional, it means the loading is done
  loaderUuid: string;
  message: string;
}

export interface MinecraftDownload {
  instance_id: string;
  instance_name: string;
  type: LoadingBarTypeEnum.MinecraftDownload;
}

export interface PluginDownload {
  plugin_name: string;
  type: LoadingBarTypeEnum.PluginDownload;
}

export const isLoadingPayload = (
  payload: LauncherEventPayload,
): payload is LoadingPayload => 'event' in payload && 'fraction' in payload;
