export interface ApiConfig {
  features?: string[];
  version: string; // semver version requirement, e.g. "^1.0.0"
}

export type LoadConfig =
  | {
      file: string;
      memory_limit?: number;
      type: 'extism';
    }
  | {
      lib_path: string;
      type: 'native';
    };

export type PathMapping = [string, string]; // (path on disk, plugin path)

export interface PluginManifest {
  api: ApiConfig;
  load: LoadConfig;
  metadata: PluginMetadata;
  runtime: RuntimeConfig;
}

export interface PluginMetadata {
  authors: string[];
  description?: string;
  id: string;
  license?: string;
  name: string;
  version: string; // semver string, e.g. "1.2.3"
}

export interface RuntimeConfig {
  allowed_hosts?: string[];
  allowed_paths?: PathMapping[];
}
