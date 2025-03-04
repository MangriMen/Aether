import type { PluginInfo } from '@/entities/plugins';
import {
  openPluginsFolder,
  refetchPlugins,
  scanPlugins,
  usePlugins,
} from '@/entities/plugins';
import { createMemo, createSignal, For, Show, type Component } from 'solid-js';
import { CombinedTooltip, Separator, IconButton, showToast } from '@/shared/ui';
import { cn } from '@/shared/lib';
import type { SettingsPaneProps } from '../SettingsPane';
import { SettingsPane } from '../SettingsPane';
import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { PluginInfoCard } from './PluginInfoCard';
import { PluginCard } from './PluginCard';
import { isAetherLauncherError } from '@/shared/model';

export type PluginsEntryProps = SettingsPaneProps;

export const PluginsPane: Component<PluginsEntryProps> = (props) => {
  const plugins = usePlugins();

  const pluginsValues = createMemo(() => Array.from(plugins.values()));

  const [selectedPluginId, setSelectedPluginId] = createSignal<
    PluginInfo['id'] | null
  >(null);

  const selectedPlugin = () => {
    const id = selectedPluginId();
    return id ? plugins.get(id) : null;
  };

  const handleOpenPluginsFolder = async () => {
    try {
      await openPluginsFolder();
    } catch (e) {
      if (isAetherLauncherError(e)) {
        showToast({
          title: 'Failed to open plugins folder',
          variant: 'destructive',
          description: e.message,
        });
      }
    }
  };

  const handleRefreshPlugins = async () => {
    await scanPlugins();
    await refetchPlugins();
  };

  return (
    <SettingsPane
      class='container max-w-screen-lg'
      label={
        <div class='flex justify-between'>
          Plugins
          <div class='flex gap-2'>
            <CombinedTooltip
              label='Open plugins folder'
              as={IconButton}
              icon={MdiFolderIcon}
              onClick={handleOpenPluginsFolder}
            />
            <CombinedTooltip
              label='Refresh'
              as={IconButton}
              icon={MdiReloadIcon}
              onClick={handleRefreshPlugins}
            />
          </div>
        </div>
      }
      {...props}
    >
      <Show when={plugins?.size} fallback={<span>There is no plugins</span>}>
        <div class='flex size-full'>
          <div
            class={cn('flex-1 basis-72 min-w-72', {
              'max-w-72': !!selectedPluginId(),
            })}
          >
            <For each={pluginsValues()}>
              {(plugin) => (
                <PluginCard
                  role='button'
                  isSelected={plugin.metadata.plugin.id === selectedPluginId()}
                  plugin={plugin}
                  onClick={() => setSelectedPluginId(plugin.metadata.plugin.id)}
                />
              )}
            </For>
          </div>

          <Show when={selectedPlugin()}>
            {(plugin) => (
              <>
                <Separator class='mx-2' orientation='vertical' />
                <PluginInfoCard
                  class='my-2 flex-1 basis-full'
                  plugin={plugin()}
                />
              </>
            )}
          </Show>
        </div>
      </Show>
    </SettingsPane>
  );
};
