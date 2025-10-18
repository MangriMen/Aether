import type { Accessor } from 'solid-js';

import { createMemo, createSignal } from 'solid-js';

import type { Plugin, PluginMetadata } from '@/entities/plugins';

export const usePluginSelection = (
  plugins: Accessor<Plugin[] | undefined>,
  withAnimation: Accessor<boolean>,
) => {
  const [selectedPluginId, setSelectedPluginId] = createSignal<
    PluginMetadata['id'] | undefined
  >();

  const selectedPlugin = createMemo(() =>
    plugins()?.find(
      (plugin) => plugin.manifest.metadata.id === selectedPluginId(),
    ),
  );

  const selectPlugin = (pluginId: PluginMetadata['id']) =>
    setSelectedPluginId(pluginId);

  const unselectPlugin = () => setSelectedPluginId();

  const [hasSelectedPlugin, setHasSelectedPlugin] = createSignal(false);

  const selectPluginAnimated = (pluginId: PluginMetadata['id']) => {
    if (pluginId === selectedPluginId()) {
      setHasSelectedPlugin(false);
      if (!withAnimation()) {
        setSelectedPluginId(undefined);
      }
      return;
    }

    setHasSelectedPlugin(true);
    setSelectedPluginId(pluginId);
  };

  const unselectPluginAnimated = () => {
    if (withAnimation() && hasSelectedPlugin() === false) {
      setSelectedPluginId(undefined);
    }
  };

  return {
    selectedPluginId,
    selectedPlugin,
    selectPlugin,
    unselectPlugin,
    hasSelectedPlugin,
    selectPluginAnimated,
    unselectPluginAnimated,
  };
};
