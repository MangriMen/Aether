export enum LoadingBarTypeEnum {
  MinecraftDownload = 'minecraft_download',
  JavaDownload = 'java_download',
}

export interface MinecraftDownload {
  type: LoadingBarTypeEnum.MinecraftDownload;
  instance_name: string;
  instance_name_id: string;
}

export interface JavaDownload {
  type: LoadingBarTypeEnum.JavaDownload;
  version: number;
}

export type LoadingBarType = MinecraftDownload | JavaDownload;

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
