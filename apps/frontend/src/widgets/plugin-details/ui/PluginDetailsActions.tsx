import IconMdiDelete from '~icons/mdi/delete';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  PluginUpdateButton,
  useRemovePlugin,
  useUpdatePlugin,
  type Plugin,
} from '@/entities/plugins';
import { TogglePluginButton } from '@/features/toggle-plugin-button';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Separator } from '@/shared/ui';

import { PluginVersionDropdown } from './PluginVersionDropdown';

export type PluginDetailsActionsProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsActions: Component<PluginDetailsActionsProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);
  const [{ t }] = useTranslation();

  const removePlugin = useRemovePlugin();
  const updatePlugin = useUpdatePlugin();

  const isMutating = createMemo(() => updatePlugin.isPending);

  const handleRemove = () => {
    removePlugin.mutate(local.plugin.manifest.metadata.id);
  };

  return (
    <div class={cn('flex items-center gap-2', local.class)} {...others}>
      <PluginUpdateButton plugin={local.plugin} />
      <TogglePluginButton plugin={local.plugin} disabled={isMutating()} />

      <div class='flex overflow-hidden rounded-md border'>
        <Button
          variant='destructive'
          size='sm'
          class='rounded-none border-0'
          leadingIcon={IconMdiDelete}
          onClick={handleRemove}
          disabled={isMutating()}
        >
          {t('common.remove')}
        </Button>
        <Separator orientation='vertical' />
        <PluginVersionDropdown plugin={local.plugin} disabled={isMutating()} />
      </div>
    </div>
  );
};
