import type { Accessor } from 'solid-js';

import { createMemo, createSignal } from 'solid-js';

import type { Plugin, PluginMetadata } from '@/entities/plugins';

export const usePluginSelection = (plugins: Accessor<Plugin[] | undefined>) => {
  const [selectedPluginId, setSelectedPluginId] = createSignal<
    PluginMetadata['id'] | undefined
  >();

  const selectedPlugin = createMemo(() =>
    plugins()?.find(
      (plugin) => plugin.manifest.metadata.id === selectedPluginId(),
    ),
  );

  const unselectPlugin = () => setSelectedPluginId();

  const selectPlugin = (pluginId: PluginMetadata['id']) => {
    if (pluginId === selectedPluginId()) {
      unselectPlugin();
      return;
    }

    setSelectedPluginId(pluginId);
  };

  return {
    selectedPluginId,
    selectedPlugin,
    selectPlugin,
    unselectPlugin,
  };
};
