import type { PluginMetadata } from '@/entities/plugins';
import { refetchPlugins, scanPlugins, usePlugins } from '@/entities/plugins';
import { createSignal, For, Show, type Component } from 'solid-js';
import { CombinedTooltip, Separator, IconButton } from '@/shared/ui';
import { cn } from '@/shared/lib';
import type { SettingsPaneProps } from '../SettingsPane';
import { SettingsPane } from '../SettingsPane';
import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { revealInExplorer } from '@/entities/instances';
import { PluginInfoCard } from './PluginInfoCard';
import { PluginCard } from './PluginCard';

export type PluginsEntryProps = SettingsPaneProps;

export const PluginsPane: Component<PluginsEntryProps> = (props) => {
  const pluginsMetadata = usePlugins();

  const [selectedMetadata, setSelectedMetadata] =
    createSignal<PluginMetadata | null>(null);

  const handleOpenPluginsFolder = () => {
    revealInExplorer(
      'C:/Users/mangr/AppData/Roaming/com.mangrimen.aether/plugins',
      true,
    );
  };

  const handleRefreshPlugins = async () => {
    await scanPlugins();
    refetchPlugins();
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
      <Show
        when={pluginsMetadata()?.length}
        fallback={<span>There is no plugins</span>}
      >
        <div class='flex size-full'>
          <div
            class={cn('flex-1 basis-72 min-w-72', {
              'max-w-72': !!selectedMetadata(),
            })}
          >
            <For each={pluginsMetadata()}>
              {(metadata) => (
                <PluginCard
                  role='button'
                  isActive={
                    metadata.plugin.name === selectedMetadata()?.plugin.name
                  }
                  plugin={metadata}
                  onClick={() => setSelectedMetadata(metadata)}
                />
              )}
            </For>
          </div>

          <Show when={selectedMetadata()}>
            {(metadata) => (
              <>
                <Separator class='mx-2' orientation='vertical' />
                <PluginInfoCard
                  class='my-2 flex-1 basis-full'
                  plugin={metadata()}
                />
              </>
            )}
          </Show>
        </div>
      </Show>
    </SettingsPane>
  );
};
