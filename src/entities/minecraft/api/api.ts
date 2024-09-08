import { invoke } from '@tauri-apps/api';

import { VersionManifest } from '../model';

export const initializeState = () => invoke('initialize_state');

export const getMinecraftVersionManifest = () =>
  invoke<VersionManifest>('get_minecraft_version_manifest');
