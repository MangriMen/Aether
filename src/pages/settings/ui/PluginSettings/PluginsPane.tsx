import type { PluginMetadata } from '@/entities/plugins';
import { refetchPlugins, scanPlugins, usePlugins } from '@/entities/plugins';
import { createSignal, For, Show, type Component } from 'solid-js';
import { Button, CombinedTooltip, Separator, IconButton } from '@/shared/ui';
import { cn } from '@/shared/lib';
import type { SettingsPaneProps } from '../SettingsPane';
import { SettingsPane } from '../SettingsPane';
import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { revealInExplorer } from '@/entities/instances';
import { PluginInfoCard } from './PluginInfoCard';

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
          <div class='basis-48'>
            <For each={pluginsMetadata()}>
              {(metadata) => (
                <Button
                  variant='secondary'
                  class={cn('w-full', {
                    'bg-primary':
                      metadata.plugin.name === selectedMetadata()?.plugin.name,
                  })}
                  onClick={() => setSelectedMetadata(metadata)}
                >
                  {metadata.plugin.name}
                </Button>
              )}
            </For>
          </div>

          <Separator class='mx-2 bg-primary' orientation='vertical' />

          <Show
            when={selectedMetadata()}
            fallback={<span>Select a plugin</span>}
          >
            {(metadata) => (
              <PluginInfoCard class='size-full' plugin={metadata()} />
            )}
          </Show>
        </div>
      </Show>
    </SettingsPane>
  );
};
