import type { ContextMenuRootProps } from '@kobalte/core/context-menu';
import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui';

export type InstanceContextMenuProps = {
  disableOpenFolder?: boolean;
  isLoading?: boolean;
  onOpenFolder?: ComponentProps<'button'>['onClick'];
  onOpenSettings?: ComponentProps<'button'>['onClick'];
  onPlay?: ComponentProps<'button'>['onClick'];
  onRemove?: ComponentProps<'button'>['onClick'];
} & ContextMenuRootProps;

export const InstanceContextMenu: Component<InstanceContextMenuProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'onPlay',
    'onOpenFolder',
    'onOpenSettings',
    'onRemove',
    'isLoading',
    'disableOpenFolder',
    'children',
  ]);

  const [{ t }] = useTranslation();

  return (
    <ContextMenu {...others}>
      {local.children}
      <ContextMenuContent>
        <ContextMenuItem
          class='w-full hover:!bg-success hover:text-success-foreground'
          disabled={local.isLoading}
          onClick={local.onPlay}
        >
          {t('instance.launch')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full'
          disabled={local.isLoading || local.disableOpenFolder}
          onClick={local.onOpenFolder}
        >
          {t('instance.openFolder')}
        </ContextMenuItem>
        <ContextMenuItem
          class='w-full'
          disabled={local.isLoading}
          onClick={local.onOpenSettings}
        >
          {t('instance.settings')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full hover:!bg-destructive hover:text-destructive-foreground'
          disabled={local.isLoading}
          onClick={local.onRemove}
        >
          {t('common.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
