export interface Settings {
  launcherDir?: string;
  metadataDir?: string;

  maxConcurrentDownloads: number;

  enabledPlugins: Set<string>;
}
