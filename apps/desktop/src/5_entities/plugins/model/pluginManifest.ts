export interface PluginManifest {
  metadata: PluginMetadata;
  runtime: RuntimeConfig;
  load: LoadConfig;
  api: ApiConfig;
}

export interface PluginMetadata {
  id: string;
  name: string;
  version: string; // semver string, e.g. "1.2.3"
  description?: string;
  authors: string[];
  license?: string;
}

export interface RuntimeConfig {
  allowedHosts?: string[];
  allowedPaths?: PathMapping[];
}

export type PathMapping = [string, string]; // (path on disk, plugin path)

export type LoadConfig =
  | {
      type: 'extism';
      file: string;
      memoryLimit?: number;
    }
  | {
      type: 'native';
      libPath: string;
    };

export interface ApiConfig {
  version: string; // semver version requirement, e.g. "^1.0.0"
  features?: string[];
}
