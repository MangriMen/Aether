export enum LoadingBarTypeEnum {
  MinecraftDownload = 'minecraft_download',
}

export interface MinecraftDownload {
  type: LoadingBarTypeEnum.MinecraftDownload;
  instance_name: string;
  instance_name_id: string;
}

export type LoadingBarType = MinecraftDownload;

export interface LoadingPayload {
  event: LoadingBarType;
  loaderUuid: string;
  fraction?: number; // by convention, if optional, it means the loading is done
  message: string;
}
