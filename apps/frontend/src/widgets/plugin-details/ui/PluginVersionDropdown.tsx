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
  useCheckForUpdates,
  usePluginSource,
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
  Skeleton,
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
  const source = usePluginSource(pluginId);
  const updates = useCheckForUpdates(pluginId);
  const updatePlugin = useUpdatePlugin();

  const [searchQuery, setSearchQuery] = createSignal('');

  // Show dropdown for any plugin that has a stored source (GitHub, etc.)
  // Local plugins have no source → no updates, so no dropdown
  const hasSource = createMemo(() => source.data != null);

  const filteredVersions = createMemo(() => {
    const all = updates.data?.all_releases ?? [];
    const query = searchQuery().toLowerCase();
    if (!query) return all;
    return all.filter((r) => r.tag_name.toLowerCase().includes(query));
  });

  const handleSwitchVersion = (tag: string) => {
    updatePlugin.mutate({ id: pluginId(), targetTag: tag });
  };

  return (
    <Show when={hasSource()}>
      <DropdownMenu>
        <DropdownMenuTrigger
          as={Button<'button'>}
          variant='destructive'
          size='sm'
          class='aspect-square rounded-none border-0 p-0 text-base'
          aria-label={t('plugins.switchVersion')}
          disabled={local.disabled}
        >
          <IconMdiChevronDown />
        </DropdownMenuTrigger>
        <DropdownMenuContent class='min-w-52 rounded-lg'>
          <Show when={source.isLoading || updates.isLoading}>
            <div class='flex flex-col gap-0.5'>
              <Skeleton height={32} class='w-full rounded-md' />
              <Skeleton height={32} class='w-full rounded-md' />
              <Skeleton height={32} class='w-full rounded-md' />
              <Skeleton height={32} class='w-full rounded-md' />
            </div>
          </Show>

          {/* Search input */}
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
                    disabled={release.tag_name === updates.data?.current_tag}
                  >
                    <span>{release.tag_name}</span>
                    <Show when={release.tag_name === updates.data?.current_tag}>
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
        </DropdownMenuContent>
      </DropdownMenu>
    </Show>
  );
};
