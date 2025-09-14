import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui';
import type { ContextMenuRootProps } from '@kobalte/core/context-menu';

import { useTranslation } from '@/shared/model';

export type InstanceContextMenuProps = ContextMenuRootProps & {
  onPlay?: ComponentProps<'button'>['onClick'];
  onOpenFolder?: ComponentProps<'button'>['onClick'];
  onOpenSettings?: ComponentProps<'button'>['onClick'];
  onRemove?: ComponentProps<'button'>['onClick'];
  isLoading?: boolean;
  disableOpenFolder?: boolean;
};

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
          onClick={local.onPlay}
          disabled={local.isLoading}
        >
          {t('instance.launch')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full'
          onClick={local.onOpenFolder}
          disabled={local.isLoading || local.disableOpenFolder}
        >
          {t('instance.openFolder')}
        </ContextMenuItem>
        <ContextMenuItem
          class='w-full'
          onClick={local.onOpenSettings}
          disabled={local.isLoading}
        >
          {t('instance.settings')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full hover:!bg-destructive hover:text-destructive-foreground'
          onClick={local.onRemove}
          disabled={local.isLoading}
        >
          {t('common.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
