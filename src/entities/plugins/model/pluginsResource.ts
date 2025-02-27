import { getIsPluginEnabled, getPlugin, listPlugins } from '../api';
import type { Plugin, PluginInfo } from './plugin';
import { ReactiveMap } from '@solid-primitives/map';

const plugins = new ReactiveMap<PluginInfo['id'], Plugin>();

const fetchPlugins = async () => {
  try {
    const pluginsMetadata = await listPlugins();
    const pluginsState = await Promise.all(
      pluginsMetadata.map((metadata) => getIsPluginEnabled(metadata.plugin.id)),
    );

    const removed_plugins = Array.from(plugins.keys()).filter((id) =>
      pluginsMetadata.every((metadata) => metadata.plugin.id !== id),
    );

    for (const id of removed_plugins) {
      plugins.delete(id);
    }

    for (let i = 0; i < pluginsMetadata.length; i++) {
      const metadata = pluginsMetadata[i];
      plugins.set(metadata.plugin.id, {
        enabled: pluginsState[i],
        metadata,
      });
    }
  } catch {
    console.error("Can't get plugins");
  }
};

const fetchPlugin = async (id: PluginInfo['id']) => {
  try {
    const pluginMetadata = await getPlugin(id);
    const pluginState = await getIsPluginEnabled(id);

    plugins.set(id, { enabled: pluginState, metadata: pluginMetadata });
  } catch {
    console.error("Can't get plugins");
  }
};

export const initializePlugins = fetchPlugins;

export const usePlugins = () => plugins;

export const refetchPlugins = fetchPlugins;

export const refetchPlugin = (id: PluginInfo['id']) => fetchPlugin(id);
