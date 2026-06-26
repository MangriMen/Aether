import IconMdiChevronDown from '~icons/mdi/chevron-down';
import {
  createMemo,
  createSignal,
  For,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  useCheckPluginUpdates,
  useUpdatesData,
  useUpdatePlugin,
  type Plugin,
} from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import {
  Button,
  CombinedTextField,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/shared/ui';

export type PluginVersionDropdownProps = ComponentProps<'div'> & {
  plugin: Plugin;
  disabled?: boolean;
};

export const PluginVersionDropdown: Component<PluginVersionDropdownProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [local] = splitProps(props, ['plugin', 'class', 'disabled']);

  const pluginId = () => local.plugin.manifest.metadata.id;
  const updates = useUpdatesData(pluginId);
  const checkUpdates = useCheckPluginUpdates();
  const updatePlugin = useUpdatePlugin();

  const [searchQuery, setSearchQuery] = createSignal('');

  // Show dropdown for any plugin that has cached updates
  const hasChecked = createMemo(() => updates.data != null);

  const isDisabled = createMemo(
    () => local.disabled || updatePlugin.isPending || checkUpdates.isPending,
  );

  const filteredVersions = createMemo(() => {
    const all = updates.data?.all_releases ?? [];
    const query = searchQuery().toLowerCase();
    if (!query) return all;
    return all.filter((r) => r.tag_name.toLowerCase().includes(query));
  });

  const handleCheckUpdates = () => {
    checkUpdates.mutate(pluginId());
  };

  const handleSwitchVersion = (tag: string) => {
    updatePlugin.mutate({ id: pluginId(), targetTag: tag });
  };

  return (
    <Show when={hasChecked() || checkUpdates.isPending}>
      <DropdownMenu>
        <DropdownMenuTrigger
          as={Button<'button'>}
          variant='destructive'
          size='sm'
          class='aspect-square rounded-none border-0 p-0 text-base'
          aria-label={t('plugins.switchVersion')}
          disabled={isDisabled()}
        >
          <IconMdiChevronDown />
        </DropdownMenuTrigger>
        <DropdownMenuContent class='min-w-52 rounded-lg'>
          <Show
            when={hasChecked()}
            fallback={
              <div class='p-2'>
                <Button
                  variant='default'
                  size='sm'
                  class='w-full'
                  onClick={handleCheckUpdates}
                  disabled={checkUpdates.isPending}
                  loading={checkUpdates.isPending}
                >
                  {t('plugins.checkUpdates')}
                </Button>
              </div>
            }
          >
            <Show when={updates.data && updates.data.all_releases.length > 0}>
              <CombinedTextField
                class='p-1'
                value={searchQuery()}
                onChange={setSearchQuery}
                inputProps={{
                  placeholder: t('plugins.searchVersion'),
                }}
              />
              <DropdownMenuSeparator />
              <div class='max-h-60 overflow-y-auto'>
                <For each={filteredVersions()}>
                  {(release) => (
                    <DropdownMenuItem
                      as={Button}
                      variant='ghost'
                      class='w-full justify-start'
                      onClick={() => handleSwitchVersion(release.tag_name)}
                      disabled={
                        isDisabled() ||
                        release.tag_name === updates.data?.current_tag
                      }
                    >
                      <span>{release.tag_name}</span>
                      <Show
                        when={release.tag_name === updates.data?.current_tag}
                      >
                        <span class='ml-auto text-xs text-muted-foreground'>
                          {t('common.current')}
                        </span>
                      </Show>
                    </DropdownMenuItem>
                  )}
                </For>
                <Show when={filteredVersions().length === 0}>
                  <div class='px-2 py-3 text-center text-sm text-muted-foreground'>
                    {t('plugins.noVersionsFound')}
                  </div>
                </Show>
              </div>
            </Show>
          </Show>
        </DropdownMenuContent>
      </DropdownMenu>
    </Show>
  );
};
