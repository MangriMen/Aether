import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui';

import type { InstanceContextMenuProps } from './types';
import { useTranslate } from '@/shared/model';

export const InstanceContextMenu: Component<InstanceContextMenuProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'onPlay',
    'onOpenFolder',
    'onOpenSettings',
    'onRemove',
    'isLoading',
    'children',
  ]);

  const [{ t }] = useTranslate();

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
          disabled={local.isLoading}
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
