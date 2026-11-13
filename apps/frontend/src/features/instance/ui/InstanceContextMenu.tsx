import type { ContextMenuRootProps } from '@kobalte/core/context-menu';
import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from '@/shared/ui-kit';

export type InstanceContextMenuProps = ContextMenuRootProps & {
  onPlay?: () => void;
  onOpenFolder?: () => void;
  onOpenSettings?: () => void;
  onRemove?: () => void;
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
          class='
            hover:bg-success! hover:text-success-foreground
            w-full
          '
          onSelect={local.onPlay}
          disabled={local.isLoading}
        >
          {t('instance.launch')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='w-full'
          onSelect={local.onOpenFolder}
          disabled={local.isLoading || local.disableOpenFolder}
        >
          {t('instance.openFolder')}
        </ContextMenuItem>
        <ContextMenuItem
          class='w-full'
          onSelect={local.onOpenSettings}
          disabled={local.isLoading}
        >
          {t('instance.settings')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          class='
            hover:bg-destructive! hover:text-destructive-foreground
            w-full
          '
          onSelect={local.onRemove}
          disabled={local.isLoading}
        >
          {t('common.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
